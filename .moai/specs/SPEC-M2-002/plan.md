# SPEC-M2-002 Implementation Plan

---
spec_id: SPEC-M2-002
version: 1.0.0
status: draft
created: 2026-04-16
---

## 1. Approach 요약

SwiftUI `EnvironmentKey` + `EnvironmentValues` extension 을 신설해 "활성 pane" 과 "활성 워크스페이스" 를 뷰 트리 하위에 전파하고, 동시에 `WorkspaceViewModel` 에 `@Observable var activePane: ActivePaneContext` 프로퍼티를 추가해 `RootSplitView.setupPaletteController` 클로저 (CommandPaletteView 오버레이) 에서도 접근 가능하게 만든다. 이 공통 기반 위에서 `TerminalSurface(workspace:)` 실연결 (P-1) 과 Command Palette `onSurfaceOpen` / `onPaneSplit` 콜백 활성화 (P-2, P-3) 를 수행한다. FFI 변경 없음, 기존 339 테스트 regression 0, 신규 테스트 최소 15건.

---

## 2. Technology Stack

| 영역 | 선택 | 근거 |
|------|------|------|
| 환경값 주입 | SwiftUI `EnvironmentKey` + `EnvironmentValues` extension | 뷰 트리 하향 전파의 표준 패턴. `ActivePaneContext`/`WorkspaceSnapshot?` 둘 다 value type 이므로 `@Observable` 매크로보다 적합 |
| Command Palette 접근 경로 | `WorkspaceViewModel.activePane: ActivePaneContext` `@Observable` 프로퍼티 | `CommandPaletteView` 는 `.overlay` 로 `NavigationSplitView.detail` 트리 밖에 있으므로 `EnvironmentKey` 전파 불가. 기존 `@Observable` 주입 경로 재사용 |
| Terminal 렌더 | 기존 `GhosttyHost(workspace:)` + `onFailure()` 콜백 | SPEC-M2-001 에서 스켈레톤 완료. GhosttyKit xcframework 연결만 교체 (본문 교체 최소 범위) |
| Pane split 경로 | 기존 `PaneTreeModel.splitActive(_, direction:)` | 키보드 단축키 경로와 동일 호출. 팔레트 경로 추가 |
| Tab 생성 경로 | 기존 `TabBarViewModel.newTab(kind:statePath:)` | 기본 시그니처 그대로 사용 (statePath 기본 nil) |
| FFI | swift-bridge 변경 없음 | SPEC-M2-001 C-5 해소 (JSON FFI 경로) 유지 |
| `@MainActor` 격리 | `ActivePaneHolder` 대신 기존 `WorkspaceViewModel` (@MainActor) 확장 | 신규 Holder 클래스 추가 없이 기존 DI 경로 재사용 |
| 테스트 | XCTest + MockRustCoreBridge 재사용 | SPEC-M2-001 MS-3 에서 검증된 mock 패턴 |

[HARD] swift-bridge FFI 표면 변경 금지. Rust 파일 수정 0. Metal 60fps 실측은 Exclusions §7 로 이월.

---

## 3. 구현 플랜 (Milestone 별)

### 3.1 MS-1 — ActivePaneProvider 설계 + 구현 (RG-M2.5-1, P-4 해소)

**목표**: `ActivePaneContext` value struct + 2종 `EnvironmentKey` + `WorkspaceViewModel.activePane` 프로퍼티를 추가하여 이후 MS-2/MS-3 의 공통 기반을 확보한다.

**디자인**:

- **`ActivePaneContext` struct (Shell/Splits/ActivePaneProvider.swift)**:
    - 필드: `paneId: Int64?`, `model: PaneTreeModel?`, `workspace: WorkspaceSnapshot?`
    - 정적 상수: `static let empty = ActivePaneContext(paneId: nil, model: nil, workspace: nil)`
    - Equatable: paneId 기준 (참조형 `model` 비교 제외)
    - `@MainActor` 격리 (`model` 접근 시 필수)

- **`ActivePaneProviderKey: EnvironmentKey`**:
    - `static let defaultValue: ActivePaneContext = .empty`

- **`WorkspaceEnvironmentKey: EnvironmentKey`**:
    - `static let defaultValue: WorkspaceSnapshot? = nil`

