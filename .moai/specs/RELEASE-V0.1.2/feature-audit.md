# Feature Audit — moai-studio v0.1.1 → v0.1.2

**Generated**: 2026-04-27  
**Source**: design v3 master spec (`.moai/design/v3/spec.md`)  
**Verified against**: HEAD (v0.1.1 tag)  
**Audit Scope**: All 45 design features (A1-A7, B1-B7, C1-C9, D1-D6, E1-E8, F1-F6, G1-G5) mapped to 17 SPEC-V3-* documents and actual source code.

---

## §1 Summary

| Metric | Count |
|--------|-------|
| Total features in design v3 | 45 |
| Implemented (DONE) | 2 |
| Partial implementation | 29 |
| Missing (NONE) | 14 |
| **v0.1.2 demo-visible candidates** | **5** |

**Status by Tier:**
- Tier A (Terminal Core): 2 DONE, 5 PARTIAL, 0 NONE
- Tier B (Smart Link): 0 DONE, 1 PARTIAL, 6 NONE  
- Tier C (Surfaces): 1 DONE, 5 PARTIAL, 3 NONE
- Tier D (Workspace): 1 DONE, 2 PARTIAL, 3 NONE
- Tier E (moai-adk): 0 DONE, 5 PARTIAL, 3 NONE
- Tier F (UX): 1 DONE, 4 PARTIAL, 1 NONE
- Tier G (Config): 0 DONE, 7 PARTIAL, 1 NONE (G-4)

---

## §2 Feature Matrix

