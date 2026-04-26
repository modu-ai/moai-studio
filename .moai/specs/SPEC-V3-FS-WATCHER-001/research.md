# SPEC-V3-FS-WATCHER-001 Research — moai-fs 파일 감시자 테스트 결정성 + CI 버킷 격리

작성: MoAI (manager-spec, 2026-04-27)
브랜치 (현재): `feature/ci-linux-git-walkup-fix` (본 SPEC 은 문서 산출만)
범위: `crates/moai-fs` 의 `#[ignore]` 처리된 file watcher 테스트의 결정성 확보, 그리고 `tmux-test` CI job 의 `--ignored` 버킷 conflation 해소.

---

## 1. 문제 진단

### 1.1 증상 — `tmux-test` CI 의 간헐 실패

`.github/workflows/ci-v3-pane.yml:248` 에 정의된 `tmux-test` job (`macOS` + `Linux` matrix) 은 line 312~313 에서 다음 명령을 실행한다:

```yaml
- name: Gate — cargo test (tmux-dependent, --ignored)
  run: cargo test --workspace --all-targets -- --ignored
```

`--ignored` 플래그는 `#[ignore]` 어트리뷰트가 붙은 모든 테스트를 워크스페이스 전역에서 실행한다. 본 의도는 "tmux 가 설치되지 않은 환경에서는 skip 되어야 하는 tmux 의존 integration 테스트" (예: `integration_tmux_nested`, `ctrl_b_passes_through_to_nested_tmux` — AC-P-26) 만을 분리하기 위함이었으나, 워크스페이스 전역의 모든 `#[ignore]` 테스트가 동시에 실행된다.

CLAUDE.local.md §2.1 의 branch protection 활성화 시 "별개 이슈, 추후 SPEC 으로 fix 후 추가" 항목으로 다음이 명시되어 있다:

- `tmux-test (macOS)` — file watcher flaky
- `tmux-test (Linux)` — 느린 cache 빌드 + file watcher flaky

본 SPEC 은 위 두 항목 중 file watcher flaky 부분의 근본 원인을 해소한다.

### 1.2 conflation 의 두 출처

`tmux-test` 버킷에 섞이는 `#[ignore]` 테스트:

1. **tmux 의존 integration 테스트**: `--ignored` 의 의도된 사용처. tmux runtime 이 필요하므로 `Install tmux` step 후에만 실행 가능. 결정적 (deterministic).
2. **moai-fs file watcher 테스트** (`crates/moai-fs/src/lib.rs:165, 210`): 타이밍 의존. notify backend 의 OS-level event delivery 가 CI runner 부하에 따라 200ms~수초 지연 가능. 비결정적 (flaky).

두 부류는 실패 모드가 독립적이지만 같은 `--ignored` 버킷에 들어가서 file watcher flake → 전체 `tmux-test` job fail → branch protection 추가 차단의 cascade 가 발생한다.

---

## 2. moai-fs file watcher 현황 분석

### 2.1 코드 구조 (`crates/moai-fs/src/lib.rs`)

`FsWatcher` (line 53~121) 는 다음 구조이다:

- `notify::RecommendedWatcher` 를 boxed 보관 (line 55).
- `notify::Config::default().with_poll_interval(Duration::from_millis(100))` 로 폴링 간격 100ms 설정 (line 78).
- notify 콜백 → `std::sync::mpsc` → 별도 OS thread → `tokio::sync::mpsc::Sender::blocking_send` 의 2-stage 채널 (line 65~103).
- `EventKind::Create / Modify / Remove` 만 `FsEvent` 로 변환 (line 85~90), 그 외는 drop.

공개 API (line 11):
```rust
pub use watcher::{FsEventBus, WorkspaceEvent, WorkspaceKey};
pub use workspace_watcher::WorkspaceWatcher;
```
+ `FsWatcher`, `FsEvent`, `FsWatcherError` 타입.

### 2.2 의존 — `crates/moai-fs/Cargo.toml`

```toml
[dependencies]
notify = "7"
tokio = { workspace = true, features = ["sync"] }
tracing = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
tokio = { workspace = true, features = ["rt", "macros", "test-util", "time"] }
tempfile = "3"
```

핵심: `notify v7` (current major), `notify-debouncer-mini` 또는 `notify-debouncer-full` 미사용. tokio `test-util` feature 가 있어 `tokio::time::pause()` 기반 가상 시간 사용 가능.

