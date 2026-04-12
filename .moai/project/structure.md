# structure.md — MoAI Studio

> **출처**: README.md, DESIGN.v4.md §3, §8, REFERENCES.md
> **상태**: M1 Complete — 12 Rust crates + SwiftUI app + GhosttyKit + 186 tests
> **브랜드**: MoAI Studio (확정)
> **패키지 식별자**: `moai-studio`
> **작성일**: 2026-04-11

---

## 1. 5단 계층 모델 (런타임 도메인)

DESIGN.v4 §3.1 의 정신 모델. UI · 데이터 · 격리 단위가 모두 이 계층 위에 정렬된다.

```
Window
 └── Project          ← git 루트 + .moai/ 감지
      └── Workspace   ← 1 claude subprocess = 1 git worktree
           ├── agent_host: claude_code_sdk | shell | tmux_cg
           ├── binds: SPEC-{DOMAIN}-{NNN}
           └── Pane    ← NSSplitView 자체 binary tree
                └── Surface  (10종 — §6 참조)
```

**핵심 원칙:**

1. **1 Workspace = 1 Claude subprocess = 1 git worktree** — 병렬 에이전트 격리 단위
2. **Tokio actor 가 워크스페이스 라이프사이클 소유** — actor crash 시 one_for_one 재시작
3. **하나의 Project 가 N 개 Workspace** — 16+ 동시 운영 가능
4. **Surface 는 Pane 의 자식, Pane 은 NSSplitView 노드** — 드래그로 임의 분할

---

## 2. 프로세스 토폴로지

DESIGN.v4 §3.2.

```
┌─────────────────────────────────────────────────────────┐
│           MoAI Studio.app (macOS, SwiftUI + Rust)       │
│                                                         │
│  ┌──────────────────────────────────┐                   │
│  │     Swift UI Layer               │                   │
│  │  • SwiftUI Shell + AppKit bridge │                   │
│  │  • Sidebar · Tabs · NSSplitView  │                   │
│  │  • 10 Surface implementations    │                   │
│  │  • GhosttyKit (Terminal surface) │                   │
│  │  • Command Palette               │                   │
│  └────────────┬─────────────────────┘                   │
│               │ swift-bridge FFI                        │
│  ┌────────────▼────────────────────────────────────┐    │
│  │          Rust Core (moai-core workspace)        │    │
│  │  • RootSupervisor (Tokio actor tree)            │    │
│  │  • WorkspaceSupervisor × N                      │    │
│  │  • ClaudeSubprocessManager                      │    │
│  │  • StreamJsonCodec (SDKMessage)                 │    │
│  │  • IdeMcpServer (axum + rmcp/jsonrpsee)         │    │
│  │  • HookHttpEndpoint                             │    │
│  │  • Store (rusqlite WAL + r2d2 pool)             │    │
│  │  • Git (git2)                                   │    │
│  │  • FsWatcher (notify)                           │    │
│  │  • PluginInstaller                              │    │
│  │  • LockfileDaemon                               │    │
│  └────────┬────────────────────────┬────────────────┘   │
│           │ stdin/stdout            │                   │
└───────────┼─────────────────────────┼───────────────────┘
            │                         │
            ▼                         ▼
┌──────────────────────┐    ┌────────────────────────┐
│  claude subprocess   │    │   Plugin http hooks    │
│  (per workspace)     │    │   POST 127.0.0.1/...   │
│  --bare -p           │───►│   X-Auth-Token         │
│  --mcp-config ...    │    │   → hook_http endpoint │
└──────────────────────┘    └────────────────────────┘
```

**채널 이중화:**

- **Event plane**: Claude → stdin/stdout → Rust `StreamJsonCodec` → EventBus
- **Hook plane**: Claude → HTTP POST → Rust `HookHttpEndpoint` → 같은 EventBus
- **UI**: EventBus subscriber (Swift `AsyncStream`)

---

## 3. 현재 저장소 트리 (설계 단계)

