//! moai-plugin-installer: 플러그인 설치 관리자
//!
//! MoAI Studio 플러그인을 `~/.claude/plugins/moai-studio@local/`에 설치합니다.

// @MX:ANCHOR: PluginInstaller 공개 API — 플러그인 설치/제거의 진입점
// @MX:REASON: 외부 바이너리와 통합 테스트에서 직접 호출하는 공개 인터페이스

use std::path::PathBuf;

use thiserror::Error;

/// 플러그인 설치 에러 타입
#[derive(Debug, Error)]
pub enum InstallerError {
    /// I/O 에러 (파일 복사, 디렉토리 생성 등)
    #[error("I/O 에러: {0}")]
    IoError(#[from] std::io::Error),

    /// 매니페스트 파싱 에러
    #[error("매니페스트 에러: {0}")]
    ManifestError(String),

    /// 이미 설치된 상태
    #[error("플러그인이 이미 설치되어 있습니다: {0}")]
    AlreadyInstalled(PathBuf),
}

/// 플러그인 설치 관리자
pub struct PluginInstaller {
    /// 플러그인 소스 디렉토리 (`.claude-plugin/` 포함)
    plugin_source_dir: PathBuf,
    /// 플러그인 설치 대상 디렉토리
    target_dir: PathBuf,
}

impl PluginInstaller {
    /// 새 설치 관리자를 생성합니다.
    ///
    /// # 인자
    /// - `plugin_source_dir`: 플러그인 소스 루트 (`.claude-plugin/plugin.json`과 `hooks/hooks.json` 포함)
    /// - `target_dir`: 설치 대상 디렉토리
    pub fn new(plugin_source_dir: PathBuf, target_dir: PathBuf) -> Self {
        PluginInstaller {
            plugin_source_dir,
            target_dir,
        }
    }

    /// 플러그인을 설치합니다.
    ///
    /// 설치 과정:
    /// 1. 대상 디렉토리 생성
    /// 2. `plugin.json` 복사
    /// 3. `hooks.json` 복사
    /// 4. 설치 검증
    pub fn install(&self) -> Result<(), InstallerError> {
        // 이미 설치된 경우 에러 반환
        if self.is_installed() {
            return Err(InstallerError::AlreadyInstalled(self.target_dir.clone()));
        }

        // 대상 디렉토리 생성
        std::fs::create_dir_all(&self.target_dir)?;

        // plugin.json 복사
        let plugin_json_src = self
            .plugin_source_dir
            .join(".claude-plugin")
            .join("plugin.json");
        let plugin_json_dst = self.target_dir.join("plugin.json");
        if plugin_json_src.exists() {
            std::fs::copy(&plugin_json_src, &plugin_json_dst)?;
        } else {
            return Err(InstallerError::ManifestError(format!(
                "plugin.json을 찾을 수 없습니다: {}",
                plugin_json_src.display()
            )));
        }

        // hooks.json 복사
        let hooks_json_src = self.plugin_source_dir.join("hooks").join("hooks.json");
        let hooks_json_dst = self.target_dir.join("hooks.json");
        if hooks_json_src.exists() {
            std::fs::copy(&hooks_json_src, &hooks_json_dst)?;
        } else {
            return Err(InstallerError::ManifestError(format!(
                "hooks.json을 찾을 수 없습니다: {}",
                hooks_json_src.display()
            )));
        }

        tracing::info!("플러그인 설치 완료: {}", self.target_dir.display());
        Ok(())
    }

    /// 플러그인이 설치되어 있는지 확인합니다.
    ///
    /// 대상 디렉토리가 존재하고 `plugin.json`이 있으면 설치된 것으로 판단합니다.
    pub fn is_installed(&self) -> bool {
        self.target_dir.exists() && self.target_dir.join("plugin.json").exists()
    }

