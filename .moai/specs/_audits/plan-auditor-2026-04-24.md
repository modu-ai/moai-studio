# Plan Auditor 전 SPEC 감사 보고서 (2026-04-24)

**감사자**: plan-auditor subagent (adversarial, read-only)
**대상**: 10 SPEC × spec.md + 관련 산출물
**Branch**: feature/SPEC-V3-003-ms2-tabcontainer @ c12be23
**판정 요약**: 12 C + 18 M + 18 m / 2 FAILED + 7 CONDITIONAL + 1 APPROVED

## 수정 완료 (2026-04-24)

| SPEC | 수정 내용 | Severity |
|------|-----------|----------|
| SPEC-V3-001 | status draft → completed / MSRV 1.82+ → 1.93 / MS-2 이관 표시 / frontmatter schema | C-001, C-002, M-001, M-002 |
| SPEC-V3-002 | terminal-spike → ghostty-spike (10회) / copypasta → arboard / frontmatter | C-001, m-002, m-001 |
| SPEC-V3-003 | status approved → run-in-progress-ms2-complete / milestone_status 필드 / §7.7 TabBar abstract | C-001, M-001 |
| SPEC-M2-003 | archived-v2-design 동결 (status + superseded_by: SPEC-V3-003) | C-001 (AC 네임스페이스), C-002 |
| SPEC-M3-001 | archived-v2-design 동결 (status + superseded_by: TBD v3 rewrite) | C-001 |

## 잔여 (Priority Low — 별도 정비)

- M0/M1/M2-001/M2-002: archive 로 supersede 처리 권장 (M1/M2-001/M2-002 는 기능은 archive 이관되었으나 completion-report 유효)
- EARS [If-Then]/[Complex] 라벨 정규화 (5 SPEC 해당)
- REQ-ID 부여 (legacy SPEC 다수)
- SPEC-V3-002 §7 test 파일명 drift (pty_contract.rs / pty_fd_cleanup.rs 등 실 파일명으로 전수 갱신) — MS-3 sync phase 예정

## 실제 코드 검증

- `cargo check --workspace --all-targets` → 성공
- `cargo clippy --workspace --all-targets -- -D warnings` → 0 경고
- `cargo test --workspace --all-targets` → 전원 PASS (moai-studio-terminal 13 + moai-studio-ui 143 lib + integration + doc + 기타 crate 170+)
- **실제 코드 버그 0건**. 모든 감사 findings 는 문서-코드 drift (문서가 stale).

## 참조

- plan-auditor agent 출력 전문은 본 conversation log 내 agent return value
- SPEC 편집 commit: (본 세션)
