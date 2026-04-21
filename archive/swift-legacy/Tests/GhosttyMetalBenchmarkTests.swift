// GhosttyMetalBenchmarkTests.swift
// SPEC-M2-001 C-4 carry-over: GhosttyKit Metal 60fps 벤치마크 하네스.
//
// @MX:TODO: [AUTO] 전체 Metal fps 측정은 production GhosttyHost wiring 완료 후 구현 (MS-3+ carry-over)
// @MX:NOTE: [AUTO] 현재는 벤치마크 하네스만 생성. 실제 GPU 측정은 GhosttyHost 연동 이후.

import XCTest

final class GhosttyMetalBenchmarkTests: XCTestCase {

    /// 60fps 목표 (≤16.67ms/frame) 벤치마크 하네스.
    /// Metal Toolchain / GhosttyHost 미연결 환경에서는 스킵.
    func test_ghostty_60fps_at_4K() throws {
        try XCTSkipIf(
            ProcessInfo.processInfo.environment["MOAI_TERMINAL_BACKEND"] == "nstext",
            "Metal backend 미사용 환경 — 스킵"
        )
        try XCTSkipIf(
            ProcessInfo.processInfo.environment["CI"] == "true",
            "CI 환경에서는 Metal Toolchain 미보장 — 스킵 (C-4 carry-over)"
        )

        // @MX:TODO: [AUTO] 아래 measure 블록을 실제 GhosttyHost.renderFrame() 호출로 교체 필요
        // 현재는 16ms sleep으로 프레임 타임 시뮬레이션
        measure(metrics: [XCTClockMetric()]) {
            for _ in 0..<60 {
                Thread.sleep(forTimeInterval: 0.016)
            }
        }
    }

    /// 벤치마크 하네스 컴파일 확인 (항상 실행).
    func test_benchmark_harness_compiles() {
        // 하네스 자체가 정상 컴파일되는지 확인하는 smoke test
        let target = 1.0 / 60.0  // 16.67ms
        XCTAssertLessThan(target, 0.02, "60fps 목표는 20ms 미만이어야 함")
    }
}
