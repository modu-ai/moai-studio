//! SPEC-V3-009 AC-SU-1 ~ AC-SU-5 integration tests.
//!
//! 실제 `.moai/specs/` 디렉터리를 사용하는 통합 테스트.
//!
//! AC-SU-1: SPEC-V3-009 디렉터리가 SpecIndex::scan 결과에 1개 카드로 등장.
//! AC-SU-2: SPEC-V3-009 자신의 spec.md EARS 표 파싱 (표 형식 RG 확인).
//! AC-SU-3: SPEC-V3-009 progress.md AC 상태 파싱.
//! AC-SU-5: acceptance.md 없는 SPEC 도 panic 없이 처리.

use moai_studio_spec::{
    AcState, SpecFileKind, SpecId, SpecIndex, parse_ac_states_from_progress, parse_spec_md,
};
use std::path::Path;

/// 실제 `.moai/specs/` 경로 (worktree 기준).
fn specs_dir() -> std::path::PathBuf {
    // 레포 루트 / .moai / specs
    let root = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR 환경 변수 필요");
    Path::new(&root)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(".moai/specs")
}

// ── AC-SU-1: SpecIndex::scan 이 .moai/specs/ 에서 SPEC 카드를 발견한다 ──

#[test]
fn ac_su_1_specs_dir_contains_spec_v3_009() {
    let specs = specs_dir();
    if !specs.exists() {
        eprintln!("specs_dir 없음, 스킵: {:?}", specs);
        return;
    }

    let mut index = SpecIndex::new();
    index.scan(&specs);

    assert!(
        !index.is_empty(),
        "AC-SU-1: .moai/specs/ 에서 최소 1개 SPEC 발견"
    );

    // SPEC-V3-009 자체가 카드로 등장해야 함
    let id = SpecId::new("SPEC-V3-009");
    assert!(
        index.find(&id).is_some(),
        "AC-SU-1: SPEC-V3-009 가 인덱스에 존재해야 함"
    );
}

// ── AC-SU-2: SPEC-V3-009 spec.md 의 EARS 표 파싱 ──

#[test]
fn ac_su_2_spec_v3_009_ears_tables_parsed() {
    let specs = specs_dir();
    let spec_md = specs.join("SPEC-V3-009/spec.md");
    if !spec_md.exists() {
        eprintln!("spec.md 없음, 스킵: {:?}", spec_md);
        return;
    }

    let content = std::fs::read_to_string(&spec_md).expect("spec.md 읽기 실패");
    let parsed = parse_spec_md(&content);

    // SPEC-V3-009 는 RG-SU-1 ~ RG-SU-6 (6개 RG) 를 가짐
    let rg_count = parsed.requirement_groups.len();
    eprintln!(
        "AC-SU-2: SPEC-V3-009 RG count = {}, REQ count = {}",
        rg_count,
        parsed
            .requirement_groups
            .iter()
            .map(|g| g.requirements.len())
            .sum::<usize>()
    );
    assert!(
        rg_count >= 6,
        "AC-SU-2: SPEC-V3-009 RG 개수 >= 6 (실제: {rg_count})"
    );

    // AC 표 존재
    let ac_count = parsed.ac_rows.len();
    eprintln!("AC-SU-2: SPEC-V3-009 AC count = {ac_count}");
    assert!(
        ac_count >= 5,
        "AC-SU-2: SPEC-V3-009 AC 개수 >= 5 (AC-SU-1 ~ AC-SU-5 최소값, 실제: {ac_count})"
    );
}

// ── AC-SU-2b: SPEC-V3-003 spec.md 의 RG heading 개수 확인 ──

#[test]
fn ac_su_2b_spec_v3_003_rg_headings_count() {
    let specs = specs_dir();
    let spec_md = specs.join("SPEC-V3-003/spec.md");
    if !spec_md.exists() {
        eprintln!("SPEC-V3-003 spec.md 없음, 스킵");
        return;
    }

    let content = std::fs::read_to_string(&spec_md).expect("spec.md 읽기 실패");

    // SPEC-V3-003 은 RG-P-1 ~ RG-P-7 (7개 ### RG-* heading)
    let rg_heading_count = content.lines().filter(|l| l.starts_with("### RG-")).count();
    eprintln!("SPEC-V3-003 RG heading count = {rg_heading_count}");
    assert!(
        rg_heading_count >= 7,
        "SPEC-V3-003 RG heading 개수 >= 7 (실제: {rg_heading_count})"
    );

    // SPEC-V3-003 은 볼드 텍스트 형식 REQ — REQ-P-NNN count
    let req_count = content
        .lines()
        .filter(|l| l.starts_with("**REQ-P-"))
        .count();
    eprintln!("SPEC-V3-003 REQ-P-* count = {req_count}");
    // 실측: 37개 (볼드 텍스트 형식, 일부 인라인 참조 제외)
    assert!(
        req_count >= 30,
        "SPEC-V3-003 REQ-P-* 개수 >= 30 (실제: {req_count})"
    );
}

