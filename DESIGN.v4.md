# MoAI Studio 설계 문서 (v4)

> **정체성**: moai-adk 의 공식 macOS 네이티브 IDE-쉘. Claude Code 를 **subprocess 로 spawn + stream-json 프로토콜 제어 + IDE MCP Server Pattern** 으로 통합한다.
>
> **작성일**: 2026-04-11 (v4 — 공식 Anthropic 문서 검증 + IDE MCP Server Pattern 채택 + Rust core 재도입 + **macOS 영구 단독** 확정)
>
> **플랫폼**: **macOS only (forever).** Linux/WSL/Windows 지원은 영구 폐기. 근거는 §1.3.
>
> **근거 자료** (동일 폴더):
> - `research/R1-native-ai-shells.md` — 경쟁사 리서치 (Warp/Wave/cmux/Zed/Ghostty)
> - `research/B1-bridge-direct-connect.md` — Claude Code Bridge/Remote 소스 분석
> - `research/B2-hook-events-tool-system.md` — 27 hook + tool system 소스 분석
> - `research/B3-extension-points.md` — Plugin/MCP/Skill 소스 분석
> - **`research/B4-official-docs-verification.md`** — docs.anthropic.com / code.claude.com 공식 문서 검증
> - **`research/B5-wsl-wslg-windows-coverage.md`** — WSL/WSLg 현실성 조사 (참고용 — macOS 단독 결정의 근거)
>
> **이전 버전**:
> - `DESIGN.md` (v2, 참고용 유지)
> - `DESIGN.v3.md` (v3, 참고용 유지)
> - **`DESIGN.v4.md` (v4, 현 기준)**

---

## 0. 3가지 핵심 피벗 (v3 → v4)

### 피벗 1: **"임베드" 가 아니라 "subprocess 호스트" 다**

