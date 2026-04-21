//
//  CommandPaletteTests.swift
//  Command Palette 모델 테스트 (SPEC-M2-001 MS-6 T-073).
//

import XCTest
@testable import MoAIStudio

// MARK: - CommandPaletteControllerTests

@MainActor
final class CommandPaletteControllerTests: XCTestCase {

    // MARK: - open / close

    func test_open_showsPalette_andClearsQuery() {
        // Arrange
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        let controller = CommandPaletteController(registry: registry)
        controller.query = "이전 쿼리"

        // Act
        controller.open()

        // Assert
        XCTAssertTrue(controller.isPresented)
        XCTAssertEqual(controller.query, "")
    }

    func test_close_hidesPalette() {
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        let controller = CommandPaletteController(registry: registry)
        controller.open()

        controller.close()

        XCTAssertFalse(controller.isPresented)
    }

    // MARK: - refreshResults

    func test_refreshResults_withEmptyQuery_returnsAllCommands() {
        // Arrange
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        let controller = CommandPaletteController(registry: registry)
        let totalCount = registry.commands.count

        // Act
        controller.query = ""
        controller.refreshResults()

        // Assert — 빈 쿼리이면 모든 명령어 반환
        XCTAssertEqual(controller.results.count, totalCount)
    }

    func test_refreshResults_withQuery_filtersCommands() {
        // Arrange
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        let controller = CommandPaletteController(registry: registry)

        // Act — 특정 쿼리로 필터링
        controller.query = "plan"
        controller.refreshResults()

        // Assert — 결과는 전체보다 적고, 모두 plan 관련 명령어여야 한다
        XCTAssertLessThan(controller.results.count, registry.commands.count)
        XCTAssertTrue(controller.results.count > 0, "plan 쿼리에 매칭되는 명령어가 존재해야 한다")
    }

    func test_refreshResults_withNoMatchingQuery_returnsEmpty() {
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        let controller = CommandPaletteController(registry: registry)

        controller.query = "zzzzXXXnoMatch9999"
        controller.refreshResults()

        XCTAssertEqual(controller.results.count, 0)
    }

    // MARK: - moveSelection

    func test_moveSelection_boundsCheck() {
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        let controller = CommandPaletteController(registry: registry)
        controller.query = ""
        controller.refreshResults()
        controller.selectedIndex = 0

        // 위로 이동 — 최소값 0 에서 클램프
        controller.moveSelection(-1)
        XCTAssertEqual(controller.selectedIndex, 0)

        // 마지막으로 이동
        let last = controller.results.count - 1
        controller.selectedIndex = last
        controller.moveSelection(1)
        XCTAssertEqual(controller.selectedIndex, last, "마지막 항목에서 아래로 이동 불가")
    }

    func test_moveSelection_down_incrementsIndex() {
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        let controller = CommandPaletteController(registry: registry)
        controller.query = ""
        controller.refreshResults()
        controller.selectedIndex = 0

        controller.moveSelection(1)
        XCTAssertEqual(controller.selectedIndex, 1)
    }

    // MARK: - execute

    func test_execute_callsCommandHandler_andClosesPalette() {
        var handlerCalled = false
        let cmd = PaletteCommand(
            id: "test.cmd",
            title: "Test Command",
            subtitle: nil,
            category: .moai,
            keywords: [],
            handler: { handlerCalled = true }
        )
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        registry.register(cmd)
        let controller = CommandPaletteController(registry: registry)
        controller.open()

        controller.execute(cmd)

        XCTAssertTrue(handlerCalled)
        XCTAssertFalse(controller.isPresented, "명령어 실행 후 팔레트가 닫혀야 한다")
    }

    func test_execute_recordsHistory() {
        let cmd = PaletteCommand(
            id: "test.history",
            title: "History Test",
            subtitle: nil,
            category: .surface,
            keywords: [],
            handler: {}
        )
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        registry.register(cmd)
        let controller = CommandPaletteController(registry: registry)

        controller.execute(cmd)
        controller.execute(cmd)  // 중복 실행

        // 최근 명령어 히스토리에 기록됨
        XCTAssertTrue(controller.historyIds.contains("test.history"))
        // 중복은 한 번만 기록
        let count = controller.historyIds.filter { $0 == "test.history" }.count
        XCTAssertEqual(count, 1, "동일 명령어는 히스토리에 한 번만 기록")
    }
}

// MARK: - FuzzyMatcherTests

@MainActor
final class FuzzyMatcherTests: XCTestCase {

    private func makeCmd(id: String, title: String, subtitle: String? = nil, keywords: [String] = []) -> PaletteCommand {
        PaletteCommand(id: id, title: title, subtitle: subtitle, category: .moai, keywords: keywords, handler: {})
    }

    func test_emptyQuery_returnsAllWithBaseScore() {
        let cmds = [makeCmd(id: "a", title: "Alpha"), makeCmd(id: "b", title: "Beta")]
        let results = FuzzyMatcher.match(query: "", commands: cmds)
        XCTAssertEqual(results.count, 2)
        for r in results {
            XCTAssertGreaterThan(r.score, 0)
        }
    }

