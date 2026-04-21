# SPEC-V3-002 Acceptance Criteria — Given/When/Then 시나리오

---
spec_id: SPEC-V3-002
version: 0.2.0-draft
created: 2026-04-21
updated: 2026-04-21
author: MoAI (manager-spec)
language: Korean
revision: iter 2 (plan-auditor iter 1 defect 반영 — D3/D4/D6/D7, AC-T-11 신설)
---

## 1. 개요

SPEC-V3-002 spec.md §3 의 11 개 AC (AC-T-1 ~ AC-T-11) 를 Given/When/Then 시나리오로 상세화. 각 시나리오는 **검증 방법 (command, file, metric, log)** 을 명시해 구현/CI 가 그대로 수행 가능하도록 한다. AC-T-11 은 iter 2 에서 plan-auditor D2 (pin-policy AC 부재) 해결을 위해 신설.

[HARD] 본 문서의 모든 성능 metric 은 **검증 방법이 명시된 것만** 포함한다. 검증 불가능한 "빠름", "안정적" 같은 문구는 금지.

---

## 2. AC-T-1 — Cold 빌드 성공 + 시간 제약

### Given
- libghostty-rs 가 `rev = "dfac6f3e8e7ff1567a7dead6639ef36c42e4f15a"` 로 `Cargo.toml` 에 pin 되어 있다
- Zig 0.15.2 가 `mise`/`asdf`/직접 설치로 PATH 에 존재한다
- linux-x86_64 GitHub Actions runner (`ubuntu-22.04`) 이며, **cargo cache 는 비어있다** (cold)
- 빌드 프로파일은 **debug** (dev ergonomics 우선, release 는 별도 bench 로 분리)

### When
```bash
cargo build -p moai-studio-terminal 2>&1 | tee build.log
```
가 실행된다. (`--release` 미사용 — spec.md §3 AC-T-1 과 동일 프로파일)

### Then
1. 빌드 exit code = 0
2. `build.log` 에 `warning:` 문자열이 나타나지 않는다 (`grep -c "^warning:" build.log` == 0)
3. 빌드 wall-clock ≤ **150 초** (cold, `/usr/bin/time -v` 또는 GitHub Actions step duration). 근거: research.md §4 — Linux cold 의 Zig CI overhead 60–120s (`setup-zig` + libghostty-vt Zig 빌드) + Rust dep compile (tokio full, portable-pty, arboard, gpui). iter 1 의 90s budget 은 Zig-side 하한 (60s) 에 Rust 포함 여유가 없어 상향.
4. Warm run (Swatinem/rust-cache hit) 재시행 시 ≤ **30 초**

### 검증 방법
- CI step `Cold build benchmark` 에서 `/usr/bin/time -v cargo build -p moai-studio-terminal` (debug) 실행 후 `Elapsed (wall clock) time` 추출
- Swatinem/rust-cache 는 별도 PR 에서 warm 경로 확인
- release profile 은 본 AC 범위 밖 — 필요 시 별도 bench / job 으로 분리 (Phase 2.5 검토)

---

## 3. AC-T-2 — Zig 미설치 환경 에러 메시지

### Given
- Zig 실행 파일이 `PATH` 에서 제거된 환경 (`env -i PATH=/usr/bin:/bin cargo build ...`)

### When
```bash
env -i PATH=/usr/bin:/bin cargo build -p moai-studio-terminal 2>&1 | tee err.log
```

### Then
1. exit code ≠ 0 (정확히 1 또는 build.rs panic 신호)
2. `err.log` 에 다음 문자열이 **정확히** 포함된다:
   ```
   Zig 0.15.x required — install via mise/asdf/ziglang.org
   ```
3. `err.log` 에 패닉 backtrace (`note: run with RUST_BACKTRACE=1`) 는 **포함되지 않는다** (build.rs 에서 `println!("cargo:warning=...")` + `std::process::exit(1)` 사용)

### 검증 방법
- 통합 테스트 `tests/build_rs_missing_zig.rs` 또는 CI step `Missing Zig sanity check`
- `grep -F "Zig 0.15.x required — install via mise/asdf/ziglang.org" err.log`

---

## 4. AC-T-3 — 스파이크 실행 + prompt 렌더 latency