- **`EnvironmentValues` extension**:
    - `var activePane: ActivePaneContext` get/set
    - `var activeWorkspace: WorkspaceSnapshot?` get/set

- **`WorkspaceViewModel` 확장 (ViewModels/WorkspaceViewModel.swift)**:
    - `public var activePane: ActivePaneContext = .empty` 프로퍼티 추가 (`@Observable` 매크로가 자동 관찰 대상화)

- **`PaneSplitContainerView` 수정 (Shell/Splits/PaneSplitView.swift)**:
    - `activePaneId` 변경 시 `workspaceVM.activePane = ActivePaneContext(paneId: newId, model: model, workspace: workspace)` 업데이트
    - 하위 뷰에 `.environment(\.activePane, ActivePaneContext(...))` 주입 (LeafPaneView 내부에서 확장 가능성 대비)

**파일**:

| 작업 | 경로 | 비고 |
|------|------|------|
| 신규 | `app/Sources/Shell/Splits/ActivePaneProvider.swift` | 2종 Key + extension + `ActivePaneContext` + `.empty` |
| 수정 | `app/Sources/ViewModels/WorkspaceViewModel.swift` | `var activePane: ActivePaneContext` 프로퍼티 추가 (~5줄) |
| 수정 | `app/Sources/Shell/Splits/PaneSplitView.swift` | `PaneSplitContainerView` 에 `@Environment(WorkspaceViewModel.self)` 주입 + activePaneId 변경 시 업데이트 (~15줄 추가) |
| 신규 | `app/Tests/ActivePaneProviderTests.swift` | 기본값/주입/중첩/leaf assertion 검증 (≥5건) |

**테스트 (AC-1.1 ~ AC-1.5 대응)**:

- `test_defaultActivePaneContext_hasAllNilFields` — `.empty` 상수 검증
- `test_environmentInjection_propagatesContext` — `.environment(\.activePane, ctx)` 주입 후 `@Environment` 조회
- `test_nestedEnvironmentOverride_wins` — 중첩 `.environment` 시 가장 가까운 값이 이긴다
- `test_activePaneChange_updatesWorkspaceViewModel` — `activePaneId` 변경 시 `workspaceVM.activePane.paneId` 반영
- `test_splitNode_doesNotBecomeActive` — split (non-leaf) 노드 id 주입 시 assertion failure (DEBUG) 또는 무시 (RELEASE)

**의존**: 없음 (MS-1 은 독립 시작 가능)

**완료 기준**: AC-1.1 ~ AC-1.5 통과, 기존 339 테스트 regression 0, 빌드 warning 증가 0.

---

### 3.2 MS-2 — TerminalSurface GhosttyHost 실연결 (RG-M2.5-2, P-1 해소)

**목표**: `TerminalSurfacePlaceholder` 를 제거하고 `SurfaceRouter.terminal` 케이스가 실제 `TerminalSurface(workspace:)` 를 렌더링하도록 교체. `WorkspaceSnapshot` 은 `.environment(\.activeWorkspace)` 로 주입한다.

**디자인**:

- **`PaneContainer.swift` (Shell/Content/)**:
    - `loadModelIfNeeded(for:)` 내부에서 `workspaceVM.workspace(id: workspaceUuid)` 로 `WorkspaceSnapshot?` 확보
    - `contentView(for:)` 가 `PaneSplitContainerView(model:)` 를 반환할 때 `.environment(\.activeWorkspace, snapshot)` 주입

- **`SurfaceRouter` 수정 (Shell/Splits/PaneSplitView.swift:252-305)**:
    - `@Environment(\.activeWorkspace) var activeWorkspace: WorkspaceSnapshot?` 추가
    - `.terminal` 케이스를:
        ```
        if let ws = activeWorkspace {
            TerminalSurface(workspace: ws)   // 실연결
        } else {
            TerminalFallback_Unavailable()   // workspace nil 시 안내
        }
        ```
        (코드 스니펫은 설명용 의사코드; 실제 Run phase 에서 작성)

