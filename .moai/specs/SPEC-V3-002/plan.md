# SPEC-V3-002 Plan — Terminal Core Implementation Plan

---
spec_id: SPEC-V3-002
version: 0.2.0-draft
created: 2026-04-21
updated: 2026-04-21
author: MoAI (manager-spec)
language: Korean
revision: iter 2 (plan-auditor iter 1 defect 반영 — D1 canonical layout, D2 T9 pin-policy guard, D4 trybuild, D8 T8 그래프 정합)
---

## 1. 개요

SPEC-V3-002 의 11 개 Acceptance Criteria (iter 2 에서 AC-T-11 pin-policy gate 추가) 를 만족시키기 위한 작업 분해 (T1~T10) 및 기술 스택, 위험, MX tag 계획. 시간 추정은 사용하지 않고 **우선순위 (High/Medium/Low) + 의존성 그래프** 로 표현한다.

파일 레이아웃은 **spec.md §9 File layout (canonical)** 을 단일 진실원으로 따른다. 본 plan 의 각 task 대상 파일은 해당 §9 트리와 line-for-line 일치한다 (iter 1 D1 cross-document divergence 해결).

본 plan 은 `quality.yaml` 의 `development_mode` (ddd | tdd) 와 무관하게 공통 적용되는 task decomposition 이다. Run phase 에서 manager-ddd 또는 manager-tdd 가 동일 task 목록을 방법론별 cycle 로 풀어낸다.

---

## 2. 기술 스택 (확정)

| 영역 | 선택 | 근거 |
|------|------|------|
| VT parser | `libghostty-vt` (via libghostty-rs `rev=dfac6f3e`) | research.md §1 |
| PTY | `portable-pty = "0.9"` | WezTerm 검증, cross-platform |
| 비동기 | tokio 1.x + `task::block_in_place` | portable-pty blocking API bridge |
| 채널 | `tokio::sync::mpsc::unbounded_channel` | backpressure drop 방지 (RG-V3-002-3) |
| 클립보드 | `arboard = "3.0"` | macOS NSPasteboard + Linux X11/Wayland + Windows (Phase 7) 통합 |
| UI | `gpui = "0.2.2"` (SPEC-V3-001 계승) | Zed terminal pattern |
| 빌드 툴체인 | Rust stable 1.82+ + Zig 0.15.2 | libghostty-vt 빌드 요구 |
| CI setup-zig | `mlugg/setup-zig@v2.2.1` + `actions/cache@v4` | research.md §4 |
| 로깅 | `tracing` (SPEC-V3-001 계승) | adaptive buffer 전환 추적 |

---

## 3. Task 분해 (T1~T10)

### T1: libghostty-rs 의존성 + Zig 빌드체인

**Priority**: High (모든 후속 task 의 블로커)
**선행 의존성**: 없음
**대상 파일**:
- `crates/moai-studio-terminal/Cargo.toml` (신규 dep 추가)
- `crates/moai-studio-terminal/build.rs` (Zig 검증 + libghostty-vt 빌드 호출)
- `crates/moai-studio-terminal/src/libghostty_ffi.rs` (신규, FFI wrapper 모듈)
- `crates/moai-studio-terminal/src/vt.rs` (신규, 고수준 Rust 인터페이스)

**예상 산출**:
- `libghostty-vt = { git = "https://github.com/Uzaaft/libghostty-rs", rev = "dfac6f3e8e7ff1567a7dead6639ef36c42e4f15a" }`
- build.rs 가 `which zig` 실패 시 `"Zig 0.15.x required — install via mise/asdf/ziglang.org"` 출력 + exit 1
- `libghostty_ffi::{Terminal, KeyEncoder, MouseEncoder, RenderState}` re-export
- AC-T-1, AC-T-2 대응

**참조 구현**: research.md §1 (라인 52~75, Cargo.toml pattern + wrapper isolation)

**MX tag 계획**:
- `src/libghostty_ffi.rs` 전체를 `@MX:ANCHOR(libghostty-ffi-boundary)` — FFI 경계, upstream churn 격리점
- build.rs 의 Zig 검증 로직에 `@MX:WARN(zig-toolchain-precondition)` — 환경 검증 실패 시 전체 워크스페이스 빌드 차단

