//
//  MoAIStudioApp.swift
//  MoAI Studio — M1 Working Shell 메인 진입점.
//
//  @MX:ANCHOR: App 진입점 (fan_in>=3 — SwiftUI SceneBuilder, AppDelegate, WindowState)
//  @MX:REASON: MoAI Studio 의 유일한 `@main` 엔트리. 윈도우 복원/ViewModel 주입의 단일 지점.
//

import SwiftUI

@main
struct MoAIStudioApp: App {
    // WorkspaceViewModel 은 앱 전 생애주기 동안 단일 인스턴스.
    @State private var viewModel = WorkspaceViewModel()

    // UserDefaults 기반 윈도우 상태 저장소.
    @State private var windowState = WindowStateStore()

    var body: some Scene {
        WindowGroup("MoAI Studio") {
            MainWindow()
                .environment(viewModel)
                .environment(windowState)
                .frame(
                    minWidth: 800,
                    idealWidth: windowState.windowWidth,
                    minHeight: 500,
                    idealHeight: windowState.windowHeight
                )
        }
        .windowStyle(.titleBar)
        .windowResizability(.contentMinSize)
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("New Workspace") {
                    viewModel.requestNewWorkspace()
                }
                .keyboardShortcut("n", modifiers: [.command])
            }
        }
    }
}
