# SPEC-M1-001: M1 Working Shell — Multi-Workspace + UI Shell

---
id: SPEC-M1-001
version: 1.1.0
status: completed
created: 2026-04-11
updated: 2026-04-13
author: MoAI (manager-spec)
priority: High
issue_number: null
---

## HISTORY

| 버전 | 날짜 | 변경 내용 |
|------|------|-----------|
| 1.0.0 | 2026-04-11 | 초안 작성. M0 conditional GO 기반, M0 carry-over 항목 포함 |
| 1.1.0 | 2026-04-13 | 구현 완료. 조건부 GO. 30 tasks, 186 tests, 7 commits. |

---

## 1. 개요

MoAI Studio 의 "Working Shell" 마일스톤. M0 에서 검증된 Rust core + Swift FFI 기반 위에 **다중 워크스페이스 관리, 기본 UI shell (사이드바 + 터미널), Claude subprocess 전체 스택 통합**을 구현한다. 사용자가 앱을 열고, 워크스페이스를 생성/전환/삭제하며, Claude 와 대화할 수 있는 최소 완성 형태를 달성한다.

**성공 기준**: 앱 실행 -> 사이드바에서 워크스페이스 목록 확인 -> "New Workspace" -> Claude subprocess spawn -> 터미널 surface 에 zsh 표시 -> 사용자 메시지 전송 -> assistant 응답 실시간 표시 -> 다른 워크스페이스로 전환 -> 이전 워크스페이스 삭제.

**선행 조건**: SPEC-M0-001 conditional GO 완료. 103 tests, 12 crates 동작 검증.

**M0 carry-over 항목**:
- GhosttyKit xcframework (Metal Toolchain 블로커)
- SwiftUI windowed app (Xcode 프로젝트 필요)
- swift-bridge 전환 (현재 C ABI 사용)
- Full E2E: UI -> Claude -> response display

**참조 문서**:
- `DESIGN.v4.md` 3.1-3.2, 4.1-4.3, 7.1-7.3
- `SPEC-M0-001/spec.md` (M0 산출물 기준)
- `.moai/project/product.md` 4, 7 (핵심 기능, 비기능 요구사항)
- `.moai/project/tech.md` 2, 3 (Swift/Rust 스택)
- `.moai/project/structure.md` 4 (목표 모노레포 트리)

---

## 2. M0 carry-over 제약

M0 에서 미완료된 항목이 M1 의 선행 작업으로 포함된다.

| # | 분류 | 제약 | 적용 대상 |
|---|------|------|-----------|
| CO-1 | 빌드 | GhosttyKit xcframework: Metal Toolchain 설치 후 `zig build -Demit-xcframework=true` 성공 필요 | RG-M1-2 |
| CO-2 | UI | SwiftUI windowed app: Xcode 프로젝트 또는 xcodegen 기반 빌드 체인 구성 필요 | RG-M1-1 |
| CO-3 | FFI | swift-bridge 전환: 현재 C ABI (`#[no_mangle] extern "C"`) 에서 `#[swift_bridge::bridge]` 로 마이그레이션 | RG-M1-3 |
| CO-4 | E2E | Full E2E: UI -> Rust FFI -> Claude subprocess -> stream-json -> UI 표시 전체 파이프라인 미검증 | RG-M1-5 |

---

## 3. 요구사항 그룹 (EARS 형식)

### RG-M1-1: SwiftUI Windowed App (M0 carry-over)

**[Event-Driven]** 앱이 실행되면 (When), MoAI Studio 는 메인 윈도우를 **표시해야 한다** (shall display). 메인 윈도우는 사이드바 (워크스페이스 목록) + 콘텐츠 영역으로 구성된다.

**[Ubiquitous]** Xcode 프로젝트 (또는 xcodegen spec) 는 `MoAI Studio.app` bundle 로 macOS 14+ 를 타겟으로 **빌드되어야 한다** (shall build).

**[Ubiquitous]** 앱 윈도우는 `NavigationSplitView` 또는 `NSSplitView` 기반으로 사이드바와 콘텐츠 영역을 **분리해야 한다** (shall separate).