    func test_exactMatch_scoreHighest() {
        let cmds = [
            makeCmd(id: "a", title: "plan"),
            makeCmd(id: "b", title: "unrelated xyz"),
        ]
        let results = FuzzyMatcher.match(query: "plan", commands: cmds)
        XCTAssertEqual(results.first?.command.id, "a", "정확 매칭이 상위에 있어야 한다")
    }

    func test_prefixMatch_getsBonusScore() {
        let cmds = [
            makeCmd(id: "prefix", title: "plan something"),
            makeCmd(id: "middle", title: "something plan"),
        ]
        let results = FuzzyMatcher.match(query: "plan", commands: cmds)
        let prefixResult = results.first { $0.command.id == "prefix" }
        let middleResult = results.first { $0.command.id == "middle" }
        XCTAssertNotNil(prefixResult)
        XCTAssertNotNil(middleResult)
        XCTAssertGreaterThan(prefixResult!.score, middleResult!.score, "접두사 매칭이 더 높은 점수")
    }

    func test_subsequenceMatch_works() {
        let cmds = [makeCmd(id: "a", title: "open file tree")]
        let results = FuzzyMatcher.match(query: "oft", commands: cmds)
        XCTAssertEqual(results.count, 1)
    }

    func test_noMatch_returnsNil() {
        let cmds = [makeCmd(id: "a", title: "terminal")]
        let results = FuzzyMatcher.match(query: "zzzxxx", commands: cmds)
        XCTAssertEqual(results.count, 0)
    }

    func test_sortedByScore_descending() {
        let cmds = [
            makeCmd(id: "exact", title: "plan"),
            makeCmd(id: "partial", title: "split pane"),
            makeCmd(id: "sub", title: "open plan file"),
        ]
        let results = FuzzyMatcher.match(query: "plan", commands: cmds)
        guard results.count >= 2 else { return }
        for i in 0..<results.count - 1 {
            XCTAssertGreaterThanOrEqual(results[i].score, results[i + 1].score)
        }
    }

    func test_keywordsContributeToMatch() {
        let cmdWithKeyword = makeCmd(id: "a", title: "some command", keywords: ["moai", "plan"])
        let cmdWithout = makeCmd(id: "b", title: "another command", keywords: [])
        let results = FuzzyMatcher.match(query: "plan", commands: [cmdWithKeyword, cmdWithout])
        XCTAssertTrue(results.contains { $0.command.id == "a" }, "키워드로 매칭되어야 한다")
    }
}

// MARK: - CommandRegistryTests

@MainActor
final class CommandRegistryTests: XCTestCase {

    func test_init_registersBuiltinCommands() {
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )

        XCTAssertFalse(registry.commands.isEmpty, "내장 명령어가 등록되어 있어야 한다")
    }

    func test_register_addsCustomCommand() {
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        let before = registry.commands.count
        let cmd = PaletteCommand(id: "custom.test", title: "Custom", subtitle: nil, category: .pane, keywords: [], handler: {})

        registry.register(cmd)

        XCTAssertEqual(registry.commands.count, before + 1)
        XCTAssertTrue(registry.commands.contains { $0.id == "custom.test" })
    }

    func test_unregister_removesCommand() {
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )
        let cmd = PaletteCommand(id: "remove.me", title: "Remove Me", subtitle: nil, category: .workspace, keywords: [], handler: {})
        registry.register(cmd)

        registry.unregister(id: "remove.me")

        XCTAssertFalse(registry.commands.contains { $0.id == "remove.me" })
    }

    func test_commandCategories_allPresent() {
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )

        let categories = Set(registry.commands.map { $0.category })
        XCTAssertTrue(categories.contains(.moai), "/moai 카테고리 명령어 필요")
        XCTAssertTrue(categories.contains(.surface), "Surface 카테고리 명령어 필요")
        XCTAssertTrue(categories.contains(.workspace), "Workspace 카테고리 명령어 필요")
        XCTAssertTrue(categories.contains(.pane), "Pane 카테고리 명령어 필요")
    }

    func test_moaiSlashCommands_containExpectedEntries() {
        let registry = CommandRegistry(
            onMoaiSlash: { _ in },
            onSurfaceOpen: { _ in },
            onWorkspaceCreate: {},
            onPaneSplit: { _ in }
        )

        let moaiCmds = registry.commands.filter { $0.category == .moai }
        let ids = moaiCmds.map { $0.id }
        // 핵심 /moai 슬래시 명령어들이 등록되어야 한다
        XCTAssertTrue(ids.contains("moai.plan"), "/moai plan 명령어 필요")
        XCTAssertTrue(ids.contains("moai.run"), "/moai run 명령어 필요")
        XCTAssertTrue(ids.contains("moai.sync"), "/moai sync 명령어 필요")
    }
}
