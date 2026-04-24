# SPEC-M2-003: Surface State Persistence -- 재시작 후 완전 복원 (ARCHIVED — v2 Swift design)

> **⚠️ SUPERSEDED (2026-04-24)**: 본 SPEC 은 Swift/AppKit 기반 v2 아키텍처에서 작성되었다. 2026-04-21 v3 pivot (GPUI + libghostty-vt) 으로 모든 참조 경로 (`core/crates/moai-store/`, `app/Sources/Shell/**`) 가 `archive/swift-legacy/` 로 이관되었으며, "Surface State Persistence" 기능 자체는 **SPEC-V3-003 MS-3 Persistence (T12-T13)** 에서 pane tree + tab 목록 + cwd + last focused pane 의 복원 범위로 재정의된다.
>
> **AC-P-NN 네임스페이스 주의**: 본 SPEC 의 `AC-P-1 ~ AC-P-10` 은 SPEC-V3-003 의 `AC-P-1 ~ AC-P-27` 과 ID 충돌. SPEC-V3-003 v1.1.0 의 AC-P 가 canonical. 본 SPEC 의 AC-P 는 **historical only**.
>
> **후속 조치 (user 결정)**: (b) `status: archived-v2-design` 로 동결 채택 (2026-04-24 plan-auditor FAILED 판정 대응).

---
id: SPEC-M2-003
version: 1.1.0-archived
status: archived-v2-design
created_at: 2026-04-16
updated_at: 2026-04-24
superseded_by: SPEC-V3-003 (MS-3 Persistence, pane tree + tab list + cwd + focus)
author: MoAI (manager-spec)
priority: Medium
issue_number: 0
labels: [archived, v2-swift, persistence, superseded]
revision: v1.1.0-archived (plan-auditor 2026-04-24 감사 FAILED — v3 pivot 으로 archive 처리)
---

## HISTORY

| 버전 | 날짜 | 변경 내용 |
|------|------|-----------|
| 1.0.0 | 2026-04-16 | 초안 작성. M2 완료 보고서 "알려진 제한 사항" 4건 (P-5 statePathCache, P-6 FileTree 재귀 expand, P-7 Browser URL 영속, P-8 resolveWorkspacePath) 을 SPEC 화. |
| 1.1.0-archived | 2026-04-24 | plan-auditor 감사 FAILED — C-001 (AC-P-NN 네임스페이스 충돌 with SPEC-V3-003) + C-002 (archive 경로 drift) 이유로 archive 처리. status `draft → archived-v2-design`. frontmatter `superseded_by: SPEC-V3-003` 추가. 본 SPEC 의 5 RG / 10 AC 는 historical reference 로만 보존. persistence 기능은 SPEC-V3-003 MS-3 로 계승. |

---

## 1. 개요

MoAI Studio 의 "Surface State Persistence" 마일스톤. SPEC-M2-001 (M2 Viewers) 에서 구현된 4 종 Surface (FileTree/Markdown/Image/Browser) 는 pane tree 와 탭 레이아웃 수준에서는 재시작이 복원되지만, **각 Surface 의 내부 상태(열려 있던 파일 경로, 탭별 URL, 디렉토리 확장 상태)가 영속되지 않아 사용자가 재시작 후 "빈 탭" 으로 복원되는 UX 결함**이 존재한다. 본 SPEC 은 `surfaces.state_json` 컬럼의 스키마를 surface kind 별로 확장 정의하고, Swift 측 영속 경로를 메모리 캐시에서 DB 왕복으로 교체하며, FileTree 재귀 expand 로직과 워크스페이스 path @Environment 주입을 통해 **재시작 후 완전한 Surface 상태 복원을 보장**한다.

**성공 기준**: 앱 실행 → 워크스페이스 열기 → FileTree pane 에서 `docs/` 디렉토리 expand → `docs/README.md` 를 Markdown 탭으로 open → 다른 탭에서 Browser surface 로 `http://localhost:5173` 열기 → 이미지 파일을 Image 탭으로 open → Cmd+Q 종료 → 재실행 → **(1) FileTree 가 홈 디렉토리가 아닌 실제 워크스페이스 루트를 표시**, **(2) `docs/` 가 여전히 expand 상태**, **(3) Markdown 탭이 `docs/README.md` 렌더 상태로 복원**, **(4) Browser 탭이 `http://localhost:5173` 로 로드**, **(5) Image 탭이 원본 이미지를 표시**.

**선행 조건**: SPEC-M2-001 completed (233 Rust + 106 Swift tests, DB schema V3). 본 SPEC 은 V3 위에 V4 스키마 확장을 얹거나 기존 `surfaces.state_json` 컬럼의 JSON payload 계약을 확장하는 방식으로 **하위 호환(backwards-compatible) 진화**를 수행한다.

