# SPEC-M0-001: M0 킥오프 — Proof of Life

---
id: SPEC-M0-001
version: 1.0.0
status: draft
created: 2026-04-11
updated: 2026-04-11
author: MoAI (manager-spec)
priority: High
issue_number: null
---

## HISTORY

| 버전 | 날짜 | 변경 내용 |
|------|------|-----------|
| 1.0.0 | 2026-04-11 | 초안 작성. SPEC-SPIKE-001 결과 반영, spike errata E1-E6 통합 |

---

## 1. 개요

MoAI Studio 의 "Proof of Life" 마일스톤. Rust core 스켈레톤 + Swift UI shell + 첫 Hook round-trip 을 달성하여 전체 아키텍처의 end-to-end 실현 가능성을 검증한다.

**성공 기준**: Swift UI 창 표시 -> "New Workspace" 버튼 클릭 -> Claude subprocess spawn -> MCP 자동 연결 (`--mcp-config`) -> 사용자 메시지 전송 -> assistant 응답 스트리밍 수신 -> 화면 표시.

**선행 조건**: SPEC-SPIKE-001 GO 판정 완료 (2026-04-12).

**참조 문서**:
- `DESIGN.v4.md` 3.0-4.6, 7.1-7.3, 9.1
- `NEXT-STEPS.md` 2
- `SPEC-SPIKE-001/spike-report.md` (검증된 기술 가정)

---

## 2. 확인된 제약 사항 (Spike Errata)

SPEC-SPIKE-001 에서 검증된 6건의 errata 는 M0 전 요구사항에 반영된다.

| # | 분류 | 제약 | 적용 대상 |
|---|------|------|-----------|
| E1 | CRITICAL | PRIMARY 통합: `--mcp-config` SSE/Streamable HTTP. lockfile+WS 는 커스텀 도구 노출 불가 | RG-M0-5, RG-M0-6 |
| E2 | 명령행 | 도구 집합 제한에 `--tools` 사용. `--allowedTools` 는 additive permission list | RG-M0-3 |
| E3 | 인증 | `--bare` 모드는 OAuth/Keychain 비활성화. `ANTHROPIC_API_KEY` 환경 변수 필수 | RG-M0-3 |
| E4 | 환경 변수 | `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=0` 설정 필수. 미설정 시 `--permission-mode` 가 `default` 로 강제 리셋 | RG-M0-3 |
| E5 | Plugin 구조 | `hooks.json` 위치: `hooks/hooks.json` (plugin 루트 아님). `{"hooks": {...}}` wrapper 필수 | RG-M0-7 |
| E6 | Hook 응답 | `hookSpecificOutput` 에 `hookEventName` 필드 포함 금지. 올바른 형식: `{"hookSpecificOutput": {"permissionDecision": "allow", "updatedInput": {...}}}` | RG-M0-7 |

---

## 3. 요구사항 그룹 (EARS 형식)

### RG-M0-1: Xcode 프로젝트 + Rust Cargo Workspace 생성

**[Ubiquitous]** MoAI Studio 빌드 시스템은 `app/` (Xcode), `core/` (Rust workspace), `plugin/`, `vendor/`, `scripts/`, `tests/` 디렉토리 구조를 **유지해야 한다** (shall maintain).

**[Ubiquitous]** Rust cargo workspace 는 11개 crate member 를 **포함해야 한다** (shall include):
- `moai-core` (facade)
- `moai-supervisor`
- `moai-claude-host`
- `moai-stream-json`
- `moai-ide-server`
- `moai-hook-http`
- `moai-store`
- `moai-git`
- `moai-fs`
- `moai-plugin-installer`
- `moai-ffi`

**[Ubiquitous]** Xcode 프로젝트는 `MoAI Studio.app` bundle identifier 로 macOS 14+ 를 타겟으로 **빌드되어야 한다** (shall build).

**산출물**: `app/MoAI Studio.xcodeproj`, `core/Cargo.toml` (workspace), 11개 crate skeleton (`cargo check` 통과)

---

