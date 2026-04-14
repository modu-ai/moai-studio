//
//  FileTreeViewModelTests.swift
//  FileTreeViewModel 단위 테스트 (SPEC-M2-001 MS-4 T-056).
//

import XCTest
@testable import MoAIStudio

@MainActor
final class FileTreeViewModelTests: XCTestCase {

    // MARK: - 헬퍼

    private func makeVM(directoryJson: String = "[]", statusJson: String = "{}") -> FileTreeViewModel {
        let bridge = MockRustCoreBridge()
        bridge.stubbedDirectoryJson = directoryJson
        bridge.stubbedStatusJson = statusJson
        return FileTreeViewModel(workspacePath: "/tmp/test", bridge: bridge)
    }

    // MARK: - T-056-S1: 초기 로드

    func test_load_populatesEntries() async {
        // Arrange
        let json = """
        [{"path":"src","name":"src","is_directory":true,"git_status":"clean","depth":0},
         {"path":"readme.md","name":"readme.md","is_directory":false,"git_status":"clean","depth":0}]
        """
        let vm = makeVM(directoryJson: json)

        // Act
        await vm.load()

        // Assert
        XCTAssertEqual(vm.entries.count, 2)
        XCTAssertEqual(vm.entries[0].name, "src")
        XCTAssertTrue(vm.entries[0].isDirectory)
        XCTAssertEqual(vm.entries[1].name, "readme.md")
        XCTAssertFalse(vm.entries[1].isDirectory)
    }

    // MARK: - T-056-S2: 디렉토리 expand/collapse

    func test_toggle_directory_addsToExpandedPaths() {
        // Arrange
        let vm = makeVM()

        // Act
        vm.toggle(path: "src")

        // Assert
        XCTAssertTrue(vm.expandedPaths.contains("src"))
    }

    func test_toggle_again_removesFromExpandedPaths() {
        // Arrange
        let vm = makeVM()
        vm.toggle(path: "src")

        // Act
        vm.toggle(path: "src")

        // Assert
        XCTAssertFalse(vm.expandedPaths.contains("src"))
    }

    // MARK: - T-056-S3: git status 매핑

    func test_gitStatus_mapsFromJSON() async {
        // Arrange
        let statusJson = """
        {"src/main.rs":"modified","new_file.txt":"untracked","gone.txt":"deleted"}
        """
        let vm = makeVM(statusJson: statusJson)

        // Act
        await vm.load()

        // Assert
        XCTAssertEqual(vm.gitStatusMap["src/main.rs"], .modified)
        XCTAssertEqual(vm.gitStatusMap["new_file.txt"], .untracked)
        XCTAssertEqual(vm.gitStatusMap["gone.txt"], .deleted)
    }

    // MARK: - T-056-S4: 갱신

    func test_refresh_updatesEntries() async {
        // Arrange
        let vm = makeVM(directoryJson: "[]")
        await vm.load()
        XCTAssertEqual(vm.entries.count, 0)

        // Act: 새 JSON 으로 업데이트 후 refresh
        let bridge = vm.bridgeForTest as! MockRustCoreBridge
        bridge.stubbedDirectoryJson = """
        [{"path":"new.txt","name":"new.txt","is_directory":false,"git_status":"clean","depth":0}]
        """
        await vm.refresh()

        // Assert
        XCTAssertEqual(vm.entries.count, 1)
        XCTAssertEqual(vm.entries[0].name, "new.txt")
    }

    // MARK: - T-056-S5: 빈 디렉토리

    func test_load_emptyDirectory_producesEmptyEntries() async {
        let vm = makeVM(directoryJson: "[]")
        await vm.load()
        XCTAssertTrue(vm.entries.isEmpty)
    }

    // MARK: - T-056-S6: depth

    func test_load_entryDepthReflectedFromJSON() async {
        let json = """
        [{"path":"src/main.rs","name":"main.rs","is_directory":false,"git_status":"clean","depth":1}]
        """
        let vm = makeVM(directoryJson: json)
        await vm.load()

        XCTAssertEqual(vm.entries.first?.depth, 1)
    }
}
