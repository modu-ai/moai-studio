# SPEC-M2-002: M2.5 Polish -- Placeholder 잔재 해소 (ActivePaneProvider + GhosttyHost 실연결 + Command Palette 콜백 활성화) (ARCHIVED — v2 Swift design)

> **⚠️ SUPERSEDED (2026-04-24)**: 본 SPEC 은 Swift/AppKit 기반 v2 아키텍처를 전제한다. 2026-04-21 v3 pivot (GPUI + Rust) 으로 기존 구현 경로가 `archive/swift-legacy/` 로 이관되었으며, Swift placeholder 해소 작업 자체가 v3 pivot 으로 무효화되었다 — v3 는 새 구현이므로 legacy polish 대상이 없다.
>
> **후속 조치**: (b) `status: archived-v2-design` 로 동결 채택 (2026-04-24 Priority Low 정비).

---
id: SPEC-M2-002
version: 1.2.0-archived
status: archived-v2-design
created: 2026-04-16
updated: 2026-04-24
superseded_by: SPEC-V3-001
author: MoAI (manager-spec)
priority: High
issue_number: 0
labels: [archived, v2-swift, m2, polish, placeholder, superseded]
revision: v1.2.0-archived (Priority Low 정비 2026-04-24 — v3 pivot 으로 archive, superseded_by 명시)
---

## HISTORY

| 버전 | 날짜 | 변경 내용 |
|------|------|-----------|
| 1.2.0-archived | 2026-04-24 | v3 pivot 으로 archive. Swift/AppKit 기반 설계는 Rust + GPUI 기반 v3 SPEC (SPEC-V3-001) 로 계승. status: completed → archived-v2-design. Priority Low 정비 PR. |
| 1.1.0 | 2026-04-17 | Run phase 완료. 18 tasks 모두 GREEN-REFACTOR. 4개 placeholder 해소. Swift 테스트 130/130 PASS, Rust 289/289 PASS. @MX 태그 2건 ANCHOR + 6건 NOTE 추가. |
| 1.0.0 | 2026-04-16 | 초안 작성. SPEC-M2-001 (completed, 2026-04-15) placeholder 4건 (P-1~P-4) 해소. ActivePaneProvider → GhosttyHost 실연결 → Command Palette onSurfaceOpen/onPaneSplit 활성화 순서. |

---

## 1. 개요

MoAI Studio 의 **M2.5 Polish** 스프린트. SPEC-M2-001 (M2 Viewers) 가 2026-04-15 `completed` 로 판정되었으나, 로컬 GUI 수동 검증 시 **4개의 placeholder/no-op 잔재**가 확인되었다. M2 의 핵심 사용자 가치 (Cmd+K 로 Surface 열기, Pane 분할, 실제 터미널 렌더링) 가 "UI 는 존재하지만 동작하지 않는" 상태에 있으므로, 본 SPEC 은 해당 4건을 완결하여 **M2 산출물을 실질적으로 사용 가능한 상태로 승격**한다.

**성공 기준**: 앱 실행 → Cmd+K → "Terminal 열기" 선택 → 활성 pane 에 실제 GhosttyHost Metal surface 렌더 → Cmd+K → "File Tree 열기" → 활성 pane 에 FileTree surface 생성 → Cmd+K → "Pane 수평 분할" → 활성 pane 분할 → 모든 조작이 `activePaneId` 기반으로 정확한 pane 에 적용됨. 기존 339 개 테스트 (Rust 233 + Swift 106) regression 0.

**선행 조건**: SPEC-M2-001 `completed` (v1.2.0, 2026-04-15). `app/Sources/Surfaces/Terminal/TerminalSurface.swift` 의 `GhosttyHost` 구조체 존재, `WorkspaceSnapshot` 타입 존재, `CommandRegistry` 4종 콜백 (onMoaiSlash/onSurfaceOpen/onWorkspaceCreate/onPaneSplit) 스켈레톤 존재.

**Placeholder 목록 (m2-completion-report.md §알려진 제한 사항)**:

| # | 항목 | 현재 상태 | 영향 |
|---|------|----------|------|
| P-1 | TerminalSurface GhosttyHost 실연결 | `TerminalSurfacePlaceholder` 표시 ("Ghostty Metal surface will render here") | 터미널 탭이 실제 터미널로 동작하지 않음 |
| P-2 | Command Palette `onSurfaceOpen` | no-op (주석 `TODO(MS-7)` 만 표시) | "Open FileTree / Markdown / Image / Browser / Terminal" 명령 무동작 |
| P-3 | Command Palette `onPaneSplit` | no-op (주석 `TODO(MS-7)` 만 표시) | "Split Pane Horizontally / Vertically" 명령 무동작 |
| P-4 | ActivePaneProvider `@Environment` | 미존재 | P-2/P-3 의 선행 조건. "현재 활성 pane" 을 환경값으로 공유할 수단 부재 |