### RG-M0-2: Ghostty Submodule + GhosttyKit.xcframework 빌드 자동화

**[Event-Driven]** `scripts/build-ghostty-xcframework.sh` 가 실행되면 (When), 빌드 시스템은 `vendor/ghostty` 에서 `zig build -Demit-xcframework=true -Doptimize=ReleaseFast` 를 수행하고 `GhosttyKit.xcframework` 를 `app/Frameworks/` 에 **복사해야 한다** (shall copy).

**[Ubiquitous]** `vendor/ghostty` 는 git submodule 로 **관리되어야 한다** (shall be managed).

**[If-Then]** Metal Toolchain 이 설치되지 않은 환경에서 빌드가 실패하면 (If), 빌드 스크립트는 Metal Toolchain 설치 안내 메시지를 출력하고 비정상 종료 코드를 **반환해야 한다** (shall return).

**산출물**: `vendor/ghostty` (submodule), `scripts/build-ghostty-xcframework.sh`, `GhosttyKit.xcframework` 빌드 성공

**RG-4 조건**: Xcode 환경에서 Metal Toolchain 설치 후 재검증 필요 (spike-report.md 참조)

---

### RG-M0-3: moai-claude-host — Claude Subprocess Spawn

**[Event-Driven]** workspace 가 시작을 요청하면 (When), `moai-claude-host` 는 다음 인자로 Claude subprocess 를 **spawn 해야 한다** (shall spawn):
```
claude --bare -p "" \
  --output-format stream-json \
  --include-partial-messages \
  --verbose \
  --permission-mode acceptEdits \
  --settings <workspace_settings_path> \
  --mcp-config <moai_mcp_config_path> \
  --tools "Read,Edit,Write,Bash,Glob,Grep,mcp__moai__*"
```

**[Ubiquitous]** subprocess 환경에 `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=0` 을 **설정해야 한다** (shall set). [Errata E4]

**[Ubiquitous]** subprocess 환경에 `ANTHROPIC_API_KEY` 를 **전달해야 한다** (shall pass). [Errata E3]

**[Ubiquitous]** 도구 집합 제한에 `--tools` 플래그를 **사용해야 한다** (shall use). `--allowedTools` 를 사용하지 않는다. [Errata E2]

**[Event-Driven]** subprocess 의 stdout 에 NDJSON 메시지가 수신되면 (When), `moai-claude-host` 는 각 줄을 `\n` 구분자로 파싱하여 `SDKMessage` 로 **디코딩해야 한다** (shall decode).

**[Event-Driven]** 사용자 메시지 전송이 요청되면 (When), `moai-claude-host` 는 stdin 을 통해 `SDKUserMessage` JSON 을 subprocess 에 **전송해야 한다** (shall send).

**[If-Then]** subprocess 가 비정상 종료하면 (If), `moai-claude-host` 는 오류 이벤트를 EventBus 에 발행하고 workspace 상태를 `error` 로 **전환해야 한다** (shall transition).

**산출물**: `core/crates/moai-claude-host/src/lib.rs`, `cargo test --package moai-claude-host` 통과

---

### RG-M0-4: moai-stream-json — SDKMessage Codec

**[Ubiquitous]** `moai-stream-json` 은 spike RG-1 에서 확인된 13개 SDKMessage 타입을 **파싱할 수 있어야 한다** (shall parse):
- `system/init`, `system/hook_started`, `system/hook_response`
- `assistant` (text, tool_use, thinking content blocks)
- `user` (tool_result)
- `rate_limit_event`
- `result/success`
- `stream_event/message_start`, `stream_event/content_block_start`, `stream_event/content_block_delta`, `stream_event/content_block_stop`, `stream_event/message_delta`, `stream_event/message_stop`

**[Ubiquitous]** 메시지 구분자는 `\n` (NDJSON) 을 **사용해야 한다** (shall use).

**[Ubiquitous]** 코덱은 `serde` 기반 zero-copy deserialization 을 **지원해야 한다** (shall support).

