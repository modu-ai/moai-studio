//! T-007: git worktree lifecycle 테스트 (SPEC-M1-001 RG-M1-4).

use moai_git::WorktreeManager;
use tempfile::tempdir;

#[test]
fn create_and_list_worktree() {
    let repo_dir = tempdir().unwrap();
    let wt_root = tempdir().unwrap();
    let mgr = WorktreeManager::init(repo_dir.path()).unwrap();

    let wt_path = wt_root.path().join("ws-alpha");
    let summary = mgr.create_worktree("ws-alpha", &wt_path).unwrap();
    assert_eq!(summary.name, "ws-alpha");
    assert!(
        wt_path.exists(),
        "worktree 디렉터리가 실제로 생성되어야 한다"
    );

    let list = mgr.list_worktrees().unwrap();
    assert!(list.iter().any(|s| s.name == "ws-alpha"));
}

#[test]
fn remove_worktree_cleans_disk() {
    let repo_dir = tempdir().unwrap();
    let wt_root = tempdir().unwrap();
    let mgr = WorktreeManager::init(repo_dir.path()).unwrap();

    let wt_path = wt_root.path().join("ws-to-remove");
    mgr.create_worktree("ws-to-remove", &wt_path).unwrap();
    assert!(wt_path.exists());

    mgr.remove_worktree("ws-to-remove").unwrap();
    assert!(!wt_path.exists(), "remove 이후 디렉터리가 삭제되어야 한다");

    let list = mgr.list_worktrees().unwrap();
    assert!(!list.iter().any(|s| s.name == "ws-to-remove"));
}

#[test]
fn multiple_worktrees_coexist() {
    let repo_dir = tempdir().unwrap();
    let wt_root = tempdir().unwrap();
    let mgr = WorktreeManager::init(repo_dir.path()).unwrap();

    for i in 0..3 {
        let p = wt_root.path().join(format!("ws-{i}"));
        mgr.create_worktree(&format!("ws-{i}"), &p).unwrap();
    }
    let list = mgr.list_worktrees().unwrap();
    assert_eq!(list.len(), 3);
}

#[test]
fn duplicate_name_fails() {
    let repo_dir = tempdir().unwrap();
    let wt_root = tempdir().unwrap();
    let mgr = WorktreeManager::init(repo_dir.path()).unwrap();

    let a = wt_root.path().join("a");
    let b = wt_root.path().join("b");
    mgr.create_worktree("dup", &a).unwrap();
    let res = mgr.create_worktree("dup", &b);
    assert!(res.is_err(), "동일 이름 worktree 는 중복 생성 불가");
}
