// @MX:NOTE: [AUTO] fs-config-defaults
// FsConfig 는 fs.yaml 의 기본값을 Rust 구조체로 표현한다.
// debounce_ms=100, backpressure_buffer_max=1000, rescan_after_backpressure=true.
// from_yaml_str 는 단위 테스트 충분; 실제 파일 I/O 는 후속 wiring 단계에서 수행한다.
// @MX:SPEC: SPEC-V3-005

use serde::Deserialize;

// ============================================================
// ConfigError — from_yaml_str 파싱 실패 타입
// ============================================================

/// FsConfig YAML 파싱 실패를 나타내는 에러 타입.
#[derive(Debug)]
pub struct ConfigError(pub String);

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FsConfig 파싱 오류: {}", self.0)
    }
}

impl std::error::Error for ConfigError {}

// ============================================================
// FsConfig — fs.yaml 의 Rust 매핑
// ============================================================

/// fs.yaml 파일의 `fs:` 섹션을 매핑하는 설정 구조체.
///
/// YAML 형식:
/// ```yaml
/// fs:
///   debounce_ms: 100
///   backpressure_buffer_max: 1000
///   rescan_after_backpressure: true
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FsConfig {
    /// debounce 윈도우 (ms) — AC-FE-5 기본값 100ms
    pub debounce_ms: u64,
    /// backpressure 임계값 — REQ-FE-013 기본값 1000
    pub backpressure_buffer_max: usize,
    /// backpressure 발생 시 full rescan 수행 여부 — 기본값 true
    pub rescan_after_backpressure: bool,
}

/// serde 역직렬화를 위한 내부 raw 구조체 (fs: 최상위 키 처리용)
#[derive(Deserialize)]
struct FsConfigRaw {
    fs: FsConfigFields,
}

#[derive(Deserialize)]
struct FsConfigFields {
    #[serde(default = "default_debounce_ms")]
    debounce_ms: u64,
    #[serde(default = "default_backpressure_buffer_max")]
    backpressure_buffer_max: usize,
    #[serde(default = "default_rescan_after_backpressure")]
    rescan_after_backpressure: bool,
}

fn default_debounce_ms() -> u64 {
    100
}
fn default_backpressure_buffer_max() -> usize {
    1000
}
fn default_rescan_after_backpressure() -> bool {
    true
}

impl Default for FsConfig {
    /// fs.yaml 의 기본값을 그대로 사용하는 FsConfig 를 반환한다.
    fn default() -> Self {
        Self {
            debounce_ms: 100,
            backpressure_buffer_max: 1000,
            rescan_after_backpressure: true,
        }
    }
}

impl FsConfig {
    /// YAML 문자열을 파싱하여 FsConfig 를 반환한다.
    ///
    /// 실제 파일 I/O 는 포함하지 않는다 (후속 wiring 단계 책임).
    pub fn from_yaml_str(s: &str) -> Result<Self, ConfigError> {
        let raw: FsConfigRaw = serde_yaml::from_str(s).map_err(|e| ConfigError(e.to_string()))?;
        Ok(Self {
            debounce_ms: raw.fs.debounce_ms,
            backpressure_buffer_max: raw.fs.backpressure_buffer_max,
            rescan_after_backpressure: raw.fs.rescan_after_backpressure,
        })
    }
}

// ============================================================
// 단위 테스트 — T6 AC-FE-5
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // T6-1: Default::default() 가 fs.yaml 기본값과 일치해야 한다
    #[test]
    fn default_matches_yaml() {
        let cfg = FsConfig::default();
        assert_eq!(cfg.debounce_ms, 100, "debounce_ms 기본값은 100이어야 한다");
        assert_eq!(
            cfg.backpressure_buffer_max, 1000,
            "backpressure_buffer_max 기본값은 1000이어야 한다"
        );
        assert!(
            cfg.rescan_after_backpressure,
            "rescan_after_backpressure 기본값은 true 이어야 한다"
        );
    }

    // T6-2: from_yaml_str 가 커스텀 값을 올바르게 파싱해야 한다
    #[test]
    fn from_yaml_str_parses_custom_values() {
        let yaml = r#"
fs:
  debounce_ms: 200
  backpressure_buffer_max: 500
  rescan_after_backpressure: false
"#;
        let cfg = FsConfig::from_yaml_str(yaml).expect("YAML 파싱 실패");
        assert_eq!(cfg.debounce_ms, 200, "커스텀 debounce_ms 파싱 실패");
        assert_eq!(
            cfg.backpressure_buffer_max, 500,
            "커스텀 backpressure_buffer_max 파싱 실패"
        );
        assert!(
            !cfg.rescan_after_backpressure,
            "커스텀 rescan_after_backpressure(false) 파싱 실패"
        );
    }

    // T6-3: 잘못된 YAML 은 ConfigError 를 반환해야 한다
    #[test]
    fn from_yaml_str_returns_error_on_invalid_yaml() {
        let invalid = "not: valid: yaml: [[[";
        let result = FsConfig::from_yaml_str(invalid);
        assert!(result.is_err(), "잘못된 YAML 은 에러를 반환해야 한다");
    }

    // T6-4: from_yaml_str 가 기본값을 유지하면서 일부 필드만 오버라이드해야 한다
    #[test]
    fn from_yaml_str_partial_override_uses_defaults() {
        let yaml = r#"
fs:
  debounce_ms: 50
"#;
        let cfg = FsConfig::from_yaml_str(yaml).expect("YAML 파싱 실패");
        assert_eq!(cfg.debounce_ms, 50, "커스텀 debounce_ms");
        assert_eq!(
            cfg.backpressure_buffer_max, 1000,
            "명시되지 않은 필드는 기본값 사용"
        );
        assert!(
            cfg.rescan_after_backpressure,
            "명시되지 않은 필드는 기본값 사용"
        );
    }
}