v3 는 "Claude Code 를 SDK 라이브러리로 임베드" 라고 표현했다. 공식 문서 ([hosting](https://code.claude.com/docs/en/agent-sdk/hosting)) 재검증 결과:

> "Node.js (required by the bundled Claude Code CLI that the SDK spawns; both SDK packages include it, so no separate install is needed)."

**Claude Agent SDK 는 Python + TypeScript 2개 언어만** 공식 지원한다. 그리고 이 SDK 조차 내부에서 `claude` CLI 를 subprocess 로 spawn 한다. **어떤 언어에서도 Claude Code 를 "라이브러리로 링크" 하는 경로는 존재하지 않는다.**

공식으로 지원되는 유일한 Rust/Swift 통합 경로:

```bash
claude --bare -p "" \
  --output-format stream-json \
  --include-partial-messages \
  --settings <path> \
  --mcp-config <path> \
  --agents <json> \
  --tools "Read,Edit,Bash,mcp__moai__*" \
  --permission-mode acceptEdits

> [SPIKE ERRATA] `--allowedTools` 는 additive permission list 로 도구 집합을 제한하지 않는다. 실제 도구 집합 제한에는 `--tools` 를 사용해야 한다.
```

`--bare` 는 공식 권장 플래그이며 향후 `-p` 의 기본값이 될 예정 ([headless](https://code.claude.com/docs/en/headless)):

> "`--bare` is the recommended mode for scripted and SDK calls, and will become the default for `-p` in a future release."

`--bare` 가 비활성화하는 것들:
- `.claude/settings.json` 자동 로드 → `--settings` 로 명시
- Plugins 자동 로드 → `--plugin-dir` 로 명시
- MCP 서버 자동 로드 → `--mcp-config` 로 명시
- CLAUDE.md 자동 로드 → `--append-system-prompt-file` 로 명시
- Keychain / OAuth 읽기 → 환경 변수로 API key 전달

> [SPIKE ERRATA] `--bare` 는 OAuth/Keychain 인증을 완전히 비활성화한다. 따라서 MoAI Studio 가 Claude subprocess 를 `--bare` 모드로 spawn 할 때 반드시 `ANTHROPIC_API_KEY` 환경 변수를 subprocess 환경에 설정해야 한다. 미설정 시 인증 실패.

이로써 MoAI Studio 가 **결정론적 실행 환경**을 보장한다.

**v4 용어:**
- v3: "SDK host / embed SDK / initialize.hooks 로 27 callback 등록"
- **v4: "Subprocess host / stream-json 프로토콜 / IDE MCP Server Pattern 으로 양방향 통합"**

### 피벗 2: **`--mcp-config` SSE/Streamable HTTP Pattern 이 PRIMARY 통합 경로**

> [SPIKE ERRATA] 2026-04-11: lockfile + WS 패턴은 커스텀 도구 노출에 부적합함이 Pre-M0 spike 에서 확인됨. lockfile + WS 는 IDE 코드 인텔리전스 (VS Code/Zed 스타일) 에만 적합하며, `mcp__moai__*` 커스텀 도구를 Claude 에 노출하려면 `claude --mcp-config <config.json>` 으로 SSE/Streamable HTTP transport 를 사용해야 한다.

**PRIMARY 경로**: `claude --mcp-config <config.json>` 에 MoAI Studio 의 SSE/Streamable HTTP MCP 서버를 등록. Claude 가 시작 시 해당 config 를 읽어 MoAI Studio MCP 서버에 연결하고 `mcp__moai__*` 도구를 인식한다.

**보조 통합 (IDE 코드 인텔리전스)**: 기존 lockfile + WS 패턴은 `mcp__ide__getDiagnostics` 등 VS Code 호환 표준 도구 노출용으로 유지한다. 공식 문서 ([vs-code](https://code.claude.com/docs/en/vs-code)):

> "When the extension is active, it runs a **local MCP server** that the CLI connects to automatically... The server binds to `127.0.0.1` on a random high port and is not reachable from other machines. Each extension activation generates a fresh random auth token that the CLI must present to connect. The token is written to a lock file under `~/.claude/ide/` with `0600` permissions in a `0700` directory."

```
┌──────────────────────────────────────────────────────┐
│         MoAI Studio 호스트 프로세스 (macOS)              │
│                                                      │
│  ┌──────────────┐    ┌────────────────────────────┐  │
│  │  Swift UI    │    │   IDE MCP Server            │  │
│  │  (SwiftUI +  │◄──►│   127.0.0.1:<random>        │  │
│  │   AppKit)    │    │   auth token + lockfile     │  │
│  └──────┬───────┘    │   ~/.claude/ide/<port>.lock │  │
│         │            └──────────┬──────────────────┘  │
│         │ swift-bridge          │                     │
│  ┌──────▼────────────────────────▼───────────────┐   │
│  │           Rust Core (moai-core)                │   │
│  │           ───────────────────                  │   │
│  │  • RootSupervisor (Tokio actor tree)           │   │
│  │  • WorkspaceSupervisor × N                     │   │
│  │  • ClaudeSubprocessManager                     │   │
│  │  • StreamJsonCodec (SDKMessage)                │   │
│  │  • IdeMcpServer (axum + rmcp)                  │   │
│  │  • HookHttpEndpoint                            │   │
│  │  • Store (rusqlite WAL)                        │   │
│  │  • Git (git2)                                  │   │
│  │  • FsWatcher (notify)                          │   │
│  │  • PluginInstaller                             │   │
│  │  • LockfileDaemon                              │   │
│  └────────┬───────────────────────┬───────────────┘   │
│           │ stdin/stdout           │                   │
└───────────┼───────────────────────┼───────────────────┘
            │                        │
            ▼                        ▼
   ┌────────────────────┐   ┌──────────────────────┐
   │  claude subprocess  │   │  Plugin http hooks     │
   │  (per workspace)    │   │  POST 127.0.0.1/...    │
   │  --bare -p          │──►│  X-Auth-Token          │
   │  --mcp-config ...   │   │  → Rust hook_http      │
   └────────────────────┘   └──────────────────────┘
```

MoAI Studio 는 VS Code 확장과 **정확히 같은 패턴**으로 Claude Code 와 대화한다. wire format (MCP + JSON) 은 공개 표준이므로 안정적이다.

### 피벗 3: **Rust core + Swift UI (macOS 영구 단독)**

v3 는 "Pure Swift" 를 제안했지만, 형님의 "Rust 가 더 좋지 않나" 질문 + Rust 의 구체적 장점 + 공식 경로에서 Rust/Swift 둘 다 같은 subprocess pattern 을 쓴다는 사실을 종합해 **Rust core 재도입**을 확정한다.

**Rust core 를 쓰는 이유 (macOS 단독임에도):**

1. **Actor supervision model**: Tokio 의 actor + structured concurrency 가 16+ workspace 동시 운용에 Swift actor 보다 성숙함. Swift actor 의 reentrancy edge case 회피.
2. **Stream-json parser 성능**: 16 workspace × 수십 이벤트/초 를 파싱하는데 Rust 의 zero-cost serde 가 최적.
3. **rusqlite + r2d2 connection pooling**: 고부하 쓰기 성능이 GRDB.swift 보다 입증됨.
4. **Memory safety with no GC**: ARC 의 cycle/retain 문제 없음. Core 로직이 SwiftUI 의 update cycle 과 분리.
5. **tokio 의 network/process/fs async 통합**: Swift 의 URLSession 이나 Process 보다 유연.
6. **FFI 부담은 제한적**: Core 는 Rust, UI 는 Swift, 경계는 `swift-bridge`. 하루 30분 configure 면 끝.
7. **CLI reuse 가능성**: Rust core 는 나중에 CLI 버전 (`moai-studio-headless`) 으로 재사용 가능. Swift UI 없이 Rust 만으로 동작.
8. **형님 선호**: 이미 표명된 선호. 생산성 증대 요인.

**Pure Swift 를 쓰지 않는 이유:**
- SwiftUI 의 특정 surface (Code Viewer, Agent Run) 에서 60fps 유지가 불확실
- Swift actor reentrancy 이슈 회피
- Claude stream-json 파싱의 이론적 최대 성능

**대안 (단, 권장 안 함):** Pure Swift. M0 spike 에서 Rust core 복잡도가 너무 크면 fallback 가능. 그 경우 M0 일정 1주 단축.

### Linux/WSL/Windows 포기 결정 (§1.3 상세)

v4 이전 draft 에서는 Linux + WSL Tier 0 을 고려했으나, B5 리서치 결과 **포기**한다. 간단히:

- Ghostty 가 [WSL 을 공식 미지원](https://github.com/ghostty-org/ghostty/discussions/2563)
- GTK4 on WSLg 는 [다수 open bugs](https://github.com/microsoft/wslg/issues/754) (resize 불가, popup 이슈, empty windows)
- WSL2 의 9P 파일 시스템은 ~9x 느림
- VTE 로 대체하면 macOS 의 libghostty 와 다른 터미널 백엔드 2개를 유지해야 함 (복잡도 2배)
- Windows 네이티브 지원은 별도 3-6개월 포팅 필요

**결론**: macOS 만 정말 잘 만드는 것이 현명하다. cmux 의 검증된 선택과 동일.

---

## 1. v4 대비 v3 변경 요약

### 1.1 공식 문서 검증으로 정정된 것

| v3 가정 | v4 진실 | 근거 |
|---|---|---|
| "SDK 라이브러리 임베드" | **subprocess spawn + stream-json** | [hosting](https://code.claude.com/docs/en/agent-sdk/hosting) |
| `initialize.hooks` in-process callback | ❌ Python/TS 전용. Rust/Swift 불가 | 공식 언어 지원 목록 |
| In-process MCP `type: 'sdk'` | ❌ 동일 (Python/TS 전용) | [MCP docs](https://code.claude.com/docs/en/agent-sdk/mcp) |
| `claude server` / `cc+unix://` Direct Connect | ❌ **완전 삭제** (공식 미문서화) | [llms.txt](https://code.claude.com/docs/llms.txt) |
| IDE Lockfile = "fallback" | **보조 통합 (IDE 코드 인텔리전스)** 으로 유지, PRIMARY 는 `--mcp-config` SSE/Streamable HTTP | [vs-code](https://code.claude.com/docs/en/vs-code), spike-report.md |
| Hook 27 이벤트 모두 활용 | **18~25개 공식 + 2개 내부 (StatusLine, FileSuggestion)** | [hooks](https://code.claude.com/docs/en/agent-sdk/hooks), [plugins-reference](https://code.claude.com/docs/en/plugins-reference) |
| `updatedInput`, `permissionDecision` | ✅ 공식 문서화 확인 | [hooks](https://code.claude.com/docs/en/agent-sdk/hooks) |
| `updatedPermissions` | ⚠️ 공식 미문서화 — feature flag 로 wrap | — |
| `watchPaths`, `updatedMCPToolOutput` | ⚠️ 공식 미문서화 — feature flag 로 wrap | — |
| 브랜딩 | 경고 없음 | **"Claude Code" 명칭 사용 금지**, claude.ai OAuth 금지 | [overview](https://code.claude.com/docs/en/agent-sdk/overview) |

### 1.2 인터뷰 라운드 확정 사항

| 라운드 | 결정 | 확정값 |
|---|---|---|
| 1 | 포지셔닝 | moai-adk 공식 macOS GUI, 모노레포 (`modu-ai/moai-adk` 내부 `moai-studio/` 서브디렉토리) |
| 2 | 라이선스 | **MIT** |
| 3 | OS | **macOS 영구 단독** (v4 에서 재확정) |
| 4 | Ghostty 수용 | ✅ `GhosttyKit.xcframework` |
| 5 | Pane Splitter | NSSplitView 자체 구현 |

### 1.3 Linux/WSL/Windows 포기 근거 (영구)

B5 리서치의 주요 발견:

1. **Ghostty 공식 미지원 WSL** — 메인테이너 인용: "WSL isn't really a supported target at the moment, and there's no guarantee it will work... issues won't be actively fixed"
2. **GTK4 on WSLg 버그** — [#922 empty windows](https://github.com/microsoft/wslg/issues/922), [#754 resize broken](https://github.com/microsoft/wslg/issues/754), [#1265 popup issues](https://github.com/microsoft/wslg/issues/1265)
3. **9P 파일 시스템 병목** — `/mnt/c/` 는 ~9배 느림. "프로젝트는 WSL 내부에만" 제약 = 일반 Windows 사용자 UX 와 정면 충돌
4. **libghostty 대신 VTE 사용** 시 macOS 와 Linux 가 **다른 터미널 백엔드 2개** — 복잡도 2배, 버그 가능성 2배
5. **Tier 3 네이티브 Windows** = 3-6개월 추가 엔지니어링. 형님의 현재 스코프 아님
6. **cmux 의 성공** = macOS 단독 전략. 증명된 선택

**v4 약속**: M0 ~ M8, 전 마일스톤 macOS 만. `moai-studio-linux`, `moai-studio-wsl`, `moai-studio-windows` 는 **로드맵에 존재하지 않음**.

### 1.4 삭제된 v3 내용

- Direct Connect / Unix socket / `cc+unix://` transport (§5.8)
- `initialize.hooks` in-process callback 모델 (§5.2, §5.3 in v3)
- "Rust core 완전 제거, Pure Swift" 논리 (§0 피벗 2 in v3)
- SwiftTreeSitter / GRDB.swift / SwiftGit2 매트릭스 (§0 피벗 2 in v3)
- 모든 Linux/WSL/Windows 섹션
- Tier Ladder 전략

### 1.5 신규 추가 (v4)

- **IDE MCP Server Pattern** 을 PRIMARY 경로로 문서화 (§5.2)
- **`http` hook type** 을 통한 hook 브리징 (§5.3)
- **Plugin `.lsp.json`** 으로 LSP 를 무료 획득 (§5.6)
- **브랜딩 제약** 섹션 (§2.2)
- **Rust core 재도입 근거** 상세화 (§0 피벗 3)

---

## 2. 제품 정의

### 2.1 한 줄

> **MoAI Studio**: moai-adk 의 공식 macOS 네이티브 IDE-쉘. Claude Code 를 subprocess 로 호스트하여 27개 hook 이벤트 + 26 전문 에이전트 + TRUST 5 품질 게이트 + @MX 태그 시스템 + Kanban/SPEC 워크플로우를 한 화면에서 시각화·조작한다.

### 2.2 브랜딩 제약 ([overview](https://code.claude.com/docs/en/agent-sdk/overview))

Anthropic 공식 규정:

> "Third parties may use 'Claude Agent' or 'Claude' in their products, but MUST NOT use 'Claude Code' or 'Claude Code Agent' names, or Claude Code-branded ASCII art."

**MoAI Studio 네이밍 가이드:**

- ✅ OK: "MoAI Studio", "MoAI Agent IDE", "moai + Claude", "Powered by Claude"
- ❌ NO: "moai Claude Code GUI", "Claude Code for macOS", "Official Claude Code extension"
- ❌ NO: Claude Code 의 ASCII 로고 차용
- ✅ OK: "A native macOS IDE shell for Claude-powered agent development"

**인증**: `ANTHROPIC_API_KEY` / Bedrock / Vertex / Foundry 만. **claude.ai OAuth 구현 금지.**

### 2.3 포지셔닝 (한 문장)

> cmux 가 터미널 멀티화 + 브라우저 surface 를 잘 한다면, MoAI Studio 는 그 위에 **SPEC/TRUST/@MX/Kanban 워크플로우 시각화** 와 **Claude 가 직접 UI 를 조작할 수 있는 `--mcp-config` SSE/Streamable HTTP 커스텀 MCP 통합** 을 올린다.

### 2.4 핵심 요구사항

1. 파일 탐색기 (FileTree surface)
2. GPU 가속 터미널 (libghostty)
3. Code Viewer (SwiftTreeSitter + LSP 진단 + @MX 거터 + tri-pane diff)
4. 마크다운 뷰어 (EARS SPEC 특화)
5. 내장 브라우저 (WKWebView + DevTools)
6. 이미지 뷰어 (diff 모드)
7. Agent Run Viewer (hook event stream + cost tracking)
8. Kanban 보드 (SPEC ↔ worktree ↔ `/moai run` 자동 연동)
9. 다중 세션 (16+ 에이전트 동시)
10. `/moai *` 14 슬래시 커맨드 GUI 1-클릭 호출
11. **Memory Viewer** (`~/.claude/projects/<root>/memory/` 렌더)
12. **Instructions Loaded Graph** (세션 컨텍스트 디버거)
13. **In-app Claude 가 UI 조작** (IDE MCP server 의 `mcp__moai__*` tools)
14. **Native Permission Dialog** (TUI text prompt 대체)

### 2.5 비기능 요구사항

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
| 크래시 격리 | 에이전트 1개 크래시가 나머지에 전파되지 않음 |

---

## 3. 아키텍처

### 3.1 5단 계층 (유지)

```
Window
 └── Project          ← git 루트 + .moai/ 감지
      └── Workspace   ← 1 claude subprocess = 1 git worktree
           ├── agent_host: claude_code_sdk | shell | tmux_cg
           ├── binds: SPEC-{DOMAIN}-{NNN}
           └── Pane    ← NSSplitView 자체 binary tree
                └── Surface  (10종 — §6 참조)
```

### 3.2 프로세스 토폴로지

```
┌─────────────────────────────────────────────────────────┐
│            MoAI Studio.app (macOS, SwiftUI + Rust)          │
│                                                          │
│  ┌──────────────────────────────────┐                   │
│  │     Swift UI Layer                │                   │
│  │  ──────────────────               │                   │
│  │  • SwiftUI Shell + AppKit bridge  │                   │
│  │  • Sidebar · Tabs · NSSplitView   │                   │
│  │  • 10 Surface implementations     │                   │
│  │  • GhosttyKit (Terminal surface)  │                   │
│  │  • Command Palette                │                   │
│  └────────────┬─────────────────────┘                    │
│               │ swift-bridge FFI                         │
│  ┌────────────▼─────────────────────────────────────┐   │
│  │          Rust Core (moai-core crate workspace)    │   │
│  │  ──────────────────────────────                   │   │
│  │  • tokio actor-based RootSupervisor              │   │
│  │  • WorkspaceSupervisor × N                       │   │
│  │  • ClaudeSubprocessManager                       │   │
│  │  • StreamJsonCodec (SDKMessage parser)           │   │
│  │  • IdeMcpServer (axum + rmcp/jsonrpsee)          │   │
│  │    → 127.0.0.1:<port> + ~/.claude/ide/*.lock     │   │
│  │  • HookHttpEndpoint (axum route)                 │   │
│  │  • Store (rusqlite WAL + r2d2 pool)              │   │
│  │  • Git (git2)                                    │   │
│  │  • FsWatcher (notify)                            │   │
│  │  • PluginInstaller                               │   │
│  │  • LockfileDaemon                                │   │
│  └────────┬────────────────────────┬─────────────────┘   │
│           │ stdin/stdout            │                     │
└───────────┼────────────────────────┼─────────────────────┘
            │                         │
            ▼                         ▼
┌──────────────────────┐   ┌────────────────────────┐
│  claude subprocess    │   │   Plugin http hooks     │
│  (per workspace)      │   │   POST to loopback      │
│  --bare -p            │──►│   X-Auth-Token          │
│  --mcp-config ...     │   │   → hook_http_endpoint  │
└──────────────────────┘   └────────────────────────┘
```

### 3.3 멀티 에이전트 모델

**원칙:**

1. **1 Workspace = 1 Claude subprocess = 1 git worktree**. 병렬 에이전트 격리 단위.
2. **Rust Tokio actor 가 라이프사이클 소유**. `WorkspaceActor` 가 Claude subprocess 를 async Task 로 감싼다. actor crash 시 one_for_one 재시작.
3. **SDK control protocol + http hook endpoint 이중 채널**. Claude → stdin/stdout → Rust StreamJsonCodec → EventBus. Plugin hooks → HTTP POST → Rust HookHttpEndpoint → 같은 EventBus. UI 는 EventBus subscriber.
4. **권한 = IDE MCP bearer token**. `~/.claude/ide/<port>.lock` 의 token 이 MCP + hook HTTP 양쪽 인증.
5. **moai-adk 는 건드리지 않고 얹는다**. MoAI Studio 는 `moai` Go 바이너리를 worktree 연산을 위해 subprocess 로 호출할 뿐. `claude` 를 SDK 로 호스팅.

### 3.4 Rust Core 모듈 (moai-core crate workspace)

```
moai-core/
├── Cargo.toml                    # workspace
├── crates/
│   ├── moai-core/                 # 상위 facade crate
│   ├── moai-supervisor/           # RootSupervisor, WorkspaceSupervisor
│   ├── moai-claude-host/          # ClaudeSubprocessManager
│   ├── moai-stream-json/          # SDKMessage codec
│   ├── moai-ide-server/           # IDE MCP server + lockfile daemon
│   ├── moai-hook-http/            # Plugin http hook receiver
│   ├── moai-store/                # rusqlite store + migrations
│   ├── moai-git/                  # git2 wrapper
│   ├── moai-fs/                   # notify wrapper
│   ├── moai-plugin-installer/     # moai-studio-plugin 자동 설치
│   └── moai-ffi/                  # swift-bridge 정의
└── xtask/                         # build scripts for xcframework
```

### 3.5 FFI 경계 (Swift ↔ Rust)

**도구**: `swift-bridge` (Rust → Swift, compile-time code generation)

**교환 타입:**

```rust
// moai-ffi/src/lib.rs
#[swift_bridge::bridge]
mod ffi {
    extern "Rust" {
        type RustCore;

        #[swift_bridge(init)]
        fn new() -> RustCore;

        async fn start_workspace(&self, config: WorkspaceConfig) -> WorkspaceId;
        async fn stop_workspace(&self, id: WorkspaceId);
        fn subscribe_hook_events(&self, id: WorkspaceId) -> HookEventStream;
        async fn send_user_message(&self, id: WorkspaceId, msg: String);
        async fn interrupt_workspace(&self, id: WorkspaceId);
        async fn set_permission_mode(&self, id: WorkspaceId, mode: String);
        async fn set_model(&self, id: WorkspaceId, model: String);

        fn open_project(&self, path: String) -> ProjectId;
        fn list_workspaces(&self, project_id: ProjectId) -> Vec<WorkspaceSummary>;
        fn create_workspace_from_spec(&self, spec_id: String) -> WorkspaceId;

        fn get_cost_summary(&self, ws: WorkspaceId) -> CostSummary;
        fn query_hook_events(&self, filter: HookEventFilter) -> Vec<HookEventRecord>;
    }

    extern "Swift" {
        // UI → Rust callbacks (rare)
        fn on_permission_request(request: PermissionRequestPayload) -> PermissionDecision;
    }
}
```

**Swift 측 사용:**

```swift
import MoaiCore

@Observable
class WorkspaceViewModel {
    let core: RustCore
    var events: AsyncStream<HookEventRecord>

    init(core: RustCore, workspaceId: WorkspaceId) {
        self.core = core
        self.events = core.subscribe_hook_events(id: workspaceId)
    }

    func send(_ msg: String) async {
        await core.send_user_message(id: workspaceId, msg: msg)
    }
}
```

---

## 4. Claude Code 통합 — PRIMARY 경로

### 4.1 Subprocess Spawn

```rust
// moai-claude-host/src/lib.rs (개념 코드)
use tokio::process::{Command, Child};
use std::process::Stdio;

pub async fn spawn_claude(ws: &Workspace) -> Result<ClaudeHost> {
    let mut cmd = Command::new("claude");
    cmd.args(&[
        "--bare",
        "-p", "",
        "--output-format", "stream-json",
        "--include-partial-messages",
        "--verbose",
        "--permission-mode", "acceptEdits",
        "--settings", &ws.settings_path.to_string_lossy(),
        "--mcp-config", &moai_cli_mcp_config_path(ws)?.to_string_lossy(),
        "--plugin-dir", &moai_cli_plugin_dir().to_string_lossy(),
        // [SPIKE ERRATA] --tools 사용 (--allowedTools 는 additive 이므로 도구 제한 불가)
        "--tools", "Read,Edit,Write,Bash,Glob,Grep,mcp__moai__*,mcp__ide__*",
    ]);
    cmd.current_dir(&ws.worktree_path);
    // [SPIKE ERRATA] CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1 (subprocess 기본값) 이면
    // --permission-mode 가 강제로 `default` 로 리셋된다. MoAI Studio 는
    // 이 변수를 0 으로 설정하여 명시한 permission-mode 를 보존해야 한다.
    cmd.env("CLAUDE_CODE_SUBPROCESS_ENV_SCRUB", "0");
    cmd.envs(ws.env_vars());
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    let codec = StreamJsonCodec::new(
        child.stdin.take().unwrap(),
        child.stdout.take().unwrap(),
    );
    Ok(ClaudeHost { child, codec, workspace_id: ws.id })
}
```

### 4.2 IDE MCP Server 구현

Rust core 가 127.0.0.1 에 MCP server 를 열고 lockfile 을 drop 한다.

```rust
// moai-ide-server/src/lib.rs (개념 코드)
use axum::{Router, routing::post};
use std::net::SocketAddr;

pub async fn start_ide_server(state: Arc<AppState>) -> Result<IdeServerHandle> {
    let port = pick_random_high_port()?;
    let auth_token = generate_auth_token();  // 32-byte hex via `ring`

    // MCP tool router
    let app = Router::new()
        .route("/mcp", post(mcp_handler))
        .with_state(state.clone())
        .layer(auth_middleware(auth_token.clone()));

    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server_task = tokio::spawn(async move {
        axum::serve(listener, app).await
    });

    // ~/.claude/ide/<port>.lock drop
    let lockfile = LockFile {
        workspace_folders: state.workspace_folders(),
        pid: std::process::id() as i32,
        ide_name: "MoAI".to_string(),
        transport: "ws".to_string(),
        running_in_windows: false,
        auth_token: auth_token.clone(),
    };
    lockfile_daemon::write(lockfile, port).await?;

    Ok(IdeServerHandle { port, auth_token, server_task })
}

async fn mcp_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<McpRequest>,
) -> Json<McpResponse> {
    match req.method.as_str() {
        "tools/list" => list_moai_tools(),
        "tools/call" => dispatch_tool_call(req.params, state).await,
        _ => McpResponse::error(-32601, "method not found"),
    }
}
```

### 4.3 Hook 브리징 — `http` hook type

Rust core 에서 hook callback 을 in-process 로 받는 경로는 Python/TS SDK 전용이므로, **plugin manifest 의 `http` hook type** 을 활용한다:

```json
// moai-studio-plugin/.claude-plugin/plugin.json
// [SPIKE ERRATA] plugin.json 에 "hooks" 필드를 두지 않는다 (convention-based discovery).
// hooks.json 은 hooks/hooks.json 에 위치하며 Claude Code 가 자동 탐색한다.
{
  "name": "moai-studio",
  "version": "0.1.0",
  "description": "moai-adk IDE shell integration",
  "mcpServers": "./mcp-config.json",
  "lspServers": "./lsp.json",
  "skills": "./skills/",
  "outputStyles": "./output-styles/",
  "userConfig": {
    "ide_port": { "type": "number", "title": "MoAI Studio IDE server port" },
    "auth_token": { "type": "string", "sensitive": true, "title": "Auth token" }
  }
}
```

```json
// moai-studio-plugin/hooks/hooks.json
// [SPIKE ERRATA] hooks.json 은 plugin 루트가 아니라 hooks/ 디렉토리에 위치.
// 최상위에 "hooks" wrapper 가 필요하다.
{
  "hooks": {
  "PreToolUse": [{
    "matcher": "Bash|Write|Edit",
    "hooks": [{
      "type": "http",
      "url": "http://127.0.0.1:${user_config.ide_port}/hooks/PreToolUse",
      "headers": { "X-Auth-Token": "${user_config.auth_token}" },
      "timeout": 5000
    }]
  }],
  "PostToolUse": [{
    "matcher": "Write|Edit",
    "hooks": [{
      "type": "http",
      "url": "http://127.0.0.1:${user_config.ide_port}/hooks/PostToolUse",
      "headers": { "X-Auth-Token": "${user_config.auth_token}" }
    }]
  }],
  "SessionStart":      [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...SessionStart",      "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "SessionEnd":        [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...SessionEnd",        "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "UserPromptSubmit":  [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...UserPromptSubmit",  "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "Notification":      [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...Notification",      "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "Stop":              [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...Stop",              "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "SubagentStart":     [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...SubagentStart",     "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "SubagentStop":      [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...SubagentStop",      "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "TaskCompleted":     [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...TaskCompleted",     "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "TeammateIdle":      [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...TeammateIdle",      "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "PermissionRequest": [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...PermissionRequest", "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "ConfigChange":      [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...ConfigChange",      "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "WorktreeCreate":    [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...WorktreeCreate",    "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "WorktreeRemove":    [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...WorktreeRemove",    "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "PreCompact":        [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...PreCompact",        "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "PostCompact":       [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...PostCompact",       "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "FileChanged":       [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...FileChanged",       "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "InstructionsLoaded":[{ "matcher": "*", "hooks": [{ "type": "http", "url": "...InstructionsLoaded","headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "CwdChanged":        [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...CwdChanged",        "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }],
  "Setup":             [{ "matcher": "*", "hooks": [{ "type": "http", "url": "...Setup",             "headers": { "X-Auth-Token": "${user_config.auth_token}" } }] }]
  }
}
```

**이벤트 plane**: Claude → HTTP POST → Rust `hook_http_endpoint` → `HookCallbackRouter` → UI subscribers + Store 저장.

**응답 plane**: Rust 가 `{"hookSpecificOutput": {...}}` JSON 을 응답 → Claude 가 적용 (permission decision, updatedInput 등).

> [SPIKE ERRATA] `hookSpecificOutput` 내부에 `hookEventName` 필드를 포함하지 않는다. 올바른 형식: `{"hookSpecificOutput": {"permissionDecision": "allow", "updatedInput": {...}}}`. `hookEventName` 은 요청 payload 에만 존재하며 응답에 넣으면 무시되거나 오류 발생.

**장점:**
- Shell wrapper 없음
- 언어 무관 (Rust, Swift, Go — HTTP 서버만 돌리면 됨)
- 공식 문서화된 hook type
- 인증 통합 = IDE server 와 같은 `X-Auth-Token`

### 4.4 Hook 기능 매트릭스

| 기능 | Hook 이벤트 | Response 필드 | 공식 여부 |
|---|---|---|---|
| Bash safe wrapper | PreToolUse | `hookSpecificOutput.updatedInput` | ✅ 공식 |
| Path normalization | PreToolUse | `hookSpecificOutput.updatedInput` | ✅ 공식 |
| Native permission dialog | PermissionRequest | `hookSpecificOutput.decision` | ✅ 공식 |
| Always-allow 영구화 | PermissionRequest | `decision.updatedPermissions` | ⚠️ 미문서화 (feature flag) |
| File tree live update | FileChanged | 관찰만 | ✅ 공식 |
| Session context 주입 | SessionStart | `hookSpecificOutput.additionalContext` | ✅ 공식 |
| Watch 등록 | SessionStart | `hookSpecificOutput.watchPaths` | ⚠️ 미문서화 (feature flag) |
| Config hot-reload | ConfigChange | 관찰만 | ✅ 공식 |
| Instructions graph | InstructionsLoaded | 관찰만 | ✅ 공식 |
| Cost tracker | PostToolUse + cost_update SDK message | 관찰만 | ✅ 공식 |
| Worktree tracker | WorktreeCreate / WorktreeRemove | 관찰만 | ✅ 공식 |

**⚠️ 미문서화** 는 `try { use feature } catch { graceful degradation }` 패턴으로 감싼다.

### 4.5 IDE MCP Tool 카탈로그 (MoAI Studio 가 Claude 에 노출)

| Tool | 효과 |
|---|---|
| `mcp__moai__workspace_open` | focused workspace 전환 |
| `mcp__moai__workspace_create` | 새 workspace + git worktree + Claude subprocess |
| `mcp__moai__kanban_create_card` | Kanban 카드 생성 |
| `mcp__moai__kanban_move_card` | 카드 이동 |
| `mcp__moai__surface_reveal` | 특정 surface 포커스 |
| `mcp__moai__surface_open` | 새 surface 생성 |
| `mcp__moai__notification_post` | 네이티브 macOS 알림 |
| `mcp__moai__terminal_spawn` | 새 Ghostty PTY surface |
| `mcp__moai__file_reveal_in_finder` | Finder 에 하이라이트 |
| `mcp__moai__memory_reveal_entry` | Memory surface 의 entry 스크롤 |
| `mcp__moai__instructions_graph_highlight` | 그래프에서 파일 강조 |
| `mcp__ide__getDiagnostics` | LSP 진단 (VS Code 호환 표준) |
| `mcp__ide__openDiff` | Code Viewer tri-pane diff |
| `mcp__ide__readSelection` | 사용자 선택 영역 |

**예시 시나리오:**
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

### 4.6 LSP 통합 — `.lsp.json` plugin feature

공식 plugin manifest 의 `lspServers` 필드 ([plugins-reference](https://code.claude.com/docs/en/plugins-reference)):

```json
// moai-studio-plugin/lsp.json
{
  "gopls":         { "command": "gopls",                   "args": ["serve"],   "filetypes": ["go"] },
  "rust-analyzer": { "command": "rust-analyzer",           "filetypes": ["rust"] },
  "pyright":       { "command": "pyright-langserver",      "args": ["--stdio"], "filetypes": ["python"] },
  "tsserver":      { "command": "typescript-language-server", "args": ["--stdio"], "filetypes": ["typescript", "javascript", "tsx", "jsx"] },
  "clangd":        { "command": "clangd",                  "filetypes": ["c", "cpp", "objc", "objcpp"] },
  "sourcekit":     { "command": "sourcekit-lsp",           "filetypes": ["swift"] }
}
```

Claude Code 가 LSP 서버를 spawn + 관리. MoAI Studio Code Viewer 는 `mcp__ide__getDiagnostics` tool 호출로 진단을 받아옴. **자체 LSP 클라이언트 구현 불필요** — 이것이 v4 의 큰 단순화 중 하나.

---

## 5. Surfaces (10개)

### 5.1 Terminal Surface (GhosttyKit)

- **엔진**: `GhosttyKit.xcframework` (Ghostty submodule → Zig build → xcframework)
- **소유권**: PTY master = Ghostty. MoAI Studio 는 attach view.
- **GPU**: Metal 60fps@4K
- **Sixel / iTerm2 inline image** 네이티브 지원
- **moai-adk 데코레이션**: PostToolUse hook 수신 시 좌측 거터에 `●plan`, `●run`, `●sync`, `●fix` 아이콘
- **Scrollback 검색**: Rust core 의 rusqlite FTS5
- **Command palette → Slash injection**: Cmd+K 에서 `/moai run SPEC-AUTH-042` 선택 시 Rust core 가 `SDKUserMessage` 로 claude subprocess 에 전송

### 5.2 Code Viewer Surface (v4 핵심)

- **렌더러**: `NSTextView` 서브클래스 + **SwiftTreeSitter** (tree-sitter 바인딩)
- **LSP 진단**: `mcp__ide__getDiagnostics` tool 호출 결과 표시 (Claude 가 plugin `.lsp.json` 으로 spawn 한 LSP 서버 경유)
- **@MX 거터**: Rust core 가 `/moai mx --dry --json` 결과를 `mx_tags` 테이블에 캐시. 거터 아이콘 ★ ⚠ ℹ ☐. 클릭 → inspector.
- **LSP gate overlay**: `.moai/config/sections/quality.yaml` 의 임계값과 실시간 에러 카운트 비교. 초과 시 붉은 배너.
- **Tri-pane diff**: `HEAD:main | working tree | agent pending`. Accept / Revert 버튼.
- **SPEC 링크**: `@MX:ANCHOR SPEC-AUTH-042` 주석 클릭 → Markdown surface
- **Time travel**: `git log -p` 슬라이더 스크럽. 각 시점 task-metric bar chart.
- **Edit mode**: read-only 기본. 편집 진입 시 자동 git stash 스냅샷.

### 5.3 Markdown Surface (EARS 특화)

- **렌더러**: `Down` (Swift cmark wrapper)
- **확장**: KaTeX (수식), Mermaid (다이어그램) via WKWebView
- **라이브**: Rust `notify` → Swift 200ms debounce
- **EARS 모드**: `.moai/specs/SPEC-*/spec.md` 열면 Given/When/Then 블록을 카드로 렌더. Acceptance 체크리스트 인터랙티브.
- **2-up**: 좌 SPEC, 우 `git diff vs main`

### 5.4 Image Surface

- **엔진**: Core Image + Metal
- **Artifacts watch**: Rust `notify` 로 `artifacts/` 감지
- **Diff 모드**: SSIM 점수 + 픽셀 diff (Vision framework)
- **`/moai e2e` 연동**: Playwright 결과 스크린샷 자동 오픈

### 5.5 Browser Surface

- **엔진**: `WKWebView`
- **DevTools**: `setInspectable(true)`
- **Port 스캐너**: Rust core 가 dev 서버 포트 감지 → 사이드바 리스트
- **`/moai e2e` 연동**: Claude-in-Chrome 테스트 결과 임베드

### 5.6 FileTree Surface

- **엔진**: Rust `notify` + Swift UI list
- **Git status**: Rust `git2` → 색상 (M/A/D/?)
- **컨텍스트 액션**:
  - Reveal in Finder
  - Open in Code Viewer
  - Send path to focused agent (SDK user message injection)
  - Diff against main
  - "Create SPEC from selection"
- **드래그 드롭**: 외부 파일 → workspace worktree 복사 + 에이전트 컨텍스트 첨부
- **FileChanged hook** 구독으로 **Claude 가 만든 파일** 과 **사용자가 만든 파일** 을 색상 구분

### 5.7 Agent Run Viewer (v4 핵심)

**데이터 소스**:
- Primary: 27 hook event stream via http hook endpoint → Rust store → Swift `AsyncStream`
- Secondary: `.moai/logs/task-metrics.jsonl` tail (MoAI Studio 꺼진 동안 복구)

**레이아웃** (design-exports/05-agent-run.png 기준, v4 업그레이드):

- **좌측**: 세션/태스크 타임라인 (SubagentStart ~ SubagentStop)
- **중앙**: 선택 task 의 step-by-step 트레이스 (SessionStart / PreToolUse / PostToolUse / Notification / TaskCompleted 카드)
- **우측 (v4 신규)**: **Live agent control**
  - 현재 실행 중인 tool spinner
  - 편집 중인 파일
  - **Interrupt 버튼** (`moai_core::interrupt_workspace`)
  - **모델 변경 드롭다운** (`moai_core::set_model`)
  - **Permission mode 토글** (`moai_core::set_permission_mode`)
- **하단 액션**:
  - Replay from here
  - Open failing file
  - Revert commits by this run

### 5.8 Kanban Board

- **레인**: Backlog / To-Do / Doing / Review / Done / Blocked
- **카드**: title, body_md, spec_id, assignee, labels, linked files
- **Doing 자동화 (v4 업그레이드)**:
  1. 드래그 → Rust core 의 `create_workspace_from_spec` 호출
  2. `git worktree add` (git2) + `claude subprocess spawn`
  3. `initialize` control request (Rust → Claude)
  4. `SDKUserMessage` 로 `/moai run SPEC-AUTH-042` 전송
  5. `Surface.reveal({surface: "agent_run"})` 3-pane 자동 구성
- **Review 자동화**:
  1. `git diff main..HEAD` → Markdown surface
  2. `/moai review` → SDK user message
  3. TRUST 5 점수 + LSP gate 결과를 카드 배지로
- **Done 자동화**: `gh pr create` 옵션 + worktree archive
- **Backlog 생성**: `/moai plan` 결과를 WorktreeCreate hook 으로 감지 → 자동 카드 생성
- **저장**: rusqlite `kanban_cards`

### 5.9 Memory Surface (v4 추가)

- **데이터**: `~/.claude/projects/<sanitized-git-root>/memory/` 의 markdown 파일 직접 렌더
- **렌더러**: Markdown surface 재사용 + 특화 UI
  - 좌: `MEMORY.md` index (트리)
  - 우: 선택 토픽 파일 preview
  - 하: 25KB / 200 라인 cap progress bar
- **편집**: Edit 버튼 → Code Viewer. 저장 시 Claude Code 도 즉시 반영.
- **ConfigChange hook** 구독 → 자동 refresh
- 근거: `research/B3 §6 Memory`

### 5.10 InstructionsGraph Surface (v4 추가)

`InstructionsLoaded` hook (`coreSchemas.ts:695-707`) 을 구독해 현재 세션 컨텍스트에 어떤 CLAUDE.md / skill / memory 가 로드되었는지 시각화.

- **노드**: 각 로드된 파일
  - `memory_type`: User / Project / Local / Managed (색상 구분)
  - `load_reason`: session_start / nested_traversal / path_glob_match / include / compact (아이콘)
  - `globs`, `trigger_file_path`, `parent_file_path` → edge
- **클릭**: 해당 파일을 Markdown Surface 또는 Code Viewer 에서 열기
- **용도**: "왜 이 파일이 컨텍스트에 있는가?" 디버깅. 프롬프트 엔지니어링 블랙박스 open.
- **독창성**: 어떤 경쟁사도 없음.

### 5.11 명령 팔레트 (Cmd+K)

- **소스**: MCP tools, slash 커맨드, 파일, SPEC, 카드, 심볼 (SwiftTreeSitter)
- **moai-adk 섹션 1급**: 14개 `/moai *`
- **동사형**: "Run /moai coverage on focused workspace" 등
- **Slash injection**: 선택 → Rust core 의 `send_user_message` → Claude subprocess

---

## 6. 데이터 모델 (rusqlite WAL)

```sql
-- moai-store/migrations/v1__initial.sql

CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    root TEXT UNIQUE NOT NULL,
    name TEXT,
    is_moai_adk INTEGER NOT NULL DEFAULT 0,
    moai_version TEXT,
    opened_at INTEGER NOT NULL
);

CREATE TABLE workspaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id),
    name TEXT NOT NULL,
    branch TEXT,
    worktree_path TEXT NOT NULL,
    agent_host TEXT NOT NULL,  -- claude_code_sdk|shell|tmux_cg
    spec_id TEXT,
    status TEXT NOT NULL,      -- starting|running|waiting|review|error|archived
    claude_session_id TEXT,
    created_at INTEGER NOT NULL,
    last_active_at INTEGER NOT NULL
);

CREATE TABLE panes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id),
    parent_id INTEGER,
    split TEXT,  -- horizontal|vertical|leaf
    ratio REAL
);

CREATE TABLE surfaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pane_id INTEGER NOT NULL REFERENCES panes(id),
    kind TEXT NOT NULL,  -- terminal|code|markdown|image|browser|filetree|agent_run|kanban|memory|instructions_graph
    state_json BLOB
);

-- v4 주 데이터 소스: hook event stream
CREATE TABLE hook_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id),
    ts INTEGER NOT NULL,
    event TEXT NOT NULL,  -- 18~25 공식 이벤트
    callback_id TEXT,
    session_id TEXT,
    agent_id TEXT,
    tool_use_id TEXT,
    matcher TEXT,
    payload BLOB NOT NULL,       -- JSON serialized
    response_payload BLOB,       -- updatedInput 등
    duration_ms INTEGER
);
CREATE INDEX hook_events_ws_ts ON hook_events(workspace_id, ts);

CREATE TABLE cost_updates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id),
    ts INTEGER NOT NULL,
    turn_number INTEGER,
    model TEXT,
    input_tokens INTEGER,
    output_tokens INTEGER,
    cache_read_tokens INTEGER,
    cache_write_tokens INTEGER,
    estimated_cost_usd REAL
);

-- task-metrics.jsonl 백업 미러
CREATE TABLE task_metrics_mirror (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    workspace_id INTEGER NOT NULL REFERENCES workspaces(id),
    ts INTEGER NOT NULL,
    session_id TEXT,
    task_id TEXT,
    agent_type TEXT,
    operation TEXT,
    input_tokens INTEGER,
    output_tokens INTEGER,
    total_tokens INTEGER,
    duration_ms INTEGER,
    tool_calls INTEGER,
    tools_used TEXT,   -- JSON array
    status TEXT,
    spec_id TEXT
);
CREATE INDEX task_metrics_spec ON task_metrics_mirror(spec_id);

CREATE TABLE specs (
    id TEXT PRIMARY KEY,  -- SPEC-AUTH-042
    project_id INTEGER NOT NULL REFERENCES projects(id),
    title TEXT,
    ears_md TEXT,
    plan_md TEXT,
    status TEXT NOT NULL,  -- draft|running|review|done
    updated_at INTEGER NOT NULL
);

CREATE TABLE mx_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id),
    path TEXT NOT NULL,
    line INTEGER NOT NULL,
    kind TEXT NOT NULL,  -- ANCHOR|WARN|NOTE|TODO
    reason TEXT
);
CREATE INDEX mx_tags_path ON mx_tags(path);

CREATE TABLE kanban_boards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id),
    name TEXT NOT NULL
);

CREATE TABLE kanban_cards (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    board_id INTEGER NOT NULL REFERENCES kanban_boards(id),
    lane TEXT NOT NULL,
    title TEXT NOT NULL,
    body_md TEXT,
    workspace_id INTEGER REFERENCES workspaces(id),
    spec_id TEXT,
    assignee TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts INTEGER NOT NULL,
    kind TEXT NOT NULL,
    ref TEXT,
    body TEXT,
    read INTEGER NOT NULL DEFAULT 0
);
```

**설정**: WAL, `synchronous=NORMAL`, batch insert (100 rows / 100ms), 30일 TTL on hook_events / task_metrics_mirror.

---

## 7. 기술 스택 (확정)

### 7.1 Swift Shell

| 컴포넌트 | 기술 |
|---|---|
| UI | SwiftUI + AppKit (macOS 14+) |
| 터미널 | `GhosttyKit.xcframework` |
| 코드 하이라이트 | `SwiftTreeSitter` |
| Pane splitter | `NSSplitView` + 자체 binary tree |
| Markdown | `Down` (cmark wrapper) |
| WebView | `WKWebView` |
| 자동 업데이트 | `Sparkle` 2.x |
| 크래시 | Sentry-Cocoa (opt-in) |

### 7.2 Rust Core (`moai-core` crate workspace)

| 카테고리 | crate |
|---|---|
| Async runtime | `tokio` 1.x |
| Stream-json codec | 자체 구현 (serde_json + tokio::codec) |
| MCP server | `axum` + `jsonrpsee` (또는 `rmcp` 검증 후) |
| HTTP endpoint | `axum` |
| Database | `rusqlite` + `r2d2` + `refinery` |
| Git | `git2` (libgit2) |
| File watch | `notify` 7.x |
| Auth token | `ring` (secure random) |
| Serialization | `serde` + `serde_json` |
| Logging | `tracing` + `tracing-subscriber` |
| FFI to Swift | `swift-bridge` |

### 7.3 Build Toolchain

- **Xcode**: 15+
- **Rust**: 1.80+
- **Zig**: 0.13+ (Ghostty build)
- **cargo xtask**: xcframework 빌드 자동화
- **Makefile / `just`**: 원스텝 `just build` 로 Rust → xcframework → Xcode archive

### 7.4 moai-studio-plugin

```
moai-studio-plugin/
├── .claude-plugin/plugin.json       # manifest (hooks 필드 없음 — convention-based discovery)
├── hooks/
│   └── hooks.json                   # [SPIKE ERRATA] hooks/ 디렉토리에 위치, {"hooks": {...}} wrapper 필수
├── mcp-config.json                  # MoAI Studio IDE server 연결
├── lsp.json                         # 6개 언어 LSP
├── commands/
│   ├── kanban.md                    # /moai-studio:kanban
│   ├── memory.md                    # /moai-studio:memory
│   ├── connect.md                   # /moai-studio:connect
│   └── surface.md                   # /moai-studio:surface
├── skills/
│   ├── moai-studio-open-workspace/SKILL.md
│   └── moai-studio-focus-agent/SKILL.md
├── output-styles/
│   └── moai-studio.md                  # forceForPlugin: true
└── agents/
    └── (moai-adk 26 에이전트 참조)
```

---

## 8. 디렉토리 구조 (모노레포)

```
moai-adk-go/                        # 기존 moai-adk Go CLI 저장소
├── cmd/moai/                        # Go entry
├── internal/                        # Go 내부
├── pkg/                             # Go 공개
├── moai-studio/                        # v4 루트
│   ├── DESIGN.v4.md                 # 이 문서
│   ├── research/                    # R1, B1, B2, B3, B4, B5
│   ├── design-exports/              # PNG 목업
│   │
│   ├── app/                         # macOS Xcode 프로젝트
│   │   ├── MoAI Studio.xcodeproj
│   │   ├── Sources/
│   │   │   ├── App/                 # @main, AppDelegate
│   │   │   ├── Shell/               # Sidebar, Tabs, Splits, CommandPalette
│   │   │   ├── Surfaces/            # 10 surfaces
│   │   │   │   ├── Terminal/        # GhosttyKit wrapper
│   │   │   │   ├── CodeViewer/      # NSTextView + SwiftTreeSitter + @MX
│   │   │   │   ├── Markdown/
│   │   │   │   ├── Image/
│   │   │   │   ├── Browser/         # WKWebView
│   │   │   │   ├── FileTree/
│   │   │   │   ├── AgentRun/
│   │   │   │   ├── Kanban/
│   │   │   │   ├── Memory/          # v4
│   │   │   │   └── InstructionsGraph/  # v4
│   │   │   ├── Bridge/              # swift-bridge generated + wrappers
│   │   │   └── Theme/
│   │   └── Resources/
│   │
│   ├── core/                        # Rust workspace
│   │   ├── Cargo.toml
│   │   ├── crates/
│   │   │   ├── moai-core/           # facade
│   │   │   ├── moai-supervisor/
│   │   │   ├── moai-claude-host/
│   │   │   ├── moai-stream-json/
│   │   │   ├── moai-ide-server/
│   │   │   ├── moai-hook-http/
│   │   │   ├── moai-store/
│   │   │   ├── moai-git/
│   │   │   ├── moai-fs/
│   │   │   ├── moai-plugin-installer/
│   │   │   └── moai-ffi/            # swift-bridge definitions
│   │   └── xtask/                   # xcframework build scripts
│   │
│   ├── plugin/                      # moai-studio-plugin
│   │   ├── .claude-plugin/plugin.json
│   │   ├── hooks/
│   │   │   └── hooks.json           # [SPIKE ERRATA E5]
│   │   ├── mcp-config.json
│   │   ├── lsp.json
│   │   ├── commands/
│   │   ├── skills/
│   │   ├── output-styles/
│   │   └── agents/
│   │
│   ├── vendor/
│   │   ├── ghostty/                 # submodule
│   │   └── tree-sitter-grammars/    # submodule
│   │
│   ├── scripts/
│   │   ├── build-xcframework.sh     # Ghostty xcframework
│   │   ├── build-rust-xcframework.sh  # moai-core xcframework
│   │   ├── install-plugin.sh        # ~/.claude/plugins/moai-studio@local/
│   │   └── reload.sh
│   │
│   ├── tests/
│   │   ├── rust-unit/               # cargo test
│   │   ├── rust-integration/        # mock claude subprocess
│   │   ├── swift-unit/              # Swift Testing
│   │   ├── swift-ui/                # XCUITest
│   │   └── stress/                  # 16 workspace
│   │
│   └── docs/
│
└── .github/workflows/
    ├── ci-go.yml                    # 기존 Go CI
    ├── ci-moai-studio-rust.yml         # cargo test
    ├── ci-moai-studio-swift.yml        # Xcode build + test
    └── release-moai-studio.yml         # notarize + DMG + Sparkle
```

**Path filter**: `moai-studio/**` 변경 시만 CLI CI 트리거. Go CI 와 독립.

---

## 9. 마일스톤

| 단계 | 기간 | 산출물 |
|---|---|---|
| **Pre-M0** | 3-4일 | 검증 스파이크 (§10.1) |
| **M0** | 2주 | Xcode 프로젝트 + Rust core skeleton + GhosttyKit + Claude subprocess spawn + initialize control request + hook_callback roundtrip + IDE MCP server proof-of-life |
| **M1 Core Sessions** | 3주 | Workspace / Pane / Surface 모델, NSSplitView binary tree, rusqlite store v1, swift-bridge FFI, Sidebar, Terminal surface, HookHttpEndpoint 18 이벤트 wired |
| **M2 Viewers 1** | 3주 | FileTree, Markdown, Image, Browser |
| **M3 Code Viewer** | 3주 | SwiftTreeSitter, `mcp__ide__getDiagnostics` 연동, @MX 거터, tri-pane diff, time-travel |
| **M4 Claude 통합 심화** | 3주 | moai-studio-plugin 자동 설치, Native permission dialog, Bash input rewriter, LSP 6개 언어 등록, cost tracker, output style forceForPlugin |
| **M5 Agent Run + Kanban + Memory** | 3주 | Agent Run Viewer, Kanban board, Memory surface, Instructions Graph, EARS markdown 모드 |
| **M6 안정화 + 배포** | 2주 | Sparkle, notarize, 16-agent stress, DMG |
| **M7 (옵션)** | 2주 | Nightly 채널, 버그 수정 sprint |

**총 M0-M6: 19주** (Pre-M0 포함 20주).

### 9.1 M0 상세 (2주)

**주 1:**
- D1: Xcode 프로젝트 + Rust cargo workspace 생성
- D2: Ghostty submodule clone + `zig build -Demit-xcframework=true` + xcframework 링크 검증
- D3: Rust `moai-claude-host` 구현 + `claude --bare -p --output-format stream-json` spawn
- D4: `moai-stream-json` crate — SDKMessage codec (`assistant_message`, `user_message`, `system/init`, `tool_use`, `tool_result`, `result`)
- D5: Rust `moai-ide-server` — axum + jsonrpsee MCP 서버 + lockfile drop

**주 2:**
- D6: `claude --mcp-config moai-studio.json -p "..."` 실행 → MoAI Studio MCP server 에 auto-connect 확인
- D7: `mcp__moai__echo` 라는 단일 debug tool 왕복 검증
- D8: `moai-hook-http` crate — PreToolUse/PostToolUse/SessionStart 3개 이벤트 POST 수신 + response (updatedInput 포함)
- D9: swift-bridge 셋업 + Swift 에서 Rust `RustCore::start_workspace` 호출 성공
- D10: Go/No-Go 결정

**Go/No-Go 기준:**
- ✅ Xcode 빌드 성공 (Ghostty + Rust xcframework)
- ✅ GhosttyKit 단일 터미널 표시
- ✅ `claude --bare -p --output-format stream-json` 양방향 통신
- ✅ IDE MCP server 127.0.0.1 + lockfile + Claude auto-connect
- ✅ `mcp__moai__echo` tool 왕복
- ✅ http hook (PreToolUse/PostToolUse/SessionStart) POST 수신 + `updatedInput` rewrite 적용
- ✅ swift-bridge FFI 양방향 호출

---

## 10. Pre-M0 검증 스파이크 (3-4일)

M0 투자 전에 3가지 위험 요소를 검증한다.

### Day 1 — Claude CLI 공식 경로 검증

- `claude --bare -p "Hello" --output-format stream-json --verbose` 수동 실행
- stdin 에 `SDKUserMessage` JSON 전달 → stdout 파싱 확인
- `--settings`, `--mcp-config`, `--agents`, `--tools` (~~`--allowedTools`~~ [SPIKE ERRATA E2]), `--permission-mode` 플래그 동작 확인
- `--include-partial-messages` 로 델타 스트리밍 확인

### Day 2 — IDE MCP Server Pattern 복제 (Python prototype)

- 간단한 Python FastAPI 또는 Flask 서버로 127.0.0.1:<random> 바인드
- `~/.claude/ide/<port>.lock` drop
- `claude --mcp-config <our-config>.json -p "list files"` 실행
- Claude 가 Python 서버에 auto-connect 확인
- `mcp__ide__getDiagnostics` 등 tool call 왕복 관찰

### Day 3 — Plugin `http` hook 검증

- Minimal `.claude-plugin/plugin.json` 작성
- `hooks.json` 에 PreToolUse http type hook 1개 선언
- Python 로컬 HTTP 서버 (Flask) 가 POST 수신
- `claude -p "write hello.txt"` 실행 후 PreToolUse HTTP POST 수신 확인
- Response 로 `{"hookSpecificOutput": {"updatedInput": ...}}` 반환 → Claude 가 적용 여부 확인

### Day 4 — GhosttyKit.xcframework

- Ghostty clone
- `zig build -Demit-xcframework=true`
- 최소 Xcode 프로젝트에 xcframework 링크
- `import GhosttyKit` + 단일 터미널 view 렌더
- `zsh` spawn 해서 표시

**Output**: Go/No-Go 보고서 + `mcp__moai__*` tool 카탈로그 초안 + hook 응답 schema 검증 결과.

---

## 11. 성능 / 안정성 / 보안

### 11.1 성능 목표

- Hook callback latency (http loopback): < 10ms P95
- IDE MCP tool 호출 → UI 업데이트: < 30ms
- SDK stream-json parsing: 16 workspace × 50 msg/sec 지속 가능
- Rust core ↔ Swift FFI: < 1ms per call
- Tree-sitter incremental parse: 1MB < 100ms
- Terminal: 60fps@4K
- rusqlite batch insert: 100 rows / 100ms

### 11.2 안정성 — Rust Tokio Actor Supervision

```rust
// moai-supervisor/src/lib.rs (개념)
pub struct RootSupervisor {
    workspaces: DashMap<WorkspaceId, WorkspaceHandle>,
    ide_server: Arc<IdeServerHandle>,
    hook_http: Arc<HookHttpHandle>,
    store: Arc<Store>,
    update_manager: UpdateManager,
}

impl RootSupervisor {
    pub async fn spawn_workspace(&self, cfg: WorkspaceConfig) -> Result<WorkspaceId> {
        let (tx, rx) = mpsc::channel(256);
        let actor = WorkspaceActor::new(cfg, self.store.clone());
        let handle = tokio::spawn(async move {
            if let Err(e) = actor.run(rx).await {
                tracing::error!("workspace actor crashed: {e}");
                // one_for_one — 이 actor 만 재시작 (supervisor 정책에 따라)
            }
        });
        let id = actor.id;
        self.workspaces.insert(id, WorkspaceHandle { tx, handle });
        Ok(id)
    }
}
```

**재시작 복구:** Rust core 가 `workspaces.status == 'running'` 행에 대해 `claude --resume <session_id>` 로 세션 복구 시도.

### 11.3 보안

- **IDE lockfile**: `~/.claude/ide/<port>.lock` 0600, 디렉토리 0700
- **Auth token**: 32-byte hex via `ring`, Keychain 에 저장
- **IDE MCP server**: 127.0.0.1 bind only
- **Hook HTTP endpoint**: 127.0.0.1 + `X-Auth-Token` 헤더 검증 (same token 재사용)
- **Rust memory safety**: borrow checker 로 core 대부분 검증
- **macOS sandbox**: App Sandbox entitlements 최소화
- **Auto update**: EdDSA 서명 (Sparkle)
- **`.moai/config/sections/security.yaml` forbidden_keywords**: PreToolUse hook 에서 Bash 명령에 적용 (moai-adk 기존 기능 재사용)

### 11.4 Privacy

- **기본 0 telemetry**
- **크래시 리포트 opt-in** (Settings > Privacy)
- **Analytics opt-in**
- **네트워크 표시**: 상태 바에 outbound 연결 아이콘
- **MIT license + 전체 오픈 소스**

---

## 12. 테스트 전략

| 레벨 | 도구 | 대상 |
|---|---|---|
| Rust unit | `cargo test` | moai-core 전 crate |
| Rust integration | `cargo test --features mock-claude` | Mock Claude subprocess, stream-json codec, IDE MCP, hook HTTP roundtrip |
| Swift unit | Swift Testing | UI 로직, ViewModel |
| UI snapshot | XCUITest + swift-snapshot-testing | Sidebar, Kanban, Agent Run, Code Viewer |
| E2E | AppleScript / Robot | "이슈 → plan → run → sync → PR" 전체 플로우 |
| Stress | 자체 harness | 16 workspace × 30분, mock Claude flood |
| Claude 호환성 매트릭스 | GitHub Actions | `claude` v2.2.x / v2.3.x / nightly |

### 12.1 Mock Claude Subprocess

M0 에서 만들어야 할 핵심 인프라:
- 실제 `claude` 바이너리 없이 stream-json 프로토콜 에뮬레이트
- 임의의 hook event 를 http hook endpoint 에 주입
- 27 이벤트 fixture
- `mcp__ide__*` tool 호출 왕복 검증
- Permission dialog 라운드트립 검증

---

## 13. 경쟁 포지셔닝 (v4 최종)

| 축 | cmux | Warp | Wave | Zed | Ghostty | **MoAI Studio v4** |
|---|---|---|---|---|---|---|
| 터미널 엔진 | libghostty | 자체 Rust | Chromium | GPUI | 자체 | **libghostty** |
| UI 레이어 | Swift+AppKit | 자체 Rust UI | Electron | GPUI | AppKit | **Swift+AppKit** |
| Core | Swift | Rust | Go+TS | Rust | Zig | **Rust** |
| 라이선스 | GPL-3.0 | 폐쇄 | Apache-2.0 | 혼합 OSS | MIT | **MIT** |
| Claude 통합 | teammate shim | cloud agents | badge rollup | ACP (hook 미지원) | 없음 | **`--mcp-config` SSE/Streamable HTTP** |
| Hook 이벤트 노출 | OSC 만 | ❌ | badge | ❌ | ❌ | **18~25 events full-bidirectional** |
| LSP 통합 | ❌ | ❌ | ❌ | ✅ 자체 | ❌ | **✅ `.lsp.json` plugin (무료)** |
| Kanban / SPEC 보드 | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| Memory Viewer | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| InstructionsGraph | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| @MX 태그 거터 | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| TRUST 5 게이지 | ❌ | ❌ | ❌ | ❌ | ❌ | **✅** |
| Native permission dialog | TUI | TUI | TUI | 부분 | N/A | **✅ SwiftUI 모달** |
| In-app Claude UI 조작 | ❌ | ❌ | ❌ | ❌ | ❌ | **✅ `mcp__moai__*`** |
| Tool input rewriting | ❌ | ❌ | ❌ | ❌ | ❌ | **✅ `PreToolUse.updatedInput`** |
| OS 지원 | macOS | 3-OS | mac/linux | 3-OS | mac/linux | **macOS 전용** |

### 13.1 MoAI Studio 의 7가지 moat

1. **`--mcp-config` SSE/Streamable HTTP 로 커스텀 MCP 도구 노출** — 공식 경로, wire format 안정
2. **Hook 18~25 이벤트 양방향 노출 + tool input rewriting** — 경쟁사 0
3. **LSP as plugin feature** (`.lsp.json`) — 자체 LSP 구현 없이 6개 언어 진단
4. **Kanban + Memory + InstructionsGraph 3종** — 경쟁사 0
5. **@MX 태그 거터 + TRUST 5 게이지** — moai-adk 독점
6. **In-app Claude UI 조작** (`mcp__moai__*`) — Claude 가 직접 UI 운전
7. **Native permission dialog + updatedPermissions** — Zed 부분 구현을 완성도 높게

---

## 14. 열린 결정 사항 (v4)

### O1. MCP 서버 Rust 라이브러리 선택 — **RESOLVED (2026-04-12)**

**결정**: **`rmcp` + `axum` (Streamable HTTP transport)** 로 확정.

| 항목 | 값 |
|---|---|
| 공식 SDK | `rmcp` crate (4.7M+ downloads, MCP 공식 Rust SDK) |
| Features | `server`, `macros`, `transport-streamable-http-server`, `transport-sse-server` |
| HTTP 런타임 | `axum` 0.8 (rmcp 가 내부 transport 로 사용) |
| 진입점 위치 | `core/crates/moai-ide-server/src/lib.rs` |
| Tool 정의 | `#[tool_router]` / `#[tool]` 매크로 |

**선택 근거:**

1. **공식 SDK**: `rmcp` 는 modelcontextprotocol/rust-sdk 의 공식 crate. MCP 스펙 변경 시 자동 반영
2. **Spike 결과 정합**: Pre-M0 spike (2026-04-11) 에서 `--mcp-config` SSE 경로가 PRIMARY 통합으로 확정됨. rmcp 는 SSE + Streamable HTTP 를 모두 지원하여 이 요구를 정확히 충족
3. **Streamable HTTP 우선**: MCP 2025-03-26 스펙에서 Streamable HTTP 가 SSE 를 대체. rmcp 가 두 transport 를 동시 지원하므로 SSE (backward-compat) + Streamable HTTP (primary) 동시 운용 가능
4. **axum 이 내장 transport**: rmcp 의 `transport-streamable-http-server` feature 가 내부적으로 axum 을 사용하므로 "rmcp vs axum" 은 원래 잘못된 이분법이었음. rmcp 가 axum 위에 MCP 레이어를 얹는 구조
5. **보일러플레이트 제거**: `#[tool_router]` + `#[tool]` 매크로로 MCP 프로토콜 (initialize, tools/list, notifications/*) 자동 처리. jsonrpsee 수동 구현 대비 ~2000 줄 감소
6. **보안 기본 탑재**: rmcp 가 `Origin` 헤더 검증, DNS rebinding 방지, `Mcp-Session-Id` 세션 관리를 내장

**Cargo.toml:**

```toml
[dependencies]
rmcp = { version = "0.9", features = [
    "server",
    "macros",
    "transport-streamable-http-server",
    "transport-sse-server",  # backward-compat
]}
axum = "0.8"
tokio = { version = "1", features = ["full"] }
```

**폐기 후보**: ~~`axum + jsonrpsee` 수동 구현~~ (보일러플레이트 ~2000 줄, MCP 스펙 변경 직접 추적 부담), ~~`rust-mcp-sdk` / HyperServer~~ (커뮤니티, 공식 SDK 선호), ~~`FastRMCP`~~ (비공식, 안정성 검증 부족)

### O2. swift-bridge vs uniffi-rs — **RESOLVED (2026-04-11)**

**결정**: **swift-bridge** 로 확정.

| 항목 | 값 |
|---|---|
| 라이브러리 | `swift-bridge` |
| FFI 정의 위치 | `core/crates/moai-ffi/src/lib.rs` |
| 코드 생성 | Rust `#[swift_bridge::bridge]` → Swift 네이티브 API 자동 생성 |

**선택 근거:**

1. **Single-target 최적화**: macOS 영구 단독 전략이므로 Kotlin/Python 코드젠이 필요 없음. uniffi-rs 의 다중 타겟 장점이 오버킬
2. **Rust-first 워크플로우**: `#[swift_bridge::bridge]` mod 에 Rust 타입을 선언하면 Swift 친화적 API (Optional, throws, async) 가 자동 생성 — UDL 이중 유지보수 불필요
3. **async/Result 네이티브 매핑**: `async fn` → Swift `async throws`, `Result<T,E>` → Swift `throws` 자동 변환. cbindgen 의 수동 래핑 대비 FFI 보일러플레이트 90% 감소
4. **DESIGN.v4 §3 정합**: moai-ffi crate 가 "유일한 FFI 경계" 로 이미 설계됨

**폐기 후보**: ~~uniffi-rs~~ (UDL 이중 관리, macOS 단독에서 과잉), ~~cbindgen~~ (수동 메모리/String/Option 관리, async 미지원)

### O3. 미문서화 hook 필드 사용 정책

- `updatedPermissions` / `watchPaths` / `updatedMCPToolOutput` 는 공식 미문서화
- **권장**: Feature flag (`moai_core::features::EXPERIMENTAL_HOOK_OUTPUTS`) 로 감싸서 try, 실패 시 graceful degradation
- 사용 시 `tracing::warn!` 로 로그

### O4. Plugin 자동 설치 동의 UX

- 처음 MoAI Studio 실행 시 `~/.claude/plugins/moai-studio@local/` 자동 drop
- settings.json 의 `enabledPlugins` 에 `moai-studio@local: true` 추가
- **Onboarding 화면에서 명시적 체크박스** 로 사용자 동의
- 동의 없이 스킵 시: MoAI Studio 는 IDE MCP server 만 작동 (hook 브리징 비활성화)

### O5. `claude` 바이너리 버전 pinning 정책

- **권장**: 최소 버전만 지정 (`claude >= 2.2.0`)
- Breaking change 감지 시 `claude >= 2.2.0 < 2.3.0` 형태로 상한 추가
- MoAI Studio 가 `claude --version` 을 M0 초기에 확인, 미설치 시 설치 가이드

### O6. 브랜딩 최종 확정 — **RESOLVED (2026-04-11)**

**결정**: **MoAI Studio** 로 확정.

| 항목 | 값 |
|---|---|
| 브랜드 | **MoAI Studio** |
| 패키지/바이너리 식별자 | `moai-studio` |
| 앱 번들 | `MoAI Studio.app` |
| 배포 DMG | `MoAI-Studio.dmg` |
| Plugin 디렉토리 | `~/.claude/plugins/moai-studio@local/` |
| GitHub (장래) | `modu-ai/moai-studio` (현재 `modu-ai/moai-cli`, rename 예정) |
| 저장소 디스크 경로 (장래) | `~/moai/moai-studio` (현재 `~/moai/moai-cli`, rename 예정) |

**선택 근거:**

1. **업계 계보 연결**: Visual Studio / Android Studio / RStudio 와 동일한 "Studio" 접미사가 "이것은 CLI 가 아니라 종합 작업 환경이다" 를 0.1 초 만에 전달. "에이전틱 코딩 종합 툴" 이라는 포지셔닝과 정면 정렬
2. **"cli" 철거**: 실제로는 10개 surface 를 가진 macOS GUI 앱이므로 `moai-cli` 의 `cli` 접미사는 사실과 정면 충돌, 사용자 혼란 유발
3. **"shell" 철거**: Terminal surface 는 전체의 10% 에 불과, "moai-shell" 은 과소 대표
4. **MoAI 패밀리 역할 분리**: `moai-adk` (Go CLI, 엔진) ↔ **MoAI Studio** (macOS GUI, 플래그십) — 이름만으로 사용자 진입점이 명확
5. **Anthropic 브랜딩 제약 준수**: "Claude Code" 명칭과 0 충돌, "Powered by Claude" 서브라인 자연스러움
6. **도메인/식별자 확보 가능성**: `moai.studio`, `moaistudio.app`, `moai-studio` npm/cargo 네이밍 경쟁 낮음

**폐기 후보**: ~~`moai-cli`~~ (GUI 인데 CLI 표기), ~~`moai-ide`~~ (기술 서술어, 브랜드 아님), ~~`moai-shell`~~ (10개 surface 중 1개만 대표), ~~`Moai Studio`~~ (대소문자 미통일, `MoAI` 로 확정)

**일괄 치환 수행**: 2026-04-11, 본 결정에 따라 README/DESIGN.v4/NEXT-STEPS/REFERENCES/.moai/project/\*.md 전체에 `moai-cli` → `MoAI Studio` (브랜드) 및 `moai-studio` (식별자) 치환 완료. 저장소 디스크 rename 과 GitHub URL rename 은 별도 작업으로 보류.

---

## 15. 다음 액션

### 15.1 즉시 실행 가능 (3-4일)

§10 Pre-M0 spike 착수. 4일 후 Go/No-Go 결정.

### 15.2 M0 착수 (검증 완료 후 2주)

§9.1 상세 분해대로.

### 15.3 열린 결정 인터뷰 병렬 진행

O1 (Pre-M0 에 의존), O2, O3, O4, O5, O6 는 독립적으로 결정 가능.

### 15.4 커뮤니티 신호

- `modu-ai/moai-adk` README 에 MoAI Studio v4 로드맵 섹션 추가
- M4 완료 시점에 HN 예고
- cmux 팀에 friendly outreach (라이선스 차이로 코드 공유 불가하나 아이디어 공유 가능)

### 15.5 SPIKE ERRATA (2026-04-11)

Pre-M0 spike 수행 중 발견된 6가지 정정 사항. 상세 근거는 `spike-report.md` 참조.

| # | 분류 | 요약 | 영향 범위 |
|---|---|---|---|
| E1 | **CRITICAL** | PRIMARY 통합 경로 변경: lockfile + WS → `--mcp-config` SSE/Streamable HTTP. lockfile WS 패턴은 커스텀 도구(`mcp__moai__*`) 노출에 부적합 | §0 피벗 2, §4, §13, §16 |
| E2 | 명령행 | `--allowedTools` → `--tools`. `--allowedTools` 는 additive permission list 이므로 도구 집합 제한 불가 | §0, §4.1 spawn 코드 |
| E3 | 인증 | `--bare` 는 OAuth/Keychain 을 비활성화하므로 `ANTHROPIC_API_KEY` 환경 변수 필수 | §0, §4.1 |
| E4 | 환경 변수 | `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1` (subprocess 기본값) 이 `--permission-mode` 를 `default` 로 강제 리셋. `=0` 으로 설정 필요 | §4.1 spawn 코드 |
| E5 | Plugin 구조 | hooks.json 위치: plugin 루트가 아니라 `hooks/hooks.json`. plugin.json 에 `"hooks"` 필드 불필요 (convention-based discovery). hooks.json 최상위에 `{"hooks": {...}}` wrapper 필요 | §4.3, §7.4 |
| E6 | Hook 응답 | `hookSpecificOutput` 응답에 `hookEventName` 필드를 포함하지 않는다. 올바른 형식: `{"hookSpecificOutput": {"permissionDecision": "allow", "updatedInput": {...}}}` | §4.3 응답 plane |

---

## 16. Executive Summary (한 페이지)

### 제품
**MoAI Studio**: moai-adk 의 공식 macOS 네이티브 IDE-쉘. Claude Code 를 subprocess 로 호스트. MIT 라이선스. **macOS 영구 단독**.

### 3가지 핵심 피벗 (v3 → v4)
1. **"SDK 임베드" → "subprocess 호스트"** (공식 Agent SDK 조차 subprocess 사용)
2. **`--mcp-config` SSE/Streamable HTTP Pattern** 을 PRIMARY (lockfile + WS 는 보조 IDE 코드 인텔리전스 전용)
3. **Rust core + Swift UI** (macOS 단독이지만 Rust 의 actor supervision / stream-json 성능 / 메모리 안전성 이득)

### 핵심 아키텍처
- **Swift UI**: SwiftUI + AppKit + GhosttyKit + SwiftTreeSitter + `Down` + `Sparkle`
- **Rust Core**: tokio + axum + rusqlite + git2 + notify + `swift-bridge`
- **Claude 통합**: `claude --bare -p --output-format stream-json --mcp-config ...`
- **PRIMARY MCP 통합**: `--mcp-config` SSE/Streamable HTTP 서버 → `mcp__moai__*` 커스텀 도구 노출
- **보조 IDE MCP**: 127.0.0.1:<random> + `~/.claude/ide/<port>.lock` + bearer token → IDE 코드 인텔리전스
- **Hook 브리징**: plugin `http` hook type → `axum` endpoint

### 7가지 moat
1. `--mcp-config` SSE/Streamable HTTP 로 커스텀 MCP 도구 노출
2. Hook 18~25 이벤트 양방향 노출 + tool input rewriting
3. LSP as plugin feature (`.lsp.json`)
4. Kanban + Memory + InstructionsGraph
5. @MX 태그 거터 + TRUST 5 게이지
6. In-app Claude UI 조작 (`mcp__moai__*`)
7. Native permission dialog + updatedPermissions

### 브랜딩 제약 (Anthropic 공식)
- ❌ "Claude Code" 명칭 금지
- ✅ "Claude Agent", "Claude", "Powered by Claude" 허용
- ❌ claude.ai OAuth 금지
- ✅ API key / Bedrock / Vertex / Foundry 만

### 일정
- Pre-M0 spike: 3-4일
- M0-M6: **19주**
- 총: **20주**

### 즉시 실행 (3-4일 Pre-M0)
1. `claude --bare -p --output-format stream-json` 공식 경로 수동 검증
2. IDE MCP Server Pattern 복제 (Python prototype)
3. Plugin `http` hook type 검증
4. GhosttyKit.xcframework 빌드

---

**Version**: 4.0.1 (SPIKE ERRATA 6건 적용)
**Status**: Draft for review (spike errata applied)
**Last Updated**: 2026-04-11
**Authors**: GOOS + Claude (Opus 4.6)
**Supersedes**: DESIGN.v3.md, DESIGN.md
**Referenced research**: R1, B1, B2, B3, B4, B5
**Platform scope**: macOS 영구 단독 (Linux/WSL/Windows 로드맵 없음)

**리뷰 요청**: 형님이 v4 를 승인하시면 §10 의 4일 Pre-M0 spike 즉시 착수. spike 결과로 O1, O3, O5 답변 확보 후 M0 2주 스프린트 개시.