**참조 문서**:
- `DESIGN.v4.md` SS3.1 (Pane/Surface 아키텍처), SS6 (DB 스키마)
- `.moai/specs/SPEC-M2-001/spec.md` (v1.2.0, M2 산출물 기준)
- `.moai/specs/SPEC-M2-001/m2-completion-report.md` §알려진 제한 사항 (placeholder 원본 목록)
- `.moai/specs/SPEC-M2-001/m2-completion-report.md` §M3 권장 다음 액션 (GhosttyHost 연동 + ActivePaneProvider 우선순위)
- `.moai/project/product.md`, `.moai/project/tech.md` (Swift 6 / SwiftUI / swift-bridge FFI 스택)

---

## 2. 요구사항 그룹 (EARS 형식)

### RG-M2.5-1: ActivePaneProvider `@Environment` (P-4, 선행)

P-2/P-3 의 구현 선행 조건. SwiftUI `EnvironmentKey` 를 통해 "현재 활성 pane 의 `paneId` 와 소속 `PaneTreeModel`" 을 뷰 트리 하위에 공유한다.

**[Ubiquitous]** Swift 측 `app/Sources/Shell/Splits/ActivePaneProvider.swift` 파일을 **신설해야 한다** (shall create). 파일은 `ActivePaneContext` struct 와 `ActivePaneProviderKey: EnvironmentKey` 를 정의한다.

**[Ubiquitous]** `ActivePaneContext` 는 다음 최소 멤버를 **포함해야 한다** (shall include):
- `paneId: Int64?` — 현재 활성 leaf pane 의 DB id (없으면 nil)
- `model: PaneTreeModel?` — 활성 pane 이 속한 워크스페이스의 `PaneTreeModel` 참조 (없으면 nil)
- `workspace: WorkspaceSnapshot?` — 활성 pane 의 소속 워크스페이스 스냅샷 (없으면 nil)

**[Ubiquitous]** `EnvironmentValues` 는 `activePane: ActivePaneContext` computed property 를 **노출해야 한다** (shall expose). 기본값은 `ActivePaneContext(paneId: nil, model: nil, workspace: nil)` (모든 필드 nil) 이다.

**[Event-Driven]** `PaneSplitContainerView` 의 `activePaneId` `@State` 가 변경되면 (When activePaneId changes), 시스템은 하위 뷰 트리에 `.environment(\.activePane, ...)` modifier 로 최신 `ActivePaneContext` 를 **전파해야 한다** (shall propagate).

**[Event-Driven]** 사용자가 pane 영역을 탭하여 활성 pane 이 전환되면 (When user taps another pane), 시스템은 `activePaneId` 를 **업데이트해야 한다** (shall update). 기존 `.onTapGesture { activePaneId = paneId }` 동작은 regression 없이 **유지해야 한다** (shall preserve).

**[State-Driven]** 활성 pane 이 없는 상태인 동안 (While activePaneId is nil), `ActivePaneContext.paneId` 는 nil 을 **반환해야 한다** (shall return). 소비자 (RootSplitView 콜백) 는 nil 케이스를 no-op 으로 **처리해야 한다** (shall handle as no-op).

**[Unwanted]** `ActivePaneContext` 는 비-leaf (split 노드) 의 id 를 활성값으로 **보유해서는 안 된다** (shall not hold). 활성 pane 은 반드시 leaf 이어야 한다.

**산출물**: `app/Sources/Shell/Splits/ActivePaneProvider.swift`, `PaneSplitContainerView` 의 `.environment(\.activePane, ...)` 주입.

---

### RG-M2.5-2: TerminalSurface GhosttyHost 실연결 (P-1)

`TerminalSurfacePlaceholder` 를 제거하고, `SurfaceRouter` 의 `.terminal` 케이스가 실제 `TerminalSurface(workspace:)` 를 렌더링하도록 교체한다. `WorkspaceSnapshot` 은 `@Environment` 로 주입한다.

**[Ubiquitous]** `WorkspaceSnapshot` 을 SwiftUI `@Environment` 로 하위 뷰 트리에 주입하는 `WorkspaceEnvironmentKey: EnvironmentKey` 를 **정의해야 한다** (shall define). `EnvironmentValues.activeWorkspace: WorkspaceSnapshot?` computed property 를 노출한다.

**[Event-Driven]** `PaneContainer` 가 선택된 워크스페이스의 `PaneTreeModel` 을 로드한 시점에 (When PaneContainer loads model for workspace), 시스템은 `viewModel.workspace(id: wsId)` 결과를 `.environment(\.activeWorkspace, ...)` 로 **주입해야 한다** (shall inject).

