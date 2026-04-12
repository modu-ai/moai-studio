# MoAI Studio — Next Steps (4 단계)

> **목적**: DESIGN.v4.md 승인 후 진행할 4개 작업을 구체적으로 명시.
> **Date**: 2026-04-11
> **Status**: Ready to execute

이 문서는 형님이 `moai init` 으로 MoAI Studio 저장소를 신규 프로젝트로 초기화한 후 **순차 실행할 4 단계 작업**을 정의합니다.

---

## 단계 1 — Pre-M0 검증 스파이크 (3-4일)

**목표**: DESIGN.v4.md 의 4가지 기술 전제를 실제 명령으로 검증하고 Go/No-Go 를 결정한다. M0 투자 (2주) 전에 리스크를 제거한다.

**배경**: DESIGN.v4 는 B4 (공식 문서 검증) 에 의존하지만, 일부 필드는 `[UNVERIFIED]` 로 표시되었다. 특히:
- `--bare -p --output-format stream-json` 의 실제 동작
- `claude` CLI 가 `~/.claude/ide/*.lock` 을 자동 스캔하는지
- Plugin manifest 의 `http` hook type 이 `hookSpecificOutput.updatedInput` 을 respect 하는지
- GhosttyKit xcframework 가 Zig 최신 버전에서 빌드되는지

### 작업 1.1 — Day 1: Claude CLI 공식 경로 수동 검증

**소요**: 0.5-1일

**작업:**
```bash
# 1. bash 에서 직접 실행
claude --bare -p "Hello, world" \
  --output-format stream-json \
  --include-partial-messages \
  --verbose

# 2. 출력 스트림 파싱
claude --bare -p "Write a poem about Rust" \
  --output-format stream-json \
  --verbose | \
  jq -rj 'select(.type == "assistant_message") | .content[]?.text // empty'

# 3. 양방향 stream-json 검증
echo '{"type":"user_message","content":{"text":"Hello"}}' | \
  claude --bare --output-format stream-json --input-format stream-json

# 4. --settings, --mcp-config, --agents 플래그 동작 확인
claude --bare -p "list files" \
  --settings ./test-settings.json \
  --allowedTools "Read,Glob" \
  --permission-mode dontAsk \
  --output-format stream-json
```

**기록할 것:**
- 출력 SDKMessage 의 실제 JSON 구조 (type 필드 값 목록)
- stream-json 메시지 사이 구분자 (newline? null?)
- `initialize` / `control_request` 류가 실제로 나타나는지
- `stream-json` **입력** 포맷이 공식으로 작동하는지 (공식 미문서화)

**Go 기준**: 
- ✅ `claude --bare -p ...` 이 stream-json 을 뱉어냄
- ✅ SDKMessage 타입이 B2 에서 기록한 schema 와 일치
- ✅ `--settings`, `--allowedTools`, `--permission-mode` 가 동작

**No-Go 시 대응**: `--bare` 가 아직 릴리스 안 됐다면 `-p` 만으로도 진행 가능. 대신 context loading 을 수동으로 제어해야 함.

---

### 작업 1.2 — Day 2: IDE MCP Server Pattern 복제 (Python prototype)

**소요**: 1일

**목표**: Rust 를 쓰기 전에 Python 으로 단순 MCP server 를 만들어 Claude Code 가 자동 연결되는지 확인한다.

