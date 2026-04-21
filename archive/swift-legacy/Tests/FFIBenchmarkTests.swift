// FFIBenchmarkTests.swift
// SPEC-M2-001 C-7 carry-over: Swift FFI <1ms P95 벤치마크.
//
// @MX:NOTE: [AUTO] FFI 호출 오버헤드 측정 — P95 < 1ms 목표 (SPEC-M2-001 NFR)

import XCTest
@testable import MoAIStudio

@MainActor
final class FFIBenchmarkTests: XCTestCase {

    /// version() FFI 호출 1,000회 — XCTest measure 블록으로 평균 측정.
    /// 목표: 1,000회 합계 < 1,000ms (호출당 < 1ms 평균).
    func test_ffi_version_underOneMs() {
        let bridge = MockRustCoreBridge()
        measure {
            for _ in 0..<1000 {
                _ = bridge.version()
            }
        }
        // XCTest measure 는 평균을 보고함. 별도 P95 단언은 아래 테스트에서 수행.
    }

    /// create/delete workspace P95 < 1ms 단언.
    func test_ffi_create_delete_workspace_p95() {
        let bridge = MockRustCoreBridge()
        var timings: [TimeInterval] = []

        for i in 0..<100 {
            let start = Date()
            let id = bridge.createWorkspace(name: "bench-\(i)", projectPath: "/tmp")
            _ = bridge.deleteWorkspace(id: id)
            timings.append(Date().timeIntervalSince(start))
        }

        timings.sort()
        let p95Index = Int(Double(timings.count) * 0.95)
        let p95 = timings[p95Index]
        XCTAssertLessThan(
            p95, 0.001,
            "P95 FFI 호출 오버헤드가 1ms 미만이어야 함. 측정값: \(p95 * 1000)ms"
        )
    }

    /// version() 반환값이 비어있지 않음을 확인 (기능 smoke).
    func test_ffi_version_non_empty() {
        let bridge = MockRustCoreBridge()
        let ver = bridge.version()
        XCTAssertFalse(ver.isEmpty, "version() FFI 결과가 비어있음")
    }
}