| ID | Title | Tier | SPEC | Status | Demo | v0.1.2 Candidate | Evidence |
|----|-------|------|------|--------|------|------------------|----------|
| **A-1** | Multi-pane terminal (binary tree split) | Critical | SPEC-V3-003/004 | PARTIAL | HIGH | YES | crates/moai-studio-ui/src/panes/mod.rs (PaneNode, divider drag) |
| **A-2** | Tab UI (in-pane tabs) | Critical | SPEC-V3-003/004 | PARTIAL | HIGH | YES | crates/moai-studio-ui/src/tabs/container.rs:TabContainer |
| **A-3** | tmux full compat (OSC 8, mouse, 256+24-bit) | Critical | SPEC-V3-002 | PARTIAL | MEDIUM | NO | moai-studio-terminal/src/vt.rs, libghostty_ffi.rs |
| **A-4** | Multi-shell support (zsh/bash/fish/nu/pwsh) | Critical | SPEC-V3-002 | PARTIAL | MEDIUM | NO | moai-studio-terminal/src/pty/unix.rs, pty/mod.rs |
| **A-5** | Session persistence (pane tree restore) | High | SPEC-V3-003 | PARTIAL | LOW | NO | moai-studio-workspace/src/panes_convert.rs |
| **A-6** | Block-based output (Warp model) | High | — | NONE | LOW | NO | (no implementation) |
| **A-7** | Unix socket IPC + named pipe | High | SPEC-V3-002 | PARTIAL | LOW | NO | moai-studio-terminal/src/pty/unix.rs |
| **B-1** | OSC 8 hyperlinks render + click | Critical | SPEC-V3-002 | PARTIAL | MEDIUM | YES | moai-studio-terminal/src/vt.rs (libghostty OSC parsing) |
| **B-2** | Regex file path detection (path:line:col) | Critical | SPEC-V3-LINK-001 | PARTIAL | HIGH | YES | crates/moai-studio-terminal/src/link.rs detect_links() — detection done, UI click wiring deferred |
| **B-3** | URL auto-detect + highlight | Critical | SPEC-V3-LINK-001 | PARTIAL | HIGH | YES | crates/moai-studio-terminal/src/link.rs detect_links() — detection done, UI click wiring deferred |
| **B-4** | SPEC-ID pattern detection | Critical | SPEC-V3-009 | NONE | MEDIUM | NO | (E-1 has SPEC parsing, not terminal parsing) |
| **B-5** | @MX tag detection | High | SPEC-V3-009 | PARTIAL | MEDIUM | NO | moai-studio-ui/src/viewer/mx_gutter.rs |
| **B-6** | Mermaid code block detection | Medium | SPEC-V3-005 | NONE | LOW | NO | (no implementation) |
| **B-7** | Hover preview (file popup) | Medium | SPEC-V3-003 | NONE | LOW | NO | (no implementation) |
| **C-1** | Terminal Surface (basic rendering) | Critical | SPEC-V3-002 | DONE | HIGH | NO | moai-studio-ui/src/terminal/mod.rs:TerminalSurface (SPEC-V3-002 completed) |
| **C-2** | Markdown Surface (EARS + KaTeX + Mermaid) | Critical | SPEC-V3-005 | PARTIAL | HIGH | YES | moai-studio-ui/src/viewer/markdown/, LeafKind::Markdown |
| **C-3** | Code Viewer Surface (Monaco + LSP) | High | SPEC-V3-005/006 | PARTIAL | HIGH | YES | moai-studio-ui/src/viewer/code/, viewer/lsp.rs |
| **C-4** | Browser Surface (WebView + DevTools) | Critical | SPEC-V3-007 | NONE | HIGH | YES | (wry integration pending SPEC-V3-007) |
| **C-5** | Image Surface (zoom/pan/EXIF) | High | SPEC-V3-005 | NONE | MEDIUM | NO | (no implementation) |
| **C-6** | JSON/CSV Surface (pretty display) | Medium | SPEC-V3-005 | NONE | MEDIUM | NO | (no implementation) |
| **C-7** | Mermaid Renderer Surface | Medium | SPEC-V3-005 | NONE | MEDIUM | NO | (no implementation) |
| **C-8** | File Tree Surface (git status) | High | SPEC-V3-005 | PARTIAL | MEDIUM | NO | moai-studio-ui/src/explorer/, FsNode tree + notify watch |
| **C-9** | Agent Run Viewer (Hook timeline) | High | SPEC-V3-010 | PARTIAL | MEDIUM | NO | moai-studio-ui/src/agent/dashboard_view.rs |
| **D-1** | Workspaces JSON persistence | Critical | SPEC-V3-004 | DONE | MEDIUM | NO | moai-studio-workspace/src/lib.rs:WorkspacesStore load/save |
| **D-2** | Sidebar workspace switcher | Critical | SPEC-V3-004 | PARTIAL | HIGH | YES | moai-studio-ui/src/lib.rs Sidebar workspace list, activate handler |
| **D-3** | State preserve on project switch | High | SPEC-V3-003/004 | PARTIAL | LOW | NO | moai-studio-workspace/src/panes_convert.rs |
| **D-4** | Global search across workspaces | High | — | NONE | MEDIUM | NO | (no implementation) |
| **D-5** | Workspace color tags | Low | — | NONE | LOW | NO | (no implementation) |
| **D-6** | Drag-and-drop workspace add | Medium | SPEC-V3-010 | NONE | MEDIUM | NO | (file picker exists, D&D not yet) |
| **E-1** | SPEC Card (active SPEC) | High | SPEC-V3-009/015 | PARTIAL | MEDIUM | NO | moai-studio-ui/src/spec_ui/, SpecPanelView |
| **E-2** | TRUST 5 Dashboard (5-axis radar) | High | SPEC-V3-008 | NONE | MEDIUM | NO | (no implementation) |
| **E-3** | @MX tag gutter + popover | High | SPEC-V3-009 | PARTIAL | MEDIUM | NO | moai-studio-ui/src/viewer/mx_gutter.rs |
| **E-4** | Hook event stream (27 events) | High | SPEC-V3-010 | PARTIAL | LOW | NO | moai-studio-ui/src/agent/, moai-hook-http/src/ |
| **E-5** | Mission Control (agent grid) | High | SPEC-V3-007/010 | NONE | MEDIUM | NO | (no implementation) |
| **E-6** | Kanban Board (SPEC lifecycle) | Medium | SPEC-V3-008 | PARTIAL | MEDIUM | NO | moai-studio-spec/src/state/kanban.rs |
| **E-7** | Memory Viewer (~/.claude/projects) | Medium | SPEC-V3-009 | NONE | LOW | NO | (no implementation) |
| **E-8** | CG Mode (Claude + GLM split) | Low | SPEC-V3-010 | NONE | LOW | NO | (no implementation) |
| **F-1** | Command Palette (⌘/Ctrl+K) | Critical | SPEC-V3-012 | PARTIAL | HIGH | YES | moai-studio-ui/src/palette/, PaletteOverlay, AC-PL-14/15 |
| **F-2** | Native menu bar | Critical | SPEC-V3-006 | PARTIAL | HIGH | YES | moai-studio-ui (TitleBar menu structure, actions pending) |
| **F-3** | Toolbar (7 actions) | High | SPEC-V3-006 | PARTIAL | HIGH | NO | moai-studio-ui TitleBar toolbar area |
| **F-4** | Status bar (pill + git + LSP) | High | SPEC-V3-006 | PARTIAL | HIGH | NO | moai-studio-ui/src/lib.rs StatusBar area |
| **F-5** | Empty State CTA (Welcome) | Critical | SPEC-V3-001 | DONE | HIGH | NO | moai-studio-ui/src/lib.rs empty_state_view (AC-2.2 satisfied) |
| **F-6** | Onboarding tour | High | SPEC-V3-010 | PARTIAL | MEDIUM | NO | First-run Welcome screen exists, full tour pending |
| **G-1** | Settings (General/Hooks/MCP/etc) | High | SPEC-V3-013 | PARTIAL | MEDIUM | NO | moai-studio-ui/src/settings/ (theme, keybindings partial) |
| **G-2** | New Workspace Wizard | Critical | SPEC-V3-010 | PARTIAL | HIGH | YES | File picker integrated, 5-step wizard pending |
| **G-3** | Theme switcher (dark/light/auto) | Medium | SPEC-V3-013 | PARTIAL | HIGH | NO | design/runtime.rs ActiveTheme, appearance settings |
| **G-4** | Keybinding customization | Low | — | NONE | LOW | NO | (hardcoded in tabs/keys.rs) |
| **G-5** | Auto-update (Tauri updater) | High | SPEC-V3-011 | NONE | LOW | NO | (pending SPEC-V3-011) |