**작업:**
```bash
# 1. 로컬 Python MCP server 작성 (fastapi + websockets)
mkdir -p /tmp/moai-ide-spike && cd /tmp/moai-ide-spike
cat > server.py <<'PY'
# Minimal MCP server for Claude Code IDE integration test
import asyncio, json, secrets, os
from pathlib import Path
import websockets

PORT = 17428
TOKEN = secrets.token_hex(32)

LOCKFILE = Path.home() / ".claude" / "ide" / f"{PORT}.lock"
LOCKFILE.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
LOCKFILE.write_text(json.dumps({
    "workspaceFolders": [os.getcwd()],
    "pid": os.getpid(),
    "ideName": "MoAI-Spike",
    "transport": "ws",
    "runningInWindows": False,
    "authToken": TOKEN,
}))
os.chmod(LOCKFILE, 0o600)

async def handler(ws):
    print(f"[server] Client connected")
    async for msg in ws:
        req = json.loads(msg)
        print(f"[server] RX: {req}")
        # MCP initialize response
        if req.get("method") == "initialize":
            resp = {
                "jsonrpc": "2.0",
                "id": req["id"],
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "moai-spike", "version": "0.0.1"}
                }
            }
            await ws.send(json.dumps(resp))
        elif req.get("method") == "tools/list":
            resp = {
                "jsonrpc": "2.0",
                "id": req["id"],
                "result": {
                    "tools": [{
                        "name": "echo",
                        "description": "Echo back a message",
                        "inputSchema": {"type": "object", "properties": {"msg": {"type": "string"}}}
                    }]
                }
            }
            await ws.send(json.dumps(resp))

async def main():
    print(f"[server] Starting on 127.0.0.1:{PORT}, token={TOKEN[:8]}...")
    async with websockets.serve(handler, "127.0.0.1", PORT):
        await asyncio.Future()

asyncio.run(main())
PY

# 2. 실행
python3 server.py &
SERVER_PID=$!

# 3. 다른 터미널에서 Claude 실행
cd /tmp/moai-ide-spike
claude -p "What tools do you have via the IDE MCP server?"

# 4. 로그 확인 — Claude 가 auto-connect 했는지
kill $SERVER_PID
rm ~/.claude/ide/17428.lock
```

**기록할 것:**
- Claude 가 lockfile 을 **자동으로 스캔** 하는지
- 연결 시 Bearer token auth header 형식
- `tools/list` RPC 가 실제로 호출되는지

**Go 기준:**
- ✅ Claude 가 moai-spike 서버에 자동 연결
- ✅ `tools/list` 응답 후 `echo` tool 이 Claude 가 사용 가능한 상태로 나타남

**No-Go 시 대응**: Lockfile 자동 스캔이 안 되면 `--mcp-config` 로 명시적 등록 경로로 fallback.

---

### 작업 1.3 — Day 3: Plugin `http` hook type 검증

**소요**: 0.5-1일

**목표**: Plugin manifest 의 `http` hook type 이 실제로 작동하고, `hookSpecificOutput.updatedInput` 을 Claude 가 respect 하는지 확인.

**작업:**
```bash
# 1. Minimal plugin 구조
mkdir -p /tmp/moai-hook-spike/.claude-plugin
cat > /tmp/moai-hook-spike/.claude-plugin/plugin.json <<'PJ'
{
  "name": "moai-hook-spike",
  "version": "0.0.1",
  "description": "Spike for http hook type",
  "hooks": "./hooks.json"
}
PJ

cat > /tmp/moai-hook-spike/hooks.json <<'HK'
{
  "PreToolUse": [{
    "matcher": "Bash",
    "hooks": [{
      "type": "http",
      "url": "http://127.0.0.1:18274/hooks/PreToolUse",
      "headers": {"X-Auth-Token": "spike-test-token"},
      "timeout": 5000
    }]
  }]
}
HK

# 2. Python Flask 서버로 hook endpoint
pip install flask
cat > /tmp/moai-hook-spike/hook_server.py <<'PY'
from flask import Flask, request, jsonify
app = Flask(__name__)

@app.route("/hooks/<event>", methods=["POST"])
def handle_hook(event):
    print(f"[hook] {event} received")
    print(f"[hook] payload: {request.json}")
    auth = request.headers.get("X-Auth-Token")
    print(f"[hook] auth: {auth}")

    # Try updatedInput
    if event == "PreToolUse":
        tool_name = request.json.get("tool_name")
        tool_input = request.json.get("tool_input", {})
        if tool_name == "Bash":
            original = tool_input.get("command", "")
            # Rewrite rm to trash
            rewritten = original.replace("rm ", "trash ")
            return jsonify({
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "allow",
                    "updatedInput": {**tool_input, "command": rewritten}
                }
            })
    return jsonify({})

app.run(host="127.0.0.1", port=18274)
PY

python3 /tmp/moai-hook-spike/hook_server.py &
HOOK_PID=$!

# 3. Claude 를 plugin-dir 로 실행
cd /tmp && mkdir -p test-workdir && cd test-workdir
claude -p "rm nonexistent.txt" \
  --plugin-dir /tmp/moai-hook-spike \
  --allowedTools "Bash" \
  --permission-mode acceptEdits \
  --output-format stream-json

# 4. hook_server.py 의 출력 확인 → PreToolUse 수신 확인
# 5. Claude 의 실제 실행이 "trash" 로 rewrite 되었는지 확인

kill $HOOK_PID
```

