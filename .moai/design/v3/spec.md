# MoAI Studio v3.0 기능 정의 + 크로스플랫폼 재설계 (2026-04-21)

**대폭 pivot**:
- v1/v2: macOS 네이티브 agentic IDE (Swift/AppKit + SwiftUI)
- **v3**: **smart terminal-first multiplexer + moai-adk GUI overlay + multi-project workspace + 크로스플랫폼 (macOS/Windows/Linux)**

본 문서는 [research-v3.md](./research-v3.md) 와 system.md 를 전제한다.

---

## 1. 제품 정체성 (v3)

### Core Thesis (v3.1 — Plugin Architecture)

MoAI Studio 는 **두 층 구조**:

**Layer 1 (Base · 범용)**: cmux + Wave Terminal + VS Code multi-root workspace 의 융합
- Cross-platform native terminal multiplexer (macOS/Windows/Linux)
- Smart link handling (OSC 8 + 범용 regex)
- Inline Surfaces (Markdown/Image/JSON/CSV/Browser/Code/Mermaid)
- Multi-project workspace
- **범용 유저도 moai-adk 없이 사용 가능** (cmux 대체로 기능)

**Layer 2 (Plugin · 선택)**: moai-adk GUI overlay
- SPEC/Plan/Run/Sync 파이프라인
- @MX 태그 거터
- TRUST 5 대시보드
- Hook 27 이벤트 스트림
- Mission Control / Kanban / Memory Viewer / CG Mode
- **번들 플러그인으로 제공, 유저가 활성화/비활성화/제거 가능**
- 첫 실행 시 "moai-adk 사용?" 선택 (기본 활성화 옵션)

**이점**:
- 범용 터미널 시장 (cmux/Wave 경쟁) 진입
- moai-adk 유저에게는 특화 기능 제공
- 플러그인 인프라 → 추후 다른 에이전트/워크플로우 플러그인 (Aider, Cursor integration 등)

### Tagline

> "The native coding shell for AI agents — terminal-first, SPEC-driven, cross-platform."

---

## 2. 신규 제약 사항 (2026-04-21 추가)

### C-1. tmux 지원 필수

Claude Code 의 Agent Teams (CG Mode 포함) 는 tmux 기반. MoAI Studio 의 각 pane 은 tmux 가 내부 실행 가능한 **full-featured PTY** 를 제공해야 한다.

요구사항:
- Full terminfo 지원 (alternate screen, mouse, bracketed paste, truecolor)
- OSC 8 hyperlinks
- UTF-8 + wide char 정확 렌더
- 256-color + 24-bit color
- Bracketed paste, focus events
- Mouse tracking (1000, 1002, 1003, 1006 modes)
- `SHELL=$SHELL tmux new-session` 실행 가능
- tmux 세션 detach/reattach 동작

### C-2. 멀티 쉘 지원

`$SHELL` / 사용자 선택 쉘 자동 실행:

| OS | 기본 쉘 | 지원 목록 |
|----|---------|-----------|
| macOS | zsh | zsh, bash, fish, nu, sh, pwsh |
| Linux | bash | bash, zsh, fish, nu, sh, pwsh, dash |
| Windows | pwsh | pwsh, cmd, bash (WSL), nu |

요구사항:
- 각 pane 생성 시 쉘 선택 가능 (Command Palette 또는 툴바)
- 프로파일 파일 자동 source (`.zshrc`, `.bashrc`, etc.)
- 환경 변수 전달 (`MOAI_WORKSPACE_ID`, `MOAI_PANE_ID`, `MOAI_SOCKET_PATH`)

### C-3. 크로스플랫폼 (macOS / Windows / Linux)

단일 코드베이스로 3 플랫폼 네이티브 빌드. PC/리눅스/맥 개발자 동시 타깃.

배포 형식:
- macOS: `.app` + `.dmg` (notarized, Developer ID 서명)
- Windows: `.exe` + MSIX/MSI 인스톨러 (Authenticode 서명)
- Linux: `.AppImage` + `.deb` + `.rpm` + Flatpak

---

## 3. ⚠️ 아키텍처 결정 지점 (사용자 승인 필요)

