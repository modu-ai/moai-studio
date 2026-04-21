//
//  PaneSplitView.swift
//  NSSplitView binary tree 래퍼 (SPEC-M2-001 MS-2 T-038, T-040, T-041; MS-3 T-048).
//
//  @MX:ANCHOR: [AUTO] PaneTreeModel 을 NSSplitView 계층으로 렌더링하는 유일 진입점 (fan_in>=3)
//  @MX:REASON: [AUTO] PaneContainer, RootSplitView(리팩터링), E2E 테스트 세 경로에서 사용
//
//  NSSplitView orientation 주의:
//  - .vertical   → 분할선이 수직 → 좌우 배치 → SplitKind.horizontal 에 매핑
//  - .horizontal → 분할선이 수평 → 상하 배치 → SplitKind.vertical 에 매핑

import AppKit
import SwiftUI

// MARK: - PaneSplitView

/// pane binary tree 를 재귀적으로 NSSplitView 계층으로 렌더링하는 SwiftUI 뷰.
public struct PaneSplitView: NSViewRepresentable {
    /// 렌더링할 pane 트리 모델
    @Bindable var model: PaneTreeModel
    /// 현재 활성 pane id
    @Binding var activePaneId: Int64?

    public init(model: PaneTreeModel, activePaneId: Binding<Int64?>) {
        self.model = model
        self._activePaneId = activePaneId
    }

    public func makeNSView(context: Context) -> NSView {
        makeView(for: model.rootId, context: context)
    }

    public func updateNSView(_ nsView: NSView, context: Context) {
        // 트리 변경 시 부모 뷰가 교체하므로 여기서는 no-op
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(model: model, activePaneId: $activePaneId)
    }

    // MARK: - 재귀 뷰 생성

    private func makeView(for paneId: Int64?, context: Context) -> NSView {
        guard let paneId, let node = model.nodes[paneId] else {
            return makeEmptyView()
        }

        switch node.split {
        case .leaf:
            return makeLeafView(paneId: paneId, context: context)
        case .horizontal, .vertical:
            return makeSplitView(node: node, context: context)
        }
    }

    private func makeEmptyView() -> NSView {
        let view = NSView()
        view.wantsLayer = true
        view.layer?.backgroundColor = NSColor.windowBackgroundColor.cgColor
        return view
    }

    /// leaf 노드: TabBarView + SurfaceRouter 를 포함하는 LeafPaneView (MS-3 T-048)
    private func makeLeafView(paneId: Int64, context: Context) -> NSView {
        let leafView = LeafPaneView(
            paneId: paneId,
            bridge: model.bridge,
            activePaneId: $activePaneId
        )
        let hosting = NSHostingView(rootView: leafView)
        hosting.translatesAutoresizingMaskIntoConstraints = false
        return hosting
    }

    /// 비-leaf 노드: NSSplitView 로 두 자식을 배치
    private func makeSplitView(node: PaneNode, context: Context) -> NSSplitView {
        let splitView = MoAISplitView()
        // @MX:NOTE: [AUTO] orientation 과 split direction 의 반전 관계:
        //           horizontal split (좌우) → orientation = .vertical (수직 분할선)
        //           vertical split (상하)   → orientation = .horizontal (수평 분할선)
        splitView.isVertical = (node.split == .horizontal)
        splitView.dividerStyle = .thin
        splitView.delegate = context.coordinator

        let children = model.children(of: node.id)
        for (index, child) in children.enumerated() {
            let childView = makeView(for: child.id, context: context)
            let item = NSSplitViewItem(viewController: NSViewController())
            item.viewController.view = childView
            // @MX:NOTE: [AUTO] 최소 pane 크기 200pt — AC-1.3 요구사항
            item.minimumThickness = 200
            splitView.addArrangedSubview(childView)

            // 첫 번째 자식에 holding priority 설정 (리사이즈 스냅 방지)
            if index == 0 {
                splitView.setHoldingPriority(.defaultHigh - 1, forSubviewAt: 0)
            }
        }

        // 초기 ratio 적용
        context.coordinator.pendingRatios[node.id] = node.ratio
        context.coordinator.registerSplitView(splitView, paneId: node.id)

        return splitView
    }

