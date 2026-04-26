# SPEC-V3-FS-WATCHER-001 Implementation Plan — moai-fs 파일 감시자 테스트 결정성 + tmux-test CI 버킷 격리

작성: MoAI (manager-spec, 2026-04-27)
브랜치 (현행 SPEC 작성): `feature/ci-linux-git-walkup-fix`
브랜치 (implement 진입 시): `feature/SPEC-V3-FS-WATCHER-001-bucket-isolation` (또는 USER-DECISION-FW-B 결정 옵션 명을 반영한 변형 — CLAUDE.local.md §1.3 명명 규칙 준수)
범위: SPEC-V3-FS-WATCHER-001 spec.md 의 RG-FW-1 ~ RG-FW-2, AC-FW-1 ~ AC-FW-6 를 MS-1 / MS-2 / MS-3 으로 분할 구현.
선행: 없음 (standalone SPEC). USER-DECISION-FW-A + FW-B 결정 후 implement 진입.

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-27 | 초안 작성. T1~T9 분해 (MS-1: T1~T2 / MS-2: T3~T5 / MS-3: T6~T9). USER-DECISION-FW-A / FW-B 게이트 PENDING 명시. (Recommended) 옵션 표기. 외부 차단 0. 코드베이스 변경 범위는 spec.md §13.2 와 정합. |
| 1.1.0 | 2026-04-27 | USER-DECISION-FW-A RESOLVED → (a) A3 polling with bounded retry. USER-DECISION-FW-B RESOLVED → (a) B1 cargo test name filter. USER-DECISION-FW-C OMITTED. T3/T4/T6/T7 의 ACTIVE 옵션 표기. T8 SKIP (B1 시 코드 변경 0). spec.md status draft→ready 동조. |

---

## 1. Out of Scope (본 SPEC 이 명시적으로 변경하지 않는 것)

본 plan.md 의 모든 task 는 다음 영역을 변경하지 않는다 (spec.md §3.2 N1~N10 carry):

- `crates/moai-fs/src/lib.rs` 의 `FsWatcher` / `FsEvent` / `FsWatcherError` 공개 API 시그니처 (struct, enum, impl) — 두 테스트의 **내부 로직만** 변경 가능.
- `crates/moai-fs/src/{tree_watcher, watcher, workspace_watcher}.rs` 의 모든 코드 — 본 SPEC 무관.
- `crates/moai-fs/src/lib.rs` 의 `FsEventBus`, `WorkspaceWatcher` re-export — 무변경.
- `crates/moai-fs/Cargo.toml` 의 `[dependencies]` — 무변경 (USER-DECISION-FW-C (a) default 가정). `[dev-dependencies]` 도 무변경 (notify, tokio test-util, tempfile 이미 가용).
- `crates/moai-fs/Cargo.toml` 의 `[features]` — USER-DECISION-FW-B 가 (b) cfg feature flag 로 결정될 경우만 추가, 그 외 무변경.
- `.github/workflows/ci-v3-pane.yml` 의 `tmux-test` job 외 다른 job (`fmt`, `clippy`, `test`, `bench-smoke`) — 무변경.
- `.github/workflows/ci-v3-pane.yml:240~311` 의 `tmux-test` job 의 runner / matrix / Rust toolchain / tmux 설치 / cache step — 무변경. 변경 대상은 line 312~313 (`Gate — cargo test`) step 만.
- 다른 `.github/workflows/*.yml` 파일 — 무변경.
- `crates/moai-fs/Cargo.toml` 의 `notify = "7"` — 무변경 (USER-DECISION-FW-C (a)).
- `CLAUDE.local.md` §2.1 의 branch protection 운영 변경 — 본 SPEC 외 운영 작업. 본 SPEC 완료 후 별도 운영 task 로 분리.
- 다른 SPEC (`SPEC-V3-001` ~ `SPEC-V3-015`, `SPEC-V3-011`) 의 산출물 — 무관.
- moai-studio 의 다른 crate (`moai-studio-terminal`, `moai-studio-workspace`, `moai-studio-ui` 등) — 무관.

**RESOLVED 결정 영향 (v1.1.0)**: 본 SPEC 의 RESOLVED 결정 (A3 + B1) 은 코드 변경 ~10 LOC + CI yaml 수정만 요구. 신규 Cargo dependency 도입 없음. T8 (Cargo.toml feature 또는 신규 tests/ 파일) 은 B1 시 SKIP.

---

## 2. Milestone × Task 표