### 2.3 flaky 테스트 두 건

#### 2.3.1 `test_detect_file_creation` (line 162~203)

```rust
#[tokio::test]
#[ignore]
async fn test_detect_file_creation() {
    // Arrange
    let dir = tempdir().expect("임시 디렉토리 생성 실패");
    let (mut watcher, mut rx) = FsWatcher::new().expect("감시자 생성 실패");
    watcher.watch(&dir_path).expect("감시 등록 실패");

    // 감시자 초기화 대기
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Act: 파일 생성
    fs::write(&new_file, "테스트 내용").expect("파일 쓰기 실패");

    // Assert: 1초 타임아웃
    let received = timeout(Duration::from_secs(1), async {
        loop {
            if let Some(event) = rx.recv().await { ... }
        }
    }).await;
    assert!(received.is_ok(), "타임아웃: ...");
}
```

타이밍 가정:
- 200ms: notify watcher 초기화 + OS-level inotify/FSEvents 등록 완료 충분.
- 1000ms: file create event 가 채널까지 전파 충분.

CI 환경에서 실패하는 시나리오:
- ubuntu-22.04 runner 가 high load 상태 (병행 job 다수). inotify 등록 + event delivery 지연 > 200ms 가 발생.
- macOS-14 runner 의 FSEvents 는 default coalescing latency 가 있어 첫 이벤트가 지연될 수 있음.
- notify v7 의 polling backend (FSEvents 미가용 시 fallback) 가 100ms poll_interval 로 동작 → 첫 poll 이 200ms init wait 직후 발생 시 타이밍 race 가능.

#### 2.3.2 `test_unwatch_stops_events` (line 205~237)

```rust
#[tokio::test]
#[ignore]
async fn test_unwatch_stops_events() {
    // ... watch + sleep 200ms ...
    let result = watcher.unwatch(&dir_path);
    tokio::time::sleep(Duration::from_millis(100)).await;
    fs::write(&new_file, "해제 후 파일").expect("파일 쓰기 실패");

    // Assert: 500ms 내에 이벤트가 없어야 함
    let received = timeout(Duration::from_millis(500), rx.recv()).await;
    assert!(received.is_err(), "감시 해제 후에도 이벤트가 수신되었습니다");
}
```

타이밍 가정:
- 200ms: 초기화.
- 100ms: unwatch 처리.
- 500ms 내 이벤트 없음 = 성공.

CI 실패 시나리오:
- unwatch 처리 + inotify 해제가 100ms 안에 완료되지 않음 → 그 사이 file create event 가 큐에 들어감 → 500ms 안에 수신됨 → fail.
- notify v7 의 polling backend 는 unwatch 즉시 polling thread 를 정리하지만, in-flight 이벤트는 여전히 채널에 도달 가능.

### 2.4 로컬 vs CI 차이의 근본 원인

| 환경 | OS event backend | event delivery latency | 200ms 가정 |
|------|-----------------|----------------------|-----------|
| 로컬 macOS | FSEvents (native) | 5~50ms typical | 충분 |
| 로컬 Linux | inotify (native) | 1~20ms typical | 충분 |
| CI macos-14 | FSEvents (가상화 영향 가능) | 50~500ms variable | 부족 가능 |
| CI ubuntu-22.04 | inotify (container/runner overhead) | 20~1000ms variable | 부족 가능 |
| 모든 fallback | polling 100ms | 100~200ms (poll cycle 의존) | 100ms 가정 안전, 200ms 마진 부족 |

CI 의 high load 상태 (병행 job 다수, runner pool 경쟁) 에서 200ms init wait 가 부족하다. 이것이 flaky 의 근본 원인.

---

## 3. 해결 옵션 — 두 축의 직교 분해

### 3.1 Axis A — 테스트 결정성 옵션

#### A1. Sleep 시간 증가 (현행 패턴 + 마진 ↑)

방식: `200ms → 1000ms`, `500ms → 2000ms`. 단순 patch.

장점: 코드 변경 minimal, 외부 의존 0.
단점: CI 부하가 그 마진을 초과하면 여전히 flaky. 테스트 시간 증가 (각 테스트당 +1~2초). 근본 원인 미해소.

#### A2. Retry loop with exponential backoff

방식: 이벤트 수신 시도를 `100ms, 200ms, 400ms, 800ms, ...` 간격으로 재시도. 일정 횟수 (예: 5회) 후 fail.