> 디스크상 저장소 디렉토리 이름은 현재 `moai-cli` 입니다. 브랜드 확정에 따라 장래 `moai-studio` 로 리네임 예정입니다. 아래 트리는 **실제 디스크 상태** 를 반영합니다.

```
moai-cli/                  ← 현 저장소 디렉토리 (→ moai-studio 로 리네임 예정)
├── README.md              ← 진입점
├── DESIGN.md              ← v2 (참고)
├── DESIGN.v3.md           ← v3 (참고)
├── DESIGN.v4.md           ← v4 (★ 현 기준)
├── NEXT-STEPS.md          ← 4 단계 작업 계획
├── REFERENCES.md          ← 참조 저장소 설정 가이드
├── CLAUDE.md              ← MoAI 오케스트레이션 지시
├── .gitignore
├── .mcp.json              ← MCP 서버 정의
│
├── .moai/                 ← MoAI-ADK 프로젝트 구성
│   ├── config/
│   │   ├── config.yaml
│   │   └── sections/      ← language.yaml, quality.yaml, workflow.yaml, ...
│   ├── design/
│   ├── docs/
│   ├── logs/
│   ├── plans/
│   ├── project/           ← 이 디렉토리 (product.md, structure.md, tech.md)
│   ├── reports/
│   └── state/
│
├── .agency/               ← AI Agency 구성
│   ├── config.yaml
│   ├── context/
│   ├── fork-manifest.yaml
│   └── templates/
│
├── .claude/               ← Claude Code 프로젝트 구성
│   ├── agents/
│   ├── commands/
│   ├── hooks/
│   ├── rules/
│   ├── settings.json
│   └── skills/
│
├── .references/           ← gitignored 심볼릭 링크
│   ├── moai-adk-go    →   /Users/goos/MoAI/moai-adk-go
│   └── claude-code-map →  /Users/goos/moai/claude-code-map
│
├── design-exports/        ← 12 PNG UI 목업 + v1 PDF
│
└── research/              ← 리서치 결과 (R1, B1-B5)
    ├── R1-native-ai-shells.md           (50KB, 경쟁사)
    ├── B1-bridge-direct-connect.md      (10KB, 소스 분석)
    ├── B2-hook-events-tool-system.md    (20KB, 소스 분석)
    ├── B3-extension-points.md           (24KB, 소스 분석)
    ├── B4-official-docs-verification.md (19KB, 공식 문서)
    └── B5-wsl-wslg-windows-coverage.md  (13KB, Linux 포기 근거)
```

### 디렉토리 책임

| 경로 | 책임 |
|---|---|
| `DESIGN.v4.md` | **단일 진실 출처** — 모든 아키텍처 결정의 권위 |
| `NEXT-STEPS.md` | Pre-M0 spike + M0 킥오프 + 열린 결정 + 커뮤니티 4단계 |
| `REFERENCES.md` | `.references/` 심볼릭 링크 설정/사용법 |
| `research/` | DESIGN 의 모든 주장에 대한 1차 근거 |
| `design-exports/` | UI 목업 12개 — surface 레이아웃 비주얼 사양 |
| `.references/moai-adk-go` | moai-adk Go CLI 소스 — Hook 통합/plugin 설치/27 이벤트 wiring 검증용 |
| `.references/claude-code-map` | Claude Code mapped 소스 — stream-json/SDKMessage/hook 스키마 검증용 |
| `.moai/project/` | 이 문서들 (product.md, structure.md, tech.md, codemaps/) |

---

## 4. 목표 모노레포 트리 (M0 이후)

DESIGN.v4 §8 의 목표 구조. M0 D1 에서 `app/`, `core/`, `plugin/`, `vendor/`, `scripts/`, `tests/` 디렉토리가 추가됨.

