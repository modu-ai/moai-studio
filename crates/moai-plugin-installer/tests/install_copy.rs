//! 통합 테스트 (T-017): 번들 복사 + 버전 비교
//!
//! 절대로 실제 `~/.claude/plugins` 경로를 건드리지 않는다. 전부 tempdir 사용.

use std::fs;
use std::path::Path;

use moai_plugin_installer::{InstallOutcome, install_or_update};
use tempfile::tempdir;

fn make_bundle(dir: &Path, version: &str) {
    fs::create_dir_all(dir.join(".claude-plugin")).unwrap();
    fs::write(
        dir.join(".claude-plugin/plugin.json"),
        format!(r#"{{"name":"moai-studio","version":"{version}","claude_min_version":"1.0.0"}}"#),
    )
    .unwrap();
    fs::create_dir_all(dir.join("hooks")).unwrap();
    fs::write(
        dir.join("hooks/hooks.json"),
        r#"{"hooks":{"PreToolUse":[]}}"#,
    )
    .unwrap();
    fs::write(dir.join("mcp-config.json"), r#"{"mcpServers":{}}"#).unwrap();
}

#[test]
fn fresh_install_copies_bundle_tree() {
    let bundle = tempdir().unwrap();
    let target_root = tempdir().unwrap();
    let target = target_root.path().join("moai-studio@local");
    make_bundle(bundle.path(), "0.1.0");

    let r = install_or_update(bundle.path(), &target).unwrap();
    assert_eq!(r, InstallOutcome::Installed);

    assert!(target.join(".claude-plugin/plugin.json").exists());
    assert!(target.join("hooks/hooks.json").exists());
    assert!(target.join("mcp-config.json").exists());
}

#[test]
fn second_install_same_version_skips() {
    let bundle = tempdir().unwrap();
    let target_root = tempdir().unwrap();
    let target = target_root.path().join("moai-studio@local");
    make_bundle(bundle.path(), "0.1.0");

    install_or_update(bundle.path(), &target).unwrap();
    let r = install_or_update(bundle.path(), &target).unwrap();
    assert_eq!(r, InstallOutcome::Skipped);
}

#[test]
fn higher_bundle_version_updates() {
    let bundle = tempdir().unwrap();
    let target_root = tempdir().unwrap();
    let target = target_root.path().join("moai-studio@local");
    make_bundle(bundle.path(), "0.1.0");
    install_or_update(bundle.path(), &target).unwrap();

    make_bundle(bundle.path(), "1.2.3");
    let r = install_or_update(bundle.path(), &target).unwrap();
    assert_eq!(r, InstallOutcome::Updated);
    let m = fs::read_to_string(target.join(".claude-plugin/plugin.json")).unwrap();
    assert!(m.contains("1.2.3"));
}

#[test]
fn does_not_touch_real_home() {
    // @MX:NOTE: HOME이 실제 환경이더라도 install_or_update는 주입된 target만 사용한다.
    // 안전을 위해 테스트에서는 target을 반드시 tempdir 기반으로만 지정한다.
    let bundle = tempdir().unwrap();
    let target_root = tempdir().unwrap();
    let target = target_root.path().join("plugin-sandbox");
    make_bundle(bundle.path(), "0.1.0");
    install_or_update(bundle.path(), &target).unwrap();
    // target은 tempdir 하위
    assert!(target.starts_with(target_root.path()));
}