### Given
- macOS 14 또는 Linux (Ubuntu 22.04) 환경
- `$SHELL=/bin/zsh` (macOS) 또는 `$SHELL=/bin/bash` (Linux)
- `cargo build -p moai-studio-terminal --release` 성공 상태

### When
```bash
cargo run --release --example ghostty-spike 2>&1 | tee spike.log
```
가 실행되어 GPUI 윈도우가 뜨고, 내부적으로 `tracing` span `"spawn_shell" -> "first_prompt_render"` 가 측정된다.

### Then
1. GPUI 윈도우가 1 개 생성된다 (macOS `osascript` 또는 Linux `wmctrl -l` 로 확인)
2. spawn 부터 prompt 첫 셀 렌더까지의 경과 시간 ≤ **200 ms** (criterion bench `bench_shell_spawn_to_first_prompt` 의 p99)
3. 렌더된 Grid 의 마지막 비어있지 않은 row 가 프롬프트 정규식 중 하나와 매칭된다:
   - bash/zsh 기본: `^[$#%] ?$` 또는 `.*@.*:.*[$#] ?$`
   - starship 등 custom: 정규식 확장은 Phase 2.5 로 이관, 본 AC 는 기본 prompt 만

### 검증 방법
- Criterion bench `crates/moai-studio-terminal/benches/shell_spawn.rs::bench_shell_spawn_to_first_prompt`
- 통합 테스트 `tests/spike_prompt_match.rs` — headless 모드에서 Grid snapshot 검사

---

## 5. AC-T-4 — 키 입력 → echo → 렌더 p99 latency

### Given
- AC-T-3 의 스파이크 윈도우가 실행 중이고 TerminalSurface 에 포커스가 있다
- 쉘 prompt 가 준비된 상태 (`$ `)

### When
- 키 입력 시퀀스 `e`, `c`, `h`, `o`, ` `, `h`, `e`, `l`, `l`, `o`, `\n` 이 GPUI `simulate_key_event` 로 전송된다
- 각 키 이벤트마다 `tracing` span `"key_event" -> "grid_update"` 가 측정된다

### Then
1. 쉘이 `echo hello` 명령을 실행한 후 다음 row 에 `hello` 가 렌더된다
2. 각 키 이벤트의 key-to-render latency **p99 ≤ 16 ms** (60fps frame budget, 1000 회 반복 criterion bench)
3. 평균 (mean) latency ≤ 8 ms (추가 참고 metric, 목표치 아님)

### 검증 방법
- Criterion bench `crates/moai-studio-terminal/benches/key_echo.rs::bench_key_echo_latency`
- 통합 테스트 `tests/key_echo_hello.rs` — MockShell 환경에서 echo 결과 Grid assertion

---

## 6. AC-T-5 — 쉘 exit 시 PTY FD 회수 (고아 방지)

### Given
- AC-T-3 의 스파이크 윈도우가 실행 중이고 쉘 PID 가 알려져 있다 (`spike_pid`)
- spawn 직후 `lsof -p <spike_pid> | grep -c ptmx` = `N` (일반적으로 1 master + 1 slave)

### When
- 사용자가 터미널에 `exit\n` 을 입력하거나 외부에서 `kill -HUP <shell_pid>` 를 실행한다

### Then
1. GPUI 윈도우가 **1 초 이내** 종료된다 (`tracing` span `"shell_exit" -> "window_closed"` 측정)
2. 윈도우 종료 직후 `lsof -p <spike_pid> | grep -c ptmx` = 0 또는 spike 프로세스 자체가 이미 종료됨
3. spawn 전 대비 ptmx FD **delta = 0** (parent 프로세스 기준)

### 검증 방법
- 통합 테스트 `tests/pty_fd_cleanup.rs`:
  ```rust
  let before = count_ptmx_fds(parent_pid);
  let child = spawn_spike();
  // ... write "exit\n" to stdin ...
  wait_for_exit(child, Duration::from_secs(1));
  let after = count_ptmx_fds(parent_pid);
  assert_eq!(before, after);
  ```
- macOS/Linux 에서 `/proc/self/fd` 또는 `lsof` 호출

---

## 7. AC-T-6 — RootView content_area 에 TerminalSurface 렌더

