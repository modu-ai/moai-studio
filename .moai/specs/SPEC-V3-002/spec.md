# SPEC-V3-002: Terminal Core — libghostty-vt + PTY + Shell 통합

---
id: SPEC-V3-002
version: 1.0.0
status: draft
created: 2026-04-21
updated: 2026-04-21
author: MoAI (manager-spec, fresh rewrite per user directive)
priority: High
issue_number: 0
depends_on: SPEC-V3-001
rescope_from: SPEC-V3-001 RG-V3-3
---

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0 | 2026-04-21 | SPEC-V3-001 RG-V3-3 rescope 이후 처음 작성. research.md 의 12개 결정점을 EARS 요구사항 6개 그룹으로 정식화. 이전 stub/draft 파일을 전량 폐기하고 신규 버전으로 대체. |

---

## 1. 개요

### 1.1 목적

MoAI Studio v3 의 **Terminal Core** 레이어 (`crates/moai-studio-terminal`) 를 scaffold 상태에서 실제 작동하는 터미널 엔진으로 끌어올린다. 범위는 다음과 같다:

- libghostty-vt FFI 통합 (pinned commit, Zig 빌드 체인 포함)
- portable-pty 기반 cross-platform PTY 추상화
- `$SHELL` spawn + bi-directional I/O
- GPUI `TerminalSurface` 컴포넌트로 Grid state 렌더
- 키보드 입력 → ANSI escape → PTY 변환 경로
- GitHub Actions CI matrix 의 Zig 설치 확장

성공 기준: macOS 14 + Ubuntu 22.04 runner 상에서 `cargo run -p moai-studio-terminal --example terminal-spike` 가 GPUI 윈도우에 `$SHELL` prompt 를 렌더하고, 사용자 키 입력이 shell 까지 전달되어 명령 결과가 다시 Grid 에 표시되는 end-to-end 동작.

### 1.2 SPEC-V3-001 과의 관계

본 SPEC 은 SPEC-V3-001 RG-V3-3 (libghostty-vt 스파이크) 의 rescope 결과물이다. SPEC-V3-001 은 4/5 RG 실증 완료로 종결되었고, RG-V3-3 은 재진단에서 "Metal toolchain blocker" 가 오해임이 확인되며 본 SPEC 으로 이관되었다 (상세: `.moai/specs/SPEC-V3-001/progress.md`).

본 SPEC 은 SPEC-V3-001 의 scaffold 산출물 (4영역 GPUI layout, `moai-studio-terminal` crate skeleton, CI matrix) 을 전제로 동작한다.

### 1.3 근거 문서

- Research findings: `.moai/specs/SPEC-V3-002/research.md` (12 결정점 + 5 OPEN QUESTIONS)
- Upstream: `https://github.com/Uzaaft/libghostty-rs` (pinned commit `dfac6f3e`)
- Zed terminal reference: `https://github.com/zed-industries/zed/tree/main/crates/terminal` (아키텍처 패턴만 참고, 코드 복사 없음)
- Master plan: `.moai/design/master-plan.md` §Phase 2

---

## 2. 요구사항 그룹 (EARS)

### RG-V3-002-1: libghostty-vt 의존성 및 빌드 체인

[Ubiquitous] `moai-studio-terminal` crate 는 `libghostty-vt` crate 에 대한 Cargo 의존성을 **포함해야 한다** (git URL + pinned rev 형식).

[Ubiquitous] Workspace `Cargo.toml` 의 `rust-version` 은 libghostty-vt 가 요구하는 **1.93 이상으로 설정되어야 한다**.

[Event-Driven] 개발자가 `cargo build -p moai-studio-terminal` 을 실행하면, 시스템은 `build.rs` 에서 Zig 0.15.x 실행 가능성을 **검증해야 한다**.

[Unwanted] Zig 0.15.x 가 PATH 에 없거나 버전이 0.14 이하인 환경에서 시스템은 빌드를 **시작해서는 안 된다**. 이 경우 `build.rs` 는 stderr 로 `"Zig 0.15.x required — install via mise/asdf/ziglang.org"` 메시지를 출력하고 exit 1 **해야 한다**.

[Unwanted] 시스템은 libghostty-vt upstream 의 `main` 브랜치 HEAD 를 추적하는 형태로 의존성을 **선언해서는 안 된다** (반드시 SPEC 에 명시된 pinned commit 사용).

