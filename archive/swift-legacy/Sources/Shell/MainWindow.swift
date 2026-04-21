//
//  MainWindow.swift
//  메인 윈도우 Scene — Sidebar + ContentArea 2-pane 구조.
//
//  @MX:NOTE: RG-M1-1 / RG-M1-6 의 메인 윈도우 레이아웃 루트.
//

import SwiftUI

/// 최상위 메인 윈도우. RootSplitView 를 호스팅한다.
struct MainWindow: View {
    @Environment(WorkspaceViewModel.self) private var viewModel
    @Environment(WindowStateStore.self) private var windowState

    var body: some View {
        RootSplitView()
            .onAppear {
                viewModel.startPolling()
            }
            .onDisappear {
                viewModel.stopPolling()
                windowState.persist()
            }
    }
}
