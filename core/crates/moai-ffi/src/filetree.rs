//! FileTree FFI 함수 (SPEC-M2-001 MS-4 T-050).
//!
//! Swift FileTreeSurface 를 위한 디렉토리 리스팅 및 git status 조회 FFI.
//!
//! @MX:NOTE: [AUTO] 폴링 기반 갱신 채택. MS-7+ 에서 notify-push 로 업그레이드 예정.
//!            Swift 측 500ms 타이머가 list_directory_json 을 반복 호출한다.

use std::path::Path;

/// 필터링 대상 디렉토리/파일 이름 (숨김 + 빌드 아티팩트)
const SKIP_NAMES: &[&str] = &[
    ".git",
    ".DS_Store",
    "node_modules",
    "target",
    ".build",
    "build",
];

fn should_skip(name: &str) -> bool {
    SKIP_NAMES.contains(&name)
}

/// 디렉토리 바로 아래 항목을 JSON 배열로 반환한다.
///
/// # 인수
/// - `workspace_path`: 워크스페이스 루트 절대 경로
/// - `subpath`: 루트 기준 상대 경로 (빈 문자열이면 루트 직접 리스팅)
///
/// # JSON 스키마
/// ```json
/// [{"path":"src","name":"src","is_directory":true,"git_status":"clean","depth":0}, ...]
/// ```
///
// @MX:ANCHOR: [AUTO] Swift FileTreeViewModel.load() / refresh() 의 유일한 디렉토리 데이터 소스
// @MX:REASON: [AUTO] FileTreeViewModel, FileTreeViewModelTests, SurfaceRouter 세 경로에서 호출 (fan_in>=3)
pub(crate) fn list_directory_json(workspace_path: String, subpath: String) -> String {
    let root = Path::new(&workspace_path);
    let target = if subpath.is_empty() {
        root.to_path_buf()
    } else {
        root.join(&subpath)
    };

    // depth: subpath 의 경로 구분자 수로 계산 (루트 = 0)
    let depth: i64 = if subpath.is_empty() {
        0
    } else {
        subpath.matches('/').count() as i64 + 1
    };

    let read_result = std::fs::read_dir(&target);
    let Ok(rd) = read_result else {
        return "[]".to_string();
    };

    let mut entries: Vec<serde_json::Value> = Vec::new();

    for entry in rd.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // 필터링
        if should_skip(&name_str) {
            continue;
        }

        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

        // path: workspace_root 기준 상대 경로
        let relative = if subpath.is_empty() {
            name_str.to_string()
        } else {
            format!("{}/{}", subpath, name_str)
        };

        entries.push(serde_json::json!({
            "path": relative,
            "name": name_str.to_string(),
            "is_directory": is_dir,
            "git_status": "clean",  // git_status 는 별도 git_status_map_json 으로 덮어쓴다
            "depth": depth,
        }));
    }

    // 디렉토리 먼저, 그 다음 파일. 이름 알파벳 순 정렬.
    entries.sort_by(|a, b| {
        let a_is_dir = a["is_directory"].as_bool().unwrap_or(false);
        let b_is_dir = b["is_directory"].as_bool().unwrap_or(false);
        if a_is_dir != b_is_dir {
            // 디렉토리가 먼저
            b_is_dir.cmp(&a_is_dir)
        } else {
            let a_name = a["name"].as_str().unwrap_or("");
            let b_name = b["name"].as_str().unwrap_or("");
            a_name.cmp(b_name)
        }
    });

    serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string())
}

/// 워크스페이스 루트의 git status 맵을 JSON 객체로 반환한다.
///
/// # 반환 형식
/// ```json
/// {"src/main.rs": "modified", "new_file.txt": "untracked"}
/// ```
/// git 저장소가 아니면 빈 객체 `{}` 반환.
///
// @MX:NOTE: [AUTO] moai-git GitRepo::status_map() 을 호출한다.
//           FileTreeViewModel.load() 시 한 번, refresh() 시 재호출.
pub(crate) fn git_status_map_json(workspace_path: String) -> String {
    let path = Path::new(&workspace_path);
    let Ok(repo) = moai_git::GitRepo::open(path) else {
        // git 저장소가 아닌 경우 정상 케이스 — 빈 맵 반환
        return "{}".to_string();
    };

    match repo.status_map() {
        Ok(map) => {
            let json_map: serde_json::Map<String, serde_json::Value> = map
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect();
            serde_json::to_string(&json_map).unwrap_or_else(|_| "{}".to_string())
        }
        Err(_) => "{}".to_string(),
    }
}
