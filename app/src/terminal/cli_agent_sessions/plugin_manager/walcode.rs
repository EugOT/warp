use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use async_trait::async_trait;
use serde_json::Value;

use super::{
    compare_versions, run_cli_command_logged, CliAgentPluginManager, PluginInstallError,
    PluginInstructionStep, PluginInstructions,
};
use crate::terminal::model::session::LocalCommandExecutor;
use crate::terminal::shell::ShellType;

/// Walcode binary is `claw` (from the `rusty-claude-cli` crate).
const CLAW_BINARY: &str = "claw";
const PLUGIN_PACKAGE: &str = "@warp-dot-dev/walcode-warp";
const PLUGIN_NAME: &str = "walcode-warp";

// Keep in sync with the plugin version in warpdotdev/walcode-warp.
const MINIMUM_PLUGIN_VERSION: &str = "0.1.0";

pub(super) struct WalcodePluginManager {
    executor: LocalCommandExecutor,
    path_env_var: Option<String>,
}

impl WalcodePluginManager {
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
        run_cli_command_logged(CLAW_BINARY, args, &self.executor, env_vars, log).await
    }
}

#[async_trait]
impl CliAgentPluginManager for WalcodePluginManager {
    fn minimum_plugin_version(&self) -> &'static str {
        MINIMUM_PLUGIN_VERSION
    }

    fn can_auto_install(&self) -> bool {
        true
    }

    fn is_installed(&self) -> bool {
        let Ok(claw_dir) = walcode_home_dir() else {
            return false;
        };
        check_installed(&claw_dir)
    }

    fn needs_update(&self) -> bool {
        let Ok(claw_dir) = walcode_home_dir() else {
            return false;
        };
        match installed_version(&claw_dir) {
            Some(v) => compare_versions(&v, MINIMUM_PLUGIN_VERSION).is_lt(),
            // If `plugin.json` exists but the `version` field is missing/unreadable,
            // conservatively prompt for an update so the user converges on a build
            // that meets MINIMUM_PLUGIN_VERSION.
            None => plugin_manifest_exists(&claw_dir),
        }
    }

    async fn install(&self) -> Result<(), PluginInstallError> {
        let mut log = String::new();
        self.run_logged(&["plugins", "install", PLUGIN_PACKAGE], &mut log)
            .await?;
        Ok(())
    }

    async fn update(&self) -> Result<(), PluginInstallError> {
        let mut log = String::new();
        self.run_logged(&["plugins", "update", PLUGIN_NAME], &mut log)
            .await?;

        let still_outdated = walcode_home_dir()
            .ok()
            .and_then(|dir| installed_version(&dir))
            .map(|v| compare_versions(&v, MINIMUM_PLUGIN_VERSION).is_lt())
            .unwrap_or(true);
        if still_outdated {
            log.push_str("Post-update version check: plugin is still outdated\n");
            return Err(PluginInstallError {
                message: "Plugin update did not take effect".to_owned(),
                log,
            });
        }
        Ok(())
    }

    fn install_success_message(&self) -> &'static str {
        "Warp plugin installed. Please restart `claw` to activate."
    }

    fn update_success_message(&self) -> &'static str {
        "Warp plugin updated. Please restart `claw` to activate."
    }

    fn install_instructions(&self) -> &'static PluginInstructions {
        &INSTALL_INSTRUCTIONS
    }

    fn update_instructions(&self) -> &'static PluginInstructions {
        &UPDATE_INSTRUCTIONS
    }
}

static INSTALL_INSTRUCTIONS: LazyLock<PluginInstructions> = LazyLock::new(|| PluginInstructions {
    title: "Install Warp Plugin for Walcode (claw)",
    subtitle: "Walcode is invoked as `claw`. Install the Warp plugin into ~/.claw/plugins/, then restart claw.",
    steps: &[
        PluginInstructionStep {
            description: "Install the Warp plugin",
            command: "claw plugins install @warp-dot-dev/walcode-warp",
            executable: true,
            link: None,
        },
        PluginInstructionStep {
            description: "Optional: enable Warp notifications block in your settings file",
            command: "~/.claw/settings.json",
            executable: false,
            link: None,
        },
    ],
    post_install_notes: &[
        "Restart claw to activate the plugin.",
        "If `claw plugins install` is unavailable in your build, drop the plugin into ~/.claw/plugins/walcode-warp/.",
    ],
});

static UPDATE_INSTRUCTIONS: LazyLock<PluginInstructions> = LazyLock::new(|| PluginInstructions {
    title: "Update Warp Plugin for Walcode (claw)",
    subtitle: "Run the following command, then restart claw.",
    steps: &[PluginInstructionStep {
        description: "Update the Warp plugin",
        command: "claw plugins update walcode-warp",
        executable: true,
        link: None,
    }],
    post_install_notes: &["Restart claw to activate the update."],
});

fn manifest_path(claw_dir: &Path) -> PathBuf {
    claw_dir
        .join("plugins")
        .join(PLUGIN_NAME)
        .join("plugin.json")
}

fn plugin_manifest_exists(claw_dir: &Path) -> bool {
    manifest_path(claw_dir).exists()
}

fn read_manifest(claw_dir: &Path) -> Option<Value> {
    let contents = fs::read_to_string(manifest_path(claw_dir)).ok()?;
    serde_json::from_str::<Value>(&contents).ok()
}

fn check_installed(claw_dir: &Path) -> bool {
    let Some(parsed) = read_manifest(claw_dir) else {
        return false;
    };
    parsed.get("name").and_then(Value::as_str).is_some()
        && parsed.get("version").and_then(Value::as_str).is_some()
}

fn installed_version(claw_dir: &Path) -> Option<String> {
    read_manifest(claw_dir)?
        .get("version")?
        .as_str()
        .map(|s| s.to_owned())
}

/// Returns `~/.claw` (or `CLAW_HOME` override).
fn walcode_home_dir() -> io::Result<PathBuf> {
    if let Ok(custom) = std::env::var("CLAW_HOME") {
        return Ok(PathBuf::from(custom));
    }
    dirs::home_dir()
        .map(|home| home.join(".claw"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "could not determine home directory",
            )
        })
}

#[cfg(test)]
#[path = "walcode_tests.rs"]
mod tests;