**[Event-Driven]** 앱이 종료 후 재실행되면 (When), 이전 윈도우 크기와 사이드바 너비를 **복원해야 한다** (shall restore).

**산출물**: `app/` Xcode 프로젝트, `MoAI Studio.app` 빌드 성공, 메인 윈도우 표시

---

### RG-M1-2: GhosttyKit Terminal Surface (M0 carry-over)

**[Event-Driven]** `scripts/build-ghostty-xcframework.sh` 가 실행되면 (When), Metal Toolchain 이 설치된 환경에서 `GhosttyKit.xcframework` 를 **빌드해야 한다** (shall build). [CO-1]

**[Event-Driven]** 워크스페이스의 콘텐츠 영역이 활성화되면 (When), GhosttyKit 을 사용하여 단일 터미널 surface 를 **생성해야 한다** (shall create).

**[State-Driven]** 터미널 surface 가 활성 상태인 동안 (While), GhosttyKit Metal 렌더링으로 zsh shell 을 60fps@4K 해상도에서 **표시해야 한다** (shall render).

**[If-Then]** Metal Toolchain 이 설치되지 않은 환경에서 GhosttyKit 빌드가 실패하면 (If), 빌드 스크립트는 설치 안내 메시지를 출력하고 비정상 종료 코드를 **반환해야 한다** (shall return).

**[If-Then]** GhosttyKit 초기화가 실패하면 (If), 콘텐츠 영역은 "Terminal unavailable" 메시지와 재시도 버튼을 **표시해야 한다** (shall display).

**산출물**: `GhosttyKit.xcframework` 빌드 성공, 터미널 surface 에 zsh 렌더링 확인

---

### RG-M1-3: swift-bridge FFI 전환

**[Ubiquitous]** Rust core 의 FFI 경계는 C ABI (`#[no_mangle] extern "C"`) 대신 `swift-bridge` 의 `#[swift_bridge::bridge]` 매크로를 **사용해야 한다** (shall use). [CO-3]

**[Ubiquitous]** swift-bridge FFI 는 최소한 다음 기능을 **노출해야 한다** (shall expose):
- `RustCore::new()` -> 초기화 (tokio runtime spawn 포함)
- `RustCore::version()` -> 버전 문자열 반환
- `RustCore::create_workspace(config: WorkspaceConfig)` -> WorkspaceId 반환
- `RustCore::delete_workspace(id: WorkspaceId)` -> 삭제 결과
- `RustCore::list_workspaces()` -> 워크스페이스 목록
- `RustCore::send_user_message(workspace_id, message)` -> 메시지 전송
- `RustCore::subscribe_events(workspace_id)` -> 이벤트 스트림

**[Event-Driven]** Swift 측에서 `subscribe_events` 를 호출하면 (When), Rust core 는 해당 워크스페이스의 `SDKMessage` 이벤트를 콜백 또는 async stream 으로 **전달해야 한다** (shall deliver).

**[Ubiquitous]** Swift 측 `@Observable` ViewModel 패턴과 호환되는 async/callback 기반 FFI 를 **제공해야 한다** (shall provide).

**[Ubiquitous]** FFI call overhead 는 호출 당 1ms 미만을 **유지해야 한다** (shall maintain).

**산출물**: `core/crates/moai-ffi/src/lib.rs` (swift-bridge 정의), `app/Sources/Bridge/` (생성된 Swift 바인딩)

---

### RG-M1-4: Workspace Lifecycle 통합

**[Event-Driven]** 사용자가 "New Workspace" 를 요청하면 (When), `RootSupervisor` 는 새 `WorkspaceSupervisor` 를 **생성해야 한다** (shall create). 생성 과정:
1. `moai-store` 에 workspace 레코드 삽입
2. `moai-git` 로 git worktree 초기화 (project git root 에서)
3. `moai-fs` 로 워크스페이스 경로 감시 시작
4. `moai-claude-host` 로 Claude subprocess spawn
5. workspace 상태를 `Starting` -> `Running` 으로 전환

**[Event-Driven]** 사용자가 워크스페이스 삭제를 요청하면 (When), `RootSupervisor` 는 해당 워크스페이스의 Claude subprocess 를 종료하고, git worktree 를 정리하고, store 레코드를 삭제하고, 감시를 중단**해야 한다** (shall cleanup).