### Given
- SPEC-V3-001 의 RootView 가 초기 상태로 렌더된 상태
- 사용자가 "Create Workspace" → 폴더 선택으로 active workspace 를 1 개 생성한 상태
- content_area 에 Empty State CTA 가 표시되고 있다

### When
- 사용자가 workspace 내에서 "Open Terminal" 액션 (키보드 단축키 또는 메뉴) 을 트리거한다

### Then
1. content_area 의 Empty State CTA 엘리먼트가 DOM (GPUI element tree) 에서 제거된다
2. 해당 위치에 `TerminalSurface` GPUI 컴포넌트가 렌더된다
3. TerminalSurface 의 Grid 첫 번째 셀 (0, 0) 부터 렌더가 시작된다 (빈 공간 없음)
4. 쉘 prompt 가 AC-T-3 의 정규식과 매칭되는 위치에 표시된다

### 검증 방법
- 통합 테스트 `crates/moai-studio-ui/tests/root_view_terminal_mount.rs` — GPUI element tree inspection API (또는 test harness mock)
- `cargo test -p moai-studio-ui --test root_view_terminal_mount`

---

## 8. AC-T-7 — CI matrix 전체 wall-clock

### Given
- GitHub Actions PR 트리거
- `actions/cache@v4` 의 Zig cache key (`zig-0.15.2-${{ runner.os }}-${{ runner.arch }}`) 가 이미 존재 (cache hit)
- Swatinem/rust-cache 도 hit 상태

### When
- PR 이 push 되어 CI workflow `ci-rust.yml` 이 실행된다
- 4 job: `rust (macOS)`, `rust (Linux)`, `smoke (macOS)`, `smoke (Linux)` 이 매트릭스 병렬 실행

### Then
1. 4 job 모두 exit code 0 (conclusion: `success`)
2. 4 job 의 **최대 wall-clock** (병렬이므로 max) ≤ **5 분** (300 초)
3. smoke job 의 로그에 `Scaffold OK` 문자열이 포함된다 (AC-T-8 의 `--headless` 경로)

### 검증 방법
- `gh run list --workflow=ci-rust.yml --json conclusion,timing` 로 최신 run 확인
- `gh run view <run-id> --json jobs` 에서 각 job 의 `startedAt`/`completedAt` 차이 계산
- 또는 GitHub Actions UI 의 wall-clock 수동 확인

---

## 9. AC-T-8 — 테스트 suite ≥ 10 + 유형별 커버

### Given
- `moai-studio-terminal` crate 가 완전히 빌드된 상태

### When
```bash
cargo test -p moai-studio-terminal --all-targets 2>&1 | tee test.log
```

### Then
1. exit code = 0
2. `test result: ok. N passed` 에서 **N ≥ 10**
3. 다음 4 카테고리 테스트가 모두 포함된다:
   - **(a) MockPty**: `pty::mock::MockPty` 를 사용한 fd clone + 스크립트 응답 테스트 (최소 2 개)
   - **(b) libghostty-vt Grid assertion**: cell iteration, UTF-8 multibyte (`"한글"` 또는 `"café"`), CSI control sequence (`\x1b[2J`, `\x1b[H`) 테스트 (최소 3 개)
   - **(c) Adaptive buffer**: 4KB↔64KB 전환 단위 테스트 (최소 2 개: `transitions_to_64k_on_burst`, `returns_to_4k_after_burst`)
   - **(d) Pty trait contract**: trait 메서드 일관성 테스트 (최소 2 개: `set_window_size_propagates`, `is_alive_after_exit_returns_false`)

### 검증 방법
- CI step `Test count enforcement`:
  ```bash
  count=$(grep -oE '[0-9]+ passed' test.log | head -1 | grep -oE '[0-9]+')
  [ "$count" -ge 10 ] || exit 1
  ```
- 카테고리 커버는 테스트 파일명 규칙 (`libghostty_api_compat.rs`, `pty_contract.rs`, `worker_adaptive_buffer.rs`) 으로 `find tests/ -name '*.rs'` 검증

---

## 10. AC-T-9 — 대용량 stdout 시 p99 PTY read ≤ 5ms

### Given
- `moai-studio-terminal` 이 스파이크 예제로 실행 중이고 TerminalSurface 가 PtyEvent 를 소비하는 상태
- 1 MB 규모의 랜덤 base64 문자열이 stdin 또는 파일에서 준비된 상태

