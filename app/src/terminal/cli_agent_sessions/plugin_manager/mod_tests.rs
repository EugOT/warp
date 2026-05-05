use std::cmp::Ordering;

use super::{compare_versions, plugin_manager_for};
use crate::terminal::CLIAgent;

#[test]
fn returns_manager_for_claude() {
    assert!(plugin_manager_for(CLIAgent::Claude).is_some());
}

#[test]
fn returns_manager_for_opencode() {
    let _oc_guard = crate::features::FeatureFlag::OpenCodeNotifications.override_enabled(true);
    let _hoa_guard = crate::features::FeatureFlag::HOANotifications.override_enabled(true);
    assert!(plugin_manager_for(CLIAgent::OpenCode).is_some());
}

#[test]
fn returns_manager_for_codex() {
    let _codex_guard = crate::features::FeatureFlag::CodexNotifications.override_enabled(true);
    let _hoa_guard = crate::features::FeatureFlag::HOANotifications.override_enabled(true);
    assert!(plugin_manager_for(CLIAgent::Codex).is_some());
}

#[test]
fn returns_manager_for_gemini() {
    let _gemini_guard = crate::features::FeatureFlag::GeminiNotifications.override_enabled(true);
    let _hoa_guard = crate::features::FeatureFlag::HOANotifications.override_enabled(true);
    assert!(plugin_manager_for(CLIAgent::Gemini).is_some());
}

#[test]
fn returns_manager_for_walcode() {
    let _walcode_guard =
        crate::features::FeatureFlag::WalcodeNotifications.override_enabled(true);
    let _hoa_guard = crate::features::FeatureFlag::HOANotifications.override_enabled(true);
    assert!(plugin_manager_for(CLIAgent::Walcode).is_some());
}

#[test]
fn returns_manager_for_zeroclaw() {
    let _zeroclaw_guard =
        crate::features::FeatureFlag::ZeroclawNotifications.override_enabled(true);
    let _hoa_guard = crate::features::FeatureFlag::HOANotifications.override_enabled(true);
    assert!(plugin_manager_for(CLIAgent::Zeroclaw).is_some());
}

#[test]
fn walcode_returns_none_when_hoa_disabled() {
    let _walcode_guard =
        crate::features::FeatureFlag::WalcodeNotifications.override_enabled(true);
    let _hoa_guard = crate::features::FeatureFlag::HOANotifications.override_enabled(false);
    assert!(plugin_manager_for(CLIAgent::Walcode).is_none());
}

#[test]
fn walcode_returns_none_when_agent_flag_disabled() {
    let _walcode_guard =
        crate::features::FeatureFlag::WalcodeNotifications.override_enabled(false);
    let _hoa_guard = crate::features::FeatureFlag::HOANotifications.override_enabled(true);
    assert!(plugin_manager_for(CLIAgent::Walcode).is_none());
}

#[test]
fn zeroclaw_returns_none_when_hoa_disabled() {
    let _zeroclaw_guard =
        crate::features::FeatureFlag::ZeroclawNotifications.override_enabled(true);
    let _hoa_guard = crate::features::FeatureFlag::HOANotifications.override_enabled(false);
    assert!(plugin_manager_for(CLIAgent::Zeroclaw).is_none());
}

#[test]
fn zeroclaw_returns_none_when_agent_flag_disabled() {
    let _zeroclaw_guard =
        crate::features::FeatureFlag::ZeroclawNotifications.override_enabled(false);
    let _hoa_guard = crate::features::FeatureFlag::HOANotifications.override_enabled(true);
    assert!(plugin_manager_for(CLIAgent::Zeroclaw).is_none());
}

#[test]
fn returns_none_for_unsupported_agents() {
    assert!(plugin_manager_for(CLIAgent::Amp).is_none());
    assert!(plugin_manager_for(CLIAgent::Droid).is_none());
    assert!(plugin_manager_for(CLIAgent::Copilot).is_none());
    assert!(plugin_manager_for(CLIAgent::Auggie).is_none());
    assert!(plugin_manager_for(CLIAgent::CursorCli).is_none());
    assert!(plugin_manager_for(CLIAgent::Pi).is_none());
    assert!(plugin_manager_for(CLIAgent::Goose).is_none());
    assert!(plugin_manager_for(CLIAgent::Vibe).is_none());
    assert!(plugin_manager_for(CLIAgent::Unknown).is_none());
}

#[test]
fn compare_versions_equal() {
    assert_eq!(compare_versions("1.2.3", "1.2.3"), Ordering::Equal);
}

#[test]
fn compare_versions_less_than_major() {
    assert_eq!(compare_versions("1.0.0", "2.0.0"), Ordering::Less);
}

#[test]
fn compare_versions_less_than_minor() {
    assert_eq!(compare_versions("1.1.0", "1.2.0"), Ordering::Less);
}

#[test]
fn compare_versions_less_than_patch() {
    assert_eq!(compare_versions("1.1.0", "1.1.1"), Ordering::Less);
}

#[test]
fn compare_versions_greater_than() {
    assert_eq!(compare_versions("3.0.0", "2.0.0"), Ordering::Greater);
}

#[test]
fn compare_versions_unparseable_treated_as_zero() {
    assert_eq!(compare_versions("abc", "0.0.0"), Ordering::Equal);
    assert_eq!(compare_versions("abc", "1.0.0"), Ordering::Less);
}

#[test]
fn compare_versions_partial_version_string() {
    assert_eq!(compare_versions("2", "2.0.0"), Ordering::Equal);
    assert_eq!(compare_versions("2.1", "2.1.0"), Ordering::Equal);
}

#[test]
fn compare_versions_empty_string() {
    assert_eq!(compare_versions("", "2.0.0"), Ordering::Less);
}
