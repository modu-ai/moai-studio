# SPEC-V0-3-0-PANE-WIRE-001 — Pane Action Stub Functional Wire (3 actions)

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-3-0-PANE-WIRE-001 |
| **Title** | Pane action carry — ClosePane / FocusNextPane / FocusPrevPane functional wire |
| **Status** | draft (Sprint 2 #5) |
| **Priority** | Medium |
| **Revision** | 1.0 (lightweight) |
| **Dependencies** | SPEC-V0-3-0-MENU-WIRE-001 (carry §6 carry-to "Pane SPEC"), SPEC-V0-3-0-SURFACE-MENU-WIRE-001 (sibling carry, Surface×3) |
| **Cycle** | v0.3.0 Sprint 2 (#5 — Pane×3 functional 변환) |
| **Milestones** | MS-1 |

## HISTORY

- 2026-05-04: 초안 작성. v0.3.0 cycle Sprint 2 #5 진입. SPEC-V0-3-0-MENU-WIRE-001 (#100 머지) 가 ToggleSidebar / ToggleBanner / ReloadWorkspace 3 stub 만 functional 변환하고 잔존 5 stub 을 follow-up 으로 carry 했다. SPEC-V0-3-0-SURFACE-MENU-WIRE-001 (#107) 이 Surface×3 을 처리했고, 본 SPEC 은 마지막 carry 묶음 인 Pane×3 (`ClosePane` / `FocusNextPane` / `FocusPrevPane`) 을 functional 로 변환한다. AC 수 (≤8) / milestones (≤2) 모두 lightweight 충족.

## 1. Purpose

`crates/moai-studio-ui/src/lib.rs:2191~2199` 의 3 pane action handler 는 현재 `info!("... — pane management/focus deferred")` log 만 남기는 stub 이다. 동시에 keymap (`cmd-w`, `cmd-]`, `cmd-[`, `lib.rs:3359~3361`), View 메뉴 (`lib.rs:3422~3425`), command palette 의 `pane.focus_next` / `pane.focus_prev` 엔트리 (`palette/registry.rs:148~149`) 도 모두 stub 으로 dispatch 된다. 본 SPEC 은 이 3 handler 를 **현재 focused pane 을 대상으로 PaneTree 동작을 호출** 하도록 functional 화한다.

핵심 설계: `PaneTree::close_pane(target_id) -> Result<(), SplitError>` 메서드는 이미 `panes/tree.rs:268` 에 구현되어 sibling 승계 로직을 담고 있고, `PaneTree::leaves() -> Vec<&Leaf<L>>` 는 in-order 순회를 보증한다 (`tree.rs:328`). `RootView::tab_container.active_tab().last_focused_pane: Option<PaneId>` 가 현재 focused pane 의 ID 를 보유한다 (`lib.rs:1332`, `1666`, `1923` 사례). 따라서 본 SPEC 은 **focus resolution + leaves 순회 routing 을 cx-free helper 로 분리** 하고, action handler 를 그 helper + cx-bound 적용 단계 호출로 교체하는 surgical change 만 수행한다.

## 2. Goals

- View → Close Pane (`cmd-w`) 메뉴/단축키가 현재 focused pane 을 `PaneTree::close_pane` 으로 닫고, sibling 승계 결과로 새 focused pane 을 갱신
- View → Focus Next Pane (`cmd-]`) 메뉴/단축키가 현재 focused pane 을 in-order leaves 의 다음 leaf 로 회전 (마지막이면 wrap-around)
- View → Focus Previous Pane (`cmd-[`) 메뉴/단축키가 현재 focused pane 을 in-order leaves 의 이전 leaf 로 회전 (첫번째면 wrap-around)
- Command palette `pane.close` (신규) / `pane.focus_next` / `pane.focus_prev` 가 동일 helper 를 호출 (parity)
- 단일 leaf state 에서 ClosePane 호출은 no-op (PaneTree::close_pane 의 기존 정책 따라감), FocusNext/Prev 도 no-op (회전 대상 부재)
- focused pane 이 부재 (`last_focused_pane = None`) 인 edge state 에서 panic 없이 무동작 + 경고 로그
- TRUST 5 gates (clippy / fmt / cargo test) ALL PASS, 기존 1361 tests 회귀 0 (additive only)

## 3. Non-Goals / Exclusions

- Surface×3 stub (`NewTerminalSurface` 등) — SPEC-V0-3-0-SURFACE-MENU-WIRE-001 에서 처리 (#107)
- 새 키바인딩 추가 (기존 cmd-w / cmd-] / cmd-[ 만 사용)
- 신규 split 명령어 (split.* namespace 무관)
- Pane focus visual indicator UI (별 SPEC, focus state 변경 후 cx.notify 만)
- Leaf payload 변경 (LeafKind 무수정)
- Multi-tab 간 focus 이동 (tab 내부의 pane 간 회전만)

FROZEN (touch 금지):
- `crates/moai-studio-terminal/**`
- `crates/moai-studio-workspace/**`
- `crates/moai-studio-ui/src/panes/tree.rs::close_pane` 내부 로직 (호출만, 무수정)
- `crates/moai-studio-ui/src/panes/tree.rs::leaves` 내부 로직 (호출만, 무수정)
- 기존 `handle_search_open` / `handle_split_action` / `handle_new_*_surface_*` 동작

## 4. Requirements

- REQ-PW-001: RootView 는 `close_focused_pane(&mut self, cx: &mut Context<Self>)` helper 를 가진다. `tab_container` 의 `active_tab().last_focused_pane` 을 resolve 하고, `Some(pane_id)` 인 경우 `active_tab_mut().pane_tree.close_pane(&pane_id)` 를 호출한다. 호출 후 `pane_tree.root_pane_id().cloned()` 를 새 `last_focused_pane` 으로 set 한다 (closed leaf 가 사라진 경우 sibling 승계 결과). `cx.notify()` 를 호출하여 재렌더 트리거.
- REQ-PW-002: RootView 는 `focus_next_pane(&mut self, cx: &mut Context<Self>)` helper 를 가진다. `active_tab().pane_tree.leaves()` 의 in-order 리스트에서 현재 `last_focused_pane` 의 인덱스를 찾고, `(idx + 1) % len` 위치 leaf 의 PaneId 를 `last_focused_pane` 으로 set 한다. `cx.notify()` 호출.
- REQ-PW-003: RootView 는 `focus_prev_pane(&mut self, cx: &mut Context<Self>)` helper 를 가진다. `(idx + len - 1) % len` 위치 leaf 의 PaneId 를 `last_focused_pane` 으로 set 한다. `cx.notify()` 호출.
- REQ-PW-004: `ClosePane` action handler 는 REQ-PW-001 helper 를 호출한다 (info! deferred log 제거).
- REQ-PW-005: `FocusNextPane` action handler 는 REQ-PW-002 helper 를 호출한다.
- REQ-PW-006: `FocusPrevPane` action handler 는 REQ-PW-003 helper 를 호출한다.
- REQ-PW-007: `dispatch_command` 는 `pane.close` / `pane.focus_next` / `pane.focus_prev` 3 id 를 인식하여 각각 REQ-PW-001/002/003 helper 를 호출하고 `true` 를 반환한다. 알 수 없는 `pane.*` id 는 `false` 반환 (graceful degradation). palette `registry.rs` 에 `pane.close` 신규 entry 추가.
- REQ-PW-008: 3 helper 는 다음 edge state 에서 panic 없이 무동작 + warn 로그만 남긴다: (a) `last_focused_pane = None`, (b) `leaves().len() == 0` (이론상 불가하나 방어), (c) 단일 leaf 인 경우 FocusNext/Prev 는 자기 자신으로 회전 (no-op equivalent).

## 5. Acceptance Criteria

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-PW-1 | leaves 리스트 [A, B, C] + focused = A | cx-free helper `next_focus_in_leaves(&[A, B, C], &A)` 호출 | `Some(B)` 반환 | unit test (`next_focus_returns_next_leaf`) |
| AC-PW-2 | leaves 리스트 [A, B, C] + focused = C (마지막) | `next_focus_in_leaves(&[A, B, C], &C)` 호출 | `Some(A)` 반환 (wrap-around) | unit test (`next_focus_wraps_to_first`) |
| AC-PW-3 | leaves 리스트 [A, B, C] + focused = A (첫번째) | `prev_focus_in_leaves(&[A, B, C], &A)` 호출 | `Some(C)` 반환 (wrap-around) | unit test (`prev_focus_wraps_to_last`) |
| AC-PW-4 | leaves 리스트 [A, B, C] + focused = C | `prev_focus_in_leaves(&[A, B, C], &C)` 호출 | `Some(B)` 반환 | unit test (`prev_focus_returns_prev_leaf`) |
| AC-PW-5 | leaves 리스트 [A] + focused = A (단일 leaf) | next/prev 호출 | `Some(A)` 반환 (자기 자신, no-op equivalent) | unit test (`focus_rotation_single_leaf_is_self`) |
| AC-PW-6 | dispatch_command 의 알 수 없는 `pane.unknown_xxx` id | 호출 | `false` 반환, 어떤 helper 도 호출되지 않음 (cx-free routing 검증) | unit test (`dispatch_command_pane_unknown_returns_false`), `route_pane_command_to_kind("pane.close")` 등 routing 분리 |
| AC-PW-7 | leaves 리스트 [A, B] + focused 가 리스트에 없음 (orphan) | `next_focus_in_leaves(&[A, B], &orphan)` 호출 | `Some(A)` 반환 (첫 leaf 로 fallback) | unit test (`next_focus_orphan_falls_back_to_first`) |
| AC-PW-8 | cargo build/clippy/fmt + ui crate test | run | ALL PASS, 기존 1361 tests 회귀 0 (additive only, +7~9 신규 tests) | CI |

(AC 합계: 8. lightweight 한도 ≤8 충족.)

## 6. File Layout

| Path | Status | Note |
|------|--------|------|
| `crates/moai-studio-ui/src/lib.rs` | modified | 3 pane helper 메서드 추가, 3 action handler functional, dispatch_command 의 `pane.close` / `pane.focus_next` / `pane.focus_prev` 분기 활성화, cx-free 검증용 helper (`next_focus_in_leaves` / `prev_focus_in_leaves` / `route_pane_command_to_kind`) 노출, 신규 unit tests (T-PW 블록 ~7~9개) |
| `crates/moai-studio-ui/src/palette/registry.rs` | modified | `pane.close` CommandEntry 신규 추가 (label "Close Pane", category "Pane") |
| `.moai/specs/SPEC-V0-3-0-PANE-WIRE-001/spec.md` | created | 본 문서 |
| `.moai/specs/SPEC-V0-3-0-PANE-WIRE-001/progress.md` | created | run 진입 시 갱신 stub |

추가 파일 없음. `panes/tree.rs` 는 read-only — 기존 `close_pane` / `leaves` API 호출만.

FROZEN (touch 금지):
- `crates/moai-studio-terminal/**`
- `crates/moai-studio-workspace/**`
- `crates/moai-studio-ui/src/panes/tree.rs` (전체 read-only)
- 진행 중 SPEC (V3-004 / V3-005 / V3-014) 산출물

## 7. Test Strategy

ui crate `lib.rs::tests` 모듈에 신규 unit test 7~9개 추가 (T-PW 블록).

- AC-PW-1~5: cx-free helper `next_focus_in_leaves(&[PaneId], &PaneId) -> Option<PaneId>` / `prev_focus_in_leaves` 단위검증 (wrap-around / single leaf / orphan fallback).
- AC-PW-6: cx-free routing helper `route_pane_command_to_kind(&str) -> Option<PaneCommand>` 단위검증 (Some/None 분기).
- AC-PW-7: orphan focused pane 이 leaves 리스트에 없을 때 첫 leaf 로 fallback.

GPUI Entity 호출 (`tab_container.update(cx, |tc, _| ... )`) 자체는 cx 의존이 강하므로 **logic 분리 패턴** 을 적용 (SURFACE-MENU-WIRE-001 §7 동일):
1. `next_focus_in_leaves` / `prev_focus_in_leaves` 같은 cx-free 함수로 인덱스 회전 logic 분리
2. `route_pane_command_to_kind` 로 dispatch_command routing 분리
3. helper 본체는 cx-bound 이지만 routing + focus rotation 단위는 cx-free 로 분리하여 검증
4. cx-bound 부분은 기존 `handle_split_action` / `handle_new_*_surface_*` 패턴 동일한 정책으로 GPUI-level 검증 생략

회귀 검증: 기존 ui crate 1361 tests 무영향 (additive only).

본 SPEC run 단계에서 `cargo test -p moai-studio-ui --lib` + `cargo clippy -p moai-studio-ui` + `cargo fmt --check` 3 gate 통과 필수.

---

Version: 1.0.0 (lightweight)
Created: 2026-05-04
Cycle: v0.3.0 Sprint 2 #5
Carry-from: SPEC-V0-3-0-MENU-WIRE-001 §6 carry-to "Pane SPEC", SPEC-V0-1-2-MENUS-001 §3.1
Carry-to: (없음 — Sprint 2 carry chain 종결)
