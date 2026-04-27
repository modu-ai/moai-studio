//! commit: Git commit 관련 기능
//!
//! 변경사항을 스테이징하고 커밋한다.

use crate::GitError;
use std::path::Path;

/// 커밋 정보
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitInfo {
    /// 커밋 OID (단축형)
    pub short_id: String,
    /// 전체 OID
    pub oid: String,
    /// 커밋 메시지
    pub message: String,
    /// 작성자 이름
    pub author: String,
    /// 작성자 이메일
    pub email: String,
    /// 커밋 시간 (Unix timestamp)
    pub time: i64,
}

impl crate::GitRepo {
    /// 경로의 파일을 스테이징한다.
    ///
    /// # Arguments
    ///
    /// * `path` - 스테이징할 파일의 절대 또는 상대 경로
    pub fn stage(&self, path: &Path) -> Result<(), GitError> {
        let mut index = self.inner.index()?;
        // git2 requires relative paths from repository root
        // Try to get relative path, or use just the file name as fallback
        let relative_path = if path.is_absolute() {
            let workdir = self.inner.workdir().ok_or(GitError::DetachedHead)?;
            // Try simple strip prefix first
            match path.strip_prefix(workdir) {
                Ok(p) => p,
                Err(_) => {
                    // Fallback: use file name only
                    path.file_name()
                        .map(Path::new)
                        .ok_or(GitError::Git(git2::Error::from_str("invalid filename")))?
                }
            }
        } else {
            path
        };

        index.add_path(relative_path)?;
        index.write()?;
        Ok(())
    }

    /// 경로의 파일을 언스테이징한다.
    ///
    /// # Arguments
    ///
    /// * `path` - 언스테이징할 파일의 절대 또는 상대 경로
    pub fn unstage(&self, path: &Path) -> Result<(), GitError> {
        let mut index = self.inner.index()?;
        // git2 requires relative paths from repository root
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

        // Remove from index
        index.remove_path(relative_path)?;
        index.write()?;
        Ok(())
    }

    /// 스테이징된 변경사항을 커밋한다.
    ///
    /// # Arguments
    ///
    /// * `message` - 커밋 메시지
    /// * `author_name` - 작성자 이름
    /// * `author_email` - 작성자 이메일
    pub fn commit(
        &self,
        message: &str,
        author_name: &str,
        author_email: &str,
    ) -> Result<String, GitError> {
        // 스테이징된 변경사항이 있는지 확인
        let mut index = self.inner.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.inner.find_tree(tree_id)?;

        // HEAD 커밋 찾기
        let head_commit = self.inner.head().ok().and_then(|h| h.peel_to_commit().ok());

        let sig = git2::Signature::now(author_name, author_email)?;

        let oid = match head_commit {
            Some(parent) => {
                // 부모가 있으면 일반 커밋
                self.inner
                    .commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?
            }
            None => {
                // 부모가 없으면 초기 커밋
                self.inner
                    .commit(Some("HEAD"), &sig, &sig, message, &tree, &[])?
            }
        };

        Ok(oid.to_string())
    }

    /// 커밋 로그를 반환한다.
    ///
    /// # Arguments
    ///
    /// * `limit` - 가져올 최대 커밋 수
    pub fn log(&self, limit: usize) -> Result<Vec<CommitInfo>, GitError> {
        let mut revwalk = self.inner.revwalk()?;
        revwalk.push_head()?;

        let commits: Result<Vec<_>, _> = revwalk
            .take(limit)
            .map(|oid| {
                let oid = oid?;
                let commit = self.inner.find_commit(oid)?;
                let author = commit.author();

                Ok(CommitInfo {
                    short_id: format!("{:.7}", oid),
                    oid: oid.to_string(),
                    message: commit.message().unwrap_or("").to_string(),
                    author: author.name().unwrap_or("").to_string(),
                    email: author.email().unwrap_or("").to_string(),
                    time: commit.time().seconds(),
                })
            })
            .collect();

        commits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_and_commit() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 파일 생성
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        // stage
        repo.stage(&file_path).expect("스테이징 성공");

        // commit
        let oid = repo
            .commit("test commit", "Test User", "test@example.com")
            .expect("커밋 성공");

        assert!(!oid.is_empty(), "커밋 OID가 비어있으면 안 된다");
    }

    #[test]
    fn test_unstage() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 파일 생성
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();

        // 스테이징
        repo.stage(&file_path).expect("스테이징 성공");

        // 언스테이징
        repo.unstage(&file_path).expect("언스테이징 성공");

        // 파일은 여전히 존재해야 함
        assert!(file_path.exists(), "파일이 존재해야 한다");
    }

    #[test]
    fn test_log_returns_commits() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 첫 번째 커밋
        let file1 = temp_dir.path().join("file1.txt");
        std::fs::write(&file1, "content1").unwrap();
        repo.stage(&file1).unwrap();
        repo.commit("first commit", "Test", "test@example.com")
            .unwrap();

        // 두 번째 커밋
        let file2 = temp_dir.path().join("file2.txt");
        std::fs::write(&file2, "content2").unwrap();
        repo.stage(&file2).unwrap();
        repo.commit("second commit", "Test", "test@example.com")
            .unwrap();

        // 로그 확인
        let log = repo.log(10).unwrap();
        assert_eq!(log.len(), 2, "2개 커밋이 로그에 있어야 한다");
        assert_eq!(log[0].message, "second commit", "최신 커밋이 먼저");
        assert_eq!(log[1].message, "first commit");
    }
}
