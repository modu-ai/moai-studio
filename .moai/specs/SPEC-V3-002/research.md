# SPEC-V3-002 Research — Terminal Core (libghostty-vt + PTY + Shell 통합)

---
spec_id: SPEC-V3-002
created: 2026-04-21
author: MoAI (manager-spec, Plan Phase research)
scope: 업스트림 의존성 조사 + 내부 코드베이스 실증 + 설계 결정점 도출
---

## 0. Research 목적

SPEC-V3-001 RG-V3-3 rescope 결과, 터미널 통합은 "Metal toolchain blocker" 문제가 아니라 **업스트림이 alpha 이며 실제 작업이 시작되지 않은 상태** 임이 확인됨. 본 research.md 는 SPEC 에서 확정해야 할 6 가지 결정점을 구체적 근거 (commit hash, 버전, 파일 경로) 와 함께 제시하는 것을 목표로 한다.

조사 방법:
- 업스트림 리포지토리 `Uzaaft/libghostty-rs`, `zed-industries/zed`, `mlugg/setup-zig` 직접 fetch
- crates.io + docs.rs 에서 `portable-pty` 0.9.0 API 확인
- 내부 `crates/moai-studio-*/Cargo.toml`, `.github/workflows/ci-rust.yml` 현재 상태 확인
- Context7 MCP (portable-pty `/websites/rs_portable-pty`) 교차검증

---

## 1. libghostty-rs 업스트림 상태

### 1.1 리포지토리 개요

