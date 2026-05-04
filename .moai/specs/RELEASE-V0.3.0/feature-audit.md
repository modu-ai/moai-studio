# Feature Audit — moai-studio v0.2.0 → v0.3.0

**Generated**: 2026-05-04 (sess 15)
**Source**: design v3.1.0 spec (`.moai/design/v3/spec.md`) + v0.2.0 GA actual + audit Top 8 96.9% 결과
**Verified against**: HEAD `e957869` (v0.2.0 GA tag, 2026-05-04)
**Audit Scope**: design v3 의 54 features (A 7 + B 7 + C 9 + D 6 + E 8 + F 6 + G 5 + I 6) + v0.2.0 GA 신규 발견 영역 (mission_control, sse_ingest, onboarding, visited_link_registry, env_report rendering, shell_picker, search, workspace_menu) → 30 SPEC 문서 + 22 crate 매핑.
**Predecessor**: `.moai/specs/RELEASE-V0.2.0/feature-audit.md` (v0.1.2 → v0.2.0)
**Theme**: **Polish & feature completion** (sess 15 사용자 결정)
**Scope**: **Top 16** (carry 4 + 신규 12) — 종합적 backlog 정리 cycle

---

## §1 Summary

| Metric | v0.1.2 audit | v0.2.0 audit (baseline) | **v0.2.0 GA actual** | v0.3.0 target |
|--------|--------------|--------------------------|----------------------|---------------|
| Total features (design v3 + I Plugin) | 48 | 54 | **54** | 54 |
| **DONE** | 4 | 12 | **18** (+6) | 30 |
| **PARTIAL** | 30 | 30 | **24** (-6) | 14 |
| **NONE** | 11 | 6 | **6** | 4 |
| **DEFERRED (v0.4.0+)** | — | 6 | **6** | (이행 + 추가 carry) |

**Status by Tier (v0.2.0 GA actual):**
- Tier A (Terminal Core, 7): 1 DONE → **2 DONE** (+A-4 NEW), 5 PARTIAL → 4 PARTIAL, 0 NONE, 1 DEFERRED
- Tier B (Smart Link, 7): 2 DONE, 3 PARTIAL → 3 PARTIAL (B-1 60%→85%), 1 NONE, 1 DEFERRED
- Tier C (Surfaces, 9): 3 DONE, 5 PARTIAL, 0 NONE, 1 DEFERRED
- Tier D (Workspace, 6): 1 DONE → **3 DONE** (+D-2 +D-4), 2 PARTIAL → 0 PARTIAL, 2 NONE → 2 NONE (D-5/D-6), 1 DEFERRED
- Tier E (moai-adk, 8): 2 DONE, 4 PARTIAL → 4 PARTIAL (+E-5 NEW PARTIAL 90%), 0 NONE → 0 NONE, 2 DEFERRED
- Tier F (UX, 6): 1 DONE → **3 DONE** (+F-3 +F-6), 5 PARTIAL → 2 PARTIAL (F-2/F-4 carry), 0 NONE, 0 DEFERRED
- Tier G (Config, 5): 0 DONE, 4 PARTIAL, 0 NONE, 1 DEFERRED
- Tier I (Plugin Arch v3.1, 6): 0 DONE → **1 DONE** (+I-3), 3 PARTIAL → 2 PARTIAL, 3 NONE → 3 NONE (I-4/I-5/I-6), 0 DEFERRED

**v0.3.0 Top 16 candidates: 16** (carry B-1/E-5/F-4/clippy + polish C-2/C-3/D-5/E-1/E-3/E-4/F-1/F-6-tour/G-1/I-1+I-2/I-4 + B-4)

**Theme alignment**: v0.3.0 = "Polish & feature completion" (사용자 결정) — major new feature push 보다 v0.2.0 carry 마무리 + 미완성 module 완성도 끌어올리기 + Plugin Phase 2 (form factor 분리). v0.2.0 의 GA 모멘텀 (18 PR sequential admin merge) 을 안정적 품질 cycle 로 전환.

---

## §2 Feature Matrix (v0.2.0 GA actual)

### Tier A — Terminal Core (cmux heritage)