```
moai-studio/                       # 저장소 리네임 후
├── (위의 모든 설계 문서 + .moai/.claude/.agency 유지)
│
├── app/                          # macOS Xcode 프로젝트
│   ├── MoAI Studio.xcodeproj
│   ├── Sources/
│   │   ├── App/                  # @main, AppDelegate
│   │   ├── Shell/                # Sidebar, Tabs, Splits, CommandPalette
│   │   ├── Surfaces/             # 10 surfaces
│   │   │   ├── Terminal/         # GhosttyKit wrapper
│   │   │   ├── CodeViewer/       # NSTextView + SwiftTreeSitter + @MX
│   │   │   ├── Markdown/         # Down + KaTeX/Mermaid
│   │   │   ├── Image/            # Core Image + Vision SSIM
│   │   │   ├── Browser/          # WKWebView + DevTools
│   │   │   ├── FileTree/
│   │   │   ├── AgentRun/         # Hook event stream + live control
│   │   │   ├── Kanban/
│   │   │   ├── Memory/           # ~/.claude/projects/<root>/memory/ 렌더
│   │   │   └── InstructionsGraph/  # InstructionsLoaded hook 시각화
│   │   ├── Bridge/               # swift-bridge generated + wrappers
│   │   └── Theme/
│   ├── Frameworks/               # GhosttyKit.xcframework, MoaiCore.xcframework
│   └── Resources/
│
├── core/                         # Rust workspace
│   ├── Cargo.toml                # workspace manifest
│   ├── crates/
│   │   ├── moai-core/            # facade crate
│   │   ├── moai-supervisor/      # RootSupervisor, WorkspaceSupervisor
│   │   ├── moai-claude-host/     # ClaudeSubprocessManager
│   │   ├── moai-stream-json/     # SDKMessage codec
│   │   ├── moai-ide-server/      # IDE MCP server + lockfile daemon
│   │   ├── moai-hook-http/       # Plugin http hook receiver (axum)
│   │   ├── moai-store/           # rusqlite store + migrations
│   │   ├── moai-git/             # git2 wrapper
│   │   ├── moai-fs/              # notify wrapper
│   │   ├── moai-plugin-installer/  # ~/.claude/plugins/moai-studio@local/
│   │   └── moai-ffi/             # swift-bridge definitions
│   └── xtask/                    # xcframework 빌드 스크립트
│
├── plugin/                       # moai-studio-plugin
│   ├── .claude-plugin/plugin.json
│   ├── hooks.json                # 18~20 이벤트 http type
│   ├── mcp-config.json           # MoAI Studio IDE server 연결
│   ├── lsp.json                  # 6개 언어 LSP
│   ├── commands/                 # /moai-studio:kanban|memory|connect|surface
│   ├── skills/
│   ├── output-styles/            # forceForPlugin: true
│   └── agents/                   # moai-adk 26 에이전트 참조
│
├── vendor/
│   ├── ghostty/                  # submodule (https://github.com/ghostty-org/ghostty)
│   └── tree-sitter-grammars/     # submodule
│
├── scripts/
│   ├── build-ghostty-xcframework.sh
│   ├── build-rust-xcframework.sh
│   ├── install-plugin.sh         # ~/.claude/plugins/moai-studio@local/ drop
│   └── reload.sh
│
├── tests/
│   ├── rust-unit/                # cargo test
│   ├── rust-integration/         # mock claude subprocess
│   ├── swift-unit/               # Swift Testing
│   ├── swift-ui/                 # XCUITest
│   └── stress/                   # 16 workspace harness
│
└── docs/                         # 사용자 문서 (M5 이후)
```

### 디렉토리 의도

