//
//  StatusIconTests.swift
//  T-023: WorkspaceStatus 파싱 & 아이콘 case exhaustiveness.
//

import XCTest
@testable import MoAIStudio

final class StatusIconTests: XCTestCase {
    func testStatusFromRustStringHandlesAllCases() {
        XCTAssertEqual(WorkspaceStatus(rawString: "Running"), .running)
        XCTAssertEqual(WorkspaceStatus(rawString: "running"), .running)
        XCTAssertEqual(WorkspaceStatus(rawString: "Starting"), .starting)
        XCTAssertEqual(WorkspaceStatus(rawString: "Error"), .error)
        XCTAssertEqual(WorkspaceStatus(rawString: "Paused"), .paused)
        XCTAssertEqual(WorkspaceStatus(rawString: "Created"), .created)
        XCTAssertEqual(WorkspaceStatus(rawString: "Deleted"), .deleted)
    }

    func testUnknownStatusDefaultsToCreated() {
        XCTAssertEqual(WorkspaceStatus(rawString: "Garbage"), .created)
    }

    func testAllCasesCovered() {
        XCTAssertEqual(WorkspaceStatus.allCases.count, 6)
    }
}
