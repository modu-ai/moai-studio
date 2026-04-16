# SPEC-M2-002 Research — M2.5 Polish 깊이 있는 코드베이스 분석

---
spec_id: SPEC-M2-002
phase: research
created: 2026-04-16
author: MoAI (manager-spec)
---

## 목적

SPEC-M2-002 는 SPEC-M2-001 MS-7 완료 시점 (commit `21ae56c`) 에 잔존한 4개 placeholder (P-1 ~ P-4) 를 해소한다. 본 research 문서는 placeholder 해소에 필요한 파일 위치, 기존 패턴, 종속 타입, 제약을 정리한다. 실제 코드 수정은 본 문서 범위 밖이며, Run phase (MS-1 ~ MS-3) 에서 수행한다.

---

## 1. 현재 상태 분석 — 4개 Placeholder 의 원본 코드 위치

### 1.1 P-1: TerminalSurface GhosttyHost Placeholder

**파일**: `app/Sources/Surfaces/Terminal/TerminalSurface.swift`
**대상 라인**: 70-96 (`GhosttyHost` struct 본체)

```swift
// TerminalSurface.swift:70-96
private struct GhosttyHost: View {
    let workspace: WorkspaceSnapshot
    let onFailure: () -> Void

    var body: some View {
        ZStack {
            Color.black
            VStack(alignment: .leading, spacing: 8) {
                Text("ghostty-vt.xcframework loaded")
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(.green)
                Text("workspace: \(workspace.name)")
                    .font(.system(.body, design: .monospaced))
                    .foregroundStyle(.white)
                Text("(Ghostty Metal surface will render here — wiring in MS-6)")
                    // ← P-1 placeholder 문자열
```

**추가 placeholder**: `app/Sources/Shell/Splits/PaneSplitView.swift:270` (`SurfaceRouter.terminal` case) + `PaneSplitView.swift:309-334` (`TerminalSurfacePlaceholder` struct 전체)

```swift
// PaneSplitView.swift:267-286 (SurfaceRouter)
switch activeKind {
case .terminal, .none:
    TerminalSurfacePlaceholder(paneId: paneId)   // ← P-1: 실 TerminalSurface 로 교체 대상
case .filetree:
    FileTreeSurface(...)
case .markdown:
    MarkdownSurface(filePath: statePath)
...

// PaneSplitView.swift:313-334 (struct 정의)
struct TerminalSurfacePlaceholder: View {
    let paneId: Int64
    var body: some View {
        ZStack {
            Color.black
            VStack(alignment: .leading, spacing: 8) {
                Text("Terminal Surface")...
                Text("(MS-4 에서 실제 워크스페이스와 연결 예정)")
```

**현상**: 두 곳 모두 검정 배경 + 텍스트만 표시. 실제 GhosttyKit Metal surface 는 렌더되지 않는다.

