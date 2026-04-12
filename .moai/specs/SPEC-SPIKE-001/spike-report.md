# SPEC-SPIKE-001: Go/No-Go 보고서

> **실행일**: 2026-04-11 ~ 2026-04-12
> **스택**: Python + FastMCP 3.2.3, stdlib http.server
> **Claude Code**: v2.1.101
> **범위**: RG-1~4 (RG-4 조건부)

---

## 요약 판정

| RG | 검증 대상 | 판정 | 비고 |
|---|---|---|---|
| **RG-1** | Claude CLI stream-json | **GO** ✅ | NDJSON 포맷, 13개 메시지 타입 문서화 완료 |
| **RG-2** | IDE MCP Server 자동 연결 | **NO-GO** ⚠️ → **Fallback GO** | lockfile WS 패턴 실패, `--mcp-config` SSE 패턴 성공 |
| **RG-3** | Plugin http hook type | **GO** ✅ | PreToolUse 수신 + updatedInput 반영 확인 |
| **RG-4** | GhosttyKit xcframework | **조건부 GO** ⚠️ | Zig 0.15.2 설치 성공, Ghostty clone 성공, Metal Toolchain 누락으로 빌드 미완 |

**종합: GO (아키텍처 변경 필요 + RG-4 Xcode 환경 수정 후 재검증)**

### RG-4 상세 (2026-04-12 추가)

**성공 항목:**
- ✅ `brew install zig` → Zig 0.15.2 설치 (Ghostty 1.3.0 요구 버전과 일치)
- ✅ `git clone --depth 1 ghostty` → 131MB, 정상 clone
- ✅ Zig 빌드 시스템이 Ghostty 프로젝트를 인식하고 컴파일 시작 (887% CPU, ~92초)
- ✅ Metal shader 제외 Zig 코드 전체 컴파일 성공

**실패 항목:**
- ❌ `metal` 컴파일러 미발견 → `xcodebuild -downloadComponent MetalToolchain` 필요
- ❌ `xcodebuild -downloadComponent` 자체가 `IDESimulatorFoundation` 플러그인 로드 실패 (Xcode 26.4 Beta 내부 framework 불일치)

**Fallback 경로:**
1. Xcode GUI에서 Components → Metal Toolchain 수동 설치
2. 또는 `sudo xcodebuild -runFirstLaunch` 실행 후 재시도
3. 최종 fallback: Ghostty prebuilt binary 사용 또는 libghostty-spm (Swift Package)

**판정 근거:** Zig 빌드 시스템과 Ghostty 소스의 호환성은 확인됨. Metal Toolchain 누락은 환경 설정 이슈이지 아키텍처 불가가 아님. **조건부 GO** — M0 D1에서 Metal Toolchain 설치 후 재검증.

**Risk register 추가:**
- Xcode Beta 의 `DVTDownloads` 심볼 불일치 → Xcode 재설치가 필요할 수 있음
- Ghostty 1.3.0 이후 libghostty 독립 모듈화 → M0에서 전체 Ghostty 대신 libghostty 모듈만 사용 가능 (더 가벼운 빌드 경로)

---

## RG-1: Claude CLI stream-json 프로토콜 — GO ✅

### 확인된 SDKMessage 타입 (완전 목록)

| 타입 | 서브타입 | 발생 조건 |
|------|----------|-----------|
| `system` | `init` | 항상 (세션 메타데이터) |
| `system` | `hook_started` | `--verbose` (기본 on) |
| `system` | `hook_response` | `--verbose` |
| `assistant` | — | LLM 응답 (text, tool_use, thinking 컨텐츠) |
| `user` | — | 도구 실행 결과 (tool_result) |
| `rate_limit_event` | — | 항상 |
| `result` | `success` | 항상 마지막 |
| `stream_event` | `message_start` | `--include-partial-messages` |
| `stream_event` | `content_block_start` | `--include-partial-messages` |
| `stream_event` | `content_block_delta` | `--include-partial-messages` |
| `stream_event` | `content_block_stop` | `--include-partial-messages` |
| `stream_event` | `message_delta` | `--include-partial-messages` |
| `stream_event` | `message_stop` | `--include-partial-messages` |

