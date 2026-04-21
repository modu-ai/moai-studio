# SPEC-V3-002 (Compact) — Terminal Core

---
id: SPEC-V3-002
version: 0.3.1-draft
source: spec.md v0.3.1-draft (iter 2 revision)
generated: 2026-04-21
purpose: Run phase token-optimized load (~30% saving vs full spec.md)
revision: iter 2 — plan-auditor D1 (canonical layout) + D2 (AC-T-11) 반영
---

## EARS 요구사항 (5 모듈)

### RG-V3-002-1: libghostty-rs 통합 + pin 정책 + CI

- [Ubiquitous] `moai-studio-terminal` crate 는 libghostty-rs SHA `dfac6f3e8e7ff1567a7dead6639ef36c42e4f15a` (2026-04-20) 를 Cargo `rev =` 의존성으로 고정한다.
- [Ubiquitous, pin 정책] bump 시 요건 4개: (a) 월 1회 이상 수동, (b) `tests/libghostty_api_compat.rs` 재실행 첨부, (c) HISTORY 에 이전/신규 SHA 기록, (d) wrapper 외 파일 변경 시 annotation cycle 재개.
- [Event-Driven] `cargo build -p moai-studio-terminal` 시 Zig 0.15.x 로 libghostty-vt 빌드 + FFI 심볼 (`Terminal::new`, `feed`, `render_state`, `KeyEncoder`, `MouseEncoder`) 노출.
- [Unwanted] Zig 미설치 시 build 시작 금지, stderr `"Zig 0.15.x required — install via mise/asdf/ziglang.org"` + exit 1.
- [Ubiquitous] 모든 FFI 호출은 `src/libghostty_ffi.rs` 외부에서 직접 참조 금지 (iter 2 D5 재분류).
- [Ubiquitous, CI] `.github/workflows/ci-rust.yml` matrix 에 `mlugg/setup-zig@v2.2.1` + `actions/cache@v4` (key `zig-0.15.2-${{ runner.os }}-${{ runner.arch }}`) 포함 (macOS + Linux).
- [Event-Driven, CI] PR 시 `cargo run --example ghostty-spike -- --headless` 스모크 실행.
- [State-Driven, CI] Zig 캐시 hit 시 CI wall-clock ≤ 5분.

### RG-V3-002-2: PTY + Shell spawn (cross-platform trait)

- [Ubiquitous] `pty` 모듈은 `Pty` trait 정의: `feed(&[u8])`, `read_available() -> Vec<u8>`, `set_window_size(rows, cols)`, `is_alive() -> bool`.
- [Ubiquitous] macOS/Linux: portable-pty 0.9+ 기반 `UnixPty` 실 구현.
- [Ubiquitous] Windows: `#[cfg(windows)] ConPtyStub` — `spawn()`/`read_available()` 호출 사이트를 `compile_error!("ConPtyStub is deferred to Phase 7 (GPUI Windows GA)")` 단일 enforcement 로 차단 (iter 2 D4 — `todo!()` 제거). trybuild compile-fail 테스트로 gate 작동 검증.
- [Event-Driven] 새 terminal surface 요청 시 `$SHELL` (fallback `/bin/zsh` macOS, `/bin/bash` Linux) spawn + stdin/stdout/stderr PTY master 바인딩.
- [State-Driven] PTY alive 동안 stdout/stderr → libghostty-vt parser 실시간 전달.
- [Unwanted] shell crash/exit 시 1초 이내 PTY FD 정리 (고아 FD 금지, `lsof` 검증).
- [Ubiquitous] Windows target 에서 실 shell spawn 호출은 `compile_error!` 로 상시 차단 (iter 2 D5 재분류).

### RG-V3-002-3: PTY worker thread + PtyEvent (적응형 buffer)

- [Ubiquitous] `worker` 모듈은 전용 PTY worker thread 제공 (PTY + Parser 단일 worker, Render 는 GPUI main thread).
- [Ubiquitous, 적응형 buffer] 기본 4KB / 10ms tick, 연속 3 tick 포화 시 64KB 전환, 2 tick 반 미만 시 4KB 복귀.
- [Event-Driven] stdout 바이트 도착 → `Terminal::feed()` → `RenderState` delta 를 `PtyEvent::Output` 으로 mpsc send.
- [Event-Driven] shell exit 시 `PtyEvent::ProcessExit(i32)` emit.
- [Event-Driven] resize 시 `PtyEvent::Resize { rows, cols }` 수신 + PTY propagate.
- [Unwanted] channel backpressure 로 worker blocking PTY read 가 drop 되지 않아야 함 (`mpsc::unbounded_channel` 또는 bounded+awaiting sender, `tokio::task::block_in_place`).

### RG-V3-002-4: GPUI 렌더 + 입력 + 로컬 클립보드

