# SPEC-M0-001: 구현 계획

---

## 1. 기술 접근

### 아키텍처 개요

M0 는 5개 계층의 수직 슬라이스를 구현한다:

```
Swift UI Shell (최소 SwiftUI + GhosttyKit)
       |
   swift-bridge FFI (moai-ffi)
       |
   Rust Core (moai-core facade)
       |
   ┌───┼───────────┬──────────────┐
   |   |           |              |
 moai-claude-host  moai-ide-server  moai-hook-http
   |               |
 moai-stream-json  rmcp + axum
```

### 핵심 기술 결정

- **MCP 서버**: `rmcp` + Streamable HTTP (DESIGN.v4 O1 RESOLVED)
- **FFI**: `swift-bridge` (DESIGN.v4 O2 RESOLVED)
- **Claude 통합**: `--mcp-config` SSE/Streamable HTTP (Errata E1)
- **인증**: `ANTHROPIC_API_KEY` env var (Errata E3)
- **Subprocess 환경**: `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=0` (Errata E4)

---

## 2. 마일스톤 (Priority 기반, 시간 추정 없음)

### Milestone 1: 프로젝트 스캐폴딩 (Priority High)

**RG-M0-1 + RG-M0-2**

1. Xcode 프로젝트 생성 (`MoAI Studio.app`, macOS 14+)
2. Rust cargo workspace 생성 (11 crate skeleton)
3. `cargo check` 전체 통과 확인
4. Ghostty submodule 추가 (`vendor/ghostty`)
5. Metal Toolchain 설치 확인 (RG-4 조건부)
6. `scripts/build-ghostty-xcframework.sh` 작성 + 실행
7. Xcode 에 `GhosttyKit.xcframework` 링크

**완료 기준**: `cargo check` 0 errors + Xcode 빌드 성공

---

### Milestone 2: Claude Subprocess 통신 (Priority High)

**RG-M0-3 + RG-M0-4**

1. `moai-stream-json`: SDKMessage enum 정의 (13 타입, serde)
2. `moai-stream-json`: NDJSON codec (tokio_util::codec::Framed 또는 자체 구현)
3. `moai-stream-json`: spike fixture 기반 단위 테스트
4. `moai-claude-host`: `tokio::process::Command` 로 Claude subprocess spawn
5. `moai-claude-host`: stdin/stdout pipe 핸들링
6. `moai-claude-host`: `SDKUserMessage` 전송 기능
7. `moai-claude-host`: subprocess 비정상 종료 감지 + 오류 이벤트

**완료 기준**: `cargo test --package moai-stream-json moai-claude-host` 전체 통과

---

### Milestone 3: MCP 서버 + Hook 수신 (Priority High)

**RG-M0-5 + RG-M0-6 + RG-M0-7**

1. `moai-ide-server`: rmcp `#[tool_router]` 로 MCP 서버 구현
2. `moai-ide-server`: `mcp__moai__echo` 도구 정의
3. `moai-ide-server`: 127.0.0.1 바인드 + auth token 생성
4. `moai-ide-server`: Streamable HTTP + SSE backward-compat transport
5. MCP config JSON 생성 (`--mcp-config` 용)
6. Claude <-> MCP 서버 연결 integration test
7. `moai-hook-http`: axum `/hooks/:event` route
8. `moai-hook-http`: `X-Auth-Token` 검증
9. `moai-hook-http`: PreToolUse/PostToolUse/SessionStart 수신 + 응답
10. `plugin/hooks/hooks.json` 작성 (Errata E5 준수)

**완료 기준**: `mcp__moai__echo` round-trip 성공 + hook POST 수신 성공

---

### Milestone 4: FFI + Swift UI (Priority High)

**RG-M0-8 + RG-M0-9**

1. `moai-ffi`: `#[swift_bridge::bridge]` 정의 (new, version, start_workspace, send_user_message)
2. `moai-ffi`: `build.rs` 작성 (Swift 바인딩 자동 생성)
3. `scripts/build-rust-xcframework.sh` 작성
4. Swift 측 `MoaiCoreFFI.swift` wrapper
5. SwiftUI 메인 창 + "New Workspace" 버튼
6. GhosttyKit 터미널 surface view
7. 메시지 입력 UI + Rust core FFI 호출
8. Claude 응답 스트리밍 표시

**완료 기준**: Swift -> Rust -> Claude -> 응답 표시 flow 동작

---

### Milestone 5: End-to-End 통합 (Priority High)

**RG-M0-10**

1. 전체 컴포넌트 통합 빌드
2. End-to-end 시퀀스 수동 검증
3. Integration test suite 작성
4. M0 Go/No-Go 체크리스트 검증
5. M0 completion report 작성

**완료 기준**: 성공 기준 시퀀스 (Swift UI -> Claude spawn -> MCP -> 메시지 -> 응답 -> 표시) 완료

---

## 3. 리스크 대응 계획

| 리스크 | 대응 전략 |
|--------|-----------|
| Metal Toolchain 미설치 | Xcode GUI Components 에서 수동 설치. 실패 시 libghostty-spm fallback |
| rmcp API 변경 | 0.9.x pin + MCP protocol abstraction layer |
| swift-bridge async 불안정 | sync wrapper (tokio runtime block_on) fallback |
| Claude CLI 호환성 | mock subprocess 로 unit test 격리, integration test 는 실제 CLI 사용 |

---

## 4. 테스트 전략

### Unit Tests (Rust)
- `moai-stream-json`: fixture 기반 파싱 테스트 (13 타입)
- `moai-claude-host`: mock subprocess stdout → SDKMessage 디코딩
- `moai-ide-server`: mock MCP client → handshake + tool call
- `moai-hook-http`: mock HTTP request → hook event 처리 + 응답

### Integration Tests
- Claude subprocess spawn + stream-json 양방향 통신
- MCP 서버 + Claude 연결 + `mcp__moai__echo` round-trip
- Hook HTTP endpoint + plugin hooks.json + PreToolUse rewrite

### Manual Verification
- Swift UI 앱 실행 + GhosttyKit 터미널 표시
- "New Workspace" -> Claude spawn -> 메시지 전송 -> 응답 표시

---

## 5. 권장 전문가 상담

| 도메인 | 권장 에이전트 | 사유 |
|--------|--------------|------|
| Rust MCP 서버 | expert-backend | rmcp + axum Streamable HTTP transport 구현 |
| Swift UI + GhosttyKit | expert-frontend | SwiftUI + AppKit bridge, GhosttyKit 통합 |
| Rust <-> Swift FFI | expert-backend | swift-bridge async FFI, build.rs 구성 |
| 빌드 자동화 | expert-devops | xcframework 빌드, CI 스크립트 |
