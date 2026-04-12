# codemaps/overview.md — MoAI Studio

> **Status**: Pre-implementation placeholder.
> 코드가 아직 작성되지 않았으므로 본 codemap 은 **목표 아키텍처 스냅샷** 만 담는다.
> M0 완료 후 실제 코드 분석 결과로 교체될 예정.
>
> **Source of truth**: DESIGN.v4.md
> **브랜드**: MoAI Studio (확정)
> **작성일**: 2026-04-11

---

## 1. 시스템 경계

```
┌─────────────────────────────────────────────────────────┐
│                 MoAI Studio.app (macOS)                 │
│                                                         │
│   Swift UI Layer  ◄──── swift-bridge FFI ────►  Rust Core  │
│                                                         │
└────────────┬───────────────────────┬────────────────────┘
             │ stdin/stdout           │ HTTP loopback (127.0.0.1)
             ▼                        ▼
   ┌─────────────────────┐   ┌───────────────────────┐
   │  claude subprocess  │   │  Plugin http hooks    │
   │  (per workspace)    │   │  (Bearer X-Auth-Token)│
   └─────────────────────┘   └───────────────────────┘
             │                        │
             └──── 같은 EventBus ──────┘
                        │
                        ▼
               ┌────────────────┐
               │  Swift UI 구독  │
               └────────────────┘
```

---

## 2. 두 개의 핵심 모듈 그룹

### 2.1 Swift UI Layer (`app/Sources/`)

| 모듈 | 책임 | 핵심 경계 |
|---|---|---|
| `App/` | `@main`, AppDelegate, 라이프사이클 | 단일 entry |
| `Shell/` | Sidebar, Tabs, Splits, CommandPalette | 5단 계층의 Window/Project/Workspace/Pane 시각화 |
| `Surfaces/` | 10개 surface 독립 모듈 | Surface 간 의존 금지, 모두 EventBus 구독 |
| `Bridge/` | swift-bridge generated + Swift wrapper | **Rust 와의 유일한 출입구** |
| `Theme/` | 색상, 폰트, 디자인 토큰 | — |

### 2.2 Rust Core (`core/crates/`)

| Crate | 책임 | 외부 의존 |
|---|---|---|
| `moai-core` | facade — Swift 가 import 하는 단일 공개 API | 다른 crate 모두 |
| `moai-supervisor` | RootSupervisor + WorkspaceSupervisor (Tokio actor tree) | tokio |
| `moai-claude-host` | ClaudeSubprocessManager | tokio::process |
| `moai-stream-json` | SDKMessage codec | serde + tokio::codec |
| `moai-ide-server` | IDE MCP server (axum) + lockfile daemon | axum, jsonrpsee/rmcp |
| `moai-hook-http` | Plugin http hook receiver | axum |
| `moai-store` | rusqlite WAL store + migrations | rusqlite, r2d2, refinery |
| `moai-git` | git2 wrapper | git2 (libgit2) |
| `moai-fs` | notify wrapper | notify |
| `moai-plugin-installer` | `~/.claude/plugins/moai-studio@local/` 자동 drop | std::fs |
| `moai-ffi` | swift-bridge `#[bridge]` 정의 | swift-bridge |

---

## 3. 데이터 흐름 — Claude → Surface

```
1. Claude subprocess  --stdout-->  StreamJsonCodec
                                       │
                                       ▼
                              SDKMessage (parsed)
                                       │
                                       ▼
                                  EventBus
                                       │
              ┌────────────────────────┼────────────────────────┐
              ▼                        ▼                        ▼
       Store (rusqlite)         Swift AsyncStream         Hook subscribers
       hook_events 테이블         (Surface 구독)            (cost tracker, etc.)
                                       │
                                       ▼
                                Surface UI 업데이트
                                (Agent Run / Code Viewer / Kanban / ...)
```

---

## 4. 데이터 흐름 — Plugin Hook → MoAI Studio

```
1. Claude 가 Bash 도구 호출 시도
       │
       ▼
2. Claude → HTTP POST → 127.0.0.1:<port>/hooks/PreToolUse
       (X-Auth-Token: <bearer>)
       │
       ▼
3. moai-hook-http (axum) 수신 → 토큰 검증
       │
       ▼
4. PreToolUse handler 가 hookSpecificOutput.updatedInput 응답
       (예: rm → trash rewrite)
       │
       ▼
5. Claude 가 응답을 적용해 실제 도구 실행
       │
       ▼
6. 동시에 hook_events 테이블 + EventBus 에 publish
       │
       ▼
7. Agent Run Viewer 에 카드로 표시
```

---

## 5. 진입점 (Entry Points)

| 종류 | 위치 | 용도 |
|---|---|---|
| macOS app | `app/Sources/App/MoaiStudioApp.swift` | `@main` SwiftUI app |
| Rust facade | `core/crates/moai-core/src/lib.rs` | Swift 가 호출하는 공개 API |
| Rust FFI 정의 | `core/crates/moai-ffi/src/lib.rs` | swift-bridge `#[bridge]` mod |
| IDE MCP server | `core/crates/moai-ide-server/src/lib.rs::start_ide_server` | 127.0.0.1 bind + lockfile drop |
| Hook HTTP server | `core/crates/moai-hook-http/src/lib.rs` | axum router `/hooks/:event` |
| 빌드 진입 | `scripts/build-ghostty-xcframework.sh`, `cargo xtask build-xcframework` | xcframework 산출 |
| 헤드리스 CLI (미래) | `core/crates/moai-studio-headless` | Swift UI 없이 Rust core 만으로 동작 |

---

## 6. 외부 시스템 인터페이스

| 외부 | 인터페이스 | crate / 모듈 |
|---|---|---|
| `claude` CLI | subprocess + stream-json | `moai-claude-host` |
| Claude Code (역방향) | IDE MCP server (`mcp__moai__*`) | `moai-ide-server` |
| Claude Code plugin hooks | HTTP POST `/hooks/<event>` | `moai-hook-http` |
| moai-adk Go CLI | subprocess (`moai worktree ...`) | `moai-supervisor` (TBD) |
| git | libgit2 in-process | `moai-git` |
| 파일시스템 | `notify` 워치 | `moai-fs` |
| Ghostty | C ABI via `GhosttyKit.xcframework` | Swift 측 (`Surfaces/Terminal/`) |
| LSP 서버 | (Claude Code 가 spawn 관리) `mcp__ide__getDiagnostics` | Swift 측 (`Surfaces/CodeViewer/`) |

---

## 7. Codemap 갱신 정책

이 문서는 **placeholder** 다. M0 완료 (Rust core skeleton + Swift UI shell + Hook 왕복) 시점에 다음과 같이 분리/확장된다:

| 파일 | 자동 생성 가능 시점 |
|---|---|
| `overview.md` (이 파일) | M0 완료 후 실제 모듈 그래프로 교체 |
| `modules.md` | M1 완료 후 (각 crate / Swift module 의 공개 API) |
| `dependencies.md` | M0 완료 후 (`cargo tree` + Xcode dependency graph) |
| `entry-points.md` | M0 완료 후 (`@main`, axum routes, swift-bridge exports) |
| `data-flow.md` | M5 완료 후 (Hook event → Surface 업데이트 전체 경로) |

자동 생성 명령:
```bash
/moai codemaps              # Explore + manager-docs 가 실제 코드를 스캔
```

---

**Source of truth**: DESIGN.v4.md §3 (아키텍처) · §4 (Claude 통합) · §8 (디렉토리)
**현 상태**: Pre-M0 (코드 0 줄). 본 문서는 목표 아키텍처 청사진.
