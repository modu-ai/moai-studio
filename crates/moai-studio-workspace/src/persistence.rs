//! Pane/Tab 레이아웃 persistence — `moai-studio/panes-v1` JSON 스키마.
//!
//! ## 책임
//! - `PaneLayoutV1` 직렬화/역직렬화 (serde_json)
//! - 원자적 파일 쓰기 (tempfile → rename)
//! - 스키마 버전 검증 (`moai-studio/panes-v1`)
//! - 손상된 JSON 안전 실패 (panic 금지, Default 반환 + warn 로그)
//! - cwd fallback: 저장된 경로가 없거나 존재하지 않으면 $HOME 반환

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::warn;

// ============================================================
// @MX:ANCHOR: [AUTO] persist-schema-v1
// @MX:REASON: 스키마 변경 시 MS-1/MS-2 사용자 layout 마이그레이션 필요.
//             save_panes / load_panes / T13 hooks / CI tests 전반에서 fan_in ≥ 3.
//             호환성 깨짐 방지 invariant — 상수값 변경 시 마이그레이션 코드 필수.
// ============================================================

/// panes-v1 JSON 스키마 식별자 상수.
pub const SCHEMA_VERSION: &str = "moai-studio/panes-v1";

// ============================================================
// 스키마 타입 (T13 변환 레이어는 미포함 — T13에서 TabContainer/PaneTree 연동)
// ============================================================

/// panes-v1 최상위 레이아웃 스냅샷.
///
/// @MX:ANCHOR: [AUTO] persist-schema-v1 (위 상수와 동일 fan_in 그룹)
/// @MX:REASON: 구조 변경 시 마이그레이션 필요 — 필드 추가/제거 금지 (backward-compat).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaneLayoutV1 {
    /// 스키마 버전 — 항상 `SCHEMA_VERSION` ("moai-studio/panes-v1").
    pub schema_version: String,
    /// 모든 탭 스냅샷 목록.
    pub tabs: Vec<TabSnapshotV1>,
}

impl Default for PaneLayoutV1 {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            tabs: Vec::new(),
        }
    }
}

/// 탭 하나의 스냅샷.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TabSnapshotV1 {
    /// 탭 식별자 (TabId 문자열).
    pub id: String,
    /// 탭 타이틀.
    pub title: String,
    /// 마지막 포커스된 pane ID (없으면 None).
    pub last_focused_pane: Option<String>,
    /// Pane tree 스냅샷.
    pub pane_tree: PaneTreeSnapshotV1,
}

/// Pane tree 스냅샷 — 재귀 enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PaneTreeSnapshotV1 {
    /// 단말 pane.
    Leaf {
        /// Pane 식별자.
        id: String,
        /// 작업 디렉토리 (없으면 None).
        cwd: Option<String>,
    },
    /// 수평/수직 분할.
    Split {
        /// SplitNode 식별자.
        id: String,
        /// 분할 방향 ("horizontal" | "vertical").
        direction: String,
        /// 첫 번째 자식 비율 (0.0 ~ 1.0).
        ratio: f32,
        /// 첫 번째 자식.
        first: Box<PaneTreeSnapshotV1>,
        /// 두 번째 자식.
        second: Box<PaneTreeSnapshotV1>,
    },
}

// ============================================================
// 에러 타입
// ============================================================