    // MARK: - Coordinator

    public final class Coordinator: NSObject, NSSplitViewDelegate {
        private let model: PaneTreeModel
        private var activePaneId: Binding<Int64?>
        var pendingRatios: [Int64: Double] = [:]
        private var splitViewPaneIds: [ObjectIdentifier: Int64] = [:]

        init(model: PaneTreeModel, activePaneId: Binding<Int64?>) {
            self.model = model
            self.activePaneId = activePaneId
        }

        func registerSplitView(_ splitView: NSSplitView, paneId: Int64) {
            splitViewPaneIds[ObjectIdentifier(splitView)] = paneId
        }

        // 최소 pane 크기 200pt 강제
        public func splitView(
            _ splitView: NSSplitView,
            constrainMinCoordinate proposedMinimumPosition: CGFloat,
            ofSubviewAt dividerIndex: Int
        ) -> CGFloat {
            // @MX:NOTE: [AUTO] 200pt 최소 크기 — SPEC AC-1.3 요구사항
            return max(proposedMinimumPosition, 200)
        }

        public func splitView(
            _ splitView: NSSplitView,
            constrainMaxCoordinate proposedMaximumPosition: CGFloat,
            ofSubviewAt dividerIndex: Int
        ) -> CGFloat {
            let total = splitView.isVertical ? splitView.bounds.width : splitView.bounds.height
            return min(proposedMaximumPosition, total - 200)
        }

        // 드래그 후 ratio 영속
        public func splitViewDidResizeSubviews(_ notification: Notification) {
            guard let splitView = notification.object as? NSSplitView,
                  let paneId = splitViewPaneIds[ObjectIdentifier(splitView)],
                  splitView.subviews.count == 2
            else { return }

            let total = splitView.isVertical ? splitView.bounds.width : splitView.bounds.height
            guard total > 0 else { return }

            let firstSize = splitView.isVertical
                ? splitView.subviews[0].bounds.width
                : splitView.subviews[0].bounds.height
            let ratio = Double(firstSize / total)

            Task { @MainActor in
                self.model.updateRatio(paneId, ratio: ratio)
            }
        }
    }
}

// MARK: - MoAISplitView

/// NSSplitView 서브클래스 — 커스텀 divider 색상 제공.
private final class MoAISplitView: NSSplitView {
    override var dividerColor: NSColor {
        NSColor.separatorColor
    }
}

// MARK: - LeafPaneView (MS-3 / MS-4)

/// leaf pane 에 TabBarView + SurfaceRouter 를 렌더링하는 뷰 (T-048, T-054).
struct LeafPaneView: View {
    let paneId: Int64
    let bridge: RustCoreBridging
    @Binding var activePaneId: Int64?
    @Environment(WorkspaceViewModel.self) private var workspaceVM

    @State private var tabModel: TabBarViewModel?

    var isActive: Bool { activePaneId == paneId }

