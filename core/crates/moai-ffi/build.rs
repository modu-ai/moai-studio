// moai-ffi 빌드 스크립트
//
// swift-bridge-build는 src/lib.rs에서 #[swift_bridge::bridge] 모듈을 파싱해
// Swift 바인딩 코드(.swift, .h)를 생성한다.
//
// M0에서는 수동 C FFI를 사용하므로 파싱할 bridge 모듈이 없다.
// 파일 목록이 비어 있으면 swift-bridge-build는 아무것도 생성하지 않는다.
fn main() {
    // M0: 파싱 대상 파일 목록이 비어 있음 — bridge 모듈 없음
    // M1+에서 swift-bridge로 전환 시 "src/lib.rs" 추가
    swift_bridge_build::parse_bridges(Vec::<&str>::new());
}