### T2: `Pty` trait + UnixPty 구현 + Windows compile_error! gate + trybuild

**Priority**: High
**선행 의존성**: T1
**대상 파일** (spec.md §9 canonical layout 기준):
- `crates/moai-studio-terminal/src/pty/mod.rs` (신규, trait 정의)
- `crates/moai-studio-terminal/src/pty/unix.rs` (신규, portable-pty 래핑)
- `crates/moai-studio-terminal/src/pty/windows.rs` (신규, `#[cfg(windows)] compile_error!` stub)
- `crates/moai-studio-terminal/tests/compile_fail/conpty_spawn.rs` (신규, trybuild compile-fail 케이스)
- `crates/moai-studio-terminal/tests/compile_fail.rs` (신규, trybuild harness entry)
- `crates/moai-studio-terminal/Cargo.toml` (`[dev-dependencies] trybuild = "1"` 추가)

**예상 산출**:
```rust
// src/pty/mod.rs (요지)
pub trait Pty: Send + Sync {
    fn feed(&mut self, buf: &[u8]) -> io::Result<()>;
    fn read_available(&mut self) -> io::Result<Vec<u8>>;
    fn set_window_size(&mut self, rows: u16, cols: u16) -> io::Result<()>;
    fn is_alive(&self) -> bool;
}

#[cfg(unix)]
pub use unix::UnixPty;

#[cfg(windows)]
pub use windows::ConPtyStub;

// src/pty/windows.rs (요지, iter 2 D4 단일 enforcement)
pub struct ConPtyStub;

#[cfg(windows)]
impl Pty for ConPtyStub {
    fn feed(&mut self, _buf: &[u8]) -> io::Result<()> {
        compile_error!("ConPtyStub is deferred to Phase 7 (GPUI Windows GA)");
    }
    fn read_available(&mut self) -> io::Result<Vec<u8>> {
        compile_error!("ConPtyStub is deferred to Phase 7 (GPUI Windows GA)");
    }
    // set_window_size, is_alive 도 동일
}
```
- `UnixPty::spawn($SHELL fallback zsh/bash)` 구현
- `ConPtyStub` 의 모든 Pty trait 메서드는 `compile_error!` 단일 enforcement (iter 1 D4: `todo!()` 제거)
- `trybuild` compile-fail 테스트가 해당 에러 메시지를 stderr 에서 assert
- AC-T-10 대응 (`cargo check --target x86_64-pc-windows-msvc` 통과 + `cargo test --test compile_fail`)

**참조 구현**: research.md §2.3 (라인 132~151, PtyMaster trait 초안)

**MX tag 계획**:
- `pty/mod.rs` trait 정의: `@MX:ANCHOR(pty-trait-contract)` — cross-platform 계약, fan_in ≥ 3 예상 (UnixPty, ConPtyStub, worker, mock)
- `pty/windows.rs::ConPtyStub`: `@MX:TODO(conpty-phase-7)` with `@MX:REASON(compile-gate-deferred)` — Phase 7 작업 표시 + gate 단일 enforcement 사유
- `tests/compile_fail/conpty_spawn.rs`: `@MX:ANCHOR(conpty-compile-gate-test)` — iter 1 D4 해결 지점

### T3: PTY worker thread + PtyEvent + 적응형 buffer

**Priority**: High
**선행 의존성**: T2
**대상 파일**:
- `crates/moai-studio-terminal/src/worker.rs` (신규)
- `crates/moai-studio-terminal/src/events.rs` (신규, `PtyEvent` enum)

