// @MX:WARN(zig-toolchain-precondition)
// @MX:REASON: Zig 0.15.x 미설치 시 전체 moai-studio-terminal 빌드가 차단된다.
//             libghostty-vt 의 build.rs 는 Zig 로 C 소스를 컴파일하므로
//             이 crate 의 build.rs 에서 선제 검증하여 명확한 에러를 제공한다.
//             CI: mlugg/setup-zig@v2.2.1 + actions/cache@v4 로 해결 (SPEC-V3-002 AC-T-7).

use std::process;

fn main() {
    // Zig 설치 여부 확인 (libghostty-vt 빌드 전제)
    if let Err(e) = check_zig() {
        eprintln!("{}", e);
        process::exit(1);
    }

    // cargo 가 libghostty-vt build.rs 를 실행하기 전에 통과함을 보장
    println!("cargo:rerun-if-env-changed=PATH");
    println!("cargo:rerun-if-env-changed=GHOSTTY_SOURCE_DIR");
}

/// Zig 0.15.x 설치 여부를 검증한다.
///
/// 반환:
/// - Ok(()) — Zig 가 PATH 에 있고 버전이 0.15.x
/// - Err(String) — 표준 에러 메시지 (AC-T-2 규격)
pub fn check_zig() -> Result<(), String> {
    let output = std::process::Command::new("zig").arg("version").output();

    match output {
        Err(_) => Err("Zig 0.15.x required — install via mise/asdf/ziglang.org".to_string()),
        Ok(out) => {
            let version = String::from_utf8_lossy(&out.stdout);
            let version = version.trim();
            if !version.starts_with("0.15") {
                Err(format!(
                    "Zig 0.15.x required — install via mise/asdf/ziglang.org (found: {})",
                    version
                ))
            } else {
                Ok(())
            }
        }
    }
}
