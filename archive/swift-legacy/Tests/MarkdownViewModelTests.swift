//
//  MarkdownViewModelTests.swift
//  Markdown Surface ViewModel 테스트 (SPEC-M2-001 MS-5 T-066).
//

import XCTest
@testable import MoAIStudio

@MainActor
final class MarkdownViewModelTests: XCTestCase {

    // MARK: - 임시 파일 헬퍼

    private func makeTempMarkdownFile(content: String) throws -> String {
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString + ".md")
        try content.write(to: url, atomically: true, encoding: .utf8)
        return url.path
    }

    private func cleanup(path: String) {
        try? FileManager.default.removeItem(atPath: path)
    }

    // MARK: - load()

    func test_load_setsSource() async throws {
        // Arrange
        let content = "# Hello\n\nWorld"
        let path = try makeTempMarkdownFile(content: content)
        defer { cleanup(path: path) }
        let vm = MarkdownViewModel(filePath: path)

        // Act
        try await vm.load()

        // Assert
        XCTAssertEqual(vm.source, content)
    }

    func test_load_nonexistentFile_throwsError() async {
        // Arrange
        let vm = MarkdownViewModel(filePath: "/nonexistent/path/file.md")

        // Act & Assert
        do {
            try await vm.load()
            XCTFail("load() 는 파일이 없으면 throw 해야 한다")
        } catch {
            // 예상된 에러
        }
    }

    func test_load_populatesRenderedHTML() async throws {
        // Arrange
        let path = try makeTempMarkdownFile(content: "# Title")
        defer { cleanup(path: path) }
        let vm = MarkdownViewModel(filePath: path)

        // Act
        try await vm.load()

        // Assert — renderedHTML 이 비어 있지 않아야 한다
        XCTAssertFalse(vm.renderedHTML.isEmpty)
    }

    // MARK: - reload()

    func test_reload_updatesSourceAfterFileChange() async throws {
        // Arrange
        let path = try makeTempMarkdownFile(content: "# Original")
        defer { cleanup(path: path) }
        let vm = MarkdownViewModel(filePath: path)
        try await vm.load()
        XCTAssertEqual(vm.source, "# Original")

        // 파일 변경
        try "# Updated".write(toFile: path, atomically: true, encoding: .utf8)

        // Act
        await vm.reload()

        // Assert
        XCTAssertEqual(vm.source, "# Updated")
    }

    // MARK: - filePath

    func test_init_storesFilePath() {
        // Arrange & Act
        let vm = MarkdownViewModel(filePath: "/some/path.md")

        // Assert
        XCTAssertEqual(vm.filePath, "/some/path.md")
    }

    // MARK: - isDarkMode

    func test_isDarkMode_defaultFalse() {
        let vm = MarkdownViewModel(filePath: "/a.md")
        XCTAssertFalse(vm.isDarkMode)
    }

    func test_isDarkMode_canBeSet() {
        let vm = MarkdownViewModel(filePath: "/a.md")
        vm.isDarkMode = true
        XCTAssertTrue(vm.isDarkMode)
    }
}