**[Event-Driven]** `SurfaceRouter` 가 `activeKind == .terminal` 케이스를 처리할 때 (When routing .terminal case), 시스템은 `TerminalSurfacePlaceholder` 대신 `TerminalSurface(workspace: env.activeWorkspace)` 를 **렌더링해야 한다** (shall render). `activeWorkspace` 가 nil 이면 기존 `TerminalFallback` 스타일의 안내 뷰를 **표시해야 한다** (shall display).

**[Ubiquitous]** `TerminalSurface.swift` 의 `GhosttyHost` 구조체는 `MOAI_TERMINAL_BACKEND=ghostty` (기본값) 환경에서 실제 GhosttyKit Metal surface 를 **렌더링해야 한다** (shall render). GhosttyKit 초기화 실패 시 `onFailure()` 콜백을 호출하여 `TerminalFallback` 으로 전환한다 (기존 `loadFailed` 상태 전환 경로 유지).

**[State-Driven]** `MOAI_TERMINAL_BACKEND=nstext` 환경 변수가 설정된 상태에서 (While nstext backend selected), 시스템은 `TerminalFallback` 을 **표시해야 한다** (shall display). 개발자 워크플로우 호환성 보장 목적.

**[State-Driven]** GhosttyKit xcframework 가 Metal Toolchain 부재로 로드되지 않는 동안 (While GhosttyKit unavailable), 시스템은 앱 전체를 크래시시켜서는 **안 되며** (shall not crash), 대신 `TerminalFallback` 의 안내 메시지를 **표시해야 한다** (shall display). 기존 `@MX:WARN` 격리 정책 (SPEC-M1-001 RG-M1-2 §[If-Then]) 유지.

**[Unwanted]** `TerminalSurfacePlaceholder` struct 와 그 호출 지점은 코드베이스에 **잔존해서는 안 된다** (shall not remain). `PaneSplitView.swift:270` 의 `TerminalSurfacePlaceholder(paneId:)` 호출과 `PaneSplitView.swift:313` 의 struct 정의 모두 제거한다.

**[Event-Driven]** 사용자가 Cmd+K → "Terminal 열기" 를 선택하면 (When Terminal command executed via palette), 활성 pane 에 실제 `TerminalSurface` 가 렌더링된 새 탭을 **생성해야 한다** (shall create). (RG-M2.5-3 과 연계).

**산출물**: `WorkspaceEnvironmentKey` (신규 또는 `ActivePaneProvider.swift` 병합), `PaneContainer.swift` 의 `.environment(\.activeWorkspace, ...)` 주입, `SurfaceRouter` 의 `.terminal` 케이스 재작성, `TerminalSurfacePlaceholder` 제거.

---

### RG-M2.5-3: Command Palette `onSurfaceOpen` 콜백 활성화 (P-2)

`CommandRegistry` 의 `onSurfaceOpen` 콜백이 `ActivePaneProvider` 환경값을 이용해 실제 Surface 를 활성 pane 에 생성하도록 구현한다.

**[Event-Driven]** 사용자가 Command Palette 에서 Surface 카테고리 명령 (`surface.filetree`, `surface.markdown`, `surface.image`, `surface.browser`, `surface.terminal`) 중 하나를 실행하면 (When a Surface category command executes), 시스템은 `CommandRegistry.onSurfaceOpen(kind:)` 콜백을 **호출해야 한다** (shall invoke).

**[Event-Driven]** `RootSplitView.setupPaletteController` 에서 `onSurfaceOpen` 클로저가 호출되면 (When onSurfaceOpen fires), 시스템은 현재 `ActivePaneContext` 의 `model` 과 `paneId` 를 참조하여 활성 pane 의 `TabBarViewModel` 에 새 탭을 해당 `SurfaceKind` 로 **생성해야 한다** (shall create). 새 탭은 즉시 활성화된다.

**[State-Driven]** `ActivePaneContext.paneId` 가 nil 인 상태인 동안 (While no active pane), 시스템은 `onSurfaceOpen` 호출을 **무시해야 한다** (shall ignore). 크래시나 silent error 를 **발생시켜서는 안 된다** (shall not raise). 로그 레벨 `info` 로 "no active pane" 를 기록한다.

**[Event-Driven]** `SurfaceKind` 가 `.filetree`, `.markdown`, `.image`, `.browser`, `.terminal` 중 하나일 때 (When kind is one of 5 M2 surfaces), 시스템은 해당 surface 를 즉시 **렌더링해야 한다** (shall render). M3+ 이월 surface 종류 (`.code`, `.agentRun`, `.kanban`, `.memory`, `.instructionsGraph`) 는 기존 `NotYetImplementedSurface` 로 **폴백해야 한다** (shall fall back).

