//
//  WorkspaceViewModel.swift
//  @Observable ViewModel — swift-bridge subscribe_events 폴링 바인딩 + 사이드바/콘텐츠 상태 동기화.
//
//  @MX:ANCHOR: UI ↔ Rust Core 상태 브릿지 (fan_in>=5: List, ContextMenu, Sheet, EmptyState, ContentArea)
//  @MX:REASON: 모든 workspace 상태 mutation 을 이 ViewModel 로 집중하여 FFI 호출 지점을 단일화.
//  @MX:WARN: poll_event 타이머는 stopPolling() 또는 deinit 에서 반드시 취소할 것 (leak 위험).
//  @MX:REASON: DispatchSource.timer 는 strong retain 되므로 명시적 cancel 없이는 해제되지 않음.
//

import Foundation
import Observation

/// 워크스페이스 상태 머신 + 이벤트 스트림을 @Observable 로 노출.
///
/// Rust 측 `subscribe_events(workspace_id)` + 고빈도 `poll_event(workspace_id)` 조합을
/// `DispatchSource.timer` 16ms (≈60 Hz) 로 감싼다.
@Observable
@MainActor
public final class WorkspaceViewModel {
    public private(set) var workspaces: [WorkspaceSnapshot] = []
    public var selectedWorkspaceId: String? = nil

    /// 최근 수신된 이벤트 (테스트 및 fallback 콘솔 표시에 사용).
    public private(set) var recentEvents: [String] = []

    /// 폴링 간격. 16ms ≈ 60 Hz (spec §RG-M1-3 참조).
    public static let pollIntervalMs: Int = 16

    private let bridge: RustCoreBridging
    private var pollTimer: DispatchSourceTimer?
    private var subscribedWorkspaces: Set<String> = []

    /// 기본 이니셜라이저 — 프로덕션용 RustCoreBridge 를 자동 주입.
    public convenience init() {
        self.init(bridge: RustCoreBridge())
    }

    /// 테스트·SwiftUI preview 용 DI 이니셜라이저.
    public init(bridge: RustCoreBridging) {
        self.bridge = bridge
        refresh()
    }

    // MARK: - Public API

    /// Rust 코어에서 워크스페이스 목록을 재조회.
    public func refresh() {
        let fetched = bridge.listWorkspaces()
        workspaces = fetched
        if let id = selectedWorkspaceId, !fetched.contains(where: { $0.id == id }) {
            selectedWorkspaceId = fetched.first?.id
        }
    }

    public func workspace(id: String) -> WorkspaceSnapshot? {
        workspaces.first(where: { $0.id == id })
    }

    /// 새 워크스페이스 생성 후 목록 갱신 + 자동 선택.
    @discardableResult
    public func createWorkspace(name: String, projectPath: String) -> String? {
        let trimmedName = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedName.isEmpty else { return nil }
        let newId = bridge.createWorkspace(name: trimmedName, projectPath: projectPath)
        refresh()
        selectedWorkspaceId = newId
        _ = bridge.subscribeEvents(workspaceId: newId)
        subscribedWorkspaces.insert(newId)
        return newId
    }

    public func deleteWorkspace(workspaceId: String) {
        _ = bridge.deleteWorkspace(id: workspaceId)
        subscribedWorkspaces.remove(workspaceId)
        if selectedWorkspaceId == workspaceId {
            selectedWorkspaceId = nil
        }
        refresh()
    }

    public func requestRename(workspaceId: String) {
        // M1 에서는 자리만 확보. 실제 rename flow 는 MS-5 후속 태스크.
        // @MX:TODO: MS-5 에서 rename sheet UI 및 Rust FFI `rename_workspace` 연결.
        _ = workspaceId
    }

    public func restartClaude(workspaceId: String) {
        _ = bridge.sendUserMessage(workspaceId: workspaceId, message: "__moai_restart__")
    }

    public func requestNewWorkspace() {
        // App Menu 에서 호출되는 placeholder. 실제 sheet 표시는 WorkspaceListView 의 state 로 처리.
    }

    // MARK: - Polling (subscribe_events bridge)

    /// 16ms 간격으로 pollEvent 를 실행하는 DispatchSource 타이머 시작.
    public func startPolling() {
        guard pollTimer == nil else { return }
        let timer = DispatchSource.makeTimerSource(queue: DispatchQueue.global(qos: .userInteractive))
        timer.schedule(deadline: .now(), repeating: .milliseconds(Self.pollIntervalMs))
        timer.setEventHandler { [weak self] in
            guard let self else { return }
            Task { @MainActor in
                self.drainEventsOnce()
            }
        }
        timer.resume()
        pollTimer = timer
    }

    /// 폴링 중단 — 타이머 해제 (필수, leak 방지).
    public func stopPolling() {
        pollTimer?.cancel()
        pollTimer = nil
    }

    // @MX:WARN: deinit 에서 타이머 취소 불가 (MainActor 격리). 반드시 stopPolling() 을
    //           View.onDisappear 등에서 명시적으로 호출할 것.
    // @MX:REASON: Swift 6 엄격한 actor isolation 때문에 deinit 에서 isolated 프로퍼티 접근 금지.
    //             DispatchSourceTimer 는 ARC 로 자연 해제되지만 취소 전 tick 발생 가능 — onDisappear 필수.

    /// 모든 구독 워크스페이스의 큐에서 이벤트를 1개씩 소비.
    /// 테스트에서는 이 메서드를 직접 호출하여 타이머 없이 동기 검증 가능.
    public func drainEventsOnce() {
        // 워크스페이스마다 1건씩만 pop — 공정 분배.
        let targets = subscribedWorkspaces.isEmpty
            ? workspaces.map(\.id)
            : Array(subscribedWorkspaces)
        for id in targets {
            if let payload = bridge.pollEvent(workspaceId: id) {
                recentEvents.append(payload)
                // 버퍼 최대 200건 유지.
                if recentEvents.count > 200 {
                    recentEvents.removeFirst(recentEvents.count - 200)
                }
            }
        }
    }
}
