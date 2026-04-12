//! moai-ffi: Rust ↔ Swift FFI 경계 정의
//!
//! 이 크레이트는 Swift UI와 Rust Core 사이의 유일한 FFI 경계다.
//! M0에서는 수동 C FFI로 구현한다. M1+에서 swift-bridge로 전환 예정.

// @MX:NOTE: [AUTO] M0 FFI 표면 — version, free만 노출. start_workspace/send_user_message는 M1에서 추가

use std::ffi::CString;
use std::os::raw::c_char;

/// MoAI Studio 버전 문자열을 C 문자열로 반환
///
/// # Safety
/// 반환된 포인터는 `moai_version_free`로 해제해야 한다.
/// Swift에서는 `defer { moai_version_free(ptr) }` 패턴을 사용한다.
// @MX:ANCHOR: Swift → Rust 버전 조회 진입점
// @MX:REASON: [AUTO] Swift UnsafePointer<CChar>로 직접 호출되는 C ABI 함수
#[unsafe(no_mangle)]
pub extern "C" fn moai_version() -> *mut c_char {
    let version = moai_core::version();
    // CString 변환 실패는 Rust 내부 버그이므로 unwrap 허용
    CString::new(version).unwrap().into_raw()
}

/// `moai_version`이 반환한 C 문자열을 해제
///
/// # Safety
/// `s`는 반드시 `moai_version()`이 반환한 포인터여야 한다.
/// null 포인터는 안전하게 무시된다.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn moai_version_free(s: *mut c_char) {
    // null 포인터 방어
    if s.is_null() {
        return;
    }
    // SAFETY: moai_version()이 CString::into_raw()로 생성한 포인터임
    unsafe {
        drop(CString::from_raw(s));
    }
}

/// 버전 문자열의 Rust 네이티브 접근 함수 (테스트 및 내부 사용)
pub fn ffi_version() -> String {
    moai_core::version()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    // RED→GREEN: FFI moai_version()이 null이 아닌 유효한 버전 문자열을 반환해야 함
    #[test]
    fn test_ffi_version_returns_non_null() {
        let ptr = moai_version();
        assert!(!ptr.is_null(), "moai_version()이 null 포인터를 반환함");
        // 안전한 해제
        unsafe { moai_version_free(ptr) };
    }

    // RED→GREEN: FFI moai_version()이 moai_core::version()과 동일한 값을 반환해야 함
    #[test]
    fn test_ffi_version_matches_core_version() {
        let ptr = moai_version();
        assert!(!ptr.is_null());

        // SAFETY: ptr은 방금 생성된 유효한 CString 포인터
        // to_string()으로 값을 먼저 복사한 뒤 포인터를 해제한다
        let ffi_ver = unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .expect("유효한 UTF-8이 아님")
            .to_string();

        unsafe { moai_version_free(ptr) };

        assert_eq!(ffi_ver, moai_core::version(), "FFI 버전이 core 버전과 불일치");
    }

    // RED→GREEN: FFI moai_version()이 CARGO_PKG_VERSION과 일치해야 함
    #[test]
    fn test_ffi_version_matches_cargo_pkg_version() {
        let ver = ffi_version();
        assert_eq!(ver, env!("CARGO_PKG_VERSION"));
    }

    // RED→GREEN: moai_version_free(null)은 패닉 없이 안전하게 처리되어야 함
    #[test]
    fn test_ffi_version_free_null_is_safe() {
        // null 포인터 해제가 패닉을 일으키지 않아야 함
        unsafe { moai_version_free(std::ptr::null_mut()) };
    }
}