| Task | Milestone | 책임 영역 | 산출 파일 (변경/신규) | 의존 | AC |
|------|-----------|----------|----------------------|-----|----|
| **T1** | MS-1 | USER-DECISION-FW-A (Axis A 패턴 결정) | (게이트, 결정 기록) | — | (게이트) |
| **T2** | MS-1 | USER-DECISION-FW-B (Axis B 격리 방식 결정) + tmux 테스트 enumeration | (게이트, 결정 기록 + grep 결과) | T1 | (게이트) |
| **T3** | MS-2 | `test_detect_file_creation` 결정성 패턴 적용 | `crates/moai-fs/src/lib.rs:162~203` | T1 | AC-FW-1 (단일 환경 PASS), AC-FW-4 (worst-case 10초) |
| **T4** | MS-2 | `test_unwatch_stops_events` 결정성 패턴 적용 | `crates/moai-fs/src/lib.rs:205~237` | T1 | AC-FW-2 (단일 환경 PASS), AC-FW-4 |
| **T5** | MS-2 | 로컬 macOS + Linux 검증 | (검증, 코드 변경 없음) | T3, T4 | AC-FW-1, AC-FW-2 |
| **T6** | MS-3 | `tmux-test` job 명령 narrow (USER-DECISION-FW-B 적용) | `.github/workflows/ci-v3-pane.yml:312~313` | T2 | AC-FW-5 |
| **T7** | MS-3 | file watcher 테스트 별도 step 또는 job 추가 | `.github/workflows/ci-v3-pane.yml` (신규 step 또는 신규 job 블록) | T2, T3, T4 | AC-FW-6 |
| **T8** | MS-3 | (옵션, USER-DECISION-FW-B (b)/(c) 시) Cargo.toml feature 또는 신규 tests/ 파일 | `crates/moai-fs/Cargo.toml` 또는 `crates/moai-fs/tests/fs_watcher.rs` | T2 | (REQ-FW-013) |
| **T9** | MS-3 | 100회 연속 CI 실행 stress test + flake rate 검증 | (CI run 통계, progress 기록) | T6, T7 | AC-FW-3 |

총 task 9 건. 외부 차단 0. USER-DECISION-FW-C 는 omit (research §4 권고).

---

## 3. T1 — USER-DECISION-FW-A (테스트 결정성 패턴)

### 3.1 게이트 호출

[USER-DECISION-RESOLVED: fw-a-pattern → (a) A3 polling with bounded retry, 2026-04-27]

**RESOLVED → A3 ACTIVE.** 5초 deadline + 50~100ms 폴링 간격 + first-match-wins. 코드 변경 ~10 LOC, dep 0. T3 / T4 의 §5.2 (a) / §6.2 (a) 가이드를 ACTIVE 패턴으로 적용.

질문 (AskUserQuestion, 보존 — 향후 escalation 시 재참조):
- "moai-fs file watcher 테스트의 결정성 확보 패턴은?"

옵션:
- (a) **권장: A3 (polling with bounded retry)** — 단일 큰 deadline (예: 5초) 안에서 짧은 폴링 간격 (50~100ms) 으로 이벤트 수신 시도. 첫 매칭 즉시 성공. 코드 변경 ~10 LOC. 신규 dependency 0. deterministic upper bound 명시. typical case 100ms 이내 성공.
- (b) A2 (retry loop with exponential backoff) — `100ms, 200ms, 400ms, ...` 간격 재시도. 일정 횟수 (예: 5회) 후 fail. CI 부하 변동 적응. worst-case ~3초.
- (c) A1 (sleep 시간 증가) — `200ms → 1000ms`, `500ms → 2000ms`. 단순 patch. 근본 원인 미해소.
- (d) A4 (notify backend mock) — 신규 trait 도입. 100% deterministic. 단 production OS event 검증 못함, 공개 API 변경 위험.

상태: **RESOLVED (2026-04-27)** — option (a) A3 polling with bounded retry.

### 3.2 결정 기록

```
[USER-DECISION-FW-A RESOLVED: option = (a) A3 polling with bounded retry, date = 2026-04-27, by = user (orchestrator-mediated)]
근거: deterministic upper bound 명시 (5s deadline) + typical case 100ms 내 성공 + 코드 변경 ~10 LOC + 신규 dependency 0. (b) A2 (exponential backoff) 대비 단순. (c) A1 (sleep 증가) 대비 근본 해결. (d) A4 (mock backend) 대비 production OS event 검증 보존.
적용 영향: REQ-FW-001, REQ-FW-002 의 패턴 = A3. T3 / T4 의 §5.2 (a) / §6.2 (a) 가이드 적용.
```

옵션별 가이드 (b/c/d) 는 spec.md §10.1 + 본 plan.md 의 T3 / T4 가이드 섹션에 보존됨 (escalation 시 재참조 가능).

