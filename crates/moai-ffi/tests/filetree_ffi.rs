//! FileTree FFI 테스트 (SPEC-M2-001 MS-4 T-056).
//!
//! list_directory_json / git_status_map_json FFI 를 검증한다.

use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// 통합 테스트를 위한 RustCore 초기화 헬퍼
fn make_core() -> moai_ffi::RustCore {
    moai_ffi::RustCore::new()
}

// ── T-056-R1: 디렉토리 리스팅 ────────────────────────────────────────────────

#[test]
fn test_list_directory_json_returns_children() {
    // Arrange: 임시 디렉토리 + 하위 파일 2개, 서브디렉토리 1개 생성
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("readme.md"), "hello").unwrap();
    fs::write(dir.path().join("main.rs"), "fn main(){}").unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();

    let core = make_core();

    // Act
    let json = core.list_directory_json(dir.path().to_str().unwrap().to_string(), "".to_string());

    // Assert: JSON 배열, 3개 항목 포함
    let entries: Vec<serde_json::Value> = serde_json::from_str(&json).expect("JSON 파싱 실패");
    assert_eq!(entries.len(), 3, "파일 2개 + 디렉토리 1개 = 3개");

    // src 는 is_directory == true
    let src_entry = entries.iter().find(|e| e["name"] == "src");
    assert!(src_entry.is_some(), "src 디렉토리 항목 누락");
    assert_eq!(src_entry.unwrap()["is_directory"], true);
}

#[test]
fn test_list_directory_json_skips_hidden_and_build() {
    // Arrange: 필터링 대상 디렉토리 포함
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join(".git")).unwrap();
    fs::create_dir(dir.path().join("target")).unwrap();
    fs::create_dir(dir.path().join("node_modules")).unwrap();
    fs::create_dir(dir.path().join(".build")).unwrap();
    fs::create_dir(dir.path().join("build")).unwrap();
    fs::write(dir.path().join(".DS_Store"), "").unwrap();
    fs::write(dir.path().join("visible.txt"), "ok").unwrap();

    let core = make_core();

    // Act
    let json = core.list_directory_json(dir.path().to_str().unwrap().to_string(), "".to_string());

    let entries: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    // Assert: visible.txt 만 남아야 한다
    assert_eq!(entries.len(), 1, "필터링 후 visible.txt 만 남아야 한다");
    assert_eq!(entries[0]["name"], "visible.txt");
}

// ── T-056-R2: git status 맵 ────────────────────────────────────────────────

#[test]
fn test_git_status_map_empty_for_non_git() {
    // Arrange: git 저장소가 아닌 임시 디렉토리
    let dir = tempdir().unwrap();
    let core = make_core();

    // Act
    let json = core.git_status_map_json(dir.path().to_str().unwrap().to_string());

    // Assert: 빈 객체 "{}" 반환
    let map: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(map.is_object());
    assert_eq!(map.as_object().unwrap().len(), 0);
}

#[test]
fn test_git_status_map_reflects_modified_file() {
    // Arrange: git 저장소 초기화 + 파일 커밋 후 수정
    let dir = tempdir().unwrap();
    let repo = git2::Repository::init(dir.path()).unwrap();

    // 초기 파일 커밋
    let file_path = dir.path().join("hello.txt");
    fs::write(&file_path, "initial").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("hello.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("test", "test@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .unwrap();

    // 파일 수정 (워킹 트리)
    fs::write(&file_path, "modified content").unwrap();

    let core = make_core();

    // Act
    let json = core.git_status_map_json(dir.path().to_str().unwrap().to_string());

    let map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&json).expect("JSON 파싱 실패");

    // Assert: "hello.txt" 의 상태는 "modified"
    assert!(
        map.contains_key("hello.txt"),
        "hello.txt 키 누락: {:?}",
        map
    );
    assert_eq!(map["hello.txt"], "modified");
}

#[test]
fn test_filetree_entry_depth_zero_for_root() {
    // Arrange: 루트 디렉토리 바로 아래 파일 1개
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("root.txt"), "").unwrap();

    let core = make_core();

    // Act
    let json = core.list_directory_json(dir.path().to_str().unwrap().to_string(), "".to_string());

    let entries: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    // Assert: depth == 0 (루트 바로 아래)
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["depth"], 0);
}