**기록할 것:**
- `http` hook type 이 실제로 작동하는가?
- `updatedInput` 이 Claude 의 실제 tool 실행에 반영되는가?
- Timeout, auth header, JSON schema 실제 형식

**Go 기준:**
- ✅ PreToolUse hook endpoint 가 POST 를 수신
- ✅ `updatedInput` 으로 `rm` 이 `trash` 로 rewrite 되어 실제 실행됨

**No-Go 시 대응**: `http` hook type 이 작동 안 하면 `command` hook wrapper (shell) + curl 로 fallback. 성능은 떨어지지만 기능은 동일.

---

### 작업 1.4 — Day 4: GhosttyKit.xcframework 빌드

**소요**: 0.5-1일

**목표**: libghostty 를 macOS xcframework 로 빌드하고 최소 Xcode 프로젝트에서 단일 터미널을 표시.

**작업:**
```bash
# 1. Ghostty clone
cd ~/moai
git clone https://github.com/ghostty-org/ghostty.git
cd ghostty

# 2. Zig 버전 확인
zig version  # Ghostty 가 요구하는 Zig 버전과 일치하는지

# 3. xcframework 빌드
zig build -Demit-xcframework=true -Doptimize=ReleaseFast
ls zig-out/xcframework/GhosttyKit.xcframework

# 4. 최소 Xcode 프로젝트
cd ~/moai/moai-cli
mkdir -p spike-ghostty
# Xcode: 새 macOS App 프로젝트 생성, GhosttyKit.xcframework 링크
# ContentView.swift 에 최소 Ghostty surface 하나 렌더

# 5. 빌드 + 실행
```

**기록할 것:**
- Zig 버전 요구사항 및 빌드 시간
- xcframework 파일 크기
- Swift 에서 `import GhosttyKit` 시 사용 가능한 API 수
- 단일 터미널 렌더링 성공 여부

**Go 기준:**
- ✅ `zig build` 성공
- ✅ Xcode 에 xcframework 링크 성공
- ✅ `import GhosttyKit` 성공
- ✅ 단일 zsh terminal 이 창에 표시

**No-Go 시 대응**: Ghostty 빌드 실패 시 `libghostty-spm` (Swift Package) 사용. SPM 방식이 더 안정적일 수 있음.

---

### 단계 1 Deliverables

1. **Go/No-Go 보고서** (`NEXT-STEPS-1-spike-report.md`) — 4개 spike 결과 요약
2. **Mock Claude subprocess fixture** — 실제 SDKMessage 샘플 JSON 수집 (M0 의 테스트에 활용)
3. **Updated DESIGN.v4 errata** — 실측과 다른 부분 정정
4. **Risk register** — M0 에서 주의할 사항 목록

---

## 단계 2 — M0 킥오프 (2주)

**목표**: Rust core 스켈레톤 + Swift UI shell + 첫 Hook 왕복. "proof of life" 수준.

**전제**: 단계 1 Go 통과.

### M0 주 1

**D1 (2시간)**: Xcode 프로젝트 + Rust cargo workspace 생성
```bash
cd ~/moai/moai-cli
mkdir -p app core plugin vendor scripts tests

# Rust workspace
cat > core/Cargo.toml <<'TOML'
[workspace]
resolver = "2"
members = [
    "crates/moai-core",
    "crates/moai-supervisor",
    "crates/moai-claude-host",
    "crates/moai-stream-json",
    "crates/moai-ide-server",
    "crates/moai-hook-http",
    "crates/moai-store",
    "crates/moai-git",
    "crates/moai-fs",
    "crates/moai-plugin-installer",
    "crates/moai-ffi",
]
TOML

# 각 crate 생성
for crate in moai-core moai-supervisor moai-claude-host moai-stream-json moai-ide-server moai-hook-http moai-store moai-git moai-fs moai-plugin-installer moai-ffi; do
  cargo new --lib core/crates/$crate
done

# Xcode 프로젝트
# (수동) open Xcode → New macOS App → MoAI Studio.app
```

**D2 (1일)**: Ghostty submodule + xcframework 빌드 자동화
```bash
cd ~/moai/moai-cli
git submodule add https://github.com/ghostty-org/ghostty.git vendor/ghostty
cat > scripts/build-ghostty-xcframework.sh <<'SH'
#!/bin/bash
set -e
cd "$(dirname "$0")/../vendor/ghostty"
zig build -Demit-xcframework=true -Doptimize=ReleaseFast
cp -R zig-out/xcframework/GhosttyKit.xcframework ../../app/Frameworks/
SH
chmod +x scripts/build-ghostty-xcframework.sh
./scripts/build-ghostty-xcframework.sh
```