**참조 문서**:
- `.moai/specs/SPEC-M2-001/spec.md` RG-M2-2, RG-M2-4, RG-M2-5, RG-M2-7
- `.moai/specs/SPEC-M2-001/m2-completion-report.md` §알려진 제한 사항 (P-5~P-8 원본 설명)
- `core/crates/moai-store/migrations/V3__panes_surfaces.sql` (기존 스키마)
- `app/Sources/Shell/Tabs/TabBarViewModel.swift` (`statePathCache`, `stateJson` 빌드)
- `app/Sources/Shell/Splits/PaneSplitView.swift` (`resolveWorkspacePath()` 폴백)
- `app/Sources/Surfaces/FileTree/FileTreeSurface.swift` (루트 한 레벨 리스팅)
- `app/Sources/Bridge/RustCore+Generated.swift` (`WorkspaceSnapshot` — path 필드 미노출)

---

## 2. 요구사항 그룹 (EARS 형식)

### RG-M2-P-1: state_json 스키마 확장 (Surface kind 별 JSON 계약)

**[Ubiquitous]** `surfaces.state_json` 컬럼은 surface kind 별로 **정의된 JSON 스키마를 따라야 한다** (shall conform). V3 컬럼 타입(`TEXT`)은 유지하며, 내용만 구조화한다. 빈 문자열 `""` 과 `NULL` 은 "미상태" 로 **동일하게 취급해야 한다** (shall treat equivalently).

**[Ubiquitous]** 각 surface kind 별 JSON 스키마는 다음과 **일치해야 한다** (shall match):

| kind | state_json 스키마 | 필수 키 | 선택 키 |
|------|-------------------|---------|---------|
| `terminal` | `{}` 또는 `""` | — | — |
| `filetree` | `{"workspace_path": "<abs>", "expanded_paths": ["<rel>", ...]}` | `workspace_path` | `expanded_paths` |
| `markdown` | `{"path": "<abs>", "scroll_offset": <int?>}` | `path` | `scroll_offset` |
| `image` | `{"path": "<abs>", "zoom": <float?>}` | `path` | `zoom` |
| `browser` | `{"url": "<url>"}` | `url` | — |

**[Ubiquitous]** 모든 선택 키는 **누락 시 기본값으로 폴백해야 한다** (shall fall back). 예: `scroll_offset` 누락 → 0, `zoom` 누락 → 1.0, `expanded_paths` 누락 → 빈 집합. 알 수 없는 키는 **무시하고 로그만 남겨야 한다** (shall ignore and log).

**[State-Driven]** DB schema_version 이 4 미만인 동안 (While version < 4), 시스템은 `migrations/V4__state_json_schema.sql` 을 **적용해야 한다** (shall apply). V4 는 기존 `state_json` 행을 스캔하여 `terminal` 이 아닌 surface 의 비어 있거나 파싱 불가한 payload 를 **`{}` 로 정규화해야 한다** (shall normalize). 행 손실은 **발생해서는 안 된다** (shall not occur).

**[If]** JSON 파싱이 실패하면 (If payload invalid JSON), 시스템은 해당 surface 를 "빈 상태" 로 복원하고 오류를 `hook_events` 에 **기록해야 한다** (shall record). 탭 자체는 닫지 **말아야 한다** (shall not close).

**산출물**: `core/crates/moai-store/migrations/V4__state_json_schema.sql`, Rust `SurfaceState` enum + `serde_json` 파서, FFI 경계 스키마 문서 (`docs/state_json_schema.md`)

---

### RG-M2-P-2: Markdown / Image Surface 파일 경로 영속 (P-5 해소)

**[Ubiquitous]** `TabBarViewModel.statePathCache` 는 **DB 왕복으로 교체되어야 한다** (shall be replaced). 순수 메모리 캐시는 **재시작 시 손실**되므로, 파일 경로는 `surfaces.state_json` 에 **직접 영속해야 한다** (shall persist directly).

**[Event-Driven]** 사용자가 FileTree 에서 `.md` 또는 이미지 파일을 오픈하면 (When file opened), 시스템은 `bridge.createSurface(paneId, kind, stateJson, tabOrder)` 호출 시 `stateJson = {"path": "<abs path>"}` 로 **전달해야 한다** (shall pass). 이 경로는 **DB 에 즉시 커밋되어야 한다** (shall commit).

