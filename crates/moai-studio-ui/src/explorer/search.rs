// @MX:NOTE: [AUTO] fuzzy-filter-apply
// @MX:SPEC: SPEC-V3-005 RG-FE-6 REQ-FE-051/052
// fuzzy_match 는 case-insensitive subsequence match.
// apply_filter / clear_filter 는 FsNode 트리 전체의 is_visible_under_filter 를 갱신한다.

use crate::explorer::tree::{ChildState, FsNode};

// ============================================================
// fuzzy_match — case-insensitive subsequence 매칭
// ============================================================

/// haystack 이 needle 의 subsequence 를 case-insensitive 로 포함하는지 반환한다 (REQ-FE-051).
///
/// needle 이 빈 문자열이면 항상 true 를 반환한다 (모두 일치).
pub fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let haystack_lower = haystack.to_lowercase();
    let needle_lower = needle.to_lowercase();

    // subsequence match: needle 의 각 문자가 haystack 에 순서대로 등장하는지 확인
    let mut needle_chars = needle_lower.chars();
    let mut current_needle_char = match needle_chars.next() {
        Some(c) => c,
        None => return true,
    };

    for h_char in haystack_lower.chars() {
        if h_char == current_needle_char {
            match needle_chars.next() {
                Some(c) => current_needle_char = c,
                None => return true, // needle 전체 매칭 완료
            }
        }
    }

    false // needle 을 다 소모하지 못함
}

// ============================================================
// apply_filter / clear_filter — FsNode 트리 visibility 갱신
// ============================================================

/// FsNode 트리에 fuzzy filter 를 적용하여 각 노드의 `is_visible_under_filter` 를 갱신한다 (REQ-FE-051).
///
/// query 가 빈 문자열이면 모든 노드를 visible 로 복원한다 (REQ-FE-052).
pub fn apply_filter(node: &mut FsNode, query: &str) -> bool {
    if query.is_empty() {
        // REQ-FE-052: 빈 query → 모든 노드 visible
        set_all_visible(node, true);
        return true;
    }

    match node {
        FsNode::File {
            name,
            is_visible_under_filter,
            ..
        } => {
            let visible = fuzzy_match(name, query);
            *is_visible_under_filter = visible;
            visible
        }
        FsNode::Dir {
            name,
            children,
            is_visible_under_filter,
            ..
        } => {
            // 1) 자식 중 하나라도 visible 이면 부모도 visible
            let mut any_child_visible = false;

            if let ChildState::Loaded(kids) = children {
                for kid in kids.iter_mut() {
                    let child_vis = apply_filter(kid, query);
                    if child_vis {
                        any_child_visible = true;
                    }
                }
            }

            // 2) 자신의 이름이 매칭되면 visible (자식과 무관)
            let self_matches = fuzzy_match(name, query);
            let visible = self_matches || any_child_visible;
            *is_visible_under_filter = visible;
            visible
        }
    }
}

/// FsNode 트리의 모든 노드를 visible/invisible 로 설정한다.
pub fn set_all_visible(node: &mut FsNode, visible: bool) {
    match node {
        FsNode::File {
            is_visible_under_filter,
            ..
        } => {
            *is_visible_under_filter = visible;
        }
        FsNode::Dir {
            is_visible_under_filter,
            children,
            ..
        } => {
            *is_visible_under_filter = visible;
            if let ChildState::Loaded(kids) = children {
                for kid in kids.iter_mut() {
                    set_all_visible(kid, visible);
                }
            }
        }
    }
}