**D3 (1일)**: Rust `moai-claude-host` — subprocess spawn
- `tokio::process::Command` 로 `claude --bare -p ... --output-format stream-json` spawn
- stdin/stdout pipe handle
- 기본 테스트: `cargo test --package moai-claude-host`

**D4 (1일)**: Rust `moai-stream-json` — SDKMessage codec
- `serde` 로 SDKMessage 타입 정의 (단계 1.1 에서 수집한 실제 샘플 기반)
- `tokio_util::codec::Framed` 래퍼
- 단위 테스트: fixture 파싱

**D5 (1일)**: Rust `moai-ide-server` — MCP 서버 + lockfile
- `axum` + `tokio::net::TcpListener`
- 127.0.0.1:<random> bind
- `~/.claude/ide/<port>.lock` 생성 (0600, 0700 directory)
- 단위 테스트: mock client 연결 + handshake

### M0 주 2

**D6 (0.5일)**: `claude` 가 `moai-ide-server` 에 auto-connect 검증
- `cargo run --bin moai-ide-server-spike` 로 서버 기동
- 다른 터미널에서 `claude --mcp-config ... -p "test"` 실행
- 서버 로그에서 auto-connect 확인

**D7 (1일)**: Rust `moai-hook-http` — plugin http hook receiver
- `axum` route `/hooks/:event`
- `X-Auth-Token` 검증
- Mock hook event 수신 + `hookSpecificOutput` 응답

**D8 (1일)**: swift-bridge FFI 셋업
- `core/crates/moai-ffi/build.rs` 작성
- Swift 측 `MoaiCoreFFI.swift` 생성
- 단일 함수 호출 검증 (`RustCore::version()`)

**D9 (1일)**: Swift UI + GhosttyKit single terminal
- `app/Sources/Surfaces/Terminal/GhosttyTerminalView.swift`
- 단일 zsh surface 렌더
- Rust core 호출 성공

**D10 (1일)**: M0 통합 테스트 + Go/No-Go
- Swift UI 가 Rust core → Claude subprocess spawn → IDE MCP auto-connect → hook event round-trip 을 end-to-end 로 수행
- 모든 단계가 작동하면 M0 Go

### M0 Deliverables

- `~/moai/moai-cli/app/` — Xcode 프로젝트 (Minimal SwiftUI + GhosttyKit)
- `~/moai/moai-cli/core/` — Rust workspace (10 crate 스켈레톤)
- `~/moai/moai-cli/plugin/` — 기본 plugin manifest
- `~/moai/moai-cli/vendor/ghostty` — submodule
- `~/moai/moai-cli/scripts/` — 빌드 자동화
- `M0-COMPLETION.md` — M0 완료 보고서

**성공 기준**: Swift UI 창에서 "New Workspace" 버튼 클릭 → Claude subprocess spawn → IDE MCP auto-connect → 사용자 메시지 전송 → assistant 응답 스트리밍 수신 → 화면 표시.

---

## 단계 3 — 열린 결정 인터뷰 (O1-O6)

**목표**: DESIGN.v4.md §14 의 6개 열린 결정에 대한 답변 수집.

**전제**: 단계 1-2 와 병렬 진행 가능 (기술 검증에 의존 안 함).

### 결정 대상

| # | 결정 | 의존 | 기한 |
|---|---|---|---|
| ~~O1~~ | ~~MCP 서버 Rust 라이브러리~~ | **RESOLVED → `rmcp` + `axum` Streamable HTTP** (2026-04-12, DESIGN.v4 §14 O1) | — |
| ~~O2~~ | ~~`swift-bridge` vs `uniffi-rs` vs `cbindgen`~~ | **RESOLVED → swift-bridge** (2026-04-11, DESIGN.v4 §14 O2) | — |
| O3 | 미문서화 hook 필드 (`updatedPermissions`, `watchPaths`) 사용 정책 | 단계 1.3 결과 | M1 |
| O4 | Plugin 자동 설치 동의 UX | 없음 | M4 |
| O5 | `claude` 바이너리 버전 pinning 정책 | 없음 | M4 |
| ~~O6~~ | ~~브랜딩 최종 확정~~ | **RESOLVED → MoAI Studio** (2026-04-11, DESIGN.v4 §14 O6) | — |

