//
//  ContentArea.swift
//  우측 콘텐츠 영역 — 활성 워크스페이스의 터미널 또는 빈 상태 표시.
//

import SwiftUI

struct ContentArea: View {
    @Environment(WorkspaceViewModel.self) private var viewModel
    let selected: String?

    var body: some View {
        Group {
            if let id = selected, let workspace = viewModel.workspace(id: id) {
                TerminalSurface(workspace: workspace)
            } else {
                EmptyState()
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
