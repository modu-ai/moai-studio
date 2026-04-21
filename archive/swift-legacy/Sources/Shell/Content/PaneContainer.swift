//
//  PaneContainer.swift
//  선택된 워크스페이스의 PaneTreeModel 을 관리하고 PaneSplitContainerView 를 렌더링 (T-043).
//
//  @MX:NOTE: [AUTO] 워크스페이스별 PaneTreeModel 캐시를 유지하여 전환 시 재로드를 방지.
//             선택된 워크스페이스가 없으면 EmptyState 를 표시한다.
//

import SwiftUI

/// 활성 워크스페이스의 pane tree 를 로드하고 PaneSplitContainerView 를 호스팅하는 컨테이너.
// @MX:ANCHOR: [AUTO] WorkspaceViewModel ↔ PaneTreeModel ↔ PaneSplitView 연결 허브 (fan_in>=3)
// @MX:REASON: [AUTO] RootSplitView, WorkspaceViewModel, PaneSplitContainerView 세 경로에서 참조
public struct PaneContainer: View {
    @Environment(WorkspaceViewModel.self) private var workspaceVM
    let selectedWorkspaceId: String?

    /// 워크스페이스 UUID → DB id → PaneTreeModel 캐시
    @State private var modelCache: [String: PaneTreeModel] = [:]
    @State private var isLoading = false

    public var body: some View {
        Group {
            if let wsId = selectedWorkspaceId {
                contentView(for: wsId)
            } else {
                EmptyState()
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onChange(of: selectedWorkspaceId) { _, newId in
            guard let newId else { return }
            loadModelIfNeeded(for: newId)
        }
    }

    // MARK: - 내부

    @ViewBuilder
    private func contentView(for workspaceId: String) -> some View {
        if let model = modelCache[workspaceId] {
            // MS-2 T-M2.5-006: WorkspaceSnapshot 확보 후 하위 뷰에 .environment(\.activeWorkspace) 주입
            let snapshot = workspaceVM.workspace(id: workspaceId)
            PaneSplitContainerView(model: model)
                .environment(\.activeWorkspace, snapshot)
        } else {
            ProgressView("Pane 로드 중...")
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .task {
                    loadModelIfNeeded(for: workspaceId)
                }
        }
    }

    private func loadModelIfNeeded(for workspaceUuid: String) {
        guard modelCache[workspaceUuid] == nil else { return }

        let bridge = workspaceVM.bridge
        let dbId = bridge.getWorkspaceDbId(workspaceUuid: workspaceUuid)
        guard dbId > 0 else { return }

        let model = PaneTreeModel(workspaceId: dbId, bridge: bridge)
        // 캐시에 먼저 등록 후 비동기 로드
        modelCache[workspaceUuid] = model

        Task { @MainActor in
            await model.load()
        }
    }
}