### When
```bash
# spike 내에서 다음 명령 실행 (또는 criterion bench 로 재현)
head -c 1048576 /dev/urandom | base64
```
- PTY stdout 이 buffer burst 를 일으키고, worker 가 적응형 정책을 적용한다

### Then
1. **1 MB byte count** (정확히 1,048,576 bytes) 가 worker 를 거쳐 libghostty-vt `Terminal::feed()` 에 누적 전달되었으며, 해당 byte count 가 Grid state 에 모두 반영됨을 **byte-level assertion** 으로 확인 (테스트 코드에서 `total_bytes_fed == 1_048_576` 검증). scrollback UI / line count 기반 검증은 사용하지 않으며, scrollback UI 자체는 spec.md §6 Exclusions 에 의해 Phase 2.5 로 이관된 상태.
2. PTY read cycle (worker `read_available` 1 회 호출) **p99 ≤ 5 ms** (criterion bench `bench_pty_burst_read`, 1000 sample)
3. `tracing` log 에 다음 2 개 이벤트가 나타난다:
   - `buffer_size_transition{from=4096, to=65536}` — burst 감지 후 64KB 전환
   - `buffer_size_transition{from=65536, to=4096}` — burst 종료 후 4KB 복귀
4. `mpsc::unbounded_channel` 이 drop 없이 모든 `PtyEvent::Output` 을 수신 (channel metric 또는 assertion)

### 검증 방법
- Criterion bench `crates/moai-studio-terminal/benches/pty_burst.rs::bench_pty_burst_read`
- 통합 테스트 `tests/worker_burst_1mb.rs`:
  ```rust
  let events = run_worker_against_script("head -c 1048576 /dev/urandom | base64");
  assert!(events.len() > 0);
  assert!(logs_contain("buffer_size_transition{from=4096, to=65536}"));
  assert!(logs_contain("buffer_size_transition{from=65536, to=4096}"));
  ```

---

## 11. AC-T-10 — Windows compile-gate

### Given
- 개발자 로컬 환경 또는 on-demand CI 에서 Windows target (`x86_64-pc-windows-msvc`) 이 설치되어 있다 (`rustup target add x86_64-pc-windows-msvc`)
- 실제 Windows runner 는 본 SPEC 범위 밖 (CI matrix 에 미포함)

### When
```bash
cargo check --target x86_64-pc-windows-msvc -p moai-studio-terminal 2>&1 | tee win-check.log
```

### Then
1. `cargo check --target x86_64-pc-windows-msvc` exit code = 0 (trait 정의 + stub 선언은 컴파일 통과)
2. `Pty` trait 자체가 컴파일된다 (trait 정의는 cross-platform)
3. `UnixPty` impl 은 `#[cfg(unix)]` 로 excluded 되어 Windows compile tree 에서 빠진다
4. `ConPtyStub::spawn()` / `ConPtyStub::read_available()` 호출 사이트는 `#[cfg(windows)]` impl block 내 **`compile_error!("ConPtyStub is deferred to Phase 7 (GPUI Windows GA)")` 단일 enforcement** 로 차단된다. `todo!()` panic 은 사용하지 않는다 (iter 1 D4 해결 — OR-clause 제거).
5. `trybuild` 기반 compile-fail 테스트 (`tests/compile_fail/conpty_spawn.rs`) 가 Windows target 에서 호출을 시뮬레이트하고, 컴파일러 stderr 에 `"ConPtyStub is deferred to Phase 7 (GPUI Windows GA)"` 문자열이 **정확히** 포함됨을 검증 (Unix target 에서는 `cargo test --test compile_fail` 가 해당 케이스를 skip 또는 `#[cfg(windows)]` gate).
6. 실제 `cargo build --target x86_64-pc-windows-msvc` 는 **의도적으로 실패** (compile_error! 발동) — Phase 7 까지 deferred.

### 검증 방법
- On-demand CI workflow `ci-windows-check.yml` (수동 트리거) 또는 local developer script
- 로컬 검증:
  ```bash
  rustup target add x86_64-pc-windows-msvc
  cargo check --target x86_64-pc-windows-msvc -p moai-studio-terminal
  echo "check exit=$?"     # expect 0 (trait 정의만 컴파일)
  cargo test -p moai-studio-terminal --test compile_fail
  echo "trybuild exit=$?"  # expect 0 (trybuild 가 compile_error! 메시지를 확인)
  cargo build --target x86_64-pc-windows-msvc -p moai-studio-terminal
  echo "build exit=$?"     # expect non-zero (compile_error! 발동, 의도된 실패)
  ```