| ID | Title | Pri | Status | v0.2.0 변화 | Demo | v0.3.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **A-1** | Multi-pane terminal (binary tree split) | Crit | PARTIAL | — | HIGH | NO | `crates/moai-studio-ui/src/panes/` + `tabs/container.rs`. v0.1.2 PR #64 round-trip 완성 후 변화 없음. T5/T6 cell-grid render path (B-1 의존) 시 영향 가능. |
| **A-2** | Tab UI (in-pane tabs) | Crit | PARTIAL | — | HIGH | NO | `tabs/container.rs` TabContainer + move_tab/duplicate_tab. 변화 없음 (안정). |
| **A-3** | tmux full compat (OSC 8, mouse, 256+24-bit) | Crit | PARTIAL | ⬆ | MED | YES | `moai-studio-terminal/src/vt.rs` libghostty FFI. OSC 8 click full lifecycle 완성 (PR #92/#94 — B-1 ref). hover preview 미완. |
| **A-4** | Multi-shell (zsh/bash/fish/nu/pwsh/cmd) | Crit | **DONE** | ⬆⬆ | MED | NO | `moai-studio-terminal/src/shell.rs` 11.6KB + `moai-studio-ui/src/shell_picker.rs` 5.5KB (PR #83 V0-2-0-MULTI-SHELL-001 GA). Command Palette / toolbar 진입. |
| **A-5** | Session persistence | High | PARTIAL | — | LOW | YES | `moai-studio-workspace/src/persistence.rs` 14KB, `panes_convert.rs` 12KB. v0.2.0 cycle 변화 없음. workspace 별 격리 e2e 검증 미완 → v0.3.0 후보. |
| **A-6** | Block-based output (Warp model) | High | NONE | — | LOW | NO | (no implementation). **DEFERRED to v0.4.0**. |
| **A-7** | Unix socket IPC + named pipe | High | PARTIAL | — | LOW | NO | `moai-studio-terminal/src/pty/unix.rs`. Windows pipe 미검증. **DEFERRED to v0.4.0** (Windows GA 동반). |

### Tier B — Smart Link Handling (MoAI 차별화 핵심)

| ID | Title | Pri | Status | v0.2.0 변화 | Demo | v0.3.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **B-1** | OSC 8 hyperlinks render + click | Crit | PARTIAL (85%) | ⬆⬆ | MED | **YES** | `link.rs` 23KB → 32KB (PR #92/#94 V0-2-0-OSC8-LIFECYCLE-001 MS-1/2). VisitedLinkRegistry + ClickAction::CopyUrl + ClipboardWriter trait + 우클릭 dispatch + visited tracking 완성. **MS-3 carry**: visited 색상 렌더 — T5/T6 cell-grid render path 의존. |
| **B-2** | Regex file path detection (path:line:col) | Crit | DONE | — | HIGH | NO | `link.rs` detect_links() + terminal click handler. v0.2.0 변화 없음 (안정). |
| **B-3** | URL auto-detect + highlight | Crit | DONE | — | HIGH | NO | 동 link.rs + cx.open_url() (V3-LINK-001). PR #67 terminal URL → toast → Browser tab integration. |
| **B-4** | SPEC-ID pattern detection | Crit | PARTIAL | — | MED | **YES** | PR #69 V3-009 MS-4a — 터미널 SPEC-ID 클릭 → SpecPanel mount. terminal-side SPEC-ID 하이라이트 렌더 미완 (B-1 MS-3 와 동반 가능). |
| **B-5** | @MX tag detection | High | PARTIAL | — | MED | NO | `viewer/mx_gutter.rs` 19KB. Code viewer 측 detect 완성. Terminal-side @MX detect 미구현. |
| **B-6** | Mermaid code block detection | Med | NONE | — | LOW | NO | (no terminal-side implementation). C-7 Mermaid surface 와 함께 v0.4.0. |
| **B-7** | Hover preview (file popup) | Med | NONE | — | LOW | NO | (no implementation). **DEFERRED to v0.4.0**. |

### Tier C — Surfaces (Wave heritage)

| ID | Title | Pri | Status | v0.2.0 변화 | Demo | v0.3.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **C-1** | Terminal Surface | Crit | DONE | — | HIGH | NO | SPEC-V3-002 GREEN. `moai-studio-ui/src/terminal/` + libghostty Metal/GLES 렌더. |
| **C-2** | Markdown Surface (EARS + KaTeX + Mermaid) | Crit | PARTIAL | — | HIGH | **YES** | `viewer/markdown/` + math_unicode (LaTeX→Unicode 89 LOC) + mermaid_meta. 실 KaTeX 렌더 + 실 Mermaid 렌더는 placeholder. v0.3.0 후보. |
| **C-3** | Code Viewer (LSP + 6 lang) | High | PARTIAL | — | HIGH | **YES** | `viewer/code/` + tree-sitter (PR #66 JS/JSON 추가). LSP `viewer/lsp.rs` 24KB partial. v0.3.0 후보 (Rust/Go/Python/TS 풀 통합). |
| **C-4** | Browser Surface (WebView + DevTools) | Crit | PARTIAL | — | HIGH | YES | `crates/moai-studio-ui/src/web/` 8 modules. v0.2.0 cycle 변화 없음. DevTools / 풍부한 navigation 미완. |
| **C-5** | Image Surface (zoom/pan/EXIF) | High | DONE | — | MED | NO | V3-016 MS-1/2/3. zoom toolbar + EXIF panel + 6 format decode + SVG. |
| **C-6** | JSON / CSV Surface | Med | PARTIAL | — | MED | NO | JSON tree-sitter syntax-highlighted display. 본격 pretty/tabular surface 별 SPEC 미존재. v0.4.0 후보. |
| **C-7** | Mermaid Renderer Surface | Med | NONE | — | MED | YES | placeholder type detection 만. 실 Mermaid 렌더 → wry WebView 의존. **v0.3.0 candidate** (Polish theme — overdue 결정). |
| **C-8** | File Tree Surface (git status) | High | DONE | — | MED | NO | `moai-studio-ui/src/explorer/` 12 modules. notify watch + FsNode tree + git status. |
| **C-9** | Agent Run Viewer (Hook timeline) | High | PARTIAL | ⬆ | MED | YES | `moai-studio-ui/src/agent/` 7 modules + mission_control_view.rs (PR #88) + sse_ingest pump_into_registry (PR #89). E-4 27 events 풀 wire 미완. |

### Tier D — Multi-Project Workspace (VS Code heritage)

| ID | Title | Pri | Status | v0.2.0 변화 | Demo | v0.3.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **D-1** | Workspaces JSON persistence | Crit | DONE | — | MED | NO | `moai-studio-workspace/src/lib.rs` WorkspacesStore. `~/.moai/studio/workspaces.json`. |
| **D-2** | Sidebar workspace switcher | Crit | **DONE** | ⬆⬆ | HIGH | NO | `workspace_menu.rs` 23.7KB (PR #76 skeleton + PR #82 V3-004 MS-5 + PR #84 MS-6). Rename modal + Delete confirmation + Reorder + RootView 우클릭 wire 모두 완성. AC-D2-1~14 GA. |
| **D-3** | State preserve on project switch | High | PARTIAL | — | LOW | YES | PR #64 panes round-trip + tab CRUD. Workspace 별 격리 실 e2e 검증 미완 (A-5 동반자). v0.3.0 후보. |
| **D-4** | Global search across workspaces | High | **DONE** | ⬆⬆ | MED | NO | `moai-studio-ui/src/search/` 디렉터리 신규 (PR #78~#81 V0-2-0-GLOBAL-SEARCH-001). ripgrep 통합 + SearchResultView + click-to-navigate. |
| **D-5** | Workspace color tags | Low | NONE | — | LOW | **YES** | `ws.color` 필드 존재하나 모든 workspace 가 동일 orange-red 하드코드. HashMap 분리 + UI picker 필요. Quick win. |
| **D-6** | Drag-and-drop workspace add | Med | NONE | — | MED | NO | File picker 동작 OK, D&D 미구현. v0.3.0 secondary 또는 v0.4.0. |

### Tier E — moai-adk GUI Overlay (MoAI 특화)

| ID | Title | Pri | Status | v0.2.0 변화 | Demo | v0.3.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **E-1** | SPEC Card (active SPEC) | High | PARTIAL | ⬆ | MED | **YES** | `spec_ui/` 7 modules. PR #69 터미널 SPEC-ID 클릭 + PR #75 AC chip row. **SpecPanelView master-detail 통합 미완** (list/detail/kanban 별도 view). v0.3.0 후보. |
| **E-2** | TRUST 5 Dashboard (5-axis radar) | High | DONE | — | MED | NO | V3-017. `crates/moai-studio-ui/src/quality/` 62KB: radar + dimension detail + history + quality_gate. v0.2.0 변화 없음 (안정). |
| **E-3** | @MX tag gutter + popover | High | PARTIAL | — | MED | **YES** | `viewer/mx_gutter.rs` 19KB MXPopover + MXGutterLine. 거터 표시 OK, popover hover state + popover positioning + content fetch 미완. v0.3.0 polish 후보. |
| **E-4** | Hook event stream (27 events) | High | PARTIAL | ⬆ | LOW | **YES** | `moai-hook-http/src/` HTTP server OK + sse_ingest.rs 11KB pump_into_registry (PR #89). 27 events 전수 wire 미완. v0.3.0 후보 (E-5 MS-3b 와 통합 가능). |
| **E-5** | Mission Control (parallel agents grid) | High | PARTIAL (90%) | ⬆⬆ | MED | **YES** | `mission_control.rs` 20.6KB AgentRunRegistry (PR #87) + `mission_control_view.rs` GPUI Entity (PR #88) + sse_ingest pump_into_registry (PR #89). **MS-3b carry**: HTTP client subscribe + Cmd+Shift+M (USER-DECISION-MC-A: reqwest vs ureq). |
| **E-6** | Kanban Board (SPEC lifecycle) | Med | DONE | — | MED | NO | `spec_ui/kanban_view.rs` 24KB. SPEC-V3-009 RG-SU-3. |
| **E-7** | Memory Viewer | Med | NONE | — | LOW | YES | `~/.claude/projects/…/memory/` 열람 UI. Polish theme 일환으로 v0.3.0 후보. |
| **E-8** | CG Mode (Claude + GLM split) | Low | NONE | — | LOW | NO | (no implementation). **DEFERRED to v0.5.0+**. |

### Tier F — Navigation & UX

| ID | Title | Pri | Status | v0.2.0 변화 | Demo | v0.3.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **F-1** | Command Palette (⌘/Ctrl+K) | Crit | PARTIAL | — | HIGH | **YES** | `palette/` 6 modules. CommandRegistry 40+ commands + dispatch + @mention. v0.3.0 후보 (60+ commands + category sub-menu polish). |
| **F-2** | Native menu bar | Crit | PARTIAL | — | HIGH | YES | macOS native menu 11 menu. PR #68 4 stub functional. 잔존 stub handler v0.3.0 carry. |
| **F-3** | Toolbar (7 primary actions) | High | **DONE** | ⬆⬆ | HIGH | NO | `moai-studio-ui/src/toolbar.rs` 11.8KB (PR #90 V0-2-0-TOOLBAR-WIRE-001). 7 button + on_mouse_down dispatch. AC-TW-1~6 GA. |
| **F-4** | Status bar (pill + git + LSP) | High | PARTIAL | — | HIGH | **YES** | `moai-studio-ui/src/status_bar.rs` 10KB skeleton (v0.1.2 PR #74). 실 Git/LSP/Agent state binding 미완. v0.3.0 critical (full SPEC STATUS-BAR-WIRE-001). |
| **F-5** | Empty State CTA | Crit | DONE | — | HIGH | NO | `moai-studio-ui/src/lib.rs` empty_state_view (AC-2.2). |
| **F-6** | Onboarding tour (env detect) | High | **DONE (env binding)** | ⬆⬆ | MED | **YES** (interactive tour) | `wizard.rs` 26KB (PR #93/#95) + `onboarding/` 디렉터리 (PR #91). EnvironmentReport state binding + render + cx.spawn auto-detect 완성. **interactive tour 미완** — plugin selection / step-by-step guidance 별 milestone. |

### Tier G — Configuration

| ID | Title | Pri | Status | v0.2.0 변화 | Demo | v0.3.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **G-1** | Settings (General/Hooks/MCP/Skills/Rules/Plugins/Keybindings) | High | PARTIAL | ⬆ | MED | **YES** | `settings/panes/` 11 panes + Plugins (PR #86). 모든 design 영역 pane 도입. **실 wire-up 미완** (Hooks 27 event toggle, MCP server enable/disable, Skills pack 활성, Rules path edit, Keyboard shortcut binding). v0.3.0 critical. |
| **G-2** | New Workspace Wizard | Crit | PARTIAL | ⬆ | HIGH | NO | `wizard.rs` 26KB 5-step structure + env_report integration. 5-step 실 flow + 폼 검증 부분 완성. F-6 와 통합. |
| **G-3** | Theme switcher (dark/light/auto) | Med | PARTIAL | — | HIGH | YES | `settings/panes/appearance.rs` 8KB ActiveTheme. dark default, light/auto picker partial. Nord/Dracula 색 테마 옵션 v0.3.0 secondary. |
| **G-4** | Keybinding customization | Low | PARTIAL | — | LOW | NO | `settings/panes/keyboard.rs` 12KB KeyboardPane. Static display only. **DEFERRED to v0.4.0**. |
| **G-5** | Auto-update | High | PARTIAL | — | LOW | YES | SPEC-V3-DIST-001 PR #49/#50/#60. Homebrew/Scoop/AUR/AppImage. Auto-updater Tauri-style 미통합. v0.3.0 secondary. |

### Tier I — Plugin Architecture (v3.1 §12)

design v3 spec.md v3.1.0 §12 의 Plugin Architecture. v0.2.0 cycle 에서 I-3 GA + I-1/I-2 stub 진행.

| ID | Title | Pri | Status | v0.2.0 변화 | Demo | v0.3.0 후보 | Evidence |
|----|-------|-----|--------|-------------|------|-------------|----------|
| **I-1** | Plugin trait API + manifest schema | Crit | PARTIAL | — | LOW | **YES** | `crates/moai-studio-plugin-api/src/lib.rs` 1.4KB stub. v3.1 spec §12 plugin.toml 정식 schema 미구현. v0.3.0 Plugin Phase 2 critical. |
| **I-2** | Plugin runtime (load/activate/deactivate) | Crit | PARTIAL | — | LOW | **YES** | `crates/moai-plugin-installer/` 27KB: installer + lib + verify. 실 runtime 활성/비활성 cycle 미완. v0.3.0 Plugin Phase 2 critical. |
| **I-3** | Plugin Manager UI (Settings > Plugins) | High | **DONE** | ⬆⬆ | HIGH | NO | `settings/panes/plugins.rs` (PR #86 V0-2-0-PLUGIN-MGR-001). PluginInfo + 6 bundled canonical seed + filtered_plugins. AC-PM-1~7 GA. |
| **I-4** | moai-adk plugin form factor | High | PARTIAL | — | MED | **YES** | `crates/moai-studio-plugin-moai-adk/src/lib.rs` 793 bytes stub. v0.3.0 Plugin Phase 2: in-tree (`spec_ui/`, `quality/`, `agent/`, `viewer/mx_gutter.rs`) → plugin entry point 라우팅 + plugin 비활성화 시 sidebar/menu/palette hidden. |
| **I-5** | Plugin marketplace | Low | NONE | — | LOW | NO | (no implementation). **DEFERRED to v0.5.0+**. |
| **I-6** | Plugin sandbox (WASM runtime) | Low | NONE | — | LOW | NO | (no implementation). **DEFERRED to v0.5.0+**. |

### Bonus: v3.1 spec 외 도입 영역 (참고)

| Area | Status | Evidence | Note |
|------|--------|----------|------|
| **Git UI surface** | DONE | `moai-studio-ui/src/git/` 8 modules 50KB | SPEC-V3-008. v0.2.0 변화 없음 (안정). |
| **Banners (crash/update/lsp)** | PARTIAL | `moai-studio-ui/src/banners/` BannerStack | SPEC-V3-014. 실 시나리오 wire 미검증. v0.3.0 polish 후보. |
| **SPEC Panel Overlay** | PARTIAL | `spec_ui/spec_panel_view.rs` 13KB | SPEC-V3-015. master-detail 통합 v0.3.0 (E-1 동반자). |
| **Sprint Panel** | DONE | `spec_ui/sprint_panel.rs` 10KB | Sprint Contract 통합. |
| **Mission Control View** | PARTIAL (90%) | `agent/mission_control_view.rs` 330 LOC | E-5 의 GPUI 측. MS-3b HTTP carry. |

---

## §3 Per-Tier Breakdown (v0.2.0 GA → v0.3.0 gap)

### Tier A — Terminal Core

v0.2.0 GA 진행:
- **A-4 Multi-shell DONE** (PR #83 V0-2-0-MULTI-SHELL-001): `terminal/shell.rs` 11.6KB + `ui/shell_picker.rs` 5.5KB. 5 shell 선택 UI.
- A-3 OSC 8 click full lifecycle 완성 via B-1 (PR #92/#94)

v0.3.0 critical gap:
- **A-5 Workspace 별 session 격리 e2e 검증** — persistence.rs 14KB 풀 e2e. workspace 전환 시 panes 상태 격리 + tab 상태 복원. 추정 ~400 LOC.
- **A-3 OSC 8 hover preview** — 링크 hover 시 URL params display. B-7 hover preview 와 통합 가능.

v0.4.0 deferred:
- A-6 Block-based output (Warp model)
- A-7 Windows named pipe 검증

### Tier B — Smart Link Handling

v0.2.0 GA 진행:
- **B-1 OSC 8 lifecycle 85%** (PR #92/#94 V0-2-0-OSC8-LIFECYCLE-001 MS-1/2): VisitedLinkRegistry + ClipboardWriter trait + 우클릭 dispatch + visited tracking. logic 모두 완성.

v0.3.0 critical gap:
- **B-1 MS-3 visited 색상 렌더 + T5/T6 cell-grid render path** — 큰 dependency. terminal cell-grid render path 신규 도입 + visited link span 색상/underline. 추정 ~1.5K LOC, 의존성 결정 위험.
- **B-4 SPEC-ID 터미널 하이라이트 렌더** — click 동작 OK, 터미널 측 underline / 색상 강조. B-1 MS-3 와 동반 가능 (~300 LOC additional).
- **B-5 Terminal-side @MX detect** — viewer/code 측 mx_gutter 완성, terminal 측 detect 부재. ~400 LOC.

v0.4.0 deferred:
- B-6 Mermaid code block detection (C-7 동반)
- B-7 Hover preview (file popup)

### Tier C — Surfaces

v0.2.0 GA 진행:
- (Tier C 는 v0.2.0 cycle 직접 작업 없음 — Tier D/E/F 우선)

v0.3.0 critical gap:
- **C-2 Markdown 실 KaTeX 렌더** — placeholder math_unicode 만. 실 KaTeX → GPUI custom rendering 또는 wry 의존 결정. 추정 ~1.5K LOC, 의존성 결정 위험. **Polish theme 의 핵심 (long-overdue)**.
- **C-3 LSP 풀 통합** — viewer/lsp.rs 24KB partial. Rust/Go/Python/TS LSP 풀 통합. 추정 ~1.2K LOC.
- **C-4 Browser DevTools** — surface 24KB + bridge 13KB partial. DevTools 통합 + navigation history UI. 추정 ~600 LOC.
- **C-7 Mermaid Renderer Surface** — wry WebView 의존. C-2 와 함께 의존성 결정. 추정 ~800 LOC.

### Tier D — Multi-Project Workspace

v0.2.0 GA 진행:
- **D-2 fully closed** (PR #76 + #82 + #84): rename / delete / reorder / RootView 우클릭 wire 모두 완성.
- **D-4 Global search DONE** (PR #78~#81 V0-2-0-GLOBAL-SEARCH-001): ripgrep + SearchResultView + click-to-navigate. ⌘⇧F GA.

v0.3.0 critical gap:
- **D-3 Workspace state 격리 e2e 검증** (A-5 동반자) — workspace 별 격리 실 검증.
- **D-5 Workspace color tags 풀 wire** — HashMap 분리 + UI picker. Quick win, ~300 LOC.

v0.3.0 secondary 또는 v0.4.0 deferred:
- D-6 Drag-and-drop workspace add

### Tier E — moai-adk GUI Overlay

v0.2.0 GA 진행:
- **E-5 Mission Control 90%** (PR #87/#88/#89 V0-2-0-MISSION-CTRL-001 MS-1/2/3a): AgentRunRegistry + MissionControlView GPUI + pump_into_registry + Command Palette mission.toggle. **MS-3b carry: HTTP subscribe + 키 바인딩** (USER-DECISION-MC-A).

v0.3.0 critical gap:
- **E-1 SpecPanelView master-detail 통합** — list/detail/kanban 별도 view → 마스터-디테일 통합 + AC inline expansion. ~700 LOC.
- **E-3 @MX tag popover hover 풀 동작** — MXPopover hover state + popover positioning + content fetch. ~500 LOC.
- **E-4 Hook event 27 전수 wire** — moai-hook-http server OK, GPUI 측 27 events 모두 wire. E-5 MS-3b 와 통합 가능. ~600 LOC.
- **E-5 MS-3b carry: HTTP client subscribe + Cmd+Shift+M** — USER-DECISION-MC-A (reqwest vs ureq) 결정 후 lightweight SPEC HOOK-WIRE-001 가능. ~400 LOC.

v0.3.0 secondary:
- E-7 Memory Viewer (`~/.claude/projects/…/memory/` 열람 UI)

v0.4.0+ deferred:
- E-8 CG Mode (Claude + GLM split, tmux 기반)

### Tier F — Navigation & UX

v0.2.0 GA 진행:
- **F-3 Toolbar 7 actions DONE** (PR #90 V0-2-0-TOOLBAR-WIRE-001).
- **F-6 env detect + render DONE** (PR #91/#93/#95 ONBOARDING-ENV-001 + WIZARD-ENV-001 MS-1/2): env_report state + render + auto-detect cx.spawn 모두 완성.

v0.3.0 critical gap:
- **F-4 Status Bar full state binding** — status_bar.rs 10KB skeleton 후속. git2 + LSP + agent runtime 의존성 → cross-component **full SPEC STATUS-BAR-WIRE-001**. 추정 ~700 LOC, 중간 위험.
- **F-1 Command Palette polish (40+ → 60+)** — 추가 commands (recent files, project switcher, theme picker, plugin actions, settings shortcut) + category sub-menu / fuzzy ranking 개선. ~400 LOC.
- **F-2 Native menu 잔존 stub functional** — 11 menu category 중 일부 stub handler (예: View > Zoom, SPEC > Refresh). ~200 LOC.
- **F-6 interactive tour** — env detect 후속, plugin selection (moai-adk on/off) + step-by-step guidance. ~600 LOC.

### Tier G — Configuration

v0.2.0 GA 진행:
- (Tier G 는 v0.2.0 cycle 직접 작업 없음 — v0.1.2 baseline 11 panes 유지)

v0.3.0 critical gap:
- **G-1 Settings 11 panes 실 wire-up** — Hooks 27 event toggle, MCP server enable/disable, Skills pack 활성, Rules path edit, Keyboard shortcut display, Plugins (I-3 와 통합), Editor preferences. 추정 ~1.2K LOC. **Polish theme 의 큰 축 (settings UI 완성도)**.

v0.3.0 secondary:
- G-3 Theme picker 확장 (Nord/Dracula 등 색 테마)
- G-5 Auto-update Tauri-style 통합 (GPUI 네이티브 환경)

v0.4.0 deferred:
- G-4 Runtime keybinding customization

### Tier I — Plugin Architecture (v3.1, Phase 2 진입)

v0.2.0 GA 진행 (Phase 1):
- **I-3 Plugin Manager UI DONE** (PR #86 V0-2-0-PLUGIN-MGR-001): Settings > Plugins pane + PluginInfo + 6 bundled canonical seed + filtered_plugins.
- I-1 plugin-api crate stub (1.4KB 변화 없음)
- I-2 plugin-installer 27KB stub 변화 없음
- I-4 plugin-moai-adk crate stub (793 bytes 변화 없음)

v0.3.0 critical gap (Plugin Phase 2):
- **I-1 plugin.toml 정식 manifest schema** — spec §12 plugin.toml 포맷 정확 구현. permissions / contributes (surfaces, sidebar_sections, statusbar_widgets, commands, link_parsers) 스키마 + 검증. ~500 LOC.
- **I-2 Plugin runtime full lifecycle** — load → activate → handle hook events → deactivate. moai-studio-app 에 plugin registry 통합. ~700 LOC.
- **I-4 moai-adk plugin form factor 분리** — 현재 in-tree (`spec_ui/`, `quality/`, `agent/`, `viewer/mx_gutter.rs`) 를 plugin entry point 로 라우팅. plugin 비활성화 시 sidebar/menu/palette/status_bar agent_pill 항목 hidden. ~1K LOC, **architecture refactor (Lightweight 부적격)**.

v0.5.0+ deferred:
- I-5 Plugin marketplace (`https://plugins.moaistudio.dev`)
- I-6 Plugin sandbox (WASM runtime)

---

## §4 v0.3.0 Recommended Targets (Top 16)

**Theme**: Polish & feature completion (사용자 결정). carry 4 마감 + 미완성 module 12 완성.

### Priority 1 — Carry from v0.2.0 (must close before new feature push)

1. **B-1 MS-3 + T5/T6 cell-grid render path** ⭐⭐⭐⭐⭐
   - **Why**: B-1 audit 85% → GA 마감. visited URL span 색상 렌더 + 새 cell-grid render path 도입 (terminal scroll/select/draw 정확도 향상의 토대).
   - **Demo visibility**: HIGH (terminal 클릭 후 visited 표시).
   - **Scope**: T5/T6 cell-grid render path 도입 (~1K LOC) + B-1 MS-3 visited span 렌더 (~300 LOC) + B-4 SPEC-ID 터미널 하이라이트 렌더 동반 (~300 LOC). 통합 ~1.6K LOC.
   - **SPEC**: SPEC-V0-3-0-T5T6-CELL-GRID-001 (full SPEC, 신규) + SPEC-V0-2-0-OSC8-LIFECYCLE-001 MS-3 amendment.
   - **Risk**: 중간-높음 (cell-grid render path 신규 도입, libghostty FFI 영향 가능).

2. **E-5 MS-3b HOOK-WIRE-001 (HTTP subscribe + Cmd+Shift+M)** ⭐⭐⭐⭐
   - **Why**: E-5 audit 90% → GA 마감 + E-4 Hook event 27 wire 와 통합 가능.
   - **Demo visibility**: MED (Cmd+Shift+M → Mission Control toggle).
   - **Scope**: USER-DECISION-MC-A 결정 (reqwest vs ureq) → HTTP client subscribe (~250 LOC) + Cmd+Shift+M 키 바인딩 (~80 LOC) + E-4 27 events wire (~500 LOC). 통합 ~830 LOC.
   - **SPEC**: SPEC-V0-3-0-HOOK-WIRE-001 (full SPEC, 신규) + SPEC-V0-2-0-MISSION-CTRL-001 MS-3b amendment.
   - **Risk**: 낮음 (sse_ingest pump_into_registry 토대 위에서 작업).
   - **USER-DECISION required**: reqwest (async tokio, ~400KB binary 영향) vs ureq (sync light, ~120KB).

3. **F-4 STATUS-BAR-WIRE-001 (full SPEC)** ⭐⭐⭐⭐⭐
   - **Why**: v0.2.0 audit Top 8 의 일부 (F-3 + F-4 묶음) 였으나 F-4 만 carry. status_bar.rs 10KB skeleton 후속.
   - **Demo visibility**: HIGH (창 하단 상시 표시).
   - **Scope**: AgentPill 실 binding (mission_control AgentRunRegistry + RootView state) + GitWidget 실 binding (git2 status 호출 + workspace path) + LspWidget 실 binding (viewer/lsp.rs 의 LSP diagnostics state). ~700 LOC.
   - **SPEC**: SPEC-V0-3-0-STATUS-BAR-WIRE-001 (full SPEC, 신규).
   - **Risk**: 중간 (cross-component, git2 + LSP + agent runtime 의존성).

4. **clippy mission_control.rs:468 fix PR (drive-by)** ⭐
   - **Why**: PR #89 carry, real CI clippy FAIL 유발하지만 admin override 로 우회 됨. 깔끔한 baseline 확보.
   - **Demo visibility**: NONE (internal hygiene).
   - **Scope**: 1-line `*` deref 정리. ~5 LOC.
   - **SPEC**: 없음 (drive-by fix PR, type/chore 라벨).
   - **Risk**: 없음.

### Priority 2 — Polish (audit Top 16 신규)

5. **G-1 Settings 11 panes 실 wire-up** ⭐⭐⭐⭐
   - **Why**: 11 panes skeleton 후속, **사용자가 실제 설정을 변경할 수 있게 만드는 큰 축**. 현재 모든 pane 이 display-only.
   - **Demo visibility**: MED-HIGH (Cmd+, → 모든 설정 항목 변경 가능).
   - **Scope**: Hooks 27 event toggle (UserSettings 영속) + MCP server enable/disable + Skills pack 활성/비활성 + Rules path edit + Keyboard shortcut display + Plugins (I-3 와 통합) + Editor preferences (font, theme). ~1.2K LOC.
   - **SPEC**: SPEC-V0-3-0-SETTINGS-WIRE-001 (full SPEC, 신규, V3-013 후속).
   - **Risk**: 중간 (cross-pane state management).

6. **E-1 SpecPanelView master-detail 통합** ⭐⭐⭐⭐
   - **Why**: list_view + detail_view + kanban_view + spec_panel_view 분산 → unified master-detail UI. SPEC navigation polish.
   - **Demo visibility**: MED (Cmd+Shift+P → SPEC 검색 → master-detail 화면).
   - **Scope**: master-detail layout (left list + right detail) + AC inline expansion + sticky header + scroll sync. ~700 LOC.
   - **SPEC**: SPEC-V0-3-0-SPEC-PANEL-MASTER-DETAIL-001 (full SPEC, 신규, V3-009/V3-015 후속).
   - **Risk**: 낮음-중간 (state machine 정리 필요).

7. **C-3 LSP 풀 통합 (Rust/Go/Python/TS)** ⭐⭐⭐⭐
   - **Why**: viewer/lsp.rs 24KB partial → 4 언어 LSP 풀 통합. polish theme 의 큰 축 (code viewer 완성도).
   - **Demo visibility**: HIGH (file open → LSP diagnostics + hover + go-to-def).
   - **Scope**: rust-analyzer / gopls / pyright / tsserver 통합 + diagnostics rendering + hover popover + go-to-definition. ~1.2K LOC.
   - **SPEC**: SPEC-V0-3-0-LSP-INTEGRATION-001 (full SPEC, 신규).
   - **Risk**: 중간 (LSP server 별 quirk 처리).

8. **C-2 Markdown 실 KaTeX 렌더** ⭐⭐⭐⭐
   - **Why**: math_unicode placeholder 만. 실 KaTeX 렌더는 long-overdue. Polish theme 의 visible win.
   - **Demo visibility**: HIGH (markdown 파일 open → 수식 렌더).
   - **Scope**: GPUI custom rendering (LaTeX → SVG/Path) **또는** wry WebView 의존 (KaTeX JS). 의존성 결정 후 구현. ~1.5K LOC.
   - **SPEC**: SPEC-V0-3-0-MARKDOWN-KATEX-001 (full SPEC, 신규, USER-DECISION-MD-A: GPUI custom vs wry).
   - **Risk**: 높음 (의존성 결정 + 렌더 정확도).

9. **I-1 + I-2 Plugin runtime full lifecycle (Phase 2 base)** ⭐⭐⭐⭐
   - **Why**: Plugin Architecture Phase 2 진입. plugin.toml 정식 schema + load/activate/deactivate runtime. I-4 form factor 의 전제.
   - **Demo visibility**: LOW (internal infrastructure).
   - **Scope**: I-1 plugin.toml schema (permissions / contributes) + 검증 + 에러 처리 (~500 LOC) + I-2 lifecycle state machine (load → activate → handle hook events → deactivate) + moai-studio-app plugin registry 통합 (~700 LOC). 통합 ~1.2K LOC.
   - **SPEC**: SPEC-V0-3-0-PLUGIN-RUNTIME-001 (full SPEC, 신규).
   - **Risk**: 중간 (architecture refactor base).

10. **I-4 moai-adk plugin form factor 분리** ⭐⭐⭐⭐
    - **Why**: I-1/I-2 위에서 in-tree (`spec_ui/`, `quality/`, `agent/`, `viewer/mx_gutter.rs`) → plugin entry point 라우팅. v3.1 spec §12 의 visible result.
    - **Demo visibility**: HIGH (Settings > Plugins 에서 moai-adk 토글 → sidebar/menu/palette 항목 hidden).
    - **Scope**: 4 module → plugin entry crate (~700 LOC) + RootView 분기 (plugin enabled 체크) + sidebar/menu/palette/status_bar 항목 conditional (~300 LOC). 통합 ~1K LOC.
    - **SPEC**: SPEC-V0-3-0-MOAI-ADK-PLUGIN-FORM-001 (full SPEC, 신규, I-1/I-2 의존).
    - **Risk**: 중간-높음 (architecture refactor, regression 위험).

### Priority 3 — Polish (audit Top 16 후반)

11. **E-3 @MX tag popover hover 풀 동작** ⭐⭐⭐
    - **Why**: viewer/mx_gutter.rs 19KB MXPopover skeleton 후속. hover state + popover positioning + content fetch.
    - **Scope**: hover detection + popover anchor + 거터 클릭 → popover open + ESC close. ~500 LOC.
    - **SPEC**: Lightweight SPEC SPEC-V0-3-0-MX-POPOVER-001 (적격: ≤10KB, AC ≤8, 1 milestone).

12. **F-1 Command Palette polish (40+ → 60+)** ⭐⭐⭐
    - **Why**: 추가 commands (recent files, project switcher, theme picker, plugin actions, settings shortcut, font size, layout) + category sub-menu + fuzzy ranking 개선.
    - **Scope**: CommandRegistry 확장 + fuzzy.rs 가중치 조정 + category 그룹화 UI. ~400 LOC.
    - **SPEC**: Lightweight SPEC SPEC-V0-3-0-PALETTE-POLISH-001.

13. **F-6 interactive tour (env detect 후속)** ⭐⭐⭐
    - **Why**: env detect + render DONE 이후 interactive tour (5-step tutorial — terminal / sidebar / Command Palette / settings / plugin selection). 첫 실행 UX 완성.
    - **Scope**: TourState + step-by-step overlay + plugin selection (moai-adk on/off) + 진행률 표시. ~600 LOC.
    - **SPEC**: Lightweight SPEC SPEC-V0-3-0-ONBOARDING-TOUR-001 (적격: ≤10KB, AC ≤8, 1 milestone).

14. **D-5 Workspace color tags 풀 wire (Quick win)** ⭐⭐
    - **Why**: `ws.color` 필드 존재하나 모든 workspace 가 동일 orange-red 하드코드. HashMap 분리 + UI picker.
    - **Scope**: WorkspacesStore color HashMap + ColorPicker modal (12 preset color) + workspace_menu 통합. ~300 LOC.
    - **SPEC**: Lightweight SPEC SPEC-V0-3-0-WORKSPACE-COLOR-001.

15. **B-4 SPEC-ID 터미널 하이라이트 렌더** ⭐⭐
    - **Why**: B-1 MS-3 와 동반. terminal 측 SPEC-ID 패턴 매칭 → underline / 색상 강조.
    - **Scope**: link.rs SPEC-ID detect span → terminal cell-grid render path 의 highlight layer. T5/T6 와 통합 가능 ~300 LOC additional (#1 항목과 통합 가능).
    - **SPEC**: SPEC-V0-3-0-T5T6-CELL-GRID-001 sub-feature 또는 SPEC-V3-LINK-001 amendment.

16. **C-4 Browser DevTools + navigation history UI** ⭐⭐⭐
    - **Why**: surface 24KB + bridge 13KB partial → DevTools 통합 + navigation history UI.
    - **Scope**: wry DevTools 활성 (debug build 만) + history view (back/forward stack visualization). ~600 LOC.
    - **SPEC**: Lightweight SPEC SPEC-V0-3-0-BROWSER-DEVTOOLS-001 (V3-007 follow-up).

### Secondary (scope 허용 시 — Top 16 외)

- **C-7 Mermaid Renderer Surface** (wry 의존, C-2 와 함께 의존성 결정)
- **A-5 + D-3 Workspace state 격리 e2e 검증** (persistence.rs 풀 e2e)
- **G-3 Theme picker 확장 (Nord/Dracula)**
- **G-5 Auto-update Tauri-style 통합**
- **F-2 Native menu 잔존 stub functional**
- **B-5 Terminal-side @MX detect**
- **E-7 Memory Viewer**
- **D-6 D&D workspace add**

---

## §5 Risk & Feasibility Assessment

### Quick Wins (Priority Low complexity, Lightweight 적격)

- **D-5 Workspace color tags** — HashMap + picker. ~300 LOC.
- **F-1 Command Palette polish** — registry 확장 + ranking 조정. ~400 LOC.
- **F-2 Native menu 잔존 stub** — 4-6 stub handler functional. ~200 LOC.
- **clippy mission_control.rs:468 fix** — drive-by ~5 LOC.
- **C-4 Browser DevTools** — wry built-in 활용. ~600 LOC.

### Medium Effort (Priority Medium complexity, 중간 위험, full SPEC)

- **E-1 SpecPanelView master-detail** — state machine 정리. ~700 LOC.
- **E-3 @MX popover hover** — hover state + positioning. ~500 LOC.
- **F-4 Status Bar full binding** — git2 + LSP + agent runtime. ~700 LOC.
- **F-6 interactive tour** — TourState + overlay. ~600 LOC.
- **E-5 MS-3b HOOK-WIRE** — USER-DECISION + HTTP client + 27 events wire. ~830 LOC.

### Larger Effort (Priority High complexity, 중간-높은 위험)

- **G-1 Settings 11 panes wire-up** — cross-pane state. ~1.2K LOC.
- **C-3 LSP 풀 통합** — 4 LSP server 별 quirk. ~1.2K LOC.
- **I-1 + I-2 Plugin runtime** — architecture refactor base. ~1.2K LOC.
- **I-4 moai-adk plugin form factor** — architecture refactor + regression 위험. ~1K LOC.
- **B-1 MS-3 + T5/T6 cell-grid render** — terminal render path 신규. ~1.6K LOC.
- **C-2 Markdown 실 KaTeX** — USER-DECISION + 렌더 정확도. ~1.5K LOC.

### v0.4.0 Defer 권장

- **C-7 Mermaid Renderer** — C-2 KaTeX 결정 후 follow-up
- **E-7 Memory Viewer** — 우선순위 낮음
- **E-8 CG Mode** — tmux 기반, 별 SPEC 필요
- **G-4 Runtime keybinding** — 우선순위 낮음
- **A-6 Block-based output** — Warp model, 큰 디자인 결정
- **A-7 Windows pipe 검증** — Windows GA 동반
- **B-7 Hover preview** — A-3 hover preview 와 통합

### v0.5.0+ Defer

- **I-5 Plugin marketplace** (`plugins.moaistudio.dev`)
- **I-6 Plugin sandbox (WASM runtime)**
- **외부 plugin 첫 시연** (aider-integration, cursor-mode, themes-nord 등)

---

## §6 SPEC Implementation Coverage (v0.2.0 GA actual)

| SPEC ID | Title | Status | Tier | Est. Completion |
|---------|-------|--------|------|-----------------|
| SPEC-V3-001 | GPUI scaffold + Rust core | ✅ Completed | Critical | 100% |
| SPEC-V3-002 | Terminal Core (libghostty + PTY) | ✅ Completed | Critical | 100% (multi-shell GA) |
| SPEC-V3-003 | Tab / Pane Split | ✅ Completed (MS-4) | Critical | 100% |
| SPEC-V3-004 | Workspace Switcher + Render | ✅ Completed (MS-6) | Critical | **100%** (D-2 fully closed in v0.2.0) |
| SPEC-V3-005 | File Explorer + Surfaces 통합 | 🟡 In Progress | High | 70% (sub-SPEC 분리) |
| SPEC-V3-006 | Markdown / Code Viewer | 🟡 In Progress | High | 90% (KaTeX placeholder, status_bar 별 SPEC) |
| SPEC-V3-007 | Browser Surface (wry) | 🟡 In Progress (MS-4) | High | 80% (DevTools v0.3.0 후보) |
| SPEC-V3-008 | Git Management UI | ✅ Completed | Medium | 100% |
| SPEC-V3-009 | SPEC Management UI | 🟡 In Progress (MS-4a/b) | Medium | 90% (master-detail 통합 v0.3.0) |
| SPEC-V3-010 | Agent Dashboard | 🟡 In Progress | High | **80%** (E-5 MS-1/2/3a GA + MS-3b carry) |
| SPEC-V3-011 | Cross-platform Packaging | 🟡 In Progress | Critical | 50% (auto-update 미통합) |
| SPEC-V3-012 | Palette Surface | ✅ Completed (MS-4) | High | 95% (60+ commands polish v0.3.0) |
| SPEC-V3-013 | Settings Surface | 🟡 In Progress | High | **85%** (11 panes 도입, 실 wire-up v0.3.0) |
| SPEC-V3-014 | Banners Surface | 🟡 In Progress | Medium | 60% (실 시나리오 wire 미검증) |
| SPEC-V3-015 | SPEC Panel Overlay | 🟡 In Progress | Medium | 70% (E-1 master-detail 동반자) |
| SPEC-V3-016 | Image Surface | ✅ Completed | High | 100% |
| SPEC-V3-017 | TRUST 5 Quality Engine | ✅ Completed | High | 100% |
| SPEC-V3-DIST-001 | Distribution Channels | ✅ Completed | Low | 100% |
| SPEC-V3-FS-WATCHER-001 | FS Watcher | ✅ Completed | Low | 100% |
| SPEC-V3-LINK-001 | Smart Link Detection | 🟡 In Progress | Critical | **95%** (B-1 MS-3 carry, B-4 highlight v0.3.0) |
| SPEC-V3-PALETTE-001 | Palette enhancement | ✅ Completed | High | 100% |
| SPEC-V0-1-2-MENUS-001 | Native Menu polish | 🟡 In Progress | High | 90% (잔존 stub v0.3.0) |
| SPEC-V0-2-0-GLOBAL-SEARCH-001 | D-4 Global search | ✅ Completed | High | 100% |
| SPEC-V0-2-0-MISSION-CTRL-001 | E-5 Mission Control | 🟡 In Progress | High | **90%** (MS-3b carry — HTTP subscribe) |
| SPEC-V0-2-0-MULTI-SHELL-001 | A-4 Multi-shell | ✅ Completed | Critical | 100% |
| SPEC-V0-2-0-ONBOARDING-ENV-001 | F-6 env detect | ✅ Completed | High | 100% (env detect logic) |
| SPEC-V0-2-0-OSC8-LIFECYCLE-001 | B-1 OSC 8 lifecycle | 🟡 In Progress (MS-1/2) | Critical | **85%** (MS-3 carry — visited 색상 렌더, T5/T6 의존) |
| SPEC-V0-2-0-PLUGIN-MGR-001 | I-3 Plugin Manager UI | ✅ Completed | High | 100% |
| SPEC-V0-2-0-TOOLBAR-WIRE-001 | F-3 Toolbar wire | ✅ Completed | High | 100% |
| SPEC-V0-2-0-WIZARD-ENV-001 | F-6 wizard render | ✅ Completed (MS-1/2) | High | 100% |

**Key Insight (v0.2.0 GA actual):**
- **Completed (16 SPECs)**: V3-001/002/003/004/008/012/016/017/DIST/FS-WATCHER/PALETTE-001 + V0-2-0-GLOBAL-SEARCH/MULTI-SHELL/ONBOARDING-ENV/PLUGIN-MGR/TOOLBAR-WIRE/WIZARD-ENV = **17 completed** (+5 from v0.1.2 baseline)
- **In Progress (12 SPECs)**: V3-005/006/007/009/010/011/013/014/015/LINK + V0-1-2-MENUS-001 + V0-2-0-MISSION-CTRL/OSC8-LIFECYCLE = **13 partial**
- v0.2.0 release momentum = **18 PR sequential admin --squash merge** (#78~#95) + 1 release prep (#96) = **19 PR / 1 session 14 sprint**

**v0.3.0 신규 SPEC 후보 (Top 16 기준):**
- SPEC-V0-3-0-T5T6-CELL-GRID-001 (B-1 MS-3 + T5/T6 통합, full SPEC)
- SPEC-V0-3-0-HOOK-WIRE-001 (E-5 MS-3b + E-4 27 events 통합, full SPEC)
- SPEC-V0-3-0-STATUS-BAR-WIRE-001 (F-4 full state binding, full SPEC)
- SPEC-V0-3-0-SETTINGS-WIRE-001 (G-1 11 panes 실 wire-up, full SPEC)
- SPEC-V0-3-0-SPEC-PANEL-MASTER-DETAIL-001 (E-1 통합, full SPEC)
- SPEC-V0-3-0-LSP-INTEGRATION-001 (C-3 4 lang LSP, full SPEC)
- SPEC-V0-3-0-MARKDOWN-KATEX-001 (C-2 실 KaTeX, full SPEC, USER-DECISION-MD-A)
- SPEC-V0-3-0-PLUGIN-RUNTIME-001 (I-1 + I-2 Phase 2, full SPEC)
- SPEC-V0-3-0-MOAI-ADK-PLUGIN-FORM-001 (I-4 form factor 분리, full SPEC)
- SPEC-V0-3-0-MX-POPOVER-001 (E-3, lightweight)
- SPEC-V0-3-0-PALETTE-POLISH-001 (F-1 60+ commands, lightweight)
- SPEC-V0-3-0-ONBOARDING-TOUR-001 (F-6 interactive, lightweight)
- SPEC-V0-3-0-WORKSPACE-COLOR-001 (D-5 quick win, lightweight)
- SPEC-V0-3-0-BROWSER-DEVTOOLS-001 (C-4 polish, lightweight)
- (B-4 SPEC-ID 하이라이트 + clippy fix 는 별 SPEC 없음 — T5/T6 amendment + drive-by PR)

**SPEC 분류 합계**: full SPEC 9 + lightweight 5 + amendment/drive-by 2 = **16 deliverables**.

---

## §7 v0.3.0 Release Recommendation

### Release Scope (Polish & Feature Completion)

**Must-Have for v0.3.0 GA:**
1. ✅ v0.2.0 GA carry-forward (17 completed SPEC base)
2. 🟡 **Carry close (4)**: B-1 MS-3 + T5/T6 / E-5 MS-3b HOOK-WIRE / F-4 STATUS-BAR-WIRE / clippy fix
3. 🟡 **Plugin Phase 2 (I-1/I-2/I-4)**: plugin runtime + moai-adk form factor 분리
4. 🟡 **Settings 완성 (G-1)**: 11 panes 실 wire-up
5. 🟡 **SPEC navigation polish (E-1)**: SpecPanelView master-detail
6. 🟡 **Code viewer (C-3)**: LSP 풀 통합 4 lang

**Should-Have (scope 허용 시):**
- C-2 Markdown 실 KaTeX 렌더 (USER-DECISION-MD-A)
- E-3 @MX popover hover 풀 동작
- E-4 Hook event 27 전수 wire (HOOK-WIRE 와 통합 가능)
- F-1 Command Palette polish (60+)
- F-6 interactive tour
- D-5 Workspace color tags
- C-4 Browser DevTools

**Optional (v0.3.x patch 후보):**
- C-7 Mermaid Renderer Surface
- A-5 + D-3 Workspace state 격리 e2e 검증
- B-5 Terminal-side @MX detect
- E-7 Memory Viewer
- G-3 Theme picker 확장 (Nord/Dracula)
- G-5 Auto-update Tauri-style
- F-2 Native menu 잔존 stub
- D-6 D&D workspace add

### Suggested Sprint Plan (v0.3.0 16-task plan)

| Sprint | Focus | Target SPECs | Demo |
|--------|-------|-------------|------|
| 1 | Carry close: clippy + D-5 + F-1 + F-2 (Quick wins) | drive-by + WORKSPACE-COLOR + PALETTE-POLISH + V0-1-2-MENUS amendment | clean baseline + color picker + 60+ commands |
| 2 | F-4 STATUS-BAR-WIRE + E-3 MX-POPOVER (Medium) | STATUS-BAR-WIRE-001 + MX-POPOVER-001 | status bar 실 binding + MX popover hover |
| 3 | E-5 MS-3b HOOK-WIRE + E-4 27 events (Medium) | HOOK-WIRE-001 (full SPEC) | Cmd+Shift+M + 27 events GPUI wire |
| 4 | E-1 master-detail + F-6 interactive tour (Medium) | SPEC-PANEL-MASTER-DETAIL-001 + ONBOARDING-TOUR-001 | SPEC nav + first-run tour |
| 5 | G-1 Settings 실 wire-up (Larger) | SETTINGS-WIRE-001 (full SPEC) | 11 panes 모두 실 동작 |
| 6 | C-3 LSP 4 lang 통합 (Larger) | LSP-INTEGRATION-001 (full SPEC) | code viewer LSP diagnostics + hover + go-to-def |
| 7 | I-1 + I-2 Plugin runtime Phase 2 (Larger) | PLUGIN-RUNTIME-001 (full SPEC) | plugin.toml + lifecycle |
| 8 | I-4 moai-adk plugin form factor (Larger) | MOAI-ADK-PLUGIN-FORM-001 (full SPEC) | Settings > Plugins 토글 → sidebar/menu hidden |
| 9 | B-1 MS-3 + T5/T6 cell-grid + B-4 highlight (Larger) | T5T6-CELL-GRID-001 (full SPEC) + OSC8-LIFECYCLE MS-3 amendment | terminal cell-grid render + visited 색상 |
| 10 | C-2 Markdown 실 KaTeX (Larger, USER-DECISION) | MARKDOWN-KATEX-001 (full SPEC) | KaTeX 렌더 |
| 11 | C-4 Browser DevTools (Quick) | BROWSER-DEVTOOLS-001 (lightweight) | DevTools + history view |
| 12+ | Polish + Should-have | V3-014 banners wire 검증 + secondary | banners + edge cases |

### Success Criteria for v0.3.0

- [ ] v0.2.0 baseline 회귀 0 (1312 ui + 26 workspace + 47 terminal + 129 agent + 기타 crates)
- [ ] Plugin Architecture Phase 2: I-1 + I-2 + I-4 통합 (plugin runtime + moai-adk form factor)
- [ ] Settings G-1: 11 panes 모두 실 wire-up (사용자 설정 변경 e2e)
- [ ] SPEC navigation: E-1 master-detail (list + detail + AC inline)
- [ ] LSP C-3: Rust/Go/Python/TS 4 lang 풀 통합 (diagnostics + hover + go-to-def)
- [ ] F-4 Status Bar: git2 + LSP + agent runtime 실 binding
- [ ] B-1 MS-3 + T5/T6 cell-grid: terminal render path + visited 색상 렌더
- [ ] E-5 MS-3b: HTTP subscribe + Cmd+Shift+M 키 바인딩
- [ ] E-4 Hook event 27 전수 GPUI wire (HOOK-WIRE 통합)
- [ ] @MX 태그 신규 fan_in >= 3 함수에 ANCHOR 추가 (continuous)
- [ ] Playwright e2e 50+ → 80+ 시나리오
- [ ] macOS + Linux CI 통과 (Windows 검증은 v0.4.0 carry)
- [ ] 신규 SPEC 9 full + 5 lightweight = 14 SPEC 추가
- [ ] CHANGELOG ## [0.3.0] entry — Sprint 1~12 종합 분류
- [ ] Test count: ui 1312 → **1700+** / agent 129 → **220+** / terminal 47 → **75+** / workspace 26 → **40+** (+580 누적)

### Velocity Notes

v0.2.0 cycle 의 18 PR / 1 cycle (sess 9~14, single-rolling-session admin merge) 패턴이 검증됨. v0.3.0 도 동일 패턴 사용 권장:
- **Larger SPECs 9개** (G-1 / C-3 / I-1+2 / I-4 / B-1+T5/T6 / C-2 / E-1 / F-4 / HOOK-WIRE) — Priority High complexity sprint
- **Lightweight 5개** — Priority Low complexity sprint
- **메인 세션 직접 fallback 패턴 유지** (sub-agent 1M context 한도 회피, sess 12+ 검증된 패턴)
- Sprint 진행 순서: Quick wins (sprint 1) → Medium → Larger 순으로 momentum 확보

---

## §8 Carry-over to v0.4.0+

### Critical (v0.4.0 release blocker 후보)

- **A-6 Block-based output (Warp model)** — 명령 단위 output grouping. shell prompt 감지 + collapse/expand UI.
- **A-7 Windows named pipe 검증** — Windows GA 동반.
- **B-7 Hover preview (file popup)** — 파일 hover → 내용 미리보기 (A-3 hover preview 와 통합).
- **C-7 Mermaid Renderer Surface** — wry WebView 통합 (C-2 KaTeX 결정 후 follow-up).
- **G-4 Runtime keybinding customization** — 사용자 키바인딩 편집.
- **G-5 Auto-update Tauri-style 통합** — GPUI 네이티브 환경 자동 업데이트.

### Polish & Quality (v0.4.0 secondary)

- **E-7 Memory Viewer** — `~/.claude/projects/…/memory/` 열람 UI.
- **G-3 Theme picker 확장** — Nord/Dracula 등 색 테마.
- **F-2 Native menu 잔존 stub functional** — 11 menu 의 mid-priority stub.
- **B-5 Terminal-side @MX detect** — terminal 측 @MX 태그 감지 + popup.
- **D-6 Drag-and-drop workspace add** — File picker 확장 → drag-drop.
- **A-5 + D-3 Workspace state 격리 e2e 검증** — persistence.rs 풀 e2e (v0.3.0 secondary 진행 시 closed).

### Plugin Ecosystem (v0.5.0+ stretch)

- **I-5 Plugin marketplace** (`https://plugins.moaistudio.dev`).
- **I-6 Plugin sandbox (WASM runtime)**.
- 외부 plugin 후보: aider-integration, cursor-mode, themes-nord, themes-dracula, lsp-extra, vim-keybindings.

### Cross-Platform (v0.4.0+)

- **Windows native build** — GPUI Windows GA 검증.
- **Linux package distribution** — .deb / .rpm / Flatpak (현재 AppImage 만).

### v0.5.0+ (Long-term)

- **E-8 CG Mode** (Claude + GLM split, tmux 기반).
- **External plugin SDK** — Rust 동적 라이브러리 + WASM compatibility.
- **Theme marketplace** — 색 테마 + 폰트 + UI density.
- **Mobile companion** — iPad/Android 보조 view (read-only).

---

## §9 Plugin Architecture Audit (v3.1 §12 Phase 2 진입)

design v3 spec.md v3.1.0 §12 의 두-층 구조:

```
Layer 1 (Base · 범용): cmux + Wave + VS Code 융합
Layer 2 (Plugin · 선택): moai-adk GUI overlay
```

### v0.2.0 GA 시점 Plugin 구현 상태

| 기본 번들 Plugin | 기본 활성화 | v0.2.0 GA 상태 | Plugin form factor 분리 (v0.3.0 target) |
|----------------|------------|----------------|---------------------------------------|
| `moai-adk` | 유저 선택 (onboarding) | ✅ in-tree 구현 + I-3 Plugin Manager UI 진입점 | 🎯 **v0.3.0 I-4 critical** |
| `web-browser` | ✅ | ✅ in-tree 구현 (`web/` 8 modules) | v0.4.0 |
| `image-viewer` | ✅ | ✅ in-tree 구현 (`viewer/image.rs` 27KB) | v0.4.0 |
| `markdown-viewer` | ✅ | 🟡 in-tree (`viewer/markdown/`, KaTeX placeholder) | v0.4.0 (C-2 KaTeX 완성 후) |
| `json-csv-viewer` | ✅ | 🟡 partial (`viewer/code/` JSON tree-sitter) | v0.4.0 |
| `monaco-editor` | ✅ | 🟡 partial (`viewer/code/` GPUI 자체 렌더) | v0.4.0 |

### v0.3.0 Plugin Phase 2 진입 전략

**Phase 2 (v0.3.0 cycle)**: 실 plugin runtime + moai-adk form factor 분리

1. **I-1 plugin.toml manifest 정식 schema** (SPEC-V0-3-0-PLUGIN-RUNTIME-001 part 1)
   - permissions / contributes (surfaces, sidebar_sections, statusbar_widgets, commands, link_parsers) 스키마
   - serde_yaml (혹은 toml) 검증 + 에러 처리
   - moai-studio-plugin-api crate 의 trait 정식화

2. **I-2 Plugin runtime full lifecycle** (SPEC-V0-3-0-PLUGIN-RUNTIME-001 part 2)
   - load → activate → handle hook events → deactivate
   - moai-studio-app PluginRegistry 통합
   - Settings > Plugins 토글 → registry mutation

3. **I-4 moai-adk plugin form factor 분리** (SPEC-V0-3-0-MOAI-ADK-PLUGIN-FORM-001)
   - 현재 in-tree 4 module → moai-studio-plugin-moai-adk crate entry point 라우팅:
     - `spec_ui/` → SpecsPlugin::contributes_sidebar_section
     - `quality/` → TrustQualityPlugin::contributes_statusbar_widget + sidebar
     - `agent/` (mission_control 포함) → AgentDashboardPlugin::contributes_surface + commands
     - `viewer/mx_gutter.rs` → MxGutterPlugin::contributes_gutter_overlay
   - RootView 분기: plugin enabled 체크 → conditional rendering
   - sidebar/menu/palette/status_bar agent_pill 항목 conditional

**Phase 3 (v0.4.0+ cycle)**: 다른 in-tree plugin form factor 분리
- web-browser / image-viewer / markdown-viewer / json-csv-viewer / monaco-editor

**Phase 4 (v0.5.0+ cycle)**: Marketplace + Sandbox
- I-5 plugins.moaistudio.dev marketplace
- I-6 WASM sandbox
- 외부 plugin 첫 시연 (aider-integration 등)

---

## §10 v0.2.0 → v0.3.0 Carry-Forward Backlog

CHANGELOG `[0.2.0]` 의 "Deferred to v0.3.0" + design v3 spec.md 재매핑 + 잔존 carry 4건:

### Carry from v0.2.0 audit Top 8 (직접 carry)

| Carry ID | Original | v0.3.0 후속 SPEC 후보 | Priority |
|----------|----------|----------------------|----------|
| B-1 MS-3 | OSC 8 visited 색상 렌더 (T5/T6 의존) | NEW SPEC-V0-3-0-T5T6-CELL-GRID-001 + OSC8-LIFECYCLE MS-3 amendment | P1 (audit Top 16 #1) |
| E-5 MS-3b | Mission Control HTTP subscribe + 키바인딩 | NEW SPEC-V0-3-0-HOOK-WIRE-001 + MISSION-CTRL MS-3b amendment | P1 (audit Top 16 #2) |
| F-4 | Status Bar 실 state binding | NEW SPEC-V0-3-0-STATUS-BAR-WIRE-001 (full SPEC) | P1 (audit Top 16 #3) |
| clippy mission_control.rs:468 | explicit_auto_deref carry | drive-by fix PR (no SPEC) | P1 (audit Top 16 #4) |

### Carry from v0.2.0 audit Should/Optional (간접 carry)

| Carry ID | Original | v0.3.0 후속 SPEC 후보 | Priority |
|----------|----------|----------------------|----------|
| C-2 | Markdown 실 KaTeX 렌더 | NEW SPEC-V0-3-0-MARKDOWN-KATEX-001 (USER-DECISION-MD-A) | P2 (audit Top 16 #8) |
| C-3 | LSP 풀 통합 (Rust/Go/Python/TS) | NEW SPEC-V0-3-0-LSP-INTEGRATION-001 | P2 (audit Top 16 #7) |
| C-4 | Browser DevTools | NEW SPEC-V0-3-0-BROWSER-DEVTOOLS-001 (lightweight) | P3 (audit Top 16 #16) |
| E-1 | SpecPanelView master-detail | NEW SPEC-V0-3-0-SPEC-PANEL-MASTER-DETAIL-001 | P2 (audit Top 16 #6) |
| E-3 | @MX popover hover 풀 동작 | NEW SPEC-V0-3-0-MX-POPOVER-001 (lightweight) | P3 (audit Top 16 #11) |
| E-4 | Hook event 27 전수 wire | NEW SPEC-V0-3-0-HOOK-WIRE-001 통합 (E-5 MS-3b 와 통합) | P1 (audit Top 16 #2 통합) |
| F-1 | Palette 60+ commands polish | NEW SPEC-V0-3-0-PALETTE-POLISH-001 (lightweight) | P3 (audit Top 16 #12) |
| F-2 | Native menu 잔존 stub | V0-1-2-MENUS-001 amendment (lightweight) | Sprint 1 quick win |
| F-6 | Interactive tour (env detect 후속) | NEW SPEC-V0-3-0-ONBOARDING-TOUR-001 (lightweight) | P3 (audit Top 16 #13) |
| G-1 | Settings 11 panes 실 wire-up | NEW SPEC-V0-3-0-SETTINGS-WIRE-001 (full SPEC) | P2 (audit Top 16 #5) |
| G-3 | Theme picker 확장 (Nord/Dracula) | v0.4.0 carry | Defer |
| G-5 | Auto-update Tauri-style 통합 | v0.4.0 carry | Defer |
| D-3 | Workspace state 격리 e2e | A-5 동반 v0.3.0 secondary | P3-secondary |
| D-5 | Workspace color tags | NEW SPEC-V0-3-0-WORKSPACE-COLOR-001 (lightweight) | P3 (audit Top 16 #14) |
| D-6 | D&D workspace add | v0.4.0 carry | Defer |
| B-4 | SPEC-ID 터미널 하이라이트 렌더 | T5T6-CELL-GRID amendment 또는 LINK-001 amendment | P3 (audit Top 16 #15, #1과 통합) |
| B-5 | Terminal-side @MX detect | v0.4.0 carry | Defer |
| E-7 | Memory Viewer | v0.3.0 secondary 또는 v0.4.0 | Defer |

### Plugin Architecture Phase 2 (v0.3.0 신규 진입)

| ID | 후속 SPEC 후보 | Phase | Priority |
|----|---------------|-------|----------|
| I-1 plugin.toml manifest | NEW SPEC-V0-3-0-PLUGIN-RUNTIME-001 part 1 | Phase 2 | P2 (audit Top 16 #9) |
| I-2 plugin runtime | NEW SPEC-V0-3-0-PLUGIN-RUNTIME-001 part 2 | Phase 2 | P2 (audit Top 16 #9) |
| I-4 moai-adk form factor | NEW SPEC-V0-3-0-MOAI-ADK-PLUGIN-FORM-001 | Phase 2 | P2 (audit Top 16 #10) |
| I-5 marketplace | v0.5.0+ stretch | Phase 4 | Defer |
| I-6 sandbox | v0.5.0+ stretch | Phase 4 | Defer |

### v0.2.0 audit 의 stale 매핑 정정 결과

v0.2.0 audit 의 Top 8 candidates 와 audit Top 8 진척 결과 (96.9%) 의 정확한 분배:

| 원 항목 | v0.2.0 audit 표시 | 실제 v0.2.0 GA 결과 |
|--------|-------------------|---------------------|
| #1 D-4 Global search | ⭐⭐⭐⭐⭐ | ✅ DONE (PR #78~#81) |
| #2 E-5 Mission Control | ⭐⭐⭐⭐⭐ | 🟡 90% (MS-1/2/3a, MS-3b carry) |
| #3 I-3 Plugin Manager UI | ⭐⭐⭐⭐ | ✅ DONE (PR #86) |
| #4 D-2 follow-up | ⭐⭐⭐⭐ | ✅ DONE (PR #82/#84) |
| #5 F-3 Toolbar + F-4 Status Bar | ⭐⭐⭐⭐ | ✅ F-3 DONE (PR #90), 🟡 F-4 carry |
| #6 F-6 Onboarding env detect | ⭐⭐⭐ | ✅ DONE (PR #91/#93/#95) |
| #7 B-1 OSC 8 lifecycle | ⭐⭐⭐ | 🟡 85% (MS-1/2, MS-3 carry) |
| #8 A-4 Multi-shell | ⭐⭐⭐ | ✅ DONE (PR #83) |

**Score**: 6 GA + 0.9 (E-5) + 0.85 (B-1) = **7.75/8 = 96.9%**.

v0.2.0 audit 는 매우 정확했음 (Top 8 추천 적중률 75% GA + 25% partial-90%/85%). v0.3.0 audit 는 동일 방법론 + 확장 (Top 16) 으로 작성.

---

## Appendix A: Methodology

**Data Collection (v0.2.0 GA actual):**
1. Design v3 spec (`/Users/goos/MoAI/moai-studio/.moai/design/v3/spec.md` v3.1.0, 2026-04-21) — 54 features (A 7 + B 7 + C 9 + D 6 + E 8 + F 6 + G 5 + I 6) 추출.
2. SPEC 디렉터리 검토 (30 SPEC):
   - SPEC-V3-001 ~ V3-017, V3-DIST-001, V3-FS-WATCHER-001, V3-LINK-001, V3-PALETTE-001
   - SPEC-V0-1-2-MENUS-001
   - SPEC-V0-2-0-{GLOBAL-SEARCH, MISSION-CTRL, MULTI-SHELL, ONBOARDING-ENV, OSC8-LIFECYCLE, PLUGIN-MGR, TOOLBAR-WIRE, WIZARD-ENV}-001
3. Source code grep (HEAD `e957869` v0.2.0 GA tag):
   - `crates/moai-studio-ui/src/` (24 modules: panes, tabs, terminal, viewer, settings, palette, explorer, agent, spec_ui, banners, design, web, git, quality, status_bar, toolbar, wizard, workspace_menu, onboarding, search, shell_picker, lib.rs)
   - `crates/moai-studio-terminal/src/` (PTY, VT parser, libghostty FFI, link 32KB OSC 8 lifecycle, shell 11.6KB multi-shell)
   - `crates/moai-studio-agent/src/` (mission_control 20.6KB, sse_ingest 11KB, cost, control, events, instructions, ring_buffer, stream_ingest, view, filter, quality)
   - `crates/moai-studio-workspace/src/` (Workspace + persistence + panes_convert)
   - `crates/moai-studio-spec/src/` (Kanban + AC state + branch + watch)
   - `crates/moai-studio-plugin-api/`, `crates/moai-studio-plugin-moai-adk/`, `crates/moai-plugin-installer/` (Plugin Architecture)
   - `crates/moai-hook-http/`, `crates/moai-store/` (Hook server / 저장)
4. Predecessor audit: `.moai/specs/RELEASE-V0.2.0/feature-audit.md` (v0.1.2 → v0.2.0, 2026-05-01).
5. Memory snapshot: `~/.claude/projects/-Users-goos-MoAI-moai-studio/memory/project_current_phase.md` (sess 14 종료 시점, audit Top 8 96.9%).

**Status Definitions:**
- **DONE**: 코드 모듈 존재 + acceptance criteria 충족 + 테스트 통과 (>= 80% coverage).
- **PARTIAL**: 코드 존재하나 incomplete; 일부 acceptance criteria 충족 OR scaffolded.
- **PARTIAL (NN%)**: audit 추적 중인 부분 진척률 (예: B-1 85%, E-5 90%).
- **NONE**: 코드베이스에서 구현 미발견.
- **DEFERRED**: design 또는 audit 에서 v0.4.0+ 명시.

**v0.2.0 변화 마커:**
- ⬆: v0.2.0 cycle 에서 진전 있음
- ⬆⬆: v0.2.0 cycle 에서 큰 진전 (NONE → PARTIAL/DONE 또는 PARTIAL → DONE)
- —: v0.2.0 cycle 에서 변화 없음

**Demo-Visibility Scoring:**
- **HIGH**: main terminal area, toolbar, sidebar, primary modal, status bar 가시. 0-1 클릭으로 인터랙션.
- **MED**: 보조 pane, 1-2 클릭으로 발견.
- **LOW**: 내부 / 깊이 내장.

**v0.3.0 Candidacy:**
- Candidates: PARTIAL OR NONE features WITH (HIGH OR MED) demo-visibility AND realistic single-sprint completion.
- Priority weighting: carry from v0.2.0 (P1) > polish-impactful (P2) > quick wins / lightweight (P3).
- Excludes: DONE features, LOW demo features (impression 영향 작음, e.g. A-7 Windows pipe), DEFERRED features (v0.4.0+ 예정).

**Theme alignment**: v0.3.0 = Polish & feature completion (사용자 결정). 새 major feature push 보다 carry 마감 + 미완성 module 완성도 + Plugin Phase 2 (form factor 분리) 우선.

---

**Document Generated by**: MoAI orchestrator (main session, fallback per memory pattern `feedback_main_session_fallback`)
**Last Verified**: 2026-05-04 v0.2.0 GA tag `e957869`
**Next Review**: v0.3.0 SPEC plan 시작 시 (Sprint 1 진입 직전)
**Related memory**: `~/.claude/projects/-Users-goos-MoAI-moai-studio/memory/project_current_phase.md` (v0.2.0 GA actual snapshot)
