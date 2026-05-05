use super::{derive_agent_attribution_toggle_state, AgentAttributionToggleState};
use crate::workspaces::workspace::AdminEnablementSetting;

#[test]
fn respect_user_setting_returns_user_pref_unlocked() {
    let state = derive_agent_attribution_toggle_state(
        &AdminEnablementSetting::RespectUserSetting,
        true,
        true,
    );
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: true,
            is_forced_by_org: false,
            is_disabled: false,
        }
    );
}

#[test]
fn respect_user_setting_with_user_off_returns_unchecked_unlocked() {
    let state = derive_agent_attribution_toggle_state(
        &AdminEnablementSetting::RespectUserSetting,
        false,
        true,
    );
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: false,
            is_forced_by_org: false,
            is_disabled: false,
        }
    );
}

#[test]
fn team_enable_locks_toggle_on_regardless_of_user_pref() {
    let state = derive_agent_attribution_toggle_state(&AdminEnablementSetting::Enable, false, true);
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: true,
            is_forced_by_org: true,
            is_disabled: true,
        }
    );
}

#[test]
fn team_disable_locks_toggle_off_regardless_of_user_pref() {
    let state = derive_agent_attribution_toggle_state(&AdminEnablementSetting::Disable, true, true);
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: false,
            is_forced_by_org: true,
            is_disabled: true,
        }
    );
}

#[test]
fn ai_globally_disabled_marks_toggle_disabled_but_not_forced() {
    let state = derive_agent_attribution_toggle_state(
        &AdminEnablementSetting::RespectUserSetting,
        true,
        false,
    );
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: true,
            is_forced_by_org: false,
            is_disabled: true,
        }
    );
}

#[test]
fn team_force_takes_precedence_over_global_ai_disabled() {
    let state =
        derive_agent_attribution_toggle_state(&AdminEnablementSetting::Enable, false, false);
    assert_eq!(
        state,
        AgentAttributionToggleState {
            is_enabled: true,
            is_forced_by_org: true,
            is_disabled: true,
        }
    );
}

// -- CliAgentPluginStatus tests --------------------------------------------

use super::CliAgentPluginStatus;
use crate::terminal::cli_agent_sessions::plugin_manager::PluginModalKind;
use crate::terminal::CLIAgent;

#[test]
fn plugin_status_label_returns_user_facing_strings() {
    assert_eq!(CliAgentPluginStatus::Installed.label(), "Installed");
    assert_eq!(
        CliAgentPluginStatus::UpdateAvailable.label(),
        "Update available"
    );
    assert_eq!(CliAgentPluginStatus::NotInstalled.label(), "Not installed");
}

#[test]
fn plugin_status_modal_kind_maps_to_install_or_update_only() {
    assert_eq!(CliAgentPluginStatus::Installed.modal_kind(), None);
    assert_eq!(
        CliAgentPluginStatus::UpdateAvailable.modal_kind(),
        Some(PluginModalKind::Update)
    );
    assert_eq!(
        CliAgentPluginStatus::NotInstalled.modal_kind(),
        Some(PluginModalKind::Install)
    );
}

#[test]
fn plugin_status_button_label_only_shown_when_action_exists() {
    // Installed has nothing to do, so no button.
    assert_eq!(CliAgentPluginStatus::Installed.button_label(), None);
    assert_eq!(
        CliAgentPluginStatus::UpdateAvailable.button_label(),
        Some("Update")
    );
    assert_eq!(
        CliAgentPluginStatus::NotInstalled.button_label(),
        Some("Install")
    );
}

#[test]
fn plugin_status_modal_kind_and_button_label_have_consistent_action_set() {
    // Whenever a button is shown, a modal kind must exist; and vice-versa.
    for status in [
        CliAgentPluginStatus::Installed,
        CliAgentPluginStatus::UpdateAvailable,
        CliAgentPluginStatus::NotInstalled,
    ] {
        assert_eq!(status.button_label().is_some(), status.modal_kind().is_some());
    }
}

#[test]
fn plugin_status_compute_returns_none_for_agents_without_plugin_support() {
    // Agents in the catch-all None arm of `plugin_manager_for` must produce
    // None from compute(), regardless of feature flag state.
    assert!(CliAgentPluginStatus::compute(CLIAgent::Amp).is_none());
    assert!(CliAgentPluginStatus::compute(CLIAgent::Unknown).is_none());
}