**예상 산출**:
```rust
// src/events.rs
pub enum PtyEvent {
    Output(Vec<u8>),
    ProcessExit(i32),
    Resize { rows: u16, cols: u16 },
}

// src/worker.rs
pub struct PtyWorker { /* buffer_size: AtomicUsize, burst_tick_count: ... */ }

impl PtyWorker {
    pub async fn run(mut self, pty: Box<dyn Pty>, tx: UnboundedSender<PtyEvent>) {
        // 적응형 buffer: 기본 4KB, burst 3 tick 포화 시 64KB, 2 tick 반 미만 시 4KB 복귀
        // tokio::task::block_in_place 로 pty.read_available() 호출
    }
}
```
- 적응형 buffer 정책 구현 (RG-V3-002-3)
- AC-T-9 대응 (p99 read ≤ 5ms + 64KB 전환)

**참조 구현**: research.md §3.2 (라인 238~274, Zed 3-thread 패턴)

**MX tag 계획**:
- `PtyWorker::run`: `@MX:WARN(blocking-read-in-async-context)` with `@MX:REASON(portable-pty-blocking-api)` — tokio async runtime 내 blocking call 명시
- 적응형 buffer 전환 로직: `@MX:NOTE(adaptive-buffer-4k-to-64k)` — burst 감지 heuristic

### T4: libghostty-vt Terminal::feed 연동 + RenderState delta

**Priority**: High
**선행 의존성**: T1, T3
**대상 파일**:
- `crates/moai-studio-terminal/src/vt.rs` (T1 에서 신규 생성, 여기서 확장)
- `crates/moai-studio-terminal/src/worker.rs` (T3 에서 확장: feed 호출 삽입)

**예상 산출**:
- `vt::Terminal::feed(&[u8])` 이 내부 `libghostty_ffi::Terminal` 에 전달
- `vt::Terminal::render_state() -> RenderState { grid: Grid<Cell>, cursor: (row, col), attrs: ... }`
- worker 에서 `pty.read_available()` → `terminal.feed(buf)` → `PtyEvent::Output` emit
- AC-T-8 (b) 대응 (Grid assertion 테스트 가능)

**참조 구현**: research.md §3.1 (라인 171~193, RenderState Grid 구조)

**MX tag 계획**:
- `vt::Terminal::render_state`: `@MX:ANCHOR(render-state-contract)` — GPUI render 쪽 fan_in 경계

### T5: `TerminalSurface` GPUI 컴포넌트

**Priority**: High
**선행 의존성**: T4
**대상 파일** (spec.md §9 canonical layout 기준):
- `crates/moai-studio-ui/src/terminal/mod.rs` (신규, `TerminalSurface` 렌더 본체)
- `crates/moai-studio-ui/src/lib.rs` (re-export 추가)
- `crates/moai-studio-ui/src/root_view.rs` (content_area 분기 수정 — TerminalSurface vs Empty State)

**예상 산출**:
```rust
pub struct TerminalSurface {
    grid: Arc<RwLock<RenderState>>,
    font: Font,
    cursor_blink: bool,
    selection: Option<Selection>,
}

impl gpui::Render for TerminalSurface {
    fn render(&mut self, cx: &mut RenderContext) -> impl IntoElement {
        // per-cell glyph rendering + background fill + cursor + selection
    }
}

impl TerminalSurface {
    pub fn on_pty_event(&mut self, event: PtyEvent, cx: &mut WindowContext) {
        // grid snapshot 갱신 + cx.notify()
    }
    
    pub fn pixel_to_cell(&self, px: f32, py: f32) -> (u16, u16) { /* research.md §5.1 */ }
}
```
- RootView 통합 (SPEC-V3-001 의 content_area 자리에 TerminalSurface 배치)
- AC-T-3, AC-T-6 대응

**참조 구현**: research.md §3.1 (라인 195~205, gpui-ghostty pattern)

**MX tag 계획**:
- `TerminalSurface::render`: `@MX:ANCHOR(terminal-surface-render)` — GPUI 렌더 진입점
- `TerminalSurface::pixel_to_cell`: `@MX:NOTE(font-metric-coord-mapping)`

### T6: 키보드 입력 → ANSI encoding → PTY stdin

**Priority**: High
**선행 의존성**: T5
**대상 파일** (spec.md §9 canonical layout 기준):
- `crates/moai-studio-ui/src/terminal/input.rs` (신규, key event → ANSI dispatch)
- `crates/moai-studio-ui/src/terminal/mod.rs` (T5 확장: `on_key` 호출 site)
- `crates/moai-studio-terminal/src/vt.rs` (KeyEncoder re-export)

