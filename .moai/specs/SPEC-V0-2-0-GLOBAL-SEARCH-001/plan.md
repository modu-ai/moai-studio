---
id: SPEC-V0-2-0-GLOBAL-SEARCH-001-PLAN
spec: SPEC-V0-2-0-GLOBAL-SEARCH-001
version: 1.0.0
status: ready
created_at: 2026-05-02
author: MoAI (sess 9 main session, post annotation iteration 1)
parent: spec.md v1.1.0-ready
methodology: TDD (per .moai/config/sections/quality.yaml development_mode)
---

# SPEC-V0-2-0-GLOBAL-SEARCH-001 Implementation Plan

milestone × task × file × AC 매핑. 본 plan 은 spec.md v1.1.0-ready (USER-DECISION 3건 모두 (a) lock-in) 를 전제로 작성됨.

## 0. 전체 요약

| MS | 핵심 산출 | 산출 LOC (추정) | 의존 | 검증 AC |
|----|-----------|-----------------|------|---------|
| MS-1 | `crates/moai-search/` 신규 crate (engine domain) | ~250 LOC | none (workspace member 추가만) | AC-GS-1, AC-GS-2, AC-GS-3, AC-GS-4, AC-GS-5, AC-GS-6 |
| MS-2 | `crates/moai-studio-ui/src/search/` 모듈 (SearchPanel GPUI) | ~250 LOC | MS-1 | AC-GS-7, AC-GS-8, AC-GS-9, AC-GS-12 (UI) |
| MS-3 | navigation wire + Command Palette 갱신 | ~150 LOC | MS-1, MS-2, SPEC-V3-LINK-001, SPEC-V3-004, SPEC-V3-006 | AC-GS-10, AC-GS-11 |
| MS-4 | polish + integration test | ~150 LOC | MS-1, MS-2, MS-3 | 모든 AC final regression |
| **합계** | | **~800 LOC** | | 12 AC |

각 milestone 은 별 PR 권장 (CLAUDE.local.md §1.3 SPEC당 1 PR + auto-merge 유지). 본 plan 의 MS-1 PR 은 본 세션에서 진행. 후속 MS-2/3/4 는 후속 세션 또는 후속 PR 에서.

---

## 1. MS-1 Engine Crate (`crates/moai-search/`)

### 1.1 Scope

신규 crate `crates/moai-search/` — GPUI 의존 0 의 search engine domain. logic-only.

### 1.2 Files (신규)

| File | 추정 LOC | 책임 |
|------|---------|------|
| `crates/moai-search/Cargo.toml` | ~25 | manifest (workspace 의존 + ignore + regex + thiserror + tracing) |
| `crates/moai-search/src/lib.rs` | ~30 | re-export 퍼블릭 API + crate-level docstring |
| `crates/moai-search/src/types.rs` | ~80 | `SearchHit`, `SearchOptions`, `SearchError` struct/enum + Default + 단위 테스트 |
| `crates/moai-search/src/cancel.rs` | ~40 | `CancelToken` (`Arc<AtomicBool>` wrapper) + clone/cancel/is_cancelled + 단위 테스트 |
| `crates/moai-search/src/matcher.rs` | ~60 | literal substring + regex auto-detect + compile-fail fallback + 단위 테스트 |
| `crates/moai-search/src/walker.rs` | ~120 | `walk_workspace` (ignore::WalkBuilder + binary skip + cap 적용 + cancel poll) + 단위 테스트 |
| `crates/moai-search/src/session.rs` | ~50 | `SearchSession` (worker handle 모음 + lifecycle) + 단위 테스트 |

총 신규 LOC ~ 405 (테스트 포함). prod LOC ~ 250.

### 1.3 Files (수정)

| File | 변경 | 이유 |
|------|------|------|
| `Cargo.toml` (root) | `members = ["crates/*"]` 가 이미 wildcard 이므로 자동 인식. workspace `[workspace.dependencies]` 에 `ignore = "0.4"`, `regex = "1"` 추가. | USER-DECISION-A (a) + B (a) |

