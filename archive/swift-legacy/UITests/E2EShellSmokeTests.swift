//
//  E2EShellSmokeTests.swift
//  MS-6 에서 확장될 UI smoke 테스트 자리잡이. 현재는 build 가능성만 확인.
//

import XCTest

final class E2EShellSmokeTests: XCTestCase {
    func testAppLaunchesWithoutCrash() throws {
        // MS-6 에서 XCUIApplication 기반 E2E 시나리오로 확장 예정.
        // 현재는 UITest 타겟 빌드 자체를 검증.
        let app = XCUIApplication()
        app.launchEnvironment["MOAI_TERMINAL_BACKEND"] = "nstext"
        // 실제 launch 는 MS-6 에서 활성화 — CI 에서는 app bundle 준비되어야 함.
        XCTAssertNotNil(app)
    }
}
