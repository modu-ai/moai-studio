//! @MX:NOTE(example-smoke-entrypoint)
//! ghostty-spike 예제 바이너리 — CI 스모크 진입점.
//!
//! --headless 모드:
//!   GPUI 윈도우 없이 PTY spawn → `echo "Scaffold OK"` 실행 → stdout 검증 → exit 0.
//!   AC-T-3, AC-T-7 대응.
//!
//! 비-headless 모드:
//!   GPUI 윈도우 + TerminalSurface stub (TODO: terminal-ui teammate 가 구현).

use moai_studio_terminal::pty::{Pty, UnixPty};
use std::io::{self, Write};
use std::process;
use std::time::{Duration, Instant};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let headless = args.iter().any(|a| a == "--headless");

    if headless {
        run_headless();
    } else {
        run_with_ui();
    }
}

/// --headless 모드: PTY spawn + echo + stdout 검증 (AC-T-3, AC-T-7)
fn run_headless() {
    eprintln!("[ghostty-spike] headless 모드 시작");

    // PTY spawn — $SHELL fallback
    let mut pty = match UnixPty::spawn_shell() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ghostty-spike] PTY spawn 실패: {}", e);
            process::exit(1);
        }
    };

    // echo "Scaffold OK" 실행
    if let Err(e) = pty.feed(b"echo 'Scaffold OK'\n") {
        eprintln!("[ghostty-spike] feed 실패: {}", e);
        process::exit(1);
    }
    if let Err(e) = pty.feed(b"exit\n") {
        eprintln!("[ghostty-spike] exit feed 실패: {}", e);
        process::exit(1);
    }
    io::stdout().flush().ok();

    // stdout 에서 "Scaffold OK" 확인
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut output = String::new();

    loop {
        match pty.read_available() {
            Ok(bytes) if !bytes.is_empty() => {
                let chunk = String::from_utf8_lossy(&bytes);
                output.push_str(&chunk);
                if output.contains("Scaffold OK") {
                    eprintln!("[ghostty-spike] stdout 검증 성공: 'Scaffold OK' 확인");
                    process::exit(0);
                }
            }
            Ok(_) => {
                if !pty.is_alive() {
                    break;
                }
                if Instant::now() >= deadline {
                    eprintln!("[ghostty-spike] 타임아웃 — 'Scaffold OK' 미수신");
                    process::exit(1);
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                eprintln!("[ghostty-spike] read 오류: {}", e);
                break;
            }
        }
    }

    // PTY 종료 후 버퍼 재확인
    if output.contains("Scaffold OK") {
        eprintln!("[ghostty-spike] stdout 검증 성공 (종료 후)");
        process::exit(0);
    }

    eprintln!(
        "[ghostty-spike] 실패 — 'Scaffold OK' 미수신. 출력:\n{}",
        output
    );
    process::exit(1);
}

/// 비-headless 모드: GPUI 윈도우 + TerminalSurface (Phase 2 구현 중)
fn run_with_ui() {
    // TODO: terminal-ui teammate (T5, T6) 가 TerminalSurface + GPUI 윈도우를 구현 예정.
    //       현재는 headless 경로로 위임.
    eprintln!("[ghostty-spike] GPUI UI 모드는 terminal-ui teammate 가 구현 예정 (T5/T6).");
    eprintln!("[ghostty-spike] --headless 플래그를 사용하세요.");
    process::exit(0);
}