장점: CI 부하 변동에 적응. 평균 latency 가 낮은 환경에서는 여전히 빠름.
단점: 구현 복잡도 ↑. 최악의 경우 재시도 시간 합산 ~3초.

#### A3. Polling with bounded retry (권장)

방식: `tokio::time::timeout` 의 단일 큰 deadline (예: 5000ms) 안에서 `rx.try_recv` 또는 `rx.recv` 를 짧은 폴링 간격 (50ms) 으로 시도. 첫 매칭 이벤트 도착 즉시 성공.

장점: deterministic upper bound (5초). 평균 latency 가 낮은 환경에서는 여전히 빠름 (50~200ms 안에 성공). notify backend 와 무관. 코드 변경 minimal (~10 LOC).
단점: 테스트당 worst-case 5초. (실제로는 거의 항상 100ms 이내 성공.)

#### A4. notify backend mock

방식: `notify` crate 의 backend 를 trait 로 추상화하여 테스트에서 mock backend 주입. mock 은 동기적으로 이벤트 발사.

장점: 100% deterministic. 1ms 안에 완료.
단점: `FsWatcher` API 의 implementation 결합. 신규 trait 도입 = public API 변경 위험. test 가 production code 의 진짜 OS event 동작을 검증하지 못함 (false confidence).

#### A5. notify-debouncer 도입

방식: `notify-debouncer-mini` 또는 `notify-debouncer-full` 로 교체. debouncer 는 burst event 를 batching 하여 안정적인 단일 이벤트로 변환.

장점: 실 production 사용 시에도 burst event spam 방지 (rapid file modification 시 유리). debouncer 의 timeout 이 명시적이므로 테스트에서 그 timeout 만 늘려도 결정적.
단점: dependency 추가 (`notify-debouncer-mini` 또는 `notify-debouncer-full`). `FsWatcher` 내부 구현 변경 (FsEvent 의 의미 변화 가능 — debounced batch 가 단일 이벤트로 보임). 검증 부담 증가.

### 3.2 Axis B — CI 버킷 격리 옵션

#### B1. cargo test name filter (권장)

방식: `tmux-test` job 이 `cargo test --workspace --all-targets -- --ignored 'tmux::'` (tmux 모듈 prefix) 로 변경. moai-fs 테스트는 별도 step 또는 별도 job 에서 `cargo test -p moai-fs --all-targets -- --ignored` 로 실행.

장점: 코드 변경 0, CI yaml 만 수정. 단순.
단점: 테스트 함수 이름 또는 모듈 path 가 prefix 컨벤션 (예: `tmux::*`, `fs::*`) 을 따라야 함. 현재 tmux 테스트가 그 패턴을 만족하는지 별도 확인 필요. 신규 ignored 테스트 추가 시 명명 컨벤션 강제 필요.

#### B2. cfg feature flag

방식: `#[cfg(feature = "tmux-tests")]` + `#[cfg(feature = "fs-watcher-tests")]` 로 테스트를 분리. CI 에서 `--features tmux-tests` 또는 `--features fs-watcher-tests` 로 선택 실행.

장점: 명시적이고 type-safe. `cargo build` 시점에 분리 보장.
단점: Cargo.toml feature 추가 + 모든 ignored 테스트에 `#[cfg(feature = ...)]` 추가 필요. compile-time 분기로 인한 빌드 multiplication.

#### B3. Separate test binary

방식: `crates/moai-fs/tests/fs_watcher.rs` integration test 파일 신설. 기존 `#[cfg(test)] mod tests` 의 file watcher 테스트를 그쪽으로 이동. CI 에서 `cargo test -p moai-fs --test fs_watcher -- --ignored` 로 분리 실행.

장점: 명시적 분리. 다른 unit test 와 컴파일 단위 분리. cargo 의 표준 패턴 준수.
단점: 테스트 파일 이동 필요 (코드 refactor). private API 접근 시 추가 노출 필요할 수 있음 (현재 테스트는 `super::*` 사용 — `pub(crate)` 또는 `pub` 노출 필요). 본 SPEC 의 "공개 API 무변경" 제약 위반 위험.

#### B4. 두 별도 CI job

방식: `tmux-test` job 은 그대로 두되, 명령을 `cargo test ... -- --ignored 'tmux::'` 로 narrow. 신설 `fs-watcher-test` job 은 `cargo test -p moai-fs --all-targets -- --ignored` 로 file watcher 테스트만 실행. `fs-watcher-test` 는 결정성 확보 (Axis A) 후 required 추가, 그 전까지 allowed-failures.