    var body: some View {
        VStack(spacing: 0) {
            if let model = tabModel {
                // 탭 바
                TabBarView(
                    items: model.tabs,
                    activeId: Binding(
                        get: { model.activeTabId },
                        set: { model.activeTabId = $0 }
                    ),
                    onNewTab: { _ = model.newTab() },
                    onCloseTab: { _ = model.closeTab($0) },
                    onReorder: { from, to in model.reorder(from: from, to: to) },
                    onSelect: { model.selectTab($0) }
                )
                Divider()
                // Surface 콘텐츠
                SurfaceRouter(
                    activeKind: model.activeTabKind(),
                    paneId: paneId,
                    bridge: bridge,
                    onFileOpen: { path in
                        let kind = SurfaceRouter.kindForExtension(path)
                        _ = model.newTab(kind: kind, statePath: path)
                    },
                    statePath: model.activeStatePath()
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .contentShape(Rectangle())
                .onTapGesture { activePaneId = paneId }
            } else {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .task {
                        let model = TabBarViewModel(paneId: paneId, bridge: bridge)
                        await model.load()
                        tabModel = model
                        // MS-3 T-M2.5-011: workspaceVM 에 등록하여 Command Palette 에서 접근 가능하게 함
                        workspaceVM.registerTabModel(model, forPane: paneId)
                    }
            }
        }
        .overlay(
            RoundedRectangle(cornerRadius: 2)
                .strokeBorder(
                    isActive ? Color.accentColor.opacity(0.6) : Color.clear,
                    lineWidth: 1
                )
        )
        .onDisappear {
            // pane close 시 tabModels 해제 — SwiftUI 뷰 파괴 시 호출됨
            // 주: PaneTreeModel.closePane 경로에서도 추가 해제 가능하지만
            //     onDisappear 가 주 해제 지점으로 충분함
            workspaceVM.unregisterTabModel(forPane: paneId)
        }
    }
}

// MARK: - SurfaceRouter

/// 활성 탭의 SurfaceKind 에 따라 해당 Surface 뷰를 선택하는 라우터.
///
// @MX:NOTE: [AUTO] MS-2 — @Environment(\.activeWorkspace) 주입으로 TerminalSurface(workspace:) 직접 렌더.
//            workspace nil 시 WorkspaceUnavailablePlaceholder 표시.
// @MX:NOTE: [AUTO] T-054: 파일 확장자 → SurfaceKind 매핑.
//            .md/.markdown → .markdown, image 확장자 → .image, 나머지 → .terminal
// @MX:NOTE: [AUTO] MS-5: statePath 를 state_json 에서 추출.
//            {"path":"/abs/path"} 형식 또는 단순 경로 문자열 양쪽 지원.
struct SurfaceRouter: View {
    let activeKind: SurfaceKind?
    let paneId: Int64
    let bridge: RustCoreBridging
    let onFileOpen: (String) -> Void

    // @MX:NOTE: [AUTO] statePath 는 TabBarViewModel.activeTab?.statePath 에서 전달.
    var statePath: String = ""

    // MS-2: @Environment(\.activeWorkspace) 로 workspace 주입
    @Environment(\.activeWorkspace) private var activeWorkspace: WorkspaceSnapshot?

    // MS-4/5 에서는 홈 디렉토리 폴백
    private func resolveWorkspacePath() -> String {
        activeWorkspace?.name.isEmpty == false
            ? FileManager.default.homeDirectoryForCurrentUser.path
            : FileManager.default.homeDirectoryForCurrentUser.path
    }

    var body: some View {
        switch activeKind {
        case .terminal, .none:
            // MS-2: 실 TerminalSurface(workspace:) 렌더 (MS-2 T-M2.5-007)
            if let ws = activeWorkspace {
                TerminalSurface(workspace: ws)
            } else {
                WorkspaceUnavailablePlaceholder()
            }
        case .filetree:
            FileTreeSurface(
                workspacePath: resolveWorkspacePath(),
                bridge: bridge,
                onFileOpen: onFileOpen
            )
        case .markdown:
            MarkdownSurface(filePath: statePath)
        case .image:
            ImageSurface(filePath: statePath)
        case .browser:
            BrowserSurface()
        case .code, .agentRun, .kanban, .memory, .instructionsGraph:
            NotYetImplementedSurface(kind: activeKind!)
        }
    }