| 경로 | 단일 책임 |
|---|---|
| `app/` | SwiftUI + AppKit shell. **순수 UI**. 비즈니스 로직 금지 (모두 Rust core 호출) |
| `app/Sources/Surfaces/` | 10개 surface 각각 독립 모듈. surface 간 의존 금지 |
| `app/Sources/Bridge/` | swift-bridge 생성 코드 + Swift-side wrapper. FFI 단일 출입구 |
| `core/crates/moai-core/` | 외부 facade. Swift 가 import 하는 단일 공개 API |
| `core/crates/moai-*/` | 단일 책임 crate. 서로 간섭 최소화. 단위 테스트 격리 |
| `core/crates/moai-ffi/` | swift-bridge `#[bridge]` 정의. **유일한 FFI 경계** |
| `plugin/` | Claude Code plugin manifest. MoAI Studio 가 자동 설치 (`~/.claude/plugins/moai-studio@local/`) |
| `vendor/ghostty/` | submodule. **소스 수정 금지**. xcframework 빌드만 |
| `tests/stress/` | 16 workspace 동시 실행 + mock Claude flood |

---

## 5. 5단 계층 ↔ 디렉토리 매핑

| 계층 | Swift 위치 | Rust 위치 |
|---|---|---|
| Window | `app/Sources/App/` | — |
| Project | `app/Sources/Shell/Sidebar/` | `core/crates/moai-store/` (projects 테이블) |
| Workspace | `app/Sources/Shell/Tabs/` | `core/crates/moai-supervisor/` (WorkspaceActor) |
| Pane | `app/Sources/Shell/Splits/` (NSSplitView 자체 binary tree) | `core/crates/moai-store/` (panes 테이블) |
| Surface (10종) | `app/Sources/Surfaces/<Type>/` | `core/crates/moai-store/` (surfaces 테이블, state JSON) |

---

## 6. 데이터 저장 위치

| 데이터 | 위치 | 형식 |
|---|---|---|
| 프로젝트/워크스페이스/Pane/Surface 메타 | `~/Library/Application Support/MoAI Studio/store.db` | rusqlite WAL |
| Hook event stream | 같은 store, `hook_events` 테이블 | rusqlite WAL, 30일 TTL |
| Cost updates | 같은 store, `cost_updates` 테이블 | rusqlite WAL |
| task-metrics 미러 | 같은 store, `task_metrics_mirror` | rusqlite WAL |
| Memory (Claude Code 본 저장소) | `~/.claude/projects/<root>/memory/` | Markdown (외부 소스) |
| IDE MCP lockfile | `~/.claude/ide/<port>.lock` | JSON, 0600 |
| Auth token | macOS Keychain | — |
| Plugin install | `~/.claude/plugins/moai-studio@local/` | manifest + assets |

---

## 7. 외부 시스템과의 경계

| 경계 | 프로토콜 | 인증 |
|---|---|---|
| MoAI Studio ↔ Claude subprocess | stdin/stdout stream-json | (subprocess 격리) |
| Claude ↔ MoAI Studio IDE MCP server | WebSocket / HTTP @ 127.0.0.1:`<port>` | Bearer token (lockfile) |
| Claude plugin ↔ MoAI Studio hook endpoint | HTTP POST @ 127.0.0.1:`<port>`/hooks/`<event>` | `X-Auth-Token` header (same token) |
| MoAI Studio ↔ moai-adk Go CLI | subprocess (`moai worktree add ...`) | (CLI 인증) |
| MoAI Studio ↔ git | libgit2 (in-process) | — |
| Swift ↔ Rust | swift-bridge FFI (in-process) | — |

**중요**: 모든 네트워크 바인딩은 `127.0.0.1` 만. 외부 네트워크 접근 0.

---

## 8. 빌드 산출물 위치

| 산출물 | 위치 | 빌드 도구 |
|---|---|---|
| `GhosttyKit.xcframework` | `app/Frameworks/GhosttyKit.xcframework` | `zig build -Demit-xcframework=true` |
| `MoaiCore.xcframework` | `app/Frameworks/MoaiCore.xcframework` | `cargo xtask build-xcframework` |
| `MoAI Studio.app` | `~/Library/Developer/Xcode/DerivedData/.../Build/Products/` | Xcode |
| `MoAI-Studio.dmg` (배포) | `release/` | `notarize.sh` + `create-dmg` |

---

**Source of truth**: DESIGN.v4.md §3 (아키텍처) · §8 (디렉토리)
