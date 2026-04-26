// @MX:ANCHOR: [AUTO] dnd-drag-payload
// @MX:REASON: [AUTO] DragPayload 는 REQ-FE-040/041/042 의 공개 API 경계.
//   fan_in >= 3: FileExplorer::drag_start, FileExplorer::drop_on_dir, 통합 테스트.
// @MX:SPEC: SPEC-V3-005

use std::path::{Path, PathBuf};

// ============================================================
// DragPayload — drag 시작 시 캡처되는 소스 경로
// ============================================================

/// drag 시작 시 캡처되는 소스 경로 (REQ-FE-040).
///
/// `source` 는 워크스페이스 루트 기준 상대 경로.
#[derive(Debug, Clone, PartialEq)]
pub struct DragPayload {
    /// 드래그 중인 파일/폴더의 상대 경로
    pub source: PathBuf,
}

impl DragPayload {
    /// 새 DragPayload 를 생성한다.
    pub fn new(source: PathBuf) -> Self {
        Self { source }
    }
}

// ============================================================
// DropError — drop 유효성 오류
// ============================================================

/// drop 유효성 검사 실패 오류 (REQ-FE-042/043).
#[derive(Debug, PartialEq)]
pub enum DropError {
    /// source 와 target 이 동일한 경우 (self-drop)
    SelfDrop,
    /// target 이 source 의 하위 디렉토리인 경우 (descendant-drop)
    DescendantDrop,
    /// fs 오류 (cross-device link 등)
    Other(String),
}

impl std::fmt::Display for DropError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DropError::SelfDrop => write!(f, "자기 자신에게 drop 할 수 없다"),
            DropError::DescendantDrop => write!(f, "하위 디렉토리에 drop 할 수 없다"),
            DropError::Other(msg) => write!(f, "drop 실패: {msg}"),
        }
    }
}

// ============================================================
// validate_drop — source/target 유효성 검사
// ============================================================

/// source 와 target_dir 간 drop 유효성을 검사하고 목적지 경로를 반환한다 (REQ-FE-042).
///
/// 성공 시 `target_dir.join(source.file_name())` 경로를 반환한다.
/// 실패 시 DropError 를 반환한다 (no panic, REQ-FE-043).
pub fn validate_drop(source: &Path, target_dir: &Path) -> Result<PathBuf, DropError> {
    // self-drop: source == target_dir
    if source == target_dir {
        return Err(DropError::SelfDrop);
    }

    // descendant-drop: target_dir 가 source 의 하위인 경우
    // target_dir 가 source 로 시작하면 descendant
    if target_dir.starts_with(source) {
        return Err(DropError::DescendantDrop);
    }

    // source 와 target_dir 의 부모가 동일하고 이름도 같은 경우 (target = source 의 parent)
    // → source 파일을 자신의 부모로 이동하는 것 = 이미 있는 위치
    // 이는 허용 (덮어쓰기 없음, 위치 변화 없음)이 아니라 논리적 no-op.
    // SPEC 은 이 케이스를 명시하지 않으므로 허용으로 처리 (rename 호출 결과로 처리)

    let file_name = source
        .file_name()
        .ok_or_else(|| DropError::Other("source 에 파일 이름이 없다".to_string()))?;

    Ok(target_dir.join(file_name))
}

// ============================================================
// perform_drop — fs::rename 으로 파일 이동
// ============================================================

/// DragPayload 를 target_dir 로 이동한다 (REQ-FE-041/043).
///
/// 내부적으로 validate_drop 을 수행한 후 std::fs::rename 을 호출한다.
/// 실패 시 panic 없이 io::Error 를 반환한다 (REQ-FE-043).
pub fn perform_drop(
    payload: &DragPayload,
    workspace_root: &Path,
    target_dir_rel: &Path,
) -> Result<PathBuf, DropError> {
    // 절대 경로로 변환
    let abs_source = workspace_root.join(&payload.source);
    let abs_target_dir = workspace_root.join(target_dir_rel);

    // 유효성 검사 (상대 경로 기준)
    let dest_rel = validate_drop(&payload.source, target_dir_rel)?;
    let abs_dest = workspace_root.join(&dest_rel);

    // fs::rename 실행
    std::fs::rename(&abs_source, &abs_dest)
        .map_err(|e| DropError::Other(format!("{abs_source:?} → {abs_target_dir:?}: {e}")))?;

    Ok(dest_rel)
}