- [Ubiquitous] `TerminalSurface` 는 libghostty-vt `RenderState` (Grid<Cell>) → Glyph 렌더.
- [Event-Driven] `PtyEvent` 도착 시 grid snapshot 갱신 + `cx.notify()` → partial re-render.
- [State-Driven] active workspace + TerminalSurface 존재 시 content_area 는 Empty State 대신 TerminalSurface 표시.
- [Event-Driven, 키 입력] 포커스 시 GPUI key event → libghostty-vt `KeyEncoder` → ANSI escape → PTY stdin write.
- [Event-Driven, 로컬 복사] macOS `Cmd+C` / Linux `Ctrl+Shift+C` → arboard 3.0 로 로컬 클립보드 복사. SIGINT (`Ctrl+C`) 와 구분.
- [Ubiquitous] 원격 OSC 52 sequence 는 parser 가 상시 silently ignore (Phase 3 로 이관, iter 2 D5 재분류).
- [Optional] 커서 blink 설정 시 GPUI timer 기반 blink 제공.

### RG-V3-002-5: Mouse 선택 + Grid 매핑 + 예제 바이너리

- [Ubiquitous] `pixel_to_cell(x: f32, y: f32) -> (row: u16, col: u16)` 매핑 함수 제공 (font advance_width, line_height 기반).
- [Event-Driven] mouse drag 시 시작/현재 셀 사각형 선택 영역 반투명 하이라이트 렌더.
- [Ubiquitous] `examples/ghostty-spike.rs` 는 `--headless` 플래그 지원: GPUI 윈도우 없이 PTY spawn + `echo "Scaffold OK"` + stdout 검증 + exit 0.
- [State-Driven] 비-`--headless` 모드는 GPUI 윈도우 + TerminalSurface + shell prompt 표시.

---

## Acceptance Criteria (11개, iter 2 에서 AC-T-11 추가)

| AC | Given | When | Then |
|----|-------|------|------|
| AC-T-1 | libghostty-rs `dfac6f3e` pin + Zig 0.15.x 설치, linux-x86_64 cold runner, **debug profile** | `cargo build -p moai-studio-terminal` (no `--release`) | 성공, warning 0, 빌드 ≤ **150초** (cold, iter 2 D6/D7: research §4 Zig 60-120s + Rust dep compile 근거) / ≤ 30초 (Swatinem/rust-cache hit) |
| AC-T-2 | Zig PATH 제거 환경 | `cargo build -p moai-studio-terminal` | exit 1 + stderr `"Zig 0.15.x required — install via mise/asdf/ziglang.org"` |
| AC-T-3 | PTY spawn 가능 환경, `$SHELL=/bin/zsh` | `cargo run --example ghostty-spike` | GPUI 윈도우, spawn → prompt 첫 렌더 ≤ 200ms (criterion), `$SHELL` prompt 정규식 match |
| AC-T-4 | 스파이크 실행 중 + TerminalSurface 포커스 | 키 입력 `echo hello\n` | Grid 에 `hello` 렌더, p99 key-echo latency ≤ 16ms (criterion, 60fps budget) |
| AC-T-5 | 스파이크 실행 중 | shell `exit` 또는 `kill -HUP <shell_pid>` | 1초 이내 윈도우 종료 + PTY FD 정리, `lsof -p <parent_pid> \| grep -c ptmx` delta = 0 |
| AC-T-6 | RootView + active workspace | 사용자 TerminalSurface 생성 트리거 | content_area 에 TerminalSurface 렌더, 첫 셀부터 시작 |
| AC-T-7 | CI PR 트리거, Zig cache hit | GitHub Actions (macOS + Linux matrix) | Zig install + build + `--headless` 스모크 통과, 4 job 중 최장 wall-clock ≤ 5분 |
| AC-T-8 | `moai-studio-terminal` 테스트 하네스 | `cargo test -p moai-studio-terminal` | 신규 테스트 ≥ 10, 포함: (a) `MockPty` fd clone+script, (b) libghostty-vt Grid assertion (cell iter, UTF-8 multibyte, CSI), (c) adaptive buffer 전환 단위, (d) `Pty` trait contract |
| AC-T-9 | 대용량 stdout (`head -c 1M /dev/urandom \| base64`) | TerminalSurface 가 PtyEvent 연속 소비 | **1 MB byte count (1,048,576 bytes) 가 Grid state 에 byte-level 반영** (scrollback line count 아님, iter 2 D3), p99 PTY read cycle ≤ 5ms (criterion `bench_pty_burst_read`), adaptive buffer 64KB ↔ 4KB 전환을 tracing log 로 확인 |
| AC-T-10 | Windows target (on-demand, CI matrix 미포함) | `cargo check --target x86_64-pc-windows-msvc -p moai-studio-terminal` | `Pty` trait 컴파일 통과, `UnixPty` 는 `#[cfg(unix)]` 제외, `ConPtyStub::spawn()` 호출 사이트는 **`compile_error!("ConPtyStub is deferred to Phase 7 (GPUI Windows GA)")` 단일 enforcement** (iter 2 D4 — `todo!()` 제거) + trybuild compile-fail 테스트 (`tests/compile_fail/conpty_spawn.rs`) 가 메시지 assert. 실빌드는 의도된 실패 (Phase 7 deferred) |
| **AC-T-11** (iter 2 신설) | `Cargo.toml` 의 libghostty-rs `rev =` 라인이 변경되는 PR | CI `pin-policy-guard` job 실행 | (i) `cargo test --test libghostty_api_compat` PASS, (ii) spec.md HISTORY diff 에 old+new SHA 둘 다 포함 (`check-history-sha.sh`), (iii) wrapper 외 파일 수정 시 `annotation-cycle-required` 라벨 부착 + warning (`check-wrapper-scope.sh`). (i) 또는 (ii) 실패 시 merge 차단 |