**[Event-Driven]** `onSurfaceOpen` 이 `.markdown` 또는 `.image` 를 요청한 경우 (When kind is .markdown or .image), 시스템은 statePath 가 빈 새 탭을 생성하고, 사용자가 FileTree 에서 파일을 선택할 때까지 해당 Surface 의 EmptyState 를 **표시해야 한다** (shall display). (명시적 파일 경로 선택 UX 는 본 SPEC 범위 외; M3 이월).

**[Unwanted]** `RootSplitView.swift:79-82` 의 `onSurfaceOpen: { _ in }` no-op 블록과 주석 `TODO(MS-7)` 은 코드베이스에 **잔존해서는 안 된다** (shall not remain). 실제 구현 코드로 교체한다.

**산출물**: `RootSplitView.swift` 의 `onSurfaceOpen` 클로저 재작성, `TabBarViewModel.newTab(kind:)` 호출, `ActivePaneContext` 소비 로직.

---

### RG-M2.5-4: Command Palette `onPaneSplit` 콜백 활성화 (P-3)

`CommandRegistry` 의 `onPaneSplit` 콜백이 `ActivePaneProvider` 환경값을 이용해 활성 pane 에 실제 분할을 트리거하도록 구현한다.

**[Event-Driven]** 사용자가 Command Palette 에서 `pane.split.horizontal` 또는 `pane.split.vertical` 명령을 실행하면 (When a Pane split command executes), 시스템은 `CommandRegistry.onPaneSplit(direction:)` 콜백을 **호출해야 한다** (shall invoke).

**[Event-Driven]** `RootSplitView.setupPaletteController` 에서 `onPaneSplit` 클로저가 호출되면 (When onPaneSplit fires), 시스템은 현재 `ActivePaneContext` 의 `model.splitActive(paneId, direction:)` 를 **호출해야 한다** (shall call). 반환된 새 leaf pane id 로 `activePaneId` 를 업데이트하여 새 pane 이 즉시 활성화되도록 한다.

**[State-Driven]** `ActivePaneContext.paneId` 또는 `ActivePaneContext.model` 이 nil 인 동안 (While no active pane context), 시스템은 `onPaneSplit` 호출을 **무시해야 한다** (shall ignore). 크래시를 **발생시켜서는 안 된다** (shall not crash).

**[Ubiquitous]** Command Palette 경로의 pane 분할 결과는 기존 키보드 단축키 (Cmd+\, Cmd+Shift+\) 경로의 결과와 **동일해야 한다** (shall match). 즉 `PaneTreeModel.splitActive` 를 동일한 방식으로 호출하여 DB 영속화 + UI 갱신 동작이 regression 없이 유지된다.

**[Event-Driven]** 분할 후 새 leaf pane 이 생성된 시점에 (When new leaf created), 시스템은 `activePaneId` 를 새 leaf 의 id 로 **설정해야 한다** (shall set). 기존 키보드 단축키 경로 (`PaneSplitView.swift:382, 388`) 와 동일한 동작.

**[Unwanted]** `RootSplitView.swift:86-89` 의 `onPaneSplit: { _ in }` no-op 블록과 주석 `TODO(MS-7)` 은 **잔존해서는 안 된다** (shall not remain). 실제 구현 코드로 교체한다.

**산출물**: `RootSplitView.swift` 의 `onPaneSplit` 클로저 재작성, `PaneTreeModel.splitActive` 호출, `ActivePaneContext.paneId` 업데이트 로직.

---

## 3. 수용 기준

### AC-1 (RG-M2.5-1): ActivePaneProvider

- **AC-1.1**: `app/Sources/Shell/Splits/ActivePaneProvider.swift` 파일이 존재하며 `ActivePaneContext` struct, `ActivePaneProviderKey: EnvironmentKey`, `EnvironmentValues.activePane` computed property 3종이 모두 정의되어 있다.
- **AC-1.2**: 단위 테스트 `ActivePaneProviderTests.swift` 가 기본값 (모든 필드 nil), 값 주입 후 조회, 중첩 `.environment` override 를 모두 검증하며 통과한다.
- **AC-1.3**: `PaneSplitContainerView` 내부에서 `activePaneId` 가 변경될 때 하위 뷰의 `@Environment(\.activePane) var ctx` 가 새 값을 반영한다 (SwiftUI snapshot 또는 ViewInspector 기반 테스트).
- **AC-1.4**: `ActivePaneContext.paneId` 는 항상 leaf pane id 이며, split (non-leaf) 노드 id 가 할당되면 assertion failure (DEBUG 빌드) 또는 로그 경고 (RELEASE) 가 발생한다.
- **AC-1.5**: 기존 `PaneTreeModelTests`, `TabBarViewModelTests` 등 M2 Swift 테스트 41건 + MS-3~MS-7 추가분 모두 regression 0.

### AC-2 (RG-M2.5-2): TerminalSurface GhosttyHost 실연결

