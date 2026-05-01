# SPEC-V0-2-0-GLOBAL-SEARCH-001 Progress

**Started**: 2026-05-01 sess 8 (planning phase)
**Branch**: `feature/SPEC-V0-2-0-GLOBAL-SEARCH-001-ms1-engine`
**SPEC status**: ready (annotation iteration 1 complete 2026-05-02 sess 9, USER-DECISION 3건 모두 (a) lock-in)
**Completion date**: TBD

## Implementation Timeline

| 일자 | 세션 | 단계 | 산출 |
|------|------|------|------|
| 2026-05-01 | sess 8 | planning (manager-spec) | research.md (22.5KB) + spec.md v1.0.0-draft (36.9KB) + progress.md (template) |
| 2026-05-02 | sess 9 | annotation iteration 1 | spec.md v1.1.0-ready (USER-DECISION 3건 lock-in) + progress.md 갱신 + plan.md (milestones × tasks × files × AC 매핑) |
| 2026-05-02 | sess 9 | MS-1 implementation (manager-tdd 위임) | `crates/moai-search/` 신규 crate (Cargo.toml + 6 src files) + 18 unit tests + 2 doc-tests + workspace deps (`ignore = "0.4"`, `regex = "1"`) — AC-GS-1~6 PASS, AC-GS-12 (regex meta) PASS, clippy 0 warning, fmt clean, 워크스페이스 회귀 0. |

## Milestone Status

- [x] **MS-1**: `crates/moai-search/` 신규 crate — `SearchSession` / `SearchHit` / `SearchOptions` / `CancelToken` + walker (ignore::WalkBuilder) + matcher (regex/literal fallback) + cancel token. AC-GS-1 ~ AC-GS-6 ✅ PASS (sess 9 manager-tdd, 2026-05-02).
- [ ] **MS-2**: `crates/moai-studio-ui/src/search/` 모듈 — SearchPanel GPUI Entity + result row rendering + 사이드바 section toggle + ⌘⇧F dispatch. AC-GS-7, AC-GS-8, AC-GS-9, AC-GS-12 (UI 측).
- [ ] **MS-3**: navigation wire — SearchHit click → workspace activate + new tab + line jump (OpenCodeViewer adapter). Command Palette `workspace.search` entry handler dispatch + label/keybinding 갱신. AC-GS-10, AC-GS-11.
- [ ] **MS-4**: polish — backpressure (1000 cap auto-cancel + message), per-workspace progress spinner, ↑↓ keyboard navigation, match highlight in preview, integration test `tests/integration_search.rs`. final regression sweep.

## USER-DECISION Resolutions

| Decision ID | 질문 요약 | 권장 | 결정 결과 | 결정 일자 | 영향 |
|-------------|----------|------|-----------|----------|------|
| USER-DECISION-A | 검색 엔진 선택 (pure Rust ignore+regex / ripgrep crate / ripgrep subprocess / tantivy) | (a) pure Rust | ✅ (a) pure Rust (`ignore = "0.4"` + `regex = "1"`) | 2026-05-02 | MS-1 dependency lock-in |
| USER-DECISION-B | 신규 crate `moai-search` vs ui 모듈 통합 | (a) 신규 crate | ✅ (a) 신규 crate `crates/moai-search/` | 2026-05-02 | MS-1 workspace member 등록 |
| USER-DECISION-C | 결과 cap 디폴트 (per-file/per-workspace/total) | (a) 50/200/1000 | ✅ (a) per-file 50 / per-workspace 200 / total 1000 | 2026-05-02 | REQ-GS-024 디폴트 lock-in |

## Acceptance Criteria Status

| AC ID | Status | Notes |
|-------|--------|-------|
| AC-GS-1  | ✅ PASS | engine domain model — 4 핵심 타입 정의, GPUI 의존 0, `cargo build -p moai-search` GREEN, 18 unit + 2 doc-tests PASS (sess 9) |
| AC-GS-2  | ✅ PASS | walk_workspace happy path — `test_walk_workspace_happy_path` (sess 9) |
| AC-GS-3  | ✅ PASS | gitignore + custom exclude — `test_walk_workspace_respects_gitignore`, `test_walk_workspace_custom_excludes` (sess 9). 단위 테스트는 `.ignore` 파일로 검증 (tempdir에서 .gitignore 인식 안 됨, production은 git repo이므로 정상 작동). |
| AC-GS-4  | ✅ PASS | binary file skip — `test_walk_workspace_skips_binary_files` (NUL byte 첫 8KB heuristic, sess 9) |
| AC-GS-5  | ✅ PASS | cancel mid-walk — `test_walk_workspace_cancel_mid_walk` (per-file + per-line poll, sess 9) |
| AC-GS-6  | ✅ PASS | result cap (per-file 50 / per-workspace 200 / total 1000) — 3 단위 테스트 (per-file/per-workspace/total auto-cancel, sess 9) |
| AC-GS-7  | ⬜ TODO | SearchPanel ⌘⇧F 토글 + input focus (MS-2) |
| AC-GS-8  | ⬜ TODO | result row 2-line layout + batch flush (100 hits / 1000ms) (MS-2) |
| AC-GS-9  | ⬜ TODO | status (Empty / Searching / No matches) + empty query no-spawn (MS-2) |
| AC-GS-10 | ⬜ TODO | navigation (touch + new_tab + OpenCodeViewer) (MS-3) |
| AC-GS-11 | ⬜ TODO | Command Palette `workspace.search` entry dispatch + label/keybinding 갱신 (MS-3) |
| AC-GS-12 | 🟡 PARTIAL | regex meta auto-detect + compile fail → literal fallback ✅ PASS (`test_regex_auto_detect`, `test_regex_compile_failure_fallback_to_literal`, sess 9). UI 측 edge cases (0 ws / 1 ws) MS-2 carry. |