**[Event-Driven]** 앱 재시작 후 pane 이 로드될 때 (When pane restored), Swift 는 `bridge.listSurfaces(paneId)` 로 탭 목록을 받은 뒤 각 탭의 `state_json` 을 **파싱하여 SurfaceKind 별 ViewModel 에 주입해야 한다** (shall parse and inject). Markdown 탭은 `MarkdownViewModel(filePath:)`, Image 탭은 `ImageViewModel(filePath:)` 로 생성된다.

**[If]** 파일이 `state_json.path` 에 지정되어 있으나 실제 파일 시스템에 존재하지 않으면 (If file missing), Surface 는 **"File not found: <path>" placeholder 를 표시해야 한다** (shall show). 탭은 유지하되 상단에 **"다시 열기" / "탭 닫기" 액션을 제공해야 한다** (shall provide).

**[Ubiquitous]** `statePathCache` 필드 자체는 **삭제되거나 read-through cache 로 좁혀져야 한다** (shall be removed or narrowed). 모든 `statePath` 조회는 DB 가 **원천 소스(single source of truth)** 가 되어야 한다.

**산출물**: `app/Sources/Shell/Tabs/TabBarViewModel.swift` (statePathCache 제거/축소), `app/Sources/Shell/Splits/PaneSplitView.swift` (SurfaceRouter 에서 state_json 파싱), state_json 파싱 헬퍼 (`SurfaceStateDecoder.swift`)

---

### RG-M2-P-3: FileTree 재귀 expand + expansion state 영속 (P-6 해소)

**[Ubiquitous]** FileTree Surface 는 루트 한 레벨만이 아닌 **임의 깊이의 하위 디렉토리를 expand 할 수 있어야 한다** (shall be able to expand). 현재 `FileTreeViewModel.entries` 는 Rust FFI `listDirectoryJson(subpath: "")` 의 단일 호출 결과이나, 확장된 경로 집합에 대해서도 **하위 리스팅을 병합해야 한다** (shall merge).

**[Event-Driven]** 사용자가 디렉토리 행을 클릭하여 expand 하면 (When directory expanded), 시스템은 `bridge.listDirectoryJson(workspacePath, subpath: "<rel>")` 를 호출하여 자식 항목을 **로드하고 entries 에 삽입해야 한다** (shall load and insert). 삽입 위치는 부모 다음 연속 슬롯이며, depth 값은 부모 `depth + 1` 로 **설정되어야 한다** (shall set).

**[Event-Driven]** 사용자가 이미 expand 된 디렉토리를 collapse 하면 (When collapsed), 시스템은 해당 경로를 prefix 로 하는 모든 하위 entries 를 **제거해야 한다** (shall remove). expanded_paths 집합에서 **해당 경로 및 하위 경로를 제거한다**.

**[State-Driven]** expand/collapse 상태가 변경될 때마다 (While state mutating), 시스템은 `expanded_paths` 를 **1초 debounce 로 DB 에 flush 해야 한다** (shall flush debounced). 빈번한 UI 토글에 대한 DB write 폭주를 **방지해야 한다** (shall prevent).

**[State-Driven]** 앱이 종료 상태일 때 (While terminated), FileTree surface 의 `state_json.expanded_paths` 는 **보존되어야 한다** (shall be preserved). 재시작 시 FileTreeViewModel.load() 완료 후 **저장된 expanded_paths 각각에 대해 하위 리스팅을 재귀적으로 로드해야 한다** (shall recursively load).

**[If]** 저장된 expanded_paths 중 일부가 더 이상 존재하지 않으면 (If path deleted), 시스템은 해당 경로를 expanded 집합에서 **조용히 제거해야 한다** (shall silently prune). 사용자 알림은 **불필요하다** (not required).

**[Ubiquitous]** 재귀 expand 는 **한 번의 디렉토리 리스팅당 최대 5,000 항목**으로 **제한되어야 한다** (shall cap). 초과 시 상위 5,000 개만 표시하고 "…외 N 개 항목 생략" 메시지를 **표시해야 한다** (shall show).

**산출물**: `FileTreeViewModel` 에 `expandedPaths` 영속 로직 추가, Rust `list_directory_json` 의 subpath 파라미터 검증(`..` 경로 탈출 금지), debounced flush 헬퍼

---

### RG-M2-P-4: Browser URL 영속 (P-7 해소)

**[Ubiquitous]** `BrowserViewModel.currentURL` 은 **DB 에 영속되어야 한다** (shall persist). 현재 `DevServerDetector.detect()` 폴백만 존재하여 재시작 시 항상 감지된 localhost 포트로 리셋된다. 사용자가 명시적으로 입력한 URL 이 **우선되어야 한다** (shall take precedence).