- **`TerminalSurfacePlaceholder` 제거**:
    - `PaneSplitView.swift:309-334` struct 전체 삭제
    - `grep -r "TerminalSurfacePlaceholder" app/Sources/` 결과 0건 보장

- **`GhosttyHost` 본문 교체 (Surfaces/Terminal/TerminalSurface.swift:70-96)**:
    - 기존 placeholder 텍스트 (`"ghostty-vt.xcframework loaded"`, `"(Ghostty Metal surface will render here — wiring in MS-6)"`) 제거
    - 실제 GhosttyKit Metal surface 초기화 + `NSViewRepresentable` 래핑
    - 초기화 실패 시 `onFailure()` 호출 (기존 경로 유지)
    - `MOAI_TERMINAL_BACKEND=nstext` 분기는 `TerminalSurface.swift:35-43` 기존 `if` 문 그대로 유지 → `TerminalFallback` 표시

**파일**:

| 작업 | 경로 | 비고 |
|------|------|------|
| 수정 | `app/Sources/Shell/Content/PaneContainer.swift` | `.environment(\.activeWorkspace, snapshot)` 주입 (~5줄) |
| 수정 | `app/Sources/Shell/Splits/PaneSplitView.swift` | `SurfaceRouter` 의 `.terminal` 케이스 재작성 + `TerminalSurfacePlaceholder` struct 제거 (~30줄 삭제 + ~10줄 추가) |
| 수정 | `app/Sources/Surfaces/Terminal/TerminalSurface.swift` | `GhosttyHost.body` 를 실 GhosttyKit 래퍼로 교체 (~30줄 교체) |
| 신규 | `app/Tests/TerminalSurfaceEnvironmentTests.swift` | 환경 주입 + fallback 검증 (≥3건) |

**테스트 (AC-2.1 ~ AC-2.6 대응)**:

- `test_TerminalSurfacePlaceholder_grep_zero` — 소스 검색 결과 0건 (grep 기반 빌드 테스트 또는 CI 검증)
- `test_SurfaceRouter_terminal_withActiveWorkspace_rendersTerminalSurface` — `@Environment(\.activeWorkspace)` 주입 + `.terminal` 케이스 → `TerminalSurface` 생성
- `test_SurfaceRouter_terminal_withNilWorkspace_rendersFallback` — nil 주입 시 fallback 안내 뷰
- `test_GhosttyHost_initFailure_triggersOnFailure` — mock 기반 실패 시 `loadFailed = true` 로 전환 (기존 경로)
- `test_nstextBackend_usesTerminalFallback` — 환경변수 `MOAI_TERMINAL_BACKEND=nstext` 설정 시 `TerminalFallback` 렌더링

**의존**: MS-1 완료 (`WorkspaceEnvironmentKey`)

**완료 기준**: AC-2.1 ~ AC-2.6 통과, `grep -r "TerminalSurfacePlaceholder" app/Sources/` 0건, 기존 339 테스트 regression 0, Metal Toolchain 설치 환경에서 수동 UI 검증 시 실제 터미널 렌더 확인.

---

### 3.3 MS-3 — Command Palette 콜백 활성화 (RG-M2.5-3, RG-M2.5-4, P-2/P-3 해소)

**목표**: `RootSplitView.setupPaletteController` 의 `onSurfaceOpen` / `onPaneSplit` no-op 클로저를 실제 구현으로 교체. `workspaceVM.activePane` 을 소비하여 활성 pane 의 `TabBarViewModel` / `PaneTreeModel` 에 접근한다.

**디자인 상세**:

- **`RootSplitView.setupPaletteController` 재작성 (Shell/RootSplitView.swift:68-93)**:

  기존 no-op 패턴:
  ```
  onSurfaceOpen: { _ in
      // TODO(MS-7)
  },
  onPaneSplit: { _ in
      // TODO(MS-7)
  }
  ```

  교체 후 의사코드:
  ```
  onSurfaceOpen: { kind in
      let ctx = vm.activePane
      guard let paneId = ctx.paneId else {
          Logger.info("Command Palette: onSurfaceOpen ignored — no active pane")
          return
      }
      guard let tabModel = lookupTabModel(paneId: paneId) else { return }
      _ = tabModel.newTab(kind: kind)    // M2 범위 5종 직접 렌더, M3+ 는 NotYetImplementedSurface
  },
  onPaneSplit: { direction in
      let ctx = vm.activePane
      guard let paneId = ctx.paneId, let model = ctx.model else {
          Logger.info("Command Palette: onPaneSplit ignored — no active pane context")
          return
      }
      let splitKind: SplitKind = (direction == .horizontal) ? .horizontal : .vertical
      _ = model.splitActive(paneId, direction: splitKind)
      // activePaneId 반영은 PaneSplitContainerView 의 @State 가 담당하므로
      // PaneTreeModel 이 새 id 반환 → workspaceVM.activePane 업데이트는 MS-1 훅에서 처리
  }
  ```