---

## 4. T2 — USER-DECISION-FW-B (CI 버킷 격리 방식) + tmux 테스트 enumeration

### 4.1 사전 enumeration

T2 의 결정 전, MS-1 의 sub-task 로 워크스페이스 전역의 `#[ignore]` 테스트를 enumerate 한다:

```bash
grep -RIn "#\[ignore\]" --include="*.rs" crates/ | head -50
```

기대 결과: tmux 의존 테스트 (예: `integration_tmux_nested`, `ctrl_b_passes_through_to_nested_tmux`) + moai-fs file watcher 테스트 (`test_detect_file_creation`, `test_unwatch_stops_events`) 가 식별된다.

식별된 tmux 테스트의 **함수 이름 + 모듈 path** 를 plan.md 의 본 §4.2 에 기록한다 (B1 또는 B4 결정 시 prefix 컨벤션 확인).

### 4.2 게이트 호출

[USER-DECISION-RESOLVED: fw-b-isolation → (a) B1 cargo test name filter, 2026-04-27]

**RESOLVED → B1 ACTIVE.** tmux-test job 의 명령은 `cargo test ... -- --ignored 'tmux'` substring filter. file watcher 는 신규 step `cargo test -p moai-fs --all-targets -- --ignored` 로 분리. 코드 변경 0, CI yaml 수정만. T6 의 §8.2 (a) / T7 의 §9.2 (a) 가이드를 ACTIVE 패턴으로 적용. **MS-1 의 §4.1 enumeration 단계에서 tmux 테스트 prefix 컨벤션 확인 필수 (R-FW-3 완화).**

질문 (AskUserQuestion, 보존 — 향후 escalation 시 재참조):
- "`tmux-test` 의 `--ignored` 버킷에서 file watcher 테스트를 격리하는 방식은?"

옵션:
- (a) **권장: B1 (cargo test name filter)** — `cargo test ... -- --ignored 'tmux'` (또는 동등 substring) 로 narrow. file watcher 테스트는 별도 step `cargo test -p moai-fs --all-targets -- --ignored` 로 실행. 코드 변경 0, CI yaml 만 수정. tmux 테스트가 prefix 컨벤션을 따르는지 §4.1 enumeration 으로 확인 필요.
- (b) B2 (cfg feature flag) — `#[cfg(feature = "tmux-tests")]` + `#[cfg(feature = "fs-watcher-tests")]`. 명시적, type-safe. Cargo.toml feature 추가 + 모든 ignored 테스트에 cfg 추가. 빌드 multiplication.
- (c) B3 (separate test binary) — `crates/moai-fs/tests/fs_watcher.rs` integration test 로 이동. cargo 표준 패턴. 단 super::* private API 접근 시 추가 노출 필요할 수 있음 (G5 위반 위험).
- (d) B4 (두 별도 CI job) — `tmux-test` job 은 narrow + 신설 `fs-watcher-test` job. 가장 명확한 분리. 단 CI yaml 가장 큰 변경 + job duplication.

상태: **RESOLVED (2026-04-27)** — option (a) B1 cargo test name filter.

### 4.3 결정 기록

```
[USER-DECISION-FW-B RESOLVED: option = (a) B1 cargo test name filter, date = 2026-04-27, by = user (orchestrator-mediated)]
근거: 코드 변경 0 (CI yaml 수정만) + 가장 단순 + tmux 테스트가 substring 매칭 가능한 prefix 컨벤션 (예: `tmux_*`, `integration_tmux_*`) 따른다는 가정. (b) B2 (cfg feature flag) 대비 빌드 multiplication 회피. (c) B3 (separate test binary) 대비 G5 (공개 API 무변경) 위반 위험 회피. (d) B4 (별도 job) 대비 CI yaml 변경 minimal.
적용 영향: REQ-FW-010, REQ-FW-011 의 격리 방식 = B1. REQ-FW-013 (옵션 보조 변경) = SKIP. T8 = SKIP. T6 의 §8.2 (a) + T7 의 §9.2 (a) ACTIVE.
prerequisite: §4.1 enumeration 단계에서 tmux 테스트 prefix 확인 필수.
```

옵션별 가이드 (b/c/d) 는 spec.md §10.2 + 본 plan.md 의 T6 / T7 가이드 섹션에 보존됨.

---

## 5. T3 — `test_detect_file_creation` 결정성 패턴 적용

### 5.1 변경 대상

`crates/moai-fs/src/lib.rs:162~203` (현재 라인 — implement 시 시점 라인 번호로 재확인).