**[Event-Driven]** 사용자가 URL bar 에 주소를 입력하고 Enter 를 누르면 (When URL submitted), 시스템은 `bridge.updateSurfaceStateJson(surfaceId, {"url": "<url>"})` 를 **호출해야 한다** (shall call). WKWebView 의 `didFinish` 네비게이션 콜백 또한 **최종 URL 로 state_json 을 갱신해야 한다** (shall update). 리다이렉션 후의 `webView.url` 이 기록된다.

**[Event-Driven]** Browser Surface 가 재시작 후 활성화되면 (When surface restored), 시스템은 `state_json.url` 이 존재하면 해당 URL 을 **먼저 로드해야 한다** (shall load first). 없을 때만 `DevServerDetector.detect()` **폴백을 시도해야 한다** (shall fall back).

**[If]** 저장된 URL 이 `http://` 또는 `https://` 스킴이 아니거나 URL 파싱에 실패하면 (If malformed), 시스템은 해당 URL 을 **무시하고 빈 상태로 시작해야 한다** (shall ignore). 에러는 로그에만 남긴다.

**[Ubiquitous]** URL flush 는 **debounce 없이 즉시 (on submit / on didFinish)** 수행해야 한다. Markdown/FileTree 와 달리 Browser 네비게이션은 빈도가 낮다.

**산출물**: `BrowserViewModel` 에 `updateStateJson` 훅 추가, `BrowserSurface` 초기화 시 state_json 주입 (현재 `init()` 은 무인자이므로 `init(stateJson: String?)` 으로 변경)

---

### RG-M2-P-5: Workspace path @Environment 주입 (P-8 해소)

**[Ubiquitous]** Swift 측 `WorkspaceSnapshot` (`app/Sources/Bridge/RustCore+Generated.swift`) 은 `projectPath: String` 필드를 **추가로 노출해야 한다** (shall expose). 현재는 `id`, `name`, `status` 3 개만 있으며 FileTree 가 홈 디렉토리로 폴백하는 근본 원인이다.

**[Ubiquitous]** Rust FFI 레이어는 `WorkspaceInfo` 를 Swift 로 전달할 때 `project_path` 를 **포함해야 한다** (shall include). `moai-supervisor::WorkspaceSnapshot.project_path` 는 이미 존재하므로 FFI wrapper 구조체만 확장한다.

**[Event-Driven]** 활성 워크스페이스가 변경되면 (When active workspace switched), 상위 SwiftUI 뷰 계층은 해당 `WorkspaceSnapshot` 을 **`@Environment(\.activeWorkspace)` 로 배포해야 한다** (shall publish). `EnvironmentKey` 정의와 `.environment()` 호출을 포함한다.

**[Ubiquitous]** `PaneSplitView.resolveWorkspacePath()` 는 **제거되거나 `@Environment(\.activeWorkspace)?.projectPath` 로 구현되어야 한다** (shall be replaced). `FileManager.default.homeDirectoryForCurrentUser.path` 로의 폴백은 **금지한다** (shall not fall back to home dir).

**[If]** 어떤 이유로든 `@Environment(\.activeWorkspace)` 가 `nil` 이면 (If unset), FileTreeSurface 는 **"워크스페이스가 선택되지 않음" placeholder 를 표시해야 한다** (shall show placeholder). 트리 렌더링은 **시도해서는 안 된다** (shall not attempt).

**산출물**: `WorkspaceSnapshot` 구조체 확장, `WorkspaceEnvironmentKey.swift` 신규, `PaneSplitView` resolveWorkspacePath 제거, FFI `ws_list_json` / `ws_current_json` project_path 포함

---

## 3. 수용 기준 (재시작 시나리오 기반)

### AC-P-1 (state_json 스키마 정규화)

**Given** 기존 V3 DB 가 존재하고 `surfaces.state_json` 에 일부 빈 문자열 / 일부 legacy `{"path":"..."}` payload 가 섞여 있다.
**When** V4 마이그레이션이 적용된다.
**Then**
- 모든 행의 `state_json` 이 kind 별 스키마로 정규화된다.
- 파싱 불가한 payload 는 `{}` 로 대체되고 `hook_events` 에 원본 payload 가 기록된다.
- V4 마이그레이션은 재실행 시에도 idempotent 하다 (두 번 적용 시 행 내용이 동일).

### AC-P-2 (Markdown/Image 파일 경로 영속)

**Given** 사용자가 `docs/ARCHITECTURE.md` 를 Markdown 탭으로 열고 `logo.png` 를 Image 탭으로 열었다.
**When** 앱을 Cmd+Q 로 종료하고 재실행한다.
**Then**
- Markdown 탭이 `docs/ARCHITECTURE.md` 를 렌더한 상태로 복원된다.
- Image 탭이 `logo.png` 를 표시한다.
- `TabBarViewModel.statePathCache` 는 빈 상태이거나 DB read-through 값으로 재구축된다.

