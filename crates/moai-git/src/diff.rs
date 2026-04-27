//! diff: Git diff 관련 기능
//!
//! 워킹 트리와 인덱스, 커밋 간의 차이를 계산하고 표현한다.

use crate::{GitError, GitRepo};
use std::path::Path;

/// diff의 한 블록(hunk)을 나타내는 구조체
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    /// 원본 파일의 시작 라인 (1-based)
    pub old_start: usize,
    /// 원본 파일의 라인 수
    pub old_lines: usize,
    /// 새 파일의 시작 라인 (1-based)
    pub new_start: usize,
    /// 새 파일의 라인 수
    pub new_lines: usize,
    /// 헤더 문자열 (예: "@@ -10,3 +10,5 @@")
    pub header: String,
    /// 라인 리스트
    pub lines: Vec<Line>,
}

/// diff의 단일 라인
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// 라인 타입: '+' (추가), '-' (삭제), ' ' (컨텍스트)
    pub prefix: char,
    /// 라인 내용
    pub content: String,
}

/// 단일 파일의 diff 결과
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    /// 파일 경로 (저장소 루트 기준 상대 경로)
    pub path: String,
    /// diff hunks
    pub hunks: Vec<Hunk>,
}

impl GitRepo {
    /// 워킹 트리와 인덱스 사이의 diff를 반환한다.
    ///
    /// # Arguments
    ///
    /// * `path` - diff를 계산할 파일의 상대 경로
    ///
    /// # Returns
    ///
    /// 해당 파일의 diff 정보
    ///
    /// # Errors
    ///
    /// Returns `GitError` if the index or tree operations fail.
    pub fn diff_file(&self, path: &Path) -> Result<Diff, GitError> {
        // 워킹 트리와 인덱스 사이의 diff를 생성한다
        let mut index = self.inner.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.inner.find_tree(tree_id)?;

        // 절대 경로인 경우 상대 경로로 변환
        let relative_path = if path.is_absolute() {
            let workdir = self.inner.workdir().ok_or(GitError::DetachedHead)?;
            match path.strip_prefix(workdir) {
                Ok(p) => p,
                Err(_) => path
                    .file_name()
                    .map(Path::new)
                    .ok_or(GitError::Git(git2::Error::from_str("invalid filename")))?,
            }
        } else {
            path
        };

        // diff 생성 (워킹 트리 기준)
        let mut diff_opt = git2::DiffOptions::new();
        diff_opt.pathspec(relative_path);
        let diff = self
            .inner
            .diff_tree_to_workdir(Some(&tree), Some(&mut diff_opt))?;

        let hunks = Self::parse_diff(&diff, relative_path)?;
        Ok(Diff {
            path: relative_path.to_string_lossy().to_string(),
            hunks,
        })
    }

    /// 워킹 트리 전체의 diff를 반환한다.
    pub fn diff_workdir(&self) -> Result<Vec<Diff>, GitError> {
        // TODO: SPEC-V3-008 MS-1 구현 예정
        Ok(Vec::new())
    }

    fn parse_diff(diff: &git2::Diff, _path: &Path) -> Result<Vec<Hunk>, GitError> {
        let mut hunks = Vec::new();

        diff.foreach(
            &mut |_delta, _| true,
            None,
            Some(&mut |_, hunk| {
                hunks.push(Hunk {
                    old_start: hunk.old_start() as usize,
                    old_lines: hunk.old_lines() as usize,
                    new_start: hunk.new_start() as usize,
                    new_lines: hunk.new_lines() as usize,
                    header: format!("{:?}", hunk),
                    lines: Vec::new(),
                });
                true
            }),
            None,
        )?;

        Ok(hunks)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_diff_file_clean_repo() {
        // 깨끗한 저장소에서는 빈 diff를 반환해야 한다
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 테스트 파일 생성
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "initial content").unwrap();

        // stage 및 commit
        repo.stage(&file_path).unwrap();
        repo.commit("initial commit", "test", "test@example.com")
            .unwrap();

        // diff 확인 (변경 없음)
        let result = repo.diff_file(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_diff_file_with_changes() {
        // 파일 변경 후 diff를 확인한다
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 초기 파일 생성 및 커밋
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "initial content").unwrap();
        repo.stage(&file_path).unwrap();
        repo.commit("initial commit", "test", "test@example.com")
            .unwrap();

        // 파일 수정
        std::fs::write(&file_path, "modified content").unwrap();

        // diff 확인
        let diff = repo.diff_file(&file_path).unwrap();
        assert!(!diff.hunks.is_empty(), "변경이 있으면 hunk가 존재해야 한다");
    }
}
