//! branch: Git 브랜치 관련 기능
//!
//! 브랜치 목록 조회, 생성, 전환 기능을 제공한다.

use crate::GitError;

/// 브랜치 정보
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchInfo {
    /// 브랜치 이름
    pub name: String,
    /// 현재 HEAD인지 여부
    pub is_head: bool,
    /// 로컬 브랜치 여부 (false면 원격 브랜치)
    pub is_local: bool,
}

impl crate::GitRepo {
    /// 모든 브랜치 목록을 반환한다.
    ///
    /// 로컬 브랜치와 원격 브랜치를 모두 포함한다.
    ///
    /// # Errors
    ///
    /// Returns `GitError` if the branch iteration or HEAD lookup fails.
    pub fn branches(&self) -> Result<Vec<BranchInfo>, GitError> {
        let mut branches = Vec::new();

        // 로컬 브랜치 수집
        for branch in self.inner.branches(Some(git2::BranchType::Local))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                let head_ref = self.inner.head()?;
                let is_head = head_ref.name().map(|n| n.ends_with(name)).unwrap_or(false);

                branches.push(BranchInfo {
                    name: name.to_string(),
                    is_head,
                    is_local: true,
                });
            }
        }

        // 원격 브랜치 수집
        for branch in self.inner.branches(Some(git2::BranchType::Remote))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                branches.push(BranchInfo {
                    name: name.to_string(),
                    is_head: false,
                    is_local: false,
                });
            }
        }

        Ok(branches)
    }

    /// 새 브랜치를 생성한다.
    ///
    /// # Arguments
    ///
    /// * `name` - 생성할 브랜치 이름
    /// * `target` - 시작 지점 (기본값: HEAD)
    ///
    /// # Errors
    ///
    /// Returns `GitError` if the branch name already exists or the target is invalid.
    pub fn create_branch(&self, name: &str, target: Option<&str>) -> Result<(), GitError> {
        let head = self.inner.head()?;
        let target_commit = if let Some(target_oid) = target {
            self.inner.revparse_single(target_oid)?.peel_to_commit()?
        } else {
            head.peel_to_commit()?
        };

        self.inner.branch(name, &target_commit, false)?;
        Ok(())
    }

    /// 지정된 브랜치로 전환한다.
    ///
    /// # Arguments
    ///
    /// * `name` - 전환할 브랜치 이름 (예: "refs/heads/main")
    ///
    /// # Errors
    ///
    /// Returns `GitError` if the branch does not exist or checkout fails.
    pub fn checkout(&self, name: &str) -> Result<(), GitError> {
        let obj = self.inner.revparse_single(name)?;
        self.inner.checkout_tree(&obj, None)?;
        self.inner.set_head(name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_branch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 초기 커밋 생성
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();
        repo.stage(&file_path).unwrap();
        repo.commit("initial commit", "Test", "test@example.com")
            .unwrap();

        // 브랜치 생성
        repo.create_branch("feature/test", None)
            .expect("브랜치 생성 성공");

        // 브랜치 목록 확인
        let branches = repo.branches().unwrap();
        let feature_branch = branches
            .iter()
            .find(|b| b.name == "feature/test")
            .expect("생성한 브랜치가 목록에 있어야 한다");

        assert!(feature_branch.is_local);
        assert!(!feature_branch.is_head);
    }

    #[test]
    fn test_checkout_branch() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 초기 커밋 및 브랜치 생성
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();
        repo.stage(&file_path).unwrap();
        repo.commit("initial commit", "Test", "test@example.com")
            .unwrap();

        repo.create_branch("new-branch", None)
            .expect("브랜치 생성 성공");

        // 체크아웃
        repo.checkout("refs/heads/new-branch")
            .expect("체크아웃 성공");

        // 현재 브랜치 확인
        let current = repo.current_branch().unwrap();
        assert_eq!(current, "new-branch", "브랜치가 전환되어야 한다");
    }

    #[test]
    fn test_branches_includes_head() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 초기 커밋
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "content").unwrap();
        repo.stage(&file_path).unwrap();
        repo.commit("initial commit", "Test", "test@example.com")
            .unwrap();

        // 브랜치 목록에서 현재 HEAD 확인
        let branches = repo.branches().unwrap();
        let head_branch = branches
            .iter()
            .find(|b| b.is_head)
            .expect("현재 HEAD 브랜치가 있어야 한다");

        assert!(head_branch.is_local);
    }
}