### 1.4 Tasks (TDD RED-GREEN-REFACTOR)

| T# | 작업 | RED test | GREEN code | AC |
|----|------|----------|------------|----|
| T1 | `Cargo.toml` manifest + `lib.rs` skeleton 작성 + workspace 인식 검증 | (없음 — `cargo build -p moai-search` 통과 검증) | manifest + lib.rs + 빈 modules | (build 게이트) |
| T2 | `types.rs` `SearchOptions::default()` 검증 (cap defaults = 50/200/1000, case_sensitive=false) | `test_search_options_defaults` | struct + Default impl | AC-GS-1 (일부) |
| T3 | `types.rs` `SearchHit` 필드 + clone/debug 검증 | `test_search_hit_fields_and_clone` | struct + derives | AC-GS-1 (일부) |
| T4 | `cancel.rs` `CancelToken::new()` + `cancel()` + `is_cancelled()` + clone share | `test_cancel_token_default_false`, `test_cancel_token_clone_shares_state`, `test_cancel_propagates_after_cancel_call` | `Arc<AtomicBool>` wrapper | AC-GS-1 (일부) |
| T5 | `matcher.rs` literal substring 검증 (case-insensitive default) | `test_literal_substring_match`, `test_literal_case_insensitive` | `Matcher::Literal` variant + `is_match` | AC-GS-12 (일부) |
| T6 | `matcher.rs` regex meta-character auto-detect + compile + fallback | `test_regex_auto_detect`, `test_regex_compile_failure_fallback_to_literal` | `Matcher::Regex` variant + auto-detect heuristic | AC-GS-12 (regex meta + fallback) |
| T7 | `walker.rs` `walk_workspace` happy path (3 파일 tempdir, query "use" → 2 hits) | `test_walk_workspace_happy_path` | `WalkBuilder::new(root).standard_filters(true).build()` + line iter + matcher | AC-GS-2 |
| T8 | `walker.rs` gitignore + custom exclude (target/node_modules/dist/log) | `test_walk_workspace_respects_gitignore`, `test_walk_workspace_custom_excludes` | `OverrideBuilder` 또는 `add_custom_ignore_filename` 합성 | AC-GS-3 |
| T9 | `walker.rs` binary file skip (NUL byte heuristic, 첫 8KB) | `test_walk_workspace_skips_binary_files` | `is_binary(first_8kb)` helper + skip | AC-GS-4 |
| T10 | `walker.rs` cap 적용 — per-file 50, per-workspace 200, total 1000 | `test_walk_workspace_per_file_cap`, `test_walk_workspace_per_workspace_cap`, `test_walk_workspace_total_cap_auto_cancels` | counter + early break | AC-GS-6 |
| T11 | `walker.rs` cancel mid-walk — 50ms 후 cancel → break check | `test_walk_workspace_cancel_mid_walk` | cancel.is_cancelled() poll per file + per line | AC-GS-5 |
| T12 | `session.rs` `SearchSession::new()` + `spawn_workers(workspaces)` + `cancel_all()` | `test_search_session_spawn_and_cancel` | `Vec<JoinHandle>` + cancel propagation | AC-GS-1 (session) |

각 task 는 RED → GREEN → REFACTOR 1 cycle. RED 단계의 단위 테스트가 spec.md AC 의 실 검증 source.

### 1.5 Quality Gates

- `cargo build -p moai-search` GREEN (no warning)
- `cargo test -p moai-search` 모든 단위 테스트 PASS
- `cargo clippy -p moai-search -- -D warnings` 0 warning
- `cargo fmt -p moai-search --check` clean
- `cargo build --workspace` v0.1.2 baseline 회귀 0 (1148 ui tests 보존, 다른 crate 테스트도 GREEN)

### 1.6 Out of Scope (MS-1 에서 절대 안 함)

- GPUI Entity 정의 (MS-2)
- 사이드바 RootView 통합 (MS-2)
- 키바인딩 wire (MS-2)
- workspace activate / new tab / line jump (MS-3)
- Command Palette 갱신 (MS-3)
- match highlight rendering (MS-4)
- keyboard navigation (MS-4)

