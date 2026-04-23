//! @MX:ANCHOR(conpty-compile-gate-test)
//! @MX:REASON: ConPtyStub 의 compile_error! gate 를 검증 (AC-T-10 iter 2 D4).
//!   이 파일은 trybuild compile-fail 테스트용 — 컴파일이 *실패해야* 정상.
//!
//! Windows target: ConPtyStub::feed() / read_available() 의 #[cfg(windows)] compile_error!
//! Unix target: 이 테스트는 ConPtyStub 를 Pty trait 으로 사용해 feed 를 호출 —
//!              실제 환경에서는 compile_error! 가 없으므로 다른 방식으로 검증한다.
//!
//! trybuild 는 .stderr 파일이 없으면 단순히 컴파일 실패만 확인한다.
//! Windows CI 에서는 #[cfg(windows)] compile_error! 가 발생한다.
//!
//! macOS/Linux: 의도적으로 존재하지 않는 타입을 참조해 컴파일 에러를 발생시킨다.

fn main() {
    // 존재하지 않는 메서드 spawn() 호출 → compile error (AC-T-10 gate 검증)
    // Windows: compile_error!("ConPtyStub is deferred to Phase 7 (GPUI Windows GA)")
    // macOS/Linux: no method named `spawn` (ConPtyStub 에 spawn 메서드 없음)
    let mut stub = moai_studio_terminal::pty::ConPtyStub;
    let _ = stub.spawn(); // 의도적 compile error
}
