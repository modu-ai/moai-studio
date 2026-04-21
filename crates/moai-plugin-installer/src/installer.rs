//! 플러그인 번들 설치/업데이트 로직 (T-017)
//!
//! 번들 디렉토리의 모든 파일을 대상 디렉토리로 재귀 복사하며,
//! `plugin.json`의 `version` 필드를 semver로 비교하여 업데이트 여부를 결정한다.

// @MX:ANCHOR: BundleInstaller 공개 API — 플러그인 번들 복사 및 버전 비교 진입점
// @MX:REASON: installer 외부(FFI, 통합 테스트)에서 직접 호출하는 고-fan-in 경로

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use semver::Version;
use serde::Deserialize;
use thiserror::Error;
use walkdir::WalkDir;

/// 설치/업데이트 결과
#[derive(Debug, PartialEq, Eq)]
pub enum InstallOutcome {
    /// 신규 설치 완료
    Installed,
    /// 번들 버전이 더 높아 업데이트 완료
    Updated,
    /// 설치된 버전이 같거나 더 높아 건너뜀
    Skipped,
}

/// 번들 설치 에러
#[derive(Debug, Error)]
pub enum BundleError {
    #[error("I/O 에러: {0}")]
    Io(#[from] io::Error),

    #[error("매니페스트 파싱 실패: {0}")]
    Manifest(String),

    #[error("버전 파싱 실패: {0}")]
    Version(String),

    /// 쓰기 권한 오류 (EACCES/EPERM). 호출자는 수동 설치 안내를 표시해야 한다.
    // @MX:WARN: 이 variant가 반환되면 UI 레이어가 사용자에게 안내를 표시해야 한다
    // @MX:REASON: macOS 샌드박스 또는 소유권 문제로 ~/.claude에 쓰기 실패할 수 있음
    #[error("쓰기 권한 없음: {path} ({source})")]
    PermissionDenied {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

#[derive(Debug, Deserialize)]
struct PluginManifestVersion {
    version: String,
}

/// 번들 플러그인을 대상 디렉토리에 설치하거나 업데이트한다.
///
/// - `bundle_dir`: 소스 플러그인 루트 (예: `<app>/plugin/`)
/// - `target_dir`: 설치 대상 (예: `~/.claude/plugins/moai-studio@local/`)
///
/// 동작:
/// 1. 번들 `plugin.json` 읽어 version 추출
/// 2. 대상에 기존 `plugin.json`이 있다면 version 추출하여 비교
/// 3. installed >= bundle → `Skipped`
/// 4. 그 외: 대상 디렉토리 내용 제거 후 전체 트리 복사
pub fn install_or_update(
    bundle_dir: &Path,
    target_dir: &Path,
) -> Result<InstallOutcome, BundleError> {
    let bundle_version = read_manifest_version(&bundle_dir.join(".claude-plugin/plugin.json"))?;

    let installed_manifest = target_dir.join(".claude-plugin/plugin.json");
    if installed_manifest.exists() {
        match read_manifest_version(&installed_manifest) {
            Ok(installed) => {
                if installed >= bundle_version {
                    tracing::info!("설치된 버전 {installed} >= 번들 {bundle_version} — 건너뜀");
                    return Ok(InstallOutcome::Skipped);
                }
            }
            Err(err) => {
                tracing::warn!("기존 매니페스트 파싱 실패({err}) — 강제 재설치");
            }
        }

        copy_tree(bundle_dir, target_dir)?;
        return Ok(InstallOutcome::Updated);
    }

    copy_tree(bundle_dir, target_dir)?;
    Ok(InstallOutcome::Installed)
}

/// 번들 디렉토리를 대상으로 재귀 복사 (기존 대상 내용은 제거 후 복사).
fn copy_tree(bundle_dir: &Path, target_dir: &Path) -> Result<(), BundleError> {
    if target_dir.exists() {
        remove_dir_all_mapped(target_dir)?;
    }
    create_dir_all_mapped(target_dir)?;

    for entry in WalkDir::new(bundle_dir).into_iter().filter_map(Result::ok) {
        let src_path = entry.path();
        let rel = match src_path.strip_prefix(bundle_dir) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if rel.as_os_str().is_empty() {
            continue;
        }
        let dst_path = target_dir.join(rel);

        if entry.file_type().is_dir() {
            create_dir_all_mapped(&dst_path)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dst_path.parent() {
                create_dir_all_mapped(parent)?;
            }
            copy_file_mapped(src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn read_manifest_version(manifest_path: &Path) -> Result<Version, BundleError> {
    let content = fs::read_to_string(manifest_path)
        .map_err(|e| BundleError::Manifest(format!("{}: {e}", manifest_path.display())))?;
    let parsed: PluginManifestVersion = serde_json::from_str(&content)
        .map_err(|e| BundleError::Manifest(format!("JSON 파싱 실패: {e}")))?;
    Version::parse(&parsed.version)
        .map_err(|e| BundleError::Version(format!("'{}': {e}", parsed.version)))
}

// --- 권한 오류 매핑 헬퍼 ---

fn is_permission_error(err: &io::Error) -> bool {
    matches!(err.kind(), io::ErrorKind::PermissionDenied)
}

fn map_perm(path: &Path, err: io::Error) -> BundleError {
    if is_permission_error(&err) {
        BundleError::PermissionDenied {
            path: path.to_path_buf(),
            source: err,
        }
    } else {
        BundleError::Io(err)
    }
}

fn create_dir_all_mapped(path: &Path) -> Result<(), BundleError> {
    fs::create_dir_all(path).map_err(|e| map_perm(path, e))
}

fn remove_dir_all_mapped(path: &Path) -> Result<(), BundleError> {
    fs::remove_dir_all(path).map_err(|e| map_perm(path, e))
}

fn copy_file_mapped(src: &Path, dst: &Path) -> Result<(), BundleError> {
    fs::copy(src, dst).map(|_| ()).map_err(|e| map_perm(dst, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_bundle(dir: &Path, version: &str) {
        fs::create_dir_all(dir.join(".claude-plugin")).unwrap();
        fs::write(
            dir.join(".claude-plugin/plugin.json"),
            format!(
                r#"{{"name":"moai-studio","version":"{version}","claude_min_version":"1.0.0"}}"#
            ),
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
    fn semver_equal_skips() {
        let bundle = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin");
        write_bundle(bundle.path(), "0.1.0");
        // First install
        let r = install_or_update(bundle.path(), &target_path).unwrap();
        assert_eq!(r, InstallOutcome::Installed);
        // Same version → skipped
        let r = install_or_update(bundle.path(), &target_path).unwrap();
        assert_eq!(r, InstallOutcome::Skipped);
    }

    #[test]
    fn semver_greater_updates() {
        let bundle = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin");
        write_bundle(bundle.path(), "0.1.0");
        install_or_update(bundle.path(), &target_path).unwrap();
        // Bump bundle version
        write_bundle(bundle.path(), "0.2.0");
        let r = install_or_update(bundle.path(), &target_path).unwrap();
        assert_eq!(r, InstallOutcome::Updated);
        let installed = fs::read_to_string(target_path.join(".claude-plugin/plugin.json")).unwrap();
        assert!(installed.contains("0.2.0"));
    }

    #[test]
    fn semver_installed_greater_skips() {
        let bundle = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin");
        write_bundle(bundle.path(), "0.9.0");
        install_or_update(bundle.path(), &target_path).unwrap();
        // Downgrade bundle
        write_bundle(bundle.path(), "0.1.0");
        let r = install_or_update(bundle.path(), &target_path).unwrap();
        assert_eq!(r, InstallOutcome::Skipped);
    }

    #[test]
    fn prerelease_ordering() {
        // 0.2.0 > 0.2.0-beta (per semver)
        let bundle = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin");
        write_bundle(bundle.path(), "0.2.0-beta");
        let r = install_or_update(bundle.path(), &target_path).unwrap();
        assert_eq!(r, InstallOutcome::Installed);
        write_bundle(bundle.path(), "0.2.0");
        let r = install_or_update(bundle.path(), &target_path).unwrap();
        assert_eq!(r, InstallOutcome::Updated);
        // And back to prerelease → skipped
        write_bundle(bundle.path(), "0.2.0-beta");
        let r = install_or_update(bundle.path(), &target_path).unwrap();
        assert_eq!(r, InstallOutcome::Skipped);
    }

    #[test]
    fn copies_full_tree() {
        let bundle = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin");
        write_bundle(bundle.path(), "0.1.0");
        // Add nested file
        fs::create_dir_all(bundle.path().join("hooks/sub")).unwrap();
        fs::write(bundle.path().join("hooks/sub/extra.txt"), "hi").unwrap();

        install_or_update(bundle.path(), &target_path).unwrap();
        assert!(target_path.join(".claude-plugin/plugin.json").exists());
        assert!(target_path.join("hooks/hooks.json").exists());
        assert!(target_path.join("mcp-config.json").exists());
        assert!(target_path.join("hooks/sub/extra.txt").exists());
    }

    #[test]
    fn invalid_bundle_version_errors() {
        let bundle = tempdir().unwrap();
        let target = tempdir().unwrap();
        fs::create_dir_all(bundle.path().join(".claude-plugin")).unwrap();
        fs::write(
            bundle.path().join(".claude-plugin/plugin.json"),
            r#"{"name":"x","version":"not-semver"}"#,
        )
        .unwrap();
        let r = install_or_update(bundle.path(), &target.path().join("out"));
        assert!(matches!(r, Err(BundleError::Version(_))));
    }
}