### 진행 방식

- MoAI 소크라테스 인터뷰 스타일 (v2-v4 작업 중 이미 사용한 패턴)
- 각 결정에 대해 2-4개 옵션 + trade-off 제시
- `AskUserQuestion` 도구로 단일 선택
- 답변을 `DESIGN.v4.md` §14 에 반영 (O → RESOLVED 로 상태 변경)

### Deliverables

- `DESIGN.v4.md` §14 가 모두 RESOLVED 상태
- 각 결정의 이유를 `.moai/memory/decisions/<ODD>-<YYYY-MM-DD>.md` 에 기록

---

## 단계 4 — 커뮤니티 시작 신호

**목표**: MoAI Studio 존재를 알리고 early adopter 를 유치. **단, M4 (Claude 통합 심화) 완료 전까지는 공개 마케팅 자제.**

**전제**: M4 완료.

### 작업 4.1 — `modu-ai/moai-adk` README 에 로드맵 섹션 추가

```markdown
## 🗿 MoAI Studio (Coming Soon)

moai-adk 의 공식 macOS 네이티브 IDE-쉘. Claude Code 를 subprocess 로 호스트하여
27개 hook 이벤트 + 26 전문 에이전트 + TRUST 5 품질 게이트 + Kanban/SPEC 워크플로우를
한 화면에서 시각화 · 조작한다.

- **Status**: Design complete, implementation in progress
- **Target**: macOS 14+
- **License**: MIT
- **Repo**: [MoAI Studio](https://github.com/modu-ai/moai-studio)
```

### 작업 4.2 — GitHub 저장소 공개

- `~/moai/moai-cli` → `modu-ai/moai-studio` push
- README.md, DESIGN.v4.md, NEXT-STEPS.md, research/ 모두 공개
- Issues, Discussions 활성화

### 작업 4.3 — Show HN 예고 포스팅 (M5 완료 시점)

제목: "MoAI Studio: A native macOS IDE shell for Claude Agent workflows"

본문 포인트:
- cmux 를 잇는 위치 (Swift + libghostty) + SPEC/Kanban/Memory 차별화
- 공식 IDE MCP Server Pattern 활용 (VS Code 와 동일)
- 27 hook 이벤트 양방향 + tool input rewriting
- MIT license, open source

### 작업 4.4 — cmux 팀 friendly outreach

cmux 는 GPL-3.0, 우리는 MIT 이므로 코드 공유는 어렵다. 그러나:
- 아이디어 공유
- 상호 링크
- libghostty 생태계 협력

이메일 또는 GitHub Discussion 통한 인사.

### 작업 4.5 — Anthropic Developer Relations contact

공식 문서의 브랜딩 가이드라인 ("Claude Code" 명칭 금지) 준수 여부 확인 겸 introduction. MoAI Studio 가 공식 IDE MCP Server Pattern 을 모범적으로 구현한 사례로 제시 가능.

### Deliverables

- `modu-ai/moai-adk` README 업데이트 PR
- `modu-ai/moai-studio` GitHub 저장소 공개
- Show HN draft 포스트
- cmux outreach 기록
- Anthropic contact 기록

---

## 우선순위 요약

| 단계 | 우선순위 | 소요 | 병렬 가능 |
|---|---|---|---|
| 1. Pre-M0 spike | **즉시** | 3-4일 | 단독 |
| 2. M0 kickoff | 단계 1 완료 후 | 2주 | 단계 3 과 병렬 |
| 3. 열린 결정 인터뷰 | 단계 2 와 병렬 | ~2-3일 (총) | 단계 2 와 병렬 |
| 4. 커뮤니티 신호 | M4 완료 후 | 1주 | 순차 |

---

## 다음 액션 (지금 당장 할 일)

1. **`moai init` 으로 이 저장소를 신규 프로젝트로 초기화** (형님이 직접)
2. **단계 1.1 (Day 1)** 시작 — `claude --bare -p --output-format stream-json` 수동 검증
3. 결과를 `.moai/memory/spike-day1.md` 에 기록
4. Day 2 로 진행

**Questions to answer before starting:**
- [x] O2 (swift-bridge vs uniffi-rs) — **RESOLVED → swift-bridge** (2026-04-11)
- [x] O6 (브랜딩) — **RESOLVED → MoAI Studio** (2026-04-11)

---

**Version**: 1.0.0
**Last Updated**: 2026-04-11
**Based on**: DESIGN.v4.md