**[Event-Driven]** 앱이 재시작되면 (When), `moai-store` 에서 이전 워크스페이스 목록을 로드하고 UI 에 **복원해야 한다** (shall restore). Claude subprocess 는 사용자가 해당 워크스페이스를 활성화할 때 다시 spawn 한다 (lazy restart).

**[State-Driven]** 워크스페이스가 `Running` 상태인 동안 (While), `moai-fs` 는 워크스페이스 디렉토리의 파일 변경을 감지하여 EventBus 에 **발행해야 한다** (shall publish).

**[If-Then]** Claude subprocess 가 비정상 종료하면 (If), 워크스페이스 상태를 `Error` 로 전환하고 사이드바에 오류 아이콘을 **표시해야 한다** (shall display). 사용자가 재시작 버튼을 클릭하면 subprocess 를 다시 spawn 한다.

**[Ubiquitous]** `moai-store` 는 `workspaces` 테이블에 `id, name, project_path, worktree_path, status, spec_id, claude_session_id, created_at, updated_at` 를 **저장해야 한다** (shall persist).

**[Ubiquitous]** workspace 상태는 `Created`, `Starting`, `Running`, `Paused`, `Error`, `Deleted` 6가지를 **지원해야 한다** (shall support).

**산출물**: `moai-supervisor` multi-workspace orchestration, `moai-store` workspace CRUD, `moai-git` worktree 관리, `moai-fs` 파일 감시

---

### RG-M1-5: Claude Subprocess Full Integration

**[Event-Driven]** 워크스페이스가 `Starting` 상태로 전환되면 (When), `moai-claude-host` 는 다음 인자로 Claude subprocess 를 **spawn 해야 한다** (shall spawn):
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

**[Event-Driven]** Claude subprocess stdout 에서 stream-json 메시지가 수신되면 (When), `moai-stream-json` 이 `SDKMessage` 로 디코딩하고 `moai-ide-server` 에 전달하여 **처리해야 한다** (shall process).

**[Event-Driven]** 사용자 메시지가 전송되면 (When), `moai-claude-host` 는 stdin 을 통해 `SDKUserMessage` JSON 을 **전송해야 한다** (shall send).

**[Complex]** MCP 서버가 실행 중인 상태에서 (While), Claude subprocess 에 MCP 도구 호출 프롬프트가 전달되면 (When), `moai-ide-server` 는 도구 호출 -> 처리 -> 결과 반환의 full round-trip 을 **완료해야 한다** (shall complete).

**[Event-Driven]** Claude 가 hook 이벤트를 발생시키면 (When), plugin 의 `hooks/hooks.json` 설정에 따라 HTTP POST 가 `moai-hook-http` 엔드포인트에 수신되고 EventBus 에 **발행되어야 한다** (shall be published).

**[Ubiquitous]** 각 워크스페이스는 독립된 Claude subprocess 인스턴스를 **소유해야 한다** (shall own). 워크스페이스 간 subprocess 공유는 금지.

**산출물**: E2E 통합: UI -> FFI -> Claude subprocess -> stream-json -> EventBus -> UI 표시

---

### RG-M1-6: Sidebar + Content Layout

**[Ubiquitous]** 메인 윈도우 레이아웃은 사이드바 (왼쪽) + 콘텐츠 영역 (오른쪽) 의 2-pane 구조를 **유지해야 한다** (shall maintain).

**[Ubiquitous]** 사이드바는 현재 프로젝트의 모든 워크스페이스를 목록으로 **표시해야 한다** (shall display). 각 항목은 워크스페이스 이름, 상태 아이콘 (Starting: 스피너, Running: 녹색 원, Error: 빨간 원, Paused: 회색 원) 을 포함한다.

**[Event-Driven]** 사용자가 사이드바에서 워크스페이스를 선택하면 (When), 콘텐츠 영역은 해당 워크스페이스의 터미널 surface 로 **전환해야 한다** (shall switch).