- **Repo**: [github.com/Uzaaft/libghostty-rs](https://github.com/Uzaaft/libghostty-rs)
- **License**: MIT OR Apache-2.0
- **활동성**: 254 stars · 16 forks · 5 open PRs (fetch 기준 2026-04-21)
- **Workspace 구조**:
  - `crates/libghostty-vt-sys` (v0.1.1) — raw FFI bindings (libghostty C 심볼)
  - `crates/libghostty-vt` (v0.1.1) — safe Rust wrapper (Terminal, RenderState, KeyEncoder, MouseEncoder)
  - `example/ghostling_rs` — macroquad 기반 minimal terminal 예제 (nix 0.31.2 사용, GPUI 참고 대상 아님)

### 1.2 릴리즈/배포 상태

- **crates.io 미배포** (no published releases on the registry)
- **GitHub Releases 없음**: 모든 개발이 `master` 브랜치 HEAD 에서 진행
- **결론**: 외부 프로젝트는 `{ git = "...", rev = "<SHA>" }` 형식으로 pin 필수

### 1.3 Pinned commit 선정

조사 대상 최신 HEAD (fetch 기준 2026-04-21):

| 필드 | 값 |
|------|----|
| SHA | `dfac6f3e8e7ff1567a7dead6639ef36c42e4f15a` |
| Date | 2026-04-20 UTC 20:57:23 |
| Author | Patrick Hall (phall1) |
| Message | callback userdata 안정성 개선 — `std::ptr::from_mut(self.vtable.as_mut())` 로 provenance 정확화 + move regression test 강화 |

**선정 근거**:
1. 2026-04-20 커밋은 "terminal 객체 relocation 시 userdata pointer 안정성" 을 수정 — 우리가 Rust 소유 구조 (Box, Vec) 내부에 Terminal 을 넣을 경우 직격타인 안정성 수정
2. 같은 날짜 커밋은 move regression test 를 강화해 유사 regression 방지 기능 추가
3. 이 커밋 이전 (2026-04 중반 이전) 은 FFI 안정성 문제 가능성 존재

**리스크**:
- alpha 단계이므로 API churn 가능성 ~15-20% (월별 breaking change 체감)
- 대응: 본 SPEC 완료 후 90일 내 pinned commit 재평가, research/evolution-log 에 기록

### 1.4 Public API surface (`libghostty-vt` crate)

`crates/libghostty-vt/src/terminal.rs` 기준:

```rust
pub struct Terminal<'alloc, 'cb> { ... }

impl Terminal<'_, '_> {
    pub fn new(opts: Options) -> Result<Self>;
    pub fn new_with_alloc<'ctx: 'alloc>(
        alloc: &'alloc Allocator<'ctx>,
        opts: Options,
    ) -> Result<Self>;

    pub fn vt_write(&mut self, data: &[u8]);   // PTY → VT parser 주입
    pub fn on_pty_write(&mut self, f: impl PtyWriteFn<'alloc, 'cb>) -> Result<&mut Self>;

    pub fn cols(&self) -> Result<u16>;
    pub fn rows(&self) -> Result<u16>;
    pub fn resize(&mut self, cols: u16, rows: u16, cw_px: u32, ch_px: u32) -> Result<()>;

    pub fn cursor_x(&self) -> Result<u16>;
    pub fn cursor_y(&self) -> Result<u16>;
    pub fn is_cursor_pending_wrap(&self) -> Result<bool>;
    pub fn is_cursor_visible(&self) -> Result<bool>;
}

impl Drop for Terminal<'_, '_> { /* ffi::ghostty_terminal_free */ }
```

Public modules: `alloc`, `build_info`, `error`, `fmt`, `focus`, `key`, `kitty`, `mouse`, `osc`, `paste`, `render`, `screen`, `sgr`, `style`, `terminal`.

Re-exports: `Error`, `RenderState`, `Terminal`, `TerminalOptions`.

### 1.5 [CRITICAL] Send/Sync 제약

> `libghostty-vt` 의 **모든 객체는 `!Send + !Sync`** 로 명시 (소스 doc comment)
> "All `libghostty-vt` objects are **not** thread-safe, and have been marked `!Send + !Sync` accordingly."

**설계 영향**:
- Terminal 객체를 tokio::spawn task 간에 넘길 수 없음
- portable-pty reader 는 blocking thread 에 두고, 바이트를 mpsc channel 로 GPUI foreground task 로 전달 후 vt_write 해야 함
- 통상적 "async 전체" 설계 불가 — GPUI 엔티티 소유권 모델과 잘 맞지만 tokio 기반 설계는 재조정 필요

### 1.6 [CRITICAL] MSRV 충돌

| 프로젝트 | MSRV |
|----------|------|
| `moai-studio` workspace (`Cargo.toml`) | **1.85** (Rust 2024 edition) |
| `libghostty-rs` (`crates/libghostty-vt/Cargo.toml`) | **1.93** (2024 edition) |

**결론**: 본 SPEC 에서 workspace MSRV 를 **1.93 이상으로 상향** 하거나, libghostty-rs 포크 + MSRV 완화 중 택1 필요.

**권장**: MSRV 상향 (포크는 유지보수 부담 과다). 2026-04 기준 Rust stable 은 1.93+, CI `dtolnay/rust-toolchain@stable` 는 자동 수용.

**OPEN QUESTION Q1**: MSRV 1.85 → 1.93 상향이 다른 crate (GPUI 0.2, moai-core 등) 에 호환 문제를 일으키는가? → Q1 답변은 annotation cycle 에서 확정.

### 1.7 Zig 빌드 체인

- **요구사항**: Zig 0.15.x on PATH (공식 README)
- **빌드 동작**: `build.rs` 가 Ghostty C 소스를 자동 fetch → Zig 로 컴파일 → static lib 생성 → Rust sys crate 가 link
- **`GHOSTTY_SOURCE_DIR` env var** 로 로컬 checkout 오버라이드 가능 (offline CI 또는 커스텀 Ghostty 버전 시)
- **Zig 미설치 환경**: `build.rs` 가 build script 단계에서 실패 → 명확한 에러 메시지를 사용자에게 포워딩할 책임은 `moai-studio-terminal` `build.rs` 쪽에 있음 (upstream 은 panic 만)

---

## 2. portable-pty 0.9.x 패턴

### 2.1 최신 버전 및 개요

- **Latest on crates.io**: **0.9.0**
- **Documentation coverage**: 71.74% (docs.rs)
- **License**: MIT
- **Origin**: wezterm monorepo 내부 crate, cross-platform PTY 추상화

### 2.2 핵심 API

```rust
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

let pty_system = native_pty_system();
let pair = pty_system.openpty(PtySize {
    rows: 24,
    cols: 80,
    pixel_width: 0,
    pixel_height: 0,
})?;

// Shell spawn
let cmd = CommandBuilder::new("zsh");  // 환경 상속은 CommandBuilder::env 필요
let child = pair.slave.spawn_command(cmd)?;

// I/O 핸들 획득
let mut reader = pair.master.try_clone_reader()?;   // Box<dyn Read + Send>
let mut writer = pair.master.take_writer()?;        // Box<dyn Write + Send>

// resize
pair.master.resize(PtySize { rows: 30, cols: 120, .. })?;
```

### 2.3 Send/Sync 특성

- `try_clone_reader()` → `Box<dyn Read + Send>` — **Send 가능**, 별도 std::thread 로 이동 OK
- `take_writer()` → `Box<dyn Write + Send>`
- 단, `Box<dyn Read>` 자체는 blocking read → tokio async reactor 에 직접 등록 불가
- **권장 패턴**: `std::thread::spawn` 으로 reader 를 blocking loop 에 두고 `std::sync::mpsc::channel` 또는 `tokio::sync::mpsc` (blocking send) 로 비동기 경계 건너기

### 2.4 플랫폼 지원

- `aarch64-apple-darwin` (macOS Apple Silicon) OK
- `aarch64-unknown-linux-gnu`, `x86_64-unknown-linux-gnu` OK
- `i686/x86_64-pc-windows-msvc` OK (ConPTY 자동 선택)

**본 SPEC 범위**: macOS + Linux. Windows ConPTY 는 Phase 7 대상이지만 API 추상화 (`native_pty_system()`) 가 동일해 대응 비용 거의 없음 — 설계상 이를 깨는 선택 (직접 openpty syscall 호출 등) 은 금지.

### 2.5 alternative: `pty-process`, `tokio-pty-process`

| Crate | 장점 | 단점 | 선택 |
|-------|------|------|------|
| `portable-pty` 0.9.0 | Cross-platform (ConPTY 포함), Zed/Warp/WezTerm 채택 | 동기 Read/Write | **선택** |
| `pty-process` 0.5.x | Tokio 통합 async API | **Unix 전용**, Windows 미지원 → Phase 7 재작성 필요 | 탈락 |
| `tokio-pty-process` | 오래된 crate | 유지보수 중단 (2021 이후 활동 없음) | 탈락 |

**결론**: `portable-pty` 0.9.0 + `std::thread` reader + `tokio::sync::mpsc` 조합이 v3 스택 (tokio + GPUI) 과 최적.

---

## 3. Zed terminal 구현 참고 (alacritty_terminal 기반)

### 3.1 중요한 발견: Zed 는 libghostty 를 쓰지 않는다

`crates/terminal/Cargo.toml` (`zed-industries/zed` main branch):
- **Core VT engine**: `alacritty_terminal` (workspace dep)
- **Thread model**: `smol` async runtime (not tokio)
- **Font / rendering**: `gpui` (workspace dep, unreleased)

즉, "Zed Terminal = GPUI + libghostty" 추측은 **틀린 정보**. 우리 프로젝트는 GPUI 는 Zed 와 공유하지만 VT core 는 새로 (libghostty-vt 로) 구현해야 한다. 이는 "Zed 코드 복사" 가 아니라 **아키텍처 패턴 참고** 수준으로 reference 가치를 낮춘다.

### 3.2 Zed Terminal 아키텍처 패턴 (참고용)

`crates/terminal/src/terminal.rs` 에서 추출한 패턴:

1. **핵심 struct** (~55 fields, line 1100-1200):
   - `FairMutex<Term<ZedListener>>` — alacritty Term 감싸는 lock
   - `VecDeque<InternalEvent>` — 비동기 이벤트 큐
   - `Notifier` — PTY write channel
   - `TerminalContent { cells: Vec<IndexedCell>, cursor, selection, scroll }` — UI 가 소비하는 snapshot

2. **Event loop**:
   - `EventLoop::new()` 가 `_io_thread` spawn → PTY read/write 전담
   - `UnboundedSender<AlacTermEvent>` → `ZedListener` → main task
   - Unix: `cx.spawn()` (dedicated thread), Windows: `background_spawn()`

3. **Throttling** (60fps 보장 메커니즘):
   - **첫 이벤트 즉시 처리** (latency 최소화)
   - **그 후 최대 100 event / 4ms 배칭** → 한 번만 `cx.notify()`
   - Hyperlink 검색은 mouse move >5px or >100ms elapsed 시에만 재시도

4. **Key → escape sequence** (`crates/terminal/src/mappings/keys.rs`, ~430 lines):
   - `to_esc_str(keystroke, mode, option_as_meta) -> Option<Cow<'static, str>>`
   - Manual mapping: arrow / Home / End / F1-F20 / Tab / Enter / Backspace / Delete
   - Modifier code: Shift=2, Alt=3, Ctrl=5, Shift+Ctrl=6, Alt+Ctrl=7, Shift+Alt+Ctrl=8
   - ALT_SCREEN 모드: Shift+Home/End 등 bracketed escape
   - APP_CURSOR 모드: 기본 arrow 가 `\x1b[A` → `\x1bOA` (SS3) 로 변경
   - Ctrl+C, Ctrl+D: ASCII 0x03, 0x04 (caret notation 일반 매핑)

### 3.3 libghostty-vt 가 제공하는 대안

libghostty-vt 는 `KeyEncoder`, `MouseEncoder` 를 re-export 하므로 **Zed 의 ~430 lines `mappings/keys.rs` 재구현 불필요**. SPEC 설계 시 libghostty-vt 의 KeyEncoder 로 GPUI KeyDownEvent → bytes 변환 위임을 1순위로 한다.

**OPEN QUESTION Q2**: libghostty-vt::key::KeyEncoder 의 입력 타입이 GPUI KeyDownEvent 와 직접 매핑 가능한가, 아니면 중간 adapter 필요한가? → 구현 시점 (Phase 2 Run) 에 확인, SPEC 에서는 adapter layer 를 non-functional 로 기술.

### 3.4 Rendering 패턴 (Grid → GPUI Element)

Zed 의 경우 `Terminal::sync()` → `make_content()` → `TerminalContent { cells: Vec<IndexedCell> }` 를 produce. UI layer (`terminal_view` crate) 가 `TerminalContent` 를 소비하여 `gpui::Element::paint` 로 cell-by-cell glyph 그림.

libghostty-vt 는 유사한 snapshot API (`RenderState` + row/cell iterator) 제공:
- 각 row: foreground/background color, flags (bold/underline/reverse), Unicode grapheme
- GPUI `Element` trait 구현에서 `render` 마다 `RenderState` 를 읽어 cell-by-cell paint

---

## 4. Zig 0.15.x CI 설치

### 4.1 `mlugg/setup-zig` action

- **Repo (primary)**: [codeberg.org/mlugg/setup-zig](https://codeberg.org/mlugg/setup-zig) (GitHub 은 read-only mirror)
- **Latest tag**: **v2.2.1** (2026-01-19 release)
- **Inputs**:
  - `version`: `0.15.1` (권장, specific release) / `master` (nightly, 불안정) / `latest` (resolve from build.zig.zon)
  - `mirror`: custom URL (기본 Zig 공식 CDN 제외)
  - `use-cache`: boolean (default true)
  - `cache-key`: matrix 전용
  - `cache-size-limit`: MiB 단위, default 2048

### 4.2 캐시 전략

- Global Zig cache directory (`~/.cache/zig`) 를 runner 간 보존
- Local caches 는 shared location 으로 redirect
- 2GiB 초과 시 자동 clearing

### 4.3 플랫폼 지원

- `ubuntu-latest` OK (명시적 예제)
- `macos-14` (Apple Silicon) OK (homebrew 우회, action 이 직접 Zig 바이너리 fetch)
- `windows-latest` OK (Phase 7 대비 검증)

### 4.4 SPEC 반영 권장 설정

```yaml
- name: Install Zig 0.15.1
  uses: mlugg/setup-zig@v2  # v2.2.1 당시 유효, v2 tag 사용으로 patch 자동 수용
  with:
    version: 0.15.1
    use-cache: true
    cache-size-limit: 2048
```

**설치 시간**: v2 의 cache 활성 시 30-60s (cold), 5-10s (warm). 캐시 warm 기준 SPEC-V3-001 의 기존 CI total runtime (3-4분) 에 +10% 이내 영향.

---

## 5. 입력 파이프라인 (Keyboard → ANSI escape → PTY)

### 5.1 GPUI KeyDownEvent (gpui 0.2.2)

현재 프로젝트는 `gpui = "0.2"` (Cargo.toml line 16, `crates.io 0.2.2`). 본 연구 시점에 gpui 0.2.2 공식 문서는 [docs.rs/gpui/0.2.2](https://docs.rs/gpui/0.2.2) 에 publish 되어 있으나 **unreleased 빌드가 자주 변경** 되는 crate 이므로 구현 단계에서 version-specific API 재검증 필요.

주요 타입 (일반 패턴):
- `KeyDownEvent { keystroke: Keystroke, is_held: bool }`
- `Keystroke { modifiers: Modifiers, key: SharedString, ime_key: Option<SharedString> }`
- `Modifiers { control, alt, shift, platform /* = Cmd on macOS */, function }`

**[OPEN QUESTION Q3]**: gpui 0.2.2 의 KeyDownEvent shape 이 Zed main (unreleased) 과 호환되는지 확인. → Phase 2 Run 시점에 `cargo doc --package gpui --open` 으로 실측.

### 5.2 VT100 vs xterm-256color 선택

- **TERM 환경변수**: `xterm-256color` 로 spawn 시 설정 — 현대 shell 표준, emoji/nerdfont 호환
- **VT100**: legacy, 256 color 미지원, 선택 안함
- **ghostty**: libghostty-vt 기본 모드와 일치하도록 향후 `$TERM = ghostty` 도 고려하되 대부분의 ncurses terminfo 가 ghostty 지원 안함 → **xterm-256color 유지 권장**

### 5.3 표준 escape sequence (fallback 참조)

libghostty-vt::key::KeyEncoder 위임이 1순위. Fallback / debug 시 참고할 표준:

| Key | Default | App Cursor Mode |
|-----|---------|-----------------|
| Up | `\x1b[A` | `\x1bOA` |
| Down | `\x1b[B` | `\x1bOB` |
| Right | `\x1b[C` | `\x1bOC` |
| Left | `\x1b[D` | `\x1bOD` |
| F1 | `\x1bOP` | |
| F5 | `\x1b[15~` | |
| F12 | `\x1b[24~` | |
| Home | `\x1b[H` | `\x1bOH` |
| End | `\x1b[F` | `\x1bOF` |
| Enter | `\r` (`\x0d`) | |
| Tab | `\t` (`\x09`) | |
| Backspace | `\x7f` | |
| Ctrl+A..Z | `\x01..\x1a` | |
| Alt+X | `\x1b` + X (option_as_meta=true) | |

---

## 6. 클립보드 통합

### 6.1 crate 선택

| Crate | 장점 | 단점 |
|-------|------|------|
| `copypasta` 0.10.x | macOS + Linux (X11/Wayland) + Windows 공식 지원, 활발 | deps 상대적으로 큼 (objc, x11-clipboard) |
| `arboard` 3.x | 경량, 이미지 지원 | Wayland 지원 불완전 |
| gpui built-in | GPUI context 와 통합 | gpui 0.2.2 에서 API 공개 여부 불명 — 확인 필요 |

**권장**: `copypasta` 우선, gpui built-in 이 안정적이면 마이그레이션. Phase 2 Run 단계에서 실측.

### 6.2 OSC 52 (원격 클립보드)

- SSH 세션에서 호스트 → 로컬 클립보드 복사 표준
- libghostty-vt 는 OSC 파서 제공 (`osc` 모듈)
- **본 SPEC 범위 아님** — SSH 자체가 Phase 3+. OSC 52 수신 시 silent drop 으로 처리, Phase 3 에서 활성화.

### 6.3 플랫폼별 단축키

| 플랫폼 | Copy | Paste |
|--------|------|-------|
| macOS | `Cmd+C` (selection 존재) / SIGINT (no selection) | `Cmd+V` |
| Linux | `Ctrl+Shift+C` | `Ctrl+Shift+V` |
| Linux | `Ctrl+C` — SIGINT 로 PTY 전송 (clipboard 아님) | |

**중요**: macOS `Cmd+C` vs Linux `Ctrl+C` 는 SIGINT 와 구분 로직 필수. selection 존재 여부가 분기 조건.

---

## 7. Rendering 성능

### 7.1 목표

- `yes | head -1000000` / `find /` 와 같은 고출력 명령 중에도 **60 fps** 유지
- 빈 대기 상태 (idle) CPU < 1%
- 스크롤 latency < 16ms (한 프레임)

### 7.2 Throttling 전략 (Zed 패턴 차용)

1. **Event batching**:
   - 첫 이벤트는 **즉시 처리** (keystroke → echo 왕복 latency 최소화)
   - 이후 최대 N events or M ms 배칭 → 한 번 `cx.notify()`
   - 권장 초기값: `N = 100, M = 8ms` (Zed: 100/4ms, 60fps 기준 더 여유)

2. **Dirty row 추적**:
   - libghostty-vt 의 RenderState 가 row dirty flag 제공 여부 확인 필요 ([OPEN QUESTION Q4])
   - 제공하지 않으면 GPUI 측에서 직전 snapshot 과 diff 해 dirty row 만 repaint

3. **Scrollback 제한**:
   - 기본 scrollback = 10,000 rows
   - 메모리 상한: 10k rows × 200 cols × ~24 bytes/cell ≈ 48MB/terminal
   - 사용자 설정 가능, 상한 100k rows

### 7.3 알려진 Pitfall (Zed / WezTerm / Warp 사례)

- **GPU context 재생성 비용**: GPUI element 마다 re-create 하지 말고 Element struct 에 Grid snapshot 만 전달
- **Font shaping cache**: grapheme → glyph 매핑 캐시 필수, 매 프레임 reshape 금지
- **Mouse selection 중 full redraw**: Zed 는 selection 상태 변경 시 전체 redraw 필요 → SPEC 에서는 selection rectangle 만 re-paint 하는 최적화를 non-functional 에 명시

---

## 8. 내부 코드베이스 현재 상태

### 8.1 `crates/moai-studio-terminal/` (현재 scaffold only)

- `src/lib.rs` — `pub fn hello()` stub only, 실제 로직 0 lines
- `Cargo.toml`:
  - `tokio` with `process + io-util + sync` features (workspace)
  - `thiserror` (workspace)
  - `tracing` (workspace)
  - TODO 주석: `libghostty-vt = { git = ..., rev = "PINNED_COMMIT" }` 미추가
  - TODO 주석: `portable-pty = "0.9"` 미추가

### 8.2 `.github/workflows/ci-rust.yml` 현재 상태

- 2 platform matrix (macOS-14, ubuntu-22.04) — Windows 주석 처리
- 4 quality gates (fmt, clippy, test, build --release) + 스모크 (--scaffold flag) ALL GREEN (run 24708460052)
- TODO 주석 (line 70-74): Zig 0.15.x 설치 스텝 미추가

### 8.3 Workspace MSRV 불일치

```toml
# Cargo.toml (root)
rust-version = "1.85"

# libghostty-rs workspace
rust-version = "1.93"
```

**결론**: SPEC-V3-002 Phase 2 Run 첫 커밋에서 workspace `rust-version` 을 `1.93` 으로 상향 필요. Edge case 검증은 annotation cycle 에서 [Q1] 응답에 따라 결정.

---

## 9. 결정점 요약

| # | 결정점 | 제안 | OPEN QUESTION |
|---|--------|------|----------------|
| D1 | Pinned commit | `dfac6f3e8e7ff1567a7dead6639ef36c42e4f15a` (2026-04-20) | 사용자 승인 필요 |
| D2 | MSRV | 1.85 → 1.93 상향 | Q1 — 다른 deps 호환성 |
| D3 | VT core | libghostty-vt (Zed 의 alacritty_terminal 경로는 버림) | — |
| D4 | PTY crate | portable-pty 0.9.0 | — |
| D5 | Threading | portable-pty reader = std::thread, bytes → mpsc → GPUI foreground (vt_write) | Send/Sync 제약 이해 확인 |
| D6 | Zig CI | `mlugg/setup-zig@v2` with `version: 0.15.1` | — |
| D7 | Key encoding | libghostty-vt KeyEncoder 위임 | Q2 — GPUI 이벤트 매핑 방법 |
| D8 | TERM env | `xterm-256color` | — |
| D9 | Clipboard | `copypasta` 0.10.x (gpui built-in 조사 후 재검토) | Q3 — gpui 0.2.2 clipboard API |
| D10 | Scrollback 기본값 | 10,000 rows | — |
| D11 | Rendering throttle | 첫 이벤트 즉시 처리 + 100 events/8ms batch | Q4 — RenderState dirty flag |
| D12 | 시험 범위 | macOS + Linux. Windows Phase 7 | — |

---

## 10. OPEN QUESTIONS (annotation cycle 대상)

- **Q1** [MSRV]: workspace rust-version 을 1.85 → 1.93 으로 상향해도 기존 crate (moai-core 289 tests, gpui 0.2.2, serde, tokio) 에 영향 없는가? → Phase 2 Run 첫 커밋 시 `cargo check` 로 실측.
- **Q2** [KeyEncoder]: libghostty-vt::key::KeyEncoder 가 GPUI Keystroke 구조와 직접 매핑되는가, adapter 필요한가? → 구현 단계에서 결정, SPEC 에서는 "adapter layer 허용" 만 기술.
- **Q3** [Clipboard]: `copypasta` vs gpui built-in clipboard 중 어느 쪽이 macOS+Linux 이식성·경량성에서 우위인가? → Phase 2 Run 시 decision.
- **Q4** [Render dirty tracking]: libghostty-vt RenderState 가 row-level dirty flag 를 expose 하는가? 없으면 diff 기반 자체 구현. → 구현 시점 확인, SPEC 에서는 non-functional 성능 목표만 명시.
- **Q5** [Pinned commit upgrade cadence]: 본 SPEC 완료 후 pinned commit 을 얼마나 자주 갱신하는가? 월 1회? SPEC-V3-003 단위? → annotation cycle 에서 합의.

---

## 11. Sources

- libghostty-rs repo + commits: https://github.com/Uzaaft/libghostty-rs (commit `dfac6f3e` 2026-04-20)
- libghostty-rs README: https://raw.githubusercontent.com/Uzaaft/libghostty-rs/master/README.md
- libghostty-vt Terminal API: https://raw.githubusercontent.com/Uzaaft/libghostty-rs/master/crates/libghostty-vt/src/terminal.rs
- libghostty-vt lib.rs (module list): https://raw.githubusercontent.com/Uzaaft/libghostty-rs/master/crates/libghostty-vt/src/lib.rs
- awesome-libghostty consumers: https://github.com/Uzaaft/awesome-libghostty
- portable-pty docs.rs 0.9.0: https://docs.rs/portable-pty/latest/portable_pty/
- portable-pty context7 ID: /websites/rs_portable-pty (297 snippets, reputation High)
- Zed terminal Cargo.toml: https://raw.githubusercontent.com/zed-industries/zed/main/crates/terminal/Cargo.toml
- Zed terminal.rs (architecture): https://raw.githubusercontent.com/zed-industries/zed/main/crates/terminal/src/terminal.rs
- Zed keys mapping: https://raw.githubusercontent.com/zed-industries/zed/main/crates/terminal/src/mappings/keys.rs
- setup-zig primary: https://codeberg.org/mlugg/setup-zig (v2.2.1, 2026-01-19)
- setup-zig mirror: https://github.com/mlugg/setup-zig
- xuanwo gpui-ghostty tutorial: https://xuanwo.io/2026/01-gpui-ghostty/ (low content value — 과정 서술 위주)

---

Version: 1.0.0 · 2026-04-21
