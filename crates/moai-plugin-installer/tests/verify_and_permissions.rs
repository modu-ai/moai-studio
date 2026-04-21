//! 통합 테스트 (T-018): 무결성 검증 + 쓰기 권한 오류 처리

use std::fs;
use std::path::Path;

use moai_plugin_installer::{BundleError, VerifyError, install_or_update, verify_plugin};
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
fn install_then_verify_succeeds() {
    let bundle = tempdir().unwrap();
    let target_root = tempdir().unwrap();
    let target = target_root.path().join("moai-studio@local");
    make_bundle(bundle.path(), "0.1.0");
    install_or_update(bundle.path(), &target).unwrap();
    verify_plugin(&target).expect("verify must succeed after install");
}

#[test]
fn verify_rejects_e5_array_format() {
    let bundle = tempdir().unwrap();
    let target_root = tempdir().unwrap();
    let target = target_root.path().join("moai-studio@local");
    make_bundle(bundle.path(), "0.1.0");
    install_or_update(bundle.path(), &target).unwrap();
    // 손상: hooks를 배열로 만듦
    fs::write(target.join("hooks/hooks.json"), r#"{"hooks":[]}"#).unwrap();
    let err = verify_plugin(&target).unwrap_err();
    assert!(matches!(err, VerifyError::InvalidHooksWrapper));
}

#[cfg(unix)]
#[test]
fn permission_denied_returns_specific_variant() {
    // @MX:NOTE: Unix 전용 — chmod 0o555로 읽기 전용 디렉토리를 만들어
    // install_or_update가 PermissionDenied variant를 반환하는지 확인한다.
    use std::os::unix::fs::PermissionsExt;

    // root 사용자는 권한 검사를 우회하므로 스킵 (CI runner 대부분은 non-root이지만 안전장치)
    // SAFETY: libc 호출로 uid 확인
    // SAFETY justification: getuid is a simple read-only syscall with no side effects.
    let is_root = unsafe { libc_getuid() } == 0;
    if is_root {
        eprintln!("skipping perm test under root");
        return;
    }

    let bundle = tempdir().unwrap();
    make_bundle(bundle.path(), "0.1.0");

    let readonly_root = tempdir().unwrap();
    // 읽기 전용 디렉토리: 하위 생성 불가
    let mut perms = fs::metadata(readonly_root.path()).unwrap().permissions();
    perms.set_mode(0o555);
    fs::set_permissions(readonly_root.path(), perms).unwrap();

    let target = readonly_root.path().join("moai-studio@local");
    let err = install_or_update(bundle.path(), &target).unwrap_err();

    // 테스트 후 권한 복원 (tempdir drop을 위해)
    let mut perms = fs::metadata(readonly_root.path()).unwrap().permissions();
    perms.set_mode(0o755);
    let _ = fs::set_permissions(readonly_root.path(), perms);

    assert!(
        matches!(err, BundleError::PermissionDenied { .. }),
        "기대: PermissionDenied, 실제: {err:?}"
    );
}

#[cfg(unix)]
unsafe extern "C" {
    #[link_name = "getuid"]
    fn libc_getuid() -> u32;
}