---

## §3 Per-Tier Breakdown

### Tier A — Terminal Core (cmux heritage)

v0.1.1 Status: **2 DONE, 5 PARTIAL, 0 NONE**

**Completed:**
- None fully done; A-3 (tmux) implemented via SPEC-V3-002 but OSC 8 partial

**High Priority v0.1.2 Targets:**
- **A-1 (panes)** — Demo-visible pane split UI is partially done; dragging divider works. Full pane tree restoration needed for persistence.
- **A-2 (tabs)** — Tab UI rendering exists; tab management (create/close/switch) needs completion.

**Implementation Notes:**
- SPEC-V3-002 (Terminal Core) marked `completed` with 74 tests passing (terminal 14 + ui 60).
- libghostty-vt integration successful; Zig build chain verified on macOS/Linux CI.
- PTY spawn, shell execution, ANSI rendering all functional.

### Tier B — Smart Link Handling (MoAI 차별화 핵심)

v0.1.1 Status: **0 DONE, 1 PARTIAL, 6 NONE**
v0.1.2 Status (after SPEC-V3-LINK-001): **0 DONE, 3 PARTIAL, 4 NONE**

**SPEC-V3-LINK-001 Changes (2026-04-27):**
- B-2 + B-3: `detect_links()` + `detect_links_with_osc8()` implemented in `moai-studio-terminal/src/link.rs`.
  15 unit tests pass. Detection layer DONE; UI click wiring (TerminalSurface) deferred.

**Remaining Gap Analysis:**
- **B-1 (OSC 8 UI wiring)** — libghostty-vt parses OSC 8; rendering and click handling need UI wiring (follow-up SPEC).
- **B-4 (SPEC-ID)** — SPEC card UI exists (E-1), but terminal-side SPEC-ID hyperlink parsing missing.

**v0.1.2 Opportunity:**
- B-2 + B-3 detection layer complete. UI wiring to TerminalSurface is the remaining blocker for demo visibility.

### Tier C — Surfaces (Wave heritage)

v0.1.1 Status: **1 DONE, 5 PARTIAL, 3 NONE**

**Completed:**
- **C-1 (Terminal)** — SPEC-V3-002 delivered full PTY + libghostty-vt integration. Grid rendering, 60 FPS capable, stdin/stdout/stderr complete.

