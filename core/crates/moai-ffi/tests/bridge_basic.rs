//! T-002 RED→GREEN: swift-bridge 기반 RustCore opaque type 기본 동작
//!
//! Swift 바인딩이 없어도 Rust 측에서 RustCore::new() / version() 을 직접 호출해
//! 동작을 검증한다. 실제 Swift → Rust 호출은 bridge 매크로가 생성한 C ABI
//! 심볼(`__swift_bridge__$RustCore$new` 등)을 통해 이루어지므로, 여기서는
//! Rust 수준의 계약만 확인한다.

use moai_ffi::RustCore;

#[test]
fn rust_core_new_returns_handle() {
    // 스모크: 생성이 패닉 없이 성공한다.
    let _core = RustCore::new();
}

#[test]
fn rust_core_version_matches_moai_core() {
    let core = RustCore::new();
    assert_eq!(core.version(), moai_core::version());
}

#[test]
fn rust_core_version_matches_cargo_pkg_version() {
    let core = RustCore::new();
    assert_eq!(core.version(), env!("CARGO_PKG_VERSION"));
}