    /// 파일 확장자로부터 적합한 SurfaceKind 를 결정한다.
    ///
    // @MX:NOTE: [AUTO] T-054: 확장자 → SurfaceKind 매핑.
    //            .md/.markdown → .markdown
    //            이미지 확장자 → .image
    //            나머지 → .terminal (기본값)
    static func kindForExtension(_ path: String) -> SurfaceKind {
        let ext = (path as NSString).pathExtension.lowercased()
        switch ext {
        case "md", "markdown":
            return .markdown
        case "png", "jpg", "jpeg", "gif", "webp", "svg":
            return .image
        default:
            return .terminal
        }
    }
}

// MARK: - WorkspaceUnavailablePlaceholder

/// 워크스페이스가 주입되지 않은 경우 표시되는 안내 뷰.
struct WorkspaceUnavailablePlaceholder: View {
    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "externaldrive.badge.questionmark")
                .font(.system(size: 32))
                .foregroundStyle(.secondary)
            Text("워크스페이스를 선택하세요")
                .font(.headline)
            Text("사이드바에서 워크스페이스를 선택하면 터미널을 사용할 수 있습니다.")
                .font(.caption)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - NotYetImplementedSurface

/// MS-4+ 구현 예정 Surface 의 플레이스홀더.
struct NotYetImplementedSurface: View {
    let kind: SurfaceKind

    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: kind.systemImage)
                .font(.system(size: 32))
                .foregroundStyle(.secondary)
            Text("\(kind.defaultTitle) Surface")
                .font(.headline)
            Text("MS-4+ 구현 예정")
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

// MARK: - PaneSplitContainerView (단축키 포함 래퍼)

/// 단축키(T-040)와 PaneSplitView 를 통합한 컨테이너 뷰.
// @MX:ANCHOR: [AUTO] 단축키 → PaneTreeModel 변이 → PaneSplitView 렌더링의 통합 진입점 (fan_in>=3)
// @MX:REASON: [AUTO] PaneContainer, 키보드 단축키 핸들러, 테스트 세 경로에서 참조
public struct PaneSplitContainerView: View {
    @Bindable var model: PaneTreeModel
    @State private var activePaneId: Int64?
    @Environment(WorkspaceViewModel.self) private var workspaceVM

    public init(model: PaneTreeModel) {
        self.model = model
    }

    public var body: some View {
        PaneSplitView(model: model, activePaneId: $activePaneId)
            .environment(\.activePane, makeActivePaneContext())
            .onAppear {
                // 초기 활성 pane 설정
                if activePaneId == nil {
                    activePaneId = model.rootId
                }
                syncActivePane(to: activePaneId)
            }
            .onChange(of: activePaneId) { _, newId in
                // split 노드 활성 불가 검증 (leaf 만 허용)
                if let id = newId, let node = model.nodes[id] {
                    assert(node.split == .leaf, "활성 pane 은 반드시 leaf 노드여야 합니다 (id: \(id))")
                }
                syncActivePane(to: newId)
            }
            // T-040: Cmd+\ — 수평 분할 (좌우)
            .onKeyboardShortcut(.init("\\", modifiers: .command)) {
                guard let paneId = activePaneId else { return }
                let newId = model.splitActive(paneId, direction: .horizontal)
                if let newId { activePaneId = newId }
            }
            // T-040: Cmd+Shift+\ — 수직 분할 (상하)
            .onKeyboardShortcut(.init("\\", modifiers: [.command, .shift])) {
                guard let paneId = activePaneId else { return }
                let newId = model.splitActive(paneId, direction: .vertical)
                if let newId { activePaneId = newId }
            }
            // T-040: Cmd+Shift+W — 활성 pane 닫기
            .onKeyboardShortcut(.init("w", modifiers: [.command, .shift])) {
                guard let paneId = activePaneId else { return }
                let closed = model.closePane(paneId)
                if closed {
                    activePaneId = model.rootId
                }
            }
    }

    // MARK: - 내부 헬퍼

    /// 현재 상태로 `ActivePaneContext` 를 생성한다.
    private func makeActivePaneContext() -> ActivePaneContext {
        ActivePaneContext(
            paneId: activePaneId,
            model: model,
            workspace: workspaceVM.activePane.workspace
        )
    }

    /// `activePaneId` 변경 시 `workspaceVM.activePane` 을 동기화한다.
    private func syncActivePane(to newId: Int64?) {
        workspaceVM.activePane = ActivePaneContext(
            paneId: newId,
            model: model,
            workspace: workspaceVM.activePane.workspace
        )
    }
}

// MARK: - View 확장 (onKeyboardShortcut 헬퍼)

private extension View {
    /// SwiftUI .commands 없이 View 레벨에서 키보드 단축키를 처리하는 헬퍼.
    // @MX:NOTE: [AUTO] NSSplitView 포커스 관리와 SwiftUI .keyboardShortcut 충돌 방지를 위해
    //            NSViewRepresentable 레이어 대신 SwiftUI overlay 에서 처리.
    func onKeyboardShortcut(_ shortcut: KeyboardShortcut, action: @escaping () -> Void) -> some View {
        self.background(
            Button("") { action() }
                .keyboardShortcut(shortcut)
                .frame(width: 0, height: 0)
                .opacity(0)
        )
    }
}