---

## 주요 변경 파일 (canonical layout — spec.md §9 참조)

### `crates/moai-studio-terminal/`
- `Cargo.toml` — libghostty-rs dep, portable-pty, arboard, tokio, `[dev-dependencies] trybuild = "1"`
- `build.rs` — (필요 시) Zig toolchain 검증
- `src/lib.rs` — 모듈 선언
- `src/libghostty_ffi.rs` — FFI wrapper 단일 파일 (upstream churn 격리점, wrapper-external 금지)
- `src/vt.rs` — 고수준 Rust 인터페이스
- `src/events.rs` — `PtyEvent` enum 정의
- `src/worker.rs` — PTY worker thread + adaptive buffer (4KB ↔ 64KB)
- `src/pty/mod.rs` — `Pty` trait 정의
- `src/pty/unix.rs` — portable-pty 기반 `UnixPty` (macOS/Linux)
- `src/pty/windows.rs` — `#[cfg(windows)] ConPtyStub` + `compile_error!` 단일 enforcement
- `examples/ghostty-spike.rs` — `--headless` 지원 예제
- `tests/libghostty_api_compat.rs` — characterization (AC-T-11 에서 pin bump 시 재실행)
- `tests/pty_contract.rs` — Pty trait contract
- `tests/worker_adaptive_buffer.rs` — adaptive buffer 단위
- `tests/pty_fd_cleanup.rs` — AC-T-5 FD 누수 검증
- `tests/compile_fail.rs` + `tests/compile_fail/conpty_spawn.rs` — trybuild compile-fail (AC-T-10)
- `benches/bench_pty_burst_read.rs` — AC-T-9 criterion
- `benches/bench_key_echo_latency.rs` — AC-T-4 criterion

### `crates/moai-studio-ui/`
- `src/terminal/mod.rs` — `TerminalSurface` GPUI 컴포넌트 (렌더 본체)
- `src/terminal/input.rs` — 키 이벤트 → ANSI encoding
- `src/terminal/clipboard.rs` — arboard 로컬 복사 + Selection
- `src/root_view.rs` — content_area 분기 (TerminalSurface vs Empty State)

### CI + 가드 스크립트
- `.github/workflows/ci-rust.yml` — Zig setup + cache + smoke job + **pin-policy-guard job (iter 2 신설, AC-T-11)**
- `scripts/ci/check-history-sha.sh` — AC-T-11 (ii) HISTORY SHA 포함 검증
- `scripts/ci/check-wrapper-scope.sh` — AC-T-11 (iii) wrapper 외 변경 시 `annotation-cycle-required` 라벨

---

## Exclusions (§6)

- OSC 52 원격 클립보드 → Phase 3 (SPEC-V3-003 Smart Links)
- Windows ConPTY 실 구현 → Phase 7 (GPUI Windows GA 대기). 본 SPEC 은 trait 추상화 + compile-gate 만
- Shell configuration loader — OS 에 위임
- Terminal scrollback UI — Phase 2.5
- Tab / Pane split — SPEC-V3-003
- SSH / remote terminal — Phase 3+
- Windows 실 빌드 실행 — Phase 7
- Font fallback, Ligature, BiDi — Phase 4 (Surfaces) 이후

---

## 전제 의존성

- SPEC-V3-001 Phase 1.8 완료 (RootView + workspace 4영역)
- SPEC-V3-001 CI matrix (run 24708460052 ALL GREEN)
- libghostty-rs `dfac6f3e8e7ff1567a7dead6639ef36c42e4f15a`
- portable-pty 0.9.x, arboard 3.0, Zig 0.15.2, tokio 1.x