- **`lookupTabModel(paneId:)` 설계**:
    - 옵션 A (채택): `LeafPaneView` 가 `task { ... }` 블록에서 생성한 `TabBarViewModel` 을 `workspaceVM.tabModels[paneId]` 사전에 등록 → RootSplitView 에서 조회.
    - 옵션 B (대안): `ActivePaneContext` 에 `tabModel: TabBarViewModel?` 필드 추가 → `LeafPaneView` 가 활성화 시 업데이트. (research.md §9 개방 질문 2 의 대안.)
    - **선택**: 옵션 A. `WorkspaceViewModel.tabModels: [Int64: TabBarViewModel] = [:]` `@Observable` 사전을 추가. `LeafPaneView` 의 `task` 블록에서 생성 후 등록, pane close 시 제거. ActivePaneContext 는 paneId 만 보유하며, tabModel 조회는 ViewModel 에서.

- **Pane split 성공 후 `activePaneId` 전파**:
    - 기존 키보드 경로 (`PaneSplitContainerView.swift:378-389`) 에서 `activePaneId = newId` 할당 후 MS-1 에서 추가한 훅이 `workspaceVM.activePane` 업데이트.
    - 팔레트 경로도 동일 경로를 거치도록 `onPaneSplit` 콜백이 `model.splitActive` 호출 시 자동으로 반영됨 (`PaneSplitContainerView` 의 `@State activePaneId` 는 이 경로에서는 직접 갱신되지 않지만, `SplitKind` 변경이 `model.nodes` 를 업데이트하므로 SwiftUI 재렌더 발생. activePaneId 지연 갱신 이슈 방지를 위해 MS-3 에서 `PaneSplitContainerView` 에 `onChange(of: model.rootId)` 기반 재계산 로직 또는 `workspaceVM.activePane` 의 새 pane id 를 역으로 `activePaneId` 에 반영하는 onChange 핸들러를 추가).

**파일**:

| 작업 | 경로 | 비고 |
|------|------|------|
| 수정 | `app/Sources/Shell/RootSplitView.swift` | `setupPaletteController` 재작성 (~40줄 교체) |
| 수정 | `app/Sources/ViewModels/WorkspaceViewModel.swift` | `tabModels: [Int64: TabBarViewModel]` 사전 추가 + register/unregister 메서드 (~15줄) |
| 수정 | `app/Sources/Shell/Splits/PaneSplitView.swift` | `LeafPaneView.task` 블록에서 `workspaceVM.tabModels[paneId] = model` 등록 (~5줄) |
| 수정 | `app/Sources/Shell/Splits/PaneSplitView.swift` | `PaneSplitContainerView` 에 `onChange(of: model.rootId)` 기반 activePaneId 보정 (~10줄) |
| 신규 | `app/Tests/CommandPaletteSurfaceOpenTests.swift` | 5종 SurfaceKind + nil 케이스 (≥6건) |
| 신규 | `app/Tests/CommandPalettePaneSplitTests.swift` | 2종 방향 + nil + 반환값 (≥4건) |

**테스트 (AC-3.1 ~ AC-3.5, AC-4.1 ~ AC-4.5 대응)**:

- **SurfaceOpen**:
    - `test_onSurfaceOpen_filetree_callsNewTabWithFiletree`
    - `test_onSurfaceOpen_markdown_callsNewTabWithMarkdown`
    - `test_onSurfaceOpen_image_callsNewTabWithImage`
    - `test_onSurfaceOpen_browser_callsNewTabWithBrowser`
    - `test_onSurfaceOpen_terminal_callsNewTabWithTerminal`
    - `test_onSurfaceOpen_nilActivePane_noops`