- **AC-2.1**: `TerminalSurfacePlaceholder` struct 및 그 호출 지점이 저장소 grep 결과에서 0 건이다 (`grep -r "TerminalSurfacePlaceholder" app/Sources/` 결과 없음).
- **AC-2.2**: `SurfaceRouter` 의 `.terminal` 케이스가 `TerminalSurface(workspace:)` 를 렌더링하며, `WorkspaceSnapshot` 은 `@Environment(\.activeWorkspace)` 에서 조회된다.
- **AC-2.3**: `PaneContainer` 는 선택된 워크스페이스 UUID 에 해당하는 `WorkspaceSnapshot` 을 `.environment(\.activeWorkspace, ...)` 로 하위 뷰 트리에 주입한다.
- **AC-2.4**: 환경변수 `MOAI_TERMINAL_BACKEND` 미설정 (기본 `.ghostty`) 시 `GhosttyHost` 가 렌더링되고, `nstext` 설정 시 `TerminalFallback` 이 렌더링된다.
- **AC-2.5**: GhosttyKit 초기화 실패 시 `onFailure()` 가 호출되어 `loadFailed = true` 로 전환되고 `TerminalFallback` 으로 교체된다 (mock/stub 기반 유닛 테스트).
- **AC-2.6**: 수동 UI 검증 (macOS 14+, Metal Toolchain 설치 환경): 앱 실행 → 워크스페이스 선택 → 기본 탭에 실제 GhosttyKit Metal surface 가 렌더된다 ("(Ghostty Metal surface will render here)" 텍스트가 더 이상 보이지 않는다).

### AC-3 (RG-M2.5-3): Command Palette `onSurfaceOpen`

- **AC-3.1**: `RootSplitView.swift` 의 `onSurfaceOpen: { _ in ... }` 블록이 실제 구현으로 교체되어 있으며 `TODO(MS-7)` 주석이 제거되어 있다.
- **AC-3.2**: 유닛 테스트 `CommandPaletteSurfaceOpenTests.swift`: mock `ActivePaneContext` 주입 → `onSurfaceOpen(.filetree)` 호출 → mock `TabBarViewModel.newTab(kind:)` 가 `.filetree` 인자로 정확히 1회 호출된다.
- **AC-3.3**: 동일 테스트가 `.markdown`, `.image`, `.browser`, `.terminal` 5 종 SurfaceKind 에 대해 모두 통과한다.
- **AC-3.4**: `ActivePaneContext.paneId = nil` 상태에서 `onSurfaceOpen(.filetree)` 호출 시 `TabBarViewModel.newTab` 이 호출되지 않으며 예외도 발생하지 않는다.
- **AC-3.5**: 수동 UI 검증: Cmd+K → "File Tree 열기" 선택 → 활성 pane 에 `FileTreeSurface` 가 렌더된 새 탭이 생성되고 즉시 활성화된다.

### AC-4 (RG-M2.5-4): Command Palette `onPaneSplit`

- **AC-4.1**: `RootSplitView.swift` 의 `onPaneSplit: { _ in ... }` 블록이 실제 구현으로 교체되어 있으며 `TODO(MS-7)` 주석이 제거되어 있다.
- **AC-4.2**: 유닛 테스트 `CommandPalettePaneSplitTests.swift`: mock `ActivePaneContext` 주입 → `onPaneSplit(.horizontal)` 호출 → mock `PaneTreeModel.splitActive(_, direction:)` 가 `(mockPaneId, .horizontal)` 인자로 정확히 1회 호출된다. `.vertical` 도 동일하게 검증.
- **AC-4.3**: `splitActive` 반환값 (새 leaf pane id) 이 `activePaneId` 상태에 반영된다.
- **AC-4.4**: `ActivePaneContext` 의 `model` 또는 `paneId` 가 nil 일 때 `onPaneSplit` 호출이 no-op 이며 크래시하지 않는다.
- **AC-4.5**: 수동 UI 검증: Cmd+K → "Pane 수평 분할" 선택 → 활성 pane 이 실제로 좌우 분할되며, 결과가 Cmd+\ 키보드 단축키 사용 시와 동일하다 (DB 영속화 포함).

### AC-Global: 전체 Regression

- **AC-G.1**: `cargo test --workspace` 전체 233건 통과 (Rust 영역 변경 없음, regression 0).
- **AC-G.2**: `xcodebuild test` 기존 106건 + 본 SPEC 신규 테스트 모두 통과. 신규 테스트 수는 최소 15건 이상 (ActivePaneProvider 5 + SurfaceOpen 5 + PaneSplit 5).
- **AC-G.3**: `cargo clippy --workspace -- -D warnings` 0 warning, `cargo fmt --all -- --check` PASS, Swift 빌드 warning 증가 0.
- **AC-G.4**: 앱 재시작 후 pane tree 레이아웃 + 탭 순서 + 활성 pane 의 surface kind 가 100% 복원된다 (SPEC-M2-001 RG-M2-1/RG-M2-2 State-Driven 조항 regression 없음).

