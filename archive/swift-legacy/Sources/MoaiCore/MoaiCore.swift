import MoaiCoreFFI

// @MX:NOTE: [AUTO] Rust Core FFI Swift 래퍼 — M0 브릿지 검증용
// @MX:ANCHOR: Swift 코드가 Rust Core를 호출하는 유일한 진입점
// @MX:REASON: [AUTO] 모든 Rust 호출을 이 struct로 집중시켜 FFI 경계를 단일화

/// MoAI Studio Rust Core 에 대한 Swift 래퍼.
/// C FFI 포인터 수명 관리를 캡슐화한다.
public struct RustCore: Sendable {
    public init() {}

    /// Rust core 버전 문자열을 반환한다.
    /// 내부적으로 C 포인터를 defer 패턴으로 안전하게 해제한다.
    public func version() -> String {
        // moai_version()은 CString::into_raw()로 생성된 포인터를 반환
        guard let ptr = moai_version() else { return "unknown" }
        defer { moai_version_free(ptr) }
        return String(cString: ptr)
    }
}