- **PaneSplit**:
    - `test_onPaneSplit_horizontal_callsSplitActiveWithHorizontal`
    - `test_onPaneSplit_vertical_callsSplitActiveWithVertical`
    - `test_onPaneSplit_nilPaneIdOrModel_noops`
    - `test_onPaneSplit_newPaneId_reflectedInActivePaneId`

- **TODO(MS-7) 주석 제거 검증**: grep `TODO(MS-7)` 결과 0건 (RootSplitView.swift 내).

**의존**: MS-1 완료 (`ActivePaneContext` + `workspaceVM.activePane`), MS-2 완료 (`TerminalSurface` 실연결 — AC-3.5 의 `.terminal` 케이스 수동 검증용).

**완료 기준**: AC-3.1 ~ AC-3.5, AC-4.1 ~ AC-4.5 통과, `grep "TODO(MS-7)" app/Sources/Shell/RootSplitView.swift` 0건, 기존 339 테스트 regression 0, 수동 검증 체크리스트 (spec.md §7.4) 5단계 모두 통과.

---

## 4. 태스크 분해 (T-M2.5-001 ~ T-M2.5-013)

### MS-1 클러스터

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-M2.5-001 | `ActivePaneContext` struct + `.empty` 정의 | `app/Sources/Shell/Splits/ActivePaneProvider.swift` (신규) | — |
| T-M2.5-002 | `ActivePaneProviderKey`, `WorkspaceEnvironmentKey` + `EnvironmentValues` extension | 동일 | T-M2.5-001 |
| T-M2.5-003 | `WorkspaceViewModel.activePane` `@Observable` 프로퍼티 추가 | `app/Sources/ViewModels/WorkspaceViewModel.swift` | T-M2.5-001 |
| T-M2.5-004 | `PaneSplitContainerView` 에 `@Environment(WorkspaceViewModel.self)` 연결 + activePaneId 변경 시 `workspaceVM.activePane` 업데이트 + `.environment(\.activePane, ...)` 하위 주입 | `app/Sources/Shell/Splits/PaneSplitView.swift` | T-M2.5-002, T-M2.5-003 |
| T-M2.5-005 | `ActivePaneProviderTests.swift` (≥5건, AC-1.1 ~ AC-1.5 검증) | `app/Tests/ActivePaneProviderTests.swift` (신규) | T-M2.5-004 |

### MS-2 클러스터

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-M2.5-006 | `PaneContainer` 가 `WorkspaceSnapshot` 확보 후 `.environment(\.activeWorkspace, ...)` 주입 | `app/Sources/Shell/Content/PaneContainer.swift` | T-M2.5-002 |
| T-M2.5-007 | `SurfaceRouter.terminal` 케이스 → `TerminalSurface(workspace:)` 실연결 + `TerminalSurfacePlaceholder` struct 제거 | `app/Sources/Shell/Splits/PaneSplitView.swift` | T-M2.5-006 |
| T-M2.5-008 | `GhosttyHost.body` 를 실 GhosttyKit Metal 래퍼로 교체 (placeholder 텍스트 제거, 실패 시 `onFailure()` 유지) | `app/Sources/Surfaces/Terminal/TerminalSurface.swift` | T-M2.5-007 |
| T-M2.5-009 | `TerminalSurfaceEnvironmentTests.swift` (≥3건, AC-2.1 ~ AC-2.5 검증) | `app/Tests/TerminalSurfaceEnvironmentTests.swift` (신규) | T-M2.5-008 |