---

## 2. MS-2 SearchPanel UI (`crates/moai-studio-ui/src/search/`)

### 2.1 Scope

GPUI Entity 측 SearchPanel + 사이드바 toggleable section + ⌘⇧F dispatch.

### 2.2 Files (신규)

| File | 추정 LOC | 책임 |
|------|---------|------|
| `crates/moai-studio-ui/src/search/mod.rs` | ~30 | re-export + module-level docstring |
| `crates/moai-studio-ui/src/search/panel.rs` | ~150 | `SearchPanel` Entity (input, cancel button, result list, status line) + state machine |
| `crates/moai-studio-ui/src/search/result_view.rs` | ~80 | result row 2-line layout (workspace_name/rel_path:line + preview) + match highlight |
| `crates/moai-studio-ui/src/search/keymap.rs` | ~30 | ⌘⇧F (macOS) / Ctrl+Shift+F (other) keymap action 등록 |

### 2.3 Files (수정)

| File | 변경 |
|------|------|
| `crates/moai-studio-ui/src/lib.rs` | `RootView` 에 `search_panel: Option<Entity<SearchPanel>>` 필드 추가 (기존 필드 rename/delete 금지 R3). `RootView::new` 초기화. 사이드바 영역에 `SearchPanel` mount toggle. |
| `crates/moai-studio-ui/Cargo.toml` | `[dependencies]` 에 `moai-search = { path = "../moai-search" }` 추가 |

### 2.4 Tasks (TDD)

| T# | 작업 | AC |
|----|------|----|
| T1 | `mod.rs` skeleton + Cargo.toml 의존 추가 + 빌드 통과 | (build) |
| T2 | `SearchPanel` Entity 정의 + `is_visible()` / `toggle()` / `focus_input()` API | AC-GS-7 (logic 부분) |
| T3 | input field state + `set_query(s)` + empty trim → result clear | AC-GS-9 (empty query no-spawn) |
| T4 | result list state machine (`Empty` / `Searching` / `HasResults` / `NoMatches` / `CapReached`) + status line render | AC-GS-9 (status) |
| T5 | result row 2-line layout (`render_result_row`) + match highlight (preview text 의 match_start..match_end 강조) | AC-GS-8 (row layout) |
| T6 | batch flush 1000ms / 100 hits — 결과 stream → buffer → batch update | AC-GS-8 (batch) |
| T7 | ⌘⇧F (macOS) / Ctrl+Shift+F (other) keymap action 정의 + RootView 통합 | AC-GS-7 (key dispatch) |
| T8 | edge cases — 0 workspace input disabled + placeholder, 1 workspace 단일 그룹 | AC-GS-12 (UI 측) |

### 2.5 Out of Scope (MS-2 에서 절대 안 함)

- 결과 click navigation (MS-3)
- Command Palette `workspace.search` entry handler (MS-3)
- ↑↓ keyboard navigation (MS-4)
- per-workspace progress spinner (MS-4)

---

## 3. MS-3 Navigation Wire

### 3.1 Scope

SearchHit click → workspace activate + new tab + line jump. Command Palette `workspace.search` entry handler dispatch + label/keybinding 갱신.

### 3.2 Files (신규)

| File | 추정 LOC | 책임 |
|------|---------|------|
| `crates/moai-studio-ui/src/search/navigation.rs` | ~80 | `SearchHit` → `OpenCodeViewer { path, line, col }` adapter + dispatch |

### 3.3 Files (수정)

| File | 변경 |
|------|------|
| `crates/moai-studio-ui/src/search/result_view.rs` | row click handler 추가 — `navigation::open_hit(hit, cx)` 호출 |
| `crates/moai-studio-ui/src/search/panel.rs` | row click event → navigation 호출 wiring |
| `crates/moai-studio-ui/src/palette/registry.rs` | `workspace.search` entry: label "Search in all workspaces", keybinding `Some("Cmd+Shift+F")` 갱신 + handler dispatch (SearchPanel toggle) |
| `crates/moai-studio-ui/src/lib.rs` | RootView 에 `handle_search_open(hit)` 메서드 추가 — `WorkspacesStore::touch` + `TabContainer::new_tab(LeafKind::Code(rel_path))` + scroll-to-line dispatch |

