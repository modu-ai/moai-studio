---
id: SPEC-V3-012
version: 1.0.0
status: implemented
created_at: 2026-04-26
updated_at: 2026-04-30
author: MoAI (manager-spec)
priority: High
issue_number: 0
depends_on: [SPEC-V3-001, SPEC-V3-002, SPEC-V3-003, SPEC-V3-004, SPEC-V3-009]
parallel_with: []
milestones: [MS-1, MS-2, MS-3, MS-4]
language: ko
labels: [phase-3, ui, palette, command, fuzzy, surface, brand]
revision: v1.0.0 (initial draft, Palette Surface — CmdPalette/CommandPalette/SlashBar 통합 module)
---

# SPEC-V3-012: Palette Surface — CmdPalette / CommandPalette / SlashBar + 공용 Scrim + Fuzzy Match

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-26 | 초안 작성. RG-PL-1 ~ RG-PL-9, AC-PL-1 ~ AC-PL-15, MS-1/MS-2/MS-3. tokens.json v2.0.0 round2_component.palette canonical 참조. moai-revisions.jsx (Round 2 시안) CmdPalette/CommandPalette/SlashBar 시각 사양 채택. 코드베이스 변경 — `crates/moai-studio-ui/src/palette/` 신규 모듈, `crates/moai-studio-ui/src/lib.rs` RootView overlay slot 1지점 한정. CLAUDE.local.md Enhanced GitHub Flow 정합. |

---

## 1. Overview

### 1.1 Purpose

Palette Surface 는 moai-studio 사용자가 키보드 중심으로 파일 / 명령 / slash command 에 빠르게 접근하는 통합 overlay UI 다. 본 SPEC 은 다음을 정의한다:

- 3 variant — `CmdPalette` (Cmd+P, file quick open) / `CommandPalette` (Cmd+Shift+P, command runner) / `SlashBar` (`/moai *`, slash command launcher)
- 3 variant 가 공유하는 backdrop — `Scrim` (theme-aware, click-to-dismiss)
- 3 variant 가 공유하는 core — `PaletteView` (input + list + keyboard navigation)
- 공통 fuzzy matcher — subsequence 기반, score + highlight 위치 반환
- RootView overlay slot 통합 + 글로벌 키바인딩 (Cmd+P / Cmd+Shift+P / `/`)

### 1.2 Enhanced GitHub Flow 와의 정합

CLAUDE.local.md §1 의 branch model 에 따라 본 SPEC 은 `feature/SPEC-V3-012-palette-surface` 브랜치에서 develop → squash merge 한다. MS-1/MS-2/MS-3 는 단일 feature branch 안에서 commit 단위로 진행한다 (multi-PR 분리 비목표).

### 1.3 References

- `IMPLEMENTATION-NOTES.md` v1.1 §14 C 항목 — Palette Surface scope.
- `.moai/design/from-claude-design/project/moai-revisions.jsx` — Round 2 시안 (CmdPalette / CommandPalette / SlashBar 컴포넌트 마크업).
- `.moai/design/tokens.json` v2.0.0 `round2_component.palette` — width/row_height/font_size/scrim color canonical.
- `crates/moai-studio-ui/src/design/{tokens,layout,typography}.rs` — 기존 Rust design module.
- `crates/moai-studio-ui/src/lib.rs` — RootView (palette overlay slot 통합 지점).
- `.moai/specs/SPEC-V3-009` — Tabs surface 와 동일 패턴 (Entity + Render + Theme-aware).
- `.moai/specs/SPEC-V3-012/research.md` — UX 패턴 / fuzzy 알고리즘 비교 / 위험.

---

## 2. Background and Motivation

상세는 `research.md` 참조. 요약:

