//
//  EmptyState.swift
//  활성 워크스페이스가 없을 때 표시되는 환영 화면 + "Create Workspace" CTA.
//

import SwiftUI

struct EmptyState: View {
    @Environment(WorkspaceViewModel.self) private var viewModel
    @State private var showSheet = false

    var body: some View {
        VStack(spacing: 16) {
            Image(systemName: "terminal")
                .font(.system(size: 72, weight: .light))
                .foregroundStyle(.secondary)
            Text("Welcome to MoAI Studio")
                .font(.largeTitle)
                .bold()
            Text("Create a workspace to start a Claude session.")
                .foregroundStyle(.secondary)
            Button {
                showSheet = true
            } label: {
                Label("Create Workspace", systemImage: "plus.circle.fill")
                    .padding(.horizontal, 8)
            }
            .buttonStyle(.borderedProminent)
            .controlSize(.large)
            .accessibilityIdentifier("emptyState.createWorkspace")
        }
        .padding()
        .sheet(isPresented: $showSheet) {
            NewWorkspaceSheet(isPresented: $showSheet)
        }
    }
}