C-3 (크로스플랫폼) 이 기존 Swift/AppKit (macOS 전용) 결정을 **근본적으로 재고**하게 합니다.

### 현재 투자
- ✅ Rust core (289 tests, 완전 크로스플랫폼)
- ✅ Swift UI (130 tests, macOS 전용 — **크로스플랫폼 불가**)
- ✅ Ghostty xcframework (Metal 60Hz, macOS 전용)
- ✅ swift-bridge FFI (Rust ↔ Swift)

### 3 가지 경로

#### Path A: **Tauri v2 전면 재작성** (Recommended for 크로스플랫폼)

```
┌─────────────────────────────────────────────┐
│  MoAI Studio (Tauri v2, single codebase)   │
│  ┌─── Frontend (WebView) ─────────────┐   │
│  │  React + TypeScript + Tailwind      │   │
│  │  xterm.js + WebGL renderer          │   │
│  │  Monaco Editor (Code Viewer)        │   │
│  │  shadcn/ui components               │   │
│  │  Design tokens (system.md)          │   │
│  └──────────────────────────────────────┘   │
│                 ↕ Tauri IPC                 │
│  ┌─── Backend (Rust, 재사용) ─────────┐   │
│  │  moai-core crates (289 tests ✓)     │   │
│  │  portable-pty (크로스플랫폼 PTY)    │   │
│  │  tokio + unix socket + named pipe   │   │
│  │  SQLite (rusqlite)                  │   │
│  │  Hook HTTP + WebSocket              │   │
│  └──────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
     ↓ 단일 CI 빌드 → 3 플랫폼 동시
  macOS .app    Windows .exe    Linux .AppImage
```

**장점**:
- Rust 코어 100% 재사용 (289 tests 보존)
- 번들 크기 ~10MB (Electron 100MB 대비)
- WebKit (macOS) / WebView2 (Windows) / WebKitGTK (Linux) — 네이티브 느낌
- 빌트인 updater (Sparkle 대체)
- 단일 CI 파이프라인
- xterm.js WebGL renderer → 60Hz@4K 가능 (Ghostty Metal 준하는 성능)

**단점**:
- Swift UI ~2,500 줄 대체 필요
- 130 Swift tests 재작성 필요 (→ Playwright / Vitest)
- Ghostty Metal → xterm.js WebGL 변경 (벤치마크 검증 필요)
- Pencil 12 frames → React 컴포넌트 매핑 (Pencil-to-code 스킬 활용 가능)
- React/TypeScript 숙련도 전제
- macOS 전용 특화 기능 손실 (MenuBarExtra, etc.)

#### Path B: **플랫폼별 UI 레이어 (Swift + Tauri 병행)**

macOS: 기존 Swift 유지 + 확장
Windows/Linux: Tauri 별도 구현
공유: Rust core

**장점**:
- 기존 Swift 투자 보존
- macOS 는 최고 네이티브 경험

**단점**:
- 유지보수 2x (버그도 2x)
- UX 일관성 격차
- 2 CI 파이프라인
- 기능 시차 발생

#### Path C: **Swift macOS 우선 + Tauri 후속 포팅**

M2.5 까지 완료된 Swift 로 macOS 먼저 배포 → v4 에서 Tauri 재작성

**장점**:
- 즉시 macOS 배포
- 시장 반응 확인 후 투자

**단점**:
- 사실상 2번 개발
- Windows/Linux 유저 수개월 대기
- v4 마이그레이션 시 브랜드 불안정

### 권장: **Path A (Tauri v2 전면 재작성)**

근거:
1. C-3 요구사항 (3 플랫폼 동시) 를 가장 깔끔히 충족
2. Rust core (가장 큰 투자) 완전 보존
3. Swift UI 재작성은 큰 손실로 보이지만, **현재 Swift UI 는 GUI 조립 공백 상태** (사용자 피드백) — 실제 가치 낮음
4. Wave Terminal (Apache-2.0, Electron) 선례 — 크로스플랫폼 터미널 성공 모델
5. Tauri v2 생태계 성숙 (2026 기준 안정)
6. 유지보수 단일 코드베이스의 경제성

---

## 4. 기능 인벤토리 (v3, 25 기능)

