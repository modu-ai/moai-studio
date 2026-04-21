#ifndef MOAI_FFI_H
#define MOAI_FFI_H

// MoAI Core FFI — Rust extern "C" 함수 선언
// moai-ffi 크레이트에서 #[unsafe(no_mangle)] 로 노출

/// MoAI Studio 버전 문자열 반환.
/// 호출자가 moai_version_free() 로 반드시 해제해야 한다.
char* moai_version(void);

/// moai_version() 반환 포인터 해제. NULL 안전.
void moai_version_free(char* ptr);

#endif /* MOAI_FFI_H */
