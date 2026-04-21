//
//  BrowserViewModelTests.swift
//  Browser Surface ViewModel 테스트 (SPEC-M2-001 MS-5 T-066).
//

import XCTest
@testable import MoAIStudio

@MainActor
final class BrowserViewModelTests: XCTestCase {

    // MARK: - 초기 상태

    func test_init_currentURLEmpty() {
        let vm = BrowserViewModel()
        XCTAssertEqual(vm.currentURL, "")
    }

    func test_init_canGoBackFalse() {
        let vm = BrowserViewModel()
        XCTAssertFalse(vm.canGoBack)
    }

    func test_init_canGoForwardFalse() {
        let vm = BrowserViewModel()
        XCTAssertFalse(vm.canGoForward)
    }

    func test_init_isLoadingFalse() {
        let vm = BrowserViewModel()
        XCTAssertFalse(vm.isLoading)
    }

    // MARK: - load()

    func test_load_updatesCurrentURL() {
        // Arrange
        let vm = BrowserViewModel()

        // Act
        vm.load("https://example.com")

        // Assert
        XCTAssertEqual(vm.currentURL, "https://example.com")
    }

    func test_load_localhostURL_setsCurrentURL() {
        let vm = BrowserViewModel()
        vm.load("http://localhost:3000")
        XCTAssertEqual(vm.currentURL, "http://localhost:3000")
    }

    // MARK: - goBack / goForward (상태 반영)

    func test_setCanGoBack_true_canGoBack() {
        let vm = BrowserViewModel()
        vm.setNavigationState(canGoBack: true, canGoForward: false)
        XCTAssertTrue(vm.canGoBack)
    }

    func test_setCanGoForward_true_canGoForward() {
        let vm = BrowserViewModel()
        vm.setNavigationState(canGoBack: false, canGoForward: true)
        XCTAssertTrue(vm.canGoForward)
    }

    func test_setLoading_true_isLoading() {
        let vm = BrowserViewModel()
        vm.setLoading(true)
        XCTAssertTrue(vm.isLoading)
    }

    func test_setLoading_false_notLoading() {
        let vm = BrowserViewModel()
        vm.setLoading(true)
        vm.setLoading(false)
        XCTAssertFalse(vm.isLoading)
    }

    // MARK: - URL 정규화

    func test_load_emptyURL_doesNotCrash() {
        let vm = BrowserViewModel()
        vm.load("")
        // 크래시 없이 완료
    }
}
