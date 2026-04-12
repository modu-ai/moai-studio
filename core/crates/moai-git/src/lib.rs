//! moai-git: git2 기반 Git 작업 래퍼
//!
//! 워크스페이스 관리를 위한 Git 작업(워크트리, 상태, 브랜치)을 제공한다.

use std::path::Path;
use thiserror::Error;

/// Git 작업 오류
#[derive(Debug, Error)]
pub enum GitError {
    /// git2 라이브러리 오류
    #[error("Git 오류: {0}")]
    Git(#[from] git2::Error),

    /// HEAD가 브랜치를 가리키지 않음
    #[error("현재 브랜치를 결정할 수 없음: HEAD가 detached 상태임")]
    DetachedHead,
}

/// 워크스페이스의 Git 상태 요약
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStatus {
    /// 수정된 파일 수
    pub modified: usize,
    /// 추가된 파일 수 (스테이지드 포함)
    pub added: usize,
    /// 삭제된 파일 수
    pub deleted: usize,
}

/// git2::Repository를 감싸는 구조체
pub struct GitRepo {
    inner: git2::Repository,
}

impl GitRepo {
    /// 기존 Git 저장소를 연다.
    pub fn open(path: &Path) -> Result<Self, GitError> {
        let repo = git2::Repository::open(path)?;
        Ok(Self { inner: repo })
    }

    /// 새 Git 저장소를 초기화한다.
    pub fn init(path: &Path) -> Result<Self, GitError> {
        let repo = git2::Repository::init(path)?;
        Ok(Self { inner: repo })
    }

    /// 현재 HEAD 브랜치 이름을 반환한다.
    ///
    /// HEAD가 detached 상태이면 `GitError::DetachedHead`를 반환한다.
    pub fn current_branch(&self) -> Result<String, GitError> {
        let head = self.inner.head()?;
        head.shorthand()
            .map(|s| s.to_string())
            .ok_or(GitError::DetachedHead)
    }

    /// 커밋되지 않은 변경사항(워킹 트리 또는 인덱스)이 있으면 `true`를 반환한다.
    pub fn is_dirty(&self) -> Result<bool, GitError> {
        let statuses = self.inner.statuses(None)?;
        let dirty = statuses.iter().any(|s| {
            // Ignored 파일은 제외하고 실제 변경사항만 확인한다.
            !s.status().is_ignored()
        });
        Ok(dirty)
    }

    /// 변경된/추가된/삭제된 파일 수를 집계하여 반환한다.
    pub fn status_summary(&self) -> Result<GitStatus, GitError> {
        let statuses = self.inner.statuses(None)?;

        let mut modified = 0usize;
        let mut added = 0usize;
        let mut deleted = 0usize;

        for entry in statuses.iter() {
            let s = entry.status();
            if s.is_ignored() {
                continue;
            }
            // 인덱스(스테이지드) 상태 집계
            if s.contains(git2::Status::INDEX_NEW) || s.contains(git2::Status::WT_NEW) {
                added += 1;
            } else if s.contains(git2::Status::INDEX_DELETED)
                || s.contains(git2::Status::WT_DELETED)
            {
                deleted += 1;
            } else if s.contains(git2::Status::INDEX_MODIFIED)
                || s.contains(git2::Status::WT_MODIFIED)
            {
                modified += 1;
            }
        }

        Ok(GitStatus {
            modified,
            added,
            deleted,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// 테스트용 임시 Git 저장소를 초기화하는 헬퍼
    fn init_test_repo() -> (tempfile::TempDir, GitRepo) {
        let dir = tempdir().unwrap();
        let repo = GitRepo::init(dir.path()).unwrap();
        (dir, repo)
    }

    #[test]
    fn test_init_repo() {
        // 임시 디렉터리에 저장소를 초기화할 수 있어야 한다.
        let dir = tempdir().unwrap();
        let result = GitRepo::init(dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_current_branch_after_first_commit() {
        // 첫 번째 커밋 후 브랜치 이름을 읽을 수 있어야 한다.
        let dir = tempdir().unwrap();
        let repo_raw = git2::Repository::init(dir.path()).unwrap();

        // 커밋 없이는 HEAD가 unborn이므로 파일을 커밋해야 한다.
        let readme = dir.path().join("README.md");
        fs::write(&readme, b"hello").unwrap();

        let mut index = repo_raw.index().unwrap();
        index.add_path(Path::new("README.md")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo_raw.find_tree(tree_id).unwrap();

        let sig = git2::Signature::now("test", "test@example.com").unwrap();
        repo_raw
            .commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
            .unwrap();

        let repo = GitRepo::open(dir.path()).unwrap();
        let branch = repo.current_branch().unwrap();
        // git 기본 브랜치는 'master' 또는 'main'이다.
        assert!(branch == "master" || branch == "main", "브랜치: {}", branch);
    }

    #[test]
    fn test_clean_repo_not_dirty() {
        // 아무 변경도 없는 저장소는 dirty하지 않아야 한다.
        let (_dir, repo) = init_test_repo();

        // 빈 저장소(커밋 없음)도 is_dirty()를 지원해야 한다.
        let dirty = repo.is_dirty().unwrap();
        assert!(!dirty, "빈 저장소는 dirty하지 않아야 한다");
    }

    #[test]
    fn test_dirty_after_file_create() {
        // 파일을 생성하면 저장소가 dirty 상태가 되어야 한다.
        let (dir, repo) = init_test_repo();

        // 추적되지 않은 파일 생성
        let new_file = dir.path().join("hello.txt");
        fs::write(&new_file, b"world").unwrap();

        let dirty = repo.is_dirty().unwrap();
        assert!(dirty, "파일 생성 후 dirty 상태여야 한다");
    }

    #[test]
    fn test_status_summary_counts() {
        // 파일 생성 후 status_summary가 올바른 카운트를 반환해야 한다.
        let (dir, repo) = init_test_repo();

        // 추적되지 않은 파일 2개 생성
        fs::write(dir.path().join("file1.txt"), b"a").unwrap();
        fs::write(dir.path().join("file2.txt"), b"b").unwrap();

        let summary = repo.status_summary().unwrap();
        // 새 파일은 added로 집계된다.
        assert_eq!(summary.added, 2, "새 파일 2개는 added여야 한다");
        assert_eq!(summary.modified, 0);
        assert_eq!(summary.deleted, 0);
    }

    #[test]
    fn test_open_nonexistent_repo_fails() {
        // 존재하지 않는 경로 열기는 오류를 반환해야 한다.
        let result = GitRepo::open(Path::new("/tmp/nonexistent-moai-git-repo-xyz"));
        assert!(result.is_err());
    }
}