**[Event-Driven]** 사용자가 사이드바 하단의 "+" 버튼을 클릭하면 (When), 새 워크스페이스 생성 플로우를 **시작해야 한다** (shall initiate). 최소한 워크스페이스 이름 입력 필드를 포함한다.

**[Event-Driven]** 사용자가 사이드바 항목을 우클릭하면 (When), 컨텍스트 메뉴에 "Rename", "Restart Claude", "Delete" 옵션을 **표시해야 한다** (shall show).

**[State-Driven]** 활성 워크스페이스가 없는 상태에서 (While), 콘텐츠 영역은 환영 메시지와 "Create Workspace" 버튼을 **표시해야 한다** (shall display).

**[Ubiquitous]** 사이드바 너비는 사용자 드래그로 조정 가능하고 200px ~ 400px 범위를 **유지해야 한다** (shall maintain). 기본값 250px.

**산출물**: 사이드바 SwiftUI 뷰, 워크스페이스 목록, 상태 아이콘, 콘텐츠 영역 전환

---

### RG-M1-7: Plugin Auto-Installation

**[Event-Driven]** 앱이 최초 실행되면 (When), `moai-plugin-installer` 는 `plugin/` 디렉토리의 내용을 `~/.claude/plugins/moai-studio@local/` 에 **복사해야 한다** (shall copy).

**[Event-Driven]** 플러그인 설치 후 (When), `moai-plugin-installer` 는 `plugin.json` 과 `hooks/hooks.json` 의 무결성을 **검증해야 한다** (shall verify). 필수 필드 존재 여부와 JSON 파싱 성공 확인.

**[Event-Driven]** 앱이 업데이트 후 실행되면 (When), `moai-plugin-installer` 는 번들 내 플러그인 버전과 설치된 버전을 비교하고 필요 시 **업데이트해야 한다** (shall update).

**[If-Then]** 플러그인 설치 경로에 쓰기 권한이 없으면 (If), `moai-plugin-installer` 는 오류를 로그에 기록하고 사용자에게 수동 설치 안내를 **표시해야 한다** (shall display).

**[Ubiquitous]** 플러그인 디렉토리 구조는 다음을 **포함해야 한다** (shall include):
- `.claude-plugin/plugin.json`
- `hooks/hooks.json` (E5 errata 준수: `{"hooks": {...}}` wrapper)
- `mcp-config.json`

**산출물**: `moai-plugin-installer` 자동 설치 로직, 버전 비교, 무결성 검증

---

## 4. 산출물 요약

| # | 산출물 | 경로 |
|---|--------|------|
| 1 | SwiftUI windowed macOS app | `app/` (Xcode 프로젝트), `MoAI Studio.app` |
| 2 | GhosttyKit xcframework | `app/Frameworks/GhosttyKit.xcframework` |
| 3 | swift-bridge FFI 정의 | `core/crates/moai-ffi/src/lib.rs`, `app/Sources/Bridge/` |
| 4 | Multi-workspace supervisor | `core/crates/moai-supervisor/` |
| 5 | Workspace CRUD + 상태 관리 | `core/crates/moai-store/` |
| 6 | Git worktree 통합 | `core/crates/moai-git/` |
| 7 | 파일 감시 통합 | `core/crates/moai-fs/` |
| 8 | Plugin auto-installer | `core/crates/moai-plugin-installer/` |
| 9 | 사이드바 + 콘텐츠 UI | `app/Sources/Shell/` |
| 10 | E2E 통합 (UI -> Claude -> UI) | 통합 테스트 |

---

## 5. 비기능 요구사항

| 항목 | 목표 |
|------|------|
| 콜드 스타트 (M1 MacBook) | < 1.0s (M1 목표, M6 에서 < 0.6s) |
| 터미널 렌더링 | 60fps@4K (GhosttyKit Metal) |
| FFI call overhead | < 1ms per call |
| Hook HTTP loopback latency | < 10ms P95 |
| MCP tool round-trip | < 50ms (M1 목표, M4 에서 < 30ms) |
| Workspace 생성 시간 | < 3s (Claude subprocess spawn 포함) |
| Workspace 전환 시간 | < 100ms (UI 전환) |
| 동시 워크스페이스 | 최소 4개 안정 동작 (M6 에서 16+) |
| 메모리 사용량 | < 400MB (4 workspace, 4 PTY) |
| Store 쿼리 성능 | < 5ms (workspace CRUD) |
| Rust core `cargo check` | 0 errors, 0 warnings |
| Xcode 빌드 | 0 errors |

