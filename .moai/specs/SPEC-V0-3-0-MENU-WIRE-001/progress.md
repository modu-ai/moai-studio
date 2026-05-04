# SPEC-V0-3-0-MENU-WIRE-001 — Progress

## Status

- 2026-05-04: 초안 작성 + MS-1 구현 완료. 8 AC ALL ✅. ui crate 1312 → 1320 (+8 tests). clippy/fmt clean.

## Milestone Tracker

### MS-1 — 3 stub functional wire (✅ DONE)

| AC | Status | Note |
|----|--------|------|
| AC-MW-1 | ✅ | RootView default `sidebar_visible == true && banner_visible == true` |
| AC-MW-2 | ✅ | `toggle_sidebar_visible()` flips state |
| AC-MW-3 | ✅ | `toggle_banner_visible()` flips state |
| AC-MW-4 | ✅ | `reload_workspaces_from_storage()` replaces in-memory list (count returned) |
| AC-MW-5 | ✅ | reload preserves `active_id` when present in new list |
| AC-MW-6 | ✅ | reload resets `active_id` to most-recent (or None when empty) |
| AC-MW-7 | ✅ | `main_body(sidebar_visible: bool)` skips sidebar branch when false |
| AC-MW-8 | ✅ | cargo build/clippy/fmt + 1320 ui tests PASS, 회귀 0 |

### Implementation summary

- RootView 신규 필드 2개: `sidebar_visible: bool`, `banner_visible: bool` (default `true`).
- 신규 메서드 3개 (cx 무관):
  - `toggle_sidebar_visible(&mut self)` — boolean flip
  - `toggle_banner_visible(&mut self)` — boolean flip
  - `reload_workspaces_from_storage(&mut self) -> Result<usize, WorkspaceError>` — `WorkspacesStore::load` 호출 + active_id 보존/재설정
- 3 stub action handler 본체 변경:
  - `ToggleSidebar` → `toggle_sidebar_visible() + cx.notify()`
  - `ToggleBanner` → `toggle_banner_visible() + cx.notify()`
  - `ReloadWorkspace` → `reload_workspaces_from_storage()` 호출 (Ok/Err 모두 info! 로그)
- `main_body` 시그니처에 `sidebar_visible: bool` 인자 추가 (6번째). false 일 때 sidebar element 미렌더.
- `Render::render` 의 banner_strip child 를 `self.banner_visible` 로 가드 (`.then(...)` Option 패턴).
- 8 신규 unit tests (T-MW 블록).

## Carry-Forward (별 SPEC)

5 잔존 stub (별 SPEC 으로 carry):

- `NewTerminalSurface` — Surface SPEC 의존
- `NewMarkdownSurface` — Surface SPEC 의존
- `NewCodeViewerSurface` — Surface SPEC 의존
- `ClosePane` — Pane SPEC 의존 (PaneTree::close_pane API 활용)
- `FocusNextPane` / `FocusPrevPane` — Pane SPEC 의존 (PaneTree focus traversal)

`dispatch_command` 의 `surface.*` / `pane.*` namespace 가 stub 으로 남아 있어, 선결 작업은 이 namespace 활성화 또는 직접 RootView 메서드 추가 (Surface/Pane SPEC 결정).

## Notes

- Lightweight SPEC fast-track 8번째 적용 (PR #86 PLUGIN-MGR / #90 TOOLBAR-WIRE / #91 ONBOARDING-ENV / #92 OSC8-LIFECYCLE MS-1 / #93 WIZARD-ENV MS-1 / #94 OSC8-LIFECYCLE MS-2 / #95 WIZARD-ENV MS-2 이후).
- main 세션 직접 fallback 적용 (sub-agent 1M context 회피, sess 13/14 패턴 재사용).
- audit Top 16 진척: F-2 (Native menu 잔존 stub) 9 stub 中 3 functional, 5 carry, 1 (`ToggleFind` / `ToggleTheme` / `OpenCommandPalette` / `OpenSpecPanel` / `SplitRight` / `SplitDown` 등은 V0-1-2-MENUS-001 에서 wire 완료).
- v0.3.0 cycle Sprint 1 두 번째 PR (PR #99 clippy fix 다음).
