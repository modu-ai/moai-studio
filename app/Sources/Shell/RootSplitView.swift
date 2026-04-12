//
//  RootSplitView.swift
//  NavigationSplitView 기반 사이드바 + 콘텐츠 영역 레이아웃 (RG-M1-1, RG-M1-6).
//
//  @MX:NOTE: 사이드바 너비는 navigationSplitViewColumnWidth(min:200, ideal:250, max:400) 로 클램프.
//

import SwiftUI

struct RootSplitView: View {
    @Environment(WorkspaceViewModel.self) private var viewModel
    @Environment(WindowStateStore.self) private var windowState

    var body: some View {
        @Bindable var viewModelBindable = viewModel

        NavigationSplitView {
            WorkspaceListView()
                .navigationSplitViewColumnWidth(
                    min: WindowStateStore.sidebarMinWidth,
                    ideal: windowState.sidebarWidth,
                    max: WindowStateStore.sidebarMaxWidth
                )
        } detail: {
            ContentArea(selected: viewModelBindable.selectedWorkspaceId)
        }
        .navigationSplitViewStyle(.balanced)
    }
}