크로스플랫폼 + tmux + 멀티쉘 반영. 이전 21 → 25 로 확장.

### Tier A — Terminal Core (cmux heritage)

| # | 기능 | 우선순위 |
|---|------|---------|
| A-1 | 멀티 pane 터미널 (binary tree split) | Critical |
| A-2 | 탭 UI (pane 내부) | Critical |
| A-3 | tmux 완전 호환 (OSC 8, mouse, bracketed paste, 256+24bit color) | Critical |
| A-4 | 멀티 쉘 지원 (zsh/bash/fish/nu/pwsh/cmd) | Critical |
| A-5 | 세션 persistence (재시작 시 pane tree 복원) | High |
| A-6 | Block-based output (Warp 모델, 명령 단위 그룹화) | High |
| A-7 | Unix socket IPC (macOS/Linux) + named pipe (Windows) | High |

### Tier B — Smart Link Handling (**MoAI 차별화 핵심**)

| # | 기능 | 우선순위 |
|---|------|---------|
| B-1 | OSC 8 hyperlinks 렌더 + 클릭 | Critical |
| B-2 | Regex 기반 파일 경로 자동 감지 (`path:line:col`) | Critical |
| B-3 | URL 자동 감지 + 하이라이트 | Critical |
| B-4 | SPEC-ID 패턴 감지 (`SPEC-[DOMAIN]-\d+`) | Critical |
| B-5 | @MX 태그 패턴 감지 (`@MX:(NOTE\|WARN\|ANCHOR\|TODO)`) | High |
| B-6 | Mermaid 코드 블록 감지 (\`\`\`mermaid) | Medium |
| B-7 | Hover preview (파일 hover 시 popup) | Medium |

### Tier C — Surfaces (Wave heritage)

| # | 기능 | 우선순위 |
|---|------|---------|
| C-1 | Terminal Surface (기본, xterm.js WebGL) | Critical |
| C-2 | Markdown Surface (EARS + KaTeX + Mermaid, inline images) | Critical |
| C-3 | Code Viewer Surface (Monaco + 6 언어 LSP) | High |
| C-4 | Browser Surface (WebView, DevTools) | Critical |
| C-5 | Image Surface (zoom/pan, EXIF) | High |
| C-6 | JSON / CSV Surface (pretty display, tabular) | Medium |
| C-7 | Mermaid Renderer Surface (다이어그램 렌더) | Medium |
| C-8 | File Tree Surface (재귀, git status) | High |
| C-9 | Agent Run Viewer Surface (Hook event timeline) | High |

### Tier D — Multi-Project Workspace (VS Code heritage)

| # | 기능 | 우선순위 |
|---|------|---------|
| D-1 | Workspaces JSON 저장 (`~/.moai/studio/workspaces.json`) | Critical |
| D-2 | 사이드바 workspace 스위처 (드롭다운 + 최근) | Critical |
| D-3 | 프로젝트 전환 시 pane tree / tab state 보존 | High |
| D-4 | 글로벌 검색 across workspaces | High |
| D-5 | Workspace 별 색상 태그 | Low |
| D-6 | 드래그앤드롭으로 workspace 추가 (폴더 드롭) | Medium |

### Tier E — moai-adk GUI Overlay (**MoAI 특화**)

| # | 기능 | 우선순위 |
|---|------|---------|
| E-1 | SPEC 카드 (현재 활성 SPEC) | High |
| E-2 | TRUST 5 대시보드 (5축 레이더) | High |
| E-3 | @MX 태그 거터 + popover | High |
| E-4 | Hook event stream (27 이벤트) | High |
| E-5 | Mission Control (parallel agents grid) | High |
| E-6 | Kanban Board (SPEC lifecycle) | Medium |
| E-7 | Memory Viewer (`~/.claude/projects/…/memory/`) | Medium |
| E-8 | CG Mode (Claude + GLM split) | Low |

### Tier F — Navigation & UX

| # | 기능 | 우선순위 |
|---|------|---------|
| F-1 | Command Palette (⌘/Ctrl+K) — nested + @/# mention | Critical |
| F-2 | Native menu bar (플랫폼별: macOS top menu, Windows/Linux app menu) | Critical |
| F-3 | Toolbar (7 primary actions) | High |
| F-4 | Status bar (Agent pill + Git + LSP + ⌘K 힌트) | High |
| F-5 | Empty State CTA (첫 실행 시 Welcome) | Critical |
| F-6 | Onboarding tour (환경 감지 + consent) | High |

### Tier G — Configuration

| # | 기능 | 우선순위 |
|---|------|---------|
| G-1 | Settings (General / Hooks / MCP / Skills / Rules / Keybindings) | High |
| G-2 | New Workspace Wizard (5-step + 파일 picker) | Critical |
| G-3 | Theme switcher (dark/light/auto, 색 테마 옵션) | Medium |
| G-4 | Keybinding customization | Low |
| G-5 | Auto-update (Tauri updater) | High |

---

## 5. 크로스플랫폼 기술 스택 (Path A 기준)

### Frontend

| 레이어 | 선택 | 대안 |
|--------|------|------|
| 프레임워크 | **React 18 + TypeScript** | Vue, Svelte |
| 빌드 | **Vite** | Turbopack |
| UI 컴포넌트 | **shadcn/ui (Radix + Tailwind)** | Mantine, Chakra |
| 터미널 | **xterm.js v5 + WebGL renderer** | hterm |
| 에디터 | **Monaco Editor (VS Code 엔진)** | CodeMirror 6 |
| Markdown | **react-markdown + remark + rehype** | MDX |
| Mermaid | **mermaid v11** | - |
| 수식 | **KaTeX v0.16** | MathJax |
| 아이콘 | **lucide-react** | heroicons |
| 상태 | **Zustand + TanStack Query** | Jotai, Redux |
| 라우팅 | **TanStack Router** | React Router |

### Backend (Rust, 기존 유지)

| 용도 | 선택 |
|------|------|
| 앱 프레임워크 | **Tauri v2** |
| PTY | **portable-pty** (tokio 지원) |
| IPC | Tauri commands + Unix socket (node_pty 대체) |
| 저장 | **rusqlite** (현재 유지) |
| 비동기 | **tokio** (현재 유지) |
| Hook 서버 | **axum + WebSocket** |
| 로깅 | **tracing + tracing-subscriber** |

### 터미널 렌더링 성능

- xterm.js v5 + WebGL renderer: 100K+ 행 원활 스크롤, 60 FPS 가능
- truecolor + 256-color 지원
- OSC 8 hyperlinks 지원 (v4.16+)
- Bracketed paste, mouse modes, alternate screen 모두 지원
- Ghostty Metal 대비 성능 소폭 저하 예상이지만 체감 차이 작음

### 플랫폼 차이 처리

```rust
// Unix (macOS, Linux)
#[cfg(unix)]
fn socket_path() -> PathBuf {
    std::env::temp_dir().join("moai-studio.sock")
}

// Windows
#[cfg(windows)]
fn pipe_name() -> String {
    r"\\.\pipe\moai-studio".to_string()
}
```

| 기능 | macOS | Windows | Linux |
|------|-------|---------|-------|
| PTY | forkpty | ConPTY (Win10 1809+) | forkpty |
| IPC | Unix socket | Named pipe | Unix socket |
| 파일 picker | NSOpenPanel (via Tauri dialog) | IFileDialog (via Tauri dialog) | GTK file chooser (via Tauri dialog) |
| 메뉴 바 | top bar | app menu | app menu |
| 아이콘 | ICNS | ICO | PNG + .desktop |
| 서명 | Developer ID + Notarize | Authenticode | GPG (옵션) |

---

## 6. 기존 12 Pencil Frames 재평가

v3.0 pivot 하에 각 frame 의 적용성 재평가:

| Frame | v2 평가 | v3 재평가 |
|-------|---------|-----------|
| 01. Main Workspace | 유지 | **재설계**: 중앙이 터미널 grid, Right panel 에 agent run, Sidebar 에 workspace 스위처 + SPEC |
| 02. Kanban Board | 유지 | 유지 (E-6) |
| 03. Project Dashboard | 유지 | 유지 + TRUST 5 레이더 강화 (E-2) |
| 04. Code Viewer Deep Dive | 유지 | **Monaco 로 변경** (C-3) |
| 05. Agent Run Viewer | 유지 | 유지 + WebSocket 스트림 (E-4) |
| 06. File Explorer + EARS Markdown | 부분 | **분리**: FileTree Surface (C-8) + Markdown Surface (C-2) |
| 07. Browser + Image Viewer | 유지 | **분리**: Browser (C-4) + Image (C-5) + JSON/CSV (C-6) |
| 08. Command Palette | 유지 | 유지 + @/# mention + SPEC/MX/MCP 섹션 |
| 09. New Workspace Wizard | 유지 | 유지 + 크로스플랫폼 파일 picker (G-2) |
| 10. Settings | 유지 | 유지 + Tiered 섹션 (G-1) |
| 11. Onboarding | 유지 | 유지 + 환경 감지 (플랫폼별 쉘, tmux 탐지) |
| 12. CG Mode View | 유지 | 낮은 우선순위 (E-8) |

### 신규 Frames (v3, 총 8 개)

| Frame | 목적 |
|-------|------|
| 13. Mission Control (v2 계획) | Parallel agents grid |
| 14. Agent Thread (v2 계획) | Conversational + tool calls |
| 15. Context Panel (v2 계획) | @/# mention picker |
| 16. Diff Review (v2 계획) | Hunk-level accept/reject |
| 17. Memory Viewer (v2 계획) | auto-memory 열람 |
| 18. Hooks & MCP Panel (v2 계획) | 27 hooks + MCP 서버 관리 |
| **19. Workspace Switcher** (v3 신규) | 다중 프로젝트 드롭다운 UI |
| **20. Smart Link Hover Preview** (v3 신규) | 터미널 링크 hover popup |

**총 20 Pencil frames** (기존 12 유지 + 신규 8).

---

## 7. Information Architecture (v3)

```
MoAI Studio App
├── Top Bar (플랫폼별)
│   ├── macOS: Menu Bar (native top)
│   ├── Windows/Linux: App Menu (in-window)
│   └── 공통: File / Edit / View / Pane / Surface / SPEC / Agent / Go / Window / Help
│
├── Toolbar (36pt, customizable)
│   [+ New Workspace] [⊟ ⊞ Split] [▶ Run SPEC] [⌘K Palette] [⊙ Agent] [⚠ Diagnostics] [? Help]
│
├── Sidebar (260pt, toggleable)
│   ├── Workspace Switcher (드롭다운) ← D-1/D-2 신규
│   │   └── [프로젝트 A ▼] Switch · + Add · 최근 4개
│   ├── Current Project Section
│   │   ├── Panes Tree (binary tree 시각화)
│   │   ├── SPECs (active/draft/completed)
│   │   ├── Git Worktrees
│   │   └── Recent Files
│   └── [+ New Pane] button (safeAreaBottom)
│
├── Main Pane Area (split-able)
│   ├── Terminal Surface (기본, xterm.js)
│   ├── Code Viewer Surface
│   ├── Markdown Surface
│   ├── Browser Surface
│   ├── Image Surface
│   ├── JSON/CSV Surface
│   ├── Mermaid Renderer
│   └── File Tree Surface
│
├── Right Panel (460pt, toggleable ⌘⌥R)
│   ├── Agent Run Viewer (기본)
│   ├── Context Panel (@/# mention)
│   ├── SPEC Card (active SPEC)
│   └── TRUST 5 게이지
│
├── Status Bar (28pt)
│   └── [⎇ main ↑2] · [LSP] · [⊙ Agent: idle] · [모델] · [⌘K]
│
└── Overlays
    ├── Command Palette (⌘K)
    ├── Mission Control (⌘⇧A)
    ├── Sheets (New Workspace, Settings, Rename)
    └── Smart Link Hover Preview
```

---

## 8. 플랫폼별 키바인딩

| 동작 | macOS | Windows/Linux |
|------|-------|---------------|
| New Workspace | ⌘N | Ctrl+N |
| New Pane | ⌘⇧N | Ctrl+Shift+N |
| New Tab | ⌘T | Ctrl+T |
| Close Tab | ⌘W | Ctrl+W |
| Split Horizontally | ⌘\ | Ctrl+\ |
| Split Vertically | ⌘⇧\ | Ctrl+Shift+\ |
| Command Palette | ⌘K | Ctrl+K |
| Go to File | ⌘P | Ctrl+P |
| Go to Symbol | ⌘⇧P | Ctrl+Shift+P |
| Settings | ⌘, | Ctrl+, |
| Toggle Sidebar | ⌘0 | Ctrl+0 |
| Toggle Agent Run | ⌘⌥R | Ctrl+Alt+R |
| Mission Control | ⌘⇧A | Ctrl+Shift+A |
| Find in Project | ⌘⇧F | Ctrl+Shift+F |
| Focus Terminal | ⌘J | Ctrl+J |
| Clear Terminal | ⌘K (in terminal) | Ctrl+L |

모든 단축키는 app 메뉴에 상응 MenuItem 을 가진다 (플랫폼 관례).

---

## 9. SPEC 로드맵 재조정 (v3)

Path A 선택 시:

| SPEC ID | 제목 | 범위 | 우선순위 |
|---------|------|------|----------|
| SPEC-V3-001 | Tauri v2 스캐폴드 + Rust core 마이그레이션 | 모든 앞선 SPEC 재작성 | Critical |
| SPEC-V3-002 | Terminal Core (xterm.js + PTY + tmux) | A-1~A-7 | Critical |
| SPEC-V3-003 | Smart Link Handling (OSC 8 + regex) | B-1~B-7 | Critical |
| SPEC-V3-004 | Multi-Project Workspace | D-1~D-6 | Critical |
| SPEC-V3-005 | Surfaces (Markdown/Code/Browser/Image/JSON/Mermaid/FileTree) | C-1~C-8 | High |
| SPEC-V3-006 | Command Palette + Menu + Toolbar + Status Bar | F-1~F-6 | High |
| SPEC-V3-007 | Agent Run Viewer + Mission Control + Thread | E-4, E-5 | High |
| SPEC-V3-008 | SPEC Card + TRUST 5 + @MX 거터 + Kanban | E-1, E-2, E-3, E-6 | Medium |
| SPEC-V3-009 | Memory Viewer + Hooks/MCP Panel | E-7, G-1 | Medium |
| SPEC-V3-010 | Settings + Onboarding + New Workspace Wizard | F-5, F-6, G-1~G-3 | High |
| SPEC-V3-011 | Cross-platform packaging + 자동 업데이트 | macOS/Windows/Linux 빌드 | Critical |
| SPEC-V3-012 | E2E 테스트 (Playwright) + Performance 벤치 | 크로스플랫폼 스모크 | High |

### Path A 폐기/재활용

| 기존 SPEC | v3 운명 |
|-----------|---------|
| SPEC-M0-001 | ✅ 유지 (Rust core) |
| SPEC-M1-001 | 부분 재활용 (Rust layers OK, Swift UI 폐기) |
| SPEC-M2-001 | 부분 재활용 (Rust pane/surface schema OK) |
| SPEC-M2-002 | 폐기 (Swift placeholder 수정은 무관) |
| SPEC-M2-003 | 부분 재활용 (state_json persistence 로직 Rust 로 재작성) |
| SPEC-M3-001 | 폐기 (Monaco 로 완전 재작성) |
| SPEC-M2-UX-001/002 | 폐기 |
| SPEC-M5-001~003 | 폐기 (React 로 재작성) |
| SPEC-M6-001 | 폐기 |

---

## 10. Cross-Cutting Concerns (플랫폼별)

### 파일 시스템 경로
- macOS/Linux: `~/.moai/studio/` (`$XDG_CONFIG_HOME/moai-studio/`)
- Windows: `%APPDATA%\moai-studio\`

### 자동 업데이트
- Tauri updater (서명된 JSON manifest)
- macOS: EdDSA 서명 + Developer ID
- Windows: Authenticode 서명 + MSIX 자동 업데이트
- Linux: AppImage updater + Flatpak

### 터미널 렌더링 벤치마크 (Path A 리스크)
- Target: 10K 줄 스크롤 60 FPS
- 도구: xterm.js dev benchmark + Playwright
- Fallback: 성능 미달 시 canvas renderer 로 다운그레이드

### 글로벌 단축키
- macOS: NSEvent global monitor (Tauri 지원)
- Windows: RegisterHotKey
- Linux: X11 grab / Wayland 제한

---

## 11. 성공 기준 (DoD v3)

- ✅ 3 플랫폼 (macOS/Windows/Linux) 배포 가능한 단일 코드베이스
- ✅ 25 기능 전체 구현 + 20 Pencil frames 반영
- ✅ tmux + zsh/bash/fish/pwsh 정상 동작
- ✅ OSC 8 hyperlinks + 모든 custom parsers 동작
- ✅ N 프로젝트 동시 로드 + 전환 + persistence
- ✅ 터미널 60 FPS@10K lines
- ✅ Rust core 289+ tests 유지
- ✅ E2E 30+ Playwright 시나리오
- ✅ 자동 업데이트 3 플랫폼 모두 정상
- ✅ 배포 가능 DMG/MSI/AppImage
- ✅ README + Landing + Demo video

---

---

## 12. Plugin Architecture (v3.1 신규)

### Plugin 정의

MoAI Studio plugin = **Rust 동적 라이브러리 + TS/React UI 컴포넌트 번들**

Tauri v2 plugin system 기반:
- Base app 은 Rust core + React shell
- Plugin 은 Rust crate + React chunk (lazy-loaded)
- Plugin 설치/활성화/제거 runtime 지원

### 기본 번들 Plugins

| Plugin ID | 이름 | 기본 설치 | 기본 활성화 |
|-----------|------|-----------|-------------|
| `moai-adk` | MoAI ADK (SPEC-First DDD + 에이전트) | ✅ | 유저 선택 (onboarding) |
| `web-browser` | WebKit browser surface | ✅ | ✅ |
| `image-viewer` | Image surface | ✅ | ✅ |
| `markdown-viewer` | Markdown + KaTeX + Mermaid | ✅ | ✅ |
| `json-csv-viewer` | 데이터 파일 Pretty display | ✅ | ✅ |
| `monaco-editor` | Code Viewer (LSP + 6 언어) | ✅ | ✅ |

moai-adk 만 onboarding 에서 사용자가 활성/비활성 선택. 나머지는 기본 켜짐.

### Plugin Manifest (`plugin.toml`)

```toml
[plugin]
id = "moai-adk"
name = "MoAI ADK"
version = "1.0.0"
description = "SPEC-First DDD workflow with TRUST 5, @MX tags, and Hook events"
author = "MoAI Team"
homepage = "https://github.com/moai/moai-adk"
license = "Apache-2.0"
min_studio_version = "3.0.0"

[permissions]
workspace = ["read", "write"]
agent = ["run", "observe"]
hook = ["listen", "emit"]
fs = ["read"]
net = ["http"]

[contributes]
surfaces = ["spec-card", "trust5-radar", "agent-run-viewer", "kanban", "memory-viewer", "cg-mode"]
sidebar_sections = ["specs", "worktrees"]
statusbar_widgets = ["agent-pill", "trust5-mini", "model-pill"]

[[contributes.commands]]
id = "moai.plan"
title = "/moai plan"
shortcut_macos = "Cmd+Shift+M P"
shortcut_other = "Ctrl+Shift+M P"

[[contributes.link_parsers]]
id = "spec-id"
pattern = "SPEC-[A-Z0-9]+-\\d+"
action = "open-spec-card"

[[contributes.link_parsers]]
id = "mx-tag"
pattern = "@MX:(NOTE|WARN|ANCHOR|TODO)(?::([A-Z0-9_]+))?"
action = "jump-to-tag"
```

### Plugin API (Rust 쪽)

```rust
// Plugin trait (plugins implement this)
pub trait MoaiPlugin: Send + Sync {
    fn manifest(&self) -> &PluginManifest;
    fn on_activate(&self, ctx: PluginContext) -> Result<()>;
    fn on_deactivate(&self) -> Result<()>;
    fn register_surfaces(&self, registry: &mut SurfaceRegistry);
    fn register_commands(&self, registry: &mut CommandRegistry);
    fn register_link_parsers(&self, registry: &mut LinkParserRegistry);
    fn handle_hook_event(&self, event: HookEvent) -> PluginAction;
}
```

### Plugin API (React 쪽)

```typescript
// Plugin export
export default definePlugin({
  id: "moai-adk",
  surfaces: {
    "spec-card": lazy(() => import("./surfaces/SpecCard")),
    "trust5-radar": lazy(() => import("./surfaces/Trust5Radar")),
    // ...
  },
  sidebar: {
    "specs": lazy(() => import("./sidebar/SpecsList"))
  },
  statusBar: {
    "agent-pill": lazy(() => import("./statusBar/AgentPill"))
  }
});
```

### Plugin 관리 UI

**Settings > Plugins** (Frame 21 신규):

```
┌─────────────────────────────────────────┐
│ Plugins                                 │
├─────────────────────────────────────────┤
│ [Search] [Installed v] [Install From URL]│
├─────────────────────────────────────────┤
│ ✅ moai-adk                   v1.0.0    │
│   SPEC-First DDD + agents              │
│   [Settings] [Disable] [Update]        │
├─────────────────────────────────────────┤
│ ✅ markdown-viewer           v1.0.0    │
│   EARS + KaTeX + Mermaid               │
│   [Settings] [Disable]                  │
├─────────────────────────────────────────┤
│ ⬜ cursor-agent-integration  v0.3.0    │
│   Cursor-style Composer mode           │
│   [Install] [Details]                  │
├─────────────────────────────────────────┤
│  + Install from file (.moai-plugin.zip) │
└─────────────────────────────────────────┘
```

### moai-adk Plugin 비활성화 시 동작

moai-adk plugin 이 꺼진 상태:
- Sidebar: `SPECs` 섹션 숨김, `Git Worktrees` 만 표시
- StatusBar: `agent-pill` 숨김
- Command Palette: `/moai *` 명령어 숨김
- Link parsers: SPEC-ID, @MX tag 파싱 안 함 (일반 텍스트)
- Surfaces: spec-card, agent-run-viewer, kanban, memory-viewer 등 사용 불가
- 메뉴 바: `SPEC`, `Agent` 메뉴 숨김

결과: **순수 cmux 스타일 터미널 앱** (Wave Terminal 과 유사)

### 외부 Plugin 생태계 (미래)

- Marketplace: `https://plugins.moaistudio.dev`
- 플러그인 제출: Pull Request 또는 CLI 배포
- 검증: 보안 스캔, 권한 검토, 호환성 테스트
- 카테고리: 에이전트 integration, 언어 지원, 테마, 도구

예상 플러그인:
- `aider-integration`: Aider CLI 통합
- `cursor-mode`: Cursor Composer 스타일 에이전트
- `themes-nord` / `themes-dracula`: 테마 팩
- `lsp-extra`: 추가 언어 LSP

### 플러그인 API 보안

- **권한 명시**: manifest 의 `permissions` 필드에 선언
- **활성화 시 유저 동의**: 첫 활성화 시 권한 다이얼로그
- **Sandbox**: Rust plugin 은 WASM runtime 에서 실행 (stretch goal, v3.2)
- **Hook event 접근**: plugin 이 수신할 이벤트 화이트리스트

---

## 13. Onboarding 분기 (v3.1 업데이트)

첫 실행 플로우:

```
Launch
 → Welcome 화면
 → 환경 감지 (OS, 쉘, tmux, git, Node, Python, Go, Rust, etc.)
 → "Use MoAI ADK?" 선택
     ├─ Yes → moai-adk plugin 활성화 + 샘플 SPEC 워크스페이스 제안
     ├─ No  → 범용 cmux 모드 (moai-adk plugin 비활성)
     └─ Later → Settings 에서 언제든 전환
 → "첫 워크스페이스" 생성
     ├─ 기존 프로젝트 열기 (파일 picker)
     ├─ 샘플 프로젝트
     └─ 빈 터미널 (home directory)
 → Main Workspace 진입
```

---

버전: 3.1.0 · 2026-04-21 (+ Plugin architecture + moai-adk optional)
