# product.md — MoAI Studio

> **출처**: README.md, SPEC-V3-001/002/003 (현 기준), CLAUDE.local.md (Enhanced GitHub Flow)
> **상태**: SPEC-V3-003 functionally complete — 3 milestones × 29 AC GREEN
> **버전**: 0.0.x pre-release (v0.1.0 release/ 분기 candidate)
> **브랜드**: MoAI Studio (확정)
> **패키지 식별자**: `moai-studio`
> **작성일**: 2026-04-11 (v2 baseline) → 2026-04-25 (v3 Pivot 반영)

---

## 0. v3 Pivot Notice (2026-04-25)

본 문서는 **v2 Swift/macOS 단독** 설계 기준으로 작성되었으며, **v3 Rust/GPUI 크로스플랫폼** 피벗이 단계적으로 반영되는 중이다. v3 정합 섹션은 §0.x 와 §0.y 로 별도 표시한다. v2 Swift 관련 SPEC (SPEC-M0/M1/M2) 은 모두 ARCHIVED 마킹 완료.

### 0.1 v3 한 줄 정의

**MoAI Studio v3** 는 **moai-adk (Agentic Development Kit) 의 크로스플랫폼 (macOS / Linux / Windows) Agentic Coding IDE** 다.

moai-adk 는 Claude Code · Codex CLI 환경에서 동작하는 AI 하네스이며, MoAI Studio 는 그 위에 **터미널 · 마크다운/코드 뷰어 · 웹 브라우저 · 파일 탐색기 · git 관리 · SPEC 관리 · 에이전트 진행 상황** 통합 GUI 를 제공한다.

### 0.2 v3 정체성

