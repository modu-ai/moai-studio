//! UserSettings — SPEC-V3-013 MS-3 영속화 대상 사용자 환경설정.
//!
//! ## 책임
//! - UserSettings struct + serde JSON 직렬화/역직렬화.
//! - schema_version `moai-studio/settings-v1` 포함.
//! - load_or_default: fail-soft load + .bak.{timestamp} 백업.
//! - save_atomic: tempfile + rename atomic write.
//! - settings_path: dirs::config_dir() + fallback.
//!
//! ## persistence.rs 패턴 carry
//! workspace/persistence.rs 의 atomic write + fail-soft + schema_version 패턴과 동일.

use crate::settings::settings_state::{AccentColor, Density, ThemeMode};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::warn;

// ============================================================
// 스키마 버전 상수
// ============================================================

// @MX:ANCHOR: [AUTO] user-settings-schema-v1
// @MX:REASON: [AUTO] settings.json 스키마 식별자. 향후 마이그레이션 trigger 로 사용.
//   fan_in >= 3: load_or_default, save_atomic, AC-V13-12 테스트, runtime init.
//   변경 시 기존 사용자 파일과 호환성 깨짐 — 마이그레이션 코드 필수.

/// settings.json 스키마 식별자.
pub const SCHEMA_VERSION: &str = "moai-studio/settings-v1";

// ============================================================
// AppearanceSettings
// ============================================================

/// AppearancePane 의 영속화 설정.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceSettings {
    /// 테마 선택 (dark/light/system)
    pub theme: ThemeMode,
    /// 밀도 선택 (compact/comfortable)
    pub density: Density,
    /// 액센트 색상 (4종)
    pub accent: AccentColor,
    /// 폰트 크기 (12~18px)
    pub font_size_px: u8,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            density: Density::Comfortable,
            accent: AccentColor::Teal,
            font_size_px: 14,
        }
    }
}

// ============================================================
// KeyboardSettings
// ============================================================

/// KeyboardPane 의 영속화 설정.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct KeyboardSettings {
    /// 커스텀 키 바인딩 목록 (기본값 제외, 사용자가 변경한 것만 저장).
    pub bindings: Vec<KeyBindingEntry>,
}

/// 단일 키 바인딩 직렬화 엔트리.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyBindingEntry {
    /// 액션 식별자 (예: "command_palette")
    pub action: String,
    /// 단축키 문자열 (예: "cmd-shift-p")
    pub shortcut: String,
}

// ============================================================
// Sub-pane 설정 (skeleton + 1 setting each)
// ============================================================

/// EditorPane 의 영속화 설정 (v0.1.0 skeleton — 1 setting).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EditorSettings {
    /// 탭 크기 (2~8, default 4)
    #[serde(default = "default_tab_size")]
    pub tab_size: u8,
}

fn default_tab_size() -> u8 {
    4
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self { tab_size: 4 }
    }
}

/// TerminalPane 의 영속화 설정 (v0.1.0 skeleton — 1 setting).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalSettings {
    /// 스크롤백 줄 수 (1000~100000, default 10000)
    #[serde(default = "default_scrollback_lines")]
    pub scrollback_lines: u32,
}

fn default_scrollback_lines() -> u32 {
    10_000
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            scrollback_lines: 10_000,
        }
    }
}

/// AgentPane 의 영속화 설정 (v0.1.0 skeleton — 1 setting).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AgentSettings {
    /// 자동 승인 여부 (default false)
    #[serde(default)]
    pub auto_approve: bool,
}

/// AdvancedPane 의 영속화 설정 (v0.1.0 skeleton — 1 setting).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AdvancedSettings {
    /// 실험적 플래그 목록 (read-only placeholder, default 빈 목록)
    #[serde(default)]
    pub experimental_flags: Vec<String>,
}

// ============================================================
// UserSettings — 루트 struct
// ============================================================

// @MX:ANCHOR: [AUTO] user-settings-root
// @MX:REASON: [AUTO] SPEC-V3-013 MS-3. UserSettings 는 모든 영속화 설정의 루트.
//   fan_in >= 3: load_or_default, save_atomic, ActiveTheme::from_settings, RootView init.
//   schema_version 필드 → 향후 마이그레이션 분기 진입점. 변경 금지 (R-V13-6).

