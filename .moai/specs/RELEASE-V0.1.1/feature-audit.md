---
id: RELEASE-V0.1.1-FEATURE-AUDIT
version: 1.0.0
status: draft
created: 2026-04-27
updated: 2026-04-27
classification: design-vs-implementation-audit
parent: RELEASE-V0.1.1
source: .moai/design/v3/spec.md (25 features, 8 신규 frames, 9 menu categories)
---

# v0.1.0 Design vs Implementation Audit

`.moai/design/v3/spec.md` 의 25 functional features + 9 menu categories vs v0.1.0 GA 시점의 실제 구현 매트릭스. 미구현 기능 식별 + v0.1.x / v0.2.0 분할.

## 0. 종합 통계

| 카테고리 | Critical | High | Medium | Low | 합계 |
|---------|---------|------|--------|-----|------|
| Tier A (Terminal Core) | 4 | 3 | 0 | 0 | 7 |
| Tier B (Smart Link) | 4 | 1 | 2 | 0 | 7 |
| Tier C (Surfaces) | 3 | 4 | 2 | 0 | 9 |
| Tier D (Workspace) | 2 | 2 | 1 | 1 | 6 |
| Tier E (moai-adk) | 0 | 5 | 2 | 1 | 8 |
| Tier F (Navigation) | 3 | 3 | 0 | 0 | 6 |
| Tier G (Configuration) | 1 | 2 | 1 | 1 | 5 |
| **합계** | **17** | **20** | **8** | **3** | **48** |

## 1. Tier A — Terminal Core (cmux heritage)

| # | 기능 | 우선순위 | v0.1.0 상태 | 비고 |
|---|------|---------|-------------|------|
| A-1 | 멀티 pane 터미널 (binary tree split) | Critical | 🟢 SPEC-V3-003 GREEN | PaneTree + TabContainer 완료 |
| A-2 | 탭 UI (pane 내부) | Critical | 🟢 SPEC-V3-003 GREEN | TabBar + keybinding |
| A-3 | tmux 완전 호환 (OSC 8, mouse, bracketed paste, 24bit) | Critical | 🟡 SPEC-V3-002 (libghostty-vt 통합) | OSC 8 처리 검증 미완 |
| A-4 | 멀티 쉘 (zsh/bash/fish/nu/pwsh/cmd) | Critical | 🔴 default $SHELL only, 사용자 선택 UI 부재 | v0.1.2 |
| A-5 | 세션 persistence (재시작 시 pane tree 복원) | High | 🟢 SPEC-V3-003 MS-3 GREEN | panes-v1 schema |
| A-6 | Block-based output (Warp 모델) | High | 🔴 미구현 | v0.2.0 |
| A-7 | Unix socket IPC + named pipe (Windows) | High | 🔴 미구현 | v0.2.0 |

**Critical gap (Tier A)**: A-4 멀티 쉘 사용자 선택 UI

## 2. Tier B — Smart Link Handling (MoAI 차별화 핵심)

| # | 기능 | 우선순위 | v0.1.0 상태 | 비고 |
|---|------|---------|-------------|------|
| B-1 | OSC 8 hyperlinks 렌더 + 클릭 | Critical | 🔴 미구현 | libghostty-vt 가 parse, 클릭 wire 부재 |
| B-2 | Regex 파일 경로 자동 감지 (`path:line:col`) | Critical | 🔴 미구현 | TerminalSurface 에 link parser 부재 |
| B-3 | URL 자동 감지 + 하이라이트 | Critical | 🔴 미구현 | 동일 |
| B-4 | SPEC-ID 패턴 감지 (`SPEC-[DOMAIN]-\d+`) | Critical | 🔴 미구현 | 동일 |
| B-5 | @MX 태그 패턴 감지 | High | 🟡 viewer 의 mx_gutter (V3-006) | terminal 영역 미감지 |
| B-6 | Mermaid 코드 블록 감지 | Medium | 🔴 미구현 | v0.2.0+ |
| B-7 | Hover preview (파일 hover 시 popup) | Medium | 🔴 미구현 | v0.2.0+ |

