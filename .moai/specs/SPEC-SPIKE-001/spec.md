# SPEC-SPIKE-001: Pre-M0 검증 스파이크

---
id: SPEC-SPIKE-001
version: 1.0.0
status: Planned
created: 2026-04-11
updated: 2026-04-11
author: manager-spec
priority: High
issue_number: null
---

## HISTORY

| 버전 | 날짜 | 변경 내용 |
|------|------|-----------|
| 1.0.0 | 2026-04-11 | 최초 작성 |

---

## 개요

MoAI Studio (macOS 네이티브 Agent IDE) 의 DESIGN.v4.md 가 의존하는 **4가지 기술 전제**를 실제 명령으로 검증하고 Go/No-Go 를 결정한다. M0 투자 전에 리스크를 제거하는 것이 목적이다.

**배경**: DESIGN.v4 는 B4 (공식 문서 검증) 에 의존하지만, 일부 필드는 `[UNVERIFIED]` 로 표시되어 있다. 검증 대상:
- `--bare -p --output-format stream-json` 의 실제 동작
- `claude` CLI 의 `~/.claude/ide/*.lock` 자동 스캔 여부
- Plugin manifest `http` hook type 의 `hookSpecificOutput.updatedInput` 지원 여부
- GhosttyKit xcframework 의 Zig 빌드 가능 여부

**스택**: Swift (UI) + Rust (Core), swift-bridge FFI
**플랫폼**: macOS 14+

---

## 요구사항

### RG-1: Claude CLI stream-json 프로토콜 검증

#### RG-1.1 stream-json 출력 검증

**When** `claude --bare -p "<prompt>" --output-format stream-json` 을 실행하면, the system **shall** SDKMessage JSON 스트림을 stdout 으로 출력한다.

#### RG-1.2 SDKMessage 타입 일치 검증

**When** stream-json 출력을 파싱하면, the system **shall** B2 리서치에서 기록한 SDKMessage schema 와 일치하는 type 필드 값을 반환한다.

#### RG-1.3 CLI 플래그 동작 검증

**When** `--settings`, `--allowedTools`, `--permission-mode` 플래그를 지정하면, the system **shall** 각 플래그의 명시된 동작을 수행한다.

#### RG-1.4 양방향 stream-json 검증

**When** stdin 으로 `stream-json` 포맷의 메시지를 전송하면, the system **shall** 해당 메시지를 파싱하여 응답을 생성한다.

**기록 대상:**
- SDKMessage 의 실제 JSON 구조 (type 필드 값 목록)
- stream-json 메시지 구분자 (newline / null)
- `initialize` / `control_request` 메시지의 실제 출현 여부
- stream-json 입력 포맷 공식 동작 여부 (공식 미문서화)

---

### RG-2: IDE MCP Server Pattern 자동 연결 검증

#### RG-2.1 lockfile 기반 자동 연결

**When** `~/.claude/ide/<port>.lock` 파일이 존재하고 MCP 서버가 해당 포트에서 리스닝 중이면, the system **shall** Claude CLI 가 lockfile 을 자동 스캔하여 서버에 연결한다.

#### RG-2.2 MCP tools/list RPC 동작

**When** Claude CLI 가 MCP 서버에 연결하면, the system **shall** `tools/list` JSON-RPC 를 호출하고 응답에 포함된 tool 을 사용 가능 상태로 등록한다.

#### RG-2.3 Bearer token 인증

**While** MCP 서버가 lockfile 에 authToken 을 기록한 상태에서, **when** Claude CLI 가 연결을 시도하면, the system **shall** Bearer token 을 인증 헤더에 포함하여 전송한다.

**기록 대상:**
- Claude 의 lockfile 자동 스캔 동작 여부
- Bearer token auth header 형식
- `tools/list` RPC 실제 호출 여부

---

### RG-3: Plugin http hook type 검증

#### RG-3.1 http hook endpoint 수신

**When** Plugin manifest 에 `http` hook type 을 정의하고 Claude 가 해당 이벤트를 트리거하면, the system **shall** 지정된 URL 로 POST 요청을 전송한다.

#### RG-3.2 updatedInput 반영

**When** hook endpoint 가 `hookSpecificOutput.updatedInput` 을 응답으로 반환하면, the system **shall** 원래 tool input 대신 rewrite 된 input 으로 tool 을 실행한다.

#### RG-3.3 인증 및 타임아웃

**While** http hook 이 `X-Auth-Token` 헤더와 timeout 을 설정한 상태에서, **when** hook 이벤트가 발생하면, the system **shall** 지정된 헤더를 포함하고 timeout 내에 응답을 대기한다.

**기록 대상:**
- `http` hook type 실제 작동 여부
- `updatedInput` 이 실제 tool 실행에 반영되는지
- timeout, auth header, JSON schema 실제 형식