**예상 산출**:
- GPUI `KeyEvent` → `libghostty_vt::KeyEncoder::encode()` → ANSI bytes → `pty.feed(&bytes)`
- 특수 키 조합 처리 (Arrow, Home, End, PageUp/Down, F1-12)
- AC-T-4 대응 (p99 key-echo ≤ 16ms)

**MX tag 계획**:
- `TerminalSurface::on_key`: `@MX:NOTE(key-to-ansi-dispatch)` — 키 조합 분기점

### T7: 로컬 클립보드 (arboard) + 선택 영역

**Priority**: Medium
**선행 의존성**: T5, T6
**대상 파일** (spec.md §9 canonical layout 기준):
- `crates/moai-studio-terminal/Cargo.toml` (arboard 추가)
- `crates/moai-studio-ui/src/terminal/clipboard.rs` (신규, arboard + Selection state + copy)
- `crates/moai-studio-ui/src/terminal/mod.rs` (selection 렌더 통합)

**예상 산출**:
- `Selection { start: (row, col), end: (row, col) }` 드래그 상태 관리
- `Cmd+C` (macOS) / `Ctrl+Shift+C` (Linux) → `arboard::Clipboard::set_text(grid_to_text())`
- `Ctrl+C` (선택 없을 때) → SIGINT (PTY `\x03` 전송)
- OSC 52 는 parser 에서 silently ignore (별도 코드 작성 불필요)
- AC-T-4 의 hello echo 시나리오와 무관한 별도 AC 는 없지만 RG-V3-002-4 충족

**참조 구현**: research.md §5.2, §5.3 (라인 428~480, arboard 사용 패턴)

**MX tag 계획**:
- `copy_selection`: `@MX:NOTE(local-clipboard-only-no-osc52)` — OSC 52 제외 정책 명시

### T8: `ghostty-spike` 예제 바이너리

**Priority**: Medium
**선행 의존성**: T5, T6
**대상 파일**:
- `crates/moai-studio-terminal/examples/ghostty-spike.rs` (신규)
- `crates/moai-studio-terminal/Cargo.toml` (`[[example]]` entry)

**예상 산출**:
- `clap` 또는 `std::env::args` 로 `--headless` 플래그 파싱
- headless: PTY spawn → `echo "Scaffold OK"` → stdout 검증 → exit 0
- non-headless: GPUI 윈도우 + TerminalSurface + spawn 된 쉘 prompt 표시
- AC-T-3, AC-T-7 (스모크 테스트) 대응

**MX tag 계획**:
- 예제 파일 상단: `@MX:NOTE(example-smoke-entrypoint)` — CI 스모크 진입점 문서화

### T9: CI matrix 확장 (setup-zig + cache + smoke + pin-policy-guard)

**Priority**: High
**선행 의존성**: T1, T8
**대상 파일** (spec.md §9 canonical layout 기준):
- `.github/workflows/ci-rust.yml` (수정 — Zig setup + cache + smoke job + pin-policy-guard job 추가)
- `scripts/ci/check-history-sha.sh` (신규, AC-T-11 (ii) 검증 스크립트)
- `scripts/ci/check-wrapper-scope.sh` (신규, AC-T-11 (iii) 검증 스크립트 — `annotation-cycle-required` 라벨 부착)

