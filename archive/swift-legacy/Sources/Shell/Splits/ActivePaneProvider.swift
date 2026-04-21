//
//  ActivePaneProvider.swift
//  활성 pane 환경값 키 + ActivePaneContext value struct.
//  SPEC-M2-002 MS-1 (T-M2.5-001 ~ T-M2.5-004)
//
//  @MX:ANCHOR: [AUTO] 활성 pane 환경값의 유일한 타입 (fan_in>=3)
//  @MX:REASON: [AUTO] RootSplitView (Command Palette), PaneSplitContainerView (상태 주입),
//              LeafPaneView/SurfaceRouter (소비) 세 경로 공유

import SwiftUI

// MARK: - ActivePaneContext

/// 활성 pane 의 컨텍스트 snapshot.
///
/// `@Environment(\.activePane)` 로 하향 전파되며, Command Palette 오버레이 경로에서는
/// `WorkspaceViewModel.activePane` `@Observable` 프로퍼티로 접근한다.
///
/// `model: PaneTreeModel?` 접근은 반드시 `@MainActor` 컨텍스트에서 수행해야 한다.
public struct ActivePaneContext: Equatable {
    /// Rust DB rowid. nil = 활성 pane 없음.
    public var paneId: Int64?
    /// pane tree 모델 — split/close 연산에 사용. nil 이면 연산 불가.
    public var model: PaneTreeModel?
    /// 현재 워크스페이스 snapshot. nil 이면 워크스페이스 미선택.
    public var workspace: WorkspaceSnapshot?

    public init(
        paneId: Int64? = nil,
        model: PaneTreeModel? = nil,
        workspace: WorkspaceSnapshot? = nil
    ) {
        self.paneId = paneId
        self.model = model
        self.workspace = workspace
    }

    /// 활성 pane 없음을 나타내는 기본 상수.
    public static let empty = ActivePaneContext()

    // MARK: Equatable — paneId 기준 비교 (reference 타입 model 제외)
    public static func == (lhs: ActivePaneContext, rhs: ActivePaneContext) -> Bool {
        lhs.paneId == rhs.paneId && lhs.workspace?.id == rhs.workspace?.id
    }
}

// MARK: - EnvironmentKey: ActivePaneProviderKey

/// `@Environment(\.activePane)` 의 키.
///
// @MX:ANCHOR: [AUTO] @Environment(\.activePane) 의 유일한 진입점
// @MX:REASON: [AUTO] 환경값 소비자는 전부 이 computed property 를 거친다
private struct ActivePaneProviderKey: EnvironmentKey {
    static let defaultValue: ActivePaneContext = .empty
}

// MARK: - EnvironmentKey: WorkspaceEnvironmentKey

/// `@Environment(\.activeWorkspace)` 의 키.
///
/// `PaneContainer` 가 `.environment(\.activeWorkspace, snapshot)` 으로 주입하고,
/// `SurfaceRouter` 가 `TerminalSurface(workspace:)` 생성에 소비한다.
private struct WorkspaceEnvironmentKey: EnvironmentKey {
    static let defaultValue: WorkspaceSnapshot? = nil
}

// MARK: - EnvironmentValues extension

public extension EnvironmentValues {
    /// 현재 활성 pane 의 컨텍스트. 기본값 = `ActivePaneContext.empty`.
    var activePane: ActivePaneContext {
        get { self[ActivePaneProviderKey.self] }
        set { self[ActivePaneProviderKey.self] = newValue }
    }

    /// 현재 활성 워크스페이스 snapshot. 기본값 = nil.
    var activeWorkspace: WorkspaceSnapshot? {
        get { self[WorkspaceEnvironmentKey.self] }
        set { self[WorkspaceEnvironmentKey.self] = newValue }
    }
}
