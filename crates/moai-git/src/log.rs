//! log: Git 로그 및 diff 기능
//!
//! 커밋 로그 조회 및 커밋 간 diff 계산 기능을 제공한다.

use crate::{GitError, commit::CommitInfo};
use git2::Oid;

impl crate::GitRepo {
    /// 커밋 간의 diff를 반환한다.
    ///
    /// # Arguments
    ///
    /// * `oid` - diff를 계산할 커밋의 OID
    ///
    /// # Errors
    ///
    /// Returns `GitError` if the OID is invalid or the commit/tree lookup fails.
    pub fn diff_commit(&self, oid: &str) -> Result<crate::diff::Diff, GitError> {
        let commit_oid = Oid::from_str(oid)?;
        let commit = self.inner.find_commit(commit_oid)?;

        if commit.parent_count() > 0 {
            let parent = commit.parent(0)?;
            let tree_a = parent.tree()?;
            let tree_b = commit.tree()?;
            let diff = self
                .inner
                .diff_tree_to_tree(Some(&tree_a), Some(&tree_b), None)?;

            // 첫 번째 파일의 diff만 반환 (간단 구현)
            let mut hunks = Vec::new();
            let mut path = String::new();

            diff.foreach(
                &mut |delta, _| {
                    if let Some(p) = delta.new_file().path() {
                        path = p.to_string_lossy().to_string();
                    }
                    true
                },
                None,
                Some(&mut |_, hunk| {
                    use crate::diff::Hunk;
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

            Ok(crate::diff::Diff { path, hunks })
        } else {
            // 초기 커밋인 경우 빈 diff
            Ok(crate::diff::Diff {
                path: String::new(),
                hunks: Vec::new(),
            })
        }
    }

    /// 커밋 정보를 상세히 조회한다.
    ///
    /// # Arguments
    ///
    /// * `oid` - 조회할 커밋의 OID
    ///
    /// # Errors
    ///
    /// Returns `GitError` if the OID is invalid or the commit does not exist.
    pub fn show_commit(&self, oid: &str) -> Result<CommitInfo, GitError> {
        let commit_oid = Oid::from_str(oid)?;
        let commit = self.inner.find_commit(commit_oid)?;
        let author = commit.author();

        Ok(CommitInfo {
            short_id: format!("{:.7}", commit_oid),
            oid: commit_oid.to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: author.name().unwrap_or("").to_string(),
            email: author.email().unwrap_or("").to_string(),
            time: commit.time().seconds(),
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_diff_commit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 첫 번째 커밋
        let file1 = temp_dir.path().join("file1.txt");
        std::fs::write(&file1, "initial content").unwrap();
        repo.stage(&file1).unwrap();
        let _oid1 = repo
            .commit("first commit", "Test", "test@example.com")
            .unwrap();

        // 두 번째 커밋
        std::fs::write(&file1, "modified content").unwrap();
        repo.stage(&file1).unwrap();
        let oid2 = repo
            .commit("second commit", "Test", "test@example.com")
            .unwrap();

        // 두 번째 커밋의 diff 확인
        let diff = repo.diff_commit(&oid2).unwrap();
        assert!(!diff.path.is_empty() || !diff.hunks.is_empty());
    }

    #[test]
    fn test_show_commit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 커밋 생성
        let file1 = temp_dir.path().join("file1.txt");
        std::fs::write(&file1, "content").unwrap();
        repo.stage(&file1).unwrap();
        let oid = repo
            .commit("test message", "Test User", "test@example.com")
            .unwrap();

        // 커밋 정보 조회
        let info = repo.show_commit(&oid).unwrap();
        assert_eq!(info.message, "test message");
        assert_eq!(info.author, "Test User");
        assert_eq!(info.email, "test@example.com");
    }
}