**예상 산출**:
```yaml
# .github/workflows/ci-rust.yml (발췌)
jobs:
  rust:
    # ... 기존 matrix + Zig setup ...
    steps:
      - name: Install Zig 0.15.2
        uses: mlugg/setup-zig@v2.2.1
        with:
          version: 0.15.2

      - name: Cache Zig installation
        uses: actions/cache@v4
        with:
          path: |
            ~/.cache/zig
            ~/Library/Caches/zig
          key: zig-0.15.2-${{ runner.os }}-${{ runner.arch }}

      - name: Smoke test (ghostty-spike --headless)
        run: cargo run --example ghostty-spike -- --headless

  # iter 2 신설 job (AC-T-11)
  pin-policy-guard:
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 2 }
      - name: Detect rev change
        id: rev-diff
        run: |
          if git diff HEAD~1 -- crates/moai-studio-terminal/Cargo.toml | grep -qE '^[+-].*rev\s*='; then
            echo "changed=true" >> "$GITHUB_OUTPUT"
          fi
      - name: (i) Re-run characterization
        if: steps.rev-diff.outputs.changed == 'true'
        run: cargo test -p moai-studio-terminal --test libghostty_api_compat
      - name: (ii) Assert HISTORY includes old + new SHA
        if: steps.rev-diff.outputs.changed == 'true'
        run: bash scripts/ci/check-history-sha.sh
      - name: (iii) Check wrapper-external file changes
        if: steps.rev-diff.outputs.changed == 'true'
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          PR_NUMBER: ${{ github.event.pull_request.number }}
        run: bash scripts/ci/check-wrapper-scope.sh
```

**pin-policy-guard 가드 스크립트 스펙** (실제 파일 생성은 Run phase, 본 plan 에는 스펙만 기술):

- `scripts/ci/check-history-sha.sh`:
  - 입력: `git diff HEAD~1 -- .moai/specs/SPEC-V3-002/spec.md` (HISTORY 섹션)
  - 로직: `Cargo.toml` 의 old SHA (HEAD~1 기준) 와 new SHA (HEAD 기준) 추출 → spec.md HISTORY diff 에 두 SHA 모두 포함되었는지 `grep -F` 로 검증
  - exit 0 on success, exit 1 on missing SHA
- `scripts/ci/check-wrapper-scope.sh`:
  - 입력: `git diff --name-only HEAD~1`
  - 로직: `crates/moai-studio-terminal/src/` 하위 수정 파일 중 `libghostty_ffi.rs` 외 파일이 있는지 검사. 있으면 `gh api -X POST repos/:owner/:repo/issues/$PR_NUMBER/labels -f labels='["annotation-cycle-required"]'` 호출 + `::warning::...` 출력
  - exit 0 (경고 only, 차단 아님)

- AC-T-1, AC-T-7, **AC-T-11** 대응 (iter 2 신설)

**참조 구현**: research.md §4 (라인 354~375, CI 구성 권장), spec.md §2 RG-V3-002-1 (pin 정책 a~d)

### T10: 테스트 suite (≥ 10) + characterization tests + criterion bench

**Priority**: High
**선행 의존성**: T1~T9
**대상 파일** (spec.md §9 canonical layout 기준):
- `crates/moai-studio-terminal/tests/libghostty_api_compat.rs` (신규, characterization — AC-T-11 에서 pin bump 시 재실행)
- `crates/moai-studio-terminal/tests/pty_contract.rs` (신규, Pty trait contract)
- `crates/moai-studio-terminal/tests/worker_adaptive_buffer.rs` (신규)
- `crates/moai-studio-terminal/tests/pty_fd_cleanup.rs` (신규, AC-T-5)
- `crates/moai-studio-terminal/tests/compile_fail.rs` (신규, trybuild harness — T2 에서 생성)
- `crates/moai-studio-terminal/tests/compile_fail/conpty_spawn.rs` (T2 에서 생성, 여기서는 검증)
- `crates/moai-studio-terminal/src/pty/mock.rs` (`#[cfg(test)]` MockPty — pty 디렉터리 모듈 내부)
- `crates/moai-studio-terminal/benches/bench_pty_burst_read.rs` (신규, criterion, AC-T-9)
- `crates/moai-studio-terminal/benches/bench_key_echo_latency.rs` (신규, criterion, AC-T-4)

