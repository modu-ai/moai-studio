# SPEC-V0-3-0-SURFACE-MENU-WIRE-001 — Progress

| Field | Value |
|-------|-------|
| **SPEC** | SPEC-V0-3-0-SURFACE-MENU-WIRE-001 |
| **Status** | run-complete (MS-1 DONE) |
| **Cycle** | v0.3.0 Sprint 2 #4 |
| **Milestones** | MS-1 |

## Plan Phase

- [x] spec.md 작성 (2026-05-04)
- [x] progress.md stub 생성 (2026-05-04)
- [x] plan audit (skipped — lightweight SPEC, plan-auditor deferred)

## MS-1 (Implementation — 2026-05-04)

- [x] T-SMW-1: `route_surface_new_to_kind(&str) -> Option<SurfaceKind>` cx-free routing helper 추가
- [x] T-SMW-2: `new_terminal_surface_in_focused_pane` helper 추가 + `NewTerminalSurface` action handler functional
- [x] T-SMW-3: `new_markdown_surface_in_focused_pane` helper 추가 + `NewMarkdownSurface` action handler functional
- [x] T-SMW-4: `new_codeviewer_surface_in_focused_pane` helper 추가 + `NewCodeViewerSurface` action handler functional
- [x] T-SMW-5: `dispatch_command` 의 `surface.new.terminal` / `surface.new.markdown` / `surface.new.codeviewer` 3 분기 활성
- [x] T-SMW-6: `last_focused_pane = None` edge state 처리 (panic 없는 무동작, warn log)
- [x] T-SMW-7: 6 unit tests 추가 (T-SMW 블록) → AC-SMW-1~6 각각 대응
- [x] T-SMW-8: cargo test/clippy/fmt 3 gate PASS, 기존 1355 tests 회귀 0

### Implementation Summary

**Files modified:**
- `crates/moai-studio-ui/src/lib.rs` (+95 lines including 6 tests)

**New public API:**
- `SurfaceKind` enum (Terminal / Markdown / CodeViewer)
- `route_surface_new_to_kind(&str) -> Option<SurfaceKind>` (module-level cx-free)
- `RootView::new_terminal_surface_in_focused_pane(&mut self, cx)`
- `RootView::new_markdown_surface_in_focused_pane(&mut self, cx)`
- `RootView::new_codeviewer_surface_in_focused_pane(&mut self, cx)`
- `RootView::resolve_focused_pane_id(&self, cx) -> Option<PaneId>` (private)

**Action handler changes:**
- `NewTerminalSurface` stub → calls `new_terminal_surface_in_focused_pane`
- `NewMarkdownSurface` stub → calls `new_markdown_surface_in_focused_pane`
- `NewCodeViewerSurface` stub → calls `new_codeviewer_surface_in_focused_pane`

**dispatch_command surface.new.* routing:**
- `surface.new.terminal` → `route_surface_new_to_kind` → `Some(Terminal)` → `true`
- `surface.new.markdown` → `route_surface_new_to_kind` → `Some(Markdown)` → `true`
- `surface.new.codeviewer` → `route_surface_new_to_kind` → `Some(CodeViewer)` → `true`
- `surface.new.unknown_xxx` → `route_surface_new_to_kind` → `None` → `false`

### AC Validation Table

| AC | Test name | Result |
|----|-----------|--------|
| AC-SMW-1 | `resolve_focused_pane_id_returns_none_when_no_tab_container` | PASS |
| AC-SMW-2 | `dispatch_command_surface_new_terminal_routes_to_helper` | PASS |
| AC-SMW-3 | `dispatch_command_surface_new_markdown_routes_to_helper` | PASS |
| AC-SMW-4 | `dispatch_command_surface_new_codeviewer_routes_to_helper` | PASS |
| AC-SMW-5 | `dispatch_command_surface_new_unknown_returns_false` | PASS |
| AC-SMW-6 | `new_surface_helpers_no_op_when_no_focused_pane` | PASS |
| AC-SMW-7 | cargo test/clippy/fmt gates | PASS (1361 tests, 0 warnings) |

## Quality Gates (run-end)

- [x] cargo build -p moai-studio-ui PASS (implicit — tests compile)
- [x] cargo clippy -p moai-studio-ui --all-targets -- -D warnings PASS (0 warnings)
- [x] cargo fmt --check PASS
- [x] cargo test -p moai-studio-ui --lib PASS (1355 existing + 6 new = 1361 total, 0 failed)

## Sync Phase

- [ ] PR 생성 (base: main, label: type/feature, area/ui-shell, priority/p2-medium)
- [ ] auto-merge --squash 활성
- [ ] HISTORY 갱신 (PR# + main commit)

---

Created: 2026-05-04
Last Updated: 2026-05-04 (MS-1 run-complete)