/// 사용자 환경설정 루트 struct.
///
/// settings.json 에 직렬화되며 앱 시작 시 자동 load, 변경 시 200ms debounce 후 atomic write.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserSettings {
    /// JSON 스키마 버전 — 항상 `SCHEMA_VERSION` ("moai-studio/settings-v1").
    pub schema_version: String,
    /// AppearancePane 설정
    pub appearance: AppearanceSettings,
    /// KeyboardPane 설정
    pub keyboard: KeyboardSettings,
    /// EditorPane 설정
    pub editor: EditorSettings,
    /// TerminalPane 설정
    pub terminal: TerminalSettings,
    /// AgentPane 설정
    pub agent: AgentSettings,
    /// AdvancedPane 설정
    pub advanced: AdvancedSettings,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            appearance: AppearanceSettings::default(),
            keyboard: KeyboardSettings::default(),
            editor: EditorSettings::default(),
            terminal: TerminalSettings::default(),
            agent: AgentSettings::default(),
            advanced: AdvancedSettings::default(),
        }
    }
}

// ============================================================
// 에러 타입
// ============================================================

/// UserSettings 영속화 에러.
#[derive(Debug, Error)]
pub enum SettingsPersistError {
    /// I/O 에러 (파일 읽기/쓰기/rename 실패).
    #[error("settings I/O 실패: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 직렬화/역직렬화 에러.
    #[error("JSON 직렬화 실패: {0}")]
    Serde(#[from] serde_json::Error),

    /// 스키마 버전 불일치.
    #[error("스키마 버전 불일치: expected '{expected}', got '{got}'")]
    SchemaMismatch { expected: String, got: String },
}

// ============================================================
// 영속화 경로
// ============================================================

/// settings.json 의 platform-appropriate 저장 경로를 반환한다 (REQ-V13-051).
///
/// - macOS: `~/Library/Application Support/moai-studio/settings.json`
/// - Linux: `~/.config/moai-studio/settings.json`
/// - Windows: `%APPDATA%\Roaming\moai-studio\settings.json`
///
/// dirs::config_dir() 가 None 이면 `std::env::temp_dir()/moai-studio/settings.json` fallback + warn.
pub fn settings_path() -> PathBuf {
    match dirs::config_dir() {
        Some(config) => config.join("moai-studio").join("settings.json"),
        None => {
            warn!("dirs::config_dir() 가 None — temp_dir fallback 사용");
            std::env::temp_dir()
                .join("moai-studio")
                .join("settings.json")
        }
    }
}

// ============================================================
// Fail-soft load
// ============================================================

/// settings.json 을 읽어 UserSettings 를 반환한다 (REQ-V13-054, fail-soft).
///
/// - 파일 없음 → Default 반환 (warn 없음 — first run).
/// - JSON 파싱 실패 또는 schema_version 불일치 → .bak.{timestamp} 백업 + Default + warn.
pub fn load_or_default(path: &Path) -> UserSettings {
    // 파일 없음 → first run, Default 반환.
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return UserSettings::default();
        }
        Err(e) => {
            warn!(
                path = %path.display(),
                error = %e,
                "settings.json 읽기 실패, default 반환"
            );
            return UserSettings::default();
        }
    };

    // JSON 파싱
    let settings: UserSettings = match serde_json::from_slice(&bytes) {
        Ok(s) => s,
        Err(e) => {
            warn!(
                path = %path.display(),
                error = %e,
                "settings.json JSON 파싱 실패, .bak 백업 후 default 반환"
            );
            backup_corrupted(path);
            return UserSettings::default();
        }
    };

    // schema_version 검증
    if settings.schema_version != SCHEMA_VERSION {
        warn!(
            path = %path.display(),
            got = %settings.schema_version,
            expected = %SCHEMA_VERSION,
            "settings.json schema_version 불일치, .bak 백업 후 default 반환"
        );
        backup_corrupted(path);
        return UserSettings::default();
    }

    settings
}

/// 손상된 settings.json 을 `.bak.{utc_timestamp}` 로 rename 한다 (REQ-V13-055).
fn backup_corrupted(path: &Path) {
    let ts = {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    };
    let bak = path.with_file_name(format!(
        "{}.bak.{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("settings.json"),
        ts
    ));
    if let Err(e) = std::fs::rename(path, &bak) {
        warn!(
            path = %path.display(),
            bak = %bak.display(),
            error = %e,
            "settings.json .bak rename 실패"
        );
    }
}

