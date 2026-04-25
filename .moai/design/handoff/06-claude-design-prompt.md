# Claude Design Prompt — MoAI Studio v3

---
title: Claude Design Input Prompt
version: 1.0.0
source: Complete v3 design specification
last_updated: 2026-04-25
---

## 사용 안내

**이 섹션 전체를 복사하여 claude.ai/design 의 "Design Brief" 또는 "Project Description" 에 붙여넣으세요.**

Claude Design 이 본 prompt 와 reference files (다른 .md 문서들) 를 함께 읽으면, moai-studio v3 의 완전한 UI/UX 시안을 생성할 수 있습니다.

---

## DESIGN BRIEF

### 프로젝트명
**MoAI Studio v3** — Cross-platform Agentic Coding IDE

### 한 줄 설명
moai-adk (AI development kit) 의 공식 GUI. 터미널, 코드 에디터, 파일 탐색기, AI 진행 상황 대시보드를 한 화면에 통합. macOS / Linux / Windows cross-platform (GPUI 0.2.2 Rust).

### 비전 & 목표

#### 핵심 비전
- **Agentic coding IDE**: 터미널 + 코드 + AI 진행상황을 동시에 본다
- **Cross-platform native**: Electron 대신 60fps@4K GPUI native rendering
- **Multi-workspace**: 16+ 워크스페이스 병렬 운영 (각각 독립 git worktree + Claude subprocess)
- **Terminal-dominant**: libghostty-vt GPU-accelerated terminal 이 핵심 surface
- **Developer-first**: dark theme 우선, keyboard-centric, 한국어 + 영문 이중언어

#### 타깃 사용자
1. moai-adk 개발자 — `/moai *` 슬래시 커맨드를 1-클릭으로 실행
2. Claude Code 파워유저 — hook event, cost, instructions 그래프 시각화
3. 멀티 에이전트 운영자 — 16+ 병렬 SPEC development
4. macOS/Linux 네이티브 IDE 선호자 — VS Code/Cursor 의 Electron 무게 회피

#### 성공 기준
- **Light time-to-first-mockup**: < 2초
- **Smooth 60fps animation** for pane split, tab switch, scrolling
- **WCAG 2.1 AA compliance** (dark/light theme, keyboard navigation, focus rings)
- **Design consistency** across 9 surfaces × dark/light theme × 4 states (empty/loading/populated/error)

### 디자인 시스템 요약

#### 색상
- **Primary (moai blue)**: `#2563EB` — CTA, active state, focus ring
- **Secondary (AI violet)**: `#8B5CF6` — agent activity
- **Accent (cyan)**: `#06B6D4` — links, terminal highlight
- **Neutral scale (Zinc)**: `#FFFFFF` → `#09090B` (50→950)
- **Semantic**: success `#10B981`, warning `#F59E0B`, error `#EF4444`, info `#3B82F6`

#### Typography
- **Sans (UI)**: Pretendard (한글 우선) → Inter → system-ui
- **Mono (Code/Terminal)**: JetBrains Mono → Fira Code (ligatures)
- **Base size**: 14px (UI body + code editor)
- **Markdown body**: 16px, line-height 1.75 (relaxed)

#### Spacing
- **4-base scale**: 0 / 4 / 8 / 12 / 16 / 20 / 24 / 32 / 40 / 48 / 64 / 80 / 96 px
- **Button padding**: 12px (x) / 8px (y)
- **Default gap**: 16px

#### Radius
- **Default (md)**: 6px (buttons, inputs, cards)
- **Range**: sm 4px ~ 2xl 16px

#### Motion
- **Default duration**: 200ms (normal)
- **Easing**: easeOut (quick appear, slow vanish)
- **Spring**: tab/pane split (bouncy feel)
- **Reduce motion**: all animations → 0ms (WCAG compliance)

### 디자인 의뢰 범위