---

## 4. 마일스톤

| MS | 스프린트 | 선행 | 산출물 | 참조 RG |
|----|----------|------|--------|---------|
| MS-1 | ActivePaneProvider 설계 + 구현 | (없음) | `ActivePaneProvider.swift`, `WorkspaceEnvironmentKey`, `ActivePaneProviderTests` | RG-M2.5-1 |
| MS-2 | TerminalSurface GhosttyHost 실연결 | MS-1 | `SurfaceRouter.terminal` 케이스 재작성, `TerminalSurfacePlaceholder` 제거, `PaneContainer` `.environment(\.activeWorkspace, ...)` 주입 | RG-M2.5-2 |
| MS-3 | Command Palette 콜백 활성화 | MS-1 | `RootSplitView.onSurfaceOpen/onPaneSplit` 실제 구현, `CommandPaletteSurfaceOpenTests`, `CommandPalettePaneSplitTests`, `TODO(MS-7)` 주석 전량 제거 | RG-M2.5-3, RG-M2.5-4 |

**마일스톤 순서 근거**:

- MS-1 이 **반드시 선행**되어야 한다. ActivePaneProvider 는 MS-2 의 `WorkspaceSnapshot` 주입 채널과 MS-3 의 `paneId/model` 소비의 공통 기반이다.
- MS-2 와 MS-3 는 MS-1 완료 후 **병렬 가능**하지만, MS-2 (GhosttyHost 실연결) 가 육안 검증 가치가 커 먼저 배치한다.
- 각 MS 는 독립적인 커밋 + 테스트 통과가 보장되어야 한다 (부분 rollback 가능성 확보).

---

## 5. 비기능 요구사항 (NFR)

| 항목 | 목표 | 측정 방법 |
|------|------|-----------|
| Metal surface 렌더 fps | ≥ 60 fps @ 1080p (C-4 후속 측정에 부합) | GhosttyMetalBenchmarkTests 하네스 재사용 |
| FFI P95 지연 | < 1 ms (SPEC-M2-001 C-7 기준 유지) | `FFIBenchmarkTests` regression 0 |
| Command Palette 열기 → 실행 → 결과 반영 | ≤ 300 ms (기존 200 ms 팔레트 열기 + 100 ms 실행 버짓) | Manual timing + XCTest performance |
| Pane 분할 반응 (팔레트 경로) | < 100 ms (SPEC-M2-001 NFR 동일 기준) | 키보드 단축키 경로와 동일한 `splitActive` 호출이므로 regression 0 |
| Swift 빌드 시간 증가 | < 5% | CI `xcodebuild build-for-testing` 비교 |
| Rust 테스트 regression | 0 건 | `cargo test --workspace` 233/233 |
| Swift 테스트 regression | 0 건 | `xcodebuild test` 106/106 + 신규 통과 |
| 메모리 (8 pane, 8 tab, 실제 GhosttyHost 8개) | RSS < 600 MB (SPEC-M2-001 NFR 상한 유지) | Instruments 또는 `ps` 샘플링 |
| 레이아웃 복원 정확도 | 100% 일치 | 앱 재시작 후 `pane tree` + `tab order` + `surface kind` 비교 |

---

## 6. 의존성 및 제약

**내부 의존성**:

- **swift-bridge FFI**: `RustCoreBridging.getWorkspaceDbId(workspaceUuid:)`, `PaneTreeModel.splitActive(_, direction:)` 등 기존 함수 재사용. FFI 표면 변경 **없음**.
- **SwiftUI EnvironmentKey 패턴**: `@MainActor` 격리 하에서 `EnvironmentKey.defaultValue` 를 정의하고, `EnvironmentValues` extension 으로 computed property 노출. `ActivePaneContext` 는 `Sendable` 을 요구하지 않음 (뷰 트리 내부 전용).
- **NSSplitView binary tree (SPEC-M2-001 RG-M2-1)**: 기존 `PaneTreeModel` / `PaneSplitContainerView` 의 `activePaneId` `@State` 를 `@Environment` 로 브릿지. 기존 단축키 경로는 regression 없이 유지.
- **TabBarViewModel.newTab(kind:)**: MS-3 에서 M2 범위의 5종 `SurfaceKind` 를 모두 받는 메서드를 사용. 필요 시 기본 `statePath` 파라미터 추가 가능 (비파괴 변경).
- **GhosttyKit xcframework**: M1 에서 확보. 본 SPEC 은 래핑 로직 교체만 수행 (GhosttyKit API 학습 곡선은 작음 — 기존 `GhosttyHost` 스켈레톤 존재).

