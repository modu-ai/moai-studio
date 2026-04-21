//
//  NewWorkspaceSheet.swift
//  "+" 버튼으로 열리는 새 워크스페이스 생성 시트 (T-024).
//

import SwiftUI

struct NewWorkspaceSheet: View {
    @Environment(WorkspaceViewModel.self) private var viewModel
    @Binding var isPresented: Bool

    @State private var name: String = ""
    @State private var projectPath: String = ""

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Create Workspace")
                .font(.title2)
                .bold()

            Form {
                TextField("Name", text: $name)
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier("newWorkspace.name")
                TextField("Project Path", text: $projectPath, prompt: Text("/absolute/path/to/project"))
                    .textFieldStyle(.roundedBorder)
                    .accessibilityIdentifier("newWorkspace.projectPath")
            }

            HStack {
                Spacer()
                Button("Cancel") {
                    isPresented = false
                }
                .keyboardShortcut(.cancelAction)

                Button("Create") {
                    viewModel.createWorkspace(name: name, projectPath: projectPath)
                    isPresented = false
                }
                .keyboardShortcut(.defaultAction)
                .disabled(name.trimmingCharacters(in: .whitespaces).isEmpty)
            }
        }
        .padding(20)
        .frame(width: 420)
    }
}