#### 9개 주요 Surface
1. **Terminal** (SPEC-V3-002, 이미 구현됨) — GPU-accelerated shell, 60fps@4K
2. **Panes + Tabs** (SPEC-V3-003, 구현됨) — Binary tree split, tab bar, last-focused restoration
3. **File Explorer** (SPEC-V3-005, 설계 완료) — File tree, git status badges, fuzzy search
4. **Markdown Viewer** (SPEC-V3-006 MS-1) — CommonMark + GFM, syntax highlight, @MX gutter
5. **Code Viewer** (SPEC-V3-006 MS-2) — LSP diagnostics, syntax highlight, tree-sitter
6. **Agent Dashboard** (SPEC-V3-010) — Event timeline, cost tracking, instructions graph
7. **Git Management** (SPEC-V3-008) — Status, diff, commit, branch UI
8. **SPEC Management** (SPEC-V3-009) — List, detail, Kanban, AC state tracking
9. **Web Browser** (SPEC-V3-007) — URL bar, navigation, dev server auto-detect

#### 각 Surface 상태 조합
- **Dark theme**: neutral.950 (app) / neutral.900 (panel) / neutral.800 (surface)
- **Light theme**: neutral.0 (app) / neutral.50 (panel) / neutral.100 (surface)
- **States**: empty / loading / populated / error

**총 설계 화면**: 9 surfaces × 2 themes × 4 states = **72 상세 화면** (+ 컴포넌트 variation)

#### UI 컴포넌트 라이브러리
- Button (primary / secondary / ghost / destructive) × 4 사이즈 (sm/md/lg/xl)
- Input (text / textarea / select / search)
- Checkbox, Radio, Switch, Toggle
- Tooltip, Popover, Dialog, Modal
- Toast (success / error / warning / info)
- Banner, Loading spinner, Progress bar
- Empty state illustrations

#### 사용자 흐름
1. **First-Run Flow**: Workspace 선택 → .moai/specs load → 첫 탭 생성
2. **File Open Flow**: Explorer 클릭 → 파일 타입별 routing (markdown/code/image/binary)
3. **Pane/Tab Management**: Cmd+T (new tab) → Cmd+\\ (split) → Cmd+W (close)

#### Edge Cases & Error States
- Empty workspace (no files)
- LSP server unavailable (graceful degradation)
- Network failure (agent dashboard, web browser)
- Large file (virtual scrolling, lazy load)
- Corrupted JSON layout (fallback to default)
- Dirty file before close (unsaved changes confirm)

### 기술 제약 (GPUI native rendering)

#### 가능한 것
- Flexbox-like layout (GPUI FlexChild)
- Smooth animations (easing + duration)
- GPU-accelerated scrolling
- Native text rendering (subpixel on macOS)
- Draggable dividers (hit testing + constraints)
- Virtual scrolling (1000+ lines)
- Custom gradients (simple, not complex filters)

#### 불가능한 것
- CSS filters (blur, shadow filter)
- SVG complex paths (simple icons OK)
- HTML/CSS embed (GPUI native only)
- Blur/glassmorphism effects
- 3D transforms

#### Cross-platform Native Handling
- macOS: Cmd key, top menu bar, native file picker
- Linux: Ctrl key, in-window menu, native file picker
- Windows: Ctrl key, in-window menu, native file picker
- Font fallback: system-ui automatic (cross-platform)

### 산출물 요청

#### 1단계: High-Fidelity Mockups (필수)
- 모든 9 surfaces × dark/light = 18 core mockup
- 각 surface × 4 states = +36 variants
- 색상, typography, spacing, radius, shadow 100% 정확
- Design token 일관성 (본 brief 의 색상 hex 값 그대로 사용)
- Figma-style high-fidelity (realistic but GPUI-implementable)

#### 2단계: Interactive Prototype (권장)
- Major surfaces 간 navigation prototype
- Tab switching animation demo
- Pane split/resize interaction demo
- File explorer expand/collapse demo
- Hover/focus/active state transitions

#### 3단계: Design Spec JSON (권장)
- Inspect-mode style properties per component
- Color, font-family, font-size, font-weight, padding, radius 등
- Animation duration/easing
- States (default / hover / active / disabled / focus)
- Layout grid (16px base)

#### 4단계: Component Library (선택)
- Reusable UI components (button, input, etc.) definition
- Sizing scale (sm/md/lg/xl variants)
- Color variants (primary/secondary/ghost/destructive)
- Documentation per component

### 우선순위 & 페이징

