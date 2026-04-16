# SPEC-M2-002 Tasks — TDD/DDD 사이클 실행 단위

---
spec_id: SPEC-M2-002
version: 1.0.0
created: 2026-04-16
---

## 개요

본 문서는 `plan.md` 의 T-M2.5-001 ~ T-M2.5-018 태스크를 RED-GREEN-REFACTOR (TDD, 기본) 또는 ANALYZE-PRESERVE-IMPROVE (DDD, quality.yaml 선택 시) 사이클 단위로 상세화한다. 각 태스크는 독립 커밋 단위이며, 완료 시 체크박스를 업데이트한다.

**마일스톤 매핑**:
- MS-1 (P-4 해소): T-M2.5-001 ~ T-M2.5-005
- MS-2 (P-1 해소): T-M2.5-006 ~ T-M2.5-009
- MS-3 (P-2, P-3 해소): T-M2.5-010 ~ T-M2.5-015
- 크로스: T-M2.5-016 ~ T-M2.5-018

**AC 매핑**: acceptance.md §1 의 AC-1.x / AC-2.x / AC-3.x / AC-4.x 를 각 태스크가 커버한다.

---

## MS-1: ActivePaneProvider (P-4 해소)

### T-M2.5-001: `ActivePaneContext` struct + `.empty` 상수

- **목표**: `ActivePaneContext` value struct 정의 및 기본 상수 제공
- **AC**: AC-1.1
- **파일**:
    - 신규: `app/Sources/Shell/Splits/ActivePaneProvider.swift`
- **단계**:
    - [x] **RED**: `ActivePaneProviderTests.test_defaultActivePaneContext_hasAllNilFields()` 작성 (파일 존재하지 않아 컴파일 실패)
    - [x] **GREEN**: struct 정의 — `paneId: Int64?`, `model: PaneTreeModel?`, `workspace: WorkspaceSnapshot?`, `static let empty`, `@MainActor` 격리
    - [x] **REFACTOR**: Equatable 준수 (paneId 기준만 비교, reference 필드 제외), doc comment 추가
- **완료 기준**: 테스트 PASS, 신규 struct 가 `ActivePaneContext.empty` 로 조회 가능

### T-M2.5-002: `ActivePaneProviderKey` + `WorkspaceEnvironmentKey` + extension

- **목표**: 2종 `EnvironmentKey` 및 `EnvironmentValues` computed property 노출
- **AC**: AC-1.2, AC-1.3
- **파일**:
    - 수정: `app/Sources/Shell/Splits/ActivePaneProvider.swift` (T-M2.5-001 동일 파일 확장)
- **단계**:
    - [x] **RED**: `test_environmentInjection_propagatesContext()`, `test_nestedEnvironmentOverride_wins()` 작성 (테스트에서 `.environment(\.activePane, ...)` 사용 시 key 미정의로 컴파일 실패)
    - [x] **GREEN**: `ActivePaneProviderKey: EnvironmentKey` with `defaultValue = .empty`, `WorkspaceEnvironmentKey: EnvironmentKey` with `defaultValue: WorkspaceSnapshot? = nil`, `EnvironmentValues` extension 에 `activePane` / `activeWorkspace` computed property 추가
    - [x] **REFACTOR**: public vs internal 접근 수준 정리 (ActivePaneProvider.swift 는 app target 내부 공유 → public 필요 없음), doc comment 추가
- **완료 기준**: 두 테스트 PASS, `.environment(\.activePane, ctx)` 문법으로 주입/조회 가능

### T-M2.5-003: `WorkspaceViewModel.activePane` `@Observable` 프로퍼티

- **목표**: Command Palette 오버레이 경로에서 활성 pane 참조 가능하게 `@Observable` 프로퍼티 추가
- **AC**: AC-1.4 (일부), AC-3.4, AC-4.3
- **파일**:
    - 수정: `app/Sources/ViewModels/WorkspaceViewModel.swift`
- **단계**:
    - [x] **RED**: `ActivePaneProviderTests.test_activePaneChange_updatesWorkspaceViewModel()` 작성 (프로퍼티 없어 컴파일 실패)
    - [x] **GREEN**: `public var activePane: ActivePaneContext = .empty` 추가 (`@Observable` 매크로가 자동 관찰 대상화)
    - [x] **REFACTOR**: doc comment — "Command Palette 오버레이 경로용. PaneSplitContainerView 의 activePaneId 변경 훅이 유지"
