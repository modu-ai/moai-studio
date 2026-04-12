# SPEC-M0-001: Acceptance Criteria

---

## 1. Given-When-Then 시나리오

### AC-1: Rust Workspace 빌드 (RG-M0-1)

**Given** 11개 crate skeleton 이 `core/crates/` 에 생성되어 있고
**When** `cargo check --workspace` 를 실행하면
**Then** 0 errors, 0 warnings 로 완료된다

---

### AC-2: GhosttyKit xcframework 빌드 (RG-M0-2)

**Given** `vendor/ghostty` submodule 이 clone 되어 있고, Metal Toolchain 이 설치되어 있고
**When** `scripts/build-ghostty-xcframework.sh` 를 실행하면
**Then** `app/Frameworks/GhosttyKit.xcframework` 가 생성된다

---

### AC-3: SDKMessage 파싱 (RG-M0-4)

**Given** spike-report.md 의 SDKMessage fixture JSON 이 준비되어 있고
**When** `moai-stream-json` codec 에 fixture 를 입력하면
**Then** 13개 메시지 타입이 올바른 Rust enum variant 로 디코딩되고, 알 수 없는 타입은 `Unknown` 으로 보존된다

---

### AC-4: Claude Subprocess Spawn (RG-M0-3)

**Given** `ANTHROPIC_API_KEY` 환경 변수가 설정되어 있고
**When** `moai-claude-host` 가 Claude subprocess 를 spawn 하면
**Then** 다음 조건을 모두 만족한다:
- subprocess 인자에 `--bare`, `--output-format stream-json`, `--tools` 가 포함됨
- subprocess 환경에 `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=0` 이 설정됨
- subprocess 환경에 `ANTHROPIC_API_KEY` 가 전달됨
- stdout 에서 `system/init` SDKMessage 가 수신됨

---

### AC-5: 사용자 메시지 전송 + 응답 수신 (RG-M0-3)

**Given** Claude subprocess 가 실행 중이고
**When** stdin 으로 `SDKUserMessage` ("Hello, MoAI!") 를 전송하면
**Then** stdout 에서 `assistant` 타입 SDKMessage 가 수신되고, `content[].type == "text"` 블록이 포함된다

---

### AC-6: MCP 서버 바인드 + 도구 등록 (RG-M0-5)

**Given** `moai-ide-server` 가 초기화되었고
**When** mock MCP client 가 `initialize` + `tools/list` 요청을 전송하면
**Then** 응답에 `mcp__moai__echo` 도구가 포함되고, 인증 토큰 검증이 통과된다

---

### AC-7: MCP 도구 Round-trip (RG-M0-6)

**Given** MCP 서버가 실행 중이고 Claude subprocess 가 `--mcp-config` 로 연결된 상태에서
**When** Claude 에게 "echo 도구로 'proof-of-life' 를 반환해줘" 를 요청하면
**Then** `mcp__moai__echo` 가 호출되고 "proof-of-life" 문자열이 Claude 응답에 포함된다

---

### AC-8: Hook HTTP 수신 (RG-M0-7)

**Given** `plugin/hooks/hooks.json` 이 `{"hooks": {"PreToolUse": [...]}}` 구조이고, `moai-hook-http` 가 실행 중이고
**When** Claude 가 Bash 도구를 사용하려 하면
**Then** `moai-hook-http` 엔드포인트에 PreToolUse POST 가 수신되고:
- `X-Auth-Token` 헤더가 검증됨
- payload 에 `tool_name`, `tool_input`, `hook_event_name` 이 포함됨
- 응답에 `hookEventName` 필드가 포함되지 않음 (Errata E6)

---

### AC-9: Hook updatedInput Rewrite (RG-M0-7)

**Given** `moai-hook-http` 가 PreToolUse 에 `updatedInput` 을 포함한 응답을 반환하도록 설정되어 있고
**When** Claude 가 Bash 도구로 `rm test.txt` 를 실행하려 하면
**Then** hook 응답의 `updatedInput.command` 값 (예: `trash test.txt`) 으로 실제 실행이 대체된다

---

### AC-10: swift-bridge FFI (RG-M0-8)

**Given** `moai-ffi` crate 가 빌드되어 Swift 바인딩이 생성되어 있고
**When** Swift 에서 `RustCore()` 를 생성하고 `version()` 을 호출하면
**Then** Rust 측에서 정의한 버전 문자열이 반환된다

---

### AC-11: GhosttyKit 터미널 표시 (RG-M0-9)

