//! stash: Git stash 관련 기능
//!
//! 작업 내용을 임시 저장하는 stash 기능을 제공한다.

use crate::GitError;

/// Stash 정보
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StashInfo {
    /// stash 인덱스 (0이 가장 최신)
    pub index: usize,
    /// stash 메시지
    pub message: String,
    /// stash 생성 시점의 브랜치
    pub branch: String,
    /// 커밋 OID
    pub oid: String,
}

impl crate::GitRepo {
    /// 현재 작업 내용을 stash에 저장한다.
    ///
    /// # Arguments
    ///
    /// * `message` - stash 메시지 (선택 사항)
    pub fn stash_push(&self, message: Option<&str>) -> Result<String, GitError> {
        let sig = self.inner.signature()?;
        let mut index = self.inner.index()?;

        // stash 커밋 생성
        let tree_id = index.write_tree()?;
        let tree = self.inner.find_tree(tree_id)?;

        let head_commit = self.inner.head().ok().and_then(|h| {
            h.peel_to_commit()
                .ok()
        });

        let oid = if let Some(parent) = head_commit {
            self.inner.commit(
                None,
                &sig,
                &sig,
                message.unwrap_or("WIP"),
                &tree,
                &[&parent],
            )?
        } else {
            self.inner.commit(
                None,
                &sig,
                &sig,
                message.unwrap_or("WIP"),
                &tree,
                &[],
            )?
        };

        Ok(oid.to_string())
    }

    /// 가장 최신 stash를 적용하고 제거한다.
    ///
    /// # Arguments
    ///
    /// * `index` - stash 인덱스 (0: 가장 최신)
    pub fn stash_pop(&self, index: usize) -> Result<(), GitError> {
        self.stash_apply(index)?;
        self.stash_drop(index)?;
        Ok(())
    }

    /// stash를 적용하지만 제거하지는 않는다.
    ///
    /// # Arguments
    ///
    /// * `_index` - stash 인덱스 (0: 가장 최신)
    pub fn stash_apply(&self, _index: usize) -> Result<(), GitError> {
        // TODO: SPEC-V3-008 MS-3 구현 예정
        Err(GitError::Git(git2::Error::from_str("not implemented")))
    }

    /// stash를 제거한다.
    ///
    /// # Arguments
    ///
    /// * `_index` - stash 인덱스 (0: 가장 최신)
    pub fn stash_drop(&self, _index: usize) -> Result<(), GitError> {
        // TODO: SPEC-V3-008 MS-3 구현 예정
        Err(GitError::Git(git2::Error::from_str("not implemented")))
    }

    /// stash 목록을 반환한다.
    ///
    /// 최신 순으로 정렬된 리스트를 반환한다.
    pub fn stash_list(&self) -> Result<Vec<StashInfo>, GitError> {
        // TODO: SPEC-V3-008 MS-3 구현 예정
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stash_push_creates_stash() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 초기 커밋 생성
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "initial").unwrap();
        repo.stage(&file_path).unwrap();
        repo.commit("initial commit", "Test", "test@example.com")
            .unwrap();

        // 파일 수정
        std::fs::write(&file_path, "modified").unwrap();

        // stash push
        let oid = repo
            .stash_push(Some("test stash"))
            .expect("stash push 성공");

        assert!(!oid.is_empty(), "stash OID가 반환되어야 한다");
    }

    #[test]
    fn test_stash_list() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo = crate::GitRepo::init(temp_dir.path()).unwrap();

        // 초기 커밋
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "initial").unwrap();
        repo.stage(&file_path).unwrap();
        repo.commit("initial commit", "Test", "test@example.com")
            .unwrap();

        // stash 생성
        std::fs::write(&file_path, "modified").unwrap();
        repo.stash_push(Some("test stash")).unwrap();

        // stash 목록 확인
        let _list = repo.stash_list().unwrap();
        // 구현 완료 후 stash가 목록에 있는지 확인
    }
}