/// Persistence 에러.
#[derive(Debug, Error)]
pub enum PersistError {
    /// I/O 에러 (파일 읽기/쓰기/rename 실패).
    #[error("persistence I/O 실패: {0}")]
    Io(#[from] std::io::Error),

    /// JSON 직렬화/역직렬화 에러.
    #[error("JSON 직렬화 실패: {0}")]
    Serde(#[from] serde_json::Error),

    /// 스키마 버전 불일치.
    #[error("스키마 버전 불일치: expected '{expected}', got '{got}'")]
    SchemaMismatch { expected: String, got: String },

    /// 손상된 JSON (구조 파싱 실패 — safe-fail).
    #[error("손상된 JSON, default layout 반환됨")]
    Corrupted,
}

// ============================================================
// Public API
// ============================================================

/// pane 레이아웃을 원자적으로 파일에 저장한다.
///
/// tempfile (.tmp suffix) 에 먼저 기록한 후 rename 으로 교체해
/// 동시 쓰기/크래시 시 기존 파일을 보존한다.
///
/// # @MX:WARN: [AUTO] atomic-write-race
/// # @MX:REASON: 동시 저장 시 tempfile 이름 충돌 가능.
/// #             std::process::id() + nanos suffix 로 회피.
pub fn save_panes(path: &Path, layout: &PaneLayoutV1) -> Result<(), PersistError> {
    // 부모 디렉토리 자동 생성
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(layout)?;

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
            .unwrap_or("panes.json");
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

/// pane 레이아웃을 파일에서 읽는다.
///
/// 스키마 버전이 일치하지 않으면 `SchemaMismatch` 에러를 반환한다.
/// JSON 이 손상된 경우 `Default` layout 을 반환하고 `warn` 로그를 남긴다 (panic 금지).
///
/// # @MX:NOTE: [AUTO] safe-fail-default
/// AC-P-15: 손상된 JSON 은 애플리케이션을 중단시키지 않는다.
/// Default layout (빈 탭 목록) 을 반환하고 warn 로그로 이상을 알린다.
pub fn load_panes(path: &Path) -> Result<PaneLayoutV1, PersistError> {
    let bytes = std::fs::read(path)?;

    let layout: PaneLayoutV1 = match serde_json::from_slice(&bytes) {
        Ok(l) => l,
        Err(e) => {
            // @MX:NOTE: [AUTO] safe-fail-default — AC-P-15: corrupted JSON → default layout
            warn!(
                path = %path.display(),
                error = %e,
                "load_panes: JSON 손상 감지, default layout 반환"
            );
            return Ok(PaneLayoutV1::default());
        }
    };

    // 스키마 버전 검증
    if layout.schema_version != SCHEMA_VERSION {
        return Err(PersistError::SchemaMismatch {
            expected: SCHEMA_VERSION.to_string(),
            got: layout.schema_version,
        });
    }

    Ok(layout)
}

/// cwd fallback 해석.
///
/// `saved` 경로가 Some 이고 실제로 존재하면 그대로 반환.
/// None 이거나 경로가 존재하지 않으면 `$HOME` 을 반환한다 (REQ-P-056).
pub fn resolve_cwd_with_fallback(saved: Option<&Path>) -> PathBuf {
    if let Some(p) = saved
        && p.exists()
    {
        return p.to_path_buf();
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

// ============================================================
// 유닛 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 테스트용 최소 PaneLayoutV1 픽스처.
    fn sample_layout() -> PaneLayoutV1 {
        PaneLayoutV1 {
            schema_version: SCHEMA_VERSION.to_string(),
            tabs: vec![TabSnapshotV1 {
                id: "tab-abc".to_string(),
                title: "main".to_string(),
                last_focused_pane: Some("pane-1".to_string()),
                pane_tree: PaneTreeSnapshotV1::Split {
                    id: "split-1".to_string(),
                    direction: "horizontal".to_string(),
                    ratio: 0.5,
                    first: Box::new(PaneTreeSnapshotV1::Leaf {
                        id: "pane-1".to_string(),
                        cwd: Some("/home/user".to_string()),
                    }),
                    second: Box::new(PaneTreeSnapshotV1::Leaf {
                        id: "pane-2".to_string(),
                        cwd: None,
                    }),
                },
            }],
        }
    }

    /// AC-P-12 (round-trip): serialize → deserialize 후 구조가 동일해야 한다.
    #[test]
    fn round_trip_panes_v1_preserves_structure() {
        let tmp = tempfile_path("round-trip.json");
        let original = sample_layout();

        save_panes(&tmp, &original).expect("save 성공");
        let loaded = load_panes(&tmp).expect("load 성공");

        assert_eq!(original, loaded, "round-trip 후 구조 동일");

        // 중첩 구조 검증
        assert_eq!(loaded.tabs.len(), 1);
        let tab = &loaded.tabs[0];
        assert_eq!(tab.id, "tab-abc");
        assert_eq!(tab.last_focused_pane, Some("pane-1".to_string()));

        match &tab.pane_tree {
            PaneTreeSnapshotV1::Split {
                id,
                direction,
                ratio,
                first,
                second,
            } => {
                assert_eq!(id, "split-1");
                assert_eq!(direction, "horizontal");
                assert!((ratio - 0.5).abs() < f32::EPSILON);
                match first.as_ref() {
                    PaneTreeSnapshotV1::Leaf { id, cwd } => {
                        assert_eq!(id, "pane-1");
                        assert_eq!(cwd.as_deref(), Some("/home/user"));
                    }
                    _ => panic!("first should be Leaf"),
                }
                match second.as_ref() {
                    PaneTreeSnapshotV1::Leaf { id, cwd } => {
                        assert_eq!(id, "pane-2");
                        assert!(cwd.is_none());
                    }
                    _ => panic!("second should be Leaf"),
                }
            }
            _ => panic!("root should be Split"),
        }

        let _ = std::fs::remove_file(&tmp);
    }

    /// AC-P-13: save_panes 는 tempfile → rename 방식을 사용해야 한다.
    /// tempfile 이 저장 완료 시 제거되고, 최종 파일이 존재해야 한다.
    #[test]
    fn atomic_write_uses_tempfile_rename() {
        let tmp_dir = std::env::temp_dir().join("moai-persist-atomic");
        std::fs::create_dir_all(&tmp_dir).unwrap();
        let target = tmp_dir.join("layout.json");
        let _ = std::fs::remove_file(&target);

        let layout = sample_layout();
        save_panes(&target, &layout).expect("save 성공");

        // 최종 파일 존재 확인
        assert!(target.exists(), "rename 후 최종 파일 존재");

        // tmpfile 잔재 없음 확인 (*.tmp.* 패턴)
        let tmp_leftovers: Vec<_> = std::fs::read_dir(&tmp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp."))
            .collect();
        assert!(
            tmp_leftovers.is_empty(),
            "성공 후 tmpfile 잔재 없어야 함: {:?}",
            tmp_leftovers
        );

        // JSON 내용 검증
        let raw = std::fs::read_to_string(&target).unwrap();
        assert!(raw.contains("moai-studio/panes-v1"), "스키마 버전 포함");

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }

    /// AC-P-13a: 잘못된 schema_version 은 SchemaMismatch 에러를 반환해야 한다.
    #[test]
    fn reject_unknown_schema_version() {
        let tmp = tempfile_path("wrong-schema.json");

        // 잘못된 스키마 버전을 직접 기록
        let bad_json = serde_json::json!({
            "schema_version": "moai-studio/panes-v99",
            "tabs": []
        });
        std::fs::write(&tmp, serde_json::to_string(&bad_json).unwrap()).unwrap();

        let result = load_panes(&tmp);
        match result {
            Err(PersistError::SchemaMismatch { expected, got }) => {
                assert_eq!(expected, SCHEMA_VERSION);
                assert_eq!(got, "moai-studio/panes-v99");
            }
            other => panic!("SchemaMismatch 예상, got: {:?}", other),
        }

        let _ = std::fs::remove_file(&tmp);
    }

    /// AC-P-14: saved 경로가 None 또는 존재하지 않으면 $HOME 을 반환해야 한다.
    #[test]
    fn missing_cwd_falls_back_to_home() {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .expect("HOME 환경변수 필요");

        // None → $HOME
        let result_none = resolve_cwd_with_fallback(None);
        assert_eq!(result_none, home, "None → $HOME");

        // 존재하지 않는 경로 → $HOME
        let nonexistent = PathBuf::from("/this/path/does/not/exist/12345");
        let result_missing = resolve_cwd_with_fallback(Some(&nonexistent));
        assert_eq!(result_missing, home, "존재하지 않는 경로 → $HOME");

        // 존재하는 경로 → 그대로 반환
        let existing = std::env::temp_dir();
        let result_existing = resolve_cwd_with_fallback(Some(&existing));
        assert_eq!(result_existing, existing, "존재하는 경로 → 그대로");
    }

    /// AC-P-15: 손상된 JSON 은 Default layout 을 반환해야 한다 (panic 금지).
    #[test]
    fn corrupted_json_returns_default_layout() {
        let tmp = tempfile_path("corrupted.json");

        // 유효하지 않은 JSON 기록
        std::fs::write(&tmp, b"{ this is not valid json !!!").unwrap();

        let result = load_panes(&tmp).expect("corrupted JSON 는 에러 대신 default 반환");
        assert_eq!(
            result,
            PaneLayoutV1::default(),
            "손상된 JSON → default layout (빈 탭)"
        );
        assert_eq!(result.schema_version, SCHEMA_VERSION);
        assert!(result.tabs.is_empty());

        let _ = std::fs::remove_file(&tmp);
    }

    // ---- 헬퍼 ----

    fn tempfile_path(suffix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("moai-persist-test-{}", suffix))
    }
}
