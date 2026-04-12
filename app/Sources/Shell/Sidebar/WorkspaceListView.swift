//
//  WorkspaceListView.swift
//  사이드바 워크스페이스 목록 + 상태 아이콘 (RG-M1-6 / T-023).
//

import SwiftUI

struct WorkspaceListView: View {
    @Environment(WorkspaceViewModel.self) private var viewModel
    @State private var showNewSheet = false

    var body: some View {
        @Bindable var vm = viewModel

        List(selection: $vm.selectedWorkspaceId) {
            Section("Workspaces") {
                ForEach(vm.workspaces) { workspace in
                    HStack(spacing: 8) {
                        StatusIcon(status: workspace.status)
                        Text(workspace.name)
                            .lineLimit(1)
                            .truncationMode(.tail)
                        Spacer()
                    }
                    .tag(workspace.id as String?)
                    .contextMenu {
                        WorkspaceContextMenu(workspace: workspace)
                    }
                }
            }
        }
        .listStyle(.sidebar)
        .safeAreaInset(edge: .bottom) {
            HStack {
                Button {
                    showNewSheet = true
                } label: {
                    Label("New Workspace", systemImage: "plus.circle.fill")
                }
                .buttonStyle(.borderless)
                .accessibilityIdentifier("sidebar.plus")
                Spacer()
            }
            .padding(8)
        }
        .sheet(isPresented: $showNewSheet) {
            NewWorkspaceSheet(isPresented: $showNewSheet)
        }
        .navigationTitle("MoAI Studio")
    }
}
