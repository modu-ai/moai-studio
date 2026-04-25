# MoAI Studio v3 — 앱 개요

---
title: Application Overview
version: 1.0.0
source: SPEC-V3-001/002/003, product.md, structure.md, visual-identity.md
last_updated: 2026-04-25
---

## 한 줄 정의

**MoAI Studio v3** 는 **moai-adk (Agentic Development Kit) 의 크로스플랫폼 (macOS / Linux / Windows) Agentic Coding IDE** 이다.

---

## 비전 & 정체성

### 비전 키워드
- **Agentic coding**: 터미널 + 코드 에디터 + AI 진행 상황을 한 화면에서 본다
- **Cross-platform**: macOS / Linux / Windows 동일 경험 (GPUI 0.2.2 native rendering)
- **Multi-workspace**: 16+ 워크스페이스 동시 운영 가능 (각각 독립 git worktree + Claude subprocess)
- **Terminal-dominant**: 60fps@4K GPU 가속 터미널 (libghostty-vt FFI)
- **Dark-first**: 터미널 및 코드 개발자 친화적 다크테마 (light theme 선택사항)
- **Bilingual**: 한국어 (Pretendard) + 영문 (Inter) 동등 지원
- **Keyboard-first**: 모든 작업 Cmd/Ctrl 단축키로 가능 (마우스 보조)

### 타깃 사용자
- **moai-adk 개발자**: `/moai *` 슬래시 커맨드를 GUI 1-클릭으로 실행
- **Claude Code 파워유저**: hook event, cost, instructions 그래프 실시간 관찰
- **macOS/Linux 개발자**: VS Code / Cursor 의 Electron 무게 대신 60fps@4K 네이티브 성능
- **멀티 에이전트 운영자**: SPEC-First workflow 로 16+ 병렬 SPEC 추진
- **SPEC/TRUST/MX 도입팀**: Kanban + @MX tag + TRUST 5 게이트 시각화

---

## 정보 아키텍처 (IA)

### 최상위 네비게이션

```
┌─────────────────────────────────────────────────────────────────┐
│  MoAI Studio  [File] [Edit] [View] [Workspace] [Agent] [Help] ☀ │  ← macOS menu bar
├─────────────────────────────────────────────────────────────────┤
│ Cmd+P [Search icon] ______________________ [Settings] [+Tab]    │  ← Command Palette + Toolbar
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│ [Sidebar:      [Tab Bar]                  [Panel 3]             │
│  Workspace]    ┌────────────────────┐                           │
│  ├─ File       │  Terminal   Code   │                           │
│  ├─ SPEC       │  ─────────────────│  [Divider]                │
│  ├─ Git        │                    │                           │
│  ├─ Agent      │  [Main Content]   │                           │
│  └─ Search     │  (Surface)        │                           │
│                │                    │                           │
│ ┌──────────────┤                    │                           │
│ │ More...      │                    │                           │
│ └──────────────┤                    │                           │
│                └────────────────────┘                           │
│                                                                   │
├──────────────────────────────────────────────────────────────────┤
│ [Breadcrumb] CWD: /path/to/project  |  Agent: idle / running   │
└──────────────────────────────────────────────────────────────────┘
```

### Sidebar 섹션
1. **Workspace** — 현재 프로젝트 + active worktree
2. **File Explorer** — 파일 트리 + git status badge
3. **SPEC Management** — .moai/specs/ 트리 + 상태 indicator
4. **Git** — 현재 branch + staged/unstaged file count
5. **Agent** — 활성 Claude subprocess status
6. **Search** — 전역 search, fuzzy matching

### Tab Bar (수평)
- 각 탭 = 열린 파일 (file path 로 식별)
- Active 탭: 굵은 폰트 (weight 600), 밝은 배경
- Inactive 탭: 가벼운 폰트 (weight 400), 어두운 배경
- Close button (×) — 호버 시 나타남, Cmd+W 와 동등
- Max width per tab = 240px (긴 이름은 truncate 후 tooltip)

