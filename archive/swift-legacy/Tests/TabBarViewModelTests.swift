//
//  TabBarViewModelTests.swift
//  TabBarViewModel 단위 테스트 (SPEC-M2-001 MS-3 T-049).
//
//  MockRustCoreBridge 로 FFI 없이 순수 로직을 검증한다.
//

import XCTest
@testable import MoAIStudio

@MainActor
final class TabBarViewModelTests: XCTestCase {
    // MARK: - 픽스처

    private var mock: MockRustCoreBridge!
    private var vm: TabBarViewModel!
    private let testPaneId: Int64 = 42

    override func setUp() async throws {
        mock = MockRustCoreBridge()
        vm = TabBarViewModel(paneId: testPaneId, bridge: mock)
    }

    // MARK: - T-049: load() 테스트

    /// load() 는 FFI 에서 surface 목록을 읽어 tabs 를 채운다.
    func test_load_populatesTabsFromFFI() async throws {
        // Arrange: mock 에 surface 2개 미리 삽입
        _ = mock.createSurface(paneId: testPaneId, kind: "terminal", stateJson: "", tabOrder: 0)
        _ = mock.createSurface(paneId: testPaneId, kind: "markdown", stateJson: "", tabOrder: 1)

        // Act
        await vm.load()

        // Assert
        XCTAssertEqual(vm.tabs.count, 2)
        XCTAssertEqual(vm.tabs[0].kind, .terminal)
        XCTAssertEqual(vm.tabs[1].kind, .markdown)
        XCTAssertEqual(vm.tabs[0].tabOrder, 0)
        XCTAssertEqual(vm.tabs[1].tabOrder, 1)
    }

    /// surface 가 없을 때 load() 는 기본 Terminal 탭을 자동 생성한다.
    func test_load_withNoSurfaces_autoCreatesDefaultTerminalTab() async throws {
        // Arrange: 빈 상태

        // Act
        await vm.load()

        // Assert: 탭이 1개 생성되어야 하고 kind == terminal 이어야 한다
        XCTAssertEqual(vm.tabs.count, 1, "기본 탭 1개가 자동 생성되어야 함")
        XCTAssertEqual(vm.tabs.first?.kind, .terminal, "기본 탭은 terminal 이어야 함")
        // DB 에도 저장되었는지 확인
        XCTAssertEqual(mock.surfaces.values.filter { $0.paneId == testPaneId }.count, 1)
    }

    // MARK: - T-049: newTab() 테스트

    /// newTab() 은 탭을 추가하고 증분 tab_order 를 할당한다.
    func test_newTab_addsTabWithIncrementalTabOrder() async throws {
        // Arrange
        await vm.load() // 기본 terminal 탭 1개 (order=0)

        // Act
        let newId = vm.newTab(kind: .markdown)

        // Assert
        XCTAssertNotNil(newId, "새 탭 id 가 반환되어야 함")
        XCTAssertEqual(vm.tabs.count, 2)
        let newTab = vm.tabs.first { $0.id == newId }
        XCTAssertNotNil(newTab)
        XCTAssertEqual(newTab?.kind, .markdown)
        XCTAssertEqual(newTab?.tabOrder, 1, "두 번째 탭의 tabOrder 는 1 이어야 함")
    }

    /// newTab() 은 여러 번 호출 시 tabOrder 가 순차적으로 증가한다.
    func test_newTab_multipleCallsIncrementTabOrder() async throws {
        // Arrange
        await vm.load() // order 0

        // Act
        _ = vm.newTab(kind: .terminal) // order 1
        _ = vm.newTab(kind: .browser)  // order 2

        // Assert
        XCTAssertEqual(vm.tabs.count, 3)
        let orders = vm.tabs.map { $0.tabOrder }.sorted()
        XCTAssertEqual(orders, [0, 1, 2])
    }

    // MARK: - T-049: closeTab() 테스트

    /// closeTab() 은 탭을 제거하고 true 를 반환한다.
    func test_closeTab_removesTabAndReturnsTrue() async throws {
        // Arrange: 탭 2개
        await vm.load()
        let secondId = vm.newTab(kind: .markdown)!

        // Act
        let result = vm.closeTab(secondId)

        // Assert
        XCTAssertTrue(result)
        XCTAssertEqual(vm.tabs.count, 1)
        XCTAssertNil(vm.tabs.first { $0.id == secondId })
    }

    /// closeTab() 은 마지막 탭이면 false 를 반환하고 탭을 유지한다.
    func test_closeTab_lastTab_returnsFalse() async throws {
        // Arrange: 탭 1개 (기본)
        await vm.load()
        let onlyTabId = vm.tabs[0].id

        // Act
        let result = vm.closeTab(onlyTabId)

        // Assert
        XCTAssertFalse(result, "마지막 탭은 닫을 수 없어야 함")
        XCTAssertEqual(vm.tabs.count, 1, "탭이 유지되어야 함")
    }

    // MARK: - T-049: reorder() 테스트

    /// reorder() 는 탭 순서를 변경하고 tab_order 를 DB 에 반영한다.
    func test_reorder_updatesTabOrderCorrectly() async throws {
        // Arrange: [terminal(0), markdown(1), browser(2)]
        await vm.load() // terminal 0
        _ = vm.newTab(kind: .markdown) // 1
        _ = vm.newTab(kind: .browser)  // 2

        XCTAssertEqual(vm.tabs.count, 3)

        // Act: index 2 (browser) → index 0
        vm.reorder(from: 2, to: 0)

        // Assert: [browser, terminal, markdown]
        XCTAssertEqual(vm.tabs[0].kind, .browser)
        XCTAssertEqual(vm.tabs[1].kind, .terminal)
        XCTAssertEqual(vm.tabs[2].kind, .markdown)

        // tab_order 값도 갱신되어야 한다 (0, 1, 2 로 재할당)
        XCTAssertEqual(vm.tabs[0].tabOrder, 0)
        XCTAssertEqual(vm.tabs[1].tabOrder, 1)
        XCTAssertEqual(vm.tabs[2].tabOrder, 2)
    }

    // MARK: - T-049: selectTab() 테스트

    /// selectTab() 은 activeTabId 를 업데이트한다.
    func test_selectTab_updatesActiveTabId() async throws {
        // Arrange: 탭 2개
        await vm.load()
        let secondId = vm.newTab(kind: .markdown)!

        // Act
        vm.selectTab(secondId)

        // Assert
        XCTAssertEqual(vm.activeTabId, secondId)
    }

    // MARK: - T-049: activeTabKind() 테스트

    /// activeTabKind() 는 활성 탭의 SurfaceKind 를 반환한다.
    func test_activeTabKind_returnsKindOfActiveTab() async throws {
        // Arrange
        await vm.load()
        let markdownId = vm.newTab(kind: .markdown)!
        vm.selectTab(markdownId)

        // Act
        let kind = vm.activeTabKind()

        // Assert
        XCTAssertEqual(kind, .markdown)
    }

    /// 활성 탭이 없으면 activeTabKind() 는 nil 을 반환한다.
    func test_activeTabKind_noActiveTab_returnsNil() async throws {
        // Arrange: 탭 없고 activeTabId 도 없는 상태
        vm = TabBarViewModel(paneId: 999, bridge: mock)
        // load() 를 호출하지 않으면 tabs 가 비어있음

        // Act
        let kind = vm.activeTabKind()

        // Assert
        XCTAssertNil(kind)
    }
}