### Assistant content 블록 타입

| content.type | 설명 |
|---|---|
| `text` | 텍스트 응답 |
| `tool_use` | 도구 호출 (`name`, `id`, `input`, `caller`) |
| `thinking` | Extended thinking (`thinking`, `signature`) |

### 핵심 발견

1. **메시지 구분자**: `\n` (NDJSON). null byte 아님.
2. **양방향 stream-json**: `--input-format stream-json` 동작 확인
3. **도구 제한 플래그**: `--tools` 가 실제 제한, `--allowedTools`는 추가 허용 (제한 아님!)
4. **`--bare` 제약**: `ANTHROPIC_API_KEY` 필수. OAuth/Keychain 환경에서 동작 불가
5. **subprocess 제약**: `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=1` 시 `--permission-mode` 가 `default`로 강제됨 → `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=0` 설정 필요

### DESIGN.v4 Errata (RG-1)

- `--allowedTools` → `--tools` 로 변경 필요 (도구 제한 목적)
- `--bare` 사용 시 `ANTHROPIC_API_KEY` 환경 변수 필수 명시
- `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB=0` 환경 변수 추가 필요

---

## RG-2: IDE MCP Server Pattern — NO-GO → Fallback GO ⚠️

### 검증 결과

**Lockfile + WebSocket 자동 연결: 실패**

`~/.claude/ide/<port>.lock` 파일을 작성하고 WebSocket MCP 서버를 구동했으나:

1. Claude CLI가 lockfile을 자동 스캔하여 연결하는 것은 **매우 제한적 조건** (정확히 1개의 IDE lockfile만 존재)에서만 발생
2. **연결되더라도 커스텀 MCP 도구가 `mcp__` 네임스페이스에 노출되지 않음**
3. IDE WS 연결은 **코드 인텔리전스** (LSP, code navigation) 전용으로 보임

**`--mcp-config` SSE 패턴: 성공**

- FastMCP SSE 서버 (`http://127.0.0.1:17429/sse`) 구동
- `claude --mcp-config <config.json>` 으로 명시적 등록
- `mcp__moai-spike-sse__echo` 도구가 Claude 도구 네임스페이스에 정상 등록
- 도구 호출 및 결과 반환 확인

### 아키텍처 변경 필요

DESIGN.v4는 IDE MCP Server Pattern (lockfile + WS)을 PRIMARY 통합 경로로 설정했으나, 실측 결과:

| 방식 | 커스텀 도구 노출 | 자동 연결 | 권장 용도 |
|---|---|---|---|
| lockfile + WS | ❌ 불가 | 조건부 | VS Code 등 IDE → Claude 코드 인텔리전스 전달 전용 |
| **`--mcp-config` + SSE** | ✅ 가능 | 명시적 | **MoAI Studio 커스텀 도구 (mcp__moai__*)** |

**권장 변경**: MoAI Studio는 Claude subprocess spawn 시 `--mcp-config` 플래그로 자체 MCP 서버를 명시적으로 등록해야 함. lockfile 패턴은 보조적 용도로만 유지.

### DESIGN.v4 Errata (RG-2)

- §4 "PRIMARY 통합 경로"를 "공식 IDE MCP Server Pattern" → **"--mcp-config SSE/Streamable HTTP Pattern"** 으로 변경
- lockfile + WS 패턴은 "보조 통합 (IDE 코드 인텔리전스)"으로 격하
- MoAI Studio 의 Claude spawn 명령에 `--mcp-config <moai_mcp_config.json>` 추가 필요

---

## RG-3: Plugin http Hook Type — GO ✅

### 검증 결과

