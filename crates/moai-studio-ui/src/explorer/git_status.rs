// @MX:ANCHOR: [AUTO] git-status-provider-trait
// @MX:REASON: [AUTO] GitStatusProvider 는 SPEC-V3-005 RG-FE-3 의 공개 API 경계.
//   fan_in >= 3: MoaiGitStatusProvider(default), FileExplorer::refresh_git_status,
//   미래 SPEC-V3-008 의 캐싱 구현체 주입.
// @MX:SPEC: SPEC-V3-005

use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ============================================================
// GitStatus — 파일별 git 상태 enum
// ============================================================

/// 파일 시스템 노드의 git 상태 (REQ-FE-020).
///
/// 우선순위 (roll_up_priority 에 사용):
/// Conflicted > Modified > Added > Deleted > Renamed > Untracked > Clean
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GitStatus {
    /// 변경 없음
    Clean,
    /// 수정됨 (인덱스 또는 워킹 트리)
    Modified,
    /// 새로 추가됨 (스테이지드)
    Added,
    /// 삭제됨
    Deleted,
    /// 추적되지 않음 (untracked)
    Untracked,
    /// 이름 변경됨
    Renamed,
    /// 충돌 상태
    Conflicted,
}

impl GitStatus {
    /// 우선순위 수치를 반환한다. 높을수록 상위 우선순위.
    /// Conflicted=6, Modified=5, Added=4, Deleted=3, Renamed=2, Untracked=1, Clean=0
    pub fn priority(&self) -> u8 {
        match self {
            GitStatus::Conflicted => 6,
            GitStatus::Modified => 5,
            GitStatus::Added => 4,
            GitStatus::Deleted => 3,
            GitStatus::Renamed => 2,
            GitStatus::Untracked => 1,
            GitStatus::Clean => 0,
        }
    }
}

/// 자식 노드들의 GitStatus 를 roll-up 하여 가장 높은 우선순위 상태를 반환한다 (REQ-FE-020).
///
/// 빈 슬라이스가 전달되면 Clean 을 반환한다.
pub fn roll_up_priority(children: &[GitStatus]) -> GitStatus {
    children
        .iter()
        .max_by_key(|s| s.priority())
        .copied()
        .unwrap_or(GitStatus::Clean)
}

// ============================================================
// GitStatusError — GitStatusProvider 오류 타입
// ============================================================

/// GitStatusProvider::status_map 호출 실패 오류 (REQ-FE-023).
#[derive(Debug, thiserror::Error)]
pub enum GitStatusError {
    /// git 저장소가 아닌 경로
    #[error("git 저장소가 아님: {0}")]
    NotARepo(PathBuf),
    /// git 내부 오류
    #[error("git 오류: {0}")]
    Git(String),
}

// ============================================================
// GitStatusProvider — trait (REQ-FE-021)
// ============================================================

/// git status 를 조회하는 추상 인터페이스.
///
/// SPEC-V3-008 진행 시 캐싱/invalidation 가진 별도 구현체를 주입할 수 있도록
/// trait 으로 추상화한다.
pub trait GitStatusProvider: Send + Sync {
    /// repo_root 기준 파일별 git 상태 맵을 반환한다.
    ///
    /// 키: 저장소 루트 기준 상대 경로 문자열 (예: "src/main.rs")
    /// 값: GitStatus enum
    fn status_map(
        &self,
        repo_root: &Path,
    ) -> Result<HashMap<String, GitStatus>, GitStatusError>;
}

// ============================================================
// MoaiGitStatusProvider — 기본 구현체 (moai_git::GitRepo 래퍼)
// ============================================================

/// moai_git::GitRepo::status_map() 을 호출하는 기본 GitStatusProvider 구현체.
///
/// String 라벨 ("modified", "added", "deleted", "untracked") 을
/// GitStatus enum 으로 매핑한다.
pub struct MoaiGitStatusProvider;

impl GitStatusProvider for MoaiGitStatusProvider {
    fn status_map(
        &self,
        repo_root: &Path,
    ) -> Result<HashMap<String, GitStatus>, GitStatusError> {
        // moai_git::GitRepo 를 열어서 status_map() 호출
        let repo = moai_git::GitRepo::open(repo_root)
            .map_err(|e| GitStatusError::Git(e.to_string()))?;

        let raw_map = repo
            .status_map()
            .map_err(|e| GitStatusError::Git(e.to_string()))?;

        // String 라벨 → GitStatus 매핑
        let mapped: HashMap<String, GitStatus> = raw_map
            .into_iter()
            .map(|(path, label)| {
                let status = map_label_to_status(&label);
                (path, status)
            })
            .collect();

        Ok(mapped)
    }
}

/// moai_git 의 String 라벨을 GitStatus 로 매핑한다.
fn map_label_to_status(label: &str) -> GitStatus {
    match label {
        "modified" => GitStatus::Modified,
        "added" => GitStatus::Added,
        "deleted" => GitStatus::Deleted,
        "untracked" => GitStatus::Untracked,
        "renamed" => GitStatus::Renamed,
        "conflicted" => GitStatus::Conflicted,
        _ => GitStatus::Clean,
    }
}