### Main Content Area (split panes)
- Binary tree structure (PaneTree)
- Drag-to-resize divider (4px thick, hover= primary.500)
- Min pane size: 240×120 px
- 최대 N개 pane 지원 (일반적 2-4 pane layout)
- 각 pane 은 1개 Surface 호스팅 (Terminal, Code, Markdown, etc.)

### Status Bar (하단)
- 좌: breadcrumb + CWD + git branch
- 우: Agent status + cost (USD) + clock
- 선택: Lint error count, test status, deployment status

---

## 윈도우 레이아웃 Wireframe

### Default 4-pane Layout (권장)

```
┌────────────────┬──────────────────────────────────┐
│   Sidebar      │         Tab Bar                   │
│   (240px)      │ [main.rs] [style.css] [README]   │
├────────────────┼──────────────────────────────────┤
│                │                                   │
│ File           │  Code Viewer                     │
│ ├─ src/        │  (syntax highlight + LSP)        │
│ ├─ tests/      │                                   │
│ └─ docs/       │                                   │
│                ├──────────────────────────────────┤
│ SPEC (3)       │                                   │
│ ├─ V3-001 ✓   │  Terminal                        │
│ ├─ V3-002 ✓   │  $ cargo test                    │
│ └─ V3-003 ✓   │  running...                      │
│                │                                   │
│ Git (0 staged) │                                   │
│ Agent (idle)   ├──────────────────────────────────┤
│                │                                   │
│                │  Markdown Viewer (optional)      │
│                │  # Build Output                  │
│                │  All tests passed ✓              │
│                │                                   │
└────────────────┴──────────────────────────────────┘
```

### Compact Layout (1400px 화면)

```
┌──────┬─────────────────────────────────────────┐
│ Mini │         Content (3-pane)                 │
│ Sidebar       │ Code   │ Terminal │ Markdown    │
│ (collapsed)   │───────│──────────│            │
│               │       │          │            │
└──────┴─────────────────────────────────────────┘
```

### Single Pane Layout (모바일 비목표, 하지만 fallback)

```
┌─────────────────────────────┐
│ Sidebar  [Hamburger menu ≡] │
├─────────────────────────────┤
│                              │
│  Single Content Pane        │
│  (rotate 탈 수 있음)        │
│                              │
└─────────────────────────────┘
```

---

## 화면 상태 분류

### 1. Empty State (파일 없음)
- Sidebar: "No workspace open" CTA + "Open..." button
- Main canvas: Large icon + "Select a file to open"
- 색상: neutral.400 텍스트, neutral.700 배경

### 2. First-Run State (초 실행)
- Tutorial flow: "Welcome to MoAI Studio"
- 1단계: Workspace 선택 (git 루트)
- 2단계: .moai/specs 로드
- 3단계: 첫 탭 자동 생성 (README.md 또는 터미널)

### 3. Loading State (파일/surface 로딩 중)
- Spinner animation (200ms easeOut 반복)
- Progress bar (determinate, 0~100%)
- 텍스트: "Loading main.rs..." / "Analyzing git history..."
- 취소 버튼: Cmd+Escape

### 4. Populated State (정상)
- 모든 surface 렌더 가능
- 탭 + sidebar + status bar 모두 활성
- 사용자 인터랙션 enabled

### 5. Error State (crash / 파일 누락)
- Error banner: neutral.950 배경, error.red 테두리, error.red 아이콘
- 에러 메시지 (친절한 한국어)
- Retry button (Cmd+R) + Dismiss button (Escape)
- Log inspector: 전체 error stack trace 보기

### 6. Agent Running State (Claude subprocess 활성)
- Sidebar "Agent" 섹션: "Running..." + progress
- Status bar 우측: "$(agent-icon) Running MS-1 of 3 steps"
- Tab bar 아래: Event timeline stream (mini)
- 색상: secondary.500 (violet) accent 사용

---

## Dark / Light Theme Toggle

### Theme Switcher 위치
- Toolbar 우측 (status bar 좌측 인접)
- 아이콘: ☀️ (light) / 🌙 (dark)
- Tooltip: "Appearance" / Cmd+Shift+D

