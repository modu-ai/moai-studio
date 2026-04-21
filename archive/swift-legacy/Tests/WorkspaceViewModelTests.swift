//
//  WorkspaceViewModelTests.swift
//  T-027: @Observable ViewModel + poll_event 바인딩 검증.
//

import XCTest
@testable import MoAIStudio

@MainActor
final class WorkspaceViewModelTests: XCTestCase {

    func testCreateWorkspaceRegistersAndSelects() {
        let mock = MockRustCoreBridge()
        let vm = WorkspaceViewModel(bridge: mock)

        let id = vm.createWorkspace(name: "alpha", projectPath: "/tmp/alpha")
        XCTAssertNotNil(id)
        XCTAssertEqual(vm.workspaces.count, 1)
        XCTAssertEqual(vm.selectedWorkspaceId, id)
        XCTAssertTrue(mock.subscribedIds.contains(id!))
    }

    func testCreateWorkspaceRejectsEmptyName() {
        let mock = MockRustCoreBridge()
        let vm = WorkspaceViewModel(bridge: mock)
        let id = vm.createWorkspace(name: "   ", projectPath: "/tmp")
        XCTAssertNil(id)
        XCTAssertTrue(vm.workspaces.isEmpty)
    }

    func testDeleteClearsSelection() {
        let mock = MockRustCoreBridge()
        let vm = WorkspaceViewModel(bridge: mock)
        let id = vm.createWorkspace(name: "one", projectPath: "/")!

        vm.deleteWorkspace(workspaceId: id)
        XCTAssertNil(vm.selectedWorkspaceId)
        XCTAssertTrue(vm.workspaces.isEmpty)
    }

    func testDrainEventsOncePullsFromMockQueue() {
        let mock = MockRustCoreBridge()
        let vm = WorkspaceViewModel(bridge: mock)
        let id = vm.createWorkspace(name: "ws", projectPath: "/")!

        mock.enqueue(workspaceId: id, event: "{\"type\":\"assistant\",\"text\":\"hi\"}")
        mock.enqueue(workspaceId: id, event: "{\"type\":\"partial\",\"text\":\"…\"}")

        vm.drainEventsOnce()
        vm.drainEventsOnce()

        XCTAssertEqual(vm.recentEvents.count, 2)
        XCTAssertTrue(vm.recentEvents[0].contains("assistant"))
        XCTAssertTrue(vm.recentEvents[1].contains("partial"))
    }

    func testStartStopPollingIsIdempotentAndSafe() {
        let mock = MockRustCoreBridge()
        let vm = WorkspaceViewModel(bridge: mock)
        vm.startPolling()
        vm.startPolling() // second call must be no-op
        vm.stopPolling()
        vm.stopPolling() // safe double-stop
        // Cannot deterministically assert timer count, but absence of crash = pass.
    }

    func testRestartClaudeSendsSpecialMessage() {
        let mock = MockRustCoreBridge()
        let vm = WorkspaceViewModel(bridge: mock)
        let id = vm.createWorkspace(name: "x", projectPath: "/")!
        vm.restartClaude(workspaceId: id)
        XCTAssertEqual(mock.messagesSent.last?.0, id)
        XCTAssertEqual(mock.messagesSent.last?.1, "__moai_restart__")
    }

    func testWorkspaceLookupById() {
        let mock = MockRustCoreBridge()
        let vm = WorkspaceViewModel(bridge: mock)
        let id = vm.createWorkspace(name: "lookup", projectPath: "/")!
        XCTAssertEqual(vm.workspace(id: id)?.name, "lookup")
        XCTAssertNil(vm.workspace(id: "nonexistent"))
    }

    func testRefreshSyncsSelectedWhenMissing() {
        let mock = MockRustCoreBridge()
        let vm = WorkspaceViewModel(bridge: mock)
        vm.selectedWorkspaceId = "ghost-id"
        vm.refresh()
        XCTAssertNil(vm.selectedWorkspaceId)
    }

    func testPollIntervalIs60Hz() {
        XCTAssertEqual(WorkspaceViewModel.pollIntervalMs, 16)
    }
}