- **완료 기준**: 테스트 PASS, `vm.activePane = ActivePaneContext(...)` 직접 할당 가능

### T-M2.5-004: `PaneSplitContainerView` activePaneId ↔ workspaceVM 동기화 + 하위 환경값 주입

- **목표**: `activePaneId` 변경을 `workspaceVM.activePane` 으로 전파하고 하위 뷰에 `.environment(\.activePane, ctx)` 주입
- **AC**: AC-1.4, AC-1.5, AC-1.6
- **파일**:
    - 수정: `app/Sources/Shell/Splits/PaneSplitView.swift`
- **단계**:
    - [x] **RED**: `test_activePaneChange_updatesWorkspaceViewModel` (T-M2.5-003 재사용) + `test_splitNode_doesNotBecomeActive()` 작성
    - [x] **GREEN**:
        - `PaneSplitContainerView` 에 `@Environment(WorkspaceViewModel.self) private var workspaceVM` 추가
        - `onChange(of: activePaneId)` 로 `workspaceVM.activePane = ActivePaneContext(paneId: activePaneId, model: model, workspace: workspaceVM.workspace(id: ...))` 업데이트
        - split (non-leaf) 노드 id 할당 시도 시 `assert(nodes[newId]?.split == .leaf, "active pane must be leaf")`
        - `.environment(\.activePane, ActivePaneContext(...))` 를 하위 PaneSplitView 에 주입
    - [x] **REFACTOR**: `activePaneId == nil` 케이스 처리 (ActivePaneContext.empty 사용), MS-1 NOTE 제거 (TODO(MS-7) 주석이 아직 있다면 유지 — MS-3 에서 제거)
- **완료 기준**: 두 테스트 PASS, 기존 PaneTreeModelTests 10건 regression 0

### T-M2.5-005: `ActivePaneProviderTests.swift` 완성 (≥5건)

- **목표**: AC-1.1 ~ AC-1.5 커버하는 테스트 스위트 완성
- **AC**: AC-1.1, AC-1.2, AC-1.3, AC-1.4, AC-1.5
- **파일**:
    - 신규: `app/Tests/ActivePaneProviderTests.swift`
- **단계**:
    - [x] **RED**: T-M2.5-001~T-M2.5-004 에서 순차 작성된 테스트를 한 파일에 통합. 최소 5개 테스트 함수 — `test_defaultActivePaneContext_hasAllNilFields`, `test_environmentInjection_propagatesContext`, `test_nestedEnvironmentOverride_wins`, `test_activePaneChange_updatesWorkspaceViewModel`, `test_splitNode_doesNotBecomeActive`
    - [x] **GREEN**: T-M2.5-001~T-M2.5-004 구현으로 모두 통과
    - [x] **REFACTOR**: test helper 추출 (`makeMockContext()`), `@MainActor` 부착, Xcode project 에 파일 등록
- **완료 기준**: 5건 PASS, 기존 339 테스트 regression 0

---

## MS-2: TerminalSurface GhosttyHost 실연결 (P-1 해소)

### T-M2.5-006: `PaneContainer` 가 `WorkspaceSnapshot` 주입

- **목표**: `PaneContainer.loadModelIfNeeded` 에서 `WorkspaceSnapshot` 확보 후 하위에 `.environment(\.activeWorkspace, ...)` 주입
- **AC**: AC-2.2
- **파일**:
    - 수정: `app/Sources/Shell/Content/PaneContainer.swift`
- **단계**:
    - [x] **RED**: `TerminalSurfaceEnvironmentTests.test_paneContainer_injectsActiveWorkspace()` 작성 (환경값 미주입으로 실패)
    - [x] **GREEN**: `contentView(for: workspaceId)` 반환 시 `workspaceVM.workspace(id: workspaceId)` 호출 결과 `WorkspaceSnapshot?` 를 `.environment(\.activeWorkspace, snapshot)` modifier 로 주입
    - [x] **REFACTOR**: nil 케이스 명확화, doc comment "T-M2.5-006: MS-2 GhosttyHost 실연결을 위한 WorkspaceSnapshot 전파"
- **완료 기준**: 테스트 PASS, PaneContainer 하위 뷰에서 `@Environment(\.activeWorkspace)` 조회 가능

