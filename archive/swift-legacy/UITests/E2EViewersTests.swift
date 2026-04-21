// E2EViewersTests.swift
// SPEC-M2-001 T-085: M2 E2E UITest 스켈레톤.
//
// @MX:NOTE: [AUTO] CI에서 CODE_SIGN_IDENTITY 미설정 시 스킵 (C-1 carry-over)
// @MX:NOTE: [AUTO] 전체 E2E는 Rust 측 e2e_viewers.rs 에서 프로그래밍 방식으로 검증됨

import XCTest

final class E2EViewersTests: XCTestCase {

    /// M2 전체 워크플로: pane split → tab → surface → command palette.
    ///
    /// CI 환경에서는 코드 서명 이슈(C-1 carry-over)로 스킵.
    /// 로컬에서 수동 검증 시 아래 단계를 순서대로 확인:
    ///
    /// 1. 앱 시작 → 워크스페이스 생성
    /// 2. Cmd+\ → 수평 pane 분할
    /// 3. Cmd+Shift+\ → 수직 pane 분할
    /// 4. FileTree surface 열기 → 파일 선택 → Markdown tab 열림 확인
    /// 5. Cmd+K → Command Palette 열기 → "terminal" 입력 → Enter
    /// 6. 새 Terminal tab 열림 확인
    /// 7. Cmd+Shift+W → pane 닫기
    func test_m2_full_workflow_pane_tab_palette() throws {
        try XCTSkipIf(
            ProcessInfo.processInfo.environment["CI"] == "true",
            "UITests는 코드 서명 필요 (C-1 carry-over) — CI에서 스킵"
        )
        // 이 테스트는 수동 실행 전용입니다.
        // 자동화된 E2E 검증은 core/crates/moai-ffi/tests/e2e_viewers.rs 참조.
        let app = XCUIApplication()
        app.launch()

        // 1. 앱이 시작되고 워크스페이스 화면이 표시됨
        XCTAssertTrue(app.windows.firstMatch.exists)
    }
}