**외부 의존성**:

- **Metal Toolchain**: 로컬 빌드 시 macOS 14+ 필요. CI 환경은 SPEC-M2-001 MS-7 에서 구성 완료 (C-1 해소).
- **swift-bridge 버전**: SPEC-M2-001 MS-2 에서 JSON FFI 경로로 우회 (C-5 완료). 본 SPEC 은 버전 업그레이드 **불필요**.

**제약**:

- 본 SPEC 은 **M2 placeholder 해소 범위에 한정**. FileTree expand 재귀 리스팅, Markdown CDN→로컬 번들 전환, statePath DB 영속화 등은 M3 이월 (m2-completion-report.md §알려진 제한 사항).
- `@MainActor` 격리: `ActivePaneContext` 의 `model`, `workspace` 속성 접근은 모두 `@MainActor` 컨텍스트. background thread 에서 직접 조회 금지.
- 본 SPEC 완료 시 M2 Conditional GO → Full GO 승격 가능 (C-4 Metal 60fps 실측은 GhosttyHost 실연결 후 별도 측정).

---

## 7. 테스트 전략

### 7.1 단위 테스트 (Unit)

- **ActivePaneProviderTests.swift** (MS-1): 기본값 검증, 값 주입/조회, 중첩 override, leaf assertion. 최소 5건.
- **CommandPaletteSurfaceOpenTests.swift** (MS-3): mock `ActivePaneContext` × 5 `SurfaceKind` + nil 케이스. 최소 6건.
- **CommandPalettePaneSplitTests.swift** (MS-3): mock `ActivePaneContext` × 2 direction (`.horizontal`, `.vertical`) + nil 케이스 + 새 pane id 반영 검증. 최소 4건.
- **TerminalSurfaceEnvironmentTests.swift** (MS-2): `@Environment(\.activeWorkspace)` 주입 시 `TerminalSurface(workspace:)` 가 비 nil 로 생성, nil 주입 시 fallback 안내 뷰 표시. 최소 3건.

### 7.2 통합 테스트 (Integration)

- **PaneTabSurfaceIntegrationTests.swift**: `PaneTreeModel` 로드 → `activePaneId` 설정 → `TabBarViewModel.newTab(.filetree)` → `SurfaceRouter` 가 `.filetree` 케이스로 `FileTreeSurface` 렌더링, 전체 흐름이 환경 주입 후에도 regression 없이 동작.
- 기존 `PaneTreeModelTests`, `TabBarViewModelTests` 의 모든 케이스가 `ActivePaneProvider` 도입 후에도 통과 (환경값 미주입 시 기본값 nil 로 동일 동작).

### 7.3 UI 테스트 (E2E, CI skip 옵션)

- **E2EPolishTests.swift** (UITest, 서명 필요):
    - Cmd+K → "Terminal 열기" → 활성 pane 에 GhosttyHost 렌더 확인 (screenshot assertion).
    - Cmd+K → "File Tree 열기" → `FileTreeSurface` 탭 생성 확인.
    - Cmd+K → "Pane 수평 분할" → `NSSplitView` 자식 2개 확인, 새 pane 활성화 확인.
    - 앱 재시작 → 위 레이아웃 100% 복원 확인.
- SPEC-M2-001 C-1 (UITest CI 서명) 인프라 재사용. CI 에서는 `HAS_SIGNING` 미설정 시 skip.

### 7.4 수동 검증 체크리스트 (로컬 GUI)

1. `xcodebuild` 앱 실행 → 기본 워크스페이스 생성 → 기본 탭에 실제 터미널 렌더 확인 (AC-2.6)
2. Cmd+K → 각 Surface 명령 5종 실행 → 활성 pane 에 새 탭 생성 확인 (AC-3.5)
3. Cmd+K → "Pane 수평 분할" / "Pane 수직 분할" 실행 → 좌우 / 상하 분할 확인 (AC-4.5)
4. 분할된 두 pane 중 다른 쪽을 탭 → `activePaneId` 전환 → Cmd+K → Surface 명령 → 올바른 pane 에 탭 생성 확인
5. 앱 종료 → 재실행 → 위 레이아웃 100% 복원 확인 (AC-G.4)

### 7.5 Regression Gate

- `cargo test --workspace` 233/233 PASS (Rust 변경 없음)
- `xcodebuild test` 106 + 신규 ≥15 = 121+ PASS
- `cargo clippy --workspace -- -D warnings` 0 warning
- `cargo fmt --all -- --check` PASS
- @MX 태그: MS-1 에서 `ActivePaneProvider.swift` 에 `@MX:ANCHOR` (fan_in ≥ 3 예상) + `@MX:REASON` 추가. MS-2/MS-3 에서는 `@MX:NOTE [AUTO]` 주석의 placeholder 참조 제거 + 갱신된 NOTE 추가.

