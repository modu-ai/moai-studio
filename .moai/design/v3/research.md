# MoAI Studio v3.0 Research Synthesis (2026-04-21)

**Pivot**: v1/v2 는 "agentic IDE" 로 설계. v3 는 **"smart terminal-first multiplexer (cmux+) with moai-adk GUI overlay + multi-project workspaces"** 로 재정의.

---

## 1. 결정적 발견

### 1.1 cmux = MoAI Studio 의 직접 참조 제품

cmux 는 2026-02 Manaflow 가 출시한 **native macOS 터미널 멀티플렉서**. MoAI Studio 와 **스택이 정확히 일치**:

| 요소 | cmux | MoAI Studio |
|------|------|-------------|
| 언어 | Swift + AppKit | Swift + SwiftUI |
| 터미널 렌더러 | LibGhostty | Ghostty xcframework |
| 레이아웃 | BondSplit | NSSplitView binary tree |
| 브라우저 | WebKit | WKWebView |
| IPC | Unix Domain Socket (JSON) | swift-bridge FFI + Rust core |
| Agent env | CMUX_WORKSPACE_ID / SURFACE_ID / SOCKET_PATH | (신규 필요: MOAI_WORKSPACE_ID / PANE_ID / SOCKET_PATH) |
| 단축키 | Native macOS (prefix 불필요) | Native macOS (동일) |

**cmux 로부터 계승할 패턴**:
- ✅ Vertical tabs (사이드바 = workspaces list)
- ✅ Panes and splits (binary tree)
- ✅ Tabs within panes
- ✅ No config files, no prefix keys
- ✅ Unix socket IPC (moai CLI ↔ GUI app)
- ✅ Agent context env vars (worktree 격리와 결합)
- ✅ "Primitive, Not Solution" 철학 — 저수준 빌딩 블록 + 조합 자유도

**cmux 로부터 차별화**:
- ❌ **cmux 는 graphical project management 없음** → MoAI Studio 는 SPEC 카드/Kanban/Dashboard 로 우위
- ❌ **cmux 는 drag-and-drop pane arrangement 없음** → MoAI Studio 는 지원
- ❌ **cmux 는 visual session overview 없음** → MoAI Studio 는 Mission Control
- ❌ **cmux 는 learning curve 큼** (docs + 키바인딩 암기) → MoAI Studio 는 Command Palette + 메뉴 바로 발견성 ↑
- ❌ **cmux 는 moai-adk 특화 없음** → MoAI Studio 는 SPEC/Agent Run/TRUST 5 GUI overlay

Endorsement: Mitchell Hashimoto (Ghostty), Nick Schrock (Dagster), Edward Grefenstette (DeepMind). 커뮤니티 검증됨.

### 1.2 Claude Code Desktop 2026-04-14 재출시 = 직접 경쟁