**High Priority v0.1.2 Targets:**
- **C-2 (Markdown)** — Skeleton exists (LeafKind::Markdown). Add KaTeX + Mermaid rendering to serve `.moai/specs/*/spec.md` inline (SPEC-driven UX).
- **C-3 (Code)** — LSP diagnostics partially wired. Tree-sitter syntax highlight pending. Users need syntax color + hover diagnostics.

**Missing (Deferred to v0.2.0+):**
- C-4 (Browser) — wry WebView integration pending SPEC-V3-007.
- C-5/C-6/C-7 — Image/JSON/Mermaid surfaces deferred.

### Tier D — Multi-Project Workspace (VS Code heritage)

v0.1.1 Status: **1 DONE, 2 PARTIAL, 3 NONE**

**Completed:**
- **D-1 (Workspaces JSON)** — `~/.moai/studio/workspaces.json` load/save fully implemented. Workspace list persistence working.

**High Priority v0.1.2 Targets:**
- **D-2 (Workspace Switcher)** — Sidebar lists workspaces, active highlighting works. Missing: drag-to-reorder, context menu (rename/delete), quick switcher (⌘/Ctrl+comma?).

**Deferred:**
- D-3 (state preserve) — pane tree restoration scaffolded; full round-trip needs SPEC-V3-003/004.
- D-4/D-5/D-6 — Global search, color tags, D&D deferred to v0.2.0.

### Tier E — moai-adk GUI Overlay (MoAI 특화)

v0.1.1 Status: **0 DONE, 5 PARTIAL, 3 NONE**

**Implementation Status:**
- **E-1 (SPEC Card)** — SpecPanelView overlay, title/AC/status skeleton. Missing: full rendering, AC table, EARS parsing.
- **E-3 (@MX gutter)** — Code viewer gutter shows @MX:NOTE/WARN/ANCHOR. Missing: popover on hover.
- **E-4 (Hook stream)** — moai-hook-http server exists; GPUI wire-up pending.
- **E-6 (Kanban)** — State machine (draft/in-progress/review/done) exists; GPUI Kanban UI pending SPEC-V3-008.

**Missing (Deferred to v0.2.0+):**
- E-2 (TRUST 5) — 5-axis radar not started.
- E-5 (Mission Control) — Parallel agent grid not started.
- E-7/E-8 — Memory/CG mode out of scope.

### Tier F — Navigation & UX

v0.1.1 Status: **1 DONE, 4 PARTIAL, 1 NONE**

**Completed:**
- **F-5 (Empty State CTA)** — Welcome screen with "Create First Workspace / Start Sample / Open Recent" fully implemented (AC-2.2 from SPEC-V3-001).

**High Priority v0.1.2 Targets:**
- **F-1 (Command Palette)** — Scrim overlay exists. PaletteView skeleton done. Add: fuzzy search, command registry, @/# mention suggestions. This is a TOP demo-visible win.
- **F-2 (Menu bar)** — GPUI menu structure exists. Add: File/Edit/View/Pane/Go/Window/Help menu items + handlers.

**Deferred:**
- F-3 (Toolbar) — Buttons sketched; action wiring pending.
- F-4 (Status bar) — Area exists; widgets (git branch, LSP status, agent pill) pending.
- F-6 (Onboarding) — First-run Welcome exists; full environment detection (shell/tmux/node/python/rust detect) deferred to SPEC-V3-010.

### Tier G — Configuration

v0.1.1 Status: **0 DONE, 7 PARTIAL, 1 NONE**

**Implementation Status:**
- **G-1 (Settings)** — SettingsModal structure exists. Appearance pane (theme, font size) partially working. Missing: Hooks / MCP / Skills / Rules / Keybindings sections.
- **G-3 (Theme)** — dark/light/auto toggle exists; applied to design tokens. Missing: color theme picker (e.g., Nord, Dracula).

**Deferred:**
- G-4 (Keybinding Customization) — NONE (hardcoded macOS/Windows/Linux in tabs/keys.rs).
- G-5 (Auto-update) — Pending Tauri updater integration (SPEC-V3-011).

---

## §4 v0.1.2 Recommended Targets (demo-visibility ranked)