**원인**: M2 MS-3 에서 `SurfaceProtocol` conform 이 완료되었으나 `@Environment` 로 `WorkspaceSnapshot` 을 주입하는 경로가 없어 `SurfaceRouter` 가 `TerminalSurface(workspace:)` 를 호출하지 못했다 (m2-completion-report.md §알려진 제한 사항 #2 — "MS-3 이후 TerminalSurfacePlaceholder 표시 중").

### 1.2 P-2: Command Palette `onSurfaceOpen` No-Op

**파일**: `app/Sources/Shell/RootSplitView.swift`
**대상 라인**: 79-82

```swift
// RootSplitView.swift:75-82 (setupPaletteController 내부)
let registry = CommandRegistry(
    onMoaiSlash: { text in
        let injector = SlashInjector(bridge: vm.bridge, workspaceVM: vm)
        injector.inject(text)
    },
    onSurfaceOpen: { _ in
        // @MX:NOTE: [AUTO] Surface 열기 — MS-7 에서 ActivePaneProvider @Environment 로 교체.
        // TODO(MS-7): ActivePaneProvider 통해 TabBarViewModel.newTab(kind:) 호출
    },                                              // ← P-2: no-op 교체 대상
```

**현상**: Command Palette 에서 "File Tree 열기" / "Markdown 열기" / "Image 열기" / "Browser 열기" / "Terminal 열기" 선택 시 클로저가 호출되지만 본문이 비어 있어 아무 동작도 일어나지 않는다.

**원인**: 활성 pane 의 `TabBarViewModel` 에 접근할 수단이 없다. `RootSplitView.setupPaletteController` 는 `onAppear` 시점에 1회만 실행되고, 이후 팔레트가 열릴 때 활성 pane 정보는 `PaneSplitContainerView.activePaneId` @State 에 있지만 `RootSplitView` 에서는 이를 읽을 수 없다.

### 1.3 P-3: Command Palette `onPaneSplit` No-Op

**파일**: `app/Sources/Shell/RootSplitView.swift`
**대상 라인**: 86-89

```swift
// RootSplitView.swift:83-90
    onWorkspaceCreate: {
        showSheet()
    },
    onPaneSplit: { _ in
        // @MX:NOTE: [AUTO] Pane 분할 — MS-7 에서 ActivePaneProvider @Environment 로 교체.
        // TODO(MS-7): PaneTreeModel.splitActive(activePaneId, direction:) 호출
    }                                               // ← P-3: no-op 교체 대상
)
```

**현상**: Command Palette 에서 "Pane 수평 분할" / "Pane 수직 분할" 선택 시 no-op.

**원인**: P-2 와 동일. 활성 `PaneTreeModel` + `activePaneId` 참조 수단 부재.

### 1.4 P-4: ActivePaneProvider `@Environment` 미존재

**현상**: 프로젝트 내 `EnvironmentKey` / `EnvironmentValues` extension 이 **0건** (Grep 확인 결과). 모든 `@Environment(...)` 사용은 `@Observable` 매크로 기반 `@Environment(WorkspaceViewModel.self)` / `@Environment(WindowStateStore.self)` 패턴.

**원인**: M2 범위에서 pane 활성 상태는 `PaneSplitContainerView.activePaneId: Int64?` @State 로 **지역 관리**되었다. 상위 (RootSplitView) 또는 하위 (SurfaceRouter) 에서 이 값을 공유할 메커니즘이 설계되지 않았다. m2-completion-report.md §M3 권장 다음 액션 에 "ActivePaneProvider @Environment 구현 — Command Palette onSurfaceOpen 콜백 활성화" 가 우선순위 Medium 으로 기재됨.

---

## 2. SwiftUI EnvironmentKey 패턴 조사

### 2.1 현재 프로젝트의 Environment 사용 패턴

프로젝트 내 **모든** `@Environment` 사용처 (Grep 결과 총 10건):

| 파일 | 라인 | 사용 패턴 |
|------|------|-----------|
| `Shell/MainWindow.swift` | 12-13 | `@Environment(WorkspaceViewModel.self)`, `@Environment(WindowStateStore.self)` |
| `Shell/RootSplitView.swift` | 15-16 | 동일 |
| `Shell/Sidebar/NewWorkspaceSheet.swift` | 9 | `@Environment(WorkspaceViewModel.self)` |
| `Shell/Sidebar/WorkspaceListView.swift` | 9 | 동일 |
| `Shell/Content/EmptyState.swift` | 9 | 동일 |
| `Shell/Sidebar/ContextMenu.swift` | 9 | 동일 |
| `Shell/Content/PaneContainer.swift` | 15 | `@Environment(WorkspaceViewModel.self) private var workspaceVM` |
| `Shell/Content/ContentArea.swift` | 9 | 동일 |

**핵심 관찰**: 10건 모두 `@Observable` 매크로 클래스 (WorkspaceViewModel, WindowStateStore) 를 **타입 기반**으로 주입하는 Swift 6 패턴이다. 커스텀 `EnvironmentKey` 기반 값 주입 사례는 **0건**.

### 2.2 Swift 6 @Observable 주입 vs EnvironmentKey 비교

| 측면 | `@Environment(Type.self)` | `EnvironmentKey` + `EnvironmentValues` extension |
|------|--------------------------|------------------------------------------------|
| 타입 요구 | `@Observable` 클래스 (reference) | 어떤 타입이든 (struct value 가능) |
| 기본값 | 없음 (nil 불가, 명시 주입 필요) | `EnvironmentKey.defaultValue` 정의 필요 |
| 사용처 | 전역 싱글톤 (ViewModel) | 스코프 한정 공유값 |
| 주입 문법 | `.environment(vm)` | `.environment(\.keyPath, value)` |
| `@MainActor` 격리 | 지원 | 지원 (`EnvironmentValues` extension 에 `@MainActor` 부착 가능) |

**선택 근거**: `ActivePaneContext` 는 **경량 value struct** 이며 "pane 이 없는 상태" 를 표현해야 하므로 (nil 필드), **`EnvironmentKey` 패턴이 적합**하다. `WorkspaceSnapshot?` 도 동일 이유 (nil 의미 있음).

### 2.3 표준 EnvironmentKey 구현 골격 (참조)

```swift
// 일반적 구조 (프로젝트 내 예시는 없음 — Apple 공식 SwiftUI 문서 패턴)
struct ActivePaneProviderKey: EnvironmentKey {
    static let defaultValue: ActivePaneContext = .empty
}

extension EnvironmentValues {
    var activePane: ActivePaneContext {
        get { self[ActivePaneProviderKey.self] }
        set { self[ActivePaneProviderKey.self] = newValue }
    }
}
```

**본 SPEC 의 적용 범위**: 두 Key 가 필요하다 — `ActivePaneProviderKey` (P-4), `WorkspaceEnvironmentKey` (P-1 의 `WorkspaceSnapshot?` 주입). 두 Key 는 동일 파일 `ActivePaneProvider.swift` 에 병합한다 (SPEC-M2-002 spec.md §6 의존성 조항).

---

## 3. GhosttyHost 통합 지점 조사

### 3.1 WorkspaceSnapshot 타입 정의

**파일**: `app/Sources/Bridge/RustCore+Generated.swift:13-23`

```swift
public struct WorkspaceSnapshot: Identifiable, Hashable, Sendable {
    public let id: String              // UUID 문자열
    public let name: String
    public let status: WorkspaceStatus // Created/Starting/Running/Paused/Error/Deleted

    public init(id: String, name: String, status: WorkspaceStatus) {
        self.id = id
        self.name = name
        self.status = status
    }
}
```

**관찰**:
- `Sendable` 확정 — `@Environment` 주입 가능.
- `id` 는 Swift 측 UUID 문자열, `getWorkspaceDbId(workspaceUuid:)` 로 Int64 DB id 변환 필요.
- `@MainActor` 격리 불필요 (value type, Sendable).

### 3.2 WorkspaceSnapshot 조회 경로

**메서드**: `WorkspaceViewModel.swift:57-59`

```swift
public func workspace(id: String) -> WorkspaceSnapshot? {
    workspaces.first(where: { $0.id == id })
}
```

**호출 예시**: `ContentArea.swift:14` — `if let id = selected, let workspace = viewModel.workspace(id: id) { ... }`

**MS-2 주입 지점**: `PaneContainer.swift:52-66` (`loadModelIfNeeded`). 현재 `bridge.getWorkspaceDbId(workspaceUuid:)` 로 dbId 를 얻고 있으므로, 동일 시점에 `workspaceVM.workspace(id: workspaceUuid)` 를 호출해 `WorkspaceSnapshot` 을 확보하고 `.environment(\.activeWorkspace, snapshot)` 을 하위 뷰에 주입할 수 있다.

### 3.3 TerminalSurface 시그니처

**파일**: `app/Sources/Surfaces/Terminal/TerminalSurface.swift:29-47`

```swift
struct TerminalSurface: View {
    let workspace: WorkspaceSnapshot   // ← non-optional 이 요구됨
    @State private var loadFailed: Bool = false

    var body: some View {
        Group {
            if loadFailed || TerminalBackend.current == .nstext {
                TerminalFallback(workspace: workspace) { loadFailed = false }
            } else {
                GhosttyHost(workspace: workspace, onFailure: { loadFailed = true })
            }
        }
        .background(Color.black)
    }
}
```

**제약**: `workspace: WorkspaceSnapshot` 는 **non-optional**. `@Environment(\.activeWorkspace)` 가 `WorkspaceSnapshot?` 을 반환하므로 `SurfaceRouter` 에서 nil 처리 분기가 필요 (fallback 안내 뷰 표시).

### 3.4 GhosttyHost 실연결 필요 작업

현재 `GhosttyHost.body` (TerminalSurface.swift:74-95) 는 플레이스홀더 텍스트만 표시한다. 실연결 요구사항:
1. `ghostty-vt.xcframework` 초기화 호출 (기존 의존성 연결은 M1 에서 완료)
2. 실패 시 `onFailure()` 콜백 (기존 유지)
3. Metal surface NSViewRepresentable 래핑 — `NSHostingView`/`NSView` 레벨 통합

본 SPEC 은 GhosttyKit 의 **래핑 완성** 까지 포함한다. Metal 실측 (C-4) 은 Exclusions §7 에 따라 별도 스프린트.

---

## 4. Command Palette 콜백 현황

### 4.1 CommandRegistry 4종 콜백 동작/미동작

**파일**: `app/Sources/Shell/CommandPalette/CommandRegistry.swift:94-97, 101-116`

```swift
private let onMoaiSlash: @MainActor @Sendable (String) -> Void       // ← 동작 (SlashInjector)
private let onSurfaceOpen: @MainActor @Sendable (SurfaceKind) -> Void // ← no-op (P-2)
private let onWorkspaceCreate: @MainActor @Sendable () -> Void        // ← 동작 (NewWorkspaceSheet)
private let onPaneSplit: @MainActor @Sendable (PaneSplitDirection) -> Void // ← no-op (P-3)
```

**동작 콜백 (`onMoaiSlash`)**:
- RootSplitView.swift:75-78 에서 `SlashInjector(bridge: vm.bridge, workspaceVM: vm)` 로 직접 주입.
- FFI 경로로 Claude subprocess 에 도달. E2E 검증 완료 (SPEC-M2-001 AC-3.3).

**동작 콜백 (`onWorkspaceCreate`)**:
- RootSplitView.swift:83-85. `showNewWorkspaceSheet = true` 로 sheet binding 트리거.
- `@State private var showNewWorkspaceSheet` 에 의존.

**무동작 콜백 (`onSurfaceOpen`, `onPaneSplit`)**:
- 둘 다 `{ _ in }` 본문만 존재. 이유: 활성 pane 의 `TabBarViewModel` / `PaneTreeModel` 참조 수단 부재.

### 4.2 Surface 카테고리 명령 5종 (P-2 대상)

CommandRegistry.swift:165-187 `registerSurfaceCommands` 에 정의:

| id | title | SurfaceKind | M2 구현 |
|----|-------|-------------|---------|
| `surface.filetree` | "File Tree 열기" | `.filetree` | ✅ (MS-4) |
| `surface.markdown` | "Markdown 파일 열기" | `.markdown` | ✅ (MS-5) |
| `surface.image` | "Image 파일 열기" | `.image` | ✅ (MS-5) |
| `surface.browser` | "Browser 열기" | `.browser` | ✅ (MS-5) |
| `surface.terminal` | "Terminal 열기" | `.terminal` | P-1 해소 후 완성 |

본 SPEC 에서 해당 5종 모두 동작해야 한다 (SPEC-M2-002 RG-M2.5-3).

### 4.3 Pane 분할 명령 2종 (P-3 대상)

CommandRegistry.swift:207-223 `registerPaneCommands` 에 정의:

| id | title | direction | 기존 단축키 경로 |
|----|-------|-----------|-----------------|
| `pane.split.horizontal` | "Pane 수평 분할" | `.horizontal` | Cmd+\ |
| `pane.split.vertical` | "Pane 수직 분할" | `.vertical` | Cmd+Shift+\ |

기존 단축키 경로 (`PaneSplitContainerView.swift:378-389`) 는 `model.splitActive(paneId, direction:)` 를 **정상 호출** 중. 본 SPEC 은 팔레트 경로가 **동일한 호출 시퀀스** 를 수행하도록 구현한다 (SPEC-M2-002 AC-4).

---

## 5. Pane Tree 활성 Pane 추적 현황

### 5.1 현재 activePaneId 관리 구조

**파일**: `app/Sources/Shell/Splits/PaneSplitView.swift:363-364`

```swift
public struct PaneSplitContainerView: View {
    @Bindable var model: PaneTreeModel
    @State private var activePaneId: Int64?    // ← 활성 pane 저장소
```

**활성 pane 전환 경로**:
1. 단축키 (Cmd+\, Cmd+Shift+\, Cmd+Shift+W): PaneSplitView.swift:378-397. 분할/close 후 `activePaneId = newId` 할당.
2. 탭 제스처: PaneSplitView.swift:219-220 (`LeafPaneView` 내부) `.onTapGesture { activePaneId = paneId }`.
3. 최초 로드: PaneSplitView.swift:372-377 `onAppear` 에서 `activePaneId = model.rootId`.

### 5.2 PaneTreeModel 과 활성 pane 의 분리

현재 `PaneTreeModel` (@Observable) 은 `nodes` 맵과 `rootId` 만 보유한다. **activePaneId 는 PaneTreeModel 외부 (PaneSplitContainerView) 에 있다**. 이 설계의 결과:
- RootSplitView 가 activePaneId 를 읽으려면 PaneSplitContainerView 내부에서 `.environment(\.activePane, ...)` 를 **상향 전파**해야 함 (자연스럽지 않음).
- 또는 PaneSplitContainerView 의 `@State` 를 `PaneTreeModel` 의 `@Observable` 프로퍼티로 승격 → `workspaceVM.paneTree(for: id)?.activePaneId` 형태로 접근 가능.

**본 SPEC 의 선택**: **두 번째 접근 배제**. `PaneTreeModel` 을 건드리면 기존 PaneTreeModelTests 41건에 영향 가능성. 대신 **PaneSplitContainerView 내부에서 `.environment(\.activePane, ActivePaneContext(paneId: activePaneId, model: model, workspace: env.activeWorkspace))` 를 **자식 뷰 트리에 주입** 하고, Command Palette 는 PaneSplitContainerView 의 자식 뷰 트리 밖에 있으므로 **별도 경로** 로 환경값을 공유한다.

### 5.3 PaneContainer → PaneSplitContainerView 계층

```
RootSplitView (CommandPaletteView 오버레이 포함)
└── NavigationSplitView.detail
    └── PaneContainer (workspaceId 선택)
        └── PaneSplitContainerView (PaneTreeModel 보유, activePaneId @State)
            └── PaneSplitView (NSViewRepresentable)
                └── LeafPaneView (TabBarViewModel 보유)
                    └── SurfaceRouter
                        ├── TerminalSurfacePlaceholder ← P-1
                        ├── FileTreeSurface
                        ├── MarkdownSurface
                        ├── ImageSurface
                        └── BrowserSurface

RootSplitView
└── CommandPaletteView.overlay ← 여기서 activePane 접근 필요 (P-2/P-3)
```

### 5.4 CommandPalette 환경 공유 전략

`CommandPaletteView` 는 `RootSplitView` 의 ZStack overlay 로 배치되며, `NavigationSplitView.detail` 의 자식 트리 **바깥** 에 있다. 따라서 `PaneSplitContainerView` 가 하위 뷰 트리에 주입한 `.environment(\.activePane, ...)` 값은 `CommandPaletteView` 에서 **읽히지 않는다**.

**해결책 (본 SPEC 의 접근)**:
- `CommandRegistry` 의 콜백은 `@Observable` 참조를 **클로저 캡처** 로 받는다 (`onMoaiSlash` 와 동일 패턴). 활성 pane 상태는 별도 `@Observable` Holder 클래스 (`ActivePaneHolder` 또는 기존 WorkspaceViewModel 확장) 에 저장하고, `RootSplitView.setupPaletteController` 에서 이 Holder 를 참조하는 클로저를 주입한다.
- **대안 (더 단순)**: `ActivePaneContext` 를 `WorkspaceViewModel` 의 `@Observable` 프로퍼티로 저장. `PaneSplitContainerView` 는 activePaneId 변경 시 `workspaceVM.activePane = ActivePaneContext(...)` 로 업데이트. `RootSplitView` 는 이미 `@Environment(WorkspaceViewModel.self)` 을 보유하므로 즉시 접근 가능.

**plan.md §3 MS-1** 에서 최종 구조를 결정한다. 현재 research 단계 판단: **대안 (WorkspaceViewModel.activePane 프로퍼티 확장) 을 채택** — `@Environment` 기반 "하향 전파" 는 여전히 `LeafPaneView` 내부 뷰 (`FileTreeSurface` 등) 에 필요하지만, Command Palette 경로는 `@Observable` 참조 기반이 간단하다. SPEC-M2-002 §10 용어 정의에서 `ActivePaneContext` 는 "환경값" 으로 규정되어 있으므로, 양쪽 경로 (Environment + Observable reference) 를 모두 구현하되 같은 struct 를 공유한다.

---

## 6. 참조 구현

### 6.1 기존 프로젝트 내 유사 Observable 주입 패턴

`WindowStateStore` (Shell/WindowStateStore.swift):
- `@Observable` 클래스 + `@MainActor` 격리
- `MoAIStudioApp.swift` 에서 `.environment(WindowStateStore())` 로 앱 전역 주입
- 필요 시점마다 `@Environment(WindowStateStore.self)` 로 읽기

**적용**: `ActivePaneHolder` 를 별도 `@Observable` 로 만들어 동일 경로로 주입하거나, `WorkspaceViewModel` 에 `activePane: ActivePaneContext` `@Observable` 프로퍼티를 추가.

### 6.2 Apple 공식 EnvironmentKey 예시

Apple 공식 SwiftUI 문서의 `EnvironmentKey` 최소 구현:
- `defaultValue` 정의 (non-optional)
- `EnvironmentValues` extension 에 get/set 노출
- 사용 지점에서 `.environment(\.keyPath, value)` 주입

본 SPEC 은 두 Key 를 동일 파일에 병합:
- `ActivePaneProviderKey` (defaultValue: `ActivePaneContext.empty`)
- `WorkspaceEnvironmentKey` (defaultValue: `nil`)

### 6.3 swift.md 가이드 적용

`.claude/rules/moai/languages/swift.md` 기준:
- Swift 6.0+ `@Observable` 매크로 활용 (WorkspaceViewModel 확장 시)
- `@MainActor` 격리 유지 (EnvironmentValues extension 및 Holder 모두)
- `Sendable` 준수 (`ActivePaneContext` 는 `PaneTreeModel` 참조 포함 → `@MainActor`-only 로 한정, `Sendable` 는 요구하지 않음)

---

## 7. 위험 / 제약

### 7.1 FFI 변경 금지

`swift-bridge` FFI 표면은 **변경하지 않는다**. 본 SPEC 에서 사용하는 FFI 는 모두 기존 메서드:
- `bridge.getWorkspaceDbId(workspaceUuid:)` (RustCore+Generated.swift:224)
- `PaneTreeModel.splitActive(_, direction:)` (기존)
- `TabBarViewModel.newTab(kind:statePath:)` (기존)

**결과**: Rust 측 변경 0, `cargo test --workspace` 233/233 regression 0 보장.

### 7.2 기존 테스트 Regression 0

| 테스트 | 건수 | 변경 영향 |
|--------|------|-----------|
| Rust `cargo test --workspace` | 233 | 0 (Rust 변경 없음) |
| Swift `PaneTreeModelTests` | 10 | 0 (PaneTreeModel API 변경 없음) |
| Swift `TabBarViewModelTests` | 21 (MS-3) | 0 (`newTab` 시그니처 유지) |
| Swift `CommandPaletteTests` 등 | ~70 (MS-5/MS-6) | 0 (CommandRegistry API 유지) |
| **합계 기존** | **339** | **regression 0 목표** |

신규 테스트: AC-G.2 에 따라 **최소 15건 추가** (ActivePaneProvider 5 + SurfaceOpen 6 + PaneSplit 4 + Terminal env 3 = 18건 예상).

### 7.3 Metal 60fps 유지

GhosttyHost 실연결은 C-4 실측 전제 조건이다. 본 SPEC 의 래핑 로직은 기존 `GhosttyMetalBenchmarkTests.swift` 하네스 (SPEC-M2-001 MS-7 T-080 완료) 를 통과해야 하며, 60fps 실측은 Exclusions §7 에 따라 별도 스프린트로 이월.

### 7.4 `@MainActor` 격리

- `ActivePaneContext` 의 `model: PaneTreeModel?` 접근은 모두 `@MainActor` 컨텍스트.
- `EnvironmentValues.activePane` getter/setter 는 `@MainActor` 불필요 (struct value). 단 `PaneTreeModel` 메서드 호출은 `@MainActor` 필요.
- `WorkspaceEnvironmentKey` 역시 `WorkspaceSnapshot?` (Sendable value) — 격리 불필요.

### 7.5 Swift 빌드 시간

AC-G.3 요구사항에 따라 "Swift 빌드 warning 증가 0". `@Observable` 프로퍼티 추가 시 `@MainActor` 전파 경고 발생 가능 — Holder 클래스 전체에 `@MainActor` 부착으로 해결.

### 7.6 레이아웃 복원 Regression 방지

SPEC-M2-001 RG-M2-1/RG-M2-2 의 State-Driven 조항 (앱 재시작 후 레이아웃 100% 복원) 은 DB 영속 경로 (`PaneTreeModel.load` + `TabBarViewModel.load`) 에 의존한다. 본 SPEC 은 **읽기 경로** 만 추가하므로 영속 동작에 영향 없음. AC-G.4 는 기존 테스트로 보장.

---

## 8. 확인된 파일 목록 (수정/신규 대상)

### 신규 (MS-1)
- `app/Sources/Shell/Splits/ActivePaneProvider.swift` — `ActivePaneContext`, `ActivePaneProviderKey`, `WorkspaceEnvironmentKey`, `EnvironmentValues` extension
- `app/Tests/ActivePaneProviderTests.swift` — 환경값 기본/주입/중첩 테스트 (최소 5건)
- `app/Tests/TerminalSurfaceEnvironmentTests.swift` — `@Environment(\.activeWorkspace)` 주입 테스트 (최소 3건, MS-2)
- `app/Tests/CommandPaletteSurfaceOpenTests.swift` — 5종 SurfaceKind + nil 케이스 (최소 6건, MS-3)
- `app/Tests/CommandPalettePaneSplitTests.swift` — 2종 방향 + nil + 반환값 (최소 4건, MS-3)

### 수정 (MS-2)
- `app/Sources/Surfaces/Terminal/TerminalSurface.swift` — `GhosttyHost` 본문을 실제 GhosttyKit Metal surface 로 교체 (placeholder 문자열 제거)
- `app/Sources/Shell/Splits/PaneSplitView.swift` — `SurfaceRouter.terminal` 케이스를 `TerminalSurface(workspace:)` 로 교체 + `TerminalSurfacePlaceholder` struct 제거
- `app/Sources/Shell/Content/PaneContainer.swift` — `.environment(\.activeWorkspace, snapshot)` 주입

### 수정 (MS-1 + MS-3)
- `app/Sources/ViewModels/WorkspaceViewModel.swift` — `activePane: ActivePaneContext` `@Observable` 프로퍼티 추가 (MS-1)
- `app/Sources/Shell/Splits/PaneSplitView.swift` `PaneSplitContainerView` — `activePaneId` 변경 시 `workspaceVM.activePane` 업데이트 + `.environment(\.activePane, ...)` 주입 (MS-1)
- `app/Sources/Shell/RootSplitView.swift` — `setupPaletteController` 의 `onSurfaceOpen` / `onPaneSplit` 클로저를 `workspaceVM.activePane` 소비로 교체 (MS-3)

---

## 9. 개방 질문 (Plan 단계에서 확정)

1. `ActivePaneContext` 를 `WorkspaceViewModel.activePane` 프로퍼티로 저장할지, 별도 `@Observable ActivePaneHolder` 클래스로 분리할지 — `plan.md §2 Technology stack` 에서 확정. **잠정 결정**: WorkspaceViewModel 확장 (기존 DI 경로 재사용).
2. `TabBarViewModel` 참조를 `ActivePaneContext` 에 포함할지 — 현재 `LeafPaneView` 내부에 `@State private var tabModel: TabBarViewModel?` 로 지역 보관 중. `plan.md` 에서 활성 pane 의 `tabModel` 접근 경로 확정.
3. `WorkspaceSnapshot` 의 `activeWorkspace` 환경값 주입 시점 — `PaneContainer.loadModelIfNeeded` 에서 즉시 주입 vs `PaneSplitContainerView.onAppear` — `plan.md §3 MS-2` 에서 확정.

---

**신뢰도**: HIGH (4개 placeholder 원본 위치 전수 확인, 기존 @Environment 패턴 전체 Grep 완료, 시그니처/타입 확정). 의존 타입 (WorkspaceSnapshot, PaneTreeModel, TabBarViewModel, CommandRegistry) 구조 검증 완료.