### Dark Theme (기본)
- 배경: neutral.950 (app) / neutral.900 (panel) / neutral.800 (surface)
- 텍스트: neutral.50 (primary)
- Border: neutral.700 (default) / primary.500 (focus)
- 모든 색상: tokens.json#color.theme.dark 참조

### Light Theme (대안)
- 배경: neutral.0 (app) / neutral.50 (panel) / neutral.100 (surface)
- 텍스트: neutral.950 (primary)
- Border: neutral.200 (default) / primary.500 (focus)
- 모든 색상: tokens.json#color.theme.light 참조

### 동기화 정책
- Theme preference → `~/.moai/config/appearance.yaml` 저장
- 앱 재시작 시 복원
- CMD+SHIFT+D 토글 시 즉시 theme 전환 (모든 surface 적용)

---

## 네비게이션 흐름

### Tab 간 네비게이션
- Cmd+T: 새 탭 (파일 선택 dialog)
- Cmd+W: 탭 닫기 (dirty 면 confirm)
- Cmd+1/2/3/.../9: 탭 선택 (위치별)
- Cmd+Shift+]: 다음 탭
- Cmd+Shift+[: 이전 탭

### Pane 간 네비게이션
- Cmd+\\: 현재 pane split (horizontal)
- Cmd+Shift+\\: split vertical
- Cmd+}: 오른쪽 pane focus
- Cmd+{: 왼쪽 pane focus
- Cmd+↓ / Cmd+↑: 아래/위 pane focus

### Sidebar 네비게이션
- Cmd+B: Sidebar toggle (expand/collapse)
- Cmd+1: File explorer focus
- Cmd+2: SPEC manager focus
- Cmd+3: Git status focus
- Cmd+4: Agent dashboard focus

### Global
- Cmd+P: Command Palette (search all commands + files)
- Cmd+Shift+P: Command Palette (functions only)
- Cmd+F: Find in current file
- Cmd+H: Replace (if code editor)
- Cmd+/: Comment toggle (if code editor)

---

## Responsive Behavior

### Desktop (1920px+)
- Sidebar 기본 240px (expand 가능 480px)
- Tab bar 항상 표시
- 4-pane layout 최적화

### Laptop (1440px)
- Sidebar 240px, pinned
- Tab bar 표시
- 3-pane layout (code + terminal + optional)

### Compact (1024px)
- Sidebar 축소 (icon only, 60px)
- Tab bar 스크롤 가능
- 2-pane layout (code, terminal)

**모바일 대응 비목표** — desktop-first IDE, 768px 이하 미지원.

---

## 접근성 & 성능

### WCAG 2.1 AA 준수
- 모든 텍스트 대비율 ≥ 4.5:1
- Focus ring: 5px thick, primary.500, 4px offset
- Tab order: 좌→우, 위→아래 논리 흐름
- 키보드만으로 100% 네비게이션 가능

### 성능 목표
- 초기 로드: < 2초
- 탭 전환: < 100ms
- Pane 분할 drag: 60fps smooth
- Syntax highlight 1000+ lines: < 200ms

### Motion & Animation
- Duration: 200ms (normal), 120ms (fast), 320ms (slow)
- Easing: easeOut (default), spring (pane/tab transition)
- prefers-reduced-motion: 전체 animation 0ms 로 비활성화

---

## 크로스플랫폼 일관성

### Identical (모든 플랫폼)
- 색상, typography, spacing, radius, shadow, motion
- UI layout 과 component 형태
- 키 심볼 (Cmd/Ctrl 자동 매핑)

### Platform-Specific
- Key modifier: macOS=Cmd, Linux/Windows=Ctrl (자동 감지)
- Menu bar: macOS 상단, Linux/Windows 윈도우 내부
- Native picker: rfd crate (OS native file picker)
- Font rendering: subpixel (macOS) vs cleartype (Windows)

---

**마지막 수정**: 2026-04-25  
**상태**: 완성 — SPEC-V3-003 reflect