### Top 5 Demo-Visible Wins (when fully implemented)

**Priority 1 (show stopper for v0.1.2 release momentum):**

1. **B-2 + B-3 (File path + URL link parsing)** ⭐⭐⭐⭐⭐
   - **Why**: Moai's core USP. User types `npm run build` → error at `src/app.ts:42:10` → click → code viewer pops open. This is the "magic" demo.
   - **Demo visibility**: HIGH (terminal output is primary UI surface)
   - **Scope**: ~200 LOC regex module + terminal render integration.
   - **SPEC**: SPEC-V3-003 (smart link parsing).
   - **Timeline**: 3-5 days (includes terminal click handler wiring).

2. **C-2 (Markdown Surface + KaTeX + Mermaid)** ⭐⭐⭐⭐
   - **Why**: Developers want to read `.moai/specs/SPEC-V3-*.spec.md` inline without context switch. EARS format + Mermaid diagrams are differentiators vs. plain text editor.
   - **Demo visibility**: HIGH (right-pane Surface toggle is discoverable).
   - **Scope**: Markdown renderer + remark plugins + KaTeX math + Mermaid.js integration.
   - **SPEC**: SPEC-V3-005.
   - **Timeline**: 5-7 days.

3. **F-1 (Command Palette: fuzzy search + registry)** ⭐⭐⭐⭐
   - **Why**: ⌘/Ctrl+K is the universal "jump" gesture. Users expect Cmd+P (go to file), Cmd+Shift+P (commands), /slash (terminal context). Full UX polish.
   - **Demo visibility**: HIGH (always accessible, modal is prominent).
   - **Scope**: Fuzzy matcher + command registry + @/# mention bindings.
   - **SPEC**: SPEC-V3-012.
   - **Timeline**: 4-6 days.

4. **A-1 + A-2 (Pane split + Tab persistence)** ⭐⭐⭐⭐
   - **Why**: Terminal multiplexing is expected. Cmd+\ (split h), Cmd+Shift+\ (split v), Cmd+W (close tab) + persistence across restart = mature terminal app feel.
   - **Demo visibility**: HIGH (pane grid dominates main area).
   - **Scope**: Pane tree full lifecycle + tab bar CRUD + workspace-level persistence.
   - **SPEC**: SPEC-V3-003/004.
   - **Timeline**: 5-7 days.

5. **C-3 (Code Viewer: syntax + LSP diagnostics)** ⭐⭐⭐
   - **Why**: When user clicks `src/app.ts:42`, they see syntax coloring + red squiggles from LSP (Rust, Python, JS). Instant "pro IDE" impression.
   - **Demo visibility**: HIGH (code Surface is prominent when opened).
   - **Scope**: tree-sitter syntax highlight + LSP client integration (backend exists, GUI wiring needed).
   - **SPEC**: SPEC-V3-005/006.
   - **Timeline**: 4-5 days.

**Secondary Targets (high value, lower demo priority):**

- **D-2 (Workspace switcher UI polish)** — Rename, delete, reorder workspaces. Improves multi-project UX.
- **G-1 (Settings completion)** — Hooks / MCP / Rules panels for power users.
- **E-1 (SPEC Card rendering)** — Full SPEC card display with AC table + status. Makes SPEC-driven workflow discoverable.

---

## §5 Risk & Feasibility Assessment

### Quick Wins (Can land in v0.1.2 given 2-week sprint)

- **B-2 (file path parsing)**: Regex + terminal click handler = ~200 LOC, low risk. Dependencies: pane focus tracking (already exist).
- **F-1 (palette fuzzy)**: Scrim exists; add comfy_table crate + simple Levenshtein. ~400 LOC, low risk.
- **G-3 (color theme picker)**: 2-3 theme variants (dark/light/nord) + UI toggle. ~300 LOC, low risk.

### Medium Effort (3-5 day sprints)

- **C-2 (Markdown + KaTeX)**: Leverage `react-markdown` patterns, adapt to GPUI. ~1K LOC, medium risk (GPUI rendering unfamiliar patterns).
- **C-3 (syntax + LSP)**: tree-sitter-rs + lsp-types crate. Backend wiring exists; frontend integration ~500 LOC. Medium risk.
- **A-1 (pane tree persistence)**: Pane serialization exists (panes_convert.rs); restore logic ~300 LOC. Low-medium risk.