**Critical gap (Tier B 전체 차별화 핵심)**: B-1, B-2, B-3, B-4 모두 미구현. **MoAI Studio 의 USP (unique selling proposition) 가 v0.1.0 에 부재**.

## 3. Tier C — Surfaces (Wave heritage)

| # | 기능 | 우선순위 | v0.1.0 상태 | 비고 |
|---|------|---------|-------------|------|
| C-1 | Terminal Surface (xterm.js WebGL → libghostty-vt) | Critical | 🟢 SPEC-V3-002 GREEN | GPUI 통합 |
| C-2 | Markdown Surface (EARS + KaTeX + Mermaid) | Critical | 🟡 SPEC-V3-006 부분 | KaTeX/Mermaid deferred (V3-006 OD) |
| C-3 | Code Viewer Surface (Monaco + 6 LSP) | High | 🟡 V3-006 partial | tree-sitter 일부, LSP graceful only |
| C-4 | Browser Surface (WebView, DevTools) | Critical | 🔴 미구현 | wry/WebView 통합 별 SPEC |
| C-5 | Image Surface | High | 🟡 V3-006 일부 (binary detect) | zoom/pan/EXIF 미구현 |
| C-6 | JSON / CSV Surface | Medium | 🔴 미구현 | v0.2.0+ |
| C-7 | Mermaid Renderer Surface | Medium | 🔴 미구현 | wry WebView 의존 |
| C-8 | File Tree Surface (재귀, git status) | High | 🟢 SPEC-V3-005 GREEN | git status (#34) |
| C-9 | Agent Run Viewer Surface (Hook timeline) | High | 🟢 SPEC-V3-010 GREEN | 5-pane layout |

**Critical gap (Tier C)**: C-4 Browser Surface (큰 작업, wry crate 통합)

## 4. Tier D — Multi-Project Workspace (VS Code heritage)

| # | 기능 | 우선순위 | v0.1.0 상태 | 비고 |
|---|------|---------|-------------|------|
| D-1 | Workspaces JSON 저장 | Critical | 🟢 GREEN | `~/.moai/studio/workspaces.json` |
| D-2 | 사이드바 workspace 스위처 (드롭다운 + 최근) | Critical | 🟡 부분 | 리스트만, 드롭다운 + 최근 4개 UI 미구현 |
| D-3 | 프로젝트 전환 시 pane tree / tab state 보존 | High | 🟡 부분 | tab_container 단일, workspace 별 격리 미구현 |
| D-4 | 글로벌 검색 across workspaces | High | 🔴 미구현 | v0.2.0+ |
| D-5 | Workspace 별 색상 태그 | Low | 🟡 ws.color 필드 있음 | 모든 ws 가 동일 orange-red 하드코드 (workspace 별 다름 미구현) |
| D-6 | 드래그앤드롭 폴더 추가 | Medium | 🔴 미구현 | v0.2.0 |

## 5. Tier E — moai-adk GUI Overlay (MoAI 특화)

| # | 기능 | 우선순위 | v0.1.0 상태 | 비고 |
|---|------|---------|-------------|------|
| E-1 | SPEC 카드 (현재 활성 SPEC) | High | 🟢 SPEC-V3-009/015 GREEN | List + Detail + SpecPanel overlay |
| E-2 | TRUST 5 대시보드 (5축 레이더) | High | 🟡 V3-010 일부 (run viewer) | 레이더 차트 미구현 |
| E-3 | @MX 태그 거터 + popover | High | 🟢 V3-006 GREEN | mx_gutter |
| E-4 | Hook event stream (27 이벤트) | High | 🟢 V3-010 GREEN | timeline + filter |
| E-5 | Mission Control (parallel agents grid) | High | 🔴 미구현 | v0.2.0 |
| E-6 | Kanban Board (SPEC lifecycle) | Medium | 🟢 V3-009 GREEN | (#31) |
| E-7 | Memory Viewer | Medium | 🔴 미구현 | v0.2.0+ |
| E-8 | CG Mode (Claude + GLM split) | Low | 🔴 미구현 | v0.3.0+ |

## 6. Tier F — Navigation & UX

| # | 기능 | 우선순위 | v0.1.0 상태 | 비고 |
|---|------|---------|-------------|------|
| F-1 | Command Palette (⌘/Ctrl+K) — nested + @/# mention | Critical | 🟢 SPEC-V3-012 GREEN | wire 검증 필요 |
| F-2 | Native menu bar | Critical | 🟡 v0.1.1 (C-5 fix) | 6 menu placeholder, items 미구현 |
| F-3 | Toolbar (7 primary actions) | High | 🔴 미구현 | TitleBar 만 있음, toolbar 없음 |
| F-4 | Status bar (Agent pill + Git + LSP + ⌘K 힌트) | High | 🟡 부분 | "no git" / version / ⌘K 힌트만, Agent/LSP pill 없음 |
| F-5 | Empty State CTA | Critical | 🟢 v0.1.1 fix | hero + create-first + secondary |
| F-6 | Onboarding tour (환경 감지) | High | 🔴 미구현 | v0.2.0 |

**Critical gap (Tier F)**: F-2 native menu bar items 미구현 (placeholder 만), F-3 toolbar 부재

## 7. Tier G — Configuration

| # | 기능 | 우선순위 | v0.1.0 상태 | 비고 |
|---|------|---------|-------------|------|
| G-1 | Settings (General / Hooks / MCP / Skills / Rules / Keybindings) | High | 🟡 V3-013 부분 | KeyboardPane + 4 sub-panes (#25) |
| G-2 | New Workspace Wizard (5-step + 파일 picker) | Critical | 🔴 미구현 | 현재는 단순 폴더 선택만 (5-step wizard 부재) |
| G-3 | Theme switcher (dark/light/auto) | Medium | 🟡 V3-013 일부 | dark only, light/auto 없음 |
| G-4 | Keybinding customization | Low | 🔴 미구현 | v0.3.0+ |
| G-5 | Auto-update | High | 🔴 미구현 | SPEC-V3-011 MS-3 후속 |

**Critical gap (Tier G)**: G-2 5-step Wizard

## 8. Menu Bar 구조 (design §7 IA)

design 가 명시한 9 menu category vs v0.1.1 (C-5 fix 후) 현재 상태:

| Menu | design item | v0.1.1 상태 |
|------|-------------|-------------|
| **MoAI Studio (App)** | About / Settings / Services / Quit | 🟡 About + Quit + Services. Settings (Cmd+,) 누락 |
| **File** | New Workspace / Open / Open Recent / Close Window / Save | 🔴 비어있음 |
| **Edit** | Cut / Copy / Paste / Undo / Redo / Select All / Find | 🟢 7개 OsAction wire |
| **View** | Toggle Sidebar / Toggle Status / Toggle Theme / Reload | 🔴 비어있음 |
| **Pane** | Split Horizontal / Split Vertical / Close Pane / Focus Next | 🔴 비어있음 (design 에는 별 menu) |
| **Surface** | Terminal / Markdown / Code / Browser / Image / JSON | 🔴 비어있음 |
| **SPEC** | Active SPEC / All SPECs / New SPEC / Import | 🔴 비어있음 |
| **Agent** | Start Run / Stop Run / Mission Control / Memory | 🔴 비어있음 |
| **Go** | Workspace Switcher / Recent Files / Goto Line | 🔴 비어있음 |
| **Window** | Minimize / Zoom / Bring All to Front | 🟡 비어있음 (system 자동 minimize/zoom 가능) |
| **Help** | User Guide / Report Issue / Keyboard Shortcuts | 🔴 비어있음 |

**현재 v0.1.1 menu**: 6 menu (App, File, Edit, View, Window, Help) — design 의 11 menu 중 6 부족 (Pane, Surface, SPEC, Agent, Go).

## 9. v0.1.x 분할 권장

### v0.1.1 (현재 hotfix, sess 7)

**범위**: critical regression + minimum-viable user flow.

✅ 완료:
- C-1 RefCell panic fix
- C-2 dead button wire
- C-3 TabContainer 자동 생성
- C-4 Traffic light 중복 제거
- C-5 macOS menu bar 6 menu (App/File/Edit/View/Window/Help) 등록
- H-1 Active dot token 분리

🟡 v0.1.1 추가 권장 (작은 fix):
- File menu 에 "New Workspace ⌘N" item 추가 (Cmd+N keybinding wire)
- Settings 메뉴 항목 (App menu 의 Settings ⌘, → V3-013 settings_modal toggle)
- Help 메뉴 의 "Report Issue" → GitHub issues URL

### v0.1.2 (next minor patch — Wizard + Tier A/F gap)

- A-4 멀티 쉘 선택 UI
- D-2 workspace 드롭다운 + 최근 4개
- D-3 workspace 별 tab_container 격리
- D-5 workspace 색상 태그 (HashMap)
- F-3 Toolbar (7 primary actions)
- G-2 New Workspace Wizard (5-step)
- G-3 Theme switcher light/auto
- View menu items (Toggle Sidebar, Toggle Status)
- Pane menu items (Split, Close, Focus Next)

### v0.2.0 (minor — Smart Link + Browser Surface)

- **Tier B 전체 (B-1~B-4)** — MoAI 차별화 USP 구현
  - SPEC: SPEC-V3-SMART-LINK-001
- C-4 Browser Surface (wry/WebView 통합)
- C-5 Image Surface (zoom/pan/EXIF)
- C-6 JSON/CSV Surface
- C-7 Mermaid Renderer (wry 의존)
- A-6 Block-based output (Warp model)
- A-7 Unix socket IPC + named pipe
- D-4 글로벌 검색
- D-6 드래그앤드롭 폴더 추가
- E-2 TRUST 5 레이더 차트
- E-5 Mission Control
- F-6 Onboarding tour
- Surface menu items
- SPEC menu items
- Agent menu items
- Go menu items
- Pane menu (Pane menu category)

### v0.3.0+ (major — Memory + CG + 고급)

- B-7 Hover preview
- E-7 Memory Viewer
- E-8 CG Mode
- G-4 Keybinding customization
- G-5 Auto-update (SPEC-V3-011 MS-3)
- A-3 OSC 8 클릭 wire 검증

## 10. 본 세션 추가 fix 결정

사용자 피드백 ("File 메뉴 비어있음 + 미구현 기능 찾아서 처리"):

**v0.1.1 즉시 추가** (본 세션):
- F-2 추가: File menu 에 "New Workspace ⌘N" item — handle_add_workspace 호출 wire
- F-2 추가: App menu 의 "Settings ⌘," — V3-013 settings_modal mount
- F-2 추가: Help menu 의 "Report Issue" — `https://github.com/modu-ai/moai-studio/issues` 열기
- F-2 추가: Window menu 의 "Minimize ⌘M" / "Close Window ⌘W" — system action
- D-2 부분: 사이드바 workspace 리스트의 active highlight 강화 (이미 H-1 fix)

**v0.1.2 SPEC plan**: 위 §9 의 v0.1.2 list — 별 SPEC 으로 분리 (다음 세션).

## 11. References

- 원본 design: `.moai/design/v3/spec.md` v3.0 (2026-04-21)
- 원본 system: `.moai/design/v3/system.md`
- master plan: `.moai/design/master-plan.md`
- 본 audit 시점: 2026-04-27 sess 7
- 검토자: MoAI orchestrator (main session, manager-spec fallback per memory pattern)

---

Version: 1.0.0
Status: draft (v0.1.1 즉시 fix 항목 식별 완료, 나머지 v0.1.2 SPEC plan 후보로 분리)