### MS-3 클러스터

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-M2.5-010 | `WorkspaceViewModel.tabModels: [Int64: TabBarViewModel]` 사전 + register/unregister 메서드 | `app/Sources/ViewModels/WorkspaceViewModel.swift` | T-M2.5-003 |
| T-M2.5-011 | `LeafPaneView.task` 블록에서 `tabModels[paneId]` 등록 + pane close 시 해제 | `app/Sources/Shell/Splits/PaneSplitView.swift` | T-M2.5-010 |
| T-M2.5-012 | `RootSplitView.setupPaletteController` 의 `onSurfaceOpen` 실제 구현 (5종 SurfaceKind → `newTab(kind:)`) | `app/Sources/Shell/RootSplitView.swift` | T-M2.5-011 |
| T-M2.5-013 | `RootSplitView.setupPaletteController` 의 `onPaneSplit` 실제 구현 + `PaneSplitDirection → SplitKind` 매핑 + 새 pane id 반영 | `app/Sources/Shell/RootSplitView.swift` | T-M2.5-012 |
| T-M2.5-014 | `CommandPaletteSurfaceOpenTests.swift` (≥6건, AC-3.1 ~ AC-3.4 검증) | `app/Tests/CommandPaletteSurfaceOpenTests.swift` (신규) | T-M2.5-012 |
| T-M2.5-015 | `CommandPalettePaneSplitTests.swift` (≥4건, AC-4.1 ~ AC-4.4 검증) | `app/Tests/CommandPalettePaneSplitTests.swift` (신규) | T-M2.5-013 |

### 크로스-마일스톤

| Task | 설명 | 생성/수정 파일 | 의존 |
|------|------|---------------|------|
| T-M2.5-016 | `@MX:ANCHOR` (ActivePaneProvider.swift) + `@MX:NOTE` (GhosttyHost, onSurfaceOpen, onPaneSplit) 부착 | 해당 파일들 | T-M2.5-013 |
| T-M2.5-017 | Regression sweep: `cargo test --workspace` 233/233 + `xcodebuild test` 106+신규 전부 통과 확인 | — | 전체 |
| T-M2.5-018 | 수동 UI 검증 (spec.md §7.4 체크리스트 5단계) — Metal Toolchain 설치 환경에서 실행 | — | 전체 |

총 태스크 수: **18건** (MS-1: 5건, MS-2: 4건, MS-3: 6건, 크로스: 3건)

---

## 5. 위험 및 완화

| 리스크 | 영향 | 완화 |
|--------|------|------|
| GhosttyKit API 래핑 시 Metal Toolchain 의존 | Metal Toolchain 없는 CI 에서 빌드 실패 가능 | 기존 `TerminalFallback` 분기 유지 (MOAI_TERMINAL_BACKEND=nstext). CI 는 `nstext` 로 실행, 로컬 수동 검증만 ghostty 백엔드 사용 |
| `@Observable tabModels` 사전 업데이트 타이밍 이슈 | pane 생성 직후 Command Palette 실행 시 tabModels 미등록 상태일 수 있음 | `LeafPaneView.task` 블록이 `await model.load()` 완료 후 등록하므로, 사용자가 Cmd+K 를 누르기 전에 거의 항상 완료됨. 실패 시 AC-3.4 의 no-op + info 로그로 graceful degradation |
| `PaneSplitContainerView.@State activePaneId` 와 `workspaceVM.activePane.paneId` 동기화 | 양방향 바인딩 루프 가능성 | 단방향 only: `activePaneId` 변경 → `workspaceVM.activePane` 업데이트. 역방향은 `model.splitActive` 반환값으로만 발생 |
| `SurfaceRouter` 에 `@Environment` 추가 시 SwiftUI 재렌더 폭증 | FPS 저하 | `SurfaceRouter` 는 `LeafPaneView` 의 자식. activePaneId 변경 시에만 재렌더되므로 영향 제한적 |
| FFI <1ms regression | SPEC-M2-001 C-7 기준 유지 실패 | Rust 파일 변경 0건. FFI 호출 경로 (`splitActive`, `newTab`) 는 기존 그대로. `FFIBenchmarkTests.swift` 재실행으로 확인 |
| 기존 테스트 regression | 339 건 중 1건이라도 실패 시 MS 전체 롤백 | 각 MS 완료 후 즉시 `xcodebuild test` + `cargo test --workspace` 실행, MS-1 은 마지막 단계에 5건 신규 테스트로 환경값 동작 보장 |
| Command Palette 오버레이 환경값 미공유 | `.environment(\.activePane)` 가 overlay 에 전달되지 않음 | `WorkspaceViewModel.activePane` 프로퍼티 경로로 우회 (research.md §5.4 근거). EnvironmentKey 는 `LeafPaneView` 하위만 대상 |
| `ActivePaneContext.model: PaneTreeModel?` 의 `@MainActor` 격리 | Sendable 경고 | `ActivePaneContext` 를 `@MainActor` 로 한정. 프로젝트 내 사용 지점 모두 `@MainActor` 컨텍스트이므로 전파 문제 없음 |