**주요 기능**:
- Multi-session sidebar (parallel agents)
- Drag-and-drop workspace layout
- Integrated terminal (Ctrl+`) — session 의 working directory + env 공유
- Integrated file editor + diff viewer + preview pane
- Side chats (Cmd+;) — branch conversation without polluting main
- Environment: Local / Remote / SSH
- View modes: Verbose / Normal / Summary
- Plugins + Skills + Connectors parity with CLI
- Pane types: **chat / diff / preview / terminal / file / plan / tasks / subagent**
- Click file path → file pane; HTML/PDF/image → preview pane

**Claude Code Desktop 의 약점 (MoAI Studio 차별화 기회)**:
- 🔥 **Integrated terminal 이 notable latency** 로 비판받음 → "overkill layer" 피드백
- 🔥 **chat-centric**: 터미널은 부속. 파워유저는 네이티브 터미널 선호
- 🔥 **moai-adk 특화 없음** (범용 Claude Code CLI 용)
- 🔥 **Electron 기반 추정** (parity move for Codex 비교 언급)

**MoAI Studio 포지셔닝**:
- **Terminal-first**: Ghostty Metal 60Hz → latency 문제 없음
- **moai-adk 특화**: SPEC 카드, TRUST 5 대시보드, @MX 거터
- **Native Swift/AppKit**: Electron 아님
- **Multi-project workspaces**: VS Code 스타일 독립 프로젝트 전환

### 1.3 Warp Terminal 의 Block Model

- Block-based navigation: 명령+출력을 단위로 묶어 스크롤/검색
- Terminal mode vs Agent mode (2 modes)
- Active AI recommendations (proactive)
- IDE-like input field
- 700K+ developers

**MoAI Studio 가 빌릴 패턴**:
- Block-based output grouping (명령 실행 단위로 시각적 구분)
- Active recommendations (에러 출력 감지 → `/moai fix` 제안)
- Mode toggle (Terminal ↔ Agent)

### 1.4 Wave Terminal = inline preview 선도자

**핵심 기능 (MoAI Studio 가 반드시 계승)**:
- **Inline rendering**: images, Markdown, JSON, CSV, audio/video
- **Drag & drop blocks**: 터미널/에디터/브라우저/AI 자유 배치
- **VSCode-powered editor** (Monaco engine)
- **Context-aware AI**: 터미널 출력 읽어 제안
- **BYOK**: OpenAI/Claude/Gemini/Azure/Ollama/LM Studio
- Open source, Apache-2.0

**MoAI Studio 의 Wave 대비 차별화**:
- Wave 는 범용 → MoAI Studio 는 moai-adk 특화 (SPEC-EARS 렌더링, @MX 시각화, TRUST 5)
- Wave 는 에디터에 Monaco → MoAI Studio 는 SwiftTreeSitter (네이티브, 빠름)

### 1.5 OSC 8 Hyperlinks = 표준 기술 기반

```
\033]8;;file:///absolute/path/file.ts:42\033\\file.ts:42\033]8;;\033\\
```

- 터미널 이스케이프 코드로 파일 경로/URL 을 클릭 가능 하이퍼링크화
- 지원: Ghostty ✅, iTerm2 ✅, Windows Terminal ✅, WezTerm ✅, GNOME Terminal ✅
- 미지원 터미널은 그냥 무시 — **downside 없음**
- CLI 도구 지원: `ls --hyperlink`, `eza`, `fd`, `bat`, `delta`, `rg --hyperlink-format`

**MoAI Studio 구현 전략 (3-layer)**:
1. **OSC 8 primary**: moai-adk CLI 출력을 OSC 8 로 래핑 (SPEC 문서, 파일 경로, 로그 경로)
2. **Regex secondary**: 레거시 CLI 출력에서 파일 경로/URL 정규식 매칭
3. **Custom parsers**:
   - `SPEC-[A-Z0-9]+-\d+` → SPEC 카드 오픈
   - `@MX:(NOTE|WARN|ANCHOR|TODO)` → 해당 코드 위치 점프
   - ` ```mermaid ` fence → Mermaid renderer
   - Hook event JSON (Agent Run Viewer 스트림)

Reference spec: [egmontkob/eb114294efbcd5adb1944c9f3cb5feda](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda)

### 1.6 Multi-Root Workspace = VS Code 모델 채택

**VS Code `.code-workspace` JSON**:
- `folders: [{path, name}]` 배열
- 폴더별 `.vscode/` 설정 가능
- 글로벌 검색 across folders
- 상대 경로로 공유 가능

**JetBrains 플러그인 방식 (heavier)**:
- `jb-workspace.xml`
- Build tool 자동 탐지
- 플러그인 필요

**MoAI Studio 채택 모델** (VS Code 유사, MoAI 확장):
```json
{
  "$schema": "moai-studio/workspace-v1",
  "workspaces": [
    {
      "id": "uuid",
      "name": "moai-studio",
      "projectPath": "/Users/goos/MoAI/moai-studio",
      "moaiConfig": ".moai/",
      "color": "#FF6A3D",
      "lastActive": "2026-04-21T11:40:00Z"
    },
    {
      "id": "uuid",
      "name": "moai-adk-go",
      "projectPath": "/Users/goos/MoAI/moai-adk-go",
      "moaiConfig": ".moai/",
      "color": "#3D9FFF",
      "lastActive": "2026-04-20T18:00:00Z"
    }
  ]
}
```