### AC-P-3 (파일 삭제 시나리오)

**Given** Markdown 탭이 `state_json.path = "docs/OLD.md"` 로 저장되어 있으며, 재시작 전 파일이 삭제되었다.
**When** 앱이 재실행되고 해당 탭을 활성화한다.
**Then**
- Markdown 탭이 "File not found: docs/OLD.md" placeholder 를 표시한다.
- 탭은 자동으로 닫히지 않는다.
- "탭 닫기" 버튼으로 사용자가 명시적으로 닫을 수 있다.

### AC-P-4 (FileTree 재귀 expand 유지)

**Given** 사용자가 FileTree 에서 `app/Sources/Surfaces/` 와 `app/Sources/Surfaces/Markdown/` 를 차례로 expand 한다.
**When** 앱 재시작.
**Then**
- FileTreeSurface 가 재시작 후 두 디렉토리 모두 expand 상태로 표시된다.
- `expanded_paths` 가 `["app/Sources/Surfaces", "app/Sources/Surfaces/Markdown"]` 로 저장되었음이 DB 에서 확인된다.
- expand 해제 시 1 초 이내에 DB 가 flush 된다.

### AC-P-5 (FileTree 경로 소실 시 silent prune)

**Given** `state_json.expanded_paths = ["temp/scratch"]` 이 저장되어 있고, 재시작 전 `temp/scratch/` 가 삭제되었다.
**When** 앱 재실행.
**Then**
- FileTreeSurface 가 오류 없이 로드된다.
- 저장된 `expanded_paths` 에서 `temp/scratch` 가 조용히 제거된다.
- 다음 flush 시 DB 가 정리된 집합으로 갱신된다.

### AC-P-6 (Browser URL 복원)

**Given** 사용자가 Browser 탭 URL bar 에 `https://docs.rs/tokio` 를 입력하고 Enter.
**When** 앱 재시작.
**Then**
- Browser 탭이 `https://docs.rs/tokio` 로 로드된다.
- `DevServerDetector.detect()` 는 호출되지 않는다.

### AC-P-7 (Browser URL 누락 시 dev server 폴백)

**Given** `state_json = {}` (URL 미저장) 인 Browser 탭. localhost:3000 에 dev 서버가 실행 중.
**When** 앱 재시작 후 Browser 탭 활성화.
**Then**
- Browser 탭이 `http://localhost:3000` 으로 로드된다 (기존 SPEC-M2-001 동작 유지).

### AC-P-8 (FileTree 가 워크스페이스 루트 표시)

**Given** 사용자가 워크스페이스 `/Users/goos/projects/moai-studio` 를 열었다.
**When** FileTree pane 을 생성한다.
**Then**
- FileTreeViewModel.workspacePath == "/Users/goos/projects/moai-studio" 이다.
- `FileManager.default.homeDirectoryForCurrentUser.path` 가 사용되지 않는다 (코드 grep 으로 검증).
- 루트 항목이 워크스페이스의 `Cargo.toml`, `app/`, `core/` 등이다.

### AC-P-9 (워크스페이스 미선택 시 placeholder)

**Given** 어떤 워크스페이스도 활성화되지 않았다 (`@Environment(\.activeWorkspace) == nil`).
**When** FileTree pane 이 렌더링 시도된다.
**Then**
- FileTreeSurface 가 "워크스페이스가 선택되지 않음" placeholder 를 표시한다.
- `bridge.listDirectoryJson` 호출은 발생하지 않는다.

### AC-P-10 (하위 호환: V3 DB 로딩)

**Given** SPEC-M2-001 (V3) 만 적용된 DB 파일이 이미 존재하며 내부에 `state_json = ""` 행이 다수 있다.
**When** 본 SPEC 빌드의 앱이 해당 DB 를 연다.
**Then**
- V4 마이그레이션이 자동 적용된다.
- 기존 `state_json = ""` 행은 `{}` 로 정규화되지만, `pane_id`, `kind`, `tab_order`, `created_at` 등 다른 컬럼은 **변경되지 않는다**.

---

## 4. 마일스톤

### MS-1: Rust 스키마 + 마이그레이션 (Priority: High)

- V4 마이그레이션 SQL 작성 (`V4__state_json_schema.sql`)
- `SurfaceState` enum 정의 (Rust serde 모델: Terminal/FileTree/Markdown/Image/Browser 5 variants)
- `SurfaceDao::update_state_json_typed(id, SurfaceState)` 헬퍼
- V3 → V4 데이터 정규화 유닛 테스트 (빈 문자열, NULL, 손상 JSON 세 케이스)
- FFI wrapper 에 `project_path` 필드 추가 (P-8 선행)