현재 코드 (요약):
```rust
#[tokio::test]
#[ignore]
async fn test_detect_file_creation() {
    let dir = tempdir().expect("...");
    let dir_path = dir.path().to_path_buf();
    let (mut watcher, mut rx) = FsWatcher::new().expect("...");
    watcher.watch(&dir_path).expect("...");

    tokio::time::sleep(Duration::from_millis(200)).await;     // [VARIABLE]

    let new_file = dir_path.join("test_file.txt");
    fs::write(&new_file, "테스트 내용").expect("...");

    let received = timeout(Duration::from_secs(1), async {    // [VARIABLE]
        loop {
            if let Some(event) = rx.recv().await {
                match event {
                    FsEvent::Created(path) | FsEvent::Modified(path) => {
                        if path == new_file { return true; }
                    }
                    _ => continue,
                }
            } else { return false; }
        }
    }).await;

    assert!(received.is_ok(), "타임아웃: ...");
    assert!(received.unwrap(), "파일 생성 이벤트 경로 불일치");
}
```

### 5.2 USER-DECISION-FW-A 옵션별 변환 가이드

**RESOLVED → A3 ACTIVE (2026-04-27).** 아래 (a) 패턴을 적용. (b)/(c)/(d) 는 escalation 시 참조용으로 보존.

#### (a) A3 적용 시 (권장 — ACTIVE)

핵심 변경:
- `tokio::time::sleep(200ms)` 의 init wait 제거 또는 단축 (예: 50ms) — A3 의 폴링이 init 지연도 함께 흡수.
- `timeout(Duration::from_secs(1))` 을 `timeout(Duration::from_secs(5))` 으로 확대 — deterministic upper bound 5초.
- 내부 `loop` 는 그대로 유지 (이미 first-match-wins 패턴).

스케치:
```rust
// 짧은 init grace
tokio::time::sleep(Duration::from_millis(50)).await;

let new_file = dir_path.join("test_file.txt");
fs::write(&new_file, "테스트 내용").expect("...");

// 5s deadline 안에서 first-match 즉시 성공
let received = timeout(Duration::from_secs(5), async {
    loop {
        match rx.recv().await {
            Some(FsEvent::Created(path)) | Some(FsEvent::Modified(path))
                if path == new_file => return true,
            Some(_) => continue,
            None => return false,
        }
    }
}).await;
```

#### (b) A2 적용 시

핵심 변경:
- 외부 retry loop. 각 retry 마다 file write + recv timeout 100/200/400/800/1600ms.
- 첫 retry 성공 즉시 반환.
- 5회 후에도 실패 시 panic.

스케치:
```rust
let mut delay_ms = 100;
let mut succeeded = false;
for attempt in 0..5 {
    fs::write(&new_file, format!("attempt-{}", attempt)).expect("...");
    if let Ok(Some(FsEvent::Created(path) | FsEvent::Modified(path))) =
        timeout(Duration::from_millis(delay_ms), rx.recv()).await
    {
        if path == new_file { succeeded = true; break; }
    }
    delay_ms *= 2;
}
assert!(succeeded, "5회 retry 후에도 이벤트 미수신");
```

#### (c) A1 적용 시

핵심 변경:
- `200ms → 1000ms`, `1s → 5s`. 그 외 동일.

스케치:
```rust
tokio::time::sleep(Duration::from_millis(1000)).await;
// ...
let received = timeout(Duration::from_secs(5), async { ... }).await;
```

#### (d) A4 적용 시

핵심 변경:
- `FsWatcher::new` 에 mock backend 주입을 위한 trait 도입 (별도 task 필요).
- 본 SPEC 의 G5 (공개 API 무변경) 위반 가능성 → USER-DECISION-FW-A (d) 결정 시 별 SPEC 분리 권고. 본 plan.md 는 (a)/(b)/(c) 가정.

### 5.3 검증

- 로컬 실행: `cargo test -p moai-fs --all-targets -- --ignored test_detect_file_creation`
- 100회 반복 (선택): `for i in {1..100}; do cargo test -p moai-fs --all-targets --release -- --ignored test_detect_file_creation || break; done`
- 두 환경 (macOS + Linux) 각각 PASS 확인.

### 5.4 의존

- T1 (USER-DECISION-FW-A) 결정 결과.

### 5.5 AC

- AC-FW-1 (4 환경 단일 PASS), AC-FW-4 (worst-case 10초).

---

## 6. T4 — `test_unwatch_stops_events` 결정성 패턴 적용

### 6.1 변경 대상

`crates/moai-fs/src/lib.rs:205~237`.