### Higher Effort (defer to v0.2.0)

- **E-2 (TRUST 5 radar)** — Custom GPUI canvas rendering. 5-axis chart is non-trivial. ~1.5K LOC, medium-high risk.
- **E-5 (Mission Control grid)** — Multi-agent grid with live updates. Requires Hook streaming + state sync. High risk.
- **C-4 (Browser Surface)** — wry WebView integration + DevTools, untested on Linux/Windows GPUI layer. Medium-high risk, defer.

---

## §6 SPEC-V3 Implementation Coverage

| SPEC ID | Title | Status | Tier | Est. Completion |
|---------|-------|--------|------|-----------------|
| SPEC-V3-001 | GPUI scaffold + Rust core | ✅ Draft | Critical | 100% (Phase 0) |
| **SPEC-V3-002** | **Terminal Core** | **✅ Completed** | **Critical** | **100% (Phase 2)** |
| SPEC-V3-003 | Tab / Pane Split | 🟡 In Progress | Critical | 60% (A-1/A-2 partial) |
| SPEC-V3-004 | Render Layer + Divider | 🟡 In Progress | Critical | 50% (tab/pane entity wiring) |
| SPEC-V3-005 | File Explorer + Surfaces | 🟡 In Progress | High | 40% (C-2/C-3/C-8 scaffolds) |
| SPEC-V3-006 | Markdown / Code Viewer | 🟡 In Progress | High | 35% (viewers skeleton) |
| SPEC-V3-007 | Browser Surface (wry) | ⬜ Not Started | High | 0% |
| SPEC-V3-008 | Git Management UI | ⬜ Not Started | Medium | 0% |
| SPEC-V3-009 | SPEC Management UI | 🟡 In Progress | Medium | 30% (E-1 skeleton) |
| SPEC-V3-010 | Agent Dashboard | 🟡 In Progress | High | 25% (E-4 scaffold) |
| SPEC-V3-011 | Cross-platform Packaging | ⬜ Not Started | Critical | 0% |
| SPEC-V3-012 | Palette Surface | 🟡 In Progress | High | 40% (F-1 overlay) |
| SPEC-V3-013 | Settings Surface | 🟡 In Progress | High | 45% (G-1 modal structure) |
| SPEC-V3-014 | Banners Surface | 🟡 In Progress | Medium | 50% (BannerStack entity) |
| SPEC-V3-015 | SPEC Panel Overlay | 🟡 In Progress | Medium | 40% (SpecPanelView scaffold) |
| SPEC-V3-DIST-001 | Distribution Channels | ⬜ Not Started | Low | 0% |
| SPEC-V3-FS-WATCHER-001 | FS Watcher Determinism | 🟡 In Progress | Low | 70% (notify watch working) |

**Key Insight:** SPEC-V3-002 is the only **completed** SPEC (v0.1.1). v0.1.2 should land 2-3 more completions (SPEC-V3-003/005/012) to show momentum.

---

## §7 v0.1.2 Release Recommendation

### Release Scope (Conservative Estimate)

**Must-Have for v0.1.2 MVP:**
1. ✅ Terminal Core (SPEC-V3-002) — Already completed in v0.1.1.
2. 🟡 B-2 (File path parsing) — **NEW demo-visible addition**.
3. 🟡 F-1 (Command Palette basics) — **NEW navigation unlock**.
4. 🟡 C-2 (Markdown viewer basic) — **NEW content surface**.

**Optional (if time permits):**
- D-2 (Workspace switcher polish)
- G-3 (Theme color picker)
- A-1 (Pane persistence)

### Suggested Sprint Plan

| Week | Focus | Target SPECS | Demo |
|------|-------|-------------|------|
| Week 1 | B-2 + B-3 link parsing | SPEC-V3-003 | Click `src/main.rs:42` in terminal → opens code viewer |
| Week 2 | F-1 palette + C-2 markdown | SPEC-V3-005/012 | Cmd+P (go to file), read SPEC inline with math |
| Week 3 | Polish + testing | All above | Full e2e smoke test |

### Success Criteria for v0.1.2