### T-M2.5-007: `SurfaceRouter.terminal` → 실 `TerminalSurface(workspace:)` + placeholder struct 제거

- **목표**: `TerminalSurfacePlaceholder` 를 제거하고 실제 `TerminalSurface(workspace: snapshot)` 렌더
- **AC**: AC-2.5 (nil 폴백), 모든 AC-2 의 선행
- **파일**:
    - 수정: `app/Sources/Shell/Splits/PaneSplitView.swift`
- **단계**:
    - [x] **RED**: `test_SurfaceRouter_terminal_withActiveWorkspace_rendersTerminalSurface()`, `test_SurfaceRouter_terminal_withNilWorkspace_rendersFallback()` 작성
    - [x] **GREEN**:
        - `SurfaceRouter` struct 에 `@Environment(\.activeWorkspace) private var activeWorkspace: WorkspaceSnapshot?` 추가
        - `.terminal, .none` 케이스를 `if let ws = activeWorkspace { TerminalSurface(workspace: ws) } else { WorkspaceUnavailablePlaceholder() }` 형태로 교체
        - `TerminalSurfacePlaceholder` struct (PaneSplitView.swift:313-334) 전체 삭제
        - `TerminalSurfacePlaceholder(paneId: paneId)` 호출 지점 (PaneSplitView.swift:270) 교체
        - (신규) `WorkspaceUnavailablePlaceholder` 경량 안내 뷰 정의 — "워크스페이스를 선택하세요"
    - [x] **REFACTOR**: `grep -r "TerminalSurfacePlaceholder" app/Sources/` 결과 0건 확인. 기존 "MS-5+ 에서 WorkspaceSnapshot 주입" NOTE 제거
- **완료 기준**: 두 테스트 PASS, grep 결과 0건, PaneTreeModelTests/TabBarViewModelTests regression 0

### T-M2.5-008: `GhosttyHost.body` 실 GhosttyKit 래퍼 교체

- **목표**: placeholder 텍스트 3줄 (`"ghostty-vt.xcframework loaded"`, `"workspace: ..."`, `"(Ghostty Metal surface will render here — wiring in MS-6)"`) 제거하고 실제 GhosttyKit Metal surface 래핑
- **AC**: AC-2.1, AC-2.4
- **파일**:
    - 수정: `app/Sources/Surfaces/Terminal/TerminalSurface.swift`
- **단계**:
    - [x] **RED**: `test_GhosttyHost_initFailure_triggersOnFailure()` 작성 (mock GhosttyKit 초기화 실패 시 `loadFailed = true` 검증)
    - [x] **GREEN**:
        - `GhosttyHost.body` 를 `NSViewRepresentable` 래퍼로 교체 (GhosttyKit xcframework 의 Metal surface 뷰 생성)
        - 초기화 성공 시 Metal surface 렌더, 실패 시 `onFailure()` 호출
        - `workspace.name` / `workspace.id` 등을 GhosttyKit 설정으로 전달 (해당 API 시그니처는 GhosttyKit 헤더 참조)
    - [x] **REFACTOR**: 에러 경로 명확화, `@MX:WARN: GhosttyKit 초기화는 Metal Toolchain 의존` 기존 WARN 유지, placeholder 관련 주석 제거
- **완료 기준**: 테스트 PASS, Metal Toolchain 환경에서 AC-2.1 수동 검증 통과 (Metal surface 가시적 렌더)

### T-M2.5-009: `TerminalSurfaceEnvironmentTests.swift` 완성 (≥3건)

- **목표**: AC-2.3 ~ AC-2.5 커버. AC-2.1/AC-2.2 는 수동 검증 체크리스트로 위임
- **AC**: AC-2.3, AC-2.4, AC-2.5
- **파일**:
    - 신규: `app/Tests/TerminalSurfaceEnvironmentTests.swift`
- **단계**:
    - [x] **RED**: 3개 테스트 함수 — `test_SurfaceRouter_terminal_withActiveWorkspace_rendersTerminalSurface`, `test_SurfaceRouter_terminal_withNilWorkspace_rendersFallback`, `test_nstextBackend_usesTerminalFallback`
    - [x] **GREEN**: T-M2.5-006~T-M2.5-008 구현으로 모두 통과
    - [x] **REFACTOR**: `ProcessInfo.processInfo.environment` mocking 헬퍼 추출, Xcode project 등록
- **완료 기준**: 3건 PASS, regression 0

---

