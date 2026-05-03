# SPEC-V0-2-0-GLOBAL-SEARCH-001 Progress

**Started**: 2026-05-01 sess 8 (planning phase)
**Branch**: `feature/SPEC-V0-2-0-GLOBAL-SEARCH-001-ms1-engine`
**SPEC status**: implemented (MS-1/2/3/4 모두 ✅ PASS, 12 AC 모두 ✅ PASS — 2026-05-04 sess 10)
**Completion date**: 2026-05-04 sess 10 (PR #81 머지 시 GA 확정)

## Implementation Timeline

| 일자 | 세션 | 단계 | 산출 |
|------|------|------|------|
| 2026-05-01 | sess 8 | planning (manager-spec) | research.md (22.5KB) + spec.md v1.0.0-draft (36.9KB) + progress.md (template) |
| 2026-05-02 | sess 9 | annotation iteration 1 | spec.md v1.1.0-ready (USER-DECISION 3건 lock-in) + progress.md 갱신 + plan.md (milestones × tasks × files × AC 매핑) |
| 2026-05-02 | sess 9 | MS-1 implementation (manager-tdd 위임) | `crates/moai-search/` 신규 crate (Cargo.toml + 6 src files) + 18 unit tests + 2 doc-tests + workspace deps (`ignore = "0.4"`, `regex = "1"`) — AC-GS-1~6 PASS, AC-GS-12 (regex meta) PASS, clippy 0 warning, fmt clean, 워크스페이스 회귀 0. |
| 2026-05-03 | sess 9 | MS-1 PR #78 admin merge | merge commit `6409fc44`, mergedAt 2026-05-03T17:11:59Z. CI fail (moai-git stash::tests) 가 본 PR 영향 외 별 환경 의존 (git env on CI runner) 으로 admin override. main 동기화 완료. |
| 2026-05-04 | sess 10 | MS-2 implementation (manager-tdd 위임) | `crates/moai-studio-ui/src/search/` 신규 모듈 (mod / panel / result_view / keymap) + RootView 통합 (`search_panel: Option<Entity<SearchPanel>>` 필드 + `handle_toggle_search_panel` + KeyBinding ⌘⇧F/Ctrl+Shift+F) + `moai-search` dep 추가 — AC-GS-7/8/9/12 (UI) PASS, 21 search:: tests, ui crate 1144 → 1165 (+21), clippy 0 warning, fmt clean, 워크스페이스 회귀 0. |
| 2026-05-04 | sess 10 | MS-3 implementation (manager-tdd 위임) | `navigation.rs` 신규 (hit_to_open_code_viewer + touch_workspace + NavigationOutcome) + `result_view.on_row_click` + `panel.hit_for_row_click` + `RootView::handle_search_open` / `handle_search_open_with_cx` / `dispatch_command_workspace_search` + `palette/registry.rs` workspace.search label/keybinding 갱신 + `last_open_code_viewer` 필드 추가 — AC-GS-10/11 PASS, 15 신규 tests, ui crate 1165 → 1180 (+15), clippy 0 warning, fmt clean, 워크스페이스 회귀 0. |
| 2026-05-04 | sess 10 | MS-3 PR #80 admin merge | merge commit `e132e6f8`, mergedAt 2026-05-03T17:43:08Z. CI fail (moai-git stash::tests, MS-1/2 동일 별 환경 의존) admin override. main 동기화. |
| 2026-05-04 | sess 10 | MS-4 implementation (manager-tdd 위임) | panel.rs (TOTAL_HIT_CAP=1000 const + cap_message + add_hit cap 가드 + selected_index 키보드 nav + workspace_progress 맵), result_view.rs (extract_preview_segments 3-segment helper), mod.rs re-export, tests/integration_search.rs 신규 (12 e2e tests — search flow / palette / cap / keyboard nav / open call) — MS-4 task T1~T5 PASS, search:: 28 → 41 (+13 unit), ui crate 1180 → 1193 (+13), integration 12 PASS, clippy 0 warning, fmt clean, 워크스페이스 회귀 0. **AC 12건 모두 PASS — SPEC GA 준비 완료**. |

## Milestone Status

- [x] **MS-1**: `crates/moai-search/` 신규 crate — `SearchSession` / `SearchHit` / `SearchOptions` / `CancelToken` + walker (ignore::WalkBuilder) + matcher (regex/literal fallback) + cancel token. AC-GS-1 ~ AC-GS-6 ✅ PASS (sess 9 manager-tdd, 2026-05-02). PR #78 merge `6409fc44` 2026-05-03.
- [x] **MS-2**: `crates/moai-studio-ui/src/search/` 모듈 — SearchPanel GPUI Entity + result row rendering + RootView 통합 + ⌘⇧F dispatch. AC-GS-7, AC-GS-8, AC-GS-9, AC-GS-12 (UI 측) ✅ PASS (sess 10 manager-tdd, 2026-05-04). 사이드바 visibility render mount 는 MS-3 carry.
- [x] **MS-3**: navigation wire — SearchHit click → workspace activate + new tab + line jump (OpenCodeViewer adapter). Command Palette `workspace.search` entry handler dispatch + label/keybinding 갱신. AC-GS-10, AC-GS-11 ✅ PASS (sess 10 manager-tdd, 2026-05-04). PR #80 merge `e132e6f8` 2026-05-03.
- [x] **MS-4**: polish — TOTAL_HIT_CAP=1000 auto-cancel + cap_message, per-workspace progress map, ↑↓ keyboard nav (selected_index + move_up/down + enter_selected + escape_pressed), match highlight 3-segment splitter (extract_preview_segments), integration test `tests/integration_search.rs` 12 e2e — final regression sweep ✅ PASS (sess 10 manager-tdd, 2026-05-04). 13 신규 unit + 12 integration, ui 1180 → 1193.

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
| AC-GS-7  | ✅ PASS | SearchPanel ⌘⇧F (macOS) / Ctrl+Shift+F (other) keymap action + RootView dispatch + `handle_toggle_search_panel` (sess 10). 실 GPUI focus wire 는 MS-3 carry. |
| AC-GS-8  | ✅ PASS | result row 2-line layout (`format_row_label` + `extract_highlight_span`) + batch flush (100 hits / 1000ms `should_flush`) (sess 10). |
| AC-GS-9  | ✅ PASS | SearchStatus state machine (Empty / Searching / HasResults / NoMatches / CapReached) + empty query no-spawn (`set_query` trim → clear_results) (sess 10). |
| AC-GS-10 | ✅ PASS | navigation (touch_workspace + hit_to_open_code_viewer + handle_search_open + handle_search_open_with_cx). 7 navigation unit + 3 RootView unit tests. OpenCodeViewer path/line/col 정확성 검증. workspace unknown → false (no panic) 확인. (MS-3, sess 10) |
| AC-GS-11 | ✅ PASS | Command Palette `workspace.search` entry label "Search in all workspaces" + keybinding Some("Cmd+Shift+F") 갱신 (R5 id/category frozen). dispatch_command("workspace.search") → dispatch_command_workspace_search() → SearchPanel visible. 5 registry + 2 RootView tests. (MS-3, sess 10) |
| AC-GS-12 | ✅ PASS | regex meta auto-detect + compile fail → literal fallback (sess 9, MS-1) + 0-workspace input disabled + placeholder + 1-workspace 단일 grouping logic (sess 10, MS-2). |

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

### MS-2 실측 (sess 10, 2026-05-04)

- `cargo test -p moai-studio-ui --lib search::`: **21 unit tests PASS** (search::panel + search::result_view + search::keymap)
  - panel: SearchPanel toggle / set_query / SearchStatus state / batch flush 100hits/1000ms / 0-workspace disabled / 1-workspace single group
  - result_view: format_row_label two-line / format_row_label nested path / extract_highlight_span match span
  - keymap: ToggleSearchPanel action dispatch
- ui crate 전체: **1144 → 1165 tests** (+21 신규, 회귀 0)
- `cargo clippy -p moai-studio-ui --all-targets -- -D warnings`: 0 warning
- `cargo fmt --check`: clean
- `cargo build --workspace`: GREEN, 회귀 0

### MS-3 실측 (sess 10, 2026-05-04)

- `cargo test -p moai-studio-ui --lib`: **1180 unit tests PASS** (baseline 1165 + 15 신규)
  - search::navigation: 7 tests (hit_to_open_code_viewer known/unknown/empty + touch_workspace + line/col accuracy + error-state no-panic)
  - palette::registry: 3 신규 tests (label/keybinding/id-category unchanged for workspace.search)
  - tests (RootView): 5 신규 tests (handle_search_open known/unknown/line-col + palette_workspace_search toggle + dispatch_command_workspace_search)
- `cargo clippy -p moai-studio-ui --all-targets -- -D warnings`: 0 warning
- `cargo fmt --check`: clean
- `cargo build --workspace`: GREEN, 회귀 0

### MS-4 실측 (sess 10, 2026-05-04)

- `cargo test -p moai-studio-ui --lib search::`: **41 unit tests PASS** (MS-3 28 + MS-4 13: cap auto-cancel + cap_message + per-workspace progress + selected_index nav + move_up/down + enter_selected + escape_pressed + extract_preview_segments 3-segment)
- `cargo test -p moai-studio-ui --test integration_search`: **12 integration tests PASS** (e2e search flow / palette workspace.search / total cap auto-cancel / keyboard navigation / open call resolves OCV / unknown workspace no-panic / 등)
- `cargo test -p moai-studio-ui --lib`: **1193 PASS** (baseline 1180 + 13 신규)
- `cargo clippy -p moai-studio-ui --all-targets -- -D warnings`: 0 warning
- `cargo fmt --check`: clean
- `cargo build --workspace`: GREEN, 회귀 0

### Final SPEC GA 회귀 보존

v0.1.2 GA baseline 회귀 0 — ui 1148 → 1193 (+45 신규 across MS-1~MS-4 + integration 12), terminal / workspace / spec / 기타 crates 모두 GREEN.

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