---

## 11a. AC-T-11 — libghostty-rs pin-policy CI gate (신설, iter 2)

### Given
- `Cargo.toml` 의 `libghostty-vt = { git = "...", rev = "<SHA>" }` 라인이 수정된 PR
- 수정 유형: `<old_SHA>` → `<new_SHA>` 변경
- `.github/workflows/ci-rust.yml` 에 `pin-policy-guard` job 이 추가된 상태
- spec.md §2 RG-V3-002-1 의 4 bump 요건 (a~d) 이 정책으로 확정

### When
```yaml
# .github/workflows/ci-rust.yml (발췌, plan.md T9 참조)
pin-policy-guard:
  if: contains(github.event.pull_request.changed_files, 'crates/moai-studio-terminal/Cargo.toml')
  runs-on: ubuntu-22.04
  steps:
    - uses: actions/checkout@v4
      with: { fetch-depth: 2 }
    - name: Check rev changed
      id: rev-diff
      run: |
        if git diff HEAD~1 -- crates/moai-studio-terminal/Cargo.toml | grep -q '^[+-].*rev\s*='; then
          echo "changed=true" >> "$GITHUB_OUTPUT"
        fi
    - name: (i) Re-run characterization
      if: steps.rev-diff.outputs.changed == 'true'
      run: cargo test -p moai-studio-terminal --test libghostty_api_compat
    - name: (ii) Assert HISTORY includes old + new SHA
      if: steps.rev-diff.outputs.changed == 'true'
      run: scripts/ci/check-history-sha.sh  # plan.md T9 의 가드 스크립트
    - name: (iii) Check wrapper-external file changes
      if: steps.rev-diff.outputs.changed == 'true'
      run: scripts/ci/check-wrapper-scope.sh  # annotation-cycle-required 라벨 부착 로직
```

### Then
1. PR 에 `rev =` 수정이 포함될 때 `pin-policy-guard` job 이 실행된다 (rev 미변경 PR 은 skip)
2. `cargo test -p moai-studio-terminal --test libghostty_api_compat` 가 exit 0 으로 **PASS** (characterization 재실행 성공)
3. `git diff HEAD~1 .moai/specs/SPEC-V3-002/spec.md` 의 HISTORY 섹션 diff 가 `<old_SHA>` 와 `<new_SHA>` 문자열을 **모두** 포함함을 `scripts/ci/check-history-sha.sh` 가 검증 (grep 2회 성공)
4. `git diff --name-only HEAD~1` 결과에 `src/libghostty_ffi.rs` (wrapper) 외 `crates/moai-studio-terminal/src/**/*.rs` 파일이 포함되면:
   - `scripts/ci/check-wrapper-scope.sh` 가 GitHub API 를 통해 PR 에 `annotation-cycle-required` 라벨을 부착
   - CI 로그에 `::warning::Wrapper-external files touched: annotation cycle should re-open` 을 출력
   - job 자체는 fail 아님 (경고 only)
5. (i), (ii), (iii) 중 하나라도 **failing** 조건 발생 시 `pin-policy-guard` job exit 1 → merge 차단:
   - (i) characterization 테스트 실패
   - (ii) HISTORY 에 old 또는 new SHA 누락
   - (iii) 은 경고 only (차단 아님) — 단 `annotation-cycle-required` 라벨 자동 부착

### 검증 방법
- 테스트 시나리오 1 — 정상 bump PR:
  - `Cargo.toml` rev 만 수정 + spec.md HISTORY 에 이전/신규 SHA 추가 + characterization 통과 → job PASS
- 테스트 시나리오 2 — HISTORY 누락 PR:
  - rev 수정 + characterization 통과 + **HISTORY 미업데이트** → job FAIL (조건 ii 위반)
- 테스트 시나리오 3 — characterization 회귀 PR:
  - rev 수정 + HISTORY 업데이트 + characterization **FAIL** → job FAIL (조건 i 위반)