- 저장 위치: `~/.moai/studio/workspaces.json` (또는 SQLite)
- 1 MoAI Studio 앱 = N 프로젝트 동시 로드
- 사이드바에서 프로젝트 전환 (cmux vertical tabs 패턴)
- 각 프로젝트가 자체 pane tree + tab state + SPEC 보유

---

## 2. 경쟁 매트릭스

| 기능 | cmux | Claude Code Desktop | Warp | Wave | MoAI Studio (v3) |
|------|------|---------------------|------|------|------------------|
| Native macOS (Swift/AppKit) | ✅ | ❌ (Electron 추정) | ❌ | ❌ (Electron) | ✅ |
| GPU terminal renderer | ✅ Ghostty | 내장 | ✅ | - | ✅ Ghostty Metal 60Hz |
| Multi-pane / splits | ✅ | ✅ drag-drop | ✅ | ✅ drag-drop | ✅ binary tree |
| Inline preview (MD/image) | ❌ | ✅ preview pane | ❌ | ✅ native blocks | ✅ 전 Surface |
| OSC 8 hyperlinks | - | - | - | - | ✅ (신규) |
| File click → rich surface | ❌ | ✅ (chat만) | ❌ | ✅ | ✅ (터미널 ⭐) |
| Multi-project workspace | ❌ | ✅ sidebar | - | ✅ | ✅ VS Code 스타일 |
| Parallel agents | ✅ | ✅ Multi-session | ✅ Agent mode | ✅ | ✅ Mission Control |
| SPEC / DDD workflow | ❌ | ❌ | ❌ | ❌ | ⭐ 독점 |
| @MX 코드 주석 | ❌ | ❌ | ❌ | ❌ | ⭐ 독점 |
| TRUST 5 대시보드 | ❌ | ❌ | ❌ | ❌ | ⭐ 독점 |
| Hook 이벤트 27종 스트림 | ❌ | 부분 | - | - | ⭐ 독점 |
| Unix socket IPC | ✅ | - | - | - | ✅ (신규) |
| CG Mode (Claude+GLM) | ❌ | ❌ | ❌ | ❌ | ⭐ 독점 |

⭐ = MoAI Studio 차별화 포인트.

---

## 3. 핵심 UX 패턴 (v3 반영)

### P-1. Terminal-first layout
**cmux + Wave 모델**. 사이드바 = workspaces list. 중앙 = pane tree (기본 터미널). 우측 = Agent Run Viewer.
- 기본 실행 시 터미널 Surface.
- Code Viewer / Markdown / Browser 등은 **터미널 클릭 또는 Command Palette 로만 열림**.
- Surface 변환: 같은 pane 에서 Surface 교체 (cmux 와 같음) OR 새 pane 분할.

### P-2. Smart link handling (MoAI Studio 의 core differentiator)
**OSC 8 + regex + custom parser 3-layer**.
- 터미널 출력 `src/app.swift:42` 클릭 → Code Viewer Surface 새 탭 오픈 (줄 42 로 점프)
- `https://example.com` 클릭 → Browser Surface
- `SPEC-M2-002` 클릭 → SPEC 카드 모달
- `@MX:ANCHOR` 클릭 → 해당 코드 위치 점프
- Mermaid 코드 블록 → Mermaid Renderer Surface
- Hover preview: 파일 경로 hover 시 작은 preview popup

### P-3. Multi-project workspace
**VS Code 모델 + MoAI 확장**.
- 1 MoAI Studio 앱 = N 프로젝트
- 사이드바 상단: Workspace 스위처 (드롭다운)
- 각 workspace = 독립 pane tree + tab state + SPEC list
- 프로젝트 전환 시 상태 보존 (cmux 처럼 persistent)
- 글로벌 검색 across workspaces
- Workspace 별 색상 태그 (시각 구분)

### P-4. moai-adk GUI overlay
**Right panel 또는 overlay 로 표시** (터미널 방해 없음):
- SPEC 카드 (현재 활성 SPEC)
- TRUST 5 게이지
- @MX 태그 카운트
- Agent Run 진행 상태
- Hook event 스트림
- Kanban / Dashboard / Memory 는 별도 모드