### 3.4 Tasks (TDD)

| T# | 작업 | AC |
|----|------|----|
| T1 | `navigation.rs` `hit_to_open_code_viewer(hit, store)` adapter — workspace_id resolve + path 결합 | AC-GS-10 (logic) |
| T2 | `WorkspacesStore::touch(workspace_id)` 호출 검증 (mock store) | AC-GS-10 |
| T3 | `TabContainer::new_tab(LeafKind::Code(path))` 호출 검증 (mock container) | AC-GS-10 |
| T4 | `OpenCodeViewer { path, line, col }` dispatch 검증 (line/col 정확) | AC-GS-10 |
| T5 | navigation 실패 시 panic 안 함 — workspace 누락 / file read fail / tab fail 모두 tracing warn + status bar 보고 | AC-GS-10 (안정성) |
| T6 | `palette/registry.rs` `workspace.search` entry 갱신 — label + keybinding | AC-GS-11 |
| T7 | Command Palette entry select dispatch → SearchPanel `toggle() + focus_input()` | AC-GS-11 |

### 3.5 Out of Scope (MS-3 에서 절대 안 함)

- ↑↓ keyboard navigation (MS-4)
- match highlight rendering polish (MS-4)
- integration test (MS-4)

---

## 4. MS-4 Polish + Integration Test

### 4.1 Scope

backpressure + progress spinner + keyboard nav + match highlight polish + integration test.

### 4.2 Files (신규)

| File | 추정 LOC | 책임 |
|------|---------|------|
| `crates/moai-studio-ui/tests/integration_search.rs` | ~150 | end-to-end integration: SearchPanel toggle → query → results → click → navigation. mock `WorkspacesStore` + mock `TabContainer`. |

### 4.3 Files (수정)

| File | 변경 |
|------|------|
| `crates/moai-studio-ui/src/search/panel.rs` | total cap (1000) 도달 시 auto-cancel + "Too many results — narrow your query" 메시지 |
| `crates/moai-studio-ui/src/search/result_view.rs` | per-workspace progress spinner (worker 진행 중 인디케이터) + ↑↓ keyboard navigation |
| `crates/moai-studio-ui/src/search/result_view.rs` | match highlight polish — preview text 의 match_start..match_end 색상/굵기 강조 (token 재사용) |

### 4.4 Tasks (TDD)

| T# | 작업 | AC |
|----|------|----|
| T1 | total cap auto-cancel + 메시지 표시 | AC-GS-6 (UI 측) |
| T2 | per-workspace progress spinner (각 worker 의 진행 상태) | AC-GS-9 (Searching 강화) |
| T3 | ↑↓ keyboard navigation (Up / Down focus, Enter open, Esc close) | AC-GS-7 (확장) |
| T4 | match highlight polish (preview text 의 match span 강조) | AC-GS-8 (확장) |
| T5 | integration test `tests/integration_search.rs` — 12 AC final regression sweep | 모든 AC |

### 4.5 Out of Scope (MS-4 에서도 안 함, v0.2.1+ carry)

- Settings 의 SearchPanel default 옵션 (cap, case sensitivity, exclude pattern)
- regex / case sensitivity / word boundary toggle UI
- glob include 패턴 (`*.rs` 만)
- Replace
- 검색 history / saved searches
- tab reuse 정책

---

## 5. Risk Mitigation Plan