### RG-V3-002-2: PTY 추상화 및 Shell Spawn

[Ubiquitous] `moai-studio-terminal::pty` 모듈은 `portable-pty` 0.9.x 를 래핑한 cross-platform PTY 추상화를 **제공해야 한다**.

[Event-Driven] 사용자가 터미널 surface 를 요청하면, 시스템은 `$SHELL` 환경변수 (없으면 macOS `/bin/zsh`, Linux `/bin/bash`) 를 spawn 하고 `TERM=xterm-256color`, `COLORTERM=truecolor` 를 설정**해야 한다**.

[Event-Driven] PTY pair 가 성공적으로 열리면 시스템은 `try_clone_reader()` 로 얻은 reader 를 전용 `std::thread` 에서 blocking read loop 로 구동하고, 수신된 바이트를 `tokio::sync::mpsc::UnboundedSender` 로 GPUI foreground task 에 **전달해야 한다**.

[State-Driven] PTY child process 가 살아있는 동안 (While alive) 시스템은 reader thread 를 활성 상태로 **유지해야 한다**.

[Unwanted] Shell process 가 exit 하거나 crash 하면, 시스템은 PTY master/slave FD 와 reader thread 를 **1초 이내에 정리해야 한다**. FD 누수는 허용되지 않는다.

### RG-V3-002-3: libghostty-vt Terminal 통합 및 렌더 스냅샷

[Ubiquitous] `moai-studio-terminal::vt` 모듈은 libghostty-vt `Terminal` 을 래핑하는 `VtState` 타입을 **제공해야 한다**. `VtState` 는 `!Send + !Sync` 특성을 유지하며 GPUI 엔티티 내부에만 소유된다.

[Event-Driven] PTY reader thread 가 보낸 바이트가 GPUI foreground task 에 도달하면, 시스템은 `VtState::feed(&mut self, bytes)` 를 호출해 `libghostty_vt::Terminal::vt_write` 로 파서에 주입**해야 한다**.

[Event-Driven] VT 파서가 screen state 를 변경하면, 시스템은 다음 프레임에서 GPUI `cx.notify()` 를 **트리거해야 한다** (throttled: 첫 이벤트는 즉시, 이후 최대 100 이벤트/8 ms 단위 배칭).

[Unwanted] 시스템은 `libghostty_vt::Terminal` 객체를 `tokio::spawn` task 에 이동시키거나 `Arc<Mutex<_>>` 로 감싸 스레드 간 공유**해서는 안 된다** (`!Send` 위반).

### RG-V3-002-4: GPUI TerminalSurface 컴포넌트 및 입력 경로

[Ubiquitous] `moai-studio-ui::terminal::TerminalSurface` GPUI 컴포넌트는 `VtState::render_snapshot()` 으로 얻은 cell grid 를 cell-by-cell glyph 로 **렌더해야 한다**. foreground/background color, bold/underline/reverse flag 를 포함한다.

[State-Driven] RootView 의 active workspace 가 존재하고 `TerminalSurface` 가 생성된 상태에서, content_area 는 Empty State CTA 대신 `TerminalSurface` 를 **표시해야 한다**.

[Event-Driven] `TerminalSurface` 에 포커스가 있는 상태에서 사용자가 키를 누르면, 시스템은 GPUI `KeyDownEvent` 를 `libghostty_vt::key::KeyEncoder` (또는 동등한 adapter) 로 ANSI escape sequence 바이트로 변환하고 PTY master writer 에 **write 해야 한다**.

[Event-Driven] 사용자가 Cmd+C (macOS) / Ctrl+Shift+C (Linux) 를 누르면, 시스템은 현재 selection 의 텍스트를 OS 시스템 클립보드에 **복사해야 한다**. Selection 이 비어있는 경우 Ctrl+C 는 SIGINT (ASCII 0x03) 로 PTY 에 전송되어야 하며 클립보드에 쓰지 않는다.

### RG-V3-002-5: CI Matrix 확장 및 스모크 테스트

[Ubiquitous] `.github/workflows/ci-rust.yml` 의 `rust` 및 `smoke` job 은 cargo 단계 이전에 `mlugg/setup-zig@v2` (`version: 0.15.1`, `use-cache: true`) 스텝을 **포함해야 한다**. macOS-14 및 ubuntu-22.04 runner 양쪽에 동일하게 적용된다.

