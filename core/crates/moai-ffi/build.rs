// moai-ffi 빌드 스크립트
//
// swift-bridge-build는 src/lib.rs의 `#[swift_bridge::bridge] mod ffi { ... }`
// 블록을 파싱해 Swift 바인딩(.swift)과 C 헤더(.h)를 생성한다.
//
// 생성 파일 위치: core/crates/moai-ffi/generated/
//   - SwiftBridgeCore.swift, SwiftBridgeCore.h (런타임 헬퍼)
//   - moai-ffi/moai-ffi.swift, moai-ffi/moai-ffi.h (우리 bridge 모듈)
//
// Swift 측에서는 SPM/Xcode가 이 경로를 include path로 잡아 링크한다.

use std::path::PathBuf;

// @MX:NOTE: [AUTO] bridge 선언 위치가 바뀌면 이 목록도 확장해야 함
fn main() {
    let bridges = vec!["src/lib.rs"];

    for path in &bridges {
        println!("cargo:rerun-if-changed={}", path);
    }
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("generated");

    swift_bridge_build::parse_bridges(bridges)
        .write_all_concatenated(out_dir, env!("CARGO_PKG_NAME"));
}