**Given** Xcode 프로젝트에 `GhosttyKit.xcframework` 가 링크되어 있고
**When** 앱을 실행하고 "New Workspace" 를 클릭하면
**Then** GhosttyKit Metal 렌더링으로 zsh shell 이 터미널 surface 에 표시된다

---

### AC-12: End-to-End Proof of Life (RG-M0-10)

**Given** 모든 컴포넌트가 빌드 + 연결된 상태에서
**When** 사용자가 다음 시퀀스를 수행하면:
1. MoAI Studio 앱 실행
2. "New Workspace" 클릭
3. 메시지 입력 ("What tools do you have?")
4. 전송

**Then** 다음이 순차적으로 발생한다:
1. Claude subprocess 가 spawn 됨 (`--bare --mcp-config` 포함)
2. MCP 서버에 Claude 가 연결됨 (서버 로그에 initialize 핸드셰이크 기록)
3. assistant 응답이 스트리밍으로 수신됨
4. 응답 텍스트가 UI 에 실시간 표시됨
5. 응답에 `mcp__moai__echo` 가 사용 가능한 도구로 언급됨

---

## 2. Edge Cases

### EC-1: ANTHROPIC_API_KEY 미설정

**Given** `ANTHROPIC_API_KEY` 환경 변수가 설정되지 않은 상태에서
**When** workspace 시작을 시도하면
**Then** subprocess spawn 전에 사용자에게 API key 설정 안내 메시지가 표시되고, spawn 이 차단된다

### EC-2: Claude CLI 미설치

**Given** `claude` 바이너리가 PATH 에 존재하지 않는 상태에서
**When** workspace 시작을 시도하면
**Then** Claude Code 설치 안내 메시지가 표시되고, spawn 이 차단된다

### EC-3: MCP 서버 포트 충돌

**Given** 선택된 포트가 이미 사용 중인 경우
**When** MCP 서버 바인드를 시도하면
**Then** 다른 랜덤 포트로 재시도하고 (최대 3회), 모두 실패 시 오류를 보고한다

### EC-4: Claude Subprocess 비정상 종료

**Given** Claude subprocess 가 실행 중인 상태에서
**When** subprocess 가 signal 또는 panic 으로 비정상 종료하면
**Then** 오류 이벤트가 EventBus 에 발행되고, workspace 상태가 `error` 로 전환되며, UI 에 오류 메시지가 표시된다

### EC-5: Hook HTTP 인증 실패

**Given** `moai-hook-http` 가 실행 중인 상태에서
**When** 잘못된 `X-Auth-Token` 을 포함한 요청이 수신되면
**Then** 401 Unauthorized 를 반환하고 요청을 처리하지 않는다

### EC-6: 알 수 없는 SDKMessage 타입

**Given** stream-json 스트림에서
**When** 정의되지 않은 `type` 값을 가진 JSON 이 수신되면
**Then** `Unknown` variant 로 보존하고, `tracing::warn!` 을 기록하며, 스트림을 중단하지 않는다

---

## 3. Quality Gate Criteria

### Rust Build Quality
- [ ] `cargo check --workspace`: 0 errors, 0 warnings
- [ ] `cargo test --workspace`: 전체 통과
- [ ] `cargo clippy --workspace`: 0 errors (warnings 허용)

### Xcode Build Quality
- [ ] Xcode 빌드: 0 errors
- [ ] GhosttyKit.xcframework 링크 성공
- [ ] Rust xcframework 링크 성공

### Integration Quality
- [ ] Claude subprocess spawn + stream-json 통신 검증
- [ ] MCP 서버 연결 + `mcp__moai__echo` round-trip 검증
- [ ] Hook HTTP PreToolUse 수신 + 응답 검증
- [ ] swift-bridge FFI 양방향 호출 검증

### End-to-End Quality
- [ ] "Swift UI -> Claude spawn -> MCP connect -> message -> response -> display" 시퀀스 완료

---

## 4. Definition of Done

SPEC-M0-001 은 다음 조건이 모두 충족될 때 완료로 간주한다:

1. **AC-1 ~ AC-12** 의 모든 Given-When-Then 시나리오가 통과
2. **Rust workspace**: `cargo check` + `cargo test` 0 errors
3. **Xcode project**: 빌드 성공 (GhosttyKit + Rust xcframework)
4. **6개 산출물**: spec.md 4 에 명시된 모든 산출물이 해당 경로에 존재
5. **M0 completion report**: `docs/M0-COMPLETION.md` 에 Go/No-Go 체크리스트 기록
6. **Edge case EC-1 ~ EC-6**: 최소 오류 메시지 표시 수준 (graceful degradation)