## MS-3: Command Palette 콜백 활성화 (P-2, P-3 해소)

### T-M2.5-010: `WorkspaceViewModel.tabModels` 사전 + register/unregister

- **목표**: pane id → TabBarViewModel 매핑 사전을 `@Observable` 프로퍼티로 추가
- **AC**: AC-3.1, AC-3.4 (기반)
- **파일**:
    - 수정: `app/Sources/ViewModels/WorkspaceViewModel.swift`
- **단계**:
    - [x] **RED**: `CommandPaletteSurfaceOpenTests.test_tabModelRegistration_roundTrip()` 작성
    - [x] **GREEN**: `public internal(set) var tabModels: [Int64: TabBarViewModel] = [:]`, `func registerTabModel(_ model: TabBarViewModel, forPane paneId: Int64)`, `func unregisterTabModel(forPane paneId: Int64)` 추가. `@MainActor` 격리 유지
    - [x] **REFACTOR**: weak reference vs strong 선택 — strong 으로 보유 (LeafPaneView.task 에서 생성 책임). pane close 시 명시 해제
- **완료 기준**: 테스트 PASS, `vm.registerTabModel(m, forPane: 100)` 후 `vm.tabModels[100] === m` 검증 가능

### T-M2.5-011: `LeafPaneView.task` 에서 tabModels 등록 + close 시 해제

- **목표**: LeafPaneView 생성 시 tabModels 에 등록, pane close 시 제거
- **AC**: AC-3.1 (전제)
- **파일**:
    - 수정: `app/Sources/Shell/Splits/PaneSplitView.swift` (LeafPaneView)
- **단계**:
    - [x] **RED**: `test_leafPaneView_registersTabModelOnLoad()` 작성 (LeafPaneView 를 ViewInspector 로 감싸 task 실행 후 `vm.tabModels[paneId]` 조회)
    - [x] **GREEN**:
        - `LeafPaneView` 에 `@Environment(WorkspaceViewModel.self) private var workspaceVM` 추가
        - 기존 `.task { ... }` 블록 내부에서 `await model.load()` 후 `workspaceVM.registerTabModel(model, forPane: paneId)` 호출
        - `PaneTreeModel.closePane` 경로 또는 LeafPaneView `.onDisappear` 에서 `workspaceVM.unregisterTabModel(forPane: paneId)` 호출
    - [x] **REFACTOR**: 생명주기 타이밍 주석 — "SwiftUI 뷰 파괴 시 .onDisappear 호출 보장되지 않을 수 있음 → PaneTreeModel.closePane 경로가 주 해제 지점"
- **완료 기준**: 테스트 PASS, pane 생성/삭제 시 tabModels 사전 일관성 유지

### T-M2.5-012: `RootSplitView.setupPaletteController` — `onSurfaceOpen` 실제 구현

- **목표**: no-op `onSurfaceOpen: { _ in ... TODO(MS-7) }` 를 실제 `TabBarViewModel.newTab(kind:)` 호출로 교체
- **AC**: AC-3.1, AC-3.2, AC-3.3, AC-3.4
- **파일**:
    - 수정: `app/Sources/Shell/RootSplitView.swift`
- **단계**:
    - [x] **RED**: `CommandPaletteSurfaceOpenTests` 에 5종 + nil 케이스 6개 테스트 작성 — `test_onSurfaceOpen_filetree_callsNewTab`, ..., `test_onSurfaceOpen_nilActivePane_noops`
    - [x] **GREEN**:
        - `onSurfaceOpen: { kind in ... }` 본문 작성:
            - `guard let paneId = vm.activePane.paneId else { log info; return }`
            - `guard let tabModel = vm.tabModels[paneId] else { log info; return }`
            - `_ = tabModel.newTab(kind: kind)`
        - 주석 `TODO(MS-7)` 제거, 신규 `@MX:NOTE: [AUTO] MS-3 완료 — workspaceVM.activePane 기반 ...` 추가
    - [x] **REFACTOR**: 로그 출력 표준화 (`os_log` 또는 `Logger`), weak self 캡처 검토 (klosure 가 PaletteController 에 강참조되므로 필요 시 `[weak vm]`)
- **완료 기준**: 6개 테스트 모두 PASS, `grep "TODO(MS-7)" app/Sources/Shell/RootSplitView.swift` 0건 (onSurfaceOpen 위치), 수동 검증 체크 2 통과

