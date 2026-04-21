//
//  PaneTreeModelTests.swift
//  PaneTreeModel 단위 테스트 (SPEC-M2-001 MS-2 T-039, T-042).
//
//  MockRustCoreBridge 를 사용하여 FFI 없이 순수 로직만 검증한다.
//  XCUITest (NSSplitView UI 상호작용) 는 서명 이슈(C-1) 로 제외 — 수동 검증 예정.
//

import XCTest
@testable import MoAIStudio

@MainActor
final class PaneTreeModelTests: XCTestCase {
    // MARK: - 픽스처

    private var mock: MockRustCoreBridge!
    private var model: PaneTreeModel!

    override func setUp() async throws {
        mock = MockRustCoreBridge()
        // 워크스페이스 DB id 설정
        mock.workspaceDbIds["ws-1"] = 1
        model = PaneTreeModel(workspaceId: 1, bridge: mock)
    }

    // MARK: - T-042: load() 테스트

    /// 저장된 pane 이 없으면 load() 가 루트 leaf pane 을 자동 생성한다.
    func test_load_createsRootLeafWhenEmpty() async throws {
        // Arrange: 빈 상태 (mock 의 listPanesJson 은 "[]" 반환)
        // Act
        await model.load()

        // Assert
        XCTAssertNotNil(model.rootId, "루트 pane id 가 설정되어야 함")
        XCTAssertEqual(model.nodes.count, 1)
        guard let root = model.tree() else {
            XCTFail("루트 노드가 있어야 함")
            return
        }
        XCTAssertEqual(root.split, .leaf)
        XCTAssertNil(root.parentId)
    }

    /// load() 는 기존 pane 트리를 올바르게 복원한다.
    func test_load_restoresExistingTree() async throws {
        // Arrange: 미리 pane 2개를 DB 에 생성
        let rootId = mock.createPane(workspaceId: 1, parentId: 0, split: "horizontal", ratio: 0.5)
        _ = mock.createPane(workspaceId: 1, parentId: rootId, split: "leaf", ratio: 0.5)
        _ = mock.createPane(workspaceId: 1, parentId: rootId, split: "leaf", ratio: 0.5)

        // Act
        await model.load()

        // Assert
        XCTAssertEqual(model.nodes.count, 3)
        XCTAssertEqual(model.rootId, rootId)
        guard let root = model.tree() else {
            XCTFail("루트 노드가 있어야 함")
            return
        }
        XCTAssertEqual(root.split, .horizontal)
        XCTAssertEqual(root.childIds.count, 2)
    }

    // MARK: - T-039: splitActive() 테스트

    /// splitActive() 는 leaf 를 horizontal 로 분할하고 신규 형제 pane id 를 반환한다.
    func test_splitActive_horizontal_convertsLeafToTwoChildren() async throws {
        await model.load()
        guard let rootId = model.rootId else { XCTFail(); return }

        // Act
        let siblingId = model.splitActive(rootId, direction: .horizontal)

        // Assert
        XCTAssertNotNil(siblingId, "신규 형제 pane id 가 반환되어야 함")
        guard let root = model.nodes[rootId] else { XCTFail(); return }
        XCTAssertEqual(root.split, .horizontal, "루트가 horizontal 로 전환되어야 함")
        XCTAssertEqual(root.childIds.count, 2, "자식이 정확히 2개여야 함")

        // 자식들은 leaf 여야 한다
        for childId in root.childIds {
            XCTAssertEqual(model.nodes[childId]?.split, .leaf)
        }
    }

    /// splitActive() 는 leaf 를 vertical 로 분할한다.
    func test_splitActive_vertical_createsTopBottomChildren() async throws {
        await model.load()
        guard let rootId = model.rootId else { XCTFail(); return }

        let siblingId = model.splitActive(rootId, direction: .vertical)

        XCTAssertNotNil(siblingId)
        XCTAssertEqual(model.nodes[rootId]?.split, .vertical)
    }

    /// 비-leaf 에 splitActive() 를 호출하면 nil 을 반환한다.
    func test_splitActive_nonLeaf_returnsNil() async throws {
        await model.load()
        guard let rootId = model.rootId else { XCTFail(); return }
        // 먼저 분할하여 루트를 비-leaf 로 만든다
        model.splitActive(rootId, direction: .horizontal)

        // 비-leaf 에 다시 분할 시도
        let result = model.splitActive(rootId, direction: .horizontal)
        XCTAssertNil(result, "비-leaf 노드는 직접 분할 불가")
    }

    // MARK: - closePane() 테스트

    /// closePane() 은 형제를 부모 위치로 승격한다.
    func test_closePane_promotesSimbling() async throws {
        await model.load()
        guard let rootId = model.rootId else { XCTFail(); return }
        // 분할: root → [childA, childB]
        let childB = model.splitActive(rootId, direction: .horizontal)!
        let children = model.nodes[rootId]!.childIds
        let childA = children.first(where: { $0 != childB })!

        // childB 닫기
        let closed = model.closePane(childB)

        // Assert
        XCTAssertTrue(closed)
        XCTAssertNil(model.nodes[childB], "닫힌 pane 이 제거되어야 함")
        XCTAssertNil(model.nodes[rootId], "중간 노드(기존 부모)가 제거되어야 함")
        // childA 가 새 루트여야 한다
        XCTAssertEqual(model.rootId, childA)
        XCTAssertNil(model.nodes[childA]?.parentId, "승격된 노드의 parentId 가 nil 이어야 함")
    }

    /// 마지막 pane 은 닫을 수 없다.
    func test_closePane_lastPane_returnsFalse() async throws {
        await model.load()
        guard let rootId = model.rootId else { XCTFail(); return }

        let closed = model.closePane(rootId)

        XCTAssertFalse(closed, "마지막 pane 은 닫을 수 없어야 함")
        XCTAssertNotNil(model.nodes[rootId], "마지막 pane 이 유지되어야 함")
    }

    // MARK: - updateRatio() 테스트

    /// updateRatio() 는 비율을 갱신하고 FFI 를 통해 영속한다.
    func test_updateRatio_persistsThroughBridge() async throws {
        await model.load()
        guard let rootId = model.rootId else { XCTFail(); return }
        model.splitActive(rootId, direction: .horizontal)

        model.updateRatio(rootId, ratio: 0.7)

        XCTAssertEqual(model.nodes[rootId]?.ratio ?? 0, 0.7, accuracy: 0.001)
        // mock 의 updatePaneRatio 가 호출되어 저장됐는지 확인
        XCTAssertEqual(mock.panes[rootId]?.ratio ?? 0, 0.7, accuracy: 0.001)
    }

    // MARK: - children() 테스트

    /// children(of:) 는 자식 노드 목록을 반환한다.
    func test_children_returnsChildNodes() async throws {
        await model.load()
        guard let rootId = model.rootId else { XCTFail(); return }
        model.splitActive(rootId, direction: .horizontal)

        let children = model.children(of: rootId)

        XCTAssertEqual(children.count, 2)
        XCTAssertTrue(children.allSatisfy { $0.split == .leaf })
    }

    /// leaf 의 children(of:) 는 빈 배열을 반환한다.
    func test_children_ofLeaf_returnsEmpty() async throws {
        await model.load()
        guard let rootId = model.rootId else { XCTFail(); return }

        let children = model.children(of: rootId)

        XCTAssertTrue(children.isEmpty)
    }
}