---

## 8. Exclusions (What NOT to Build)

1. **FileTree expand 재귀 리스팅** — 현재 루트 한 레벨만 리스팅. M3 범위 (m2-completion-report.md §알려진 제한 사항 #5).
2. **MarkdownSurface KaTeX/Mermaid 오프라인 번들** — 현재 CDN 의존. M3 범위 (§알려진 제한 사항 #7).
3. **BrowserSurface statePath 영속화** — 마지막 URL DB 저장. M3 범위 (§알려진 제한 사항 #6).
4. **statePathCache DB 영속화** — 앱 재시작 시 탭 statePath 손실 (메모리 캐시). M3 범위 (§알려진 제한 사항 #3).
5. **C-2 Claude CLI AC-4.1 E2E CI 자동화** — 현재 opt-in 스크립트. M3+ 범위.
6. **C-3 10분 4-ws stress CI 자동화** — 현재 opt-in 스크립트. M3+ 범위.
7. **C-4 Metal 60fps 실측 전체 측정** — 본 SPEC 완료 후 별도 측정 스프린트.
8. **Code/AgentRun/Kanban/Memory/InstructionsGraph Surface** — M3~M5 범위. 본 SPEC 은 M2 범위 5종 Surface placeholder 해소에 한정.
9. **Cross-pane 탭 drag-and-drop** — M3+ 범위 (SPEC-M2-001 §5 Exclusions #11 계승).
10. **ActivePaneProvider 기반 surface-to-surface 이벤트 버스** — M3 범위 (SPEC-M2-001 §5 Exclusions #12 계승). 본 SPEC 의 `ActivePaneContext` 는 "현재 활성 pane 조회" 단일 책임에 한정.
11. **Command Palette 히스토리 영속화** — 현재 `historyIds` 메모리 유지. M3+ 범위.
12. **TerminalSurface 를 통한 Claude subprocess 실행 통합** — 본 SPEC 은 Ghostty Metal 렌더링 연결에 한정. `/moai` 슬래시 주입 경로는 기존 SlashInjector 로 유지.

---

## 9. 참조 문서

- `DESIGN.v4.md` — Pane/Surface 아키텍처 (SS3.1), DB 스키마 (SS6)
- `.moai/specs/SPEC-M2-001/spec.md` v1.2.0 — M2 Viewers 기준선 (`completed`, 2026-04-15)
- `.moai/specs/SPEC-M2-001/m2-completion-report.md` — §알려진 제한 사항 (placeholder 원본), §M3 권장 다음 액션 (우선순위)
- `.moai/specs/SPEC-M2-001/progress.md` — MS-3/MS-4/MS-6/MS-7 TerminalSurface/ActivePaneProvider 관련 NOTE 들
- `.moai/project/product.md` — 핵심 기능 14개, M2 포지셔닝
- `.moai/project/tech.md` — Swift 6 / SwiftUI / swift-bridge / GhosttyKit 스택
- `.claude/rules/moai/languages/swift.md` — SwiftUI `@Observable`, `@Environment` 패턴 가이드
- `.claude/rules/moai/workflow/mx-tag-protocol.md` — `@MX:ANCHOR` (fan_in≥3), `@MX:NOTE` 갱신 규칙

---

## 10. 용어 정의

| 용어 | 정의 |
|------|------|
| ActivePaneContext | 현재 활성 leaf pane 의 `paneId`, 소속 `PaneTreeModel`, 소속 `WorkspaceSnapshot` 를 담는 경량 struct. `@Environment` 를 통해 뷰 트리 하위에 공유 |
| ActivePaneProvider | `ActivePaneContext` 를 `@Environment` 값으로 노출하는 메커니즘. `EnvironmentKey` + `EnvironmentValues` extension 조합 |
| WorkspaceEnvironmentKey | 현재 활성 워크스페이스의 `WorkspaceSnapshot?` 을 뷰 트리 하위에 전파하는 `EnvironmentKey`. MS-2 에서 `SurfaceRouter.terminal` 케이스가 소비 |
| Placeholder 해소 | M2 완료 시점에 잔존한 4건의 "UI 존재 + 동작 없음" 상태 (P-1~P-4) 를 실제 동작하도록 교체하는 작업 |
| GhosttyHost 실연결 | `TerminalSurface.swift` 의 `GhosttyHost` 가 더 이상 텍스트 placeholder 를 표시하지 않고 실제 GhosttyKit Metal surface 를 렌더링하도록 교체 |
| no-op 콜백 | `{ _ in }` 본문만 있는 클로저. 본 SPEC 은 `onSurfaceOpen`, `onPaneSplit` 의 no-op 을 실제 구현으로 교체 |