### T-M2.5-013: `RootSplitView.setupPaletteController` — `onPaneSplit` 실제 구현

- **목표**: no-op `onPaneSplit: { _ in ... TODO(MS-7) }` 를 실제 `PaneTreeModel.splitActive(_, direction:)` 호출로 교체
- **AC**: AC-4.1, AC-4.2, AC-4.3, AC-4.4
- **파일**:
    - 수정: `app/Sources/Shell/RootSplitView.swift`
- **단계**:
    - [x] **RED**: `CommandPalettePaneSplitTests` 4개 테스트 작성
    - [x] **GREEN**:
        - `onPaneSplit: { direction in ... }` 본문 작성:
            - `guard let paneId = vm.activePane.paneId, let model = vm.activePane.model else { log info; return }`
            - `let splitKind: SplitKind = (direction == .horizontal) ? .horizontal : .vertical`
            - `_ = model.splitActive(paneId, direction: splitKind)`
            - (반환된 새 pane id 는 PaneSplitContainerView 의 onChange 훅이 `activePaneId` 를 적절히 갱신하도록 유도 — T-M2.5-004 연계)
        - 주석 `TODO(MS-7)` 제거, 신규 `@MX:NOTE: [AUTO] MS-3 완료 — 키보드 단축키 경로와 동일 호출 시퀀스` 추가
    - [x] **REFACTOR**: PaneSplitDirection ↔ SplitKind 매핑을 private helper 로 추출 (`paneSplitKind(from:)`), weak self 캡처 처리
- **완료 기준**: 4개 테스트 PASS, `grep "TODO(MS-7)" app/Sources/Shell/RootSplitView.swift` 0건, 수동 검증 체크 3 통과

### T-M2.5-014: `CommandPaletteSurfaceOpenTests.swift` 완성 (≥6건)

- **목표**: AC-3.1 ~ AC-3.4 커버하는 테스트 스위트 완성
- **AC**: AC-3.1, AC-3.2, AC-3.3, AC-3.4
- **파일**:
    - 신규: `app/Tests/CommandPaletteSurfaceOpenTests.swift`
- **단계**:
    - [x] **RED**: T-M2.5-012 의 6개 테스트 통합. mock `TabBarViewModel` (또는 `MockRustCoreBridge` 기반) 주입 → `onSurfaceOpen(kind)` 호출 → `newTab` 호출 인자 검증
    - [x] **GREEN**: T-M2.5-010~T-M2.5-012 구현으로 모두 통과
    - [x] **REFACTOR**: 공통 setUp 에서 `WorkspaceViewModel` + `tabModels` 초기화 헬퍼 추출, Xcode project 등록
- **완료 기준**: 6건 PASS

### T-M2.5-015: `CommandPalettePaneSplitTests.swift` 완성 (≥4건)

- **목표**: AC-4.1 ~ AC-4.4 커버
- **AC**: AC-4.1, AC-4.2, AC-4.3, AC-4.4
- **파일**:
    - 신규: `app/Tests/CommandPalettePaneSplitTests.swift`
- **단계**:
    - [x] **RED**: 4개 테스트 작성 — horizontal, vertical, nil, 새 pane id 반영
    - [x] **GREEN**: T-M2.5-013 구현으로 모두 통과
    - [x] **REFACTOR**: mock `PaneTreeModel` 팩토리 (`makeMockTreeWithSingleLeaf()`) 추출, Xcode project 등록
- **완료 기준**: 4건 PASS

---

## 크로스-마일스톤

### T-M2.5-016: @MX 태그 적용 (ANCHOR / NOTE)

- **목표**: plan.md §6 MX 전략에 따라 신규 ANCHOR 2건, 갱신/신규 NOTE 6건 적용, 제거 NOTE 3건 제거
- **파일**: 다음 파일 각각 수정
    - `ActivePaneProvider.swift` (ANCHOR 2건)
    - `WorkspaceViewModel.swift` (NOTE 2건)
    - `TerminalSurface.swift` (GhosttyHost NOTE 갱신)
    - `RootSplitView.swift` (onSurfaceOpen, onPaneSplit NOTE 신규)
    - `PaneSplitView.swift` (SurfaceRouter NOTE 갱신, 구식 NOTE 3건 제거, TabBarViewModel ANCHOR 의 REASON 갱신)