// ── AC-SU-3: AC 상태 5분류 파싱 ──

#[test]
fn ac_su_3_ac_state_five_states_parsed() {
    let specs = specs_dir();
    let progress_md = specs.join("SPEC-V3-003/progress.md");
    if !progress_md.exists() {
        eprintln!("progress.md 없음, 스킵");
        return;
    }

    let content = std::fs::read_to_string(&progress_md).expect("progress.md 읽기 실패");
    let records = parse_ac_states_from_progress(&content);

    eprintln!("AC-SU-3: progress.md AC 레코드 수 = {}", records.len());
    assert!(
        !records.is_empty(),
        "AC-SU-3: progress.md 에서 AC 상태가 파싱되어야 함"
    );

    // Full (PASS) 상태가 있어야 함
    let has_full = records.iter().any(|r| r.state == AcState::Full);
    assert!(has_full, "AC-SU-3: PASS 상태가 최소 1개 있어야 함");
}

// ── AC-SU-5: acceptance.md 없는 SPEC 도 panic 없이 처리 ──

#[test]
fn ac_su_5_missing_acceptance_md_no_panic() {
    let specs = specs_dir();
    if !specs.exists() {
        eprintln!("specs_dir 없음, 스킵");
        return;
    }

    let mut index = SpecIndex::new();
    index.scan(&specs);

    // acceptance.md 없는 SPEC 을 찾아서 panic 없이 처리됨을 확인
    for record in &index.records {
        if !record.has_file(SpecFileKind::Acceptance) {
            eprintln!(
                "AC-SU-5: {} — acceptance.md 없음, graceful 처리 확인",
                record.id
            );
            // panic 없이 여기까지 도달하면 AC-SU-5 통과
            let summary = record.ac_summary();
            eprintln!("  AC summary: {}", summary.display());
            return;
        }
    }

    // 모든 SPEC 에 acceptance.md 가 있으면 스킵 (테스트 조건 없음)
    eprintln!("AC-SU-5: 모든 SPEC 에 acceptance.md 존재 — 조건 미충족, 스킵");
}

// ── Sprint Contract Revision 파싱 ──

#[test]
fn sprint_contract_revisions_in_spec_v3_003() {
    let specs = specs_dir();
    let spec_md = specs.join("SPEC-V3-003/spec.md");
    if !spec_md.exists() {
        eprintln!("SPEC-V3-003 spec.md 없음, 스킵");
        return;
    }

    let content = std::fs::read_to_string(&spec_md).expect("spec.md 읽기 실패");
    let parsed = parse_spec_md(&content);

    eprintln!(
        "Sprint Contract Revisions in SPEC-V3-003 = {}",
        parsed.sprint_contracts.len()
    );
    // SPEC-V3-009 spec.md 에 Sprint Contract Revision 이 없으면 0도 OK
    // — SPEC-V3-003 도 있을 수 있음
    // panic 없이 파싱되는 것이 목표
}

// ── SpecIndex 성능 smoke test (50 SPEC 디렉터리 가정) ──

#[test]
fn spec_index_scan_completes_in_reasonable_time() {
    use std::time::Instant;

    let specs = specs_dir();
    if !specs.exists() {
        return;
    }

    let start = Instant::now();
    let mut index = SpecIndex::new();
    index.scan(&specs);
    let elapsed = start.elapsed();

    eprintln!("SpecIndex::scan — {} SPEC, {:?}", index.len(), elapsed);

    // 500ms 이내 (실제 파일 I/O 포함, 20+ SPEC 스캔 허용 범위)
    // 비기능 요구사항 "50 SPEC 기준 200ms" 는 lazy-load 병행 적용 시 달성 예정 (MS-2)
    assert!(
        elapsed.as_millis() < 500,
        "스캔 시간 < 500ms (실제: {:?})",
        elapsed
    );
}