**[Event-Driven]** 알 수 없는 메시지 타입이 수신되면 (When), 코덱은 해당 메시지를 `Unknown` variant 로 보존하고 `tracing::warn!` 을 **기록해야 한다** (shall log).

**산출물**: `core/crates/moai-stream-json/src/lib.rs`, spike-report.md 의 fixture 기반 단위 테스트 통과

---

### RG-M0-5: moai-ide-server — MCP Server (rmcp + Streamable HTTP)

**[Event-Driven]** Rust core 가 초기화되면 (When), `moai-ide-server` 는 `127.0.0.1:<random_high_port>` 에 Streamable HTTP + SSE backward-compat MCP 서버를 **바인드해야 한다** (shall bind). [Errata E1]

**[Ubiquitous]** MCP 서버는 `rmcp` crate 의 `#[tool_router]` / `#[tool]` 매크로를 사용하여 MCP 프로토콜을 **구현해야 한다** (shall implement).

**[Ubiquitous]** MCP 서버는 `mcp__moai__echo` 디버그 도구를 **노출해야 한다** (shall expose). 이 도구는 입력 문자열을 그대로 반환한다.

**[Ubiquitous]** 인증 토큰은 32-byte hex (`ring` crate) 로 생성되어야 하며, MCP 요청에 bearer token 검증을 **적용해야 한다** (shall enforce).

**[Event-Driven]** `--mcp-config` 에 등록된 Claude subprocess 가 연결을 시도하면 (When), MCP 서버는 `initialize` + `tools/list` 핸드셰이크를 완료하고 `mcp__moai__echo` 를 도구 목록에 **반환해야 한다** (shall return).

**산출물**: `core/crates/moai-ide-server/src/lib.rs`, mock client 연결 + 핸드셰이크 테스트 통과

---

### RG-M0-6: Claude <-> moai-ide-server 연결 검증

**[Event-Driven]** Claude subprocess 가 `--mcp-config` 로 MoAI MCP 서버 주소를 전달받으면 (When), Claude 는 해당 서버에 연결하고 `mcp__moai__echo` 도구를 인식 가능한 상태로 **등록해야 한다** (shall register). [Errata E1]

**[Complex]** MCP 서버가 실행 중인 상태에서 (While), Claude subprocess 에 `mcp__moai__echo` 를 호출하는 프롬프트가 전달되면 (When), 시스템은 MCP 도구 호출 -> 서버 처리 -> 결과 반환의 full round-trip 을 **완료해야 한다** (shall complete).

**산출물**: integration test (실제 Claude 또는 mock subprocess 사용), round-trip 로그 기록

---

### RG-M0-7: moai-hook-http — Plugin HTTP Hook Receiver

**[Event-Driven]** Claude subprocess 가 PreToolUse/PostToolUse/SessionStart 이벤트를 발생시키면 (When), plugin 의 `hooks/hooks.json` 설정에 따라 HTTP POST 가 `moai-hook-http` 엔드포인트에 **수신되어야 한다** (shall be received). [Errata E5]

**[Ubiquitous]** `hooks/hooks.json` 은 `{"hooks": {...}}` wrapper 구조를 **사용해야 한다** (shall use). [Errata E5]

**[Ubiquitous]** hook HTTP 엔드포인트는 `X-Auth-Token` 헤더를 **검증해야 한다** (shall verify).

**[Event-Driven]** PreToolUse hook 이 수신되면 (When), `moai-hook-http` 는 `{"hookSpecificOutput": {"permissionDecision": "allow"}}` 형식의 응답을 **반환해야 한다** (shall respond). 응답에 `hookEventName` 필드를 포함하지 않는다. [Errata E6]

**[Event-Driven]** PreToolUse hook 에서 `updatedInput` 이 포함된 응답을 반환하면 (When), Claude 는 수정된 입력으로 도구를 **실행해야 한다** (shall execute). [Errata E6]

**산출물**: `core/crates/moai-hook-http/src/lib.rs`, `plugin/hooks/hooks.json`, mock hook event 수신 테스트 통과

---

### RG-M0-8: swift-bridge FFI Setup

