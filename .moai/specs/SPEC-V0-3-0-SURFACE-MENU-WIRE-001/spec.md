# SPEC-V0-3-0-SURFACE-MENU-WIRE-001 — Surface Menu Stub Functional Wire (3 actions)

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-3-0-SURFACE-MENU-WIRE-001 |
| **Title** | Surface menu carry — NewTerminalSurface / NewMarkdownSurface / NewCodeViewerSurface functional wire |
| **Status** | draft (Sprint 2 #4) |
| **Priority** | Medium |
| **Revision** | 1.0 (lightweight) |
| **Dependencies** | SPEC-V0-3-0-MENU-WIRE-001 (carry §6 carry-to "Surface SPEC"), SPEC-V0-1-2-MENUS-001 (carry §3.1 잔존 stub) |
| **Cycle** | v0.3.0 Sprint 2 (#4 — Surface×3 functional 변환) |
| **Milestones** | MS-1 |

## HISTORY

- 2026-05-04: 초안 작성. v0.3.0 cycle Sprint 2 #4 진입. SPEC-V0-3-0-MENU-WIRE-001 (#100 머지) 가 ToggleSidebar / ToggleBanner / ReloadWorkspace 3 stub 만 functional 변환하고 잔존 5 stub (Surface×3 + Pane×2) 을 follow-up SPEC 으로 carry 했다. 본 SPEC 은 그 중 Surface×3 (`NewTerminalSurface` / `NewMarkdownSurface` / `NewCodeViewerSurface`) 만 functional 로 변환한다. Pane×2 (`ClosePane` / `FocusNext/PrevPane`) 는 별 Pane SPEC 으로 carry. AC 수 (≤8) / milestones (≤2) 모두 lightweight 충족.

## 1. Purpose

`crates/moai-studio-ui/src/lib.rs:2200~2210` 의 3 surface action handler 는 현재 `info!("... — surface creation deferred")` log 만 남기는 stub 이다. 동시에 `dispatch_command("surface.toggle_terminal")` 같은 `surface.*` namespace 도 stub 응답 (true 반환) 만 하고 실제 surface 생성을 하지 않는다. 본 SPEC 은 이 두 진입점을 동일 helper 에 집결시켜 **현재 focused pane 의 leaf payload 를 신규 surface 로 교체** 하도록 functional 화한다.

핵심 설계: `LeafKind` enum 의 3 변종 (`Terminal(Entity<TerminalSurface>)`, `Markdown(Entity<MarkdownViewer>)`, `Code(Entity<CodeViewer>)`) 은 이미 SPEC-V3-006 에서 활성화 되어 있고, `RootView::leaf_payloads: HashMap<PaneId, LeafKind>` 에 mount 되는 패턴 (`handle_search_open` 등) 도 확립돼 있다. 따라서 본 SPEC 은 **신규 GPUI Entity 를 만드는 helper 메서드 3개** 를 노출하고, action handler 를 그 helper 호출로 교체하는 surgical change 만 수행한다.

## 2. Goals

- View → New Terminal 메뉴가 현재 focused pane 의 leaf 를 `LeafKind::Terminal(Entity<TerminalSurface>)` 로 교체
- View → New Markdown Viewer 메뉴가 현재 focused pane 의 leaf 를 `LeafKind::Markdown(Entity<MarkdownViewer>)` (빈 path) 로 교체
- View → New Code Viewer 메뉴가 현재 focused pane 의 leaf 를 `LeafKind::Code(Entity<CodeViewer>)` (빈 path) 로 교체
- `dispatch_command` 의 `surface.new.terminal` / `surface.new.markdown` / `surface.new.codeviewer` 가 동일 helper 를 호출 (parity)
- 새 surface 가 mount 된 pane 이 focused pane 으로 유지 (active_id / last_focused_pane 보존)
- 모든 helper 가 helper 끝에서 `cx.notify()` 를 호출하여 즉시 재렌더 트리거
- TRUST 5 gates (clippy / fmt / cargo test) ALL PASS, 기존 1355 tests 회귀 0

## 3. Non-Goals / Exclusions

- `ClosePane` / `FocusNextPane` / `FocusPrevPane` (별 Pane SPEC carry)
- Surface 내부 콘텐츠 렌더링 (Terminal PTY / Markdown render / CodeViewer syntax) — SPEC-V3-002 / V3-006 에서 이미 활성, 본 SPEC 은 entity 생성만
- 새 키바인딩 (메뉴 dispatch only, 단축키는 별 SPEC)
- Pane 분할 변경 (split.* namespace 무관)
- Surface state 의 영속화 (workspaces.json 저장 X)
- Multi-pane 의 동시 surface mount (single focused pane 만 대상)
- `LeafKind::Image` / `LeafKind::Web` / `LeafKind::Binary` 신규 진입점 (Surface×3 한정)

FROZEN (touch 금지):
- `crates/moai-studio-terminal/**` — terminal crate 내부 (PTY) 무수정, 기존 `TerminalSurface::new` API 만 호출
- `crates/moai-studio-workspace/**` — workspace 스키마 무수정
- `LeafKind` enum 정의 (`crates/moai-studio-ui/src/viewer/mod.rs:88~108`) 무수정
- 기존 `handle_search_open` / `handle_split_action` 의 동작 무수정

## 4. Requirements

- REQ-SMW-001: RootView 는 `new_terminal_surface_in_focused_pane(&mut self, cx: &mut Context<Self>)` helper 를 가진다. 현재 focused pane 의 `PaneId` 를 resolve 하고, 새 `Entity<TerminalSurface>` 를 `cx.new` 로 생성하여 `self.leaf_payloads` 에 `LeafKind::Terminal(entity)` 로 insert (replace) 한다. 마지막에 `cx.notify()` 를 호출한다.
- REQ-SMW-002: RootView 는 `new_markdown_surface_in_focused_pane(&mut self, cx: &mut Context<Self>)` helper 를 가진다. 빈 path 의 `Entity<MarkdownViewer>` 를 생성하여 동일하게 mount + notify 한다.
- REQ-SMW-003: RootView 는 `new_codeviewer_surface_in_focused_pane(&mut self, cx: &mut Context<Self>)` helper 를 가진다. 빈 path 의 `Entity<CodeViewer>` 를 생성하여 동일하게 mount + notify 한다.
- REQ-SMW-004: `NewTerminalSurface` action handler 는 REQ-SMW-001 helper 를 호출한다 (info! log 제거 또는 helper 내부로 이전).
- REQ-SMW-005: `NewMarkdownSurface` action handler 는 REQ-SMW-002 helper 를 호출한다.
- REQ-SMW-006: `NewCodeViewerSurface` action handler 는 REQ-SMW-003 helper 를 호출한다.
- REQ-SMW-007: `dispatch_command` 는 `surface.new.terminal` / `surface.new.markdown` / `surface.new.codeviewer` 3 id 를 인식하여 각각 REQ-SMW-001/002/003 helper 를 호출하고 `true` 를 반환한다. 알 수 없는 `surface.new.*` id 는 `false` 반환 (graceful degradation).
- REQ-SMW-008: 3 helper 는 focused pane 이 부재 (active_tab 의 `last_focused_pane` 가 None) 인 경우 무동작 (insert 생략) + 경고 로그만 남기고 panic 하지 않는다.

## 5. Acceptance Criteria

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-SMW-1 | RootView 가 default tab + focused leaf 를 가진 상태 | helper-level: `new_terminal_surface_in_focused_pane` 호출 (cx 필요) → 또는 helper 내부 helper `resolve_focused_pane_id` 만 단위검증 | focused leaf 의 PaneId 가 `Some(_)` 으로 정확히 해석됨 | unit test (`resolve_focused_pane_id_returns_active_tab_focus`) — cx-free helper 분리 |
| AC-SMW-2 | dispatch_command 호출 시 (surface.new.terminal) | (logic-level) `dispatch_command_no_cx_surface_new_terminal()` 호출 또는 분기 매칭 검증 | branch 매칭이 성공하고 helper 가 호출됨 (mock or counter 기반) | unit test (`dispatch_command_surface_new_terminal_routes_to_helper`) |
| AC-SMW-3 | dispatch_command 의 `surface.new.markdown` 분기 | 호출 | 매칭 성공 | unit test (`dispatch_command_surface_new_markdown_routes_to_helper`) |
| AC-SMW-4 | dispatch_command 의 `surface.new.codeviewer` 분기 | 호출 | 매칭 성공 | unit test (`dispatch_command_surface_new_codeviewer_routes_to_helper`) |
| AC-SMW-5 | dispatch_command 의 알 수 없는 `surface.new.unknown_xxx` id | 호출 | `false` 반환, 어떤 helper 도 호출되지 않음 | unit test (`dispatch_command_surface_new_unknown_returns_false`) |
| AC-SMW-6 | RootView 의 active tab 이 last_focused_pane = None 인 edge state | 3 helper 중 임의 1개 호출 | panic 없음, leaf_payloads 변동 없음, warn log 1건 | unit test (`new_surface_helpers_no_op_when_no_focused_pane`) |
| AC-SMW-7 | cargo build/clippy/fmt + ui crate test | run | ALL PASS, 기존 1355 tests 회귀 0 (additive only) | CI |

(AC 합계: 7. lightweight 한도 ≤8 충족.)

## 6. File Layout

| Path | Status | Note |
|------|--------|------|
| `crates/moai-studio-ui/src/lib.rs` | modified | 3 surface helper 메서드 추가, 3 action handler functional, dispatch_command 의 `surface.new.*` 3 분기 활성, cx-free 검증용 helper (예: `resolve_focused_pane_id`) 노출, 신규 unit tests (T-SMW 블록 ~6개) |
| `.moai/specs/SPEC-V0-3-0-SURFACE-MENU-WIRE-001/spec.md` | created | 본 문서 |
| `.moai/specs/SPEC-V0-3-0-SURFACE-MENU-WIRE-001/progress.md` | created | run 진입 시 갱신 stub |

추가 파일 없음. `viewer/mod.rs` (LeafKind), `terminal/mod.rs` (TerminalSurface), `viewer/markdown/*` (MarkdownViewer), `viewer/code/*` (CodeViewer) 는 read-only — 기존 생성 API 호출만.

FROZEN (touch 금지):
- `crates/moai-studio-terminal/**`
- `crates/moai-studio-workspace/**`
- `crates/moai-studio-ui/src/viewer/mod.rs:88~108` (`LeafKind` enum 정의)
- 진행 중 SPEC (V3-004 / V3-005 / V3-014) 산출물

## 7. Test Strategy

ui crate `lib.rs::tests` 모듈에 신규 unit test 6개 추가 (T-SMW 블록).

- AC-SMW-1: `resolve_focused_pane_id` cx-free helper 단위검증 (active_tab().last_focused_pane.clone() 직접 노출 또는 wrapper)
- AC-SMW-2~4: `dispatch_command_no_cx` 변종 또는 분기 helper (`route_surface_new_to_kind(&str) -> Option<SurfaceKind>` 같은 cx-free routing 함수) 로 매칭 검증
- AC-SMW-5: 알 수 없는 id → `route_surface_new_to_kind` 가 `None` 반환
- AC-SMW-6: `last_focused_pane = None` edge state 에서 helper 가 panic 없이 무동작 (직접 cx-free 분기 검증)

GPUI Entity 생성 (`cx.new(|_cx| TerminalSurface::new(...))`) 자체는 `cx` 의존이 강하므로 **logic 분리 패턴** 을 적용:
1. `route_surface_new_to_kind(&str) -> Option<SurfaceKind>` 같은 cx-free routing 함수 노출
2. helper 본체는 cx-bound 이지만 routing + focus resolution 단위는 cx-free 로 분리
3. cx-bound 부분은 기존 `handle_search_open` 패턴 (live integration test 미운영) 과 동일한 정책으로 GPUI-level 검증 생략

회귀 검증: 기존 ui crate 1355 tests 무영향 (additive only).

본 SPEC run 단계에서 `cargo test -p moai-studio-ui --lib` + `cargo clippy -p moai-studio-ui` + `cargo fmt --check` 3 gate 통과 필수.

---

Version: 1.0.0 (lightweight)
Created: 2026-05-04
Cycle: v0.3.0 Sprint 2 #4
Carry-from: SPEC-V0-3-0-MENU-WIRE-001 §6 carry-to "Surface SPEC", SPEC-V0-1-2-MENUS-001 §3.1
Carry-to: 별 Pane SPEC (잔존 2 stub: ClosePane / FocusNextPane / FocusPrevPane)