| 항목 | v3 내용 |
|---|---|
| 카테고리 | 크로스플랫폼 GPU-가속 Agentic Coding IDE |
| 본체 | [modu-ai/moai-adk](https://github.com/modu-ai/moai-adk) (Go CLI) |
| 호스트 대상 | Claude Code CLI + Codex CLI + 향후 추가 agent harness |
| 플랫폼 | **macOS 14+ + Ubuntu 22.04+ + Windows (GPUI GA 후 활성)** |
| 언어 | Rust 1.93 (UI + Core) + Zig 0.15.2 (libghostty-vt FFI) |
| UI 프레임워크 | GPUI 0.2.2 (Zed) |
| 라이선스 | MIT |
| 릴리스 모델 | Enhanced GitHub Flow (main / release/* / develop / feature/*) — CLAUDE.local.md §1 |

### 0.3 v3 SPEC 진행 현황 (2026-04-25 기준)

| SPEC | 범위 | 상태 |
|------|------|------|
| SPEC-V3-001 | GPUI 스캐폴드 + Rust core 통합 | ✅ Complete (develop 머지) |
| SPEC-V3-002 | Terminal Core (libghostty-vt + PTY + Shell) | ✅ Complete (74 tests, 변경 금지 zone) |
| SPEC-V3-003 | Pane + Tab + Persistence (3 MS × 29 AC) | ✅ Functionally complete (411 ws tests / 0 fail) |

**남은 v0.1.0 release block**:
- GitHub Actions billing 해소 후 ci-v3-pane.yml 첫 GREEN run
- v0.1.0 release/ 분기 + tag

### 0.4 v3 다음 단계 (SPEC backlog)

비전 정합 우선순위 (사용자 결정 필요):

| 후보 SPEC | 비전 정합 영역 | 우선도 추천 |
|-----------|----------------|------------|
| **SPEC-V3-004 Render Layer Integration** | TabContainer ↔ GPUI 실제 렌더 (AC-P-4 deferred 해소) | P1 — 사용자에게 보이는 첫 번째 escape hatch |
| **SPEC-V3-005 File Explorer** | 파일 탐색기 (vision 핵심) | P1 — 비전 4대 surface 중 1 |
| **SPEC-V3-006 Markdown/Code Viewer** | 마크다운 + 코드 뷰어 + syntax highlighting | P1 — 비전 4대 surface 중 2 |
| **SPEC-V3-007 Embedded Web Browser** | WKWebView equivalent 크로스플랫폼 | P2 — 비전 4대 surface 중 3 |
| **SPEC-V3-008 Git Management UI** | git status / diff / commit / branch UI | P1 — 비전 핵심 영역 |
| **SPEC-V3-009 SPEC Management UI** | .moai/specs/ 시각화 + 상태 추적 | P1 — moai-adk 통합 핵심 |
| **SPEC-V3-010 Agent Progress Dashboard** | hook event stream + cost + instructions graph | P1 — agentic 본질 영역 |
| **SPEC-V3-011 Cross-platform Packaging** | dmg / deb / msi 자동 빌드 + 자동 업데이트 | P2 — release 도달 후 |

---

## 1. (v2 legacy) 한 줄 정의

**MoAI Studio** 는 **moai-adk 의 공식 macOS 네이티브 Agent IDE** 다.
Claude Code 를 subprocess 로 호스트하여 **27개 hook 이벤트 + 26 전문 에이전트 + TRUST 5 품질 게이트 + @MX 태그 시스템 + Kanban/SPEC 워크플로우** 를 한 화면에서 시각화 · 조작한다.

---

## 2. 정체성과 포지셔닝

| 항목 | 내용 |
|---|---|
| 제품명 (브랜드) | **MoAI Studio** |
| 패키지/바이너리 식별자 | `moai-studio` |
| 카테고리 | macOS 네이티브 Agent IDE / 에이전틱 코딩 종합 툴 |
| 본체 | [modu-ai/moai-adk](https://github.com/modu-ai/moai-adk) (Go CLI) |
| 호스트 대상 | Claude Code CLI (subprocess + stream-json) |
| 통합 패턴 | **공식 IDE MCP Server Pattern** (VS Code 확장과 동일) |
| 라이선스 | MIT |
| 플랫폼 | **macOS 14+ 영구 단독** (Apple Silicon + Intel) |
| 언어 | Swift (UI) + Rust (Core) |
| 앱 번들 | `MoAI Studio.app` |

### 한 문장 포지셔닝

> cmux 가 터미널 멀티화 + 브라우저 surface 를 잘 한다면, **MoAI Studio** 는 그 위에 **SPEC/TRUST/@MX/Kanban 워크플로우 시각화** 와 **Claude 가 직접 UI 를 조작할 수 있는 공식 IDE MCP Server Pattern** 을 올린다.

### 브랜드 선택 근거 (O6 RESOLVED)

DESIGN.v4 §14 O6 의 4개 후보 — `moai-cli`, `moai-ide`, `moai-shell`, `MoAI Studio` — 중 **MoAI Studio 를 확정**. 주요 근거:

1. **"Studio" = 업계 계보** (Visual Studio / Android Studio / RStudio) — "이것은 CLI 가 아니라 종합 작업 환경이다" 를 즉시 전달
2. **"cli" 철거** — 실제로는 10개 surface 를 가진 GUI 앱이므로 "cli" suffix 는 사실과 정면 충돌
3. **"shell" 철거** — Terminal surface 는 전체의 10% 에 불과
4. **MoAI 패밀리 역할 분리** — `moai-adk` (Go CLI, 엔진) vs **MoAI Studio** (macOS GUI, 플래그십) 의 경계가 명확
5. **Anthropic 브랜딩 제약 준수** — "Claude Code" 명칭과 0 충돌

### Linux/WSL/Windows 영구 폐기

B5 리서치 결과 (Ghostty WSL 미지원, GTK4 on WSLg 다수 버그, 9P 파일시스템 ~9배 느림, libghostty/VTE 백엔드 이중화 복잡도)에 근거해 **macOS 단독 전략을 영구 확정**. cmux 의 검증된 선택과 동일.

---

## 3. 타깃 사용자

| 페르소나 | 동기 |
|---|---|
| **moai-adk 사용자** | 14개 `/moai *` 슬래시 커맨드를 GUI 1-클릭으로 실행하고 싶음 |
| **Claude Code 파워 유저** | 27 hook 이벤트, cost, instructions 그래프를 시각적으로 디버깅하고 싶음 |
| **macOS 네이티브 IDE 선호자** | Electron 무거움 (Wave/Cursor) 대신 60fps@4K 네이티브 셸 원함 |
| **멀티 에이전트 운영자** | 16+ 워크스페이스 동시 운영 + 각각의 git worktree 격리 필요 |
| **SPEC-First 개발자** | Kanban 보드와 SPEC-{DOMAIN}-{NNN} 의 양방향 자동 연동 필요 |

---

## 4. 핵심 기능 (14개)

DESIGN.v4 §2.4 의 핵심 요구사항을 그대로 승계. **M2.5 이후 실제 동작 상태**:

1. **파일 탐색기** (FileTree surface) ✅ M2 — Rust `notify` + git status 색상
2. **GPU 가속 터미널** (libghostty + Metal 60fps@4K) ✅ M2.5 — 실제 GhosttyHost Metal 렌더링
3. **Code Viewer** (SwiftTreeSitter + LSP 진단 + @MX 거터 + tri-pane diff) 📅 M3
4. **마크다운 뷰어** (EARS SPEC 특화, KaTeX/Mermaid) ✅ M2 — KaTeX/Mermaid CDN
5. **내장 브라우저** (WKWebView + DevTools + dev 서버 자동 감지) ✅ M2
6. **이미지 뷰어** (diff 모드, SSIM, Vision framework) ✅ M2
7. **Agent Run Viewer** (hook event stream + cost tracking + live agent control) 📅 M5
8. **Kanban 보드** (SPEC ↔ worktree ↔ `/moai run` 자동 연동) 📅 M5
9. **다중 세션** (16+ 에이전트 동시, actor supervision) ✅ M1 — Tokio actor tree
10. **`/moai *` 14 슬래시 커맨드 GUI 1-클릭 호출** 📅 M4
11. **Memory Viewer** (`~/.claude/projects/<root>/memory/` 렌더) 📅 M5
12. **Instructions Loaded Graph** (세션 컨텍스트 디버거) 📅 M5
13. **In-app Claude UI 조작** (IDE MCP server 의 `mcp__moai__*` tools) 📅 M4
14. **Native Permission Dialog** (TUI text prompt 대체) 📅 M4

**M2.5 현재 완전 동작**: FileTree + 실제 터미널 Metal 렌더링 + Markdown 뷰어 + 이미지 뷰어 + 브라우저 + **Cmd+K Command Palette 모든 Surface 열기 커맨드**

---

## 5. 7가지 Moat (경쟁사 0)

DESIGN.v4 §13.1 기준. 단일 경쟁자도 동시 보유하지 못한 차별점.

1. **공식 IDE MCP Server Pattern 채택** — Anthropic 공식 wire format, 안정성 보장
2. **Hook 18~25 이벤트 양방향 + tool input rewriting** (`PreToolUse.updatedInput`)
3. **LSP as plugin feature** (`.lsp.json`) — 자체 LSP 클라이언트 구현 불필요, 6개 언어 무료
4. **Kanban + Memory + InstructionsGraph 3종 surface**
5. **@MX 태그 거터 + TRUST 5 게이지** (moai-adk 독점)
6. **In-app Claude UI 조작** (`mcp__moai__*` 14+ tools — Claude 가 UI 운전)
7. **Native permission dialog + `updatedPermissions` 영구화**

### 경쟁사 매트릭스 (요약)

| 축 | cmux | Warp | Wave | Zed | Ghostty | **MoAI Studio** |
|---|---|---|---|---|---|---|
| Claude 통합 | teammate shim | cloud agents | badge | ACP (hook X) | 없음 | **공식 IDE MCP Server** |
| Hook 노출 | OSC 만 | ❌ | badge | ❌ | ❌ | **18~25 양방향** |
| Kanban/SPEC | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| Memory Viewer | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| @MX + TRUST 5 | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| Native permission dialog | TUI | TUI | TUI | 부분 | N/A | **SwiftUI 모달** |
| 라이선스 | GPL-3.0 | 폐쇄 | Apache | 혼합 | MIT | **MIT** |

---

## 6. 사용 시나리오

### 시나리오 1 — SPEC 기반 개발 워크플로우

1. 형님이 Kanban 보드의 Backlog 카드를 **Doing** 으로 드래그
2. MoAI Studio 가 자동으로:
   - `git worktree add` 로 워크스페이스 격리
   - Claude subprocess spawn (`claude --bare -p --output-format stream-json ...`)
   - `SDKUserMessage` 로 `/moai run SPEC-AUTH-042` 전송
   - `surface.reveal({surface: "agent_run"})` → 3-pane 자동 구성
3. Agent Run Viewer 에서 hook event stream 실시간 관찰
4. 완료 시 카드를 **Review** 로 → `git diff main..HEAD` Markdown surface 자동 표시
5. **Done** 으로 → `gh pr create`

### 시나리오 2 — Claude 가 UI 를 직접 조작

```
User: "SPEC-AUTH-042 를 Doing 으로 옮기고 시작해줘"
Claude:
  1. mcp__moai__kanban_move_card({id: 42, to: "doing"})
     → MoAI Studio UI 가 즉시 카드 이동 애니메이션
  2. mcp__moai__workspace_create({spec_id: "SPEC-AUTH-042"})
     → git worktree add + Claude subprocess spawn
  3. mcp__moai__surface_reveal({surface: "agent_run"})
     → 3-pane 레이아웃 자동 구성
```

### 시나리오 3 — Hook 으로 Bash 명령 안전화

`PreToolUse` 훅이 `rm <path>` 를 자동으로 `trash <path>` 로 rewrite. `hookSpecificOutput.updatedInput` 으로 Claude 의 실제 실행을 변경.

### 시나리오 4 — Instructions Graph 디버깅

세션이 의도와 다르게 동작할 때 InstructionsGraph surface 를 열어 어떤 CLAUDE.md / skill / memory 가 어떤 이유로 (`session_start`, `nested_traversal`, `path_glob_match`, `include`, `compact`) 로드되었는지 한 눈에 파악. 클릭 시 해당 파일을 Code Viewer 로 점프.

---

## 7. 비기능 요구사항

DESIGN.v4 §2.5 기준.

| 항목 | 목표 |
|---|---|
| 콜드 스타트 (M1 MacBook) | < 0.6s |
| 활성 메모리 (8 PTY + 4 code surface + 2 Claude subprocess) | < 700 MB |
| 터미널 스크롤 | 60 fps @ 4K (libghostty Metal) |
| 동시 에이전트 세션 | 16+ |
| Hook callback latency (http loopback) | < 10ms P95 |
| IDE MCP tool → UI 업데이트 | < 30ms |
| 세션 복원 | < 2s |
| Rust core ↔ Swift UI FFI overhead | < 1ms per call |
| 크래시 격리 | 1개 에이전트 크래시가 나머지에 전파되지 않음 |

---

## 8. 브랜딩 제약 (Anthropic 공식)

출처: [Claude Agent SDK overview](https://code.claude.com/docs/en/agent-sdk/overview)

### 허용

- ✅ "MoAI Studio", "MoAI Agent IDE", "moai + Claude", "Powered by Claude"
- ✅ "A native macOS Agent IDE for Claude-powered development"
- ✅ 인증: `ANTHROPIC_API_KEY`, Bedrock, Vertex, Foundry

### 금지

- ❌ "Claude Code" 명칭 사용
- ❌ "Claude Code Agent" 명칭
- ❌ Claude Code ASCII art 차용
- ❌ claude.ai OAuth 로그인 구현

---

## 9. 현재 상태와 로드맵

### 현재 상태

**M2.5 Complete (Full GO).** 12 Rust crates (289 tests) + SwiftUI app (130 tests) = 419 tests PASS. M2 placeholder 4건 해소.

산출물 현황:

| 마일스톤 | 상태 | 스프린트 | 산출물 | 품질 |
|---------|------|--------|--------|------|
| **M0** | ✅ 완료 | Pre-M0 + D1~D2 | Rust core skeleton + Swift shell | 50 tests |
| **M1 (Conditional GO)** | ✅ 완료 | T-020~T-030 | Working Shell: Sidebar + Workspace + Pane + Surface DAO | 106 tests |
| **M2 (Conditional GO v1.2.0)** | ✅ 완료 | MS-1~MS-7 (T-031~T-087) | 5 Viewers + NSSplitView + TabUI + Command Palette + CI/CD | **339 tests** |
| **M2.5 (Full GO)** | ✅ 완료 | T-M2.5-001~T-M2.5-018 | ActivePaneProvider + GhosttyHost 실연결 + Command Palette 동작 | **419 tests** |
| M3 | 📅 예정 | — | Code Viewer (SwiftTreeSitter + LSP + @MX) | — |

4개 설계 문서:

| 파일 | 버전 | 상태 |
|---|---|---|
| DESIGN.md | v2 | 참고용 (Bridge / hooks.yaml 가정 오류) |
| DESIGN.v3.md | v3 | 참고용 ("SDK 임베드" 가정, Pure Swift 제안) |
| **DESIGN.v4.md** | **v4** | **★ 현 기준** (subprocess + IDE MCP + Rust core) |
| NEXT-STEPS.md | v1 | 현 M2 단계부터 시작 |

### 로드맵 (M2 완료 후)

| 단계 | 기간 | 산출물 | 상태 |
|---|---|---|---|
| ~~Pre-M0 spike~~ | ~~3-4일~~ | ~~Claude CLI / IDE MCP / http hook / GhosttyKit 검증~~ | ✅ |
| ~~M0 킥오프~~ | ~~2주~~ | ~~Rust core skeleton + Swift UI shell + 첫 hook 왕복~~ | ✅ |
| ~~M1 Core Sessions~~ | ~~3주~~ | ~~Workspace/Pane/Surface, NSSplitView, store v1, 18 이벤트~~ | ✅ |
| ~~M2 Viewers~~ | ~~3주~~ | ~~FileTree, Markdown, Image, Browser~~ | ✅ |
| **M3 Code Viewer** | **3주** | **SwiftTreeSitter, LSP, @MX 거터, tri-pane diff** | **📅 다음** |
| M4 Claude 통합 심화 | 3주 | Plugin 자동 설치, Native permission dialog, LSP 6 언어 | 📅 |
| M5 Agent Run + Kanban + Memory | 3주 | Agent Run Viewer, Kanban, Memory, Instructions Graph | 📅 |
| M6 안정화 + 배포 | 2주 | Sparkle, notarize, 16-agent stress, DMG | 📅 |

### M2 완료 상태 요약

- **구현 코드**: Rust 1,070 LOC + Swift 3,300 LOC = 4,370 LOC
- **테스트**: Rust 233/233 + Swift 106/106 = **339/339 PASS**
- **@MX 태그**: 28개 (ANCHOR 11, WARN 3, NOTE 14)
- **Carry-over**: 8건 (4 완료, 2 부분, 2 opt-in)
- **GitHub Actions**: ci-rust.yml + ci-swift.yml 구성
- **CI/CD**: Rust + Swift 병렬 빌드, 모든 tool 자동 검사

---

## 10. 관련 저장소

- [modu-ai/moai-adk](https://github.com/modu-ai/moai-adk) — Go CLI, MoAI Studio 가 통합하는 본체
- [ghostty-org/ghostty](https://github.com/ghostty-org/ghostty) — 터미널 엔진 (libghostty)
- [anthropics/claude-code](https://github.com/anthropics/claude-code) — Claude Code CLI

---

**Source of truth**: DESIGN.v4.md · README.md · NEXT-STEPS.md
**Supersedes**: DESIGN.md (v2), DESIGN.v3.md (v3)