현재 코드 (요약):
```rust
#[tokio::test]
#[ignore]
async fn test_unwatch_stops_events() {
    // ... watch + sleep 200ms ...
    tokio::time::sleep(Duration::from_millis(200)).await;     // [VARIABLE]

    let result = watcher.unwatch(&dir_path);
    assert!(result.is_ok(), "...");

    tokio::time::sleep(Duration::from_millis(100)).await;     // [VARIABLE]

    let new_file = dir_path.join("after_unwatch.txt");
    fs::write(&new_file, "해제 후 파일").expect("...");

    let received = timeout(Duration::from_millis(500), rx.recv()).await;  // [VARIABLE]
    assert!(received.is_err(), "감시 해제 후에도 이벤트가 수신되었습니다");
}
```

이 테스트는 "이벤트가 오지 않는 것" 을 확인하는 negative test 이므로 결정성 패턴이 T3 와 다소 다르다.

### 6.2 USER-DECISION-FW-A 옵션별 변환 가이드

**RESOLVED → A3 ACTIVE (2026-04-27).** 아래 (a) 패턴을 적용. (b)/(c)/(d) 는 escalation 시 참조용으로 보존.

#### (a) A3 적용 시 (권장 — ACTIVE)

핵심 변경:
- init `sleep(200ms)` → `sleep(50ms)` + 짧은 ready-event drain (선택).
- unwatch 후 settle delay `100ms` → `500ms` 또는 `1000ms` 로 확대 (CI 부하 마진).
- final `timeout(500ms)` → `timeout(2_000ms)` 로 확대 — "이 시간 안에 이벤트 안 오면 OK" 의 더 큰 마진.

스케치:
```rust
tokio::time::sleep(Duration::from_millis(50)).await;

// (선택) 초기화 시점에 발생할 수 있는 ready 이벤트 drain
while let Ok(Some(_)) = timeout(Duration::from_millis(50), rx.recv()).await { }

let result = watcher.unwatch(&dir_path);
assert!(result.is_ok(), "...");

// settle delay 확대
tokio::time::sleep(Duration::from_millis(500)).await;

let new_file = dir_path.join("after_unwatch.txt");
fs::write(&new_file, "해제 후 파일").expect("...");

// 더 큰 마진으로 "이벤트 없음" 확인
let received = timeout(Duration::from_millis(2_000), rx.recv()).await;
assert!(received.is_err(), "감시 해제 후에도 이벤트가 수신되었습니다");
```

#### (b) A2 적용 시

핵심 변경:
- settle 단계에서 retry loop. 각 retry 마다 짧게 receive 시도.
- 모든 retry 가 timeout 이면 PASS.

#### (c) A1 적용 시

핵심 변경:
- `200ms → 1000ms`, `100ms → 1000ms`, `500ms → 3000ms`.

#### (d) A4 적용 시

T3 와 동일하게 별 SPEC 분리.

### 6.3 검증

- 로컬 실행: `cargo test -p moai-fs --all-targets -- --ignored test_unwatch_stops_events`
- 100회 반복.
- macOS + Linux 양 환경.

### 6.4 의존

- T1 (USER-DECISION-FW-A) 결정 결과.

### 6.5 AC

- AC-FW-2, AC-FW-4.

---

## 7. T5 — 로컬 macOS + Linux 검증

### 7.1 검증 시나리오

1. 로컬 macOS:
   ```bash
   cargo test -p moai-fs --all-targets -- --ignored
   ```
   기대: 4 테스트 모두 PASS. (현재 unignored 2건 + flaky-fixed 2건.)

2. 로컬 Linux (또는 Docker container `rust:1.92` + `apt-get install build-essential`):
   ```bash
   cargo test -p moai-fs --all-targets -- --ignored
   ```
   기대: 동일.

3. (선택) 로컬 stress test:
   ```bash
   for i in {1..50}; do
     cargo test -p moai-fs --all-targets --release -- --ignored || { echo "FAIL at $i"; break; }
   done
   ```
   기대: 50회 중 0회 fail.

### 7.2 의존

- T3, T4 완료.

### 7.3 AC

- AC-FW-1, AC-FW-2 (4 환경 중 로컬 2 환경 부분 검증). 나머지 CI 2 환경은 T9 에서 검증.

---

## 8. T6 — `tmux-test` job 명령 narrow

### 8.1 변경 대상

`.github/workflows/ci-v3-pane.yml:312~313`:
```yaml
- name: Gate — cargo test (tmux-dependent, --ignored)
  run: cargo test --workspace --all-targets -- --ignored
```

### 8.2 USER-DECISION-FW-B 옵션별 변환 가이드

**RESOLVED → B1 ACTIVE (2026-04-27).** 아래 (a) 패턴을 적용. (b)/(c)/(d) 는 escalation 시 참조용으로 보존.

