//! SPEC-V3-009 MS-3 RG-SU-4 — git branch 파서 + SPEC-ID 매핑.
//!
//! REQ-SU-030: `git branch --list 'feature/SPEC-*'` + `git branch --show-current` 파싱.
//! REQ-SU-031: `feature/SPEC-{area}-{nnn}-{slug}` regex 인식. legacy 는 best-effort.
//! REQ-SU-032: 활성 branch 가 SPEC feature branch 와 일치하면 `is_active = true`.
//!
//! # @MX:ANCHOR: [AUTO] BranchState
//! @MX:REASON: [AUTO] SPEC-V3-009 §12 외부 인터페이스 고정. fan_in >= 3:
//!   SpecRecord.branch, spec_index::scan_with_branches, AC-SU-8 테스트.
//!
//! # @MX:WARN: [AUTO] subprocess spawn (git)
//! @MX:REASON: [AUTO] REQ-SU-044 spirit — git 미설치 또는 .git 없을 때 panic 금지.
//!   모든 Command::new("git") 호출은 결과를 unwrap 하지 않고 graceful 처리.

use std::path::Path;

use regex::Regex;

use crate::state::SpecId;

/// 단일 git feature branch 의 상태.
///
/// @MX:ANCHOR: [AUTO] BranchState — SPEC-V3-009 §12 외부 인터페이스.
/// @MX:REASON: [AUTO] fan_in >= 3: SpecRecord, spec_index, AC-SU-8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchState {
    /// 브랜치 이름 (예: "feature/SPEC-V3-009-ms3-cli-integration")
    pub branch_name: String,
    /// 브랜치 이름에서 파싱된 SPEC ID. None 이면 "unmatched" (REQ-SU-031)
    pub spec_id: Option<SpecId>,
    /// `git branch --show-current` 결과와 일치하는지 여부 (REQ-SU-032)
    pub is_active: bool,
}

/// `feature/SPEC-{area}-{nnn}` 패턴에서 SPEC ID 를 추출한다 (REQ-SU-031).
///
/// 예시:
/// - `"feature/SPEC-V3-009-ms3-cli-integration"` → `Some(SpecId("SPEC-V3-009"))`
/// - `"feat/v3-scaffold"` → `None`
/// - `"main"` → `None`
pub fn parse_spec_id_from_branch(branch: &str) -> Option<SpecId> {
    // lazy_static 없이 직접 생성 (호출 빈도 낮음)
    let re = Regex::new(r"^feature/(SPEC-[A-Z0-9]+-[0-9]+)(?:-.+)?$")
        .expect("branch regex 는 컴파일 타임 유효해야 함");
    re.captures(branch)
        .and_then(|c| c.get(1))
        .map(|m| SpecId::new(m.as_str()))
}

