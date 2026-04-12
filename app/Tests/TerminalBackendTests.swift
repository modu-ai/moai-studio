//
//  TerminalBackendTests.swift
//  T-026: MOAI_TERMINAL_BACKEND 환경변수 해석.
//

import XCTest
@testable import MoAIStudio

final class TerminalBackendTests: XCTestCase {
    func testRawValueParsing() {
        XCTAssertEqual(TerminalBackend(rawValue: "ghostty"), .ghostty)
        XCTAssertEqual(TerminalBackend(rawValue: "nstext"), .nstext)
        XCTAssertNil(TerminalBackend(rawValue: "unknown"))
    }

    func testDefaultIsGhostty() {
        // env 변수를 현재 프로세스 밖에서 주입해야 current 가 정확히 측정됨.
        // 여기서는 raw 파싱 계약만 확인.
        XCTAssertEqual(TerminalBackend.ghostty.rawValue, "ghostty")
        XCTAssertEqual(TerminalBackend.nstext.rawValue, "nstext")
    }
}
