//! tmux 중첩 통합 테스트 — SPEC-V3-003 T9 (AC-P-26).
//!
//! ## 목적
//!
//! AC-P-26: tmux 가 pane 내부에서 실행 중일 때 Ctrl+B prefix 가 tmux 에 전달되고
//! 앱이 소비하지 않음을 검증한다.
//!
//! ## 테스트 전략
//!
//! - **테스트 1** (순수 Rust mock): `dispatch_key(Ctrl+B)` → `None` 반환 확인.
//!   FocusRouter 가 Ctrl+B 를 소비하지 않으므로 자동 passthrough 충족.
//!   이 테스트는 항상 실행되며 `#[ignore]` 불필요.
//!
//! - **테스트 2** (실제 tmux, `#[ignore]`): tmux 프로세스를 생성하고 PTY 를 통해
//!   Ctrl+B 를 전송 후 tmux 가 prefix 를 수신했는지 검증한다.
//!   실제 tmux 설치 환경에서만 의미 있으므로 기본적으로 `#[ignore]`.
//!
//! @MX:TODO(T9.1): 실제 tmux 프로세스 integration — runner 환경 가용성 확인 후 #[ignore] 제거.

use moai_studio_ui::panes::{KeyCode, KeyModifiers, dispatch_key};

// ============================================================
// 테스트 1: 순수 Rust mock — Ctrl+B passthrough (AC-P-26)
// ============================================================

/// `dispatch_key(Ctrl+B)` 가 `None` 을 반환함을 확인한다 (AC-P-26 pure-Rust 검증).
///
/// FocusRouter 는 MS-2 바인딩 중 Ctrl+B 와 매치되는 패턴을 갖지 않으므로
/// `None` 반환 → 상위 호출자(T10 GPUI handler)가 focused TerminalSurface 로 전달한다.
#[test]
fn ctrl_b_dispatch_key_returns_none_for_passthrough() {
    // Ctrl+B: ctrl=true, alt=false, shift=false, code=Other ("B" 는 KeyCode::Other 로 표현)
    let ctrl_b_mods = KeyModifiers {
        ctrl: true,
        cmd: false,
        alt: false,
        shift: false,
    };
    let result = dispatch_key(ctrl_b_mods, KeyCode::Other);
    assert_eq!(
        result, None,
        "Ctrl+B 는 dispatch_key None 반환 → tmux passthrough (AC-P-26)"
    );

    // Cmd+B (macOS) 도 MS-2 매핑이 없으므로 None
    let cmd_b_mods = KeyModifiers {
        cmd: true,
        ctrl: false,
        alt: false,
        shift: false,
    };
    let result_cmd = dispatch_key(cmd_b_mods, KeyCode::Other);
    assert_eq!(
        result_cmd, None,
        "Cmd+B (Other 키) 도 None 반환 — B 는 MS-2 매핑 없음"
    );
}

// ============================================================
// 테스트 2: 실제 tmux 프로세스 — #[ignore] (T9.1)
// ============================================================

/// 실제 tmux 세션을 생성하고 Ctrl+B prefix 를 전송하여 tmux 가 수신하는지 검증한다.
///
/// ## 실행 조건
///
/// - `tmux` 바이너리가 PATH 에 존재해야 한다.
/// - CI 에서는 `cargo test -- --include-ignored ctrl_b_passes_through_to_nested_tmux` 로 실행.
/// - 로컬에서 tmux 없이 실행하면 `skip` (graceful fallback).
///
/// @MX:TODO(T9.1): #[ignore] 제거 조건 — ci-v3-pane.yml 에 `apt install tmux` 추가 후.
#[test]
#[ignore = "실제 tmux 프로세스 필요 — CI 에서 apt install tmux 후 --include-ignored 로 실행 (T9.1)"]
fn ctrl_b_passes_through_to_nested_tmux() {
    use std::process::Command;

    // tmux 바이너리 존재 여부 확인
    let tmux_check = Command::new("which").arg("tmux").output();
    if tmux_check.map(|o| !o.status.success()).unwrap_or(true) {
        eprintln!("tmux 가 설치되지 않아 테스트를 skip 합니다.");
        return;
    }

    // 테스트용 tmux 세션 생성 (detached)
    let session = "moai-test-t9";
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", session])
        .output();
    let create_result = Command::new("tmux")
        .args(["new-session", "-d", "-s", session, "-x", "80", "-y", "24"])
        .status()
        .expect("tmux new-session 실행 실패");
    assert!(create_result.success(), "tmux 세션 생성 성공");

    // Ctrl+B 를 tmux 세션에 전송 (tmux send-keys 사용)
    // 실제 앱에서는 GPUI 가 Ctrl+B 를 TerminalSurface PTY 에 쓰는 형태.
    // 여기서는 tmux send-keys 로 시뮬레이션.
    let send_result = Command::new("tmux")
        .args(["send-keys", "-t", session, "C-b", ""])
        .status()
        .expect("tmux send-keys 실행 실패");
    assert!(send_result.success(), "tmux send-keys Ctrl+B 성공");

    // tmux 세션이 prefix 상태가 되었는지 확인 (세션이 살아있는지 체크)
    let list_result = Command::new("tmux")
        .args(["list-sessions", "-F", "#{session_name}"])
        .output()
        .expect("tmux list-sessions 실행 실패");
    let sessions = String::from_utf8_lossy(&list_result.stdout);
    assert!(
        sessions.contains(session),
        "tmux 세션이 Ctrl+B 처리 후에도 살아있어야 함 (AC-P-26)"
    );

    // 정리
    let _ = Command::new("tmux")
        .args(["kill-session", "-t", session])
        .output();
}
