//! trybuild compile-fail 테스트 하네스
//!
//! Windows target 에서 ConPtyStub::spawn() / read_available() 호출이
//! compile_error! 로 차단되는지 검증 (AC-T-10).
//!
//! 주의: trybuild 는 cargo test 에서만 실행됨 (cargo check 는 제외).
//! Windows MSVC cross-compile target 없는 환경에서는 SKIP.

#[test]
fn conpty_compile_error_gate() {
    // compile_fail/ 디렉터리의 .rs 파일들이 예상된 에러로 실패하는지 확인
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/conpty_spawn.rs");
}