/// repo_root 의 활성 branch 이름을 반환한다.
///
/// `.git` 이 없거나 git 명령 실패 시 None 반환 (NEVER panic).
pub fn active_branch(repo_root: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .current_dir(repo_root)
        .args(["branch", "--show-current"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// `feature/SPEC-*` 패턴과 일치하는 branch 목록을 반환한다 (REQ-SU-030).
///
/// `.git` 이 없거나 git 명령 실패 시 빈 Vec 반환 (NEVER panic).
/// 활성 branch 는 `is_active = true` 로 표시한다 (REQ-SU-032).
pub fn list_spec_branches(repo_root: &Path) -> Vec<BranchState> {
    // git 명령 실패 시 graceful empty 반환
    let output = match std::process::Command::new("git")
        .current_dir(repo_root)
        .args(["branch", "--list", "feature/SPEC-*"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    if !output.status.success() {
        return Vec::new();
    }

    let active = active_branch(repo_root);

    let branches_text = String::from_utf8_lossy(&output.stdout);
    branches_text
        .lines()
        .map(|line| {
            // `git branch --list` 출력에서 `* ` 또는 `  ` prefix 제거
            let name = line.trim_start_matches("* ").trim_start_matches("  ").trim();
            let spec_id = parse_spec_id_from_branch(name);
            let is_active = active.as_deref() == Some(name);
            BranchState {
                branch_name: name.to_string(),
                spec_id,
                is_active,
            }
        })
        .filter(|b| !b.branch_name.is_empty())
        .collect()
}

// ============================================================
// 단위 테스트 (RED 단계에서 먼저 작성)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    // ── parse_spec_id_from_branch 단위 테스트 ────────────────────

    #[test]
    fn parse_spec_id_extracts_v3_009() {
        // AC-SU-8: 표준 feature 브랜치에서 SPEC-V3-009 추출
        let result = parse_spec_id_from_branch("feature/SPEC-V3-009-ms3-cli-integration");
        assert_eq!(result, Some(SpecId::new("SPEC-V3-009")));
    }

    #[test]
    fn parse_spec_id_extracts_auth_001() {
        // AC-SU-8: alphanumeric area 코드 지원
        let result = parse_spec_id_from_branch("feature/SPEC-AUTH-001-jwt");
        assert_eq!(result, Some(SpecId::new("SPEC-AUTH-001")));
    }

    #[test]
    fn parse_spec_id_returns_none_for_legacy() {
        // REQ-SU-031: legacy 이름은 best-effort → None
        let result = parse_spec_id_from_branch("feat/v3-scaffold");
        assert_eq!(result, None);
    }

    #[test]
    fn parse_spec_id_returns_none_for_main() {
        // 일반 브랜치는 None
        let result = parse_spec_id_from_branch("main");
        assert_eq!(result, None);
    }

    #[test]
    fn parse_spec_id_returns_none_for_develop() {
        let result = parse_spec_id_from_branch("develop");
        assert_eq!(result, None);
    }

    #[test]
    fn parse_spec_id_extracts_without_slug() {
        // slug 없는 경우도 지원 (REQ-SU-031: `(?:-.+)?` optional)
        let result = parse_spec_id_from_branch("feature/SPEC-V3-009");
        assert_eq!(result, Some(SpecId::new("SPEC-V3-009")));
    }

    // ── active_branch / list_spec_branches: .git 없는 경우 ──────

    #[test]
    fn active_branch_returns_none_when_no_git() {
        // `.git` 없는 임시 디렉터리 → None (NEVER panic)
        let tmp = TempDir::new().unwrap();
        let result = active_branch(tmp.path());
        assert!(result.is_none());
    }

    #[test]
    fn list_spec_branches_returns_empty_when_no_git() {
        // `.git` 없는 임시 디렉터리 → empty Vec (NEVER panic)
        let tmp = TempDir::new().unwrap();
        let result = list_spec_branches(tmp.path());
        assert!(result.is_empty());
    }

    // ── git fixture 테스트: 실제 bare git repo 생성 ──────────────

    /// 테스트용 git repo 를 초기화한다.
    fn init_git_repo(dir: &Path) {
        let cmds: &[&[&str]] = &[
            &["init"],
            &["config", "user.email", "test@test.com"],
            &["config", "user.name", "Test"],
            // 초기 커밋 (빈 커밋)
            &["commit", "--allow-empty", "-m", "init"],
        ];
        for args in cmds {
            Command::new("git")
                .current_dir(dir)
                .args(*args)
                .output()
                .unwrap();
        }
    }

    fn create_branch(dir: &Path, name: &str) {
        Command::new("git")
            .current_dir(dir)
            .args(["branch", name])
            .output()
            .unwrap();
    }

    fn checkout_branch(dir: &Path, name: &str) {
        // -b 없이 checkout (이미 존재하는 브랜치)
        Command::new("git")
            .current_dir(dir)
            .args(["checkout", name])
            .output()
            .unwrap();
    }

    #[test]
    fn list_spec_branches_in_real_git_fixture() {
        // AC-SU-8: 실제 git repo fixture 에서 branch 목록 확인
        let tmp = TempDir::new().unwrap();
        init_git_repo(tmp.path());

        create_branch(tmp.path(), "feature/SPEC-V3-009-ms3-cli-integration");
        create_branch(tmp.path(), "feature/SPEC-AUTH-001-jwt");

        let branches = list_spec_branches(tmp.path());
        assert_eq!(branches.len(), 2, "feature/SPEC-* 브랜치 2개 발견");

        // SPEC ID 파싱 확인
        let ids: Vec<_> = branches
            .iter()
            .filter_map(|b| b.spec_id.as_ref())
            .map(|id| id.as_str())
            .collect();
        assert!(ids.contains(&"SPEC-V3-009"), "SPEC-V3-009 포함");
        assert!(ids.contains(&"SPEC-AUTH-001"), "SPEC-AUTH-001 포함");
    }

    #[test]
    fn active_branch_returns_current() {
        // AC-SU-8: 활성 브랜치 반환
        let tmp = TempDir::new().unwrap();
        init_git_repo(tmp.path());
        create_branch(tmp.path(), "feature/SPEC-V3-009-ms3");
        checkout_branch(tmp.path(), "feature/SPEC-V3-009-ms3");

        let result = active_branch(tmp.path());
        assert_eq!(result.as_deref(), Some("feature/SPEC-V3-009-ms3"));
    }

    #[test]
    fn list_spec_branches_marks_active_branch() {
        // AC-SU-8: is_active 표시
        let tmp = TempDir::new().unwrap();
        init_git_repo(tmp.path());
        create_branch(tmp.path(), "feature/SPEC-V3-009-ms3");
        checkout_branch(tmp.path(), "feature/SPEC-V3-009-ms3");

        let branches = list_spec_branches(tmp.path());
        assert_eq!(branches.len(), 1);
        assert!(branches[0].is_active, "체크아웃된 브랜치는 is_active=true");
    }
}
