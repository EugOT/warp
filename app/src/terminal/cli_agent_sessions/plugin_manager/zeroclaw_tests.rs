use super::ZeroclawPluginManager;
use crate::terminal::cli_agent_sessions::plugin_manager::CliAgentPluginManager;

fn manager() -> ZeroclawPluginManager {
    ZeroclawPluginManager::new(None, None, None)
}

#[test]
fn can_auto_install_is_true() {
    assert!(manager().can_auto_install());
}

#[test]
fn install_instructions_references_zeroclaw_binary() {
    let instructions = manager().install_instructions();
    assert!(!instructions.steps.is_empty());
    assert!(instructions
        .steps
        .iter()
        .any(|s| s.command.contains("zeroclaw") || s.command.contains(".zeroclaw")));
}

#[test]
fn update_instructions_has_steps() {
    let instructions = manager().update_instructions();
    assert!(!instructions.steps.is_empty());
}

#[test]
fn minimum_version_is_semver() {
    let v = manager().minimum_plugin_version();
    let parts: Vec<&str> = v.split('.').collect();
    assert_eq!(parts.len(), 3, "expected X.Y.Z, got {v}");
    for part in parts {
        assert!(part.parse::<u32>().is_ok(), "non-numeric semver part in {v}");
    }
}