**완료 기준**: `cargo test --workspace` 통과 (기존 233 개 + 신규 최소 8 개), V4 idempotent 테스트 포함.

### MS-2: Surface 별 영속 로직 (P-5, P-7) (Priority: High)

- `TabBarViewModel.statePathCache` 제거 또는 read-through 축소
- `SurfaceStateDecoder.swift` 신규 — state_json → typed Swift 구조체
- `MarkdownSurface(stateJson:)`, `ImageSurface(stateJson:)`, `BrowserSurface(stateJson:)` 이니셜라이저 도입
- `BrowserViewModel` 에 `onURLSubmit` / `onDidFinish` state_json flush
- 파일 누락 placeholder UI 구현

**완료 기준**: AC-P-2, AC-P-3, AC-P-6, AC-P-7 시나리오를 Swift XCTest 통합 테스트로 통과.

### MS-3: FileTree 재귀 expand (P-6) (Priority: Medium)

- `FileTreeViewModel` 에 subpath 기반 병합 리스팅 로직
- `expandedPaths` 집합 → state_json flush (1 초 debounce)
- 재시작 시 expanded_paths 재귀 로드
- Rust `list_directory_json` subpath 파라미터 검증 (경로 탈출 `..` 방어)
- 5,000 항목 cap 구현

**완료 기준**: AC-P-4, AC-P-5 시나리오 통과. `FileTreeViewModelTests` 에 재귀 expand + silent prune 테스트 추가.

### MS-4: Workspace path @Environment 주입 (P-8) (Priority: Medium)

- `WorkspaceSnapshot` 에 `projectPath` 필드 추가 (FFI wrapper 확장)
- `WorkspaceEnvironmentKey.swift` 신규 (`@Environment(\.activeWorkspace)`)
- App-level scene 에서 활성 워크스페이스 주입
- `PaneSplitView.resolveWorkspacePath()` 제거
- nil 시 placeholder UI

**완료 기준**: AC-P-8, AC-P-9 통과. `grep -r "homeDirectoryForCurrentUser" app/Sources/Shell/Splits/` 결과 0 건.

### MS-5: 통합 검증 + 재시작 E2E (Priority: Medium)

- Rust 쪽 `e2e_persistence.rs` 통합 테스트 (store 재오픈 시 state_json 무결성)
- Swift XCTest 에서 TabBarViewModel 재구성 시나리오 (Mock bridge 기반)
- UITest (opt-in, CI skip 허용): 4 surface 열고 종료 → 재시작 → 복원 검증
- 문서화: `docs/state_json_schema.md`

**완료 기준**: 전체 성공 기준 (§1) 시나리오가 수동 체크리스트 + 자동 E2E 로 통과.

---

## 5. 비기능 요구사항 (NFR)

| 항목 | 목표 | 측정 방법 |
|------|------|-----------|
| V4 마이그레이션 idempotency | 반복 실행 시 행 동일 | 동일 DB 에 migrate 2 회 실행 후 byte-level diff |
| 기존 데이터 손실 | 0 행 | V3 DB 스냅샷 vs V4 적용 후 row count 일치 |
| FFI overhead (state_json 왕복) | 기존 대비 +10% 이내 | FFIBenchmarkTests (P95 여전히 <1ms) |
| expanded_paths debounce flush | <1s (마지막 변경 이후) | Swift Task sleep 측정 |
| FileTree 재귀 expand | <300ms (깊이 5, 1,000 파일) | listDirectory 응답 시간 누적 |
| Browser state_json flush | <50ms (URL submit 시점 기준) | XCTest measure |
| 재시작 후 Surface 복원 | 100% 일치 (경로, URL, expand 상태) | E2E 수동/자동 체크리스트 |
| DB 스키마 하위 호환 | V3 DB 로 기동 가능 | 저장된 V3 스냅샷 fixture 로 테스트 |

---

## 6. 의존성 및 제약

### 6.1 기술적 의존성

- **SPEC-M2-001 (completed)**: V3 스키마 (`panes`, `surfaces` 테이블), `SurfaceDao` CRUD, SurfaceProtocol 기반 4 개 surface 구현체. 본 SPEC 은 이 위에서만 동작한다.
- **moai-store migration 관례**: `schema_version` 테이블 기반 순차 버전 + `include_str!("../migrations/V{N}.sql")` 패턴. V4 는 이 관례를 **유지한다**.
- **swift-bridge**: `WorkspaceInfo` 구조체 확장 시 Swift/Rust 양쪽 코드 재생성 필요. Vectorizable 관련 workaround 는 SPEC-M2-001 MS-2 에서 JSON FFI 로 해소되었으므로 추가 작업 없음.
- **rusqlite WAL 모드**: 기존과 동일. 마이그레이션 중 writer 단일 뮤텍스 직렬화 유지.
- **Foundation AttributedString**: Markdown 렌더는 변경되지 않음. 단지 경로가 영속될 뿐.