---

### RG-4: GhosttyKit xcframework 빌드 검증

#### RG-4.1 Zig 빌드 성공

**When** `zig build -Demit-xcframework=true -Doptimize=ReleaseFast` 를 실행하면, the system **shall** `GhosttyKit.xcframework` 를 생성한다.

#### RG-4.2 Xcode 링크 성공

**When** 생성된 xcframework 를 Xcode 프로젝트에 링크하면, the system **shall** `import GhosttyKit` 가 컴파일에 성공한다.

#### RG-4.3 터미널 렌더링

**When** SwiftUI 뷰에서 GhosttyKit surface 를 초기화하면, the system **shall** 단일 zsh 터미널을 창에 렌더링한다.

**기록 대상:**
- Zig 버전 요구사항 및 빌드 시간
- xcframework 파일 크기
- Swift 에서 사용 가능한 API 수
- 단일 터미널 렌더링 성공 여부

---

## Go/No-Go 기준

| 작업 | Go 기준 | No-Go 시 대응 (Fallback) |
|------|---------|--------------------------|
| RG-1 | stream-json 출력 + SDKMessage 타입 일치 + 플래그 동작 | `-p` 만 사용, `--bare` 제외. 수동 context 제어 |
| RG-2 | lockfile 자동 스캔 + tools/list 동작 + auto-connect | `--mcp-config` 명시적 등록 경로 |
| RG-3 | PreToolUse POST 수신 + updatedInput rewrite 반영 | `command` hook wrapper + curl |
| RG-4 | zig build 성공 + xcframework 링크 + 터미널 렌더링 | `libghostty-spm` (Swift Package) |

---

## 열린 결정 의존성

이 스파이크 결과에 의존하는 열린 결정:

| 결정 | 의존 작업 | 설명 |
|------|-----------|------|
| O1: MCP 서버 Rust 라이브러리 (`rmcp` vs `axum+jsonrpsee`) | RG-2 | MCP auto-connect 패턴에 따라 최적 라이브러리 결정 |
| O3: 미문서화 hook 필드 사용 정책 | RG-3 | `updatedPermissions`, `watchPaths` 등 사용 여부 결정 |

---

## 산출물

1. **Go/No-Go 보고서** -- 4개 스파이크 결과 요약
2. **Mock Claude subprocess fixture** -- 실제 SDKMessage 샘플 JSON 수집 (M0 테스트 활용)
3. **Updated DESIGN.v4 errata** -- 실측과 다른 부분 정정
4. **Risk register** -- M0 진행 시 주의 사항 목록

---

## 리스크/Fallback 매트릭스

| 리스크 | 발생 확률 | 영향도 | Fallback | 잔존 영향 |
|--------|----------|--------|----------|-----------|
| `--bare` 플래그 미출시 | 낮음 | 중간 | `-p` 단독 사용 + 수동 context 제어 | Context 오염 가능성, 추가 필터링 필요 |
| lockfile 자동 스캔 미지원 | 중간 | 중간 | `--mcp-config` 명시적 등록 | 설정 파일 관리 부담 증가 |
| `http` hook type 미작동 | 중간 | 높음 | `command` hook + curl wrapper | 성능 저하 (프로세스 fork 오버헤드) |
| `updatedInput` 미반영 | 높음 | 높음 | tool input rewriting 포기, 사후 감사 로그만 | 핵심 기능 약화, 아키텍처 재설계 필요 |
| Zig 버전 불일치로 빌드 실패 | 중간 | 낮음 | Zig 버전 다운그레이드 또는 `libghostty-spm` | SPM 방식이 더 안정적일 수 있음 |
| xcframework Xcode 링크 실패 | 낮음 | 중간 | `libghostty-spm` Swift Package | SPM 통합은 검증된 경로 |

---

## 제약사항

- The system **shall** macOS 14+ 환경에서만 검증을 수행한다.
- The system **shall** Python prototype 을 검증 도구로만 사용하고 프로덕션 코드로 전환하지 않는다.
- The system **shall** 검증 중 Claude CLI 의 로컬 설치 버전을 기록한다.

---

## Exclusions (What NOT to Build)

- 프로덕션 수준의 Rust/Swift 코드 작성 (스파이크 목적은 검증만)
- MCP 서버의 완전한 JSON-RPC 구현 (tools/list 수준만 검증)
- GhosttyKit 의 멀티탭/멀티윈도우 구현 (단일 터미널 렌더링만)
- CI/CD 파이프라인 구성
- 사용자 대면 UI 개발
- Windows/Linux 플랫폼 검증
- Claude Agent SDK (Python/TypeScript) 통합 -- subprocess 경로만 검증