| 위험 | 완화 |
|------|------|
| 위험 1 — `ignore::WalkBuilder` Windows path 동작 | MS-1 T8 단위 테스트가 cross-platform path 검증 (forward slash + backward slash). Windows GPUI e2e 는 별 SPEC. |
| 위험 2 — 큰 monorepo (10k+ 파일) P2 미달 | MS-4 polish 단계에서 `WalkBuilder::threads(N)` 튜닝. 본 SPEC v1 은 5k 파일 P2 < 500ms 까지만 보장. |
| 위험 3 — workspace 전환 시 panes/tabs 상실 (D-3 PARTIAL) | MS-3 navigation 후 D-3 round-trip (PR #64) 의존 검증. round-trip 실패 사례 발견 시 본 SPEC 책임 외, D-3 별 SPEC. |
| Spike 0 — `ignore` + `regex` workspace 추가 빌드 통과 | MS-1 T1 의 `cargo build -p moai-search` 게이트로 검증. |
| Spike 1 — 신규 crate workspace member 등록 | MS-1 T1 의 `cargo build --workspace` 게이트로 검증. |
| Spike 2 — GPUI test-support fallback (SPEC-V3-005 carry) | MS-2/3 의 GPUI Entity 단위 테스트가 logic-level fallback 사용 (SPEC-V3-005 결정 일관). |

---

## 6. Test Strategy Per Milestone

### MS-1
- 단위: `cargo test -p moai-search` (12 tasks × 1-3 tests = ~25 tests)
- 통합: 없음 (logic-only crate)
- 게이트: `cargo build -p moai-search`, clippy, fmt

### MS-2
- 단위: `cargo test -p moai-studio-ui --lib search::tests` (8 tasks × 1-2 tests = ~12 tests)
- 게이트: 워크스페이스 전체 회귀 0 (1148 baseline 보존)

### MS-3
- 단위: `cargo test -p moai-studio-ui --lib search::navigation::tests` (~5 tests)
- 단위: `cargo test -p moai-studio-ui --lib palette::registry::tests` (entry 갱신 검증)
- 게이트: 워크스페이스 전체 회귀 0

### MS-4
- integration: `cargo test -p moai-studio-ui --test integration_search` (~10 시나리오)
- 게이트: 12 AC 모두 PASS, 워크스페이스 전체 회귀 0

---

## 7. Branch / PR Strategy

- 본 plan + MS-1 → 단일 PR (`feature/SPEC-V0-2-0-GLOBAL-SEARCH-001-ms1-engine`)
- MS-2 → 별 feature 브랜치 + 별 PR
- MS-3 → 별 feature 브랜치 + 별 PR
- MS-4 → 별 feature 브랜치 + 별 PR

각 PR 라벨: `type/feature`, `area/search`, `priority/p1-high` (audit Top 1 #1), `release/minor` (v0.2.0 cycle).

CLAUDE.local.md §1.3 / §4 강제 준수: feature 브랜치 → main 직접 머지 (squash).

---

## 8. Estimated Sprint Footprint

audit §7 sprint 매핑: Sprint 4 (D-4 Global search) 단일 sprint 내 4 PR.

본 SPEC 만으로 약 1 주 sprint 추정. 단일 세션 진행 시 MS-1 ~ MS-2 까지 가능, MS-3/4 는 별 세션.

---

## 9. References

- spec.md v1.1.0-ready (USER-DECISION 3건 RESOLVED)
- research.md (4 검색 엔진 비교, 인덱싱 전략, 동시성 패턴)
- progress.md (annotation iteration 1 완료)
- `.moai/specs/RELEASE-V0.2.0/feature-audit.md` §3 Tier D + §4 Top 1
- `.moai/specs/SPEC-V3-LINK-001/spec.md` §4 (`OpenCodeViewer`)
- `.moai/specs/SPEC-V3-005/spec.md` §6 (Render Entity 분리 패턴)
- `.moai/specs/SPEC-V3-004/spec.md` (workspace switch persistence)
- `crates/moai-studio-workspace/src/lib.rs:181` (`WorkspacesStore::list/touch`)
- `crates/moai-studio-ui/src/palette/registry.rs:154` (`workspace.search` entry)
- `crates/moai-studio-ui/src/tabs/container.rs` (`TabContainer::new_tab`)