### 6.2 범위 제약

- 본 SPEC 은 `terminal` surface 의 state_json 을 변경하지 **않는다**. 터미널 세션 복원은 GhosttyHost wiring (M3 범위) 이후에 별도 SPEC 에서 다룬다.
- 본 SPEC 은 `code`, `agent_run`, `kanban`, `memory`, `instructions_graph` surface 의 state_json 을 정의하지 **않는다**. 해당 surface 들이 구현되는 마일스톤에서 각자 확장한다.
- DB migration 방향은 **forward-only**. 다운그레이드(V4 → V3) 는 지원하지 않는다. 사용자가 이전 버전으로 되돌릴 경우 DB 파일을 삭제하고 재생성하도록 안내한다.

### 6.3 외부 제약

- 파일 시스템 경로는 **절대 경로** 로 저장한다. 워크스페이스 이동 시 경로가 깨지는 것은 알려진 한계이며 본 SPEC 범위 외.
- macOS sandbox: 사용자가 선택하지 않은 디렉토리에 대한 Security-Scoped Bookmark 는 본 SPEC 에서 다루지 **않는다** (별도 보안 SPEC).

---

## 7. 테스트 전략

### 7.1 Rust (moai-store, moai-ffi)

- `core/crates/moai-store/tests/state_json_schema.rs` (신규)
  - `state_json` 정규화 (빈 문자열 → `{}`, NULL → `{}`, 손상 JSON → `{}` + 로그)
  - V4 마이그레이션 idempotency (2 회 적용 후 동일)
  - V3 → V4 forward migration 시 row count 보존
- `core/crates/moai-store/tests/surface_state_crud.rs` (확장)
  - `update_state_json_typed(Markdown { path })` 왕복
  - `update_state_json_typed(FileTree { expanded_paths })` 왕복
  - `update_state_json_typed(Browser { url })` 왕복
- `core/crates/moai-ffi/tests/e2e_persistence.rs` (신규)
  - Store 오픈 → surface 생성 → state_json 저장 → store close → 재오픈 → state_json 동일 검증

### 7.2 Swift (app/Tests)

- `TabBarViewModelPersistenceTests.swift` (신규)
  - Mock bridge: createSurface 시 stateJson 이 저장되고, listSurfaces 에서 동일 값 반환
  - statePathCache 제거 후에도 activeStatePath() 가 DB 값으로 복구
- `SurfaceStateDecoderTests.swift` (신규)
  - 각 kind 별 JSON 파싱 round-trip
  - 손상 JSON → 빈 상태 폴백
- `FileTreeViewModelRecursiveTests.swift` (신규)
  - expandedPaths 저장 → 재구축 시 재귀 리스팅 호출
  - 삭제된 경로 silent prune
  - 5,000 항목 cap
- `BrowserViewModelPersistenceTests.swift` (신규)
  - currentURL submit 시 flush 호출
  - state_json 주입 시 load() 우선, 미주입 시 DevServerDetector 폴백
- `WorkspaceEnvironmentTests.swift` (신규)
  - WorkspaceSnapshot.projectPath 주입 → FileTreeSurface.workspacePath 전파

### 7.3 UI 수동/자동 시나리오

- `E2ESurfacePersistenceTests.swift` UITest (opt-in, CI skip)
  - 탭 N = 5 개 (FileTree + Markdown × 2 + Image + Browser) 열고 각각 상태 조작
  - Cmd+Q → 앱 재실행 → 모든 탭이 동일 상태로 복원되는지 스크린샷 diff
- 수동 체크리스트 (`docs/manual-qa/spec-m2-003.md`)
  - 재시작 10 회 반복 무결성
  - 워크스페이스 전환 간 path 가 올바르게 @Environment 로 전파되는지

### 7.4 성능 / 마이그레이션

- `scripts/v3-to-v4-migration-benchmark.sh` (opt-in)
  - 10 pane × 20 tab = 200 surface 행 DB 에서 V4 적용 시간 측정 (<1s 목표)
- `FFIBenchmarkTests` 재실행 — state_json JSON 파싱 overhead 추가 후에도 P95 <1ms 유지 확인

---

## 8. 참조 문서