// ============================================================
// 단위 테스트 — AC-FE-8
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // AC-FE-8: roll_up_priority 우선순위 순서 검증
    #[test]
    fn roll_up_priority_returns_conflicted_as_highest() {
        let statuses = vec![
            GitStatus::Clean,
            GitStatus::Untracked,
            GitStatus::Conflicted,
            GitStatus::Modified,
        ];
        assert_eq!(
            roll_up_priority(&statuses),
            GitStatus::Conflicted,
            "Conflicted 가 최우선 순위여야 한다"
        );
    }

    #[test]
    fn roll_up_priority_modified_beats_added_deleted_untracked() {
        let statuses = vec![
            GitStatus::Untracked,
            GitStatus::Added,
            GitStatus::Deleted,
            GitStatus::Modified,
        ];
        assert_eq!(roll_up_priority(&statuses), GitStatus::Modified);
    }

    #[test]
    fn roll_up_priority_added_beats_deleted_renamed_untracked() {
        let statuses = vec![
            GitStatus::Renamed,
            GitStatus::Untracked,
            GitStatus::Deleted,
            GitStatus::Added,
        ];
        assert_eq!(roll_up_priority(&statuses), GitStatus::Added);
    }

    #[test]
    fn roll_up_priority_empty_returns_clean() {
        assert_eq!(roll_up_priority(&[]), GitStatus::Clean, "빈 슬라이스 → Clean");
    }

    #[test]
    fn roll_up_priority_all_clean_returns_clean() {
        let statuses = vec![GitStatus::Clean, GitStatus::Clean];
        assert_eq!(roll_up_priority(&statuses), GitStatus::Clean);
    }

    #[test]
    fn roll_up_priority_single_modified() {
        assert_eq!(
            roll_up_priority(&[GitStatus::Modified, GitStatus::Untracked, GitStatus::Clean]),
            GitStatus::Modified
        );
    }

    // AC-FE-8: 우선순위 순서 전체 커버리지 테스트
    #[test]
    fn priority_order_is_conflicted_modified_added_deleted_renamed_untracked_clean() {
        // Conflicted > Modified
        assert!(GitStatus::Conflicted.priority() > GitStatus::Modified.priority());
        // Modified > Added
        assert!(GitStatus::Modified.priority() > GitStatus::Added.priority());
        // Added > Deleted
        assert!(GitStatus::Added.priority() > GitStatus::Deleted.priority());
        // Deleted > Renamed
        assert!(GitStatus::Deleted.priority() > GitStatus::Renamed.priority());
        // Renamed > Untracked
        assert!(GitStatus::Renamed.priority() > GitStatus::Untracked.priority());
        // Untracked > Clean
        assert!(GitStatus::Untracked.priority() > GitStatus::Clean.priority());
    }

    // AC-FE-8: label 매핑 검증
    #[test]
    fn map_label_to_status_all_variants() {
        assert_eq!(map_label_to_status("modified"), GitStatus::Modified);
        assert_eq!(map_label_to_status("added"), GitStatus::Added);
        assert_eq!(map_label_to_status("deleted"), GitStatus::Deleted);
        assert_eq!(map_label_to_status("untracked"), GitStatus::Untracked);
        assert_eq!(map_label_to_status("renamed"), GitStatus::Renamed);
        assert_eq!(map_label_to_status("conflicted"), GitStatus::Conflicted);
        // 알 수 없는 라벨 → Clean fallback
        assert_eq!(map_label_to_status("unknown_label"), GitStatus::Clean);
        assert_eq!(map_label_to_status(""), GitStatus::Clean);
    }

    // AC-FE-8: MoaiGitStatusProvider — tempdir git init + 파일 추가 → Untracked 매핑 검증
    #[test]
    fn moai_git_status_provider_detects_untracked_file() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        // git init
        let _repo = moai_git::GitRepo::init(dir.path()).expect("git init 실패");
        // 파일 생성 (untracked)
        fs::write(dir.path().join("hello.txt"), b"world").expect("파일 쓰기 실패");

        let provider = MoaiGitStatusProvider;
        let map = provider
            .status_map(dir.path())
            .expect("status_map 호출 실패");

        // "hello.txt" 키가 Untracked 또는 Added (moai_git 는 WT_NEW를 untracked로 반환)
        let status = map.get("hello.txt").copied().unwrap_or(GitStatus::Clean);
        assert!(
            status == GitStatus::Untracked || status == GitStatus::Added,
            "새 파일은 Untracked 또는 Added 여야 한다, 실제: {:?}",
            status
        );
    }

    // AC-FE-8: status_map 실패 시 graceful — GitStatusError 반환 (no panic)
    #[test]
    fn moai_git_status_provider_non_repo_returns_error() {
        let dir = tempfile::tempdir().expect("tempdir 생성 실패");
        // git init 하지 않음 → 저장소 아닌 경로
        let provider = MoaiGitStatusProvider;
        let result = provider.status_map(dir.path());
        assert!(result.is_err(), "git 저장소 아닌 경로 → error 반환");
    }
}
