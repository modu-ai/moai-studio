//
//  RootSplitView.swift
//  NavigationSplitView 기반 사이드바 + 콘텐츠 영역 레이아웃 (RG-M1-1, RG-M1-6).
//
//  @MX:NOTE: [AUTO] 사이드바 너비는 navigationSplitViewColumnWidth(min:200, ideal:250, max:400) 로 클램프.
//  @MX:NOTE: [AUTO] MS-2 (T-043): detail 영역을 ContentArea → PaneContainer 로 교체.
//             ContentArea.swift 는 MS-3 에서 TabBarView 통합 완료 후 제거 예정.
//  @MX:NOTE: [AUTO] MS-6 (T-067): CommandPaletteController 를 .overlay 로 삽입.
//             Cmd+K 단축키는 Button + .keyboardShortcut 방식으로 캡처한다.
//             SwiftUI onKeyPress 의 modifier 파라미터는 macOS 15+ 에서만 지원됨.

import SwiftUI
import os.log

private let logger = Logger(subsystem: "kr.mo.ai.moai-studio", category: "RootSplitView")

struct RootSplitView: View {
    @Environment(WorkspaceViewModel.self) private var viewModel
    @Environment(WindowStateStore.self) private var windowState

    /// MS-6: Command Palette Controller — Registry 는 ViewModel 콜백과 연결
    @State private var paletteController: CommandPaletteController? = nil
    @State private var showNewWorkspaceSheet = false

    var body: some View {
        @Bindable var viewModelBindable = viewModel

        ZStack {
            NavigationSplitView {
                WorkspaceListView()
                    .navigationSplitViewColumnWidth(
                        min: WindowStateStore.sidebarMinWidth,
                        ideal: windowState.sidebarWidth,
                        max: WindowStateStore.sidebarMaxWidth
                    )
            } detail: {
                // MS-2 T-043: PaneContainer 가 PaneSplitView binary tree 를 렌더링한다.
                // 워크스페이스 미선택 시 EmptyState 를 표시한다.
                PaneContainer(selectedWorkspaceId: viewModelBindable.selectedWorkspaceId)
            }
            .navigationSplitViewStyle(.balanced)
            // @MX:NOTE: [AUTO] Cmd+K 캡처: 숨겨진 Button + .keyboardShortcut 방식 사용.
            //            NavigationSplitView 에 직접 onKeyPress(modifier:) 적용 시
            //            macOS 14 에서 컴파일 오류 발생 — keyboardShortcut 방식으로 대체.
            .background(
                Button("") {
                    paletteController?.open()
                }
                .keyboardShortcut("k", modifiers: .command)
                .opacity(0)
                .allowsHitTesting(false)
            )

            // MS-6 T-067: Command Palette 오버레이
            if let controller = paletteController {
                CommandPaletteView(controller: controller)
            }
        }
        .sheet(isPresented: $showNewWorkspaceSheet) {
            NewWorkspaceSheet(isPresented: $showNewWorkspaceSheet)
                .environment(viewModel)
        }
        .onAppear {
            setupPaletteController()
        }
    }

    // MARK: - Command Palette 초기화

    /// CommandRegistry 를 ViewModel 콜백과 연결하여 PaletteController 를 생성한다.
    private func setupPaletteController() {
        guard paletteController == nil else { return }

        let vm = viewModel
        let showSheet = { showNewWorkspaceSheet = true }

        let registry = CommandRegistry(
            onMoaiSlash: { text in
                let injector = SlashInjector(bridge: vm.bridge, workspaceVM: vm)
                injector.inject(text)
            },
            onSurfaceOpen: { kind in
                // @MX:NOTE: [AUTO] MS-3 완료 — workspaceVM.activePane 기반 TabBarViewModel.newTab(kind:) 호출.
                guard let paneId = vm.activePane.paneId else {
                    logger.info("Command Palette: onSurfaceOpen 무시 — 활성 pane 없음")
                    return
                }
                guard let tabModel = vm.tabModels[paneId] else {
                    logger.info("Command Palette: onSurfaceOpen 무시 — tabModels[\(paneId)] 미등록")
                    return
                }
                _ = tabModel.newTab(kind: kind)
            },
            onWorkspaceCreate: {
                showSheet()
            },
            onPaneSplit: { direction in
                // @MX:NOTE: [AUTO] MS-3 완료 — workspaceVM.activePane.model.splitActive 호출. 키보드 단축키와 동일 호출 시퀀스.
                guard let paneId = vm.activePane.paneId,
                      let model = vm.activePane.model else {
                    logger.info("Command Palette: onPaneSplit 무시 — 활성 pane 컨텍스트 없음")
                    return
                }
                let splitKind = paneSplitKind(from: direction)
                _ = model.splitActive(paneId, direction: splitKind)
            }
        )

        paletteController = CommandPaletteController(registry: registry)
    }

    // MARK: - 헬퍼

    /// `PaneSplitDirection` → `SplitKind` 매핑.
    private func paneSplitKind(from direction: PaneSplitDirection) -> SplitKind {
        switch direction {
        case .horizontal: return .horizontal
        case .vertical:   return .vertical
        }
    }
}