**[Event-Driven]** Swift UI 에서 `RustCore` 를 초기화하면 (When), `moai-ffi` crate 의 `#[swift_bridge::bridge]` 정의를 통해 Rust 함수가 Swift 에서 **호출 가능해야 한다** (shall be callable).

**[Ubiquitous]** FFI 경계는 최소한 다음 함수를 **노출해야 한다** (shall expose):
- `RustCore::new()` -> 초기화
- `RustCore::version()` -> 버전 문자열 반환
- `RustCore::start_workspace(config)` -> workspace ID 반환
- `RustCore::send_user_message(workspace_id, message)` -> 메시지 전송

**[Ubiquitous]** `core/crates/moai-ffi/build.rs` 가 Swift 바인딩 코드를 **자동 생성해야 한다** (shall auto-generate).

**산출물**: `core/crates/moai-ffi/src/lib.rs`, `core/crates/moai-ffi/build.rs`, Swift 측 단일 함수 호출 검증

---

### RG-M0-9: Swift UI + GhosttyKit Single Terminal Surface

**[Event-Driven]** 앱이 실행되면 (When), MoAI Studio 는 SwiftUI 기반 메인 창을 **표시해야 한다** (shall display).

**[Event-Driven]** 사용자가 "New Workspace" 버튼을 클릭하면 (When), 앱은 GhosttyKit 으로 단일 터미널 surface 를 **생성해야 한다** (shall create).

**[State-Driven]** workspace 가 활성 상태인 동안 (While), 터미널 surface 는 GhosttyKit Metal 렌더링으로 zsh shell 을 **표시해야 한다** (shall render).

**[Event-Driven]** 사용자가 메시지 입력 후 전송하면 (When), 앱은 Rust core FFI 를 통해 `send_user_message` 를 호출하고, Claude subprocess 에 메시지를 **전달해야 한다** (shall relay).

**[Event-Driven]** Claude assistant 응답이 스트리밍되면 (When), 앱은 응답 텍스트를 실시간으로 **표시해야 한다** (shall display).

**산출물**: `app/Sources/` 내 SwiftUI 뷰, GhosttyKit 터미널 surface, Claude 응답 표시 UI

---

### RG-M0-10: End-to-End 통합 테스트 + M0 Go/No-Go

**[Complex]** 모든 컴포넌트가 빌드된 상태에서 (While), end-to-end 테스트가 실행되면 (When), 시스템은 다음 시퀀스를 **완료해야 한다** (shall complete):
1. Swift UI 창 표시
2. "New Workspace" 버튼 -> Claude subprocess spawn
3. MCP 서버 자동 연결 (`--mcp-config` 경유)
4. 사용자 메시지 전송
5. Assistant 응답 스트리밍 수신
6. 화면 표시

**[Ubiquitous]** M0 Go/No-Go 보고서는 다음 항목의 통과/실패를 **기록해야 한다** (shall record):
- Xcode 빌드 성공 (Ghostty + Rust xcframework)
- GhosttyKit 단일 터미널 표시
- `claude --bare -p --output-format stream-json` 양방향 통신
- `--mcp-config` MCP 서버 연결 + `mcp__moai__echo` 도구 round-trip
- HTTP hook (PreToolUse/PostToolUse/SessionStart) POST 수신 + `updatedInput` rewrite 적용
- swift-bridge FFI 양방향 호출

**산출물**: M0 completion report, integration test suite

---

## 4. 산출물 요약

| # | 산출물 | 경로 |
|---|--------|------|
| 1 | Xcode 프로젝트 | `app/MoAI Studio.xcodeproj` |
| 2 | Rust workspace (11 crate) | `core/Cargo.toml`, `core/crates/*` |
| 3 | Plugin manifest + hooks | `plugin/.claude-plugin/plugin.json`, `plugin/hooks/hooks.json` |
| 4 | Ghostty submodule | `vendor/ghostty` |
| 5 | 빌드 자동화 스크립트 | `scripts/build-ghostty-xcframework.sh`, `scripts/build-rust-xcframework.sh` |
| 6 | M0 completion report | `docs/M0-COMPLETION.md` |