[Event-Driven] PR 이 생성되면 CI 는 SPEC-V3-001 의 기존 4 gate 에 더해 `cargo build -p moai-studio-terminal --example terminal-spike` 를 추가 gate 로 **실행해야 한다**.

[Unwanted] Zig 설치 단계가 실패하면 이후 cargo 단계는 **실행되어서는 안 되며**, CI job 은 실패 상태로 종료된다.

### RG-V3-002-6: 테스트 가능성 및 FD 위생

[Ubiquitous] `moai-studio-terminal` 의 public 타입은 PTY mock (in-memory `std::io::Cursor` 기반) 으로 단위 테스트가 가능하도록 **trait 경계로 분리되어야 한다**. 실제 PTY 는 trait 기본 구현 (`NativePty`) 으로 제공된다.

[Ubiquitous] `cargo test -p moai-studio-terminal` 은 실제 shell spawn 없이 구동 가능한 단위 테스트를 **최소 10개 이상 포함해야 한다**.

[Unwanted] 테스트 종료 후 테스트 프로세스의 open FD 수는 시작 시 대비 **증가해서는 안 된다** (lsof 기반 검증).

---

## 3. Acceptance Criteria

| AC ID | Requirement Group | Given | When | Then |
|-------|-------------------|-------|------|------|
| AC-T-1 | RG-1 | Zig 0.15.1 이 PATH 에 있고 workspace rust-version=1.93 설정 | `cargo build -p moai-studio-terminal` 실행 | 빌드 성공, clippy warning 0, libghostty-vt FFI 심볼 link 확인 |
| AC-T-2 | RG-1 | Zig 가 미설치되거나 0.14 이하 | `cargo build -p moai-studio-terminal` 실행 | exit code 1, stderr 에 `"Zig 0.15.x required"` 문자열 포함 |
| AC-T-3 | RG-1 | pinned commit `dfac6f3e...` | `cargo metadata -p libghostty-vt` | resolved rev 이 SPEC 명시와 일치, `main` HEAD 추적 아님 |
| AC-T-4 | RG-2 | PTY spawn 가능 환경 | `cargo run -p moai-studio-terminal --example terminal-spike` | GPUI 윈도우가 열리고 `$SHELL` prompt 렌더됨 (예: `zsh %` 또는 `bash $`) |
| AC-T-5 | RG-2, RG-4 | terminal-spike 윈도우 실행 중 | 사용자가 `echo hello<Enter>` 입력 | 3 초 이내 "hello" 라인이 Grid 에 표시됨 |
| AC-T-6 | RG-2 | terminal-spike 윈도우 실행 중 | 사용자가 shell 에 `exit<Enter>` | 1 초 이내 윈도우 종료, open FD 수가 spawn 전 기준선 이하로 복귀 (lsof 측정) |
| AC-T-7 | RG-3 | VT parser 상태 변경 이벤트 10,000 건 연속 발생 (yes 명령 시뮬) | 렌더 프레임 측정 | 평균 프레임 간격 ≤ 16.67 ms (60 fps), 배칭으로 인한 `cx.notify` 호출 회수 ≤ 이벤트 수의 20% |
| AC-T-8 | RG-3 | 단위 테스트 | `cargo test -p moai-studio-terminal vt_state_is_not_send` | `VtState: !Send + !Sync` 를 컴파일 타임에 검증하는 테스트 통과 |
| AC-T-9 | RG-4 | RootView 에 active workspace 존재 | 사용자가 신규 TerminalSurface 생성 트리거 | content_area 가 Empty State 대신 TerminalSurface 렌더, 첫 프레임 표시 ≤ 200 ms |
| AC-T-10 | RG-4 | terminal-spike 실행 중, shell prompt 표시 | Cmd+C (macOS) 누름, selection 비어있음 | SIGINT 가 PTY 에 전송되어 shell 에 `^C` 표시, 시스템 클립보드 값은 변하지 않음 |
| AC-T-11 | RG-4 | terminal-spike 실행 중, 마우스로 텍스트 선택 | Cmd+C (macOS) / Ctrl+Shift+C (Linux) 누름 | 선택된 텍스트가 OS 클립보드에 복사됨 (`pbpaste` / `xclip -o` 로 검증) |
| AC-T-12 | RG-5 | PR 이 이 SPEC 의 구현 커밋 포함 | GitHub Actions CI 트리거 | macOS + Linux 양쪽에서 Zig 설치 스텝 + 기존 4 gate + `terminal-spike` 빌드 gate 모두 통과 |
| AC-T-13 | RG-5 | CI 에서 의도적으로 Zig 스텝 실패 시뮬 | Zig action 을 invalid version 으로 변경 후 실행 | rust job 이 Zig 설치 단계에서 실패, 이후 cargo 스텝은 skip |
| AC-T-14 | RG-6 | moai-studio-terminal 테스트 suite | `cargo test -p moai-studio-terminal --lib` | 신규 단위 테스트 ≥ 10 개 통과, 기존 moai-core 289 tests regression 0 |

