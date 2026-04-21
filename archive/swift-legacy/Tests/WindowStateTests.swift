//
//  WindowStateTests.swift
//  T-022: UserDefaults 저장·복원 및 사이드바 너비 클램프 검증.
//

import XCTest
@testable import MoAIStudio

@MainActor
final class WindowStateTests: XCTestCase {

    private func makeDefaults() -> UserDefaults {
        let suiteName = "moai.studio.tests.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suiteName)!
        defaults.removePersistentDomain(forName: suiteName)
        return defaults
    }

    func testDefaultsWhenEmpty() {
        let defaults = makeDefaults()
        let store = WindowStateStore(defaults: defaults)

        XCTAssertEqual(store.windowWidth, WindowStateStore.defaultWindowWidth)
        XCTAssertEqual(store.windowHeight, WindowStateStore.defaultWindowHeight)
        XCTAssertEqual(store.sidebarWidth, WindowStateStore.defaultSidebarWidth)
    }

    func testPersistAndRestore() {
        let defaults = makeDefaults()
        let store1 = WindowStateStore(defaults: defaults)
        store1.setWindowSize(width: 1440, height: 900)
        store1.setSidebarWidth(320)
        store1.persist()

        let store2 = WindowStateStore(defaults: defaults)
        XCTAssertEqual(store2.windowWidth, 1440)
        XCTAssertEqual(store2.windowHeight, 900)
        XCTAssertEqual(store2.sidebarWidth, 320)
    }

    func testSidebarClampLower() {
        let defaults = makeDefaults()
        let store = WindowStateStore(defaults: defaults)
        store.setSidebarWidth(50) // below 200 floor
        XCTAssertEqual(store.sidebarWidth, WindowStateStore.sidebarMinWidth)
    }

    func testSidebarClampUpper() {
        let defaults = makeDefaults()
        let store = WindowStateStore(defaults: defaults)
        store.setSidebarWidth(999) // above 400 ceiling
        XCTAssertEqual(store.sidebarWidth, WindowStateStore.sidebarMaxWidth)
    }

    func testWindowMinimumSize() {
        let defaults = makeDefaults()
        let store = WindowStateStore(defaults: defaults)
        store.setWindowSize(width: 100, height: 100)
        XCTAssertEqual(store.windowWidth, WindowStateStore.minWindowWidth)
        XCTAssertEqual(store.windowHeight, WindowStateStore.minWindowHeight)
    }

    func testReset() {
        let defaults = makeDefaults()
        let store = WindowStateStore(defaults: defaults)
        store.setWindowSize(width: 2000, height: 1200)
        store.persist()
        store.reset()
        XCTAssertEqual(store.windowWidth, WindowStateStore.defaultWindowWidth)
        XCTAssertEqual(store.sidebarWidth, WindowStateStore.defaultSidebarWidth)
    }

    func testClampHelper() {
        XCTAssertEqual(WindowStateStore.clampSidebar(0), 200)
        XCTAssertEqual(WindowStateStore.clampSidebar(250), 250)
        XCTAssertEqual(WindowStateStore.clampSidebar(500), 400)
    }
}