### P-5. Block-based output (Warp 모델)
- 터미널 출력을 명령 단위 블록으로 그룹화
- 블록 클릭 → 명령 재실행 / 복사 / 공유
- 에러 블록 감지 → `/moai fix` 제안 (Active AI Recommendations)

### P-6. Progressive agent visibility
- 기본 상태바에 agent pill
- 실행 중 우측 패널 자동 오픈
- Mission Control 로 전체 일람
- Follow agent 토글

---

## 4. 참고 문헌

### cmux
- [cmux — The terminal built for multitasking](https://cmux.com)
- [cmux vs tmux — Agent Terminal vs Terminal Multiplexer (2026)](https://soloterm.com/cmux-vs-tmux)
- [CMUX Complete Guide](https://agmazon.com/blog/articles/technology/202603/cmux-terminal-ai-guide-en.html)
- [cmux: Native macOS Terminal for AI Coding Agents | Better Stack](https://betterstack.com/community/guides/ai/cmux-terminal/)
- [The Rise of AI Terminal Multiplexers | Beam](https://getbeam.dev/blog/ai-terminal-multiplexers-compared-2026.html)

### Claude Code Desktop
- [Use Claude Code Desktop — Docs](https://code.claude.com/docs/en/desktop)
- [Anthropic Rebuilds Claude Code Desktop App Around Parallel Sessions — MacRumors](https://www.macrumors.com/2026/04/15/anthropic-rebuilds-claude-code-desktop-app/)
- [Claude Code Desktop Redesign — VentureBeat](https://venturebeat.com/orchestration/we-tested-anthropics-redesigned-claude-code-desktop-app-and-routines-heres-what-enterprises-should-know)
- [Claude Code Desktop Redesign — buildfastwithai](https://www.buildfastwithai.com/blogs/claude-code-desktop-redesign-2026)
- [Redesigning Claude Code on desktop for parallel agents — Anthropic](https://claude.com/blog/claude-code-desktop-redesign)
- [Claude Code Desktop latency critique — The New Stack](https://thenewstack.io/claude-code-desktop-redesign/)

### Warp Terminal
- [Terminal and Agent modes — Warp Docs](https://docs.warp.dev/agent-platform/warp-agents/interacting-with-agents/terminal-and-agent-modes)
- [Warp AI Terminal 2026 Agentic CLI Workflows](https://www.digitalapplied.com/blog/warp-ai-terminal-agentic-cli-workflows-guide)
- [Warp Terminal — warp.dev](https://www.warp.dev/)

### Wave Terminal
- [Wave Terminal — waveterm.dev](https://www.waveterm.dev/)
- [Wave Terminal GitHub](https://github.com/wavetermdev/waveterm)
- [Wave: A Modern New Linux Terminal — itsfoss](https://itsfoss.com/news/wave-terminal/)

### Tabby
- [Tabby — a terminal for a more modern age](https://tabby.sh/)

### OSC 8
- [Hyperlinks in Terminal Emulators — egmontkob gist](https://gist.github.com/egmontkob/eb114294efbcd5adb1944c9f3cb5feda)
- [OSC8-Adoption list](https://github.com/Alhadis/OSC8-Adoption/)
- [xterm.js Link Handling](https://xtermjs.org/docs/guides/link-handling/)
- [Claude Code Feature Request: OSC 8 Hyperlinks](https://github.com/anthropics/claude-code/issues/13008)
- [WezTerm Hyperlinks](https://wezterm.org/recipes/hyperlinks.html)

### Multi-Root Workspace
- [What is a VS Code workspace?](https://code.visualstudio.com/docs/editing/workspaces/workspaces)
- [Multi-root Workspaces — VS Code Docs](https://code.visualstudio.com/docs/editing/workspaces/multi-root-workspaces)
- [IntelliJ Workspaces — JetBrains](https://www.jetbrains.com/help/idea/workspaces.html)

---

버전: 3.0.0 · 2026-04-21