---

## 6. 리스크

| 리스크 | 확률 | 영향 | 대응 |
|--------|------|------|------|
| GhosttyKit Metal Toolchain 설치 실패 반복 | 중간 | 높음 | Xcode GUI 수동 설치, 대안: libghostty-spm 패키지, NSTextView fallback |
| swift-bridge async FFI 불안정 | 중간 | 높음 | sync wrapper + DispatchQueue.global() 패턴, 최소 FFI surface 유지 |
| 다중 Claude subprocess 메모리 압박 | 중간 | 중간 | 4개 제한 (M1), lazy subprocess spawn, 비활성 workspace subprocess 종료 |
| GhosttyKit API 변경 (submodule pin) | 낮음 | 낮음 | 특정 tag 에 submodule pin, API wrapper 추상화 |
| NSSplitView + SwiftUI 통합 이슈 | 중간 | 중간 | NavigationSplitView 우선, AppKit bridge fallback |
| rusqlite WAL concurrent access | 낮음 | 중간 | r2d2 connection pool, WAL 모드 검증 테스트 |
| moai-git worktree 충돌 | 낮음 | 중간 | worktree prune 정기 실행, lock 파일 정리 |

---

## 7. 의존성

| 외부 의존성 | 버전 | 용도 |
|-------------|------|------|
| `claude` CLI | >= 2.1.101 | subprocess host |
| `zig` | 0.15.2 | Ghostty xcframework 빌드 |
| `swift-bridge` | latest | Rust <-> Swift FFI (M0 C ABI 대체) |
| `rmcp` | 0.9.x | MCP server SDK |
| `axum` | 0.8.x | HTTP / Streamable HTTP transport |
| `tokio` | 1.x | async runtime, actor supervision |
| `rusqlite` | latest | WAL store |
| `r2d2` | latest | connection pool |
| `refinery` | latest | DB migration |
| `git2` | latest | git worktree 관리 |
| `notify` | 7.x | 파일 시스템 감시 |
| `ring` | latest | auth token 생성 |
| Xcode | 15+ | macOS app 빌드 |
| macOS | 14+ | 타겟 플랫폼 |
| M0 산출물 | 103 tests, 12 crates | 기반 코드 |

---

## 8. Exclusions (What NOT to Build)

M1 은 "Working Shell" 이므로 다음을 명시적으로 제외한다:

1. **Pane splitting / binary tree NSSplitView** — M1 에서는 사이드바 + 단일 콘텐츠 영역만. 자유 분할은 M2+
2. **Code Viewer / Markdown / Image / Browser / FileTree surface** — M1 에서는 Terminal surface 만. 나머지 9개 surface 는 M2-M5
3. **Kanban board / Agent Run Viewer / Memory / InstructionsGraph** — M5
4. **LSP 통합** (`.lsp.json` 6개 언어) — M4
5. **Native permission dialog** — M4. M1 에서는 `--permission-mode acceptEdits` 유지
6. **Auto-update (Sparkle)** — M6
7. **16+ 에이전트 동시 세션** — M1 에서는 4개 제한. 16+ 는 M6 stress test
8. **Cost tracking / token budget UI** — M5
9. **lockfile + WS IDE 코드 인텔리전스** (보조 통합 경로) — M4. M1 에서는 `--mcp-config` SSE/Streamable HTTP 만
10. **CI/CD pipeline** — M2 이후 점진적 구축
11. **Command Palette** — M2
12. **Output Styles / forceForPlugin** — M4
13. **Tab UI** (다중 탭 인터페이스) — M2. M1 에서는 사이드바 선택으로 워크스페이스 전환
14. **Linux / WSL / Windows 지원** — 영구 제외 (DESIGN.v4 1.3)
15. **Onboarding / 첫 실행 마법사** — M4. M1 에서는 plugin 자동 설치만