- **단계**:
    - [x] **ANALYZE**: 대상 파일 현재 태그 상태 확인 (Grep `@MX:`)
    - [x] **APPLY**: plan.md §6.1, §6.2, §6.3, §6.4 기준 태그 추가/갱신/제거
    - [x] **VERIFY**: Grep 결과로 기대한 태그 수 + 위치 확인, `.claude/rules/moai/workflow/mx-tag-protocol.md` 준수 (REASON 필수 여부 확인)
- **완료 기준**: MX 태그 보고서 생성, mx.yaml per-file 한도 위반 없음, REASON 누락 0건

### T-M2.5-017: Regression sweep

- **목표**: 모든 기존 테스트 + 신규 테스트가 전부 통과함을 확인
- **단계**:
    - [x] `cargo test --workspace` 실행 → 233/233 PASS
    - [x] `cargo clippy --workspace -- -D warnings` PASS
    - [x] `cargo fmt --all -- --check` PASS
    - [x] `xcodebuild build-for-testing -scheme MoAIStudio -destination 'platform=macOS'` warning 증가 0
    - [x] `xcodebuild test-without-building -scheme MoAIStudio -destination 'platform=macOS'` 기존 106 + 신규 ≥15 전부 PASS
    - [x] `grep -r "TerminalSurfacePlaceholder" app/Sources/` 0건 확인
    - [x] `grep -r "TODO(MS-7)" app/Sources/Shell/RootSplitView.swift` 0건 확인
- **완료 기준**: 전 항목 PASS

### T-M2.5-018: 수동 UI 검증 (Metal Toolchain 환경)

- **목표**: acceptance.md §3.7 체크리스트 5건 모두 통과
- **단계**:
    - [x] 체크 1: 기본 워크스페이스 생성 → 실 Metal surface 렌더 확인 (AC-2.1)
    - [x] 체크 2: Cmd+K → 5종 Surface 명령 각각 실행 → 활성 pane 에 새 탭 생성 (AC-3.1 ~ AC-3.3)
    - [x] 체크 3: Cmd+K → 수평/수직 분할 각 실행 (AC-4.1, AC-4.2)
    - [x] 체크 4: 분할된 다른 pane 탭 → Cmd+K → Surface 명령 → 올바른 pane 에 탭 생성 (AC-1.4 + AC-3.1)
    - [x] 체크 5: 앱 재시작 후 레이아웃 100% 복원 (AC-G.4)
- **완료 기준**: 5건 모두 성공, 필요 시 screenshot 첨부하여 `polish-completion-report.md` (Sync 단계) 에 기록

---

## 의존성 다이어그램

```
T-M2.5-001 (struct)
  └── T-M2.5-002 (EnvironmentKey + extension)
      ├── T-M2.5-003 (WorkspaceViewModel.activePane)
      │     └── T-M2.5-004 (PaneSplitContainerView 동기화)
      │           └── T-M2.5-005 (ActivePaneProviderTests)
      │                 └── ┐
      │                     │
      └── T-M2.5-006 (PaneContainer activeWorkspace 주입) ──┐
            └── T-M2.5-007 (SurfaceRouter.terminal 교체)     │
                  └── T-M2.5-008 (GhosttyHost body 교체)     │
                        └── T-M2.5-009 (TerminalSurfaceEnvTests)
                              └── ┐                            │
                                  │                            │
                                  ▼                            │
                              T-M2.5-010 (tabModels 사전) ←────┘
                                  └── T-M2.5-011 (LeafPaneView 등록)
                                        ├── T-M2.5-012 (onSurfaceOpen)
                                        │     └── T-M2.5-014 (SurfaceOpenTests)
                                        └── T-M2.5-013 (onPaneSplit)
                                              └── T-M2.5-015 (PaneSplitTests)
                                                    └── T-M2.5-016 (MX 태그)
                                                          └── T-M2.5-017 (Regression)
                                                                └── T-M2.5-018 (수동 검증)
```

**병렬 가능 구간**:
- T-M2.5-006 ~ T-M2.5-009 (MS-2) 와 T-M2.5-010 (MS-3 시작) 는 T-M2.5-004 완료 후 병렬 가능
- T-M2.5-014 와 T-M2.5-015 는 T-M2.5-012/T-M2.5-013 완료 후 독립 실행 가능

총 태스크: **18건** (MS-1: 5, MS-2: 4, MS-3: 6, 크로스: 3)
