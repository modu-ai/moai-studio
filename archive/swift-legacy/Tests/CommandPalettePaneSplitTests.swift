//
//  CommandPalettePaneSplitTests.swift
//  Command Palette onPaneSplit 콜백 테스트
//  SPEC-M2-002 MS-3 T-M2.5-013, T-M2.5-015 (AC-4.1 ~ AC-4.4)
//

import XCTest
@testable import MoAIStudio

@MainActor
final class CommandPalettePaneSplitTests: XCTestCase {
    private var mock: MockRustCoreBridge!
    private var vm: WorkspaceViewModel!
    private var paneModel: PaneTreeModel!
    private var testPaneId: Int64!

    override func setUp() async throws {
        mock = MockRustCoreBridge()
        vm = WorkspaceViewModel(bridge: mock)

        // DB id=1 워크스페이스로 PaneTreeModel 생성
        mock.workspaceDbIds["ws-1"] = 1
        paneModel = PaneTreeModel(workspaceId: 1, bridge: mock)
        await paneModel.load()

        // 루트 pane id 를 활성 pane 으로 설정
        testPaneId = paneModel.rootId
        XCTAssertNotNil(testPaneId, "PaneTreeModel 로드 후 rootId 가 있어야 함")

        vm.activePane = ActivePaneContext(paneId: testPaneId, model: paneModel, workspace: nil)
    }

    // MARK: - T-M2.5-013: AC-4.1 수평 분할

    /// `onPaneSplit(.horizontal)` 은 `PaneTreeModel.splitActive(id, .horizontal)` 을 호출해야 한다.
    func test_onPaneSplit_horizontal_callsSplitActive() throws {
        let initialNodeCount = paneModel.nodes.count

        let newId = simulateOnPaneSplit(direction: .horizontal)

        XCTAssertNotNil(newId, "수평 분할 후 새 pane id 가 반환되어야 함")
        XCTAssertGreaterThan(paneModel.nodes.count, initialNodeCount, "분할 후 노드 수가 증가해야 함")
    }

    // MARK: - AC-4.2 수직 분할

    /// `onPaneSplit(.vertical)` 은 수직 방향 분할을 수행해야 한다.
    func test_onPaneSplit_vertical_callsSplitActive() throws {
        let initialNodeCount = paneModel.nodes.count

        let newId = simulateOnPaneSplit(direction: .vertical)

        XCTAssertNotNil(newId, "수직 분할 후 새 pane id 가 반환되어야 함")
        XCTAssertGreaterThan(paneModel.nodes.count, initialNodeCount)
    }

    // MARK: - AC-4.3 nil pane 시 no-op (크래시 없음)

    /// 활성 pane 이 없을 때 `onPaneSplit` 은 no-op 이어야 한다 (크래시 없음).
    func test_onPaneSplit_nilPaneIdOrModel_noops() throws {
        vm.activePane = .empty

        // no-op: guard 탈출 — 크래시 없음
        let newId = simulateOnPaneSplit(direction: .horizontal)

        XCTAssertNil(newId, "활성 pane 없을 때 newId 는 nil 이어야 함")
    }

    // MARK: - AC-4.4 새 pane id 반영

    /// 분할 후 반환된 새 pane id 가 트리에 leaf 로 존재해야 한다.
    func test_onPaneSplit_newPaneId_isLeafInTree() throws {
        guard let newId = simulateOnPaneSplit(direction: .horizontal) else {
            XCTFail("분할 후 newId 가 없어서는 안 됨")
            return
        }

        let newNode = paneModel.nodes[newId]
        XCTAssertNotNil(newNode, "새 pane id 가 트리에 존재해야 함")
        XCTAssertEqual(newNode?.split, .leaf, "새 pane 은 leaf 여야 함")
    }
}

// MARK: - 테스트 헬퍼

extension CommandPalettePaneSplitTests {
    /// `onPaneSplit` 콜백과 동일한 로직을 실행하는 헬퍼. 새 pane id 를 반환한다.
    @discardableResult
    private func simulateOnPaneSplit(direction: PaneSplitDirection) -> Int64? {
        guard let paneId = vm.activePane.paneId,
              let model = vm.activePane.model else {
            return nil
        }
        let splitKind: SplitKind = (direction == .horizontal) ? .horizontal : .vertical
        return model.splitActive(paneId, direction: splitKind)
    }
}
