//! Git worktree 생명주기 관리 (SPEC-M1-001 RG-M1-4, T-007).
//!
//! workspace 별 독립 worktree 를 생성/제거/조회한다. `git2::Repository::worktree`
//! API 를 직접 사용하며, head 가 unborn 상태이면 자동으로 빈 초기 커밋을 만든다.

// @MX:ANCHOR: [AUTO] Workspace-level git isolation 진입점 (fan_in>=3: supervisor/store/ui)
// @MX:REASON: [AUTO] SPEC-M1-001 은 workspace 당 독립 worktree 를 요구한다. 이 모듈이 단일 소스다.

use std::path::{Path, PathBuf};

use crate::GitError;

/// 조회된 worktree 요약.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorktreeSummary {
    /// worktree 이름 (프로젝트 내 고유)
    pub name: String,
    /// worktree 체크아웃 경로
    pub path: PathBuf,
}

/// 기존 저장소 위에서 worktree 조작을 수행하는 파사드.
pub struct WorktreeManager {
    repo: git2::Repository,
}

impl WorktreeManager {
    /// 기존 저장소를 열어 manager 를 생성한다.
    pub fn open(repo_path: &Path) -> Result<Self, GitError> {
        let repo = git2::Repository::open(repo_path)?;
        Ok(Self { repo })
    }

    /// 새 저장소를 init 하고 manager 를 생성한다 (테스트 편의용).
    pub fn init(repo_path: &Path) -> Result<Self, GitError> {
        let repo = git2::Repository::init(repo_path)?;
        Ok(Self { repo })
    }

    /// HEAD 가 unborn 이면 빈 트리로 초기 커밋을 만든다. worktree 생성은 HEAD 가
    /// 실제 커밋을 가리켜야만 가능하다.
    fn ensure_head(&self) -> Result<(), GitError> {
        if self.repo.head().is_ok() {
            return Ok(());
        }
        let sig = git2::Signature::now("moai", "moai@local")?;
        let tree_id = {
            let mut idx = self.repo.index()?;
            idx.write_tree()?
        };
        let tree = self.repo.find_tree(tree_id)?;
        self.repo
            .commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])?;
        Ok(())
    }

    /// 새 worktree 를 생성한다.
    ///
    /// * `name`: worktree 식별자 (브랜치와 동일하게 사용됨)
    /// * `path`: worktree 체크아웃 대상 경로 (존재하지 않아야 함)
    pub fn create_worktree(&self, name: &str, path: &Path) -> Result<WorktreeSummary, GitError> {
        self.ensure_head()?;
        let opts = git2::WorktreeAddOptions::new();
        let wt = self.repo.worktree(name, path, Some(&opts))?;
        Ok(WorktreeSummary {
            name: wt.name().unwrap_or(name).to_string(),
            path: wt.path().to_path_buf(),
        })
    }

    /// 지정한 worktree 를 제거한다 (administrative prune — 디스크 디렉터리도 삭제).
    pub fn remove_worktree(&self, name: &str) -> Result<(), GitError> {
        let wt = self.repo.find_worktree(name)?;
        let path = wt.path().to_path_buf();
        // prune 은 먼저 worktree 행정 레코드를 제거한다.
        let mut prune_opts = git2::WorktreePruneOptions::new();
        prune_opts.valid(true).working_tree(true);
        wt.prune(Some(&mut prune_opts))?;
        // 디스크 디렉터리가 남아있으면 best-effort 로 삭제한다.
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
        Ok(())
    }

    /// 현재 등록된 worktree 목록을 반환한다.
    pub fn list_worktrees(&self) -> Result<Vec<WorktreeSummary>, GitError> {
        let names = self.repo.worktrees()?;
        let mut out = Vec::new();
        for maybe_name in names.iter() {
            let Some(name) = maybe_name else { continue };
            if let Ok(wt) = self.repo.find_worktree(name) {
                out.push(WorktreeSummary {
                    name: name.to_string(),
                    path: wt.path().to_path_buf(),
                });
            }
        }
        Ok(out)
    }
}