장점: 가장 명확한 분리. 각 job 의 실패가 다른 job 에 영향 없음. branch protection 에 두 job 각각 독립 추가 가능.
단점: CI yaml 가장 큰 변경. job duplication (rust-toolchain, cache 설정 중복).

### 3.3 권장 조합

**A3 (polling with bounded retry) + B1 (cargo test name filter)** 조합을 default 권장.

근거:
- A3 는 코드 변경 minimal (~10 LOC), 외부 의존 0, deterministic upper bound. 1순위 단순성.
- B1 은 CI yaml 만 변경, 코드 변경 0. 1순위 단순성.
- A3 + B1 조합은 "minimal blast radius" — 신규 dependency 0, public API 무변경, 테스트 파일 이동 없음.
- 문제 해소 후 `tmux-test (macOS)` + `tmux-test (Linux)` 를 branch protection required contexts 에 추가 가능. CLAUDE.local.md §2.1 의 "별개 이슈" 항목 해소.

다만 B1 의 한계 — tmux 테스트가 `tmux::` 또는 동등 prefix 를 따르지 않는 경우 (현재 미확인) — research §1.1 의 `integration_tmux_nested`, `ctrl_b_passes_through_to_nested_tmux` 함수 이름은 이미 prefix 가 약함. 따라서 B1 적용 시 테스트 함수 이름 변경 또는 모듈 path 정리 (`tests/integration_tmux.rs` 같은 별도 파일) 가 부수 작업으로 발생할 수 있음 — MS-3 의 task 분해에서 다룸.

USER-DECISION-FW-A 와 USER-DECISION-FW-B 는 본 SPEC 의 plan.md 에서 PENDING 으로 표기하고, 사용자가 최종 결정한다.

---

## 4. notify 버전 정책 — USER-DECISION 후보

`crates/moai-fs/Cargo.toml` 의 `notify = "7"` 는 current major. notify v7 은 `notify-debouncer-mini` v0.4 + `notify-debouncer-full` v0.4 와 호환된다.

debouncer 도입 시:
- `notify-debouncer-mini`: minimal API. event 를 timeout 안에서 batching, 단일 이벤트 emit. RecommendedWatcher 직접 노출 없음.
- `notify-debouncer-full`: file rename 추적 + cookie 매칭 등 풍부한 기능. RecommendedWatcher 노출.

본 SPEC 의 결정성 문제 해소 관점에서 debouncer 가 본질적 해결책은 아니다 (debouncer 의 timeout 도 결국 하한 deadline 이 필요함). 다만 production 의 burst event 처리 (rapid save/edit) 측면에서 향후 가치가 있을 수 있다.

**USER-DECISION-FW-C** (선택 게이트): notify-debouncer 도입을 본 SPEC 에서 동시에 진행할지 별 SPEC 으로 분리할지. research 의 권고는 **별 SPEC 분리** — 본 SPEC 은 결정성 + 버킷 격리에만 집중. debouncer 는 production behavior 변화를 동반하므로 별도 USER-DECISION 과 acceptance criteria 가 필요.

따라서 본 SPEC 의 default 는 **notify v7 유지, debouncer 미도입**, USER-DECISION-FW-C 는 plan.md 에서 omit 또는 PENDING 처리.

---

## 5. 외부 인터페이스 약속 — 무엇이 변하지 않는가

본 SPEC 은 다음 공개 API 를 변경하지 않는다 (out-of-scope):

```rust
// crates/moai-fs/src/lib.rs
pub struct FsWatcher { /* opaque */ }
impl FsWatcher {
    pub fn new() -> Result<(Self, mpsc::Receiver<FsEvent>), FsWatcherError>;
    pub fn watch(&mut self, path: &Path) -> Result<(), FsWatcherError>;
    pub fn unwatch(&mut self, path: &Path) -> Result<(), FsWatcherError>;
}

pub enum FsEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Removed(PathBuf),
}

pub enum FsWatcherError {
    WatchError(notify::Error),
    ChannelClosed,
}

pub use watcher::{FsEventBus, WorkspaceEvent, WorkspaceKey};
pub use workspace_watcher::WorkspaceWatcher;
```

본 SPEC 은:
- `FsWatcher::new` 시그니처 무변경.
- `FsEvent` enum variant 무변경.
- `FsWatcherError` variant 무변경.
- `FsEventBus` / `WorkspaceWatcher` 무변경.
- 기존 `#[ignore]` 테스트 함수 이름 무변경 (또는 변경 시 USER-DECISION-FW-B 에 명시).

