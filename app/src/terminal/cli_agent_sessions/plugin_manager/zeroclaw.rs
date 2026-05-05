use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use async_trait::async_trait;

use super::{
    compare_versions, run_cli_command_logged, CliAgentPluginManager, PluginInstallError,
    PluginInstructionStep, PluginInstructions,
};
use crate::terminal::model::session::LocalCommandExecutor;
use crate::terminal::shell::ShellType;

const ZEROCLAW_BINARY: &str = "zeroclaw";
const CHANNEL_NAME: &str = "warp";

// Keep in sync with the channel adapter version in warpdotdev/zeroclaw-warp.
const MINIMUM_PLUGIN_VERSION: &str = "0.1.0";

pub(super) struct ZeroclawPluginManager {
    executor: LocalCommandExecutor,
    path_env_var: Option<String>,
}

impl ZeroclawPluginManager {
    pub(super) fn new(
        shell_path: Option<PathBuf>,
        shell_type: Option<ShellType>,
        path_env_var: Option<String>,
    ) -> Self {
        let shell_type = shell_type.unwrap_or(ShellType::Bash);
        Self {
            executor: LocalCommandExecutor::new(shell_path, shell_type),
            path_env_var,
        }
    }

    async fn run_logged(&self, args: &[&str], log: &mut String) -> Result<(), PluginInstallError> {
        let env_vars = self
            .path_env_var
            .as_deref()
            .map(|path| HashMap::from([("PATH".to_owned(), path.to_owned())]));
        run_cli_command_logged(ZEROCLAW_BINARY, args, &self.executor, env_vars, log).await
    }
}

#[async_trait]
impl CliAgentPluginManager for ZeroclawPluginManager {
    fn minimum_plugin_version(&self) -> &'static str {
        MINIMUM_PLUGIN_VERSION
    }

    fn can_auto_install(&self) -> bool {
        true
    }

    fn is_installed(&self) -> bool {
        let Ok(dir) = zeroclaw_home_dir() else {
            return false;
        };
        check_installed(&dir)
    }

    fn needs_update(&self) -> bool {
        let Ok(dir) = zeroclaw_home_dir() else {
            return false;
        };
        match installed_version(&dir) {
            Some(v) => compare_versions(&v, MINIMUM_PLUGIN_VERSION).is_lt(),
            // If the channel is enabled but `manifest.toml` is missing/unreadable,
            // conservatively prompt for an update so the user converges on a build
            // that meets MINIMUM_PLUGIN_VERSION.
            None => check_installed(&dir),
        }
    }

    async fn install(&self) -> Result<(), PluginInstallError> {
        let mut log = String::new();
        // Enable the warp channel adapter via the zeroclaw onboard subcommand.
        // Zeroclaw uses ACP (JSON-RPC over stdio) for IPC; the warp channel
        // adapter listens on the same transport and forwards events.
        self.run_logged(&["channels", "enable", CHANNEL_NAME], &mut log)
            .await?;
        Ok(())
    }

    async fn update(&self) -> Result<(), PluginInstallError> {
        let mut log = String::new();
        self.run_logged(&["channels", "update", CHANNEL_NAME], &mut log)
            .await?;

        let still_outdated = zeroclaw_home_dir()
            .ok()
            .and_then(|dir| installed_version(&dir))
            .map(|v| compare_versions(&v, MINIMUM_PLUGIN_VERSION).is_lt())
            .unwrap_or(true);
        if still_outdated {
            log.push_str("Post-update version check: channel is still outdated\n");
            return Err(PluginInstallError {
                message: "Channel update did not take effect".to_owned(),
                log,
            });
        }
        Ok(())
    }

    fn install_success_message(&self) -> &'static str {
        "Warp channel enabled. Please restart zeroclaw to activate."
    }

    fn update_success_message(&self) -> &'static str {
        "Warp channel updated. Please restart zeroclaw to activate."
    }

    fn install_instructions(&self) -> &'static PluginInstructions {
        &INSTALL_INSTRUCTIONS
    }

    fn update_instructions(&self) -> &'static PluginInstructions {
        &UPDATE_INSTRUCTIONS
    }
}

static INSTALL_INSTRUCTIONS: LazyLock<PluginInstructions> = LazyLock::new(|| PluginInstructions {
    title: "Enable Warp Channel for ZeroClaw",
    subtitle: "ZeroClaw uses an ACP (Agent Client Protocol) channel system. Enable the warp channel adapter, then restart zeroclaw.",
    steps: &[
        PluginInstructionStep {
            description: "Enable the warp channel adapter",
            command: "zeroclaw channels enable warp",
            executable: true,
            link: None,
        },
        PluginInstructionStep {
            description: "Optional: edit ZeroClaw config to set notification preferences",
            command: "~/.zeroclaw/config.toml",
            executable: false,
            link: None,
        },
    ],
    post_install_notes: &[
        "Restart zeroclaw to load the channel.",
        "If `zeroclaw channels` is unavailable, add `[channels.warp] enabled = true` to ~/.zeroclaw/config.toml manually.",
    ],
});

static UPDATE_INSTRUCTIONS: LazyLock<PluginInstructions> = LazyLock::new(|| PluginInstructions {
    title: "Update Warp Channel for ZeroClaw",
    subtitle: "Run the following command, then restart zeroclaw.",
    steps: &[PluginInstructionStep {
        description: "Update the warp channel",
        command: "zeroclaw channels update warp",
        executable: true,
        link: None,
    }],
    post_install_notes: &["Restart zeroclaw to activate the update."],
});

fn check_installed(dir: &Path) -> bool {
    let config_path = dir.join("config.toml");
    let Ok(contents) = fs::read_to_string(config_path) else {
        return false;
    };
    let Ok(parsed) = contents.parse::<toml::Value>() else {
        return false;
    };
    parsed
        .get("channels")
        .and_then(|c| c.get(CHANNEL_NAME))
        .and_then(|c| c.get("enabled"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

fn installed_version(dir: &Path) -> Option<String> {
    let manifest_path = dir
        .join("channels")
        .join(CHANNEL_NAME)
        .join("manifest.toml");
    let contents = fs::read_to_string(manifest_path).ok()?;
    let parsed = contents.parse::<toml::Value>().ok()?;
    parsed.get("version")?.as_str().map(|s| s.to_owned())
}

/// Returns `~/.zeroclaw` (or `ZEROCLAW_HOME` override).
fn zeroclaw_home_dir() -> io::Result<PathBuf> {
    if let Ok(custom) = std::env::var("ZEROCLAW_HOME") {
        return Ok(PathBuf::from(custom));
    }
    dirs::home_dir()
        .map(|home| home.join(".zeroclaw"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "could not determine home directory",
            )
        })
}

#[cfg(test)]
#[path = "zeroclaw_tests.rs"]
mod tests;