---

## 4. 마일스톤

| MS | 제목 | 우선순위 | Requirement 커버 | 선행 조건 |
|----|------|----------|------------------|-----------|
| MS-T-1 | Dependency + Build 체인 확립 | High | RG-1 전체 | SPEC-V3-001 완료 (전제) |
| MS-T-2 | PTY + VT wrapper (headless) | High | RG-2, RG-3, RG-6 | MS-T-1 |
| MS-T-3 | GPUI TerminalSurface + 입력 경로 | High | RG-4 | MS-T-2 |
| MS-T-4 | CI matrix 확장 + 스파이크 예제 | High | RG-5 | MS-T-1 (Zig 설치) + MS-T-3 (예제 바이너리 존재) |
| MS-T-5 | End-to-end 스모크 검증 | Medium | AC-T-4, AC-T-5, AC-T-6, AC-T-11 교차 검증 | MS-T-2, MS-T-3, MS-T-4 |

순서 제약: MS-T-1 은 모든 후속 작업 선행. MS-T-2 와 MS-T-3 은 logical 의존 (Surface 가 VtState 를 소유) 이지만 mock 기반으로 병렬 가능. MS-T-4 는 MS-T-3 완료 후 예제 바이너리 path 가 확정되어야 활성화. MS-T-5 는 최종 통합 검증.

---

## 5. 비기능 요구사항

### 5.1 성능

- Terminal 윈도우 첫 프레임 (`cx.notify` → paint) ≤ **200 ms** (AC-T-9)
- `yes | head -1000000` 수준 고출력 중 평균 프레임 간격 ≤ **16.67 ms** (60 fps, AC-T-7)
- Idle 상태 (prompt 대기) CPU 사용률 ≤ **1%** (process 기준, `top -pid`)
- 키 입력 → 화면 echo 왕복 latency ≤ **32 ms** (2 frames)

### 5.2 메모리

- Scrollback 기본 10,000 rows 기준 프로세스 RSS 증분 ≤ **60 MB/terminal**
- Scrollback 상한 100,000 rows 사용자 설정 가능
- FD 누수 금지: 테스트 종료 시 FD 수 시작점 이하 (AC-T-6, RG-6)

### 5.3 빌드 시간

- Cold `cargo build -p moai-studio-terminal` (libghostty-vt Zig 빌드 포함) ≤ **5 분** (CI macos-14 및 ubuntu-22.04 기준)
- Warm (Zig cache hit) ≤ **90 초**
- `mlugg/setup-zig@v2` cache hit 시 CI 추가 시간 ≤ **10 초**

### 5.4 안정성 / 이식성

- 기존 `moai-core` 289 tests 회귀 0
- MSRV 상향 (1.85 → 1.93) 이후 다른 crate 빌드 실패 0
- macOS 14+ (Apple Silicon) + Ubuntu 22.04+ (x86_64) 양쪽에서 동일 바이너리 동작 확인
- Windows 빌드는 본 SPEC 범위 아님 (Phase 7) — 하지만 portable-pty 추상화로 API 를 깨지 않음

### 5.5 관측성

- `tracing` crate 사용, 최소 로그 level:
  - `info!`: shell spawn/exit, surface 생성
  - `warn!`: reader thread panic 복구, FD 지연 해제
  - `debug!`: 바이트 배칭 카운터, notify 호출
- panic 시 terminal stack trace 를 structured log 로 기록

---

## 6. 의존성 및 제약

### 6.1 외부 의존성 (신규)

| Crate | 버전 / 커밋 | 비고 |
|-------|-------------|------|
| `libghostty-vt` | `git = "https://github.com/Uzaaft/libghostty-rs"`, `rev = "dfac6f3e8e7ff1567a7dead6639ef36c42e4f15a"` | 2026-04-20 HEAD, MIT/Apache-2.0 |
| `portable-pty` | `0.9` (crates.io) | MIT, wezterm monorepo |
| `copypasta` | `0.10` (crates.io) | 클립보드. Phase 2 Run 시 gpui built-in 대체 가능성 재평가 |