| 항목 | 결과 |
|---|---|
| PreToolUse POST 수신 | ✅ |
| X-Auth-Token 헤더 | ✅ `spike-test-token` 정확 수신 |
| JSON payload 구조 | `session_id`, `cwd`, `permission_mode`, `hook_event_name`, `tool_name`, `tool_input`, `tool_use_id` |
| updatedInput 반영 | ✅ `rm` → `trash` rewrite 반영, 원본 `rm` 미실행 확인 |
| http hook type 지원 | ✅ command type과 동등한 기능 |

### 올바른 Plugin 구조 (NEXT-STEPS.md 원안 대비 수정 필요)

```
plugin/
├── .claude-plugin/
│   └── plugin.json      ← "hooks" 필드 없음 (관례 기반 발견)
└── hooks/
    └── hooks.json       ← {"hooks": {"PreToolUse": [...]}} 형태
```

### DESIGN.v4 Errata (RG-3)

- Plugin 구조: `hooks.json` 위치가 `plugin/hooks/hooks.json` (plugin 루트 아님)
- `plugin.json` 에 `"hooks"` 필드 불필요 (관례 기반 자동 발견)
- hooks.json 최상위 키: `{"hooks": {...}}` 래핑 필요
- `hookSpecificOutput` 응답에 `hookEventName` 필드 불필요 — `permissionDecision` + `updatedInput` 만

---

## Risk Register (M0 용)

| 리스크 | 확률 | 영향 | 대응 |
|---|---|---|---|
| `--bare` 가 OAuth 환경에서 동작 불가 | 확인됨 | 중간 | `ANTHROPIC_API_KEY` 환경 변수 필수 설정. Keychain 연동 구현 필요 |
| `CLAUDE_CODE_SUBPROCESS_ENV_SCRUB` 강제 | 확인됨 | 높음 | subprocess spawn 시 `=0` 명시 설정 |
| `--allowedTools` 가 제한이 아닌 추가 허용 | 확인됨 | 중간 | `--tools` 플래그 사용으로 전환 |
| lockfile WS 패턴이 커스텀 도구에 부적합 | 확인됨 | **높음** | **`--mcp-config` SSE 패턴으로 아키텍처 변경** |
| Plugin hooks.json 구조가 문서와 다름 | 확인됨 | 낮음 | DESIGN.v4 errata 반영 완료 |
| RG-4 (GhosttyKit) 미검증 | 보류 | 중간 | zig 설치 후 별도 세션 필요 |

---

## Mock Claude Subprocess Fixture

### SDKMessage 샘플 (M0 테스트용)

```json
{"type":"system","subtype":"init","session_id":"abc123","tools":[{"name":"Bash"},{"name":"Read"}],"mcp_servers":[{"name":"moai","status":"connected"}]}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"Hello, world!"}]}}
{"type":"rate_limit_event","usage":{"input_tokens":100,"output_tokens":50}}
{"type":"result","subtype":"success","result":"Hello, world!","cost_usd":0.001,"total_cost_usd":0.001,"duration_ms":1234,"duration_api_ms":1000}
```

### 도구 사용 패턴

```json
{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","id":"tu_1","name":"Bash","input":{"command":"ls"}}]}}
{"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"tu_1","content":"file1.txt\nfile2.txt"}]}}
```

---

## 열린 결정 업데이트

| 결정 | Spike 전 | Spike 후 |
|---|---|---|
| **O1** (MCP 서버 Rust 라이브러리) | rmcp vs axum+jsonrpsee | **SSE/Streamable HTTP 서버 필요** → axum 기반이 적합 (WS 전용 rmcp 재평가 필요) |
| **O3** (미문서화 hook 필드) | 미확인 | hook payload에 `session_id`, `cwd`, `permission_mode` 포함 확인. `updatedPermissions`/`watchPaths`는 미출현 — 추가 검증 필요 |

---

**Version**: 1.0.0
**작성일**: 2026-04-11
**작성**: MoAI (spike-cli, spike-mcp, spike-hook 에이전트 결과 종합)