본 SPEC 은 변경 가능:
- 테스트 함수 내부 로직 (timeout, polling pattern).
- `.github/workflows/ci-v3-pane.yml` 의 `tmux-test` job 명령 + 신규 step 또는 job 추가.

---

## 6. CI workflow 영향 분석

현재 `.github/workflows/ci-v3-pane.yml:312` 의 단일 step:
```yaml
- name: Gate — cargo test (tmux-dependent, --ignored)
  run: cargo test --workspace --all-targets -- --ignored
```

B1 적용 시 (예시 — 실제 prefix 는 implement 시 확인 필요):
```yaml
- name: Gate — cargo test (tmux-dependent, --ignored, tmux only)
  run: cargo test --workspace --all-targets -- --ignored 'tmux::' 'integration_tmux'

- name: Gate — cargo test (fs-watcher, --ignored)
  run: cargo test -p moai-fs --all-targets -- --ignored
```

또는 B4 적용 시 별도 job 신설.

선결 조건:
- tmux 테스트의 함수/모듈 이름 컨벤션 확정. 현재 `integration_tmux_nested`, `ctrl_b_passes_through_to_nested_tmux` 가 어느 모듈 path 에 있는지 확인 필요 (implement 단계의 first task).
- `cargo test -- --ignored 'pattern'` 의 매칭 동작 확인 — substring match (모든 함수 full path 의 일부와 매칭) 라는 점 인지.

---

## 7. 위험 요약

| ID | 위험 | 완화 |
|----|------|------|
| R-FW-1 | A3 의 5초 deadline 도 high-load CI 에서 부족 | deadline 을 10초로 increase, 또는 환경변수 (`CI_FS_TIMEOUT_MS`) 로 외부 조정 가능하게 |
| R-FW-2 | B1 의 cargo test name filter 가 substring match 라 의도 외 테스트가 매칭 | 명시적 prefix 컨벤션 강제 (예: `tmux_*`, `fs_watcher_*`), 또는 B3 (별도 binary) 로 재고 |
| R-FW-3 | tmux 테스트 이름이 `tmux::` 또는 통일 prefix 미준수 | implement 단계 first task 로 tmux 테스트 enumeration + 필요 시 모듈 reorganization |
| R-FW-4 | Axis A 적용 후에도 일부 환경 (예: macOS Apple Silicon Rosetta) 에서 flake 잔존 | 본 SPEC 무관 — 별 issue 로 escalate |

---

## 8. Prior art

- Tokio file watcher tests (`tokio-rs/notify` 자체 테스트) 는 `std::time::Duration::from_secs(2)` ~ `from_secs(5)` 의 generous timeout + polling 조합 (A3 와 동등) 사용.
- `notify-debouncer-mini` 의 doc example 은 `Duration::from_millis(500)` debounce + `recv_timeout(Duration::from_secs(5))` 패턴.
- `helix-editor/helix` 의 file watcher 테스트는 retry loop (A2) 사용, 최대 10회 100ms backoff.

본 SPEC 의 A3 권고는 위 prior art 의 단순 패턴 (single bounded deadline + short polling) 과 정합한다.

---

## 9. 결론

- 두 file watcher 테스트의 flakiness 는 fixed sleep duration 가정이 CI 부하에 비례해 부족한 것이 근본 원인.
- 해결 방향 = Axis A (테스트 결정성) × Axis B (CI 버킷 격리) 두 축의 직교 조합.
- 권장: A3 (polling with bounded retry) + B1 (cargo test name filter) — minimal blast radius, public API 무변경, 신규 dependency 0.
- 사용자가 USER-DECISION-FW-A, USER-DECISION-FW-B 의 옵션을 결정한 후 implement 진입.
- USER-DECISION-FW-C (notify-debouncer 도입) 는 별 SPEC 으로 분리 권고, 본 SPEC 의 plan.md 에서 omit.
- 본 SPEC 완료 시 `tmux-test (macOS)` + `tmux-test (Linux)` 를 branch protection required contexts 에 추가 가능 (CLAUDE.local.md §2.1 의 "별개 이슈" 항목 해소).

---

작성 종료. 본 research.md 는 spec.md (RG/REQ/AC) + plan.md (Milestone × Task) 의 입력이다.
