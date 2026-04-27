# SPEC-V3-002 Progress

**Started**: 2026-04-21 (SPEC v1.0.0 draft)
**Branch**: feature/SPEC-V3-002-terminal-core (squash merged into main via multiple PRs)
**SPEC status**: implemented (v1.1.0)
**Completion date**: 2026-04-23

## Implementation Timeline

- 2026-04-21 SPEC v1.0.0 created — Terminal Core fresh rewrite (plan-auditor iter 2 passed)
- 2026-04-21 Annotation Cycle iter 1 PASS — 4 items user-approved
- 2026-04-23 T4: TerminalSurface GPUI component + content_area branch — `fa24281`
- 2026-04-23 T5: Key input ANSI encoding + arboard clipboard integration — `a602264`
- 2026-04-23 T6: ghostty-spike GPUI window path + example registration — `3962f95`
- 2026-04-23 T1-T3/T7/T8: libghostty-vt + PTY + worker + test harness — `34cb052`
- 2026-04-23 T9: Zig buildchain + ghostty-spike smoke + pin-policy-guard — `3219361`
- 2026-04-23 Run Phase concluded — docs sync — `3545b8e`
- 2026-04-23 MX tag advisory review (W1-W4) — `c48f6d3`

## Task Status

- [x] T1: libghostty-rs dependency + Zig buildchain (AC-T-1, AC-T-2)
- [x] T2: Pty trait + UnixPty + Windows compile_error + trybuild (AC-T-10)
- [x] T3: PTY worker thread + adaptive buffer + PtyEvent channel (RG-V3-002-3)
- [x] T4: TerminalSurface GPUI component + content_area branch (AC-T-6)
- [x] T5: Key input ANSI encoding + arboard clipboard integration
- [x] T6: ghostty-spike GPUI window path + example registration
- [x] T7: Test harness — 14 new tests (AC-T-8 >= 10 satisfied)
- [x] T8: Criterion benches (AC-T-4, AC-T-9)
- [x] T9: CI — Zig buildchain + smoke test + pin-policy-guard (AC-T-7, AC-T-11)
- [x] MX tag review — 4 advisory warnings documented (W1-W4)

## Key Files Changed

- `crates/moai-studio-terminal/Cargo.toml`: libghostty-vt rev=dfac6f3e pinned, portable-pty, arboard, trybuild, criterion deps
- `crates/moai-studio-terminal/build.rs`: Zig 0.15.x check + error message
- `crates/moai-studio-terminal/src/libghostty_ffi.rs`: FFI wrapper boundary (@MX:ANCHOR)
- `crates/moai-studio-terminal/src/vt.rs`, `src/events.rs`: High-level Rust interface + PtyEvent enum
- `crates/moai-studio-terminal/src/pty/mod.rs`: Pty trait (@MX:ANCHOR)
- `crates/moai-studio-terminal/src/pty/unix.rs`: portable-pty based UnixPty with $SHELL fallback
- `crates/moai-studio-terminal/src/pty/windows.rs`: compile_error! enforcement
- `crates/moai-studio-terminal/src/pty/mock.rs`: MockPty for testing
- `crates/moai-studio-terminal/src/worker.rs`: 4KB-64KB adaptive buffer + tokio mpsc channel
- `crates/moai-studio-ui/src/terminal/mod.rs`: TerminalSurface (Render, pixel_to_cell, Selection)
- `crates/moai-studio-ui/src/terminal/clipboard.rs`: arboard local clipboard + SIGINT separation
- `crates/moai-studio-ui/src/terminal/input.rs`: keystroke_to_ansi_bytes + clipboard/sigint detection
- `crates/moai-studio-ui/examples/ghostty_spike.rs`: Headless spike example
- `.github/workflows/ci-rust.yml`: Zig setup + smoke job + pin-policy-guard job
- `scripts/ci/check-history-sha.sh`: SHA history validation for libghostty-vt pin bumps
- `scripts/ci/check-wrapper-scope.sh`: Wrapper scope change detection

## Test Coverage

- `tests/libghostty_api_compat.rs`: 4 characterization tests
- `tests/pty_contract.rs`: 4 Pty trait dyn safety + MockPty contract tests
- `tests/pty_fd_cleanup.rs`: 1 FD cleanup test (AC-T-5)
- `tests/worker_adaptive_buffer.rs`: 4 buffer transition + backpressure tests
- `tests/compile_fail.rs` + `tests/compile_fail/conpty_spawn.rs`: trybuild compile-fail
- 33 pixel_to_cell/Selection/TerminalSurface/RootView tests (T4)
- 60 total after T5 (ANSI encoding 21, clipboard 6, sigint 3)
- `bench_pty_burst_read.rs`: 1 MB burst p99 <= 5ms benchmark
- `bench_key_echo_latency.rs`: p99 key-echo <= 16ms benchmark

## Known Limitations

- Windows PTY: compile_error! stub only (not production-ready)
- Terminal rendering: content_area branch exists but full VT rendering pipeline deferred
- link detection: added later in SPEC-V3-LINK-001 (separate SPEC)
- ghostty-spike example: headless scaffold only, no interactive rendering
- Zig 0.15.x required at build time for terminal crate