// ============================================================
// 단위 테스트 — AC-FE-11
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // AC-FE-11: validate_drop self-drop → SelfDrop 에러
    #[test]
    fn validate_drop_self_returns_self_drop_error() {
        let path = Path::new("src/main.rs");
        let result = validate_drop(path, path);
        assert_eq!(result, Err(DropError::SelfDrop), "self-drop 은 SelfDrop 에러여야 한다");
    }

    // AC-FE-11: validate_drop descendant-drop → DescendantDrop 에러
    #[test]
    fn validate_drop_descendant_returns_descendant_drop_error() {
        let source = Path::new("src");
        let target = Path::new("src/subdir");
        let result = validate_drop(source, target);
        assert_eq!(
            result,
            Err(DropError::DescendantDrop),
            "하위 디렉토리 drop 은 DescendantDrop 에러여야 한다"
        );
    }

    // AC-FE-11: validate_drop 유효한 경우 → 목적지 경로 반환
    #[test]
    fn validate_drop_valid_returns_destination_path() {
        let source = Path::new("src/main.rs");
        let target_dir = Path::new("lib");
        let result = validate_drop(source, target_dir);
        assert_eq!(
            result,
            Ok(PathBuf::from("lib/main.rs")),
            "유효한 drop 은 목적지 경로를 반환해야 한다"
        );
    }

    // AC-FE-11: validate_drop dir → dir (다른 경로)
    #[test]
    fn validate_drop_dir_to_dir_valid() {
        let source = Path::new("crates/auth");
        let target_dir = Path::new("packages");
        let result = validate_drop(source, target_dir);
        assert_eq!(result, Ok(PathBuf::from("packages/auth")));
    }

    // AC-FE-11: perform_drop tempdir 에서 파일 이동 검증
    #[test]
    fn perform_drop_moves_file_in_tempdir() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        let workspace = dir.path();

        // src 디렉토리와 target 디렉토리 생성
        fs::create_dir(workspace.join("src")).expect("src 생성 실패");
        fs::create_dir(workspace.join("target")).expect("target 생성 실패");

        // src/main.rs 파일 생성
        fs::write(workspace.join("src/main.rs"), b"fn main() {}").expect("파일 생성 실패");

        let payload = DragPayload::new(PathBuf::from("src/main.rs"));
        let result = perform_drop(&payload, workspace, Path::new("target"));

        assert!(result.is_ok(), "유효한 drop 은 성공해야 한다: {:?}", result);
        let dest = result.unwrap();
        assert_eq!(dest, PathBuf::from("target/main.rs"));
        assert!(workspace.join(&dest).exists(), "이동된 파일이 있어야 한다");
        assert!(!workspace.join("src/main.rs").exists(), "원본 파일은 없어야 한다");
    }

    // AC-FE-11: self-drop → 거부 + no panic
    #[test]
    fn perform_drop_self_drop_returns_error_no_panic() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        let workspace = dir.path();

        // 파일 생성
        fs::write(workspace.join("main.rs"), b"content").expect("파일 생성 실패");

        let payload = DragPayload::new(PathBuf::from("main.rs"));
        // target_dir 도 같은 경로 (self-drop)
        let result = perform_drop(&payload, workspace, Path::new("main.rs"));
        assert_eq!(result, Err(DropError::SelfDrop));
    }

    // AC-FE-11: descendant-drop → 거부 + no panic
    #[test]
    fn perform_drop_descendant_drop_returns_error_no_panic() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        let workspace = dir.path();

        fs::create_dir_all(workspace.join("src/subdir")).expect("디렉토리 생성 실패");

        let payload = DragPayload::new(PathBuf::from("src"));
        // target 이 source 의 하위
        let result = perform_drop(&payload, workspace, Path::new("src/subdir"));
        assert_eq!(result, Err(DropError::DescendantDrop));
    }
}