- `.moai/specs/SPEC-M2-001/spec.md` — M2 Viewers 기준 SPEC (RG-M2-2 surfaces 테이블, RG-M2-4 FileTree, RG-M2-5 Markdown, RG-M2-7 Browser)
- `.moai/specs/SPEC-M2-001/m2-completion-report.md` §"알려진 제한 사항" — 본 SPEC 의 4 개 개선 항목 (P-5, P-6, P-7, P-8) 원본 설명
- `.moai/specs/SPEC-M2-001/progress.md` — resolveWorkspacePath 폴백, statePathCache, FileTree 한 레벨 제한 관련 MX 태그
- `core/crates/moai-store/migrations/V3__panes_surfaces.sql` — 기존 surfaces 테이블 DDL (state_json TEXT 컬럼 이미 존재)
- `core/crates/moai-store/src/surface.rs` — SurfaceDao CRUD, SurfaceKind enum
- `core/crates/moai-store/src/lib.rs` — schema_version 기반 migration 패턴
- `app/Sources/Shell/Tabs/TabBarViewModel.swift` — statePathCache 메모리 보관 코드 (P-5 출처)
- `app/Sources/Shell/Splits/PaneSplitView.swift` — resolveWorkspacePath homeDirectoryForCurrentUser 폴백 (P-8 출처)
- `app/Sources/Surfaces/FileTree/FileTreeSurface.swift` — 루트 한 레벨 리스팅 (P-6 출처)
- `app/Sources/Surfaces/Browser/BrowserSurface.swift` — init() 무인자 + DevServerDetector 자동 감지 (P-7 출처)
- `app/Sources/Bridge/RustCore+Generated.swift` — WorkspaceSnapshot 구조체 (projectPath 누락)
- `.moai/project/tech.md` — Swift 6 / Rust 1.92 / SQLite WAL 스택 제약
- EARS 참조: `.claude/skills/moai-workflow-spec/SKILL.md`

---

## 9. Exclusions (What NOT to Build)

1. **Terminal surface state 영속** — GhosttyHost wiring 완료 후 별도 SPEC. 본 SPEC 의 state_json 스키마 표에서 terminal 은 `{}` 로 고정.
2. **Code / AgentRun / Kanban / Memory / InstructionsGraph state_json 스키마** — 해당 surface 구현 마일스톤에서 각자 정의.
3. **Security-Scoped Bookmark 기반 경로 영속** — macOS sandbox 복원 보안 관련 별도 SPEC.
4. **DB 다운그레이드 (V4 → V3)** — forward-only migration. 이전 버전 복귀는 DB 재생성으로 처리.
5. **워크스페이스 이동 시 경로 자동 rebase** — 절대 경로 기반 유지. 이동된 경로는 AC-P-3 "File not found" 경로로 처리.
6. **탭 간 drag-and-drop 영속** — cross-pane drag 는 M3+ 범위 (SPEC-M2-001 Exclusion 11 과 동일).
7. **Surface 간 통신 프로토콜 (surface-to-surface 이벤트 버스)** — M3 범위.
8. **Pane tree 영속 개선** — 이미 SPEC-M2-001 RG-M2-1 에서 영속됨. 본 SPEC 은 surface 내부 상태만 다룸.
9. **state_json 암호화** — Surface state 는 비밀 정보가 아니므로 plaintext 저장. 민감 토큰 영속은 moai-hook-http auth 범위.
10. **멀티 디바이스 동기화** — 로컬 SQLite 영속만. iCloud / sync 는 M6+.

---

## 10. 용어 정의

| 용어 | 정의 |
|------|------|
| state_json | `surfaces.state_json` TEXT 컬럼에 저장되는 surface kind 별 JSON payload. 본 SPEC 의 핵심 영속 매개체 |
| SurfaceState | Rust serde enum. 5 variants (Terminal/FileTree/Markdown/Image/Browser) 로 state_json 을 타입화 |
| SurfaceStateDecoder | Swift 측 JSON → typed struct 디코더 헬퍼 |
| expanded_paths | FileTree 의 현재 expand 된 디렉토리 경로 집합 (워크스페이스 루트 기준 상대 경로) |
| resolveWorkspacePath | 기존 PaneSplitView 의 홈 디렉토리 폴백 함수. 본 SPEC 에서 제거 대상 |
| @Environment(\.activeWorkspace) | 활성 워크스페이스를 SwiftUI 뷰 트리에 주입하는 신규 Environment Key |
| V4 migration | `V4__state_json_schema.sql`. state_json 정규화 + legacy payload 변환 수행 |
| forward-only migration | 상위 버전으로의 단방향 마이그레이션. 다운그레이드 미지원 |
| silent prune | 복원 시 삭제된 expanded_paths 를 사용자 알림 없이 제거하는 동작 (AC-P-5) |