#### (a) B1 적용 시 (권장 — ACTIVE)

§4.1 enumeration 결과 tmux 테스트가 `tmux_*` 또는 `integration_tmux_*` prefix 를 따른다고 가정 시:

```yaml
- name: Gate — cargo test (tmux-dependent, --ignored, tmux only)
  run: cargo test --workspace --all-targets -- --ignored 'tmux'
```

`'tmux'` substring match 가 `integration_tmux_nested` 같은 함수 이름과 매칭. file watcher 테스트 (`test_detect_file_creation`) 는 매칭 안 됨.

만약 tmux 테스트 이름이 prefix 통일 안 되어 있으면, 여러 패턴을 list 로:
```yaml
  run: cargo test --workspace --all-targets -- --ignored 'tmux' 'integration_tmux'
```

cargo test 의 multiple positional patterns 는 OR semantics.

#### (b) B2 적용 시

```yaml
- name: Gate — cargo test (tmux-dependent, --ignored, tmux feature)
  run: cargo test --workspace --all-targets --features tmux-tests -- --ignored
```

T8 에서 Cargo.toml `[features]` 에 `tmux-tests` 추가 + tmux 테스트들에 `#[cfg(feature = "tmux-tests")]` 추가 필요.

#### (c) B3 적용 시

별도 binary 로 분리된 file watcher 테스트는 `--workspace` 의 default 실행에서 자동 분리됨. 이 옵션은 명령 변경 minimal:

```yaml
- name: Gate — cargo test (tmux-dependent, --ignored)
  # 변경 없음 — file watcher 테스트가 별도 binary 로 이동되었으므로 같은 명령으로도 자동 분리
  run: cargo test --workspace --all-targets -- --ignored
```

다만 file watcher 테스트가 별도 step 에서 `--test fs_watcher` 로 실행되어야 함 (T7).

#### (d) B4 적용 시

`tmux-test` job 은 (a) B1 와 동일하게 narrow. + 신설 `fs-watcher-test` job (T7 에서 정의).

### 8.3 의존

- T2 결정 결과.

### 8.4 AC

- AC-FW-5 (tmux-test 명령에서 file watcher 테스트 부재 확인).

---

## 9. T7 — file watcher 테스트 별도 step 또는 job 추가

### 9.1 변경 대상

`.github/workflows/ci-v3-pane.yml`.

### 9.2 USER-DECISION-FW-B 옵션별 변환 가이드

**RESOLVED → B1 ACTIVE (2026-04-27).** 아래 (a) 별도 step 패턴을 적용. (b)/(c)/(d) 는 escalation 시 참조용으로 보존.

#### (a) B1 적용 시 (권장 — ACTIVE — 별도 step)

`tmux-test` job 의 `Gate — cargo test (tmux-dependent ...)` step 직후에 신규 step 추가:

```yaml
- name: Gate — cargo test (moai-fs file watcher, --ignored)
  run: cargo test -p moai-fs --all-targets -- --ignored
```

이 step 은 같은 job 안의 다른 step 이지만, 이전 step (tmux 만 narrow) 이 PASS 한 이후에만 실행된다 (default step 순서). 이 step 의 fail 은 job fail 을 유발.

#### (b) B2 적용 시 (별도 step)

```yaml
- name: Gate — cargo test (moai-fs file watcher, --ignored, fs-watcher-tests feature)
  run: cargo test -p moai-fs --all-targets --features fs-watcher-tests -- --ignored
```

#### (c) B3 적용 시 (별도 step)

```yaml
- name: Gate — cargo test (moai-fs file watcher, integration binary)
  run: cargo test -p moai-fs --test fs_watcher -- --ignored
```

#### (d) B4 적용 시 (신설 별도 job)

```yaml
fs-watcher-test:
  name: fs-watcher-test (${{ matrix.platform.name }})
  runs-on: ${{ matrix.platform.runner }}
  strategy:
    fail-fast: false
    matrix:
      platform:
        - { name: macOS,  runner: macos-14 }
        - { name: Linux,  runner: ubuntu-22.04 }
  steps:
    - uses: actions/checkout@v4
    - name: Rust toolchain
      uses: dtolnay/rust-toolchain@stable
    - name: Install Linux system deps (GPUI runtime)
      if: matrix.platform.runner == 'ubuntu-22.04'
      run: |
        sudo apt-get update
        sudo apt-get install -y libxkbcommon-dev libxkbcommon-x11-dev \
          libxcb1-dev libxcb-shape0-dev libxcb-xfixes0-dev \
          libwayland-dev libfontconfig1-dev libssl-dev pkg-config
    - name: Cache cargo
      uses: Swatinem/rust-cache@v2
      with:
        key: v3-pane-fs-watcher-${{ matrix.platform.name }}
    - name: Gate — cargo test (moai-fs file watcher, --ignored)
      run: cargo test -p moai-fs --all-targets -- --ignored
```