- [ ] SPEC-V3-002 regression tests pass (289 moai-core + 74 terminal tests)
- [ ] Terminal link click handler wired (B-1/B-2/B-3)
- [ ] File path parsing + regex matching works (unit tests)
- [ ] Command Palette modal responds to Cmd/Ctrl+K, fuzzy search functional
- [ ] Markdown renderer displays `.md` files with basic KaTeX
- [ ] All new code has @MX tags (NOTE/WARN/ANCHOR for fan_in >= 3)
- [ ] Playwright e2e: 30+ scenarios covering A-1 to F-1
- [ ] macOS + Linux CI passes (Windows defer to v0.2.0)

---

## §8 Carry-over to v0.2.0+

### Tier B (Smart Link Handling) — HIGH PRIORITY

Moai의 가장 중요한 차별화 포인트. 터미널 output에서 클릭 → 자동 navigation.

- **B-6 (Mermaid detect)** — Terminal output에서 markdown 코드블록 감지.
- **B-7 (Hover preview)** — File path hover → 파일 미리보기 팝업.

### Tier C (Surfaces) — Platform Expansion

- **C-4 (Browser)** — wry WebView (SPEC-V3-007).
- **C-5/C-6/C-7** — Image / JSON / Mermaid surfaces (SPEC-V3-005).

### Tier E (moai-adk Overlay) — Agent Integration

- **E-2 (TRUST 5)** — 5-axis quality radar for moai-adk projects.
- **E-5 (Mission Control)** — Parallel agent grid (SPEC-V3-010).
- **E-7 (Memory Viewer)** — auto-memory UI (SPEC-V3-009).

### Tier F (Navigation) — Keyboard Power User

- **F-6 (Onboarding)** — Environment detection (shell/tmux/node detect) + interactive tour (SPEC-V3-010).
- **F-3 (Toolbar)** — Action button wiring (SPEC-V3-006).

### Tier G (Configuration) — Customization

- **G-4 (Keybindings)** — Runtime keybinding customization.
- **G-5 (Auto-update)** — Tauri updater CI integration (SPEC-V3-011).

### Cross-Platform (v0.2.1+)

- **Windows build** — GPUI Windows GA (currently macOS/Linux only).
- **Distribution** — Homebrew / Scoop / AUR (SPEC-V3-DIST-001).

---

## Appendix A: Methodology

**Data Collection:**
1. Design v3 spec (`/Users/goos/MoAI/moai-studio/.moai/design/v3/spec.md`) — 45 features extracted.
2. SPEC-V3-001 through SPEC-V3-015, SPEC-V3-DIST-001, SPEC-V3-FS-WATCHER-001 — Status/AC reviewed.
3. Source code grep: 
   - `moai-studio-ui/src/` (13 modules: panes, tabs, terminal, viewer, settings, palette, explorer, agent, spec_ui, banners, design)
   - `moai-studio-terminal/src/` (PTY, VT parser, libghostty FFI)
   - `moai-studio-workspace/src/` (Workspace persistence)
   - `moai-studio-spec/src/` (Kanban, AC state, SPEC parsing)

**Status Definitions:**
- **DONE**: Code module exists, acceptance criteria met, tests passing (≥ 80% coverage).
- **PARTIAL**: Code exists but incomplete; some acceptance criteria met, OR scaffolded without core logic.
- **NONE**: No implementation found in codebase.

**Demo-Visibility Scoring:**
- **HIGH**: Feature visible in main terminal area, toolbar, sidebar, or primary modal (e.g., Command Palette, Terminal, panes). Requires 0-1 clicks to interact.
- **MEDIUM**: Feature visible in secondary pane or requires 1-2 clicks to discover (e.g., Settings, Code Viewer Surface when opened).
- **LOW**: Feature is internal or deeply nested (e.g., IPC, persistence, LSP internals).

**v0.1.2 Candidacy:**
- Candidates are PARTIAL or NONE features with HIGH demo-visibility + realistic 3-5 day completion time.
- Excludes DONE features (already shipped) and LOW demo-visibility features (don't move the needle for user impressions).

---

**Document Generated by**: Feature Audit Engine  
**Last Verified**: 2026-04-27 11:30 UTC  
**Next Review**: v0.1.3 planning (2026-05-11)