---

## 6. MX 태그 전략

본 SPEC 는 Swift 영역 변경에 한정되므로 MX 태그 언어는 `.moai/config/sections/language.yaml` 의 `code_comments: ko` 를 따른다.

### 6.1 신규 @MX:ANCHOR

| 파일 | 태그 | @MX:REASON |
|------|------|-----------|
| `ActivePaneProvider.swift` `ActivePaneContext` struct | `@MX:ANCHOR: [AUTO] 활성 pane 환경값의 유일한 타입 (fan_in>=3)` | `@MX:REASON: [AUTO] RootSplitView (Command Palette), PaneSplitContainerView (상태 주입), LeafPaneView/SurfaceRouter (소비) 세 경로 공유` |
| `ActivePaneProvider.swift` `EnvironmentValues.activePane` | `@MX:ANCHOR: [AUTO] @Environment(\.activePane) 의 유일한 진입점` | `@MX:REASON: [AUTO] 환경값 소비자는 전부 이 computed property 를 거친다` |

### 6.2 신규/갱신 @MX:NOTE

| 파일 | 태그 |
|------|------|
| `TerminalSurface.swift` `GhosttyHost` 본문 (기존 `@MX:WARN: GhosttyKit 초기화는 Metal Toolchain 의존 ...` 은 유지) | `@MX:NOTE: [AUTO] MS-2 에서 placeholder 제거 — 실제 GhosttyKit Metal surface 렌더링 적용. placeholder 텍스트 3줄 삭제` (기존 placeholder 관련 NOTE 교체) |
| `RootSplitView.swift` `onSurfaceOpen` 클로저 | `@MX:NOTE: [AUTO] MS-3 완료 — workspaceVM.activePane 기반 TabBarViewModel.newTab(kind:) 호출. 기존 TODO(MS-7) 제거` |
| `RootSplitView.swift` `onPaneSplit` 클로저 | `@MX:NOTE: [AUTO] MS-3 완료 — workspaceVM.activePane.model.splitActive 호출. 키보드 단축키 경로와 동일 호출 시퀀스` |
| `PaneSplitView.swift` `SurfaceRouter.terminal` 케이스 | `@MX:NOTE: [AUTO] MS-2 — @Environment(\.activeWorkspace) 주입으로 TerminalSurface(workspace:) 직접 렌더` (기존 "MS-4+ 에서 실제 워크스페이스..." NOTE 제거) |
| `WorkspaceViewModel.swift` `activePane` 프로퍼티 | `@MX:NOTE: [AUTO] Command Palette 오버레이는 NavigationSplitView 트리 밖이므로 @Environment 대신 @Observable 프로퍼티 경로 사용` |
| `WorkspaceViewModel.swift` `tabModels` 사전 | `@MX:NOTE: [AUTO] paneId → TabBarViewModel 사전. LeafPaneView.task 에서 등록, closePane 시 해제` |

### 6.3 기존 @MX:ANCHOR 갱신

| 파일 | 기존 태그 | 갱신 사유 |
|------|----------|-----------|
| `PaneSplitView.swift` `PaneSplitContainerView` `@MX:ANCHOR` | 기존 유지. fan_in 변화 없음 | — |
| `TabBarViewModel.swift` `@MX:ANCHOR` | 기존 유지. 새 caller `RootSplitView.setupPaletteController` 추가로 fan_in+1 — `@MX:REASON` 업데이트: "RootSplitView (Command Palette), LeafPaneView, T-049 테스트" | MS-3 완료 시점 |
| `CommandRegistry.swift` `@MX:ANCHOR` | 기존 유지 | — |

### 6.4 제거 대상 @MX:NOTE