---

## 5. 비기능 요구사항

| 항목 | 목표 |
|------|------|
| Rust core `cargo check` | 0 errors, 0 warnings |
| Xcode 빌드 | 0 errors |
| 단위 테스트 | `cargo test` 전체 통과 |
| Hook HTTP loopback latency | < 10ms P95 |
| MCP tool round-trip | < 100ms (M0 목표, M1 에서 30ms 이하로 최적화) |
| FFI call overhead | < 1ms per call |
| 타겟 플랫폼 | macOS 14+ (영구 단독) |

---

## 6. 리스크

| 리스크 | 확률 | 영향 | 대응 |
|--------|------|------|------|
| RG-4 Metal Toolchain 설치 실패 | 중간 | 높음 | Xcode GUI 에서 수동 설치, fallback: libghostty-spm |
| `--bare` 플래그 향후 API 변경 | 낮음 | 중간 | 최소 버전 pinning (`claude >= 2.1.101`), 호환성 매트릭스 CI |
| `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB` 동작 변경 | 낮음 | 높음 | integration test 에서 env scrub 동작 검증, CI regression test |
| `rmcp` crate API 불안정 | 중간 | 중간 | 0.9.x 버전 pin, MCP 프로토콜 레이어 abstraction |
| swift-bridge async FFI 불안정 | 중간 | 높음 | sync wrapper fallback, 최소 FFI surface |
| Xcode Beta (26.4) 프레임워크 불일치 | 중간 | 중간 | stable Xcode 15.x 사용 권장, Beta 는 개발 환경에서만 |
| Ghostty 1.3.0 이후 API 변경 | 낮음 | 낮음 | submodule 을 특정 tag 에 pin |

---

## 7. 의존성

| 외부 의존성 | 버전 | 용도 |
|-------------|------|------|
| `claude` CLI | >= 2.1.101 | subprocess host |
| `zig` | 0.15.2 | Ghostty xcframework 빌드 |
| `rmcp` | 0.9.x | MCP server SDK |
| `axum` | 0.8.x | HTTP / Streamable HTTP transport |
| `tokio` | 1.x | async runtime |
| `serde` / `serde_json` | 1.x | SDKMessage codec |
| `swift-bridge` | latest | Rust <-> Swift FFI |
| `ring` | latest | auth token 생성 |
| `git2` | latest | git 연산 |
| Xcode | 15+ | macOS app 빌드 |
| macOS | 14+ | 타겟 플랫폼 |

---

## 8. Exclusions (What NOT to Build)

M0 는 "Proof of Life" 이므로 다음을 명시적으로 제외한다:

1. **다중 Workspace 관리** — M0 에서는 단일 workspace 만 지원. `moai-supervisor` 의 multi-workspace orchestration 은 M1
2. **Pane splitting / NSSplitView** — 단일 surface 만. pane layout 은 M1
3. **Code Viewer / Markdown / Image / Browser / FileTree surface** — M0 에서는 Terminal surface 만
4. **Kanban board / Agent Run Viewer / Memory / InstructionsGraph** — M5
5. **Plugin 자동 설치 UX** — M4. M0 에서는 수동 plugin 배치
6. **LSP 통합** (`.lsp.json`) — M4
7. **rusqlite Store 본격 구현** — M0 에서는 skeleton 만. migration 과 query 는 M1
8. **Auto-update (Sparkle)** — M6
9. **16+ 에이전트 동시 세션** — M1+ stress test
10. **Native permission dialog** — M4. M0 에서는 `--permission-mode acceptEdits` 로 bypass
11. **lockfile + WS IDE 코드 인텔리전스** — M0 에서는 `--mcp-config` SSE/Streamable HTTP 만
12. **CI/CD pipeline** — M0 완료 후 M1 에서 구축
13. **cost tracking / token budget UI** — M5
14. **Linux / WSL / Windows 지원** — 영구 제외 (DESIGN.v4 1.3)