**예상 산출** (AC-T-8 ≥ 10 테스트):
1. `libghostty_api_compat::terminal_new_smoke` — `Terminal::new()` 성공
2. `libghostty_api_compat::feed_ascii_basic` — `feed(b"abc")` → Grid[0][0..3] = 'a','b','c'
3. `libghostty_api_compat::feed_utf8_multibyte` — `feed("한글".as_bytes())` → 올바른 cell 폭
4. `libghostty_api_compat::csi_cursor_position` — `\x1b[10;5H` → cursor (9, 4)
5. `pty_contract::mock_feed_read_roundtrip` — MockPty 스크립트 응답
6. `pty_contract::set_window_size_propagates` — SIGWINCH 전달
7. `pty_contract::is_alive_after_exit_returns_false`
8. `worker_adaptive_buffer::transitions_to_64k_on_burst` — 3 tick 연속 포화 시 전환
9. `worker_adaptive_buffer::returns_to_4k_after_burst` — 2 tick 반 미만 시 복귀
10. `worker_adaptive_buffer::no_drop_on_backpressure` — unbounded channel 검증
- Criterion bench: `benches/bench_pty_burst_read.rs::bench_pty_burst_read` (AC-T-9), `benches/bench_key_echo_latency.rs::bench_key_echo_latency` (AC-T-4)
- trybuild compile-fail: `tests/compile_fail/conpty_spawn.rs` 가 Windows target 에서 `compile_error!("ConPtyStub is deferred to Phase 7 (GPUI Windows GA)")` 메시지를 assert (AC-T-10, iter 1 D4 해결)

**MX tag 계획**:
- 각 characterization test 상단: `@MX:ANCHOR(libghostty-api-compat-test)` — upstream SHA bump 회귀 감지 지점
- MockPty: `@MX:NOTE(mock-pty-test-only)` with `#[cfg(test)]`

---

## 4. 의존성 그래프

```
T1 (libghostty-rs + Zig)
  │
  ├──► T2 (Pty trait + UnixPty + ConPtyStub + trybuild)
  │     │
  │     └──► T3 (PTY worker + PtyEvent + 적응형 buffer)
  │           │
  │           └──► T4 (VT feed + RenderState delta)
  │                 │
  │                 └──► T5 (TerminalSurface GPUI)
  │                       │
  │                       ├──► T6 (Keyboard → ANSI)
  │                       │     │
  │                       │     ├──► T7 (Clipboard + selection)
  │                       │     │
  │                       │     └──────────┐
  │                       │                ▼
  │                       └──────────────► T8 (ghostty-spike example, T5 ∧ T6 필요)
  │                                        │
  │                                        └──► T9 (CI setup-zig + smoke + pin-policy-guard)
  │
  └──► T10 (tests + bench + trybuild 검증)  ← T1~T9 산출물 검증
```

그래프 정합 메모 (iter 2 D8 해결):
- T8 은 §3 에서 "선행 의존성: T5, T6" 로 선언됨 (`--headless` 비-headless 양 경로 모두 키 입력 경로 검증 필요). 이에 맞추어 그래프도 T5 와 T6 모두에서 T8 로 edge 연결.
- iter 1 그래프는 T8 ← T5 만 연결하여 §3 선언과 불일치했음 (D8).

병렬 가능: T6 + T7 (T5 이후 동일 depth), T8 은 T6 완료 이후 T9 와 병렬 가능.

---

## 5. 위험 matrix (3 상위)

| 순위 | 리스크 | 영향 | 확률 | 완화책 |
|------|--------|------|------|--------|
| 1 | libghostty-rs alpha API churn (예: `RenderState` 필드 변경, `feed` 시그니처 변경) | 높음 (T4/T5 재작성) | 중 (alpha 라 breaking 가능) | (a) wrapper isolation (`src/libghostty_ffi.rs` 외부 직접 참조 금지, RG-V3-002-1 Unwanted), (b) SHA pin + 월 1회 수동 bump 정책, (c) characterization tests (`tests/libghostty_api_compat.rs`) 가 bump PR 마다 자동 실행 |
| 2 | Zig 0.15.x CI 설치 cold 시간 > 2 분 (특히 Linux) | 중 (wall-clock 5분 budget 초과) | 중 | (a) `actions/cache@v4` with runner-specific key, (b) `Swatinem/rust-cache` 병용 (SPEC-V3-001 계승), (c) 첫 PR 에서 cold run 측정 후 key 조정 |
| 3 | tokio `task::block_in_place` latency (IO stall → input lag > 16ms) | 중 (AC-T-4 실패 가능) | 저 | (a) 적응형 buffer (4KB/64KB) 로 syscall 횟수 최적화, (b) `tracing::instrument` 로 각 단계 latency 측정, (c) criterion bench (`bench_key_echo_latency`) 로 회귀 감지 |