// ============================================================
// Atomic write
// ============================================================

/// UserSettings 를 atomic write 로 settings.json 에 저장한다 (REQ-V13-057).
///
/// tempfile (.tmp.{pid}.{nanos}) 에 먼저 기록 후 rename 교체 — 부분 쓰기 방지.
pub fn save_atomic(path: &Path, settings: &UserSettings) -> Result<(), SettingsPersistError> {
    // 부모 디렉토리 자동 생성
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(settings)?;

    // tempfile 이름: <파일명>.tmp.<pid>.<nanos>
    let tmp_path = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let pid = std::process::id();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("settings.json");
        path.with_file_name(format!("{}.tmp.{}.{:x}", file_name, pid, nanos))
    };

    // tempfile 에 쓰기
    std::fs::write(&tmp_path, &json)?;

    // permission 설정 (Unix: 600)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o600))?;
    }

    // 원자적 rename
    std::fs::rename(&tmp_path, path)?;

    Ok(())
}

// ============================================================
// 단위 테스트 — RED phase (SPEC-V3-013 MS-3 UserSettings)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- serde roundtrip ----

    /// UserSettings default 의 serde 라운드트립이 구조를 보존한다 (AC-V13-10/11).
    #[test]
    fn user_settings_default_serde_roundtrip() {
        let original = UserSettings::default();
        let json = serde_json::to_string(&original).expect("직렬화 성공");
        let restored: UserSettings = serde_json::from_str(&json).expect("역직렬화 성공");
        assert_eq!(original, restored);
    }

    /// schema_version 이 직렬화 JSON 에 포함된다 (REQ-V13-050).
    #[test]
    fn schema_version_in_serialized_json() {
        let settings = UserSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(
            json.contains("moai-studio/settings-v1"),
            "schema_version 포함 필수"
        );
    }

    /// 변경된 설정이 라운드트립 후 유지된다.
    #[test]
    fn modified_settings_roundtrip() {
        let mut settings = UserSettings::default();
        settings.appearance.theme = ThemeMode::Light;
        settings.appearance.accent = AccentColor::Violet;
        settings.appearance.font_size_px = 16;
        settings.editor.tab_size = 2;

        let json = serde_json::to_string(&settings).unwrap();
        let restored: UserSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, restored);
        assert_eq!(restored.appearance.theme, ThemeMode::Light);
        assert_eq!(restored.appearance.accent, AccentColor::Violet);
        assert_eq!(restored.editor.tab_size, 2);
    }

    /// 누락된 필드는 serde default 로 채워진다 (partial JSON).
    #[test]
    fn missing_fields_use_serde_defaults() {
        // editor.tab_size 가 없는 JSON
        let partial = serde_json::json!({
            "schema_version": "moai-studio/settings-v1",
            "appearance": {
                "theme": "Dark",
                "density": "Comfortable",
                "accent": "Teal",
                "font_size_px": 14
            },
            "keyboard": { "bindings": [] },
            "editor": {},
            "terminal": {},
            "agent": {},
            "advanced": {}
        });
        let result: UserSettings = serde_json::from_value(partial).expect("누락 필드 허용");
        assert_eq!(result.editor.tab_size, 4, "tab_size default = 4");
        assert_eq!(
            result.terminal.scrollback_lines, 10_000,
            "scrollback_lines default = 10000"
        );
    }

    // ---- settings_path ----

    /// settings_path() 가 None 이 아닌 경로를 반환한다.
    #[test]
    fn settings_path_returns_some_path() {
        let path = settings_path();
        // 경로 끝이 "settings.json" 이어야 함.
        assert_eq!(
            path.file_name().and_then(|n| n.to_str()),
            Some("settings.json")
        );
        // 경로에 "moai-studio" 디렉토리가 포함되어야 함.
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("moai-studio"),
            "경로에 moai-studio 포함: {}",
            path_str
        );
    }

    // ---- load_or_default ----

    /// 파일이 없으면 Default 를 반환한다 (first run).
    #[test]
    fn load_or_default_missing_file_returns_default() {
        let dir = tempfile_dir("missing");
        let path = dir.join("settings.json");
        // 파일이 없는 상태
        let result = load_or_default(&path);
        assert_eq!(result, UserSettings::default());
    }

    /// 정상적인 파일을 읽으면 동일한 설정을 반환한다.
    #[test]
    fn load_or_default_valid_file_returns_settings() {
        let dir = tempfile_dir("valid");
        let path = dir.join("settings.json");

        let mut settings = UserSettings::default();
        settings.appearance.font_size_px = 16;
        save_atomic(&path, &settings).expect("save 성공");

        let loaded = load_or_default(&path);
        assert_eq!(loaded, settings);
        assert_eq!(loaded.appearance.font_size_px, 16);
    }

    /// 손상된 JSON 파일은 .bak.{ts} 백업 + Default 반환 (AC-V13-12).
    #[test]
    fn load_or_default_corrupted_json_backup_and_default() {
        let dir = tempfile_dir("corrupted");
        let path = dir.join("settings.json");

        // 손상된 JSON 작성
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(&path, b"{ this is NOT valid json !!!").unwrap();
        assert!(path.exists(), "손상된 파일이 존재해야 함");

        let result = load_or_default(&path);
        assert_eq!(result, UserSettings::default(), "손상 시 Default 반환");

        // 원본 파일이 없어져야 함 (.bak 로 이동됨)
        assert!(!path.exists(), "원본 파일이 .bak 으로 이동되어 없어야 함");

        // .bak.{ts} 파일이 존재해야 함
        let bak_files: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".bak."))
            .collect();
        assert!(
            !bak_files.is_empty(),
            ".bak 파일이 존재해야 함: {:?}",
            bak_files
        );
    }

    /// schema_version 불일치 시 .bak 백업 + Default 반환 (REQ-V13-056).
    #[test]
    fn load_or_default_schema_mismatch_backup_and_default() {
        let dir = tempfile_dir("schema-mismatch");
        let path = dir.join("settings.json");
        std::fs::create_dir_all(&dir).unwrap();

        // 잘못된 schema_version
        let bad_json = serde_json::json!({
            "schema_version": "moai-studio/settings-v99",
            "appearance": {
                "theme": "Dark",
                "density": "Comfortable",
                "accent": "Teal",
                "font_size_px": 14
            },
            "keyboard": { "bindings": [] },
            "editor": { "tab_size": 4 },
            "terminal": { "scrollback_lines": 10000 },
            "agent": { "auto_approve": false },
            "advanced": { "experimental_flags": [] }
        });
        std::fs::write(&path, serde_json::to_string(&bad_json).unwrap()).unwrap();

        let result = load_or_default(&path);
        assert_eq!(
            result,
            UserSettings::default(),
            "schema_version 불일치 → Default"
        );
        assert!(!path.exists(), "원본 파일이 .bak 으로 이동되어 없어야 함");
    }

    // ---- save_atomic ----

    /// save_atomic + load_or_default 라운드트립이 동일 struct 를 반환한다 (AC-V13-10).
    #[test]
    fn save_and_load_roundtrip_matches() {
        let dir = tempfile_dir("roundtrip");
        let path = dir.join("settings.json");

        let mut settings = UserSettings::default();
        settings.appearance.accent = AccentColor::Blue;
        settings.appearance.density = Density::Compact;
        settings.agent.auto_approve = true;

        save_atomic(&path, &settings).expect("save 성공");
        let loaded = load_or_default(&path);
        assert_eq!(settings, loaded);
    }

    /// save_atomic 성공 후 tmpfile 잔재가 없다 (REQ-V13-057).
    #[test]
    fn save_atomic_no_tmp_leftovers() {
        let dir = tempfile_dir("atomic");
        let path = dir.join("settings.json");

        save_atomic(&path, &UserSettings::default()).expect("save 성공");

        // .tmp. 파일 잔재 확인
        let tmp_leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
            .collect();
        assert!(
            tmp_leftovers.is_empty(),
            "성공 후 tmpfile 잔재 없어야 함: {:?}",
            tmp_leftovers
        );
    }

    /// 부모 디렉토리가 없어도 save_atomic 이 자동 생성한다.
    #[test]
    fn save_atomic_creates_parent_dir() {
        let dir = tempfile_dir("parent-create").join("nested").join("deep");
        let path = dir.join("settings.json");
        assert!(!dir.exists(), "디렉토리가 아직 없어야 함");

        save_atomic(&path, &UserSettings::default()).expect("save 성공");
        assert!(path.exists(), "파일이 생성되어야 함");
    }

    // ---- 헬퍼 ----

    fn tempfile_dir(suffix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("moai-settings-test-{}", suffix));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }
}
