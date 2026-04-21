import MoaiCore

// M0 마일스톤: Rust FFI 브릿지 동작 검증 진입점
// SwiftUI/AppKit 없이 CLI로 FFI 호출을 증명한다.

let core = RustCore()
let version = core.version()

print("MoAI Studio v\(version)")
print("Rust FFI bridge: OK")