추가 위험 (낮은 우선순위, 완화 계획만 기록):
- arboard 의 Wayland 지원 불안정성 → X11 fallback 자동 감지
- GPUI 의 font metrics API 변경 (SPEC-V3-001 계승, 0.2.x 안정성 의존)

---

## 6. MX Tag 계획 요약

| Tag 유형 | 개수 예상 | 주요 위치 |
|----------|-----------|-----------|
| `@MX:ANCHOR` | 5+ | `libghostty_ffi` 경계, `Pty` trait, `RenderState`, `TerminalSurface::render`, characterization test |
| `@MX:WARN` | 2+ | blocking-in-async (worker), zig-toolchain-precondition (build.rs) |
| `@MX:NOTE` | 6+ | adaptive-buffer, key-dispatch, font-metric, local-clipboard, example-entrypoint, mock-pty |
| `@MX:TODO` | 1 | `ConPtyStub::spawn` (Phase 7) |

모든 `@MX:WARN` 는 `@MX:REASON` sub-annotation 필수 (CLAUDE.md MX protocol 준수).

---

## 7. 방법론별 실행 맵

Run phase 진입 시 `quality.yaml development_mode` 에 따라 분기:

### TDD 경로 (RED-GREEN-REFACTOR)

- T1 RED: `build.rs Zig 미설치 에러 메시지 assertion` 테스트 작성 → FAIL
- T1 GREEN: build.rs 구현
- T1 REFACTOR: wrapper module 분리
- (T2~T10 동일 cycle 반복)

### DDD 경로 (ANALYZE-PRESERVE-IMPROVE)

- ANALYZE: SPEC-V3-001 의 `moai-studio-terminal` Cargo.toml TODO 주석 + 기존 워크스페이스 영향 분석
- PRESERVE: 기존 248 tests (SPEC-V3-001 baseline) 가 각 task 완료 후 여전히 GREEN 임을 검증하는 characterization test
- IMPROVE: T1→T10 순서로 점진 구현, 각 단계 후 `cargo test --workspace` 실행

---

## 8. 완료 기준 (DoD)

- [ ] 11 개 AC (AC-T-1 ~ AC-T-11) 모두 Green (iter 2 에서 AC-T-11 pin-policy-guard 추가)
- [ ] SPEC-V3-001 기존 248 tests 회귀 없음
- [ ] 신규 테스트 ≥ 10 (AC-T-8)
- [ ] CI matrix (macOS + Linux) × (rust + smoke) 4 job 모두 Green, wall-clock ≤ 5분
- [ ] MX tag 총 14+ 개 (ANCHOR 5, WARN 2, NOTE 6, TODO 1) 추가
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 warnings
- [ ] `cargo fmt --all -- --check` 통과
- [ ] Windows compile-gate 확인 (`cargo check --target x86_64-pc-windows-msvc`)
- [ ] research.md + spec.md + plan.md + acceptance.md 4 문서 정합성 유지

---

## 9. 참조

- `.moai/specs/SPEC-V3-002/spec.md` — EARS 요구사항 (RG-V3-002-1 ~ 5)
- `.moai/specs/SPEC-V3-002/research.md` — deep research 근거
- `.moai/specs/SPEC-V3-002/acceptance.md` — Given/When/Then 시나리오
- `.moai/specs/SPEC-V3-001/progress.md` — rescope 근거
- `.moai/design/master-plan.md` § Phase 2 — 아키텍처 배경
- `.claude/rules/moai/workflow/workflow-modes.md` — TDD/DDD 선택

---

Version: 0.2.0-draft · 2026-04-21 (iter 2 revision)
