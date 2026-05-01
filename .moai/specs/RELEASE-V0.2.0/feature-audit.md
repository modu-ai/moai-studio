# Feature Audit — moai-studio v0.1.2 → v0.2.0

**Generated**: 2026-05-01
**Source**: design v3 master spec (`.moai/design/v3/spec.md` v3.1.0, 2026-04-21)
**Verified against**: HEAD `1ce6b01d` (v0.1.2 GA tag, 2026-05-01)
**Audit Scope**: design v3 의 48 functional features (A1-A7, B1-B7, C1-C9, D1-D6, E1-E8, F1-F6, G1-G5) + v3.1 신규 §12 Plugin Architecture (I-1~I-6) + v0.1.2 시점 신규 발견 영역 (V3-016 Image / V3-017 TRUST 5 / V3-008 Git UI) → 28 SPEC 문서 + 22 crate 와의 매핑.
**Predecessor**: `.moai/specs/RELEASE-V0.1.2/feature-audit.md` (v0.1.1 → v0.1.2)

---

## §1 Summary

| Metric | v0.1.1 audit | v0.1.2 audit | **v0.1.2 GA actual** |
|--------|--------------|--------------|----------------------|
| Total features (design v3) | 48 | 45 | **48 + 6 (Plugin Arch)** = **54** |
| **DONE** | 4 | 4 | **12** |
| **PARTIAL** | 30 | 30 | **30** |
| **NONE** | 11 | 11 | **6** |
| **DEFERRED (v0.3.0+)** | — | — | **6** |

**Status by Tier (v0.1.2 GA actual):**
- Tier A (Terminal Core, 7): 1 DONE, 5 PARTIAL, 0 NONE, 1 DEFERRED
- Tier B (Smart Link, 7): 2 DONE, 3 PARTIAL, 1 NONE, 1 DEFERRED
- Tier C (Surfaces, 9): 3 DONE, 5 PARTIAL, 0 NONE, 1 DEFERRED
- Tier D (Workspace, 6): 1 DONE, 2 PARTIAL, 2 NONE, 1 DEFERRED
- Tier E (moai-adk, 8): 2 DONE, 4 PARTIAL, 0 NONE, 2 DEFERRED
- Tier F (UX, 6): 1 DONE, 5 PARTIAL, 0 NONE, 0 DEFERRED
- Tier G (Config, 5): 0 DONE, 4 PARTIAL, 0 NONE, 1 DEFERRED
- **Tier I (Plugin Arch v3.1, 6, 신규)**: 0 DONE, 3 PARTIAL, 3 NONE, 0 DEFERRED

**v0.2.0 demo-visible candidates: 8** (B-1, B-4, C-4 polish, D-2 follow-up, E-5, F-3, F-6, I-3 Plugin Manager UI)

---

## §2 Feature Matrix (v0.1.2 GA actual)

### Tier A — Terminal Core (cmux heritage)