    /// 플러그인을 제거합니다.
    ///
    /// 대상 디렉토리 전체를 삭제합니다.
    pub fn uninstall(&self) -> Result<(), InstallerError> {
        if self.target_dir.exists() {
            std::fs::remove_dir_all(&self.target_dir)?;
            tracing::info!("플러그인 제거 완료: {}", self.target_dir.display());
        }
        Ok(())
    }
}

/// 기본 플러그인 설치 경로를 반환합니다.
///
/// `~/.claude/plugins/moai-studio@local/`
pub fn default_target_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join("plugins")
        .join("moai-studio@local")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    /// 소스 디렉토리에 테스트용 플러그인 파일을 생성하는 헬퍼
    fn setup_plugin_source(source_dir: &Path) {
        // .claude-plugin/plugin.json 생성
        let plugin_dir = source_dir.join(".claude-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        fs::write(
            plugin_dir.join("plugin.json"),
            r#"{"name":"moai-studio","version":"0.1.0"}"#,
        )
        .unwrap();

        // hooks/hooks.json 생성
        let hooks_dir = source_dir.join("hooks");
        fs::create_dir_all(&hooks_dir).unwrap();
        fs::write(hooks_dir.join("hooks.json"), r#"{"hooks":[]}"#).unwrap();
    }

    /// default_target_dir가 올바른 경로를 반환하는지 테스트
    #[test]
    fn test_default_target_dir() {
        // Act
        let target = default_target_dir();

        // Assert: 경로에 .claude/plugins/moai-studio@local이 포함됨
        let path_str = target.to_string_lossy();
        assert!(
            path_str.contains(".claude"),
            "경로에 .claude가 없습니다: {}",
            path_str
        );
        assert!(
            path_str.contains("plugins"),
            "경로에 plugins가 없습니다: {}",
            path_str
        );
        assert!(
            path_str.contains("moai-studio@local"),
            "경로에 moai-studio@local이 없습니다: {}",
            path_str
        );
    }

    /// install()이 대상 디렉토리를 생성하는지 테스트
    #[test]
    fn test_install_creates_directory() {
        // Arrange
        let source = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin-install-dir");
        setup_plugin_source(source.path());

        let installer = PluginInstaller::new(source.path().to_path_buf(), target_path.clone());

        // Act
        installer.install().expect("설치 실패");

        // Assert: 대상 디렉토리가 생성됨
        assert!(target_path.exists(), "대상 디렉토리가 생성되지 않았습니다");
    }

    /// install()이 plugin.json을 복사하는지 테스트
    #[test]
    fn test_install_copies_plugin_json() {
        // Arrange
        let source = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin-dir");
        setup_plugin_source(source.path());

        let installer = PluginInstaller::new(source.path().to_path_buf(), target_path.clone());

        // Act
        installer.install().expect("설치 실패");

        // Assert: plugin.json이 대상 디렉토리에 존재함
        let plugin_json = target_path.join("plugin.json");
        assert!(plugin_json.exists(), "plugin.json이 복사되지 않았습니다");

        // 내용 검증
        let content = fs::read_to_string(plugin_json).unwrap();
        assert!(
            content.contains("moai-studio"),
            "plugin.json 내용이 올바르지 않습니다"
        );
    }

    /// install()이 hooks.json을 복사하는지 테스트
    #[test]
    fn test_install_copies_hooks_json() {
        // Arrange
        let source = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin-dir2");
        setup_plugin_source(source.path());

        let installer = PluginInstaller::new(source.path().to_path_buf(), target_path.clone());

        // Act
        installer.install().expect("설치 실패");

        // Assert: hooks.json이 대상 디렉토리에 존재함
        let hooks_json = target_path.join("hooks.json");
        assert!(hooks_json.exists(), "hooks.json이 복사되지 않았습니다");
    }

    /// 설치 후 is_installed()가 true를 반환하는지 테스트
    #[test]
    fn test_is_installed_after_install() {
        // Arrange
        let source = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin-dir3");
        setup_plugin_source(source.path());

        let installer = PluginInstaller::new(source.path().to_path_buf(), target_path.clone());

        // 설치 전에는 미설치 상태
        assert!(
            !installer.is_installed(),
            "설치 전에 is_installed()가 true입니다"
        );

        // Act
        installer.install().expect("설치 실패");

        // Assert: 설치 후 is_installed()가 true
        assert!(
            installer.is_installed(),
            "설치 후 is_installed()가 false입니다"
        );
    }

    /// uninstall()이 대상 디렉토리를 제거하는지 테스트
    #[test]
    fn test_uninstall_removes_directory() {
        // Arrange
        let source = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin-dir4");
        setup_plugin_source(source.path());

        let installer = PluginInstaller::new(source.path().to_path_buf(), target_path.clone());
        installer.install().expect("설치 실패");
        assert!(installer.is_installed(), "설치 확인 실패");

        // Act
        installer.uninstall().expect("제거 실패");

        // Assert: 대상 디렉토리가 제거됨
        assert!(!target_path.exists(), "제거 후 디렉토리가 남아있습니다");
        assert!(
            !installer.is_installed(),
            "제거 후 is_installed()가 true입니다"
        );
    }

    /// AlreadyInstalled 에러 테스트
    #[test]
    fn test_install_already_installed_error() {
        // Arrange
        let source = tempdir().unwrap();
        let target = tempdir().unwrap();
        let target_path = target.path().join("plugin-dir5");
        setup_plugin_source(source.path());

        let installer = PluginInstaller::new(source.path().to_path_buf(), target_path.clone());
        installer.install().expect("첫 설치 실패");

        // Act: 두 번 설치 시도
        let result = installer.install();

        // Assert: AlreadyInstalled 에러
        assert!(
            result.is_err(),
            "이미 설치된 상태에서 에러가 발생하지 않았습니다"
        );
        assert!(
            matches!(result.unwrap_err(), InstallerError::AlreadyInstalled(_)),
            "AlreadyInstalled 에러가 아닙니다"
        );
    }
}