이 job 은 `tmux-test` 와 독립. fail 시 tmux-test 의 success 영향 없음.

### 9.3 의존

- T2 결정 + T3, T4 완료.

### 9.4 AC

- AC-FW-6 (별도 step/job 존재 + 영향 격리 확인).

---

## 10. T8 — (옵션) Cargo.toml feature 또는 신규 tests/ 파일

### 10.1 USER-DECISION-FW-B 옵션별 적용

**RESOLVED → B1 ACTIVE (2026-04-27) → T8 SKIP.** 코드 변경 없음. (b)/(c)/(d) 는 escalation 시 참조용으로 보존.

#### (a) B1 적용 시 — T8 skip (ACTIVE)

코드 변경 없음. CI yaml 만 변경. T8 task 자체 omit.

#### (b) B2 적용 시 — Cargo.toml feature 추가

`crates/moai-fs/Cargo.toml`:
```toml
[features]
default = []
tmux-tests = []          # 본 SPEC 외 tmux 테스트가 사용
fs-watcher-tests = []    # 본 SPEC 의 file watcher 테스트가 사용
```

+ `crates/moai-fs/src/lib.rs:162` 의 `#[ignore]` 위에 `#[cfg(feature = "fs-watcher-tests")]` 추가 (두 테스트 각각).

#### (c) B3 적용 시 — 신규 tests/ 파일

신규 파일 `crates/moai-fs/tests/fs_watcher.rs`:
- `crates/moai-fs/src/lib.rs:162~237` 의 두 테스트 함수를 그대로 이동.
- `use moai_fs::{FsWatcher, FsEvent};` 등 외부 import.
- 단 두 테스트는 현재 `super::*` 사용 — `FsWatcher`, `FsEvent` 외 다른 internal item 접근 시 `pub(crate)` 또는 `pub` 노출 필요할 수 있음 (G5 위반 위험 — 발견 시 USER-DECISION-FW-B 재결정 권고).

#### (d) B4 적용 시 — T8 skip

코드 변경 없음 (B1 와 동일 코드 그대로).

### 10.2 의존

- T2 결정 결과 (B1 또는 B4 시 T8 skip).

### 10.3 AC

- REQ-FW-013 (옵션 보조 변경).

---

## 11. T9 — 100회 연속 CI 실행 stress test + flake rate 검증

### 11.1 검증 시나리오

옵션 1: GitHub Actions workflow_dispatch 의 manual trigger 100회 + 결과 통계.

옵션 2: 신규 일회성 stress workflow (`.github/workflows/fs-watcher-stress.yml`) 신설:
```yaml
name: FS Watcher Stress Test (100x)

on: workflow_dispatch

jobs:
  stress:
    strategy:
      matrix:
        platform:
          - { name: macOS,  runner: macos-14 }
          - { name: Linux,  runner: ubuntu-22.04 }
        run: [1, 2, 3, ..., 50]   # GitHub Actions matrix limit 256 entries
    runs-on: ${{ matrix.platform.runner }}
    steps:
      - uses: actions/checkout@v4
      - name: Rust toolchain
        uses: dtolnay/rust-toolchain@stable
      - name: Linux deps
        if: matrix.platform.runner == 'ubuntu-22.04'
        run: sudo apt-get install -y libfontconfig1-dev pkg-config
      - name: Run file watcher tests
        run: cargo test -p moai-fs --all-targets -- --ignored
```

100회를 50 × 2 platform 으로 분할. 결과 통계: `gh run list --workflow=fs-watcher-stress.yml --limit=200 --json conclusion`.

옵션 3: 로컬 stress (CI billing 부담 시): 50회 macOS + 50회 Linux Docker.

### 11.2 PASS 기준

- 100 (or 50 + 50) 실행 중 ≥ 99% PASS = AC-FW-3 충족.
- 99% 미만 시 R-FW-1 발동 — USER-DECISION-FW-A 재결정 또는 deadline 추가 확대.

### 11.3 stress workflow 정리

검증 완료 후 `.github/workflows/fs-watcher-stress.yml` 는 삭제 또는 `manual-only / disabled by default` 로 유지. 본 SPEC 은 검증 도구 측면이므로 stress workflow 는 산출물 아님 (선택 항목).

### 11.4 의존

- T6, T7 완료 (CI yaml 변경 적용 후).

