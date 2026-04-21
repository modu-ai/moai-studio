//
//  ContextMenu.swift
//  사이드바 항목 우클릭 컨텍스트 메뉴 (Rename / Restart Claude / Delete).
//

import SwiftUI

struct WorkspaceContextMenu: View {
    @Environment(WorkspaceViewModel.self) private var viewModel
    let workspace: WorkspaceSnapshot

    var body: some View {
        Button("Rename…") {
            viewModel.requestRename(workspaceId: workspace.id)
        }
        Button("Restart Claude") {
            viewModel.restartClaude(workspaceId: workspace.id)
        }
        Divider()
        Button("Delete", role: .destructive) {
            viewModel.deleteWorkspace(workspaceId: workspace.id)
        }
    }
}
