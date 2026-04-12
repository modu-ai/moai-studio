# tech.md — MoAI Studio

> **출처**: DESIGN.v4.md §0, §4, §7, §11
> **상태**: Stack 확정. 일부 항목은 Pre-M0 spike 검증 후 fix.
> **브랜드**: MoAI Studio (확정)
> **패키지 식별자**: `moai-studio`
> **작성일**: 2026-04-11

---

## 1. 핵심 결정 요약

| 항목 | 결정 | 근거 |
|---|---|---|
| **Claude 통합** | **Subprocess + stream-json** (SDK 임베드 X) | [hosting](https://code.claude.com/docs/en/agent-sdk/hosting) — 공식 SDK 조차 `claude` CLI subprocess spawn |
| **PRIMARY 통합 경로** | **`--mcp-config` SSE/Streamable HTTP Pattern** | [SPIKE ERRATA E1] lockfile + WS 는 커스텀 도구 노출 부적합. `--mcp-config` 로 SSE/Streamable HTTP MCP 서버 등록이 정확한 PRIMARY 경로 |
| **Hook 통합** | **Plugin `http` hook type** | shell wrapper 불필요, 언어 무관 |
| **UI 언어** | **Swift** (SwiftUI + AppKit) | macOS 네이티브, 60fps@4K |
| **Core 언어** | **Rust** | actor supervision, stream-json 성능, 메모리 안전, FFI 부담 제한적 |
| **터미널 엔진** | **libghostty** (`GhosttyKit.xcframework`) | cmux 와 동일, Metal 60fps@4K, Sixel/iTerm2 inline image |
| **FFI 도구** | **`swift-bridge`** (권장 — Pre-M0 D1 결정) | Swift 단독 타깃 최적화. 대안: `uniffi-rs` |
| **DB** | **rusqlite + r2d2 + refinery** | WAL, batch insert 100 rows / 100ms, 고부하 검증 |
| **Git** | **git2** (libgit2) | in-process, worktree 자동화 |
| **OS 지원** | **macOS 14+ 영구 단독** | B5 리서치 결과: WSL 미지원, 9P 9배 느림, libghostty/VTE 이중화 복잡도 |
| **개발 방법론** | **TDD** (RED-GREEN-REFACTOR) | 신규 프로젝트 자동 감지 (quality.yaml `development_mode: tdd`) |
| **라이선스** | **MIT** | — |
| **브랜드** | **MoAI Studio** | DESIGN.v4 §14 O6 RESOLVED |

---

## 2. Swift Shell 스택

DESIGN.v4 §7.1.

| 컴포넌트 | 기술 | 용도 |
|---|---|---|
| UI 프레임워크 | **SwiftUI + AppKit** (macOS 14+) | Shell, Sidebar, Tabs, Splits, Surfaces |
| 터미널 | **`GhosttyKit.xcframework`** | Terminal surface (Metal 60fps@4K, PTY 소유권 = Ghostty) |
| 코드 하이라이트 | **`SwiftTreeSitter`** | Code Viewer 의 syntax highlight, 심볼 인덱스 |
| Pane splitter | **`NSSplitView`** + 자체 binary tree | Pane 분할 (cmux 패턴) |
| Markdown | **`Down`** (cmark wrapper) | Markdown surface, EARS SPEC 렌더 |
| WebView | **`WKWebView`** | Browser surface, KaTeX/Mermaid 임베드 |
| 자동 업데이트 | **`Sparkle`** 2.x | EdDSA 서명, 배포 |
| 크래시 리포트 | **Sentry-Cocoa** (opt-in) | privacy 기본 0 telemetry |

### Swift 측 사용 패턴

```swift
import MoaiCore  // swift-bridge 생성

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

## 3. Rust Core 스택

DESIGN.v4 §7.2. Crate workspace 는 `core/crates/moai-*/` 에 위치.

| 카테고리 | crate | 역할 |
|---|---|---|
| Async runtime | **`tokio`** 1.x | Actor tree, async I/O |
| Stream-json codec | 자체 (`serde_json` + `tokio::codec::Framed`) | SDKMessage 인코딩/디코딩 |
| MCP server | **`rmcp` + `axum`** (Streamable HTTP + SSE) | IDE MCP server (`mcp__moai__*`). O1 RESOLVED 2026-04-12. `#[tool_router]` / `#[tool]` 매크로 |
| HTTP endpoint | **`axum`** | Plugin `http` hook receiver |
| Database | **`rusqlite` + `r2d2` + `refinery`** | WAL store + connection pool + 마이그레이션 |
| Git | **`git2`** (libgit2 binding) | worktree, status, diff |
| File watch | **`notify`** 7.x | FsWatcher (FileTree, Markdown live) |
| Auth token | **`ring`** | 32-byte hex secure random |
| Serialization | **`serde` + `serde_json`** | SDKMessage, hook payload |
| Logging | **`tracing` + `tracing-subscriber`** | 구조화 로깅 |
| FFI to Swift | **`swift-bridge`** | Swift `import MoaiCore` 진입점 |

### Crate 분해

```
core/crates/
├── moai-core/             # facade (Swift 가 import 하는 단일 공개 API)
├── moai-supervisor/       # RootSupervisor + WorkspaceSupervisor (Tokio actor tree)
├── moai-claude-host/      # ClaudeSubprocessManager (tokio::process)
├── moai-stream-json/      # SDKMessage codec (serde + Framed)
├── moai-ide-server/       # IDE MCP server (axum) + lockfile daemon
├── moai-hook-http/        # Plugin http hook receiver (axum)
├── moai-store/            # rusqlite store + migrations
├── moai-git/              # git2 wrapper
├── moai-fs/               # notify wrapper
├── moai-plugin-installer/ # ~/.claude/plugins/moai-studio@local/ 자동 drop
└── moai-ffi/              # swift-bridge #[bridge] 정의 (유일한 FFI 경계)
```

---

## 4. Claude Code 통합 — 명령행 (확정)

DESIGN.v4 §4.1. MoAI Studio 가 spawn 하는 정확한 형태:

```bash
claude --bare -p "" \
  --output-format stream-json \
  --include-partial-messages \
  --verbose \
  --permission-mode acceptEdits \
  --settings <ws.settings_path> \
  --mcp-config <moai_studio_mcp_config_path> \
  --plugin-dir <moai_studio_plugin_dir> \
  --tools "Read,Edit,Write,Bash,Glob,Grep,mcp__moai__*,mcp__ide__*"
  # [SPIKE ERRATA E2] --allowedTools 는 additive permission list. 도구 제한에는 --tools 사용
```

### `--bare` 의 효과 ([headless](https://code.claude.com/docs/en/headless))

> "`--bare` is the recommended mode for scripted and SDK calls, and will become the default for `-p` in a future release."

비활성화 항목 (모두 명시 전달로 대체):
- `.claude/settings.json` 자동 로드 → `--settings`
- Plugins 자동 로드 → `--plugin-dir`
- MCP 서버 자동 로드 → `--mcp-config`
- CLAUDE.md 자동 로드 → `--append-system-prompt-file`
- Keychain / OAuth → `ANTHROPIC_API_KEY` 환경 변수

> [SPIKE ERRATA E3] `--bare` 는 OAuth/Keychain 인증을 완전히 비활성화. subprocess 환경에 `ANTHROPIC_API_KEY` 필수 설정.

> [SPIKE ERRATA E4] subprocess 컨텍스트에서 `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1` (기본값) 이면 `--permission-mode` 가 `default` 로 강제 리셋됨. MoAI Studio 는 `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=0` 을 subprocess 환경에 설정해야 함.

→ MoAI Studio 가 **결정론적 실행 환경** 보장.

---

## 5. MCP 통합 — PRIMARY: `--mcp-config` SSE/Streamable HTTP

> [SPIKE ERRATA E1] lockfile + WS 패턴은 커스텀 도구(`mcp__moai__*`) 노출에 부적합. PRIMARY 경로는 `--mcp-config` 로 SSE/Streamable HTTP MCP 서버를 등록하는 것이다. 아래의 lockfile + WS 메커니즘은 **보조 통합 (IDE 코드 인텔리전스)** 으로 유지.

[VS Code 통합 문서](https://code.claude.com/docs/en/vs-code) 의 lockfile 메커니즘 (보조 경로):

```
1. Rust core 가 127.0.0.1:<random_high_port> 에 axum 서버 bind
2. 32-byte hex auth token 생성 (ring)
3. ~/.claude/ide/<port>.lock 작성 (0600, 디렉토리 0700)
   {
     "workspaceFolders": [...],
     "pid": <MoAI Studio pid>,
     "ideName": "MoAI Studio",
     "transport": "ws",
     "runningInWindows": false,
     "authToken": "<hex>"
   }
4. claude --mcp-config <path> 가 lockfile 자동 스캔 → auto-connect
5. Bearer token auth header 검증
6. tools/list, tools/call RPC 처리
```

### 노출되는 IDE MCP Tool 카탈로그

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
| `mcp__moai__file_reveal_in_finder` | Finder 하이라이트 |
| `mcp__moai__memory_reveal_entry` | Memory surface entry 스크롤 |
| `mcp__moai__instructions_graph_highlight` | 그래프 하이라이트 |
| `mcp__ide__getDiagnostics` | LSP 진단 (VS Code 호환 표준) |
| `mcp__ide__openDiff` | Code Viewer tri-pane diff |
| `mcp__ide__readSelection` | 사용자 선택 영역 |

---

## 6. Hook 브리징 — Plugin `http` hook type

DESIGN.v4 §4.3. 18~20개 이벤트를 모두 plugin manifest 의 `http` hook type 으로 wire.

```json
// plugin/hooks.json (요약)
{
  "PreToolUse": [{
    "matcher": "Bash|Write|Edit",
    "hooks": [{
      "type": "http",
      "url": "http://127.0.0.1:${user_config.ide_port}/hooks/PreToolUse",
      "headers": { "X-Auth-Token": "${user_config.auth_token}" },
      "timeout": 5000
    }]
  }],
  "PostToolUse": [...],
  "SessionStart": [...],
  "SessionEnd": [...],
  "UserPromptSubmit": [...],
  "Notification": [...],
  "Stop": [...],
  "SubagentStart": [...],
  "SubagentStop": [...],
  "TaskCompleted": [...],
  "TeammateIdle": [...],
  "PermissionRequest": [...],
  "ConfigChange": [...],
  "WorktreeCreate": [...],
  "WorktreeRemove": [...],
  "PreCompact": [...],
  "PostCompact": [...],
  "FileChanged": [...],
  "InstructionsLoaded": [...],
  "CwdChanged": [...],
  "Setup": [...]
}
```

### Hook 기능 매트릭스

DESIGN.v4 §4.4 — 공식 vs 미문서화 구분.

| 기능 | Hook 이벤트 | Response 필드 | 공식 여부 |
|---|---|---|---|
| Bash safe wrapper | PreToolUse | `hookSpecificOutput.updatedInput` | ✅ |
| Path normalization | PreToolUse | `hookSpecificOutput.updatedInput` | ✅ |
| Native permission dialog | PermissionRequest | `hookSpecificOutput.decision` | ✅ |
| Always-allow 영구화 | PermissionRequest | `decision.updatedPermissions` | ⚠️ feature flag |
| File tree live update | FileChanged | (관찰만) | ✅ |
| Session context 주입 | SessionStart | `hookSpecificOutput.additionalContext` | ✅ |
| Watch 등록 | SessionStart | `hookSpecificOutput.watchPaths` | ⚠️ feature flag |
| Config hot-reload | ConfigChange | (관찰만) | ✅ |
| Instructions graph | InstructionsLoaded | (관찰만) | ✅ |
| Cost tracker | PostToolUse + cost_update | (관찰만) | ✅ |
| Worktree tracker | WorktreeCreate / WorktreeRemove | (관찰만) | ✅ |

**⚠️ 미문서화 항목** 은 `try { use feature } catch { graceful degradation }` 패턴으로 wrap. `tracing::warn!` 로 로그.

---

## 7. LSP 통합 — `.lsp.json` plugin feature

[plugins-reference](https://code.claude.com/docs/en/plugins-reference) 의 `lspServers` 필드.

```json
// plugin/lsp.json
{
  "gopls":         { "command": "gopls", "args": ["serve"], "filetypes": ["go"] },
  "rust-analyzer": { "command": "rust-analyzer", "filetypes": ["rust"] },
  "pyright":       { "command": "pyright-langserver", "args": ["--stdio"], "filetypes": ["python"] },
  "tsserver":      { "command": "typescript-language-server", "args": ["--stdio"], "filetypes": ["typescript", "javascript", "tsx", "jsx"] },
  "clangd":        { "command": "clangd", "filetypes": ["c", "cpp", "objc", "objcpp"] },
  "sourcekit":     { "command": "sourcekit-lsp", "filetypes": ["swift"] }
}
```

**Claude Code 가 LSP 서버를 spawn + 관리.** MoAI Studio Code Viewer 는 `mcp__ide__getDiagnostics` tool 호출로 진단을 받음. **자체 LSP 클라이언트 구현 불필요** — 이것이 v4 의 큰 단순화.

---

## 8. 빌드 툴체인

DESIGN.v4 §7.3.

| 도구 | 최소 버전 | 용도 |
|---|---|---|
| **Xcode** | 15+ | macOS app build, signing, notarize |
| **Rust** | 1.80+ | core/ workspace |
| **Zig** | 0.13+ | Ghostty xcframework 빌드 |
| **cargo xtask** | — | xcframework 빌드 자동화 |
| **`just`** (또는 Makefile) | — | 원스텝 `just build` |
| **swift-bridge CLI** | latest | FFI 코드 생성 |
| **`create-dmg`** | — | DMG 배포 |
| **Sparkle 2 sign tool** | — | EdDSA 자동 업데이트 서명 |

### 원스텝 빌드

```bash
# 의도된 명령 (M0 D2 자동화 예정)
just build               # 전체: ghostty xcframework + rust xcframework + Xcode archive
just test                # cargo test + swift test + XCUITest
just dmg                 # notarize + DMG 배포
```

---

## 9. moai-studio-plugin 구조

DESIGN.v4 §7.4. `plugin/` 디렉토리에 위치, `~/.claude/plugins/moai-studio@local/` 로 자동 install.

```
plugin/
├── .claude-plugin/plugin.json    # manifest (hooks 필드 없음 — convention-based discovery)
├── hooks/
│   └── hooks.json                # [SPIKE ERRATA E5] hooks/ 디렉토리에 위치, {"hooks": {...}} wrapper 필수
├── mcp-config.json               # MoAI Studio IDE server 연결
├── lsp.json                      # 6개 언어 LSP
├── commands/
│   ├── kanban.md                 # /moai-studio:kanban
│   ├── memory.md                 # /moai-studio:memory
│   ├── connect.md                # /moai-studio:connect
│   └── surface.md                # /moai-studio:surface
├── skills/
│   ├── moai-studio-open-workspace/SKILL.md
│   └── moai-studio-focus-agent/SKILL.md
├── output-styles/
│   └── moai-studio.md            # forceForPlugin: true
└── agents/                       # moai-adk 26 에이전트 참조
```

---

## 10. 데이터 모델 (rusqlite WAL)

DESIGN.v4 §6 의 핵심 테이블. 전체 schema 는 `core/crates/moai-store/migrations/v1__initial.sql`.

**주요 테이블:**

- `projects` — git 루트, `is_moai_adk` 플래그
- `workspaces` — `worktree_path`, `agent_host`, `spec_id`, `claude_session_id`, `status`
- `panes` — binary tree (`parent_id`, `split`, `ratio`)
- `surfaces` — `pane_id`, `kind` (10종), `state_json`
- `hook_events` — **v4 주 데이터 소스**, 27 이벤트, 30일 TTL
- `cost_updates` — input/output/cache 토큰 + USD
- `task_metrics_mirror` — `.moai/logs/task-metrics.jsonl` 백업 미러
- `specs` — SPEC-{DOMAIN}-{NNN}, EARS markdown
- `mx_tags` — `kind` (ANCHOR|WARN|NOTE|TODO), `path`, `line`
- `kanban_boards`, `kanban_cards` — Kanban surface 백엔드
- `notifications` — 네이티브 알림 큐

**설정**: WAL, `synchronous=NORMAL`, batch insert (100 rows / 100ms), 30일 TTL on `hook_events`/`task_metrics_mirror`.

---

## 11. 성능 목표

DESIGN.v4 §11.1.

| 항목 | 목표 |
|---|---|
| Hook callback latency (http loopback) | < 10ms P95 |
| IDE MCP tool 호출 → UI 업데이트 | < 30ms |
| SDK stream-json parsing | 16 workspace × 50 msg/sec 지속 |
| Rust core ↔ Swift FFI | < 1ms per call |
| Tree-sitter incremental parse | 1MB < 100ms |
| Terminal | 60fps@4K |
| rusqlite batch insert | 100 rows / 100ms |
| 콜드 스타트 | < 0.6s (M1 MacBook) |
| 활성 메모리 | < 700 MB (8 PTY + 4 code surface + 2 Claude subprocess) |

---

## 12. 보안

DESIGN.v4 §11.3.

- **IDE lockfile**: `~/.claude/ide/<port>.lock` 0600, 디렉토리 0700
- **Auth token**: 32-byte hex via `ring`, **macOS Keychain** 저장
- **IDE MCP server**: **`127.0.0.1` bind only** (외부 노출 금지)
- **Hook HTTP endpoint**: 127.0.0.1 + `X-Auth-Token` 헤더 검증 (same token)
- **Rust borrow checker**: core 대부분 메모리 안전 검증
- **macOS App Sandbox**: entitlements 최소화
- **Auto update**: EdDSA 서명 (Sparkle)
- **Forbidden Bash 명령**: `.moai/config/sections/security.yaml` `forbidden_keywords` 를 PreToolUse hook 에서 검사 (moai-adk 기존 기능 재사용)
- **Privacy**: 기본 0 telemetry. 크래시 리포트/Analytics 모두 opt-in. 상태 바에 outbound 연결 아이콘 표시.

---

## 13. 테스트 전략

DESIGN.v4 §12.

| 레벨 | 도구 | 대상 |
|---|---|---|
| Rust unit | `cargo test` | moai-core 전 crate |
| Rust integration | `cargo test --features mock-claude` | Mock Claude subprocess, stream-json codec, IDE MCP, hook HTTP roundtrip |
| Swift unit | **Swift Testing** | UI 로직, ViewModel |
| UI snapshot | **XCUITest + swift-snapshot-testing** | Sidebar, Kanban, Agent Run, Code Viewer |
| E2E | AppleScript / Robot | "이슈 → plan → run → sync → PR" 전체 플로우 |
| Stress | 자체 harness | 16 workspace × 30분, mock Claude flood |
| Claude 호환성 매트릭스 | GitHub Actions | `claude` v2.2.x / v2.3.x / nightly |

### Mock Claude Subprocess (M0 핵심 인프라)

- 실제 `claude` 바이너리 없이 stream-json 프로토콜 에뮬레이트
- 임의의 hook event 를 http hook endpoint 에 주입
- 27 이벤트 fixture 보유
- `mcp__ide__*` tool 호출 왕복 검증
- Permission dialog 라운드트립 검증

---

## 14. 개발 환경 — 참조 저장소

REFERENCES.md 참조. **gitignored 심볼릭 링크**, 로컬 전용:

| 심볼릭 링크 | 타깃 | 용도 |
|---|---|---|
| `.references/moai-adk-go` | `/Users/goos/MoAI/moai-adk-go` | Hook 통합, plugin 자동 설치, 27 이벤트 wiring 검증 |
| `.references/claude-code-map` | `/Users/goos/moai/claude-code-map` | stream-json 프로토콜, SDKMessage, hook 스키마, MCP 통합 검증 |

**원칙**: `.references/` 안의 파일은 **읽기 전용**. MoAI Studio 코드는 항상 저장소 루트에만 작성. CI 에서는 `.references/` 없이도 빌드 성공해야 함.

---

## 15. 열린 결정 (Pre-M0 & M0 결정 대기)

DESIGN.v4 §14.

| # | 결정 | 권장/결정 | 기한 |
|---|---|---|---|
| ~~O1~~ | ~~MCP 서버 Rust 라이브러리~~ | **RESOLVED → `rmcp` + `axum` Streamable HTTP** (2026-04-12) | — |
| ~~O2~~ | ~~swift-bridge vs uniffi-rs vs cbindgen~~ | **RESOLVED → swift-bridge** (2026-04-11) | — |
| O3 | 미문서화 hook 필드 (`updatedPermissions`, `watchPaths`) | feature flag wrap | M1 |
| O4 | Plugin 자동 설치 동의 UX | onboarding 명시 체크박스 | M4 |
| O5 | `claude` 바이너리 버전 pinning | `>= 2.2.0` 최소만 | M4 |
| O6 | ~~브랜딩 최종~~ | **RESOLVED → MoAI Studio** (2026-04-11) | — |

---

## 16. 환경 변수 / 인증

| 변수 | 용도 |
|---|---|
| `ANTHROPIC_API_KEY` | Claude API 인증 (또는 Bedrock/Vertex/Foundry 환경 변수). [SPIKE ERRATA E3] `--bare` 모드 필수 |
| `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB` | [SPIKE ERRATA E4] `0` 으로 설정 필수. 기본값 `1` 이면 `--permission-mode` 가 `default` 로 리셋됨 |
| `MOAI_STUDIO_DEV` | 개발 모드 플래그 (verbose 로깅, mock Claude) |
| `MOAI_STUDIO_IDE_PORT` | IDE MCP server port override (테스트용) |

**금지**: claude.ai OAuth 구현. Anthropic 공식 브랜딩 가이드라인.

---

**Source of truth**: DESIGN.v4.md §0 (피벗) · §4 (Claude 통합) · §7 (스택) · §11 (성능/보안) · §12 (테스트)