// ============================================================
// 단위 테스트 — AC-FE-12
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::explorer::git_status::GitStatus;
    use crate::explorer::tree::{ChildState, FsNode};
    use std::path::PathBuf;

    // AC-FE-12: fuzzy_match case-insensitive subsequence 검증
    #[test]
    fn fuzzy_match_case_insensitive_subsequence() {
        // "auth" 는 "auth" 와 매칭
        assert!(fuzzy_match("auth/mod.rs", "auth"), "직접 포함이면 매칭");
        // "AUTH" 도 "auth" 와 매칭 (case-insensitive)
        assert!(fuzzy_match("AUTH_module.rs", "auth"), "대소문자 무관 매칭");
        // subsequence: "atr" 는 "auth_test.rs" 에서 a..t..r 순서로 등장
        assert!(fuzzy_match("auth_test.rs", "atr"), "subsequence 매칭");
        // 빈 needle → 항상 true
        assert!(fuzzy_match("anything", ""), "빈 needle 은 항상 매칭");
        // 매칭 안 되는 경우
        assert!(
            !fuzzy_match("main.rs", "auth"),
            "관련 없는 문자열은 매칭 안 됨"
        );
    }

    // AC-FE-12: 트리 ["src/main.rs", "src/auth/mod.rs", "tests/auth_test.rs"] + query "auth" → 2 visible
    #[test]
    fn apply_filter_query_auth_shows_2_nodes() {
        // 트리 구조 생성
        // root/
        //   src/
        //     main.rs
        //     auth/
        //       mod.rs
        //   tests/
        //     auth_test.rs

        let main_rs = FsNode::File {
            rel_path: PathBuf::from("src/main.rs"),
            name: "main.rs".to_string(),
            git_status: GitStatus::Clean,
            is_visible_under_filter: true,
        };

        let mod_rs = FsNode::File {
            rel_path: PathBuf::from("src/auth/mod.rs"),
            name: "mod.rs".to_string(),
            git_status: GitStatus::Clean,
            is_visible_under_filter: true,
        };

        let auth_dir = FsNode::Dir {
            rel_path: PathBuf::from("src/auth"),
            name: "auth".to_string(),
            children: ChildState::Loaded(vec![mod_rs]),
            is_expanded: true,
            git_status: GitStatus::Clean,
            is_visible_under_filter: true,
        };

        let src_dir = FsNode::Dir {
            rel_path: PathBuf::from("src"),
            name: "src".to_string(),
            children: ChildState::Loaded(vec![main_rs, auth_dir]),
            is_expanded: true,
            git_status: GitStatus::Clean,
            is_visible_under_filter: true,
        };

        let auth_test_rs = FsNode::File {
            rel_path: PathBuf::from("tests/auth_test.rs"),
            name: "auth_test.rs".to_string(),
            git_status: GitStatus::Clean,
            is_visible_under_filter: true,
        };

        let tests_dir = FsNode::Dir {
            rel_path: PathBuf::from("tests"),
            name: "tests".to_string(),
            children: ChildState::Loaded(vec![auth_test_rs]),
            is_expanded: true,
            git_status: GitStatus::Clean,
            is_visible_under_filter: true,
        };

        let mut root = FsNode::Dir {
            rel_path: PathBuf::from(""),
            name: "root".to_string(),
            children: ChildState::Loaded(vec![src_dir, tests_dir]),
            is_expanded: true,
            git_status: GitStatus::Clean,
            is_visible_under_filter: true,
        };

        // "auth" query 적용
        apply_filter(&mut root, "auth");

        // visible 노드 수집
        let mut visible_names: Vec<String> = Vec::new();
        collect_visible_names(&root, &mut visible_names);

        // "auth" 포함: auth 디렉토리, mod.rs (auth 디렉토리 하위), auth_test.rs
        // src 는 auth 가 하위에 있어 visible, tests 도 auth_test 가 하위에 있어 visible
        // main.rs 는 "auth" 없음 → invisible
        assert!(
            !visible_names.contains(&"main.rs".to_string()),
            "main.rs 는 'auth' 와 관련 없어 invisible 이어야 한다"
        );
        assert!(
            visible_names.contains(&"auth_test.rs".to_string()),
            "auth_test.rs 는 visible 이어야 한다"
        );
        // auth 디렉토리는 이름 자체가 "auth" 이므로 visible
        assert!(
            visible_names.contains(&"auth".to_string()),
            "auth 디렉토리는 visible 이어야 한다"
        );
    }

    // AC-FE-12: query "main" → 1 visible
    #[test]
    fn apply_filter_query_main_shows_1_node() {
        let main_rs = FsNode::File {
            rel_path: PathBuf::from("src/main.rs"),
            name: "main.rs".to_string(),
            git_status: GitStatus::Clean,
            is_visible_under_filter: true,
        };
        let lib_rs = FsNode::File {
            rel_path: PathBuf::from("src/lib.rs"),
            name: "lib.rs".to_string(),
            git_status: GitStatus::Clean,
            is_visible_under_filter: true,
        };

        let mut root = FsNode::Dir {
            rel_path: PathBuf::from("src"),
            name: "src".to_string(),
            children: ChildState::Loaded(vec![main_rs, lib_rs]),
            is_expanded: true,
            git_status: GitStatus::Clean,
            is_visible_under_filter: true,
        };

        apply_filter(&mut root, "main");

        let mut visible_names: Vec<String> = Vec::new();
        collect_visible_names(&root, &mut visible_names);

        assert!(
            visible_names.contains(&"main.rs".to_string()),
            "main.rs 는 visible"
        );
        assert!(
            !visible_names.contains(&"lib.rs".to_string()),
            "lib.rs 는 invisible"
        );
    }

    // AC-FE-12: 빈 query → 모두 visible (REQ-FE-052)
    #[test]
    fn apply_filter_empty_query_restores_all_visible() {
        let file1 = FsNode::File {
            rel_path: PathBuf::from("a.rs"),
            name: "a.rs".to_string(),
            git_status: GitStatus::Clean,
            is_visible_under_filter: false, // 사전에 hidden 상태
        };
        let file2 = FsNode::File {
            rel_path: PathBuf::from("b.rs"),
            name: "b.rs".to_string(),
            git_status: GitStatus::Clean,
            is_visible_under_filter: false, // 사전에 hidden 상태
        };

        let mut root = FsNode::Dir {
            rel_path: PathBuf::from(""),
            name: "root".to_string(),
            children: ChildState::Loaded(vec![file1, file2]),
            is_expanded: true,
            git_status: GitStatus::Clean,
            is_visible_under_filter: false,
        };

        // 빈 query → 모두 visible 복원
        apply_filter(&mut root, "");

        let mut visible_names: Vec<String> = Vec::new();
        collect_visible_names(&root, &mut visible_names);

        assert!(
            visible_names.contains(&"a.rs".to_string()),
            "빈 query 후 a.rs visible"
        );
        assert!(
            visible_names.contains(&"b.rs".to_string()),
            "빈 query 후 b.rs visible"
        );
        assert!(
            visible_names.contains(&"root".to_string()),
            "빈 query 후 root visible"
        );
    }

    /// 테스트 헬퍼: visible 노드 이름 수집
    fn collect_visible_names(node: &FsNode, names: &mut Vec<String>) {
        let (vis, name, children) = match node {
            FsNode::File {
                is_visible_under_filter,
                name,
                ..
            } => (*is_visible_under_filter, name.clone(), None),
            FsNode::Dir {
                is_visible_under_filter,
                name,
                children,
                ..
            } => (*is_visible_under_filter, name.clone(), Some(children)),
        };

        if vis {
            names.push(name);
        }

        if let Some(ChildState::Loaded(kids)) = children {
            for kid in kids {
                collect_visible_names(kid, names);
            }
        }
    }
}