- 테스트 시나리오 4 — wrapper 외 파일 수정 PR:
  - rev 수정 + HISTORY 업데이트 + characterization 통과 + `src/pty/unix.rs` 동시 수정 → job PASS + `annotation-cycle-required` 라벨 자동 부착 + warning 로그

### 주석
- AC-T-11 은 spec.md §2 RG-V3-002-1 의 "pin 정책" (a~d) 을 **실제로 강제** 하기 위한 CI gate. iter 1 D2 에서 auditor 가 지적한 "정책 prose 만 존재, enforcement 없음" 을 해결.
- `scripts/ci/check-history-sha.sh`, `scripts/ci/check-wrapper-scope.sh` 의 상세는 plan.md T9 에 명시 (실제 파일 생성은 Run phase).

---

## 12. Edge Cases (보조 검증, AC 번호 없음)

본 섹션은 공식 AC 에 포함되진 않지만 구현 중 반드시 확인해야 하는 경계 조건:

### EC-1: 빈 선택 영역에서 Cmd+C

- Given: TerminalSurface 포커스, 선택 영역 없음
- When: macOS 에서 `Cmd+C` 입력
- Then: 클립보드 변경 없음 (arboard 호출 skip), SIGINT 도 전송되지 않음

### EC-2: PTY 가 살아있는 채 윈도우 강제 종료

- Given: 쉘 실행 중 (PID `shell_pid`), GPUI 윈도우 open
- When: 사용자가 Cmd+Q / Alt+F4 로 윈도우 강제 종료
- Then: 부모 프로세스가 SIGTERM → 1초 후 SIGKILL 순서로 쉘 정리, ptmx FD 회수 (AC-T-5 와 동일 기준)

### EC-3: 윈도우 리사이즈 시 PTY propagate

- Given: TerminalSurface 가 80×24 로 렌더 중
- When: 윈도우 리사이즈로 120×40 이 된다
- Then: `PtyEvent::Resize { rows: 40, cols: 120 }` 가 worker 로 전달되고, `pty.set_window_size(40, 120)` 호출 (SIGWINCH). 쉘 프롬프트 재렌더 확인

### EC-4: OSC 52 escape sequence silently ignore

- Given: PTY 가 `\x1b]52;c;SGVsbG8=\x07` (OSC 52 clipboard set "Hello") 를 출력
- When: libghostty-vt parser 가 처리
- Then: 로컬 arboard 에 **"Hello" 가 기록되지 않는다** (OSC 52 는 Phase 3 로 이관). 에러/경고 없이 silently ignore.

---

## 13. Definition of Done (DoD)

SPEC-V3-002 는 다음 조건 **모두** 충족 시 완료 (Phase 2 종결):

- [ ] AC-T-1 ~ AC-T-11 (11 개) 전체 Green
- [ ] EC-1 ~ EC-4 (4 개) edge case 통합 테스트 통과
- [ ] 신규 테스트 ≥ 10 + criterion bench 2 개 (`bench_pty_burst_read`, `bench_key_echo_latency`)
- [ ] SPEC-V3-001 baseline 248 tests 회귀 없음
- [ ] CI matrix 4 job (macOS/Linux × rust/smoke) 모두 Green, wall-clock ≤ 5 분
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 warnings
- [ ] `cargo fmt --all -- --check` 통과
- [ ] Windows compile-gate (`cargo check --target x86_64-pc-windows-msvc`) 통과
- [ ] MX tag 14+ 개 추가 (plan.md §6 기준: ANCHOR 5, WARN 2, NOTE 6, TODO 1)
- [ ] `spec.md` / `plan.md` / `acceptance.md` 3 문서 정합성 유지
- [ ] progress.md 에 Phase 2 완료 기록 + 다음 Phase (SPEC-V3-003) 포인터

---

## 14. 참조

- `.moai/specs/SPEC-V3-002/spec.md` §3 — AC-T-1 ~ AC-T-10 원본
- `.moai/specs/SPEC-V3-002/plan.md` — T1~T10 작업 분해
- `.moai/specs/SPEC-V3-002/research.md` §6.1 — AC 세부화 제안 원천
- `.claude/rules/moai/workflow/workflow-modes.md` — DoD 기준 (TRUST 5)

---

Version: 0.2.0-draft · 2026-04-21 (iter 2 revision)