| ID | Title | Pri | Status | v0.1.2 변화 | Demo | v0.2.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **A-1** | Multi-pane terminal (binary tree split) | Crit | PARTIAL | ⬆ | HIGH | NO | `crates/moai-studio-ui/src/panes/` + `tabs/container.rs`. PR #64 PaneLayoutV1.active_tab_idx round-trip 추가 (AC-P-30~37). |
| **A-2** | Tab UI (in-pane tabs) | Crit | PARTIAL | ⬆ | HIGH | NO | `tabs/container.rs` TabContainer + move_tab/duplicate_tab (PR #64). |
| **A-3** | tmux full compat (OSC 8, mouse, 256+24-bit) | Crit | PARTIAL | — | MED | NO | `moai-studio-terminal/src/vt.rs` libghostty FFI. OSC 8 parse OK, click wire deferred. |
| **A-4** | Multi-shell (zsh/bash/fish/nu/pwsh/cmd) | Crit | PARTIAL | — | MED | YES | `moai-studio-terminal/src/pty/` default $SHELL only. Shell picker UI 부재 (carry from v0.1.2 audit). |
| **A-5** | Session persistence | High | PARTIAL | ⬆ | LOW | NO | `moai-studio-workspace/src/persistence.rs` 14KB, `panes_convert.rs` 12KB (PR #64 round-trip 강화). |
| **A-6** | Block-based output (Warp model) | High | NONE | — | LOW | NO | (no implementation). v0.3.0 후보. |
| **A-7** | Unix socket IPC + named pipe | High | PARTIAL | — | LOW | NO | `moai-studio-terminal/src/pty/unix.rs`. Windows pipe 미검증. **DEFERRED to v0.3.0** (Windows GA 시 검증). |

### Tier B — Smart Link Handling (MoAI 차별화 핵심)

| ID | Title | Pri | Status | v0.1.2 변화 | Demo | v0.2.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **B-1** | OSC 8 hyperlinks render + click | Crit | PARTIAL | — | MED | **YES** | libghostty parse OK, terminal `on_mouse_down` → handle_click 구현됨 (V3-LINK-001). 풀 OSC 8 lifecycle UI wire 미완. |
| **B-2** | Regex file path detection (path:line:col) | Crit | DONE | — | HIGH | NO | `moai-studio-terminal/src/link.rs` 23KB `detect_links()` + `terminal/mod.rs` click → resolve_click → TerminalClickEvent (V3-LINK-001 B-2/B-3). |
| **B-3** | URL auto-detect + highlight | Crit | DONE | — | HIGH | NO | 동 link.rs + cx.open_url() (V3-LINK-001). PR #67 에서 terminal URL → toast → Browser tab integration 까지 완성 (AC-WB-INT-1~4). |
| **B-4** | SPEC-ID pattern detection | Crit | PARTIAL | ⬆ | MED | **YES** | PR #69 V3-009 MS-4a — 터미널 SPEC-ID 클릭 → SpecPanel mount + select 와이어링 (AC-SU-13~16). regex 매칭 + click handler 구현됨. terminal-side SPEC-ID 하이라이트 렌더는 미완. |
| **B-5** | @MX tag detection | High | PARTIAL | — | MED | NO | `viewer/mx_gutter.rs` 19KB. Code viewer 측 detect 완성. Terminal-side @MX detect 미구현. |
| **B-6** | Mermaid code block detection | Med | NONE | — | LOW | NO | (no terminal-side implementation). C-7 Mermaid surface 와 함께 v0.3.0. |
| **B-7** | Hover preview (file popup) | Med | NONE | — | LOW | NO | (no implementation). **DEFERRED to v0.3.0**. |

### Tier C — Surfaces (Wave heritage)

| ID | Title | Pri | Status | v0.1.2 변화 | Demo | v0.2.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **C-1** | Terminal Surface | Crit | DONE | — | HIGH | NO | SPEC-V3-002 GREEN. `moai-studio-ui/src/terminal/` + libghostty Metal/GLES 렌더. |
| **C-2** | Markdown Surface (EARS + KaTeX + Mermaid) | Crit | PARTIAL | ⬆ | HIGH | YES | `viewer/markdown/` + PR #65 math_unicode (LaTeX→Unicode 89 LOC) + mermaid_meta diagram type detect (70 LOC, AC-MV-15~17). 실 KaTeX 렌더 + 실 Mermaid 렌더는 placeholder 상태. |
| **C-3** | Code Viewer (LSP + 6 lang) | High | PARTIAL | ⬆ | HIGH | YES | `viewer/code/` + tree-sitter (PR #66 JS/JSON/JSX/MJS/CJS/JSONC 추가, AC-MV-18~23). LSP `viewer/lsp.rs` 24KB partial. Monaco 대체 GPUI 자체 렌더. |
| **C-4** | Browser Surface (WebView + DevTools) | Crit | PARTIAL | ⬆ | HIGH | **YES** | `crates/moai-studio-ui/src/web/` 8 modules (surface 24KB, bridge 13KB, history, url, url_detector). PR #67 terminal stdout → URL detector → toast → tab integration polish (AC-WB-INT-1~4). DevTools / 더 풍부한 navigation 미완. |
| **C-5** | Image Surface (zoom/pan/EXIF) | High | DONE | ⬆⬆ | MED | NO | **신규 발견**: V3-016 MS-1/2/3 (commits `8725a0b`, `23da533`, `ac58297`). `viewer/image.rs` 27KB + `image_data.rs` + `exif.rs` 5KB. Zoom toolbar + EXIF panel + cursor feedback + SVG placeholder + PNG/JPEG/GIF/WebP/BMP/ICO decode (AC-IV-001~045 완성). |
| **C-6** | JSON / CSV Surface | Med | PARTIAL | ⬆ | MED | NO | JSON tree-sitter 통한 syntax-highlighted display 가능 (PR #66). 본격 pretty/tabular surface 별도 SPEC 미존재. v0.2.0+ 후보. |
| **C-7** | Mermaid Renderer Surface | Med | NONE | — | MED | NO | placeholder type detection 만 (PR #65). 실 Mermaid 렌더 → wry WebView 의존, **DEFERRED to v0.3.0**. |
| **C-8** | File Tree Surface (git status) | High | DONE | — | MED | NO | `moai-studio-ui/src/explorer/` 12 modules. notify watch + FsNode tree + git status (#34). |
| **C-9** | Agent Run Viewer (Hook timeline) | High | PARTIAL | — | MED | NO | `moai-studio-ui/src/agent/` 7 modules: dashboard_view, control_bar, cost_panel, detail, instructions_graph, timeline (V3-010 MS-1/2/3 완성). |

### Tier D — Multi-Project Workspace (VS Code heritage)

| ID | Title | Pri | Status | v0.1.2 변화 | Demo | v0.2.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **D-1** | Workspaces JSON persistence | Crit | DONE | — | MED | NO | `moai-studio-workspace/src/lib.rs` WorkspacesStore load/save. `~/.moai/studio/workspaces.json`. |
| **D-2** | Sidebar workspace switcher | Crit | PARTIAL | ⬆ | HIGH | **YES** | PR #76 `crate::workspace_menu` 모듈 + WorkspaceMenuAction 4-variant + WorkspaceMenu single-menu invariant (AC-D2-1~5, **skeleton**). Rename modal / delete confirmation / reorder dispatch / RootView 우클릭 와이어링 미완 → v0.2.0 follow-up. |
| **D-3** | State preserve on project switch | High | PARTIAL | ⬆ | LOW | NO | PR #64 panes round-trip + tab CRUD (AC-P-30~37). Workspace 별 격리는 실 검증 필요. |
| **D-4** | Global search across workspaces | High | NONE | — | MED | **YES** | (no implementation). v0.2.0 critical demo 후보. |
| **D-5** | Workspace color tags | Low | NONE | — | LOW | YES | `ws.color` 필드 존재하나 모든 workspace 가 동일 orange-red 하드코드. HashMap 분리 필요. |
| **D-6** | Drag-and-drop workspace add | Med | NONE | — | MED | NO | File picker 동작 OK, D&D 미구현. v0.2.0+ 후보. |

### Tier E — moai-adk GUI Overlay (MoAI 특화)

| ID | Title | Pri | Status | v0.1.2 변화 | Demo | v0.2.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **E-1** | SPEC Card (active SPEC) | High | PARTIAL | ⬆ | MED | YES | `spec_ui/` 7 modules (list_view 17KB, detail_view 14KB, kanban_view 24KB, spec_panel_view 13KB). PR #69 터미널 SPEC-ID 클릭 wiring + PR #75 SpecListView AC chip row (AC-SU-13~20). Master-detail 통합 미완. |
| **E-2** | TRUST 5 Dashboard (5-axis radar) | High | DONE | ⬆⬆ | MED | NO | **신규 발견**: V3-017 (commits `8725a0b`, `23da533`, `ac58297`). `crates/moai-studio-ui/src/quality/` 62KB: radar_chart_view 18KB, dimension_detail_view 11KB, history_view 8KB, quality_gate_view 12KB, mod 23KB. TRUST 5 scoring engine + radar chart + history. |
| **E-3** | @MX tag gutter + popover | High | PARTIAL | — | MED | NO | `viewer/mx_gutter.rs` 19KB MXPopover + MXGutterLine + MXAnnotation with icon. 거터 표시 OK, popover hover 부분. |
| **E-4** | Hook event stream (27 events) | High | PARTIAL | — | LOW | YES | `moai-hook-http/src/` HTTP server 존재. `moai-studio-ui/src/agent/` GPUI wire-up partial (timeline_view 7KB). 27 events 전수 wire 미완 → v0.2.0 carry. |
| **E-5** | Mission Control (parallel agents grid) | High | NONE | — | MED | **YES** | (no implementation). v0.2.0 critical 후보. |
| **E-6** | Kanban Board (SPEC lifecycle) | Med | DONE | — | MED | NO | `spec_ui/kanban_view.rs` 24KB. SPEC-V3-009 RG-SU-3 (PR #31). State machine (draft/in-progress/review/done) + GPUI Kanban UI 완성. |
| **E-7** | Memory Viewer | Med | NONE | — | LOW | NO | (no implementation). **DEFERRED to v0.3.0**. |
| **E-8** | CG Mode (Claude + GLM split) | Low | NONE | — | LOW | NO | (no implementation). **DEFERRED to v0.3.0+**. |

### Tier F — Navigation & UX

| ID | Title | Pri | Status | v0.1.2 변화 | Demo | v0.2.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **F-1** | Command Palette (⌘/Ctrl+K) | Crit | PARTIAL | ⬆⬆ | HIGH | NO | `palette/` 6 modules: registry 12KB (PR #63 CommandRegistry 40+ commands), palette_view 17KB, fuzzy 17KB, scrim 11KB, mod 9KB. RootView dispatch + @mention mode + slash bar pending_slash_injection (AC-PL-16~22). DONE 가까이. |
| **F-2** | Native menu bar | Crit | PARTIAL | ⬆ | HIGH | NO | macOS native menu 11 menu (App/File/Edit/View/Pane/Surface/SPEC/Agent/Go/Window/Help). PR #68 4 stub action handler → functional 동작 교체 (AC-MN-7~11). 잔존 stub handler v0.2.0 carry. |
| **F-3** | Toolbar (7 primary actions) | High | PARTIAL | — | HIGH | **YES** | `moai-studio-ui/src/toolbar.rs` 4798 bytes. TitleBar 영역에 토대 있음, 7 action 실 wiring 미완. v0.2.0 critical demo 후보. |
| **F-4** | Status bar (pill + git + LSP) | High | PARTIAL | ⬆⬆ | HIGH | YES | **신규 발견**: PR #74 `moai-studio-ui/src/status_bar.rs` 10KB state-bearing 모듈 + AgentPill / GitWidget / LspWidget + 4 mutation API (AC-SB-1~6, **skeleton**). 실 Git/LSP/Agent state binding 미완. |
| **F-5** | Empty State CTA | Crit | DONE | — | HIGH | NO | `moai-studio-ui/src/lib.rs` empty_state_view (AC-2.2). hero + create-first + secondary 완성. |
| **F-6** | Onboarding tour (env detect) | High | PARTIAL | — | MED | **YES** | `moai-studio-ui/src/wizard.rs` 12.7KB (5-step Wizard). 환경 감지 (shell/tmux/node/python/rust) + interactive tour 미완. v0.2.0 후보. |

### Tier G — Configuration

| ID | Title | Pri | Status | v0.1.2 변화 | Demo | v0.2.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **G-1** | Settings (General/Hooks/MCP/Skills/Rules/Keybindings) | High | PARTIAL | ⬆⬆ | MED | NO | `settings/panes/` 11 panes: advanced, agent, appearance, editor, hooks (PR #70), keyboard, mcp (PR #71), rules (PR #73), skills (PR #72), terminal, mod (AC-V13-13~31). 모든 design 영역 pane 도입. settings_state 45KB (UserSettings 영속화 V3-013 MS-3 완성). 실제 설정 항목 wire-up partial. |
| **G-2** | New Workspace Wizard | Crit | PARTIAL | — | HIGH | YES | `wizard.rs` 12.7KB 5-step structure. File picker integrated. 5-step 실 flow + 폼 검증 미완. |
| **G-3** | Theme switcher (dark/light/auto) | Med | PARTIAL | — | HIGH | NO | `settings/panes/appearance.rs` 8KB ActiveTheme. dark default, light/auto picker partial. |
| **G-4** | Keybinding customization | Low | PARTIAL | — | LOW | NO | `settings/panes/keyboard.rs` 12KB KeyboardPane. Static display only. **DEFERRED to v0.3.0**. |
| **G-5** | Auto-update | High | PARTIAL | — | LOW | NO | SPEC-V3-DIST-001 PR #49/#50/#60. Homebrew/Scoop/AUR/AppImage. Auto-updater Tauri-style 미통합 (GPUI 네이티브에 맞춤 필요). |

### Tier I — Plugin Architecture (v3.1 §12, 신규)

design v3 spec.md v3.1.0 §12 에서 신규 도입된 Plugin Architecture. v0.1.0/v0.1.1/v0.1.2 audit 누락 영역. v0.1.2 GA 시점 부분 구현됨.

| ID | Title | Pri | Status | v0.1.2 변화 | Demo | v0.2.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **I-1** | Plugin trait API + manifest schema | Crit | PARTIAL | — | LOW | YES | `crates/moai-studio-plugin-api/src/lib.rs` 1411 bytes (trait 정의 stub). Manifest schema 정식화 미완. v3.1 spec §12 의 plugin.toml 포맷 미구현. |
| **I-2** | Plugin runtime (load/activate/deactivate) | Crit | PARTIAL | — | LOW | YES | `crates/moai-plugin-installer/` 27KB: installer, lib, verify. Plugin discovery + verification 토대. 실 runtime 활성/비활성 cycle 미완. |
| **I-3** | Plugin Manager UI (Settings > Plugins) | High | NONE | — | HIGH | **YES** | (no implementation). settings 11 panes 中 "plugins" pane 부재. design v3 §12 Frame 21 후보. |
| **I-4** | moai-adk plugin form factor | High | PARTIAL | — | MED | YES | `crates/moai-studio-plugin-moai-adk/src/lib.rs` 793 bytes (stub). 현재 SPEC/TRUST5/@MX/Hook 영역은 in-tree 구현 (`spec_ui/`, `quality/`, `viewer/mx_gutter.rs`, `agent/`). plugin form factor 로 분리 미실행. |
| **I-5** | Plugin marketplace | Low | NONE | — | LOW | NO | (no implementation). **DEFERRED to v0.3.0+**. |
| **I-6** | Plugin sandbox (WASM runtime) | Low | NONE | — | LOW | NO | (no implementation). v3.2 stretch goal per spec §12. **DEFERRED to v0.3.0+**. |

### Bonus: v3.1 spec 외 도입 영역 (참고)

design v3 spec 에 직접 명시되지 않았으나 v0.1.2 GA 시점 구현된 영역:

| Area | Status | Evidence | Note |
|------|--------|----------|------|
| **Git UI surface** | DONE | `moai-studio-ui/src/git/` 8 modules 50KB: branch_switcher, commit_composer, diff_viewer, log_view, merge_resolver, stash_panel, status_panel | SPEC-V3-008 (Git Management UI) 완성. Tier C 의 surface 확장으로 분류 가능. |
| **Banners (crash/update/lsp)** | PARTIAL | `moai-studio-ui/src/banners/` | SPEC-V3-014. BannerStack entity 구현. 실 시나리오 wire 미검증. |
| **SPEC Panel Overlay** | PARTIAL | `spec_ui/spec_panel_view.rs` 13KB | SPEC-V3-015. SpecPanelView scaffold. master-detail 통합 v0.2.0 carry. |
| **Sprint Panel** | DONE | `spec_ui/sprint_panel.rs` 10KB | Sprint Contract 통합 (SPEC-V3-009 후속). |

---

## §3 Per-Tier Breakdown (v0.1.2 GA → v0.2.0 gap)

### Tier A — Terminal Core

v0.1.2 GA 진행:
- A-1/A-2 PaneLayoutV1.active_tab_idx round-trip + tab CRUD (PR #64) — round-trip 완성
- A-5 persistence 14KB persistence.rs + 12KB panes_convert.rs

v0.2.0 critical gap:
- **A-4 Multi-shell picker UI** — 가장 큰 user-facing gap. Command Palette 또는 toolbar 에서 zsh/bash/fish/nu/pwsh 선택 UI. 추정 ~300 LOC.

### Tier B — Smart Link Handling

v0.1.2 GA 진행:
- B-2/B-3 link.rs detect_links() 23KB + terminal click handler 완성
- B-3 PR #67 terminal URL → toast → Browser tab integration (USP 의 첫 e2e 시연)
- B-4 PR #69 SPEC-ID 터미널 클릭 → SpecPanel mount + select 완성

v0.2.0 critical gap:
- **B-1 OSC 8 click full lifecycle** — libghostty parses OSC 8, terminal click resolve_click 까지 OK. 풀 lifecycle (params hover, copy URL, follow link visited state) 미완.
- **B-4 SPEC-ID 하이라이트 렌더** — click 동작 OK, 터미널 측 underline / 색상 강조 미구현.
- **B-5 Terminal-side @MX detect** — viewer/code 측 mx_gutter 완성, terminal 측 detect 부재.

### Tier C — Surfaces

v0.1.2 GA 진행:
- **C-5 Image Surface** 완성 (V3-016 MS-1/2/3): zoom toolbar + EXIF panel + 6 format decode (PNG/JPEG/GIF/WebP/BMP/ICO) + cursor feedback + SVG placeholder
- **C-3 JS/JSON tree-sitter** (PR #66) — Markdown 외 코드 viewer 다국어 확장
- **C-4 Browser integration polish** (PR #67) — terminal URL → toast → tab 까지 e2e

v0.2.0 critical gap:
- **C-2 Markdown 실 KaTeX 렌더** — placeholder math_unicode 만 구현. 실 KaTeX 렌더는 wry WebView 또는 자체 GPUI custom rendering 필요. 추정 ~1.5K LOC, 중간 위험.
- **C-3 더 많은 LSP 통합** — viewer/lsp.rs 24KB partial. Rust/Go/Python/TS LSP 풀 통합.
- **C-4 Browser DevTools** — 현재 surface 24KB + bridge 13KB partial. DevTools 통합 + navigation history UI.

v0.3.0 deferred:
- C-7 Mermaid 실 렌더 (wry 의존)

### Tier D — Multi-Project Workspace

v0.1.2 GA 진행:
- D-2 workspace_menu.rs 9KB + WorkspaceMenuAction 4-variant + single-menu invariant (PR #76 skeleton)
- D-3 persistence round-trip (PR #64)

v0.2.0 critical gap:
- **D-2 follow-up** — rename modal, delete confirmation, reorder dispatch, RootView 우클릭 와이어링. skeleton 만 도입됨.
- **D-4 Global search** — across workspaces 검색. ripgrep/tantivy 통합 후보. 추정 ~800 LOC, 중간 위험. **MoAI 의 multi-project 차별화 핵심**.
- **D-5 Workspace color tags** — HashMap 분리 + UI picker. ~300 LOC.

v0.3.0 deferred:
- D-6 Drag-and-drop workspace add

### Tier E — moai-adk GUI Overlay

v0.1.2 GA 진행:
- **E-2 TRUST 5 Dashboard 완성** (V3-017): radar_chart 18KB + dimension_detail 11KB + history 8KB + quality_gate 12KB
- **E-6 Kanban DONE** (V3-009 RG-SU-3, PR #31)
- E-1 SpecListView AC chip + 터미널 클릭 (PR #75/#69)

v0.2.0 critical gap:
- **E-1 SpecPanelView master-detail 통합** — list/detail/kanban 별도 view. 마스터-디테일 통합 + AC inline expansion.
- **E-4 Hook event stream 27 events 전수 wire** — moai-hook-http server 측 OK, GPUI 측 27 events 모두 wire 필요.
- **E-5 Mission Control (parallel agents grid)** — 멀티 에이전트 동시 모니터링. 추정 ~1.2K LOC, hook stream 의존, 중간-높은 위험.

v0.3.0 deferred:
- E-7 Memory Viewer (~/.claude/projects/…/memory/)
- E-8 CG Mode (Claude + GLM split)

### Tier F — Navigation & UX

v0.1.2 GA 진행:
- **F-1 Command Palette** DONE 가까이: registry 40+ commands + dispatch + @mention (PR #63)
- F-2 11 menu category 등록 + 4 stub handler functional (PR #68)
- **F-4 Status Bar widgets** skeleton: AgentPill/GitWidget/LspWidget + 4 mutation API (PR #74)

v0.2.0 critical gap:
- **F-3 Toolbar 7 primary actions wire** — toolbar.rs 4798 bytes 토대만, 실 button wire 미완. F-1 dispatch 와 통합 가능.
- **F-4 Status Bar 실 state binding** — skeleton 이후 git status / LSP diagnostics / agent status 실시간 binding.
- **F-6 Onboarding tour** — wizard.rs 12.7KB 5-step structure 있음. 환경 감지 (shell/tmux/node/python/rust 자동 detect) + interactive tour 추가.

### Tier G — Configuration

v0.1.2 GA 진행:
- **G-1 모든 design 영역 pane 도입** (4 신규 panes PR #70-#73): Hooks/MCP/Skills/Rules. 11 panes 총 도입.
- G-2 wizard.rs 5-step structure (F-6 와 통합)
- G-5 V3-DIST-001 패키지 채널 완성 (PR #49/50/60)

v0.2.0 critical gap:
- **G-1 panes 실 wire-up** — 11 pane skeleton 이후 실제 설정 항목 binding (Hooks 27 event toggle, MCP server enable/disable, Skills pack 활성, Rules path edit).
- **G-3 Theme picker 확장** — light/auto + 색 테마 (Nord, Dracula 등) 옵션.

v0.3.0 deferred:
- G-4 Runtime keybinding customization
- G-5 Auto-update Tauri-style 통합 (GPUI 네이티브 환경)

### Tier I — Plugin Architecture (v3.1, 신규)

v0.1.2 GA 진행 (v3.1 spec §12 부분 구현):
- I-1 plugin-api crate stub (1.4KB)
- I-2 plugin-installer crate (27KB: installer, lib, verify)
- I-4 plugin-moai-adk crate stub (793 bytes)

v0.2.0 critical gap (Plugin Architecture 의 첫 GA 사이클):
- **I-1 Plugin manifest 정식화** — plugin.toml 포맷 (spec §12 Manifest example) 정확 구현. permissions / contributes (surfaces, sidebar_sections, statusbar_widgets, commands, link_parsers) 스키마 + 검증.
- **I-2 Plugin runtime full lifecycle** — load → activate → handle hook events → deactivate. moai-studio-app 에 plugin registry 통합.
- **I-3 Plugin Manager UI** — settings 12-th pane (Plugins). Search / Installed / Install From URL. 활성/비활성 토글 + 권한 다이얼로그. design v3 spec §12 Frame 21.
- **I-4 moai-adk plugin form factor 분리** — 현재 in-tree (`spec_ui/`, `quality/`, `agent/`, `viewer/mx_gutter.rs`) 를 plugin entry point 로 라우팅. plugin 비활성화 시 sidebar/menu/palette 항목 hidden.

v0.3.0 deferred:
- I-5 Plugin marketplace (https://plugins.moaistudio.dev)
- I-6 Plugin sandbox (WASM runtime)

---

## §4 v0.2.0 Recommended Targets (demo-visibility ranked)

### Top 8 Demo-Visible Wins

**Priority 1 (release momentum 결정 요소):**

1. **D-4 Global search across workspaces** ⭐⭐⭐⭐⭐
   - **Why**: MoAI 의 multi-project 차별화 핵심. ⌘⇧F → 전체 workspace 검색 → 결과 클릭 → tab 으로 점프. VS Code Cmd+Shift+F 와 동등 UX.
   - **Demo visibility**: HIGH (사이드바 + Command Palette 양쪽에서 진입).
   - **Scope**: ripgrep / tantivy 통합 + 결과 SearchResultView GPUI + click-to-navigate. 추정 ~800 LOC.
   - **SPEC**: SPEC-V0-2-0-GLOBAL-SEARCH-001 (신규).
   - **Risk**: 중간 (인덱싱 전략 결정 필요).

2. **E-5 Mission Control (parallel agents grid)** ⭐⭐⭐⭐⭐
   - **Why**: 다중 에이전트 동시 실행 가시화. moai-adk team mode 의 시각적 anchor. 4-cell grid + per-agent status pill.
   - **Demo visibility**: HIGH (별 surface 또는 Right Panel 에 mount).
   - **Scope**: hook event stream 의존 + grid layout + per-agent state machine. 추정 ~1.2K LOC.
   - **SPEC**: SPEC-V0-2-0-MISSION-CTRL-001 (신규, V3-010 후속).
   - **Risk**: 중간-높음 (hook stream 안정성 의존).

3. **I-3 Plugin Manager UI** ⭐⭐⭐⭐
   - **Why**: v3.1 §12 Plugin Architecture 의 visible front. 사용자가 moai-adk on/off 결정 + 추후 marketplace 진입점.
   - **Demo visibility**: HIGH (Settings 에 12-th pane 추가, Cmd+, → Plugins).
   - **Scope**: Settings pane + plugin list view + 활성/비활성 토글 + 권한 다이얼로그. 추정 ~600 LOC.
   - **SPEC**: SPEC-V0-2-0-PLUGIN-MGR-001 (신규).
   - **Risk**: 낮음 (Settings 패턴 reuse).

4. **D-2 follow-up (workspace switcher complete)** ⭐⭐⭐⭐
   - **Why**: v0.1.2 skeleton 의 자연스러운 마무리. rename modal + delete confirmation + reorder + RootView 우클릭 wire.
   - **Demo visibility**: HIGH (사이드바 우클릭).
   - **Scope**: 4 sub-feature wire. 추정 ~500 LOC.
   - **SPEC**: SPEC-V3-004 D-2 follow-up (기존 SPEC 확장).
   - **Risk**: 낮음.

**Priority 2 (high value, polish):**

5. **F-3 Toolbar 7 actions wire + F-4 Status Bar 실 binding** ⭐⭐⭐⭐
   - **Why**: 시각적 완성도. 현재 placeholder/skeleton 영역 실 동작.
   - **Scope**: F-1 dispatch 와 통합. 추정 ~700 LOC.

6. **F-6 Onboarding tour + env detect** ⭐⭐⭐
   - **Why**: 첫 실행 UX. plugin selection (moai-adk on/off) 도 onboarding 에서 처리.
   - **Scope**: wizard.rs 확장 + 환경 감지 (shell/tmux/node/python/rust). 추정 ~600 LOC.

7. **B-1 OSC 8 click full lifecycle** ⭐⭐⭐
   - **Why**: B-2/B-3 와 함께 USP 완성. visited state, copy URL, hover params.
   - **Scope**: terminal/mod.rs 확장. 추정 ~300 LOC.

8. **A-4 Multi-shell picker UI** ⭐⭐⭐
   - **Why**: v0.1.x 부터 carry 된 가장 오래된 critical gap. C-2 (멀티쉘) 제약 충족.
   - **Scope**: pty/mod.rs shell registry + Command Palette entry. 추정 ~300 LOC.

**Secondary (시간 허용 시):**

- D-5 Workspace color tags (HashMap 분리 + UI picker)
- D-6 D&D workspace add
- E-1 SpecPanelView master-detail 통합
- E-4 Hook event stream 27 events 전수 wire
- C-2 Markdown 실 KaTeX 렌더
- G-1 11 panes 실 wire-up
- G-3 Theme picker 확장 (Nord/Dracula)

---

## §5 Risk & Feasibility Assessment

### Quick Wins (5 day sprints, 낮은 위험)

- **D-2 follow-up**: 4 sub-feature, 패턴 명확. ~500 LOC.
- **A-4 Shell picker**: pty 측 registry + palette wire. ~300 LOC.
- **D-5 Color tags**: HashMap + picker. ~300 LOC.
- **B-1 OSC 8 lifecycle**: 기존 link.rs 확장. ~300 LOC.

### Medium Effort (1 주, 중간 위험)

- **F-3 Toolbar wire + F-4 Status Bar binding**: F-1 dispatch + 실 git/lsp/agent state. ~700 LOC.
- **F-6 Onboarding env detect**: wizard.rs 확장. ~600 LOC.
- **I-3 Plugin Manager UI**: Settings 패턴 reuse. ~600 LOC.

### Larger Effort (1.5-2 주, 중간-높은 위험)

- **D-4 Global search**: ripgrep/tantivy 결정 → 인덱싱 → 결과 view. ~800 LOC.
- **E-5 Mission Control**: hook stream 의존, grid layout. ~1.2K LOC.
- **C-2 Markdown 실 KaTeX**: GPUI custom rendering 또는 wry 의존. ~1.5K LOC, 의존성 결정 위험.

### v0.3.0 Defer 권장

- **C-7 Mermaid 실 렌더** — wry WebView GPUI 통합 위험.
- **E-7 Memory Viewer** — 우선순위 낮음.
- **E-8 CG Mode** — tmux 기반, 별 SPEC 필요.
- **G-4 Runtime keybinding** — 우선순위 낮음.
- **I-5/I-6 Plugin marketplace + sandbox** — v3.1 spec §12 stretch goal.
- **A-7 Windows named pipe 검증** — Windows GA 동반.
- **B-7 Hover preview** — 우선순위 낮음.

---

## §6 SPEC-V3 Implementation Coverage (v0.1.2 GA actual)

| SPEC ID | Title | Status | Tier | Est. Completion |
|---------|-------|--------|------|-----------------|
| SPEC-V3-001 | GPUI scaffold + Rust core | ✅ Completed | Critical | 100% |
| SPEC-V3-002 | Terminal Core (libghostty + PTY) | ✅ Completed | Critical | 100% |
| SPEC-V3-003 | Tab / Pane Split | ✅ Completed (MS-4) | Critical | 100% (PR #64 round-trip) |
| SPEC-V3-004 | Workspace Switcher + Render | 🟡 In Progress (MS-4 skeleton) | Critical | 80% (D-2 follow-up v0.2.0) |
| SPEC-V3-005 | File Explorer + Surfaces 통합 | 🟡 In Progress | High | 70% (C-2/C-5/C-6/C-7 별도 SPEC 분리) |
| SPEC-V3-006 | Markdown / Code Viewer | ✅ Completed (MS-4/5/7) | High | 95% (C-2 placeholder + status_bar skeleton, F-4 carry) |
| SPEC-V3-007 | Browser Surface (wry) | 🟡 In Progress (MS-4) | High | 80% (DevTools / 풍부한 nav v0.2.0) |
| SPEC-V3-008 | Git Management UI | ✅ Completed | Medium | 100% (8 modules 50KB) |
| SPEC-V3-009 | SPEC Management UI | 🟡 In Progress (MS-4a/b) | Medium | 85% (master-detail 통합 v0.2.0) |
| SPEC-V3-010 | Agent Dashboard | 🟡 In Progress (MS-1/2/3) | High | 70% (E-4 hook stream 전수 wire / E-5 Mission Control v0.2.0) |
| SPEC-V3-011 | Cross-platform Packaging | 🟡 In Progress | Critical | 50% (V3-DIST-001 패키지 채널 완성, auto-update 미통합) |
| SPEC-V3-012 | Palette Surface | ✅ Completed (MS-4) | High | 95% (CommandRegistry 40+ + dispatch) |
| SPEC-V3-013 | Settings Surface | 🟡 In Progress (MS-4a/b/c/d) | High | 80% (4 panes skeleton, 실 wire-up v0.2.0) |
| SPEC-V3-014 | Banners Surface | 🟡 In Progress | Medium | 60% (BannerStack entity, 시나리오 wire 미검증) |
| SPEC-V3-015 | SPEC Panel Overlay | 🟡 In Progress | Medium | 70% (master-detail 통합 v0.2.0) |
| SPEC-V3-016 | Image Surface | ✅ Completed (MS-1/2/3) | High | 100% (zoom + EXIF + 6 format decode + SVG) |
| SPEC-V3-017 | TRUST 5 Quality Engine | ✅ Completed (MS-1/2/3) | High | 100% (radar + history + dimension detail) |
| SPEC-V3-DIST-001 | Distribution Channels | ✅ Completed | Low | 100% (Homebrew/Scoop/AUR/AppImage) |
| SPEC-V3-FS-WATCHER-001 | FS Watcher Determinism | ✅ Completed | Low | 100% |
| SPEC-V3-LINK-001 | Smart Link Detection | ✅ Completed (B-2/B-3) | Critical | 95% (B-1 lifecycle + B-4 highlight v0.2.0) |
| SPEC-V3-PALETTE-001 | Palette enhancement | ✅ Completed | High | 100% (V3-012 흡수) |
| SPEC-V0-1-2-MENUS-001 | Native Menu polish | ✅ Completed (MS-2) | High | 90% (4 stub functional + 잔존 stub v0.2.0) |

**Key Insight (v0.1.2 GA actual):**
- **Completed (10 SPECs)**: V3-001/002/003/006/008/012/016/017/DIST/FS-WATCHER/LINK/PALETTE/V0-1-2-MENUS-001 = **12 completed**
- **In Progress (10 SPECs)**: V3-004/005/007/009/010/011/013/014/015 = **9 partial**
- v0.1.2 release momentum = **single-session 14 PR** (#63~#76, 0 regression, +82 tests)

**v0.2.0 신규 SPEC 후보:**
- SPEC-V0-2-0-GLOBAL-SEARCH-001 (D-4 글로벌 검색)
- SPEC-V0-2-0-MISSION-CTRL-001 (E-5)
- SPEC-V0-2-0-PLUGIN-MGR-001 (I-1/I-2/I-3 통합)
- SPEC-V0-2-0-MULTI-SHELL-001 (A-4)
- SPEC-V0-2-0-ONBOARDING-001 (F-6)
- SPEC-V0-2-0-TOOLBAR-WIRE-001 (F-3 + F-4 통합)
- SPEC-V3-004 D-2 follow-up (기존 SPEC 확장)
- SPEC-V3-009 master-detail follow-up (기존 SPEC 확장)

---

## §7 v0.2.0 Release Recommendation

### Release Scope (Conservative Estimate)

**Must-Have for v0.2.0 GA:**
1. ✅ v0.1.2 GA carry-forward (28 SPEC base)
2. 🟡 D-4 Global search across workspaces
3. 🟡 D-2 follow-up (workspace switcher complete)
4. 🟡 I-3 Plugin Manager UI
5. 🟡 A-4 Multi-shell picker UI
6. 🟡 F-3 Toolbar wire + F-4 Status Bar 실 binding
7. 🟡 B-1 OSC 8 lifecycle + B-4 SPEC-ID 하이라이트

**Should-Have (시간 허용 시):**
- E-5 Mission Control (parallel agents grid)
- I-1/I-2 Plugin runtime 정식화 + I-4 moai-adk plugin form factor
- F-6 Onboarding tour + env detect
- C-2 Markdown 실 KaTeX 렌더
- G-1 11 panes 실 wire-up
- E-1 SpecPanelView master-detail

**Optional (v0.2.1 tail-cycle 후보):**
- D-5/D-6 (color tags, D&D)
- E-4 Hook event 27 전수 wire
- G-3 Theme picker 확장
- C-3 LSP 풀 통합

### Suggested Sprint Plan (v0.2.0 16-task plan)

| Sprint | Focus | Target SPECs | Demo |
|--------|-------|-------------|------|
| 1 | A-4 + D-2 follow-up + B-1/B-4 polish | V3-002 / V3-004 / V3-LINK-001 | Multi-shell + workspace context menu + OSC 8 click visited |
| 2 | F-3 + F-4 wire + B-5 terminal @MX | V3-006 / V3-LINK-001 | Toolbar 7 buttons + status bar 실 binding |
| 3 | I-1/I-2/I-3 Plugin Architecture base | NEW Plugin SPEC | Settings > Plugins pane + 활성/비활성 토글 |
| 4 | D-4 Global search | NEW Search SPEC | ⌘⇧F → 결과 → 점프 |
| 5 | E-5 Mission Control + E-4 hook wire | V3-010 후속 | 4-cell agent grid + hook stream |
| 6 | I-4 moai-adk plugin form factor + F-6 Onboarding | Plugin SPEC + Onboarding SPEC | 첫 실행 → moai-adk on/off → tour |
| Polish | E-1 master-detail + G-1 wire-up + C-2 KaTeX | 기존 SPEC 확장 | 통합 polish |

### Success Criteria for v0.2.0

- [ ] v0.1.2 baseline 회귀 0 (1148 ui crate tests + 워크스페이스 + 터미널 + spec + 기타 crates)
- [ ] Plugin Architecture I-1/I-2/I-3 통합 완료 + moai-adk plugin form factor (I-4)
- [ ] Global search (D-4) ⌘⇧F → 결과 → 점프 e2e
- [ ] Mission Control (E-5) parallel agents grid + hook stream wire
- [ ] Multi-shell picker (A-4) 5+ shell 선택 가능
- [ ] D-2 workspace switcher 완전 (rename / delete / reorder / 우클릭)
- [ ] F-3 Toolbar 7 actions + F-4 Status Bar 실 binding
- [ ] B-1 OSC 8 lifecycle + B-4 SPEC-ID 하이라이트 렌더
- [ ] @MX 태그 신규 fan_in >= 3 함수에 ANCHOR 추가
- [ ] Playwright e2e 50+ 시나리오 (현재 30+ → 50+)
- [ ] macOS + Linux CI 통과 (Windows 검증은 v0.3.0 carry)
- [ ] 신규 SPEC 6+ 개 추가 + 기존 V3-004/005/007/009/010/011/013/015 폐기 또는 완료

---

## §8 Carry-over to v0.3.0+

### Critical (v0.3.0 release blocker 후보)

- **A-6 Block-based output (Warp model)** — 명령 단위 output grouping. shell prompt 감지 + collapse/expand UI.
- **A-7 Windows named pipe 검증** — Windows GA 동반.
- **B-7 Hover preview (file popup)** — 파일 hover → 내용 미리보기.
- **C-7 Mermaid Renderer** — wry WebView 통합.
- **E-7 Memory Viewer** — `~/.claude/projects/…/memory/` 열람 UI.
- **G-4 Runtime keybinding customization** — 사용자 키바인딩 편집.
- **G-5 Auto-update Tauri-style 통합** — GPUI 네이티브 환경 자동 업데이트.

### Plugin Ecosystem (v3.1 §12 stretch)

- **I-5 Plugin marketplace** (`https://plugins.moaistudio.dev`).
- **I-6 Plugin sandbox (WASM runtime)**.
- 외부 plugin 후보: aider-integration, cursor-mode, themes-nord, themes-dracula, lsp-extra.

### Cross-Platform (v0.3.0+)

- **Windows native build** — GPUI Windows GA 검증.
- **Linux package distribution** — .deb / .rpm / Flatpak (현재 AppImage 만).

### v0.4.0+ (Long-term)

- **E-8 CG Mode** (Claude + GLM split, tmux 기반).
- **External plugin SDK** — Rust 동적 라이브러리 + WASM compatibility.
- **Theme marketplace** — 색 테마 + 폰트 + UI density.

---

## §9 Plugin Architecture Audit (v3.1 §12 신규 영역)

design v3 spec.md v3.1.0 §12 에서 신규 도입된 두-층 구조:

```
Layer 1 (Base · 범용): cmux + Wave + VS Code 융합
Layer 2 (Plugin · 선택): moai-adk GUI overlay
```

### Plugin 구현 상태 매트릭스

| 기본 번들 Plugin | 기본 활성화 | v0.1.2 GA 상태 | Plugin form factor 분리 |
|----------------|------------|----------------|--------------------------|
| `moai-adk` | 유저 선택 (onboarding) | ✅ in-tree 구현 (`spec_ui/`, `quality/`, `viewer/mx_gutter.rs`, `agent/`) | ❌ stub crate 만 (`moai-studio-plugin-moai-adk` 793 bytes) |
| `web-browser` | ✅ | ✅ in-tree 구현 (`web/` 8 modules) | ❌ |
| `image-viewer` | ✅ | ✅ in-tree 구현 (`viewer/image.rs` 27KB) | ❌ |
| `markdown-viewer` | ✅ | ✅ in-tree 구현 (`viewer/markdown/`) | ❌ |
| `json-csv-viewer` | ✅ | 🟡 partial (`viewer/code/` JSON tree-sitter) | ❌ |
| `monaco-editor` | ✅ | 🟡 partial (`viewer/code/` GPUI 자체 렌더, Monaco 미통합) | ❌ |

### Plugin Architecture 도입 전략 (v0.2.0)

**Phase 1 (v0.2.0 cycle)**: Form Factor 분리 없이 visible UI 도입
- I-3 Plugin Manager UI (Settings > Plugins) — moai-adk on/off 토글만 동작
- moai-adk off 상태에서 sidebar SPECs 섹션 / Agent menu / status_bar agent_pill / Command Palette `/moai *` 숨김
- 실제 plugin runtime 분리는 deferred

**Phase 2 (v0.3.0 cycle)**: 실 plugin runtime
- I-1 plugin.toml 정식 schema
- I-2 plugin lifecycle (load/activate/deactivate)
- I-4 moai-adk in-tree → plugin crate 로 추출
- 외부 plugin 첫 시연 (e.g., aider-integration)

**Phase 3 (v0.4.0+ cycle)**: Marketplace + Sandbox
- I-5 plugins.moaistudio.dev marketplace
- I-6 WASM sandbox

---

## §10 v0.1.2 → v0.2.0 Carry-Forward Backlog (CHANGELOG 추출 + design 재매핑)

CHANGELOG `[0.1.2]` 의 "Deferred to v0.2.0" + design v3 spec.md 재매핑:

### Carry from v0.1.2 audit

| Carry ID | Original | v0.2.0 후속 SPEC 후보 | Status |
|----------|----------|----------------------|--------|
| D-4 | Workspace global search | NEW SPEC-V0-2-0-GLOBAL-SEARCH-001 | Top priority |
| D-5 | Workspace color tags | V3-004 D-2 follow-up 흡수 또는 별 SPEC | Quick win |
| D-6 | D&D workspace add | V3-004 D-2 follow-up 흡수 또는 v0.3.0 | Defer |
| E-4 | Hook GPUI wire-up | V3-010 후속 + Mission Control 통합 | Should-have |
| E-5 | Mission Control parallel agents grid | NEW SPEC-V0-2-0-MISSION-CTRL-001 | Top priority |
| E-7 | Memory Viewer | v0.3.0 carry | Defer |
| E-8 | CG Mode | v0.3.0+ carry | Defer |
| B-6 | Terminal Mermaid 감지 | C-7 와 함께 v0.3.0 carry | Defer |
| B-7 | File path hover preview | v0.3.0 carry | Defer |
| C-4 | Browser surface polish (DevTools) | V3-007 follow-up | Should-have |
| C-5 | Image surface polish | ✅ DONE in v0.1.2 (V3-016 MS-1/2/3) | Closed |
| C-6 | JSON / CSV surface | NEW SPEC-V0-2-0-DATA-SURFACE-001 | Optional |
| C-7 | Mermaid render surface | wry 의존 v0.3.0 carry | Defer |
| Quick switcher (⌘/Ctrl+,) | V3-004 D-2 carry | V3-004 D-2 follow-up | Quick win |
| F-3 | Toolbar 실 wiring | NEW SPEC-V0-2-0-TOOLBAR-WIRE-001 | Top priority |
| F-4 | Status Bar 실 wiring | NEW SPEC-V0-2-0-TOOLBAR-WIRE-001 통합 | Top priority |
| F-6 | Onboarding tour | NEW SPEC-V0-2-0-ONBOARDING-001 | Should-have |
| V3-004 D-2 follow-up | Rename modal / delete confirm / reorder / RootView 우클릭 | V3-004 D-2 follow-up | Top priority |
| V3-009 follow-up | SpecPanelView master-detail / AC inline expansion | V3-009 master-detail follow-up | Should-have |

### 신규 v3.1 §12 Plugin Architecture (v0.1.2 audit 누락)

| ID | 후속 SPEC 후보 | Phase |
|----|---------------|-------|
| I-1 plugin.toml manifest | SPEC-V0-2-0-PLUGIN-MGR-001 (Phase 1) + SPEC-V0-3-0-PLUGIN-RUNTIME-001 (Phase 2) | v0.2.0 / v0.3.0 |
| I-2 plugin runtime | 동일 | v0.2.0 / v0.3.0 |
| I-3 Plugin Manager UI | SPEC-V0-2-0-PLUGIN-MGR-001 | v0.2.0 |
| I-4 moai-adk form factor | SPEC-V0-3-0-PLUGIN-RUNTIME-001 | v0.3.0 |
| I-5 marketplace | v0.4.0+ stretch | Defer |
| I-6 sandbox | v0.4.0+ stretch | Defer |

### Stale 매핑 정정 (v0.1.2 audit 의 "stale 항목 7건" 후속)

v0.1.2 audit 에서 식별된 stale mapping 정정 (CHANGELOG 의 "chore(audit)" 참조):

| 원 항목 | v0.1.2 audit 표시 | 실제 v0.1.2 GA 상태 |
|--------|-------------------|---------------------|
| Task 1 V3-LINK-001 | "이미 implemented" | ✅ B-2/B-3 DONE (PR earlier + #67 integration polish) |
| Task 12 V3-008 E-6 Kanban | "owner mismatch (실제 V3-009 RG-SU-3)" | ✅ V3-009 owner 정정 됨, E-6 DONE (PR #31) |
| Task 13 V3-010 | "MS-1/2/3 implemented, E-4/5/8 별도 SPEC" | ✅ V3-010 MS-1/2/3 DONE, E-4/5 v0.2.0 carry |
| Task 14 V3-FS-WATCHER-001 | "PR #43/#48 implemented" | ✅ DONE |
| Task 15 V3-DIST-001 | "PR #49/50/60 implemented" | ✅ DONE (G-5 partial — auto-update 미통합) |
| Task 16 V3-005 surface mapping | "별도 도메인 (B-6/B-7/C-6/C-7)" | ✅ V3-005 file explorer scope 한정, surface 별 SPEC 분리 (V3-016/017 도입) |
| 7번째 항목 | (CHANGELOG 본문 미상세, audit 본문에 7건 식별) | ✅ V3-005 의 자식 surface SPEC 분리 |

v0.2.0 audit 에서는 모든 stale 매핑이 정정된 baseline 위에서 진행.

---

## Appendix A: Methodology

**Data Collection (v0.1.2 GA actual):**
1. Design v3 spec (`/Users/goos/MoAI/moai-studio/.moai/design/v3/spec.md` v3.1.0, 2026-04-21) — 48 features (A 7 + B 7 + C 9 + D 6 + E 8 + F 6 + G 5) + v3.1 §12 Plugin Architecture (I 6) = 54 features 추출.
2. SPEC-V3-001 ~ V3-017, V3-DIST-001, V3-FS-WATCHER-001, V3-LINK-001, V3-PALETTE-001, V0-1-2-MENUS-001 — 23 SPEC 의 status / progress.md 검토.
3. Source code grep (HEAD `1ce6b01d` v0.1.2 GA tag):
   - `crates/moai-studio-ui/src/` (21 modules: panes, tabs, terminal, viewer, settings, palette, explorer, agent, spec_ui, banners, design, web, git, quality, status_bar, toolbar, wizard, workspace_menu, lib.rs)
   - `crates/moai-studio-terminal/src/` (PTY, VT parser, libghostty FFI, link 23KB)
   - `crates/moai-studio-workspace/src/` (Workspace + persistence + panes_convert)
   - `crates/moai-studio-spec/src/` (Kanban + AC state + branch + watch)
   - `crates/moai-studio-plugin-api/`, `crates/moai-studio-plugin-moai-adk/`, `crates/moai-plugin-installer/` (Plugin Architecture, 신규)
   - `crates/moai-hook-http/`, `crates/moai-store/` (Hook server / 저장)
4. Predecessor audit: `.moai/specs/RELEASE-V0.1.2/feature-audit.md` (v0.1.1 → v0.1.2, 2026-04-29).

**Status Definitions:**
- **DONE**: 코드 모듈 존재 + acceptance criteria 충족 + 테스트 통과 (>= 80% coverage).
- **PARTIAL**: 코드 존재하나 incomplete; 일부 acceptance criteria 충족 OR scaffolded.
- **NONE**: 코드베이스에서 구현 미발견.
- **DEFERRED**: design 또는 audit 에서 v0.3.0+ 명시.

**v0.1.2 변화 마커:**
- ⬆: v0.1.2 cycle 에서 진전 있음
- ⬆⬆: v0.1.2 cycle 에서 큰 진전 (NONE → PARTIAL/DONE)
- —: v0.1.2 cycle 에서 변화 없음

**Demo-Visibility Scoring:**
- **HIGH**: main terminal area, toolbar, sidebar, primary modal 가시. 0-1 클릭으로 인터랙션.
- **MED**: 보조 pane, 1-2 클릭으로 발견.
- **LOW**: 내부 / 깊이 내장.

**v0.2.0 Candidacy:**
- Candidates: PARTIAL OR NONE features WITH HIGH demo-visibility AND realistic 5-10 day completion.
- Excludes: DONE features (v0.1.2 에서 완료), LOW demo features (impression 영향 작음), DEFERRED features (v0.3.0+ 예정).

---

**Document Generated by**: MoAI orchestrator (main session, fallback per memory pattern `feedback_main_session_fallback`)
**Last Verified**: 2026-05-01 v0.1.2 GA tag `1ce6b01d`
**Next Review**: v0.2.0 SPEC plan 시작 시 (예상 sprint 1 진입 직후)