- **키보드 중심 IDE 표준**: VS Code (Cmd+P / Cmd+Shift+P), Zed, Sublime Text, Atom 모두 동일 UX 채택. moai-studio 가 동일 mental model 을 제공하지 않으면 onboarding 마찰 발생.
- **3 variant 의 공통 구조**: input + filtered list + keyboard nav. Scrim + PaletteView 를 공통 core 로 분리하면 variant 별 코드 중복 제거 + 시각 일관성 확보.
- **Slash command 통합**: `/moai plan` / `/moai run` / `/moai sync` 등 MoAI 워크플로 명령을 SlashBar 로 노출하면 사용자가 별도 CLI 전환 없이 작업 가능.
- **브랜드 정합**: 모두의AI 청록 (#144a46 / #22938a) + Pretendard 9-weight + Scrim 반투명 — IMPLEMENTATION-NOTES.md v1.1 § Brand FROZEN 준수.

---

## 3. Goals and Non-Goals

### 3.1 Goals

- G1. `crates/moai-studio-ui/src/palette/` 신규 모듈에 `Scrim`, `PaletteView`, 3 variant, `fuzzy` matcher 가 추가된다.
- G2. `Scrim` 은 dark/light theme 감지 + 정의된 alpha (dark 0.55 / light 0.18) + 2px blur + z-index 20 으로 렌더된다.
- G3. `PaletteView` 는 width 600px / row_height 32px / input font-size 14px / list max-height 320px 정확히 일치한다.
- G4. 키보드 네비게이션 — ↑/↓ navigate, Enter select, Esc dismiss, Tab fall-through 없음 — 이 4 키 모두 동작한다.
- G5. fuzzy matcher 는 subsequence + score 기반이며 매칭된 글자 위치 (Vec<usize>) 를 반환하여 highlight 가능.
- G6. Highlight 는 accent-soft (PRIMARY_DARK alpha 0.20) em 스타일로 표현된다.
- G7. RootView 는 palette overlay slot 을 1지점에 한정하여 추가하며, 동시 표시는 1 variant 만 허용 (mutual exclusion).
- G8. 글로벌 키바인딩 — Cmd+P (CmdPalette), Cmd+Shift+P (CommandPalette), `/` in terminal pane (SlashBar) — 이 3 단축키가 RootView 수준에서 동작한다.
- G9. 단위 테스트가 각 모듈별로 존재한다 (Scrim render / PaletteView nav state / fuzzy correctness / 3 variant 각각).
- G10. Local 5 quality gates (test / clippy 0 / fmt PASS / bench / cargo check --release) PASS.
- G11. Test coverage 85%+ 가능한 모듈 분해 — `mod.rs` (re-exports only) + `scrim.rs` + `palette_view.rs` + `fuzzy.rs` + `variants/{cmd_palette,command_palette,slash_bar}.rs` (각 ~150 LOC 수준).

### 3.2 Non-Goals

- N1. **Palette content 의 실제 데이터 source 통합** — file index (CmdPalette) / command registry (CommandPalette) / slash command list (SlashBar) 는 mock data 로 처리. 실제 source 는 후속 SPEC.
- N2. **Multi-cursor / multi-select** — Enter 시 1 항목만 선택. Shift+Enter 등 멀티 선택은 비목표.
- N3. **Palette history / recent items** — 최근 사용 항목 prioritize 비목표. v0.2.0+ 후보.
- N4. **Custom command 등록 API** — CommandPalette 에 사용자 정의 명령 추가 메커니즘 비목표.
- N5. **Palette inside Pane (non-overlay)** — 항상 RootView overlay 로만 동작. Pane-local palette 비목표.
- N6. **Mobile / touch UX** — desktop keyboard 전용. touch event 비목표.
- N7. **Palette transition animation** — fade/slide 등 입퇴장 애니메이션 비목표 (즉시 표시/숨김).
- N8. **i18n** — 모든 placeholder / label 영문 한정. 로케일 별 다국어 비목표.
- N9. **Async data loading** — 데이터 source 는 동기 fetch. Spinner / loading state 비목표.
- N10. **Accessibility (ARIA / screen reader)** — 키보드 네비는 동작하나 ARIA role / live region 명시 비목표. v0.2.0+ 후보.

---

## 4. User Stories

- **US-PL-1**: 사용자가 Cmd+P 를 누르면 CmdPalette 가 화면 중앙 상단 (대략 상단 20% 위치) 에 600px 폭으로 표시된다.
- **US-PL-2**: 사용자가 Cmd+Shift+P 를 누르면 CommandPalette 가 동일 위치에 표시되며 첫 글자가 `>` prefix 로 보인다 (선택 사항 — Round 2 시안 추종).
- **US-PL-3**: 사용자가 terminal pane focus 상태에서 `/` 를 누르면 SlashBar 가 입력 라인 영역에 표시된다.
- **US-PL-4**: 사용자가 ↑/↓ 키로 list 의 항목을 이동하면 highlight 가 따라가고, Enter 시 선택, Esc 시 닫힌다.
- **US-PL-5**: 사용자가 입력 시 fuzzy match 결과가 list 에 즉시 반영되고, 매칭 글자가 시각적으로 강조된다.
- **US-PL-6**: 사용자가 Scrim 의 빈 영역 (palette container 외부) 을 클릭하면 palette 가 dismiss 된다.
- **US-PL-7**: 사용자가 light → dark theme 전환 시 Scrim alpha 와 palette container 색이 즉시 전환된다.
- **US-PL-8**: 사용자가 Cmd+P 후 Cmd+Shift+P 를 누르면 CmdPalette 가 닫히고 CommandPalette 가 열린다 (mutual exclusion).
- **US-PL-9**: 개발자가 `cargo test -p moai-studio-ui palette` 를 실행하면 모든 palette 모듈 단위 테스트가 PASS 한다.

---

## 5. Functional Requirements (EARS)

### 5.1 Scrim 요구사항

- **RG-PL-1** (State-driven): While the palette is visible, the system shall render a Scrim covering the entire root viewport at z-index 20.
- **RG-PL-2** (State-driven): While the active theme is dark, the Scrim shall use rgba(8, 12, 11, 0.55).
- **RG-PL-3** (State-driven): While the active theme is light, the Scrim shall use rgba(20, 30, 28, 0.18).
- **RG-PL-4** (Ubiquitous): The Scrim shall apply a 2px blur backdrop filter (or platform-equivalent fallback when blur is unsupported).
- **RG-PL-5** (Event-driven): When the user clicks on the Scrim outside the palette container bounds, the system shall emit a `dismiss_requested` event.

### 5.2 PaletteView 요구사항

- **RG-PL-6** (Ubiquitous): The PaletteView shall render with width 600px, row height 32px, input font-size 14px, and list max-height 320px.
- **RG-PL-7** (Event-driven): When the PaletteView gains focus, the system shall route subsequent keyboard input to the input field.
- **RG-PL-8** (Event-driven): When the user presses ArrowDown, the system shall move the list selection to the next item, wrapping to the first item if at the end.
- **RG-PL-9** (Event-driven): When the user presses ArrowUp, the system shall move the list selection to the previous item, wrapping to the last item if at the start.
- **RG-PL-10** (Event-driven): When the user presses Enter while the list has a selected item, the system shall emit an `item_selected` event with the selected entry payload.
- **RG-PL-11** (Event-driven): When the user presses Escape, the system shall emit a `dismiss_requested` event.
- **RG-PL-12** (State-driven): While the input text changes, the system shall re-run the fuzzy matcher and update the visible list within a single GPUI frame.

### 5.3 Fuzzy Matcher 요구사항

- **RG-PL-13** (Ubiquitous): The fuzzy matcher shall accept a query string and a candidate string, and return either `None` (no match) or `Some((score: i32, highlights: Vec<usize>))` where highlights are byte indices of matched characters in the candidate.
- **RG-PL-14** (Ubiquitous): The fuzzy matcher shall require all query characters to appear in the candidate in order (subsequence semantics) for a match to be returned.
- **RG-PL-15** (Ubiquitous): The fuzzy matcher shall score consecutive character matches higher than scattered matches, and exact-prefix matches higher than mid-string matches.
- **RG-PL-16** (Unwanted): If the query is empty, then the matcher shall return all candidates with score 0 and empty highlights (no filtering).

### 5.4 Variant 요구사항

- **RG-PL-17** (Optional): Where the CmdPalette variant is active, the system shall present a list sourced from a file index (mocked in MS-2; real source in a follow-up SPEC).
- **RG-PL-18** (Optional): Where the CommandPalette variant is active, the system shall present a list sourced from a command registry (mocked in MS-2).
- **RG-PL-19** (Optional): Where the SlashBar variant is active, the system shall present a list sourced from MoAI slash commands (e.g., `/moai plan`, `/moai run`, `/moai sync`).
- **RG-PL-20** (Ubiquitous): Each variant shall reuse the PaletteView core and Scrim without duplicating layout or keyboard logic.

### 5.5 RootView 통합 요구사항

- **RG-PL-21** (Event-driven): When the user presses Cmd+P with no palette visible, the system shall open the CmdPalette variant.
- **RG-PL-22** (Event-driven): When the user presses Cmd+Shift+P with no palette visible, the system shall open the CommandPalette variant.
- **RG-PL-23** (Event-driven): When the user presses `/` while a terminal pane has focus and no palette is visible, the system shall open the SlashBar variant.
- **RG-PL-24** (State-driven): While any palette variant is visible, opening another variant shall first dismiss the current variant (mutual exclusion — only one variant visible at a time).
- **RG-PL-25** (Unwanted): If the user presses a palette shortcut while editing text inside an input that is not the palette input, then the system shall suppress the shortcut and let the input field receive the key.

### 5.6 Brand / Token 정합 요구사항

- **RG-PL-26** (Ubiquitous): The palette container background shall use `tokens::brand::SURFACE_LIGHT` (light) or `tokens::neutral::N900` (dark).
- **RG-PL-27** (Ubiquitous): Highlight emphasis on matched characters shall use accent-soft styling derived from `tokens::brand::PRIMARY_DARK` with alpha 0.20.
- **RG-PL-28** (Ubiquitous): Typography shall use the Pretendard font family per `crates/moai-studio-ui/src/design/typography.rs`.

---

## 6. Acceptance Criteria

### MS-1 — Scrim + PaletteView core

- **AC-PL-1** (Scrim render): Given the root view and an active theme, when the Scrim Entity is rendered, then a full-viewport overlay at z-index 20 with the theme-correct rgba is produced. _Test_: `palette::scrim::tests::scrim_renders_dark_alpha_055`, `scrim_renders_light_alpha_018`.
- **AC-PL-2** (Scrim click dismiss): Given a visible Scrim, when a click event lands on a coordinate outside the palette container bounds, then a `dismiss_requested` event is emitted exactly once. _Test_: `palette::scrim::tests::click_outside_emits_dismiss`, `click_inside_does_not_emit`.
- **AC-PL-3** (PaletteView dimensions): Given a freshly-constructed PaletteView, when its layout is measured, then container width is 600px, row height is 32px, input font-size is 14px, list max-height is 320px (within 1px tolerance for fractional pixels). _Test_: `palette::palette_view::tests::dimensions_match_spec`.
- **AC-PL-4** (Keyboard nav state machine): Given a PaletteView with N items, when ArrowDown is pressed N+1 times, then selection cycles back to index 0; when ArrowUp is pressed from index 0, then selection wraps to N-1; when Enter is pressed with a valid selection, then `item_selected` is emitted with the entry payload; when Escape is pressed, then `dismiss_requested` is emitted. _Test_: `palette::palette_view::tests::nav_wraps`, `enter_emits_selected`, `escape_emits_dismiss`.
- **AC-PL-5** (Focus management): Given a freshly-opened PaletteView, when the open event handler runs, then the input field is focused and accepts text input on the next frame. _Test_: `palette::palette_view::tests::input_focused_on_open`.

### MS-2 — 3 variants + fuzzy match

- **AC-PL-6** (CmdPalette variant): Given a CmdPalette opened with a mocked file index of 5+ entries, when the user types a query that matches a subset, then the list filters to the matching subset and selection starts at index 0. _Test_: `palette::variants::cmd_palette::tests::filters_by_query`.
- **AC-PL-7** (CommandPalette variant): Given a CommandPalette opened with a mocked command registry, when the user types a query, then the list shows matching commands and Enter triggers `item_selected` with the command id. _Test_: `palette::variants::command_palette::tests::enter_dispatches_command`.
- **AC-PL-8** (SlashBar variant): Given a SlashBar opened, when the user types `pl`, then the list shows entries beginning with `/moai plan` (subsequence match). _Test_: `palette::variants::slash_bar::tests::filters_moai_commands`.
- **AC-PL-9** (Fuzzy correctness — subsequence): Given query "abc" and candidate "a_b_c", when fuzzy match runs, then it returns `Some((score, highlights))` with highlights pointing to the indices of 'a', 'b', 'c'. _Test_: `palette::fuzzy::tests::subsequence_match`.
- **AC-PL-10** (Fuzzy correctness — non-match): Given query "xyz" and candidate "abc", when fuzzy match runs, then it returns `None`. _Test_: `palette::fuzzy::tests::no_subsequence_returns_none`.
- **AC-PL-11** (Fuzzy scoring — consecutive bonus): Given query "abc" with candidates "abc_def" and "a_b_c", when both are matched, then the consecutive candidate scores higher than the scattered candidate. _Test_: `palette::fuzzy::tests::consecutive_scores_higher`.
- **AC-PL-12** (Fuzzy empty query): Given an empty query and any candidate, when fuzzy match runs, then it returns `Some((0, vec![]))`. _Test_: `palette::fuzzy::tests::empty_query_passthrough`.
- **AC-PL-13** (Highlight rendering): Given a matched candidate with highlight indices `[0, 2, 4]`, when the row is rendered, then the characters at those byte indices use accent-soft em styling and others use the default style. _Test_: `palette::palette_view::tests::highlight_uses_accent_soft`.

### MS-3 — RootView integration + global key bindings

- **AC-PL-14** (Cmd+P opens CmdPalette): Given a RootView with no palette visible and global focus, when Cmd+P is pressed, then CmdPalette becomes visible and Cmd+P pressed again while it is visible is a no-op (or toggles dismiss — implementation choice consistent with VS Code). _Test_: `lib::tests::cmd_p_opens_cmd_palette`.
- **AC-PL-15** (Mutual exclusion): Given a CmdPalette is visible, when Cmd+Shift+P is pressed, then CmdPalette is dismissed and CommandPalette becomes visible (single visible variant guaranteed). _Test_: `lib::tests::cmd_shift_p_replaces_cmd_palette`.

### Cross-cutting (TRUST 5)

- **AC-PL-T1** (Test coverage): `cargo tarpaulin -p moai-studio-ui --include-tests --out Stdout | grep '^palette/'` reports >= 85% line coverage on the `palette/` module subtree.
- **AC-PL-T2** (Lint): `cargo clippy -p moai-studio-ui --all-targets -- -D warnings` produces 0 warnings.
- **AC-PL-T3** (Format): `cargo fmt --all -- --check` reports no diff.
- **AC-PL-T4** (Build): `cargo check -p moai-studio-ui --release` succeeds.
- **AC-PL-T5** (Bench regression): `cargo bench -p moai-studio-ui` does not regress existing benchmarks beyond the standing 3.92µs baseline (no new bench required for palette in MS-1/2/3).

---

## 7. Milestone Breakdown

### MS-1 — Scrim + PaletteView core

**Scope**:
- `palette/mod.rs` — module skeleton, re-exports.
- `palette/scrim.rs` — Scrim Entity, theme-aware color derivation, click-to-dismiss event emission.
- `palette/palette_view.rs` — PaletteView Entity, container layout (600 / 32 / 14 / 320), input field, list rendering scaffold (variant-agnostic generic over item type).
- Keyboard navigation state machine (selection index, wrap, Enter/Esc handling).
- Unit tests: `scrim_renders_*`, `click_*_emits_*`, `dimensions_match_spec`, `nav_wraps`, `enter_emits_selected`, `escape_emits_dismiss`, `input_focused_on_open`.

**Definition of Done**:
- AC-PL-1, AC-PL-2, AC-PL-3, AC-PL-4, AC-PL-5 PASS.
- `cargo test -p moai-studio-ui palette::scrim palette::palette_view` GREEN.
- AC-PL-T2, AC-PL-T3 PASS for the new files.

**Out of scope (deferred to MS-2)**: actual variant implementations, fuzzy matcher, highlight rendering.

### MS-2 — 3 variants + fuzzy match

**Scope**:
- `palette/fuzzy.rs` — subsequence matcher, scoring (consecutive bonus, prefix bonus), highlight position vector.
- `palette/variants/mod.rs` + `cmd_palette.rs` + `command_palette.rs` + `slash_bar.rs` — each variant constructs a PaletteView with variant-specific data source (mocked).
- Highlight rendering integration in `palette_view.rs` — accent-soft em styling on matched indices.
- Unit tests: `subsequence_match`, `no_subsequence_returns_none`, `consecutive_scores_higher`, `empty_query_passthrough`, `filters_by_query`, `enter_dispatches_command`, `filters_moai_commands`, `highlight_uses_accent_soft`.

**Definition of Done**:
- AC-PL-6 ~ AC-PL-13 PASS.
- AC-PL-T1 (coverage 85%+ on `palette/`) PASS once MS-2 closes.
- `cargo test -p moai-studio-ui palette` GREEN.

**Out of scope (deferred to MS-3)**: RootView integration, global key bindings, mutual exclusion across variants.

### MS-3 — RootView integration + global key bindings

**Scope**:
- `crates/moai-studio-ui/src/lib.rs` — RootView state extension: `active_palette: Option<PaletteVariant>` + overlay slot rendering.
- Global key handler — Cmd+P / Cmd+Shift+P / `/` (in terminal pane focus) → variant open events.
- Mutual exclusion logic — opening a variant dismisses any currently visible variant.
- Integration tests in `lib::tests::*` — keypress → palette open → mock select → palette close.

**Definition of Done**:
- AC-PL-14, AC-PL-15 PASS.
- AC-PL-T1 ~ AC-PL-T5 PASS for the entire crate.
- Bench (cargo bench) does not regress beyond the 3.92µs standing baseline.
- All 5 local quality gates GREEN.

**Out of scope (deferred to follow-up SPEC)**: real file index, real command registry, real slash command list, recent items, async loading.

---

## 8. File Inventory

### 8.1 New files

| Path | Purpose | LOC est. | Milestone |
|------|---------|----------|-----------|
| `crates/moai-studio-ui/src/palette/mod.rs` | Module re-exports + `PaletteVariant` enum | ~40 | MS-1 |
| `crates/moai-studio-ui/src/palette/scrim.rs` | Scrim Entity + theme color + click dismiss | ~150 | MS-1 |
| `crates/moai-studio-ui/src/palette/palette_view.rs` | PaletteView core (layout, input, list, kbd nav, highlight render) | ~300 | MS-1 + MS-2 |
| `crates/moai-studio-ui/src/palette/fuzzy.rs` | Subsequence fuzzy matcher | ~120 | MS-2 |
| `crates/moai-studio-ui/src/palette/variants/mod.rs` | Variant re-exports | ~20 | MS-2 |
| `crates/moai-studio-ui/src/palette/variants/cmd_palette.rs` | CmdPalette variant + mock file index | ~150 | MS-2 |
| `crates/moai-studio-ui/src/palette/variants/command_palette.rs` | CommandPalette variant + mock command registry | ~150 | MS-2 |
| `crates/moai-studio-ui/src/palette/variants/slash_bar.rs` | SlashBar variant + slash command list | ~150 | MS-2 |

Total new LOC estimate: **~1,080** (including tests inline).

### 8.2 Modified files

| Path | Change | Milestone |
|------|--------|-----------|
| `crates/moai-studio-ui/src/lib.rs` | Add `mod palette;` + `active_palette` state field + RootView overlay slot + global key handlers | MS-3 |

**Carry rule (RG-P-7 from V3-002~V3-009)**: No other file in `crates/` is modified. The palette feature is additive within `palette/` + 1-point integration in `lib.rs`.

---

## 9. Dependencies

### 9.1 Internal dependencies

- `crates/moai-studio-ui/src/design/tokens.rs` — color constants (brand, neutral, semantic).
- `crates/moai-studio-ui/src/design/typography.rs` — Pretendard font family.
- `crates/moai-studio-ui/src/design/layout.rs` — spacing scale (if used for padding).
- `crates/moai-studio-ui/src/lib.rs` — RootView (MS-3 integration target).

### 9.2 External dependencies

- `gpui` (workspace pinned 0.1) — Entity / View / KeyBinding / EventEmitter primitives.
- No new third-party crate. Fuzzy matcher is implemented in-tree (no `fuzzy-matcher` crate dependency added).

### 9.3 SPEC dependencies

- **SPEC-V3-001** — workspace structure, tokens.json canonical reference.
- **SPEC-V3-002** — design module (tokens.rs / typography.rs).
- **SPEC-V3-003** — TabContainer pattern (Entity + render + theme-aware) is the reference pattern for PaletteView.
- **SPEC-V3-004** — terminal pane focus model (SlashBar trigger condition).
- **SPEC-V3-009** — Tabs surface (SPEC-V3-009 Round 2 brand migration informs Palette brand application).

### 9.4 Quality gate dependencies

- TRUST 5 framework (Tested / Readable / Unified / Secured / Trackable).
- Local 5 gates — `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt -- --check`, `cargo bench`, `cargo check --release`.
- 85%+ test coverage on `palette/` subtree (AC-PL-T1).

---

## 10. Token Reference Map

Canonical token source is `.moai/design/tokens.json` v2.0.0 `round2_component.palette`. Rust constants in `design::tokens` mirror the canonical values. The mapping below documents the 1:1 correspondence; **no values are inlined in palette code** (always reference `design::tokens::*`).

| tokens.json path | Rust constant | Value | Used by |
|------------------|---------------|-------|---------|
| `round2_component.palette.scrim.dark` | (computed in `scrim.rs` from `neutral::N950`) | rgba(8,12,11,0.55) | `scrim.rs` |
| `round2_component.palette.scrim.light` | (computed in `scrim.rs` from `brand::INK`) | rgba(20,30,28,0.18) | `scrim.rs` |
| `round2_component.palette.container.bg.light` | `brand::SURFACE_LIGHT` (#ffffff) | 0xffffff | `palette_view.rs` |
| `round2_component.palette.container.bg.dark` | `neutral::N900` (#0e1513) | 0x0e1513 | `palette_view.rs` |
| `round2_component.palette.container.width` | constant `PALETTE_WIDTH = 600.0` in `palette_view.rs` | 600px | `palette_view.rs` |
| `round2_component.palette.row.height` | constant `ROW_HEIGHT = 32.0` | 32px | `palette_view.rs` |
| `round2_component.palette.input.font_size` | typography::INPUT_PALETTE | 14px | `palette_view.rs` |
| `round2_component.palette.list.max_height` | constant `LIST_MAX_HEIGHT = 320.0` | 320px | `palette_view.rs` |
| `round2_component.palette.highlight.alpha` | constant `HIGHLIGHT_ALPHA = 0.20` (over `brand::PRIMARY_DARK`) | 0.20 | `palette_view.rs` |
| `round2_component.palette.z_index` | constant `PALETTE_Z = 20` | 20 | `scrim.rs`, `palette_view.rs` |

If `tokens.json` does not yet contain `round2_component.palette` keys (current state per Round 2 import), the canonical values above are added to `tokens.json` as part of MS-1 token sync, **without** modifying any FROZEN brand color.

---

## 11. Risks and Mitigations

| Risk | Severity | Mitigation |
|------|----------|------------|
| GPUI 0.1 has no native backdrop-filter blur — Scrim 2px blur fallback | Medium | If blur unsupported, fall back to solid alpha overlay (no blur). RG-PL-4 explicitly allows platform-equivalent fallback. |
| Mutual exclusion logic produces flicker when switching variants | Low | Single-frame state transition: dismiss + open in same update cycle. AC-PL-15 covers via integration test. |
| Global key handler conflicts with Cmd+P inside terminal pane (terminal already binds Cmd+P) | Medium | RG-PL-25 — when an input owns focus, palette shortcuts are suppressed. `/` is the only terminal-context shortcut; Cmd+P / Cmd+Shift+P always escape to RootView per RG-PL-25 unless terminal explicitly captures them (terminal does not capture Cmd+P in moai-studio per current state). |
| Fuzzy matcher byte-index vs char-index confusion produces highlight off-by-one for non-ASCII | Medium | matcher returns byte indices and rendering converts to grapheme positions before applying styling. Test coverage with Korean / emoji candidates planned in MS-2. |
| List re-render performance on every keystroke for large item sets (10k+ files) | Low (mock data scope) | Mock data caps at ~100 items in MS-2. Real data source SPEC will add virtualization. |
| Palette opens during teardown / animation produces stale event | Low | Mutual exclusion is gated on `active_palette: Option<PaletteVariant>` — if `Some`, ignore second open until Scrim's dismiss fires. |
| Token sync introduces unintended diff in `tokens.json` | Low | MS-1 token PR is reviewed against `round2_component.palette` keys only; no other key is modified. |

---

## 12. Quality Gates

### 12.1 Local 5 gates (must PASS before commit)

| Gate | Command | Pass criterion |
|------|---------|----------------|
| Test | `cargo test -p moai-studio-ui` | All tests pass; new palette tests included |
| Clippy | `cargo clippy -p moai-studio-ui --all-targets -- -D warnings` | 0 warnings |
| Fmt | `cargo fmt --all -- --check` | No diff |
| Bench | `cargo bench -p moai-studio-ui` | No regression beyond 3.92µs baseline |
| Check (release) | `cargo check -p moai-studio-ui --release` | Builds clean |

### 12.2 TRUST 5 dimensions

- **Tested**: 85%+ line coverage on `palette/` (AC-PL-T1) + 15+ test cases across MS-1/2/3.
- **Readable**: Module structure follows `tabs/` and `panes/` precedent (Entity-per-file). Naming uses domain-specific terms (Scrim, PaletteView, CmdPalette, etc.).
- **Unified**: All token references via `design::tokens::*`. No hex literals in palette code. Pretendard via `design::typography`.
- **Secured**: No external user input is executed; SlashBar entries are static strings and emit events for the orchestrator to dispatch. No injection surface.
- **Trackable**: Conventional Commits — e.g., `feat(palette): MS-1 Scrim + PaletteView core (AC-PL-1..5)`. SPEC reference in commit body.

### 12.3 MX tag plan

- `palette/mod.rs` — `// @MX:NOTE: [AUTO] palette surface module entry — 3 variants share Scrim + PaletteView core.`
- `palette/scrim.rs` — `// @MX:ANCHOR: [AUTO] Scrim — fan_in target (3 variants + integration). @MX:REASON: theme-aware backdrop, click-to-dismiss contract.`
- `palette/palette_view.rs` — `// @MX:ANCHOR: [AUTO] PaletteView — fan_in target (3 variants). @MX:REASON: keyboard nav state machine + highlight render contract.`
- `palette/fuzzy.rs` — `// @MX:NOTE: [AUTO] subsequence fuzzy matcher with scoring + highlight indices.`
- `palette/variants/*.rs` — `// @MX:NOTE: [AUTO] {variant} variant — mock data source in MS-2; real source in follow-up SPEC.`

---

## 13. Exclusions (What NOT to Build)

- E1. No file index data layer — CmdPalette in MS-2 uses mock data only.
- E2. No command registry data layer — CommandPalette in MS-2 uses mock data only.
- E3. No persistence of recently used items.
- E4. No animation (fade / slide / scale).
- E5. No mouse hover selection (only keyboard ↑/↓ moves selection; Click selects-and-confirms in a follow-up).
- E6. No drag-and-drop reordering of palette entries.
- E7. No theming knobs beyond dark/light auto-switch.
- E8. No accessibility (ARIA roles, screen reader announcements) — deferred.
- E9. No async loading / spinner.
- E10. No modification to `crates/moai-studio-ui/src/{tabs,panes,terminal,viewer,explorer,agent}/` files — palette is additive.

---

## 14. Open Questions

(None for MS-1. The following are tracked for MS-2 / MS-3 and resolved by implementer with reasonable defaults if no orchestrator clarification arrives — Auto mode):

- Q1 (MS-2 SlashBar): which exact slash command list ships in mock? **Default**: `/moai plan`, `/moai run`, `/moai sync`, `/moai project`, `/moai fix`, `/moai design`. Aligns with CLAUDE.md §3.
- Q2 (MS-3 Cmd+P toggle vs no-op): VS Code toggles, Sublime closes-on-second-press. **Default**: toggle (open if closed, dismiss if same variant already visible).
- Q3 (MS-3 SlashBar terminal-pane focus only or terminal-text-input only): RG-PL-23 says "terminal pane has focus". **Default**: pane-level focus is sufficient; do not require text-input focus inside terminal.

---

## 15. Glossary

- **Palette Surface** — the umbrella term for the 3 variants + Scrim + PaletteView core.
- **Variant** — one of {CmdPalette, CommandPalette, SlashBar}.
- **Scrim** — the dimming backdrop covering the viewport behind a visible palette.
- **PaletteView** — the variant-agnostic container (input + list + nav).
- **Subsequence match** — every character of the query appears in the candidate in order, not necessarily contiguous.
- **Highlight** — the visual emphasis applied to candidate characters that matched the query.
- **Mutual exclusion** — only one palette variant is visible at any time; opening another dismisses the current.

---

---

## 8. MS-4 Acceptance Criteria (AC-PL-16 ~ AC-PL-22)

### 8.1 배경

MS-1~3 에서 palette surface 의 core, 3-variant, RootView 통합이 완성되었으나 CommandPalette 의 데이터 소스는 mock 10개에 불과했고, CmdPalette 는 단일 File 모드만 지원했으며 RootView 에 실 dispatch 로직이 없었다. MS-4 는 이 세 가지 gap 을 해소한다.

### 8.2 Acceptance Criteria

#### AC-PL-16: CommandRegistry — real structured data source

- **Given** `CommandPalette::new()` is called
- **When** the internal item list is built
- **Then** the list is sourced from `CommandRegistry::default_registry()`, not `default_mock_commands()`
- **And** the registry contains >= 30 entries across 10 categories (File, View, Pane, Tab, Workspace, Surface, Settings, Theme, Git, Agent)
- **And** all entry ids are namespaced (contain a dot, e.g. `pane.split_horizontal`)
- **And** no duplicate ids exist

#### AC-PL-17: dispatch_command — settings routing

- **Given** `RootView::dispatch_command("settings.open")` is called
- **When** the dispatch function runs
- **Then** `settings_modal` is mounted (not None)
- **And** `palette.active_variant` is None (palette is dismissed)
- **And** the function returns `true`

#### AC-PL-18: dispatch_command — theme routing

- **Given** `RootView::dispatch_command("theme.toggle")` is called
- **When** the dispatch function runs
- **Then** `active_theme` cycles (Dark → Light or Light → Dark)
- **And** the function returns `true`

- **Given** `RootView::dispatch_command("theme.dark")` is called
- **Then** `active_theme` is set to Dark, returns `true`

- **Given** `RootView::dispatch_command("theme.light")` is called
- **Then** `active_theme` is set to Light, returns `true`

#### AC-PL-19: dispatch_command — tab / pane / surface / workspace routing

- **Given** `RootView::dispatch_command(id)` is called for any id in {tab.*, pane.*, surface.*, workspace.*, git.*, agent.*, file.*, view.*}
- **When** the dispatch function runs
- **Then** the function returns `true` (logged, delegated to render cycle)
- **And** the palette is dismissed

#### AC-PL-20: dispatch_command — unknown id

- **Given** `RootView::dispatch_command("nonexistent.command")` is called
- **When** the dispatch function runs
- **Then** the function returns `false`
- **And** a warning is emitted via `tracing::warn!`

#### AC-PL-21: inject_slash_command — terminal stdin injection

- **Given** `RootView::inject_slash_command("/moai plan")` is called
- **When** the injection function runs
- **Then** `pending_slash_injection` is set to `Some("/moai plan\n")`
- **And** the palette is dismissed
- **And** the function returns `true`

- **Given** `RootView::inject_slash_command("invalid")` is called (does not start with `/moai`)
- **Then** `pending_slash_injection` remains `None`, returns `false`

#### AC-PL-22: CmdPalette @mention mode switching

- **Given** `CmdPalette::set_query("@")` is called
- **When** the mode is detected
- **Then** `CmdPalette.mode` is `PaletteMode::Symbol`
- **And** the item list is populated from `MOCK_SYMBOLS`

- **Given** `CmdPalette::set_query("#")` is called
- **Then** `CmdPalette.mode` is `PaletteMode::Issue`
- **And** the item list is populated from `MOCK_ISSUES`

- **Given** `CmdPalette::set_query("src")` is called (no prefix)
- **Then** `CmdPalette.mode` is `PaletteMode::File`
- **And** the item list is filtered from the file index

### 8.3 Implementation Notes

- `registry.rs` 는 `palette/` 모듈 내 독립 파일. `CommandPalette::new()` 가 `CommandRegistry::default_registry()` 를 호출.
- `default_mock_commands()` 는 `#[deprecated]` 로 마킹하되 삭제하지 않음 (하위 호환성).
- `pending_slash_injection: Option<String>` 는 RootView 필드. render/update 루프에서 TerminalSurface 컨텍스트 가용 시 drain.
- `PaletteMode::detect(query: &str)` 는 첫 문자를 보고 `@` → Symbol, `#` → Issue, otherwise File.
- 실 file-index 소스 wiring (V3-PALETTE-001) 은 본 MS-4 범위 외.

### 8.4 Quality Gates (MS-4)

| Check | Command | Threshold |
|-------|---------|-----------|
| Tests | `cargo test -p moai-studio-ui` | 33+ new tests pass |
| Clippy | `cargo clippy -p moai-studio-ui --all-targets -- -D warnings` | 0 warnings |
| Fmt | `cargo fmt --all -- --check` | No diff |

---

End of SPEC-V3-012.