### 6.2 내부 의존성

- `crates/moai-studio-terminal` (scaffold 존재) — 본 SPEC 구현 대상
- `crates/moai-studio-ui` (GPUI 0.2.2) — `TerminalSurface` 컴포넌트 추가 대상
- `crates/moai-studio-workspace` — 변경 없음

### 6.3 시스템/도구 제약

- **Zig 0.15.1** 개발자 로컬 + CI PATH 필수
- **Rust stable** (2026-04 기준 1.93+) — CI `dtolnay/rust-toolchain@stable` 자동 수용
- **macOS 14+ (Darwin 23+)** / **Ubuntu 22.04+ (glibc 2.35+)**
- Windows 빌드 경로는 건드리지 않음 (Phase 7)

### 6.4 Git / Branch 제약

- 본 SPEC 구현은 `feat/v3-scaffold` 브랜치 (또는 `feat/v3-terminal-core` 서브 브랜치) 에서 진행
- `main` 직접 커밋 금지
- libghostty-rs upstream 이 breaking change 릴리즈 시 rev 갱신은 별도 PR + research.md 갱신

---

## 7. 테스트 전략

### 7.1 단위 테스트 (≥ 10 개, AC-T-14)

- `pty::tests::spawn_echo_shell_returns_prompt` (mock PtySystem, in-memory buffer)
- `pty::tests::child_exit_releases_fds` (FD 카운트 assert)
- `vt::tests::feed_empty_bytes_is_noop`
- `vt::tests::feed_simple_ascii_produces_grid_cells`
- `vt::tests::feed_ansi_color_escape_sets_fg_color`
- `vt::tests::resize_updates_grid_dimensions`
- `vt::tests::vtstate_is_not_send` (`static_assert` 또는 `impl` trait 검증)
- `key::tests::gpui_arrow_up_encodes_csi_a`
- `key::tests::ctrl_c_encodes_0x03`
- `surface::tests::empty_selection_cmd_c_sends_sigint` (macOS only #[cfg])
- `surface::tests::nonempty_selection_cmd_c_writes_clipboard_mock`

### 7.2 통합 테스트 (실제 PTY 사용)

- `tests/integration_echo.rs` — 실제 `echo hello` spawn, 출력 capture, 비교 (platform-gated by `#[cfg(unix)]`)
- `tests/integration_exit.rs` — spawn + exit, FD 누수 검증 (lsof 기반 pre/post 카운트)

### 7.3 CI 스모크 (AC-T-12, AC-T-13)

- `cargo build -p moai-studio-terminal --example terminal-spike` (링크 성공만 검증, headless 불가능한 GPUI 부분은 skip)
- Zig 설치 실패 시뮬 테스트는 별도 workflow file (`ci-rust-negative.yml`, 본 SPEC 범위 외 선택적)

### 7.4 성능 벤치 (AC-T-7)

- `benches/vt_throughput.rs` — criterion 기반, 100,000 byte ANSI stream feed → snapshot 획득 시간 측정
- 목표: 100K bytes throughput ≥ 5 MB/s/thread

### 7.5 수동 검증

- macOS: `cargo run -p moai-studio-terminal --example terminal-spike` → zsh prompt → `ls`, `echo`, `exit` 시나리오
- Linux: 동일 + `bash` default 확인
- 키보드: arrow, F1-F12, Home/End, Ctrl+C (selection 없음/있음), Cmd+C/Ctrl+Shift+C 클립보드

---

## 8. 참조 문서

- `.moai/specs/SPEC-V3-002/research.md` — 12 결정점 + 5 OPEN QUESTIONS + Sources (본 SPEC 의 설계 근거 일체)
- `.moai/specs/SPEC-V3-001/spec.md` + `progress.md` — 전제 scaffold 상태 및 rescope 근거
- `.moai/design/master-plan.md` §Phase 2 — Terminal Core 설계 방향
- `crates/moai-studio-terminal/Cargo.toml` (line 14-19) — TODO 주석 = 구현 시작점
- `.github/workflows/ci-rust.yml` (line 70-74) — Zig 설치 TODO 스텝
- [github.com/Uzaaft/libghostty-rs](https://github.com/Uzaaft/libghostty-rs) @ `dfac6f3e`
- [github.com/Uzaaft/awesome-libghostty](https://github.com/Uzaaft/awesome-libghostty)
- [docs.rs/portable-pty/0.9.0](https://docs.rs/portable-pty/0.9.0/portable_pty/)
- [codeberg.org/mlugg/setup-zig](https://codeberg.org/mlugg/setup-zig) v2.2.1

---

## 9. Exclusions (What NOT to Build in this SPEC)

다음 항목은 명시적으로 본 SPEC 범위 밖이며 별도 SPEC 에서 다룬다.

1. **Tab UI / Pane split** — SPEC-V3-003 예정. 본 SPEC 은 단일 TerminalSurface 만 다룬다. 사유: 다중 pane 은 workspace state + 키 routing + focus 관리가 별도 도메인.
2. **Shell configuration loader** (`.zshrc`, `.bashrc` 파싱 / modification) — OS shell 에 위임. 우리는 `$SHELL` 을 spawn 만 하고 설정은 건드리지 않는다.
3. **Terminal scrollback UI** (마우스 휠 히스토리 스크롤, 검색) — Phase 2.5. 본 SPEC 은 libghostty-vt 가 제공하는 scrollback buffer 보유까지만 하고 UI 는 제공하지 않는다.
4. **SSH / remote terminal / OSC 52 원격 클립보드** — Phase 3+. 본 SPEC 에서 libghostty-vt 가 OSC 52 시퀀스를 수신해도 silent drop.
5. **Windows 빌드** — Phase 7 (GPUI Windows GA 대기). portable-pty 추상화는 유지하되 테스트·CI·스모크 모두 대상 아님.
6. **tmux 호환성 검증 / 대체** — Phase 2.5+. libghostty-vt 의 screen emulation 이 tmux session 내부에서 동작하는지 여부는 본 SPEC 에서 검증하지 않는다.
7. **Command Palette / Shortcut customization** — Phase 5. 본 SPEC 의 키 매핑은 하드코딩으로 시작한다.
8. **Font / theme customization UI** — Phase 4. 본 SPEC 은 시스템 기본 monospace font + libghostty-vt 기본 color palette 사용.
9. **LSP hover / code intelligence overlay** — Phase 4+. 터미널은 순수 text 렌더만.
10. **Mermaid / image rendering inside terminal** — Phase 4. 본 SPEC 은 Unicode + SGR color 까지만.

---

## 10. 용어 정의

- **libghostty-vt**: Ghostty 에서 추출된 zero-dependency C/Zig 터미널 VT state/parser 라이브러리. Send/Sync 가 아님 (`!Send + !Sync`).
- **libghostty-rs**: libghostty-vt 의 Rust FFI 바인딩 crate (Uzaaft 유지). 본 SPEC 은 `libghostty-vt` subcrate 를 직접 의존.
- **libghostty-vt-sys**: raw FFI bindings. `libghostty-vt` 가 transitive 로 가져오며 우리는 직접 의존하지 않는다.
- **portable-pty**: wezterm 에서 유래한 cross-platform PTY 추상화. Unix `openpty` 와 Windows ConPTY 를 통합.
- **PTY**: pseudo-terminal. master/slave pair 로 shell 프로세스와 host 애플리케이션 사이의 tty 에뮬레이션 채널.
- **VT state**: Virtual Terminal state. ANSI/CSI/OSC escape sequence parser 의 결과로 유지되는 화면 격자 + 커서 + scrollback.
- **Grid snapshot**: libghostty-vt RenderState 에서 얻는 row/cell iterator. foreground/background color + SGR flag + Unicode grapheme 을 포함.
- **GPUI**: Zed Editor 가 사용하는 Rust-native GPU UI 프레임워크. Metal/Vulkan/D3D 기반.
- **MSRV**: Minimum Supported Rust Version.
- **Pinned commit**: Git 리포지토리의 특정 SHA. 본 SPEC 에서는 `dfac6f3e8e7ff1567a7dead6639ef36c42e4f15a` 를 지칭.
- **Scrollback**: 현재 뷰포트 밖으로 밀려난 출력 라인의 history buffer.
- **FD (File Descriptor)**: Unix process 가 open 한 파일/소켓/PTY 의 정수 핸들. 누수 시 `Too many open files` 유발.

---

Version: 1.0.0 · 2026-04-21
