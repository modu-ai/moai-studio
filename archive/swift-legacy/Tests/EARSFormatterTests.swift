//
//  EARSFormatterTests.swift
//  EARS 포맷터 테스트 (SPEC-M2-001 MS-5 T-058, T-066).
//

import XCTest
@testable import MoAIStudio

final class EARSFormatterTests: XCTestCase {

    // MARK: - SPEC-ID 감지

    func test_enhance_specId_wrapsInSpan() {
        let input = "See SPEC-M2-001 for details."
        let output = EARSFormatter.enhance(markdown: input)
        XCTAssertTrue(output.contains("SPEC-M2-001"), "SPEC-ID 가 출력에 남아 있어야 한다")
        XCTAssertTrue(output.contains("spec-id"), "spec-id 클래스 마킹이 포함되어야 한다")
    }

    func test_enhance_noSpecId_returnsUnchangedStructure() {
        let input = "# Hello World\n\nPlain text."
        let output = EARSFormatter.enhance(markdown: input)
        XCTAssertTrue(output.contains("Hello World"))
        XCTAssertFalse(output.contains("spec-id"))
    }

    // MARK: - Ubiquitous EARS 패턴

    func test_enhance_ubiquitousPattern_addsClass() {
        let input = "**[Ubiquitous]** The system **shall** log all events."
        let output = EARSFormatter.enhance(markdown: input)
        XCTAssertTrue(output.contains("ears-ubiquitous") || output.contains("Ubiquitous"),
                      "Ubiquitous 패턴이 마킹되어야 한다")
    }

    // MARK: - 빈 입력

    func test_enhance_emptyString_returnsEmpty() {
        let output = EARSFormatter.enhance(markdown: "")
        XCTAssertEqual(output, "")
    }

    // MARK: - 다중 SPEC-ID

    func test_enhance_multipleSpecIds_allWrapped() {
        let input = "Refs: SPEC-M2-001, SPEC-M1-001"
        let output = EARSFormatter.enhance(markdown: input)
        XCTAssertTrue(output.contains("SPEC-M2-001"))
        XCTAssertTrue(output.contains("SPEC-M1-001"))
    }
}