| 파일 | 제거 사유 |
|------|----------|
| `PaneSplitView.swift` "MS-3 에서 leaf 노드 내부는 TabBarView + SurfaceProtocol 로 교체 예정" (18-19행) | MS-3 완료 (SPEC-M2-001) — 구식 정보 |
| `PaneSplitView.swift` "MS-4+ 에서 실제 워크스페이스와 연결 예정" (TerminalSurfacePlaceholder 관련) | struct 삭제와 함께 제거 |
| `PaneSplitView.swift` "resolveWorkspacePath() 는 MS-6+ 에서 @Environment WorkspaceSnapshot 주입 후 교체 예정" (SurfaceRouter) | MS-2 완료로 해소. `@Environment(\.activeWorkspace)` 사용으로 전환 후 NOTE 삭제 |

---

## 7. 참조

- **SPEC 기반**: `.moai/specs/SPEC-M2-002/spec.md` (v1.0.0, 2026-04-16) — RG-M2.5-1 ~ RG-M2.5-4, AC-1 ~ AC-4, AC-G, Exclusions 12건
- **코드 컨텍스트**: `.moai/specs/SPEC-M2-002/research.md` — §1~§9 (placeholder 위치, 환경 패턴, 의존 타입)
- **선행 SPEC**: `.moai/specs/SPEC-M2-001/spec.md` (v1.2.0, completed 2026-04-15) — RG-M2-1 (Pane), RG-M2-2 (Tab), RG-M2-3 (Command Palette)
- **완료 보고서**: `.moai/specs/SPEC-M2-001/m2-completion-report.md` — §알려진 제한 사항 (placeholder 원본), §M3 권장 다음 액션 (우선순위)
- **설계 문서**: `DESIGN.v4.md` §3.1 (Pane/Surface 아키텍처), §6 (DB 스키마), §8 (Shell/Surfaces 디렉토리 구조)
- **가이드**: `.claude/rules/moai/languages/swift.md` (Swift 6 `@Observable`, `@Environment` 패턴), `.claude/rules/moai/workflow/mx-tag-protocol.md` (ANCHOR fan_in ≥ 3, NOTE 갱신)

---

## 8. 산출물 요약

### Swift (신규)
- `app/Sources/Shell/Splits/ActivePaneProvider.swift` (MS-1)
- `app/Tests/ActivePaneProviderTests.swift` (MS-1)
- `app/Tests/TerminalSurfaceEnvironmentTests.swift` (MS-2)
- `app/Tests/CommandPaletteSurfaceOpenTests.swift` (MS-3)
- `app/Tests/CommandPalettePaneSplitTests.swift` (MS-3)

### Swift (수정)
- `app/Sources/ViewModels/WorkspaceViewModel.swift` (MS-1 + MS-3: `activePane`, `tabModels`)
- `app/Sources/Shell/Splits/PaneSplitView.swift` (MS-1 + MS-2 + MS-3: environment 주입, SurfaceRouter, TerminalSurfacePlaceholder 제거, LeafPaneView tabModels 등록, PaneSplitContainerView onChange)
- `app/Sources/Shell/Content/PaneContainer.swift` (MS-2: `.environment(\.activeWorkspace)`)
- `app/Sources/Surfaces/Terminal/TerminalSurface.swift` (MS-2: GhosttyHost body 교체)
- `app/Sources/Shell/RootSplitView.swift` (MS-3: onSurfaceOpen/onPaneSplit 실제 구현)

### Xcode 프로젝트
- `app/MoAIStudio.xcodeproj/project.pbxproj` — 신규 파일 5개 등록 (SPEC-M2-001 MS-3 에서 정립된 절차 재사용)

### Rust
- 변경 0건 (FFI 표면 유지)

### CI/CD
- 변경 0건 (SPEC-M2-001 MS-7 워크플로우 재사용)

---

신뢰도: HIGH. 4개 placeholder 원본 위치, 기존 @Environment 패턴, 의존 타입 시그니처, 테스트 mock 기반 모두 research.md 에서 확인 완료. Run phase 는 본 plan.md 의 T-M2.5-001 ~ T-M2.5-018 순서로 RED-GREEN-REFACTOR (TDD) 또는 ANALYZE-PRESERVE-IMPROVE (DDD) 사이클에 투입 가능 (quality.yaml 에서 선택).