상태 범례:
- ⬜ TODO — 미시작
- 🟡 IN PROGRESS — 구현 중
- ✅ PASS — AC 검증 통과
- ❌ FAIL — 검증 실패 (재작업 필요)

## Test Coverage

### MS-1 실측 (sess 9, 2026-05-02)

- `cargo test -p moai-search`: **18 unit tests + 2 doc-tests = 20 total, 0 failed** (0.02s + doctests 1.25s)
  - cancel: 3 tests (default_false / clone_shares_state / propagates_after_cancel)
  - types: 2 tests (search_options_defaults / search_hit_fields_and_clone)
  - matcher: 4 tests (literal_substring / literal_case_insensitive / regex_auto_detect / regex_compile_fail_fallback)
  - walker: 8 tests (happy_path / respects_gitignore / custom_excludes / skips_binary_files / cancel_mid_walk / per_file_cap / per_workspace_cap / total_cap_auto_cancels)
  - session: 1 test (spawn_and_cancel)
  - doc-tests: 2 (session::SearchSession + cancel::CancelToken)
- `cargo clippy -p moai-search --all-targets -- -D warnings`: 0 warning
- `cargo fmt -p moai-search --check`: clean
- `cargo build --workspace`: GREEN, 회귀 0

### MS-2/3/4 목표 (carry):
- `cargo test -p moai-search` — engine 단위 테스트, AC-GS-1~6 검증
- `cargo test -p moai-studio-ui --lib search::tests` — UI 단위 테스트, AC-GS-9/12 검증
- `cargo test -p moai-studio-ui --test integration_search` (신규) — integration, AC-GS-7/8/10 검증
- `cargo clippy --workspace -- -D warnings` 0 warning
- `cargo fmt --check` clean

v0.1.2 GA baseline 회귀 0:
- 1148 ui crate tests 보존
- workspace + terminal + spec + 기타 crates 테스트 모두 GREEN

## Known Limitations (lock-in 후, v1 scope)

- v1 은 lazy walk only — 큰 monorepo (10k+ 파일) 첫 검색 지연 (P2 < 500ms target). tantivy 인덱싱은 v0.3.0+ deferred.
- regex / case sensitivity / word boundary toggle UI v1 미지원 — v0.2.1 carry.
- glob include 패턴 (`*.rs` 만) v1 미지원 — v0.2.1 carry.
- Replace v1 미지원 — v0.3.0+.
- 검색 history / saved searches v1 미지원.
- cross-workspace 단일 tab 미지원 — workspace 전환 후 tab open (D-3 정합).
- 같은 path 가 이미 다른 tab 으로 열려 있을 때의 reuse 정책 v1 미지원 (항상 새 tab) — v0.2.1 검토.
- ANSI color highlight in preview 미지원 — match highlight 만 (HTML-style mark or color).
- Windows GPUI e2e 검증은 별 SPEC.

## Carry-Over

후보:
- Settings 에 SearchPanel default 옵션 (cap, case sensitivity 디폴트, exclude pattern 추가) — settings/panes/ 에 신규 pane 또는 advanced.rs 확장.
- explorer/search.rs (single-workspace fuzzy filter, filename only) 와 본 SPEC (multi-workspace content grep) 의 통합 vs 분리 정책 (현재 분리 유지).

## Annotation Cycle Notes

### Iteration 1 (2026-05-02 sess 9) — RESOLVED

**완료 항목**:
1. ✅ USER-DECISION-A 검색 엔진 = (a) pure Rust (`ignore` + `regex`)
2. ✅ USER-DECISION-B crate 분리 = (a) 신규 crate `crates/moai-search/`
3. ✅ USER-DECISION-C 결과 cap = (a) per-file 50 / per-workspace 200 / total 1000
4. ✅ SearchPanel placement = 사이드바 toggleable section (initial draft 그대로 lock-in)
5. ✅ 결과 click 정책 = 항상 새 tab (N13 비목표 lock-in, reuse v0.2.1 carry)
6. ✅ 활성 workspace 0 일 때 = input disabled + placeholder (REQ-GS-060 lock-in)

**결과**: SPEC status draft → ready. plan.md 작성 + MS-1 manager-tdd 위임 진입 가능. 추가 annotation iteration 미예정 (구현 중 결정 필요 항목 발견 시 iteration 2 발생 가능).

---

## References

- spec.md v1.1.0-ready (본 SPEC §1~12) — EARS requirements, 12 AC, 3 USER-DECISION (RESOLVED), 4 milestones
- plan.md — milestone × task × file × AC 매핑 (sess 9 작성)
- research.md — 4 검색 엔진 비교, 인덱싱 전략, 동시성 패턴, 위험 평가
- `.moai/specs/RELEASE-V0.2.0/feature-audit.md` §3 Tier D, §4 Top 1, §10 carry table
- `.moai/design/v3/spec.md` v3.1.0 Tier D (D-4), §7 IA, §8 키바인딩
- `.moai/specs/SPEC-V3-LINK-001/spec.md` (OpenCodeViewer 패턴)
- `.moai/specs/SPEC-V3-005/spec.md` (Render Entity 분리 패턴, normalize_for_display)
- `.moai/specs/SPEC-V3-004/spec.md` (tab_container + workspace switch persistence)
- `.moai/specs/SPEC-V3-006/spec.md` (CodeViewer line/col scroll)