#### Tier 0 (기존 구현) — 재확인만
- Terminal (SPEC-V3-002) — 이미 만들어졌음, 시안과 비교 정렬
- Panes + Tabs (SPEC-V3-003) — 이미 구현, 스타일 재확인
- Sidebar, Toolbar, Status bar — 기본 layouting

#### Tier 1 (다음 구현) — Full Design Needed
- File Explorer (SPEC-V3-005) — P1
- Markdown Viewer (SPEC-V3-006 MS-1) — P1
- Code Viewer (SPEC-V3-006 MS-2) — P1
- Git Management (SPEC-V3-008) — P1
- SPEC Management (SPEC-V3-009) — P1
- Agent Dashboard (SPEC-V3-010) — P1

#### Tier 2 (이후) — 추가 설계
- Web Browser (SPEC-V3-007) — P2
- Advanced features — P2+

**권장**: Tier 0 재검증 (2~3일) → Tier 1 full design (1~2주) → Tier 2 추가 (TBD)

### 브랜드 & Tone

#### Tone
**Confident, technical, calmly direct** — 개발자 동료처럼 말한다.

- 정확한 기술 용어 (PaneTree, GPUI Entity, hook event)
- 과장 금지 ("혁신적", "차세대" 등 buzzword 회피)
- 영업/마케팅 어조 금지

#### 사용 색상 정책
- **Dark-first**: 기본 테마는 다크 (neutral.950 배경)
- **Light as alternative**: 밝은 테마는 toggle 가능 (Cmd+Shift+D)
- **Bilingual labels**: 한국어 (Pretendard) + English (Inter) 동등 가시성

#### Example UI Copy
- "파일을 선택하세요" (empty state)
- "로드 중..." (loading state with spinner)
- "에러: 파일을 읽을 수 없습니다" (error state)
- "Cmd+P로 파일을 검색하세요" (helper text)
- "7 tests passed ✓" (success state)

### 참조 문서

본 brief 외에, 다음 files 를 함께 읽으면 더 깊은 이해가 가능합니다:

- **01-app-overview.md** — IA, wireframe, state classification, responsive behavior
- **02-design-system.md** — Detailed token table (colors, typography, spacing, radius, shadow, motion)
- **surfaces/01-terminal.md** — Terminal surface spec (SPEC-V3-002)
- **surfaces/02-panes-tabs.md** — Panes & tabs spec (SPEC-V3-003)
- **surfaces/03-file-explorer.md** — File explorer spec (SPEC-V3-005)
- **surfaces/04-markdown-viewer.md** — Markdown viewer spec (SPEC-V3-006 MS-1)
- **surfaces/05-code-viewer.md** — Code viewer spec (SPEC-V3-006 MS-2)
- **surfaces/06-agent-dashboard.md** — Agent dashboard spec (SPEC-V3-010)
- **surfaces/07-git-management.md** — Git UI spec (SPEC-V3-008)
- **surfaces/08-spec-management.md** — SPEC UI spec (SPEC-V3-009)
- **surfaces/09-web-browser.md** — Web browser spec (SPEC-V3-007)
- **components/buttons-inputs.md** — UI component library
- **components/feedback.md** — Feedback elements (toast, banner, spinner)
- **flows/01-first-run.md** — First-run user flow
- **flows/02-file-open.md** — File open flow
- **flows/03-pane-tab.md** — Pane/tab management flow
- **states/edge-cases.md** — Error states, edge cases

### 추가 정보

#### 다운스트림: Implementation
Claude Design 이 생성한 mockup/prototype 은 다음과 같이 구현됩니다:
- Figma design → GPUI Rust code (expert-frontend agent)
- Design tokens → Rust const (token-to-code generation)
- Component specs → moai-studio-ui crates (18-crate workspace)

#### 리뷰 프로세스
1. Claude Design 시안 검토
2. Implementation team feedback
3. Design iteration (필요시)
4. Final approval + merge to develop

---

## END OF DESIGN BRIEF

**Above 를 복사하여 claude.ai/design 에 붙여넣으세요.**

---

**마지막 수정**: 2026-04-25  
**상태**: 완성 — claude.ai/design 입력 준비됨