### 11.5 AC

- AC-FW-3 (100회 중 99회 이상 PASS).

---

## 12. 외부 차단 / Prerequisites

본 SPEC implement 진입의 외부 차단:
- **없음**. USER-DECISION-FW-A + FW-B 결정만 필요.

연관 운영 작업 (본 SPEC 외, 본 SPEC 완료 후):
- CLAUDE.local.md §2.1 의 branch protection required contexts 에 `tmux-test (macOS)` + `tmux-test (Linux)` 추가 (gh CLI 또는 GitHub UI). 본 SPEC 의 AC-FW-3 충족이 그 운영 변경의 prerequisite.

---

## 13. 검증 / Validation Strategy

### 13.1 단위 검증 (T3 + T4)

- 로컬 macOS + Linux 단일 실행 PASS (AC-FW-1, AC-FW-2).
- worst-case 10초 이내 (AC-FW-4) — `time cargo test ...` wall-clock 측정.

### 13.2 통합 검증 (T6 + T7)

- CI yaml 변경 후 PR 생성 → CI run 결과 확인:
  - `tmux-test` job 의 step log 에 file watcher 테스트 부재 (AC-FW-5).
  - 별도 step/job 존재 + 의도적 fail 주입 시 영향 격리 확인 (AC-FW-6).

### 13.3 안정성 검증 (T9)

- 100회 연속 stress run (또는 50 + 50). PASS rate ≥ 99% (AC-FW-3).

### 13.4 회귀 검증

- moai-fs 의 기존 unignored 테스트 (`test_watcher_creation`, `test_watch_directory`) PASS 유지.
- moai-fs 공개 API 사용처 (예: `moai-studio-workspace`, `moai-studio-ui`) 의 build PASS 유지 — `cargo build --workspace` 검증.

---

## 14. 진행 추적

implement 진행 시 본 plan.md 의 §15 Progress 섹션을 신설하여 task 별 진행 상황을 누적 기록.

기록 포맷:
```
## 15. Progress

### MS-1 (시작: 2026-MM-DD)

- [ ] T1: USER-DECISION-FW-A — option (X) 결정 (date)
- [ ] T2: USER-DECISION-FW-B — option (X) 결정 (date) + tmux 테스트 enumeration (file: <path>)

### MS-2 (시작: 2026-MM-DD)

- [ ] T3: test_detect_file_creation 변경 (commit: <sha>)
- [ ] T4: test_unwatch_stops_events 변경 (commit: <sha>)
- [ ] T5: 로컬 검증 (macOS PASS, Linux PASS)

### MS-3 (시작: 2026-MM-DD)

- [ ] T6: tmux-test job 명령 narrow (commit: <sha>)
- [ ] T7: 별도 step/job 추가 (commit: <sha>)
- [ ] T8: (옵션) Cargo.toml feature 또는 tests/ 파일 (commit: <sha> 또는 SKIP)
- [ ] T9: 100회 stress test (PASS rate: XX%, run id: <gh-run-id>)
```

---

## 15. 완료 정의 (Definition of Done)

본 SPEC 의 implement 완료는 다음 모두 충족 시:

USER-DECISION 게이트 (v1.1.0 갱신):

- [x] USER-DECISION-FW-A RESOLVED (a) A3 polling with bounded retry (2026-04-27)
- [x] USER-DECISION-FW-B RESOLVED (a) B1 cargo test name filter (2026-04-27)
- USER-DECISION-FW-C OMITTED — research §4 권고, notify v7 유지 default

Acceptance / 운영 체크:

- [ ] AC-FW-1, AC-FW-2, AC-FW-3, AC-FW-4, AC-FW-5, AC-FW-6 모두 통과.
- [ ] moai-fs 공개 API 무변경 검증 (`cargo build --workspace` PASS + 의존 crate 영향 없음).
- [ ] CI yaml 변경 범위가 `tmux-test` job + 신규 step (file watcher 별도 step) 에 국한 (다른 job 무영향).
- [ ] (옵션 운영 작업) CLAUDE.local.md §2.1 의 required contexts 에 `tmux-test (macOS)` + `tmux-test (Linux)` 추가 결정 (사용자 승인).
- [ ] feature 브랜치 → develop squash merge (CLAUDE.local.md §4.1 컨벤션) — 본 plan.md §16 갱신 후 PR.

---

작성 종료. 본 plan.md 는 spec.md (RG/REQ/AC) + research.md (배경 분석) 와 함께 SPEC-V3-FS-WATCHER-001 implement 진입의 입력이다. T1 + T2 의 USER-DECISION 결정이 완료되어야 T3 이후 진입 가능.
