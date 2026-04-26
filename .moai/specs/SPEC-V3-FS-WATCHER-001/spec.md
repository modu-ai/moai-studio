---
id: SPEC-V3-FS-WATCHER-001
version: 1.1.0
status: ready
created_at: 2026-04-27
updated_at: 2026-04-27
author: MoAI (manager-spec)
priority: Medium
issue_number: 0
depends_on: []
parallel_with: []
milestones: [MS-1, MS-2, MS-3]
language: ko
labels: [type/test, type/ci, area/ci, area/fs, priority/p2-medium]
revision: v1.0.0 (initial draft, file watcher 테스트 결정성 + CI 버킷 격리)
---

# SPEC-V3-FS-WATCHER-001: moai-fs 파일 감시자 테스트 결정성 + tmux-test CI 버킷 격리

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-04-27 | 초안 작성. moai-fs `#[ignore]` file watcher 테스트의 결정성 확보 + `tmux-test` CI job 의 `--ignored` 버킷 conflation 해소. RG-FW-1 (테스트 결정성), RG-FW-2 (CI 버킷 격리). REQ 7 건, AC 6 건, MS-1/MS-2/MS-3, USER-DECISION-FW-A / FW-B 두 게이트. 공개 API 무변경, 신규 dependency 0 권장. CLAUDE.local.md §2.1 의 "별개 이슈" 항목 해소가 본 SPEC 완료의 가치. |
| 1.1.0 | 2026-04-27 | USER-DECISION-FW-A RESOLVED → A3 (polling with bounded retry, 5초 deadline, 50~100ms 간격, ~10 LOC, dep 0). USER-DECISION-FW-B RESOLVED → B1 (cargo test name filter `tmux` substring + file watcher 별도 step, 코드 변경 0). USER-DECISION-FW-C OMITTED (research §4 권고에 따라 notify v7 유지 default). status draft→ready. v0.1.x 패치 sprint 후보. |

---

## 1. 개요

### 1.1 목적

`crates/moai-fs/src/lib.rs:165` (`test_detect_file_creation`) 와 `crates/moai-fs/src/lib.rs:210` (`test_unwatch_stops_events`) 의 `#[ignore]` file watcher 테스트를 결정적으로 만들고, `.github/workflows/ci-v3-pane.yml:248` 의 `tmux-test` job 의 `--ignored` 버킷에서 file watcher 테스트를 격리하여, `tmux-test (macOS)` 와 `tmux-test (Linux)` 가 branch protection required contexts 에 추가 가능한 안정성을 확보한다.

본 SPEC 의 산출물은:

- moai-fs file watcher 테스트가 CI 환경에서 100회 연속 실행 시 99회 이상 통과 (decisively reliable).
- `tmux-test` CI job 이 tmux 의존 테스트만 실행 (file watcher 테스트는 분리된 step 또는 job 에서 실행).
- CLAUDE.local.md §2.1 의 "별개 이슈, 추후 SPEC 으로 fix 후 추가" 항목 (`tmux-test (macOS)`, `tmux-test (Linux)`) 해소.
- moai-fs 공개 API (`FsWatcher`, `FsEvent`, `FsWatcherError`, `FsEventBus`, `WorkspaceWatcher`) 무변경.

### 1.2 근거 문서

- `.moai/specs/SPEC-V3-FS-WATCHER-001/research.md` — 코드베이스 분석, 옵션 평가 (Axis A × Axis B), 권장 조합 (A3 + B1), 위험.
- `crates/moai-fs/src/lib.rs` (line 1~238) — 현행 `FsWatcher` 구현 + 두 flaky 테스트.
- `crates/moai-fs/Cargo.toml` (line 7~15) — notify v7 dependency.
- `.github/workflows/ci-v3-pane.yml` (line 240~313) — `tmux-test` job 의 `--ignored` 버킷 정의.
- `CLAUDE.local.md` §2.1 — branch protection required contexts 정의 + "별개 이슈" 항목.

---

## 2. 배경 및 동기

### 2.1 증상

`tmux-test (macOS)` 와 `tmux-test (Linux)` CI job 이 간헐 실패한다. 실패 원인은 tmux 의존성 자체가 아니라 같은 `--ignored` 버킷에 들어간 moai-fs file watcher 테스트의 timing race.

### 2.2 근본 원인

`cargo test --workspace --all-targets -- --ignored` (`.github/workflows/ci-v3-pane.yml:313`) 는 워크스페이스 전역의 모든 `#[ignore]` 테스트를 동시에 실행한다. 이 버킷에는 두 부류가 섞여 있다:

1. **결정적**: tmux 의존 integration 테스트 (`integration_tmux_nested`, `ctrl_b_passes_through_to_nested_tmux` 등) — 의도된 `--ignored` 사용처.
2. **비결정적**: moai-fs file watcher 테스트 (`crates/moai-fs/src/lib.rs:165, 210`) — `tokio::time::sleep(200ms)` + `timeout(1s)` / `timeout(500ms)` 의 fixed deadline 이 CI 부하에 비례해 부족.

두 부류 중 비결정적 부류의 flake 가 전체 job fail 을 유발한다.

### 2.3 영향

- branch protection 에 `tmux-test (macOS)` + `tmux-test (Linux)` 추가 차단 → main / develop 의 tmux 회귀 검증 자동화 불가.
- CLAUDE.local.md §2.1 의 "별개 이슈" 항목 미해소 → release infrastructure (SPEC-V3-011 / RELEASE-V0.1.0) 진입 시 외부 차단.

---

## 3. 목표 및 비목표 (Goals / Non-Goals)

### 3.1 목표 (Goals)

- G1. `test_detect_file_creation` 이 CI 환경에서 deterministic upper bound (예: 5초 deadline) 안에 첫 매칭 이벤트 도착 시 즉시 성공한다.
- G2. `test_unwatch_stops_events` 가 unwatch 후 짧은 settle delay 안에서 이벤트 미수신을 확인하되, settle delay 가 CI 부하에 robust 하다.
- G3. 두 테스트가 100회 연속 CI 실행 시 99회 이상 통과한다 (목표 flake rate < 1%).
- G4. `tmux-test` CI job 의 `--ignored` 버킷이 tmux 의존 테스트만 포함한다 (file watcher 테스트는 분리된 step 또는 job 으로 이동).
- G5. moai-fs 공개 API 가 변경되지 않는다 (`FsWatcher`, `FsEvent`, `FsWatcherError`, `FsEventBus`, `WorkspaceWatcher`).
- G6. moai-fs 의 `Cargo.toml` `[dependencies]` 가 추가되지 않는다 (USER-DECISION-FW-C 가 (b) 또는 (c) 결정 시 예외).
- G7. `tmux-test (macOS)` 와 `tmux-test (Linux)` 를 branch protection required contexts 에 추가 가능한 안정성 확보.
- G8. CI yaml 변경은 `.github/workflows/ci-v3-pane.yml` 의 `tmux-test` job 영역 (line 240~313) 에 국한된다 (다른 job 영향 없음).

### 3.2 비목표 (Non-Goals)

- N1. `FsWatcher` 또는 `FsEvent` 의 공개 시그니처 변경.
- N2. moai-fs 의 다른 모듈 (`tree_watcher`, `watcher`, `workspace_watcher`) 변경.
- N3. notify v7 → notify v8 또는 다른 major 업그레이드 (USER-DECISION-FW-C 의 (a) default 결정 시).
- N4. `FsEventBus` 또는 `WorkspaceWatcher` 의 내부 동작 변경.
- N5. 신규 외부 서비스 도입 (예: 별도 file watch backend, IPC).
- N6. tmux 의존 테스트 자체의 결정성 개선 (별 이슈, 본 SPEC 무관).
- N7. CI runner 종류 변경 (예: macos-14 → macos-15) 또는 CI billing 관련 작업.
- N8. Linux 의 inotify watch limit (`/proc/sys/fs/inotify/max_user_watches`) 조정.
- N9. moai-fs 의 production behavior 변경 (debouncer 도입 시 별 SPEC).
- N10. existing `#[ignore]` 테스트 삭제 또는 무력화 (`#[ignore]` 유지 또는 unignore 모두 가능, 단 결정성 확보 후만).

---

## 4. 사용자 스토리

- **US-FW-1**: CI 운영자가 `tmux-test (macOS)` 또는 `tmux-test (Linux)` job 실패 알림을 받았을 때, 실패 원인이 tmux 환경 문제로 좁혀진다 (file watcher flake 배제).
- **US-FW-2**: 개발자가 `cargo test -p moai-fs --all-targets -- --ignored` 를 로컬에서 실행할 때 두 테스트가 결정적으로 통과한다.
- **US-FW-3**: 개발자가 `cargo test --workspace --all-targets -- --ignored 'tmux'` (또는 동등 filter) 를 CI 에서 실행할 때 file watcher 테스트는 실행되지 않고 tmux 테스트만 실행된다.
- **US-FW-4**: Branch protection 운영자가 `tmux-test (macOS)` 와 `tmux-test (Linux)` 를 required contexts 에 추가했을 때, 100회 연속 실행 중 99회 이상 통과하여 PR 머지 차단이 발생하지 않는다.
- **US-FW-5**: 신규 contributor 가 moai-fs 의 file watcher 테스트를 읽을 때, deterministic upper bound + polling 패턴이 명확하여 의도가 self-documenting 하다.

---

## 5. 기능 요구사항 (EARS)

### RG-FW-1 — 테스트 결정성

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-FW-001 | Ubiquitous | 시스템은 `test_detect_file_creation` (`crates/moai-fs/src/lib.rs:165`) 에서 fixed sleep + fixed timeout 의 조합을 USER-DECISION-FW-A 로 결정된 결정적 패턴 (예: bounded deadline + short polling, retry with backoff, 또는 sleep margin 증가) 으로 교체한다. | The system **shall** replace the fixed-sleep + fixed-timeout pattern in `test_detect_file_creation` with the deterministic pattern chosen via USER-DECISION-FW-A. |
| REQ-FW-002 | Ubiquitous | 시스템은 `test_unwatch_stops_events` (`crates/moai-fs/src/lib.rs:210`) 에서 unwatch settle 단계의 fixed sleep 을 USER-DECISION-FW-A 로 결정된 패턴에 정합한 settle 검증 로직으로 교체한다. | The system **shall** replace the fixed-sleep settle stage in `test_unwatch_stops_events` with the settle verification logic consistent with the USER-DECISION-FW-A pattern. |
| REQ-FW-003 | Event-Driven | CI 환경에서 file watcher 테스트가 100회 연속 실행될 때, 시스템은 99회 이상 PASS 를 보장한다 (목표 flake rate < 1%). | When file watcher tests run 100 consecutive times in CI, the system **shall** PASS at least 99 times (target flake rate < 1%). |
| REQ-FW-004 | Unwanted | 시스템은 file watcher 테스트의 worst-case 실행 시간이 단일 테스트당 10초를 초과하지 않는다. | The system **shall not** allow worst-case execution time of any file watcher test to exceed 10 seconds per test. |

### RG-FW-2 — CI 버킷 격리

| REQ ID | 패턴 | 요구사항 (한국어) | 영문 보조 |
|--------|------|-------------------|-----------|
| REQ-FW-010 | Ubiquitous | 시스템은 `.github/workflows/ci-v3-pane.yml:248~313` 의 `tmux-test` job 의 `cargo test ... -- --ignored` 명령을 USER-DECISION-FW-B 로 결정된 격리 방식 (cargo test name filter, cfg feature flag, separate test binary, 또는 별 job) 으로 변경하여 tmux 의존 테스트만 실행한다. | The system **shall** modify the `cargo test ... -- --ignored` command in `tmux-test` job (`.github/workflows/ci-v3-pane.yml:248~313`) using the isolation method chosen via USER-DECISION-FW-B so only tmux-dependent tests run. |
| REQ-FW-011 | Ubiquitous | 시스템은 file watcher 테스트가 별도 CI step 또는 별 job 에서 실행되도록 `.github/workflows/ci-v3-pane.yml` 에 신설하며, file watcher 테스트의 실패가 `tmux-test` job 의 success 에 영향을 주지 않도록 격리한다. | The system **shall** add a separate CI step or job for file watcher tests in `.github/workflows/ci-v3-pane.yml` such that file watcher failures do not affect `tmux-test` job success. |
| REQ-FW-012 | State-Driven | RG-FW-1 의 결정성 보장이 검증된 동안, 시스템은 file watcher 테스트 실행 step 또는 job 을 branch protection required contexts 에 추가 가능한 상태로 둔다 (실제 추가 여부는 CLAUDE.local.md §2.1 운영 결정에 위임). | While RG-FW-1 deterministic guarantee is verified, the system **shall** keep file watcher test step/job in a state addable to branch protection required contexts (actual addition deferred to CLAUDE.local.md §2.1 operations). |
| REQ-FW-013 | Optional | USER-DECISION-FW-B 가 (b) cfg feature flag 또는 (c) separate test binary 옵션으로 결정될 경우, 시스템은 해당 옵션의 보조 변경 (Cargo.toml feature 추가, 신규 tests/ 파일 등) 을 적용한다. | Where USER-DECISION-FW-B chooses (b) cfg feature flag or (c) separate test binary, the system **shall** apply the supporting changes (Cargo.toml feature addition, new tests/ file, etc.). |

---

## 6. Acceptance Criteria

| AC ID | 검증 시나리오 | 통과 조건 | 검증 수단 | RG 매핑 | 파일 / 라인 |
|------|--------------|----------|----------|---------|------------|
| AC-FW-1 | `test_detect_file_creation` 이 로컬 macOS + 로컬 Linux + CI macos-14 + CI ubuntu-22.04 4 환경에서 단일 실행 시 PASS | 4 환경 각각 exit code = 0 + assertion `received.is_ok()` 통과 + 매칭 이벤트 경로 정확 | manual run + CI run | RG-FW-1 (REQ-FW-001) | `crates/moai-fs/src/lib.rs:165` |
| AC-FW-2 | `test_unwatch_stops_events` 가 위 4 환경에서 단일 실행 시 PASS | 4 환경 각각 exit code = 0 + assertion `received.is_err()` 통과 (unwatch 후 timeout 발생 = 정상) | manual run + CI run | RG-FW-1 (REQ-FW-002) | `crates/moai-fs/src/lib.rs:210` |
| AC-FW-3 | CI 에서 두 file watcher 테스트가 100회 연속 실행 시 99회 이상 PASS | 100회 run 의 PASS count ≥ 99 (job 1회 = 두 테스트 1회 실행 기준) | CI matrix retry script (`for i in 1..100; do gh run rerun ... ; done`) 또는 stress workflow | RG-FW-1 (REQ-FW-003) | (CI 통계) |
| AC-FW-4 | 단일 file watcher 테스트 실행 시간이 worst-case 10초 이내 | `cargo test --release -p moai-fs ... -- --ignored test_detect_file_creation` 의 wall-clock < 10s, `test_unwatch_stops_events` 도 동일 | CI assertion (timeout step) | RG-FW-1 (REQ-FW-004) | `crates/moai-fs/src/lib.rs:165, 210` |
| AC-FW-5 | `tmux-test` CI job 의 `cargo test ... -- --ignored` 명령이 변경되어 file watcher 테스트가 실행되지 않음 | `gh run view ... --log` 출력에 `test_detect_file_creation` / `test_unwatch_stops_events` 부재 + `integration_tmux_nested` 또는 `ctrl_b_passes_through_to_nested_tmux` 존재 | CI log 검증 | RG-FW-2 (REQ-FW-010) | `.github/workflows/ci-v3-pane.yml:312~313` |
| AC-FW-6 | file watcher 테스트가 별도 step 또는 job 에서 실행되며, 그 step/job 의 실패가 `tmux-test` job 의 success 에 영향을 주지 않음 | 신규 step (예: `Gate — cargo test (moai-fs file watcher, --ignored)`) 또는 신규 job (예: `fs-watcher-test`) 가 `.github/workflows/ci-v3-pane.yml` 에 존재 + dependency / `needs:` 관계가 file watcher → tmux-test 의 영향 차단 | yaml 검토 + CI run 실험 (의도적 fail 주입) | RG-FW-2 (REQ-FW-011) | `.github/workflows/ci-v3-pane.yml` |

---

## 7. 비기능 요구사항

| 항목 | 요구 |
|------|------|
| 단일 file watcher 테스트 worst-case 실행 시간 | 10초 이내 (REQ-FW-004) |
| 단일 file watcher 테스트 typical 실행 시간 | 1초 이내 (CI median 기준) |
| Flake rate (100회 연속 CI 실행 기준) | < 1% (REQ-FW-003) |
| moai-fs 공개 API 변경 | 금지 (G5, N1) |
| moai-fs `[dependencies]` 추가 | USER-DECISION-FW-C 가 (a) default 결정 시 금지 (G6); (b)/(c) 시만 허용 |
| CI yaml 변경 범위 | `.github/workflows/ci-v3-pane.yml:240~313` `tmux-test` job + 신규 step/job |
| `code_comments` 언어 | `ko` (`.moai/config/sections/language.yaml`) — 다만 본 SPEC 작성 시점 에이전트 위임 프롬프트 (CLAUDE.local.md §9.5) 가 영어 강제 시 영어로 작성 가능 |
| `documentation` 언어 | `ko` (본 spec.md / plan.md / research.md) |

---

## 8. 의존성 / 통합 인터페이스

### 8.1 선행 SPEC

- 없음. 본 SPEC 은 standalone — moai-fs 의 현행 `FsWatcher` 구현과 `.github/workflows/ci-v3-pane.yml:248` 의 `tmux-test` job 만을 입력으로 한다.

### 8.2 후속 SPEC

- **CLAUDE.local.md §2.1 운영 변경** (SPEC 외 운영 작업): 본 SPEC 완료 후 `tmux-test (macOS)` 와 `tmux-test (Linux)` 를 branch protection required contexts 에 추가. 본 SPEC 의 acceptance 가 그 운영 변경의 prerequisite.

### 8.3 관련 SPEC (영향 가능)

- **SPEC-V3-011 (Cross-platform Packaging & Auto-update)**: release infrastructure SPEC. branch protection 안정화가 release tag push 시 CI green 의 사전 조건. 본 SPEC 완료가 V3-011 의 외부 차단 일부 해소.

### 8.4 외부 의존

- `notify` v7 (`crates/moai-fs/Cargo.toml:8`) — 변경 없음 (USER-DECISION-FW-C default).
- `tokio` test-util feature — 이미 dev-dependency (line 14). 추가 없음.
- `tempfile` v3 — 이미 dev-dependency (line 15). 추가 없음.
- GitHub Actions runners (`macos-14`, `ubuntu-22.04`) — 변경 없음.

### 8.5 외부 차단

- 없음. 본 SPEC 은 implement 진입에 외부 인증서 / billing / 계약 등의 차단이 없다. USER-DECISION 두 게이트만 결정되면 즉시 implement 가능.

---

## 9. 마일스톤 (priority-based, 시간 추정 없음)

### MS-1 (Priority: High) — Research + USER-DECISION 결정

산출:
- USER-DECISION-FW-A 결정 (Axis A 옵션: A1/A2/A3/A4/A5).
- USER-DECISION-FW-B 결정 (Axis B 옵션: B1/B2/B3/B4).
- (선택) USER-DECISION-FW-C 결정 (notify-debouncer 도입 여부) — research §4 의 권고는 omit.
- 결정 결과를 plan.md 의 progress 로그 또는 본 spec.md 의 HISTORY 에 기록.
- tmux 테스트의 함수/모듈 path enumeration (B1 / B4 선택 시 prerequisite).

### MS-2 (Priority: High) — 테스트 결정성 (Axis A 적용)

산출:
- `crates/moai-fs/src/lib.rs:165` (`test_detect_file_creation`) 의 timing 패턴을 USER-DECISION-FW-A 결정 옵션으로 교체.
- `crates/moai-fs/src/lib.rs:210` (`test_unwatch_stops_events`) 의 timing 패턴 교체.
- 로컬 검증: macOS + Linux 양 환경에서 단일 실행 PASS.
- AC-FW-1, AC-FW-2, AC-FW-4 통과.

### MS-3 (Priority: High) — CI 버킷 격리 (Axis B 적용) + 안정성 검증

산출:
- `.github/workflows/ci-v3-pane.yml:312~313` 의 `tmux-test` job 명령을 USER-DECISION-FW-B 결정 옵션으로 변경.
- 필요 시 신규 step (예: `Gate — cargo test (moai-fs file watcher, --ignored)`) 또는 신규 job (예: `fs-watcher-test`) 추가.
- (B2 또는 B3 결정 시) Cargo.toml feature 추가 또는 신규 `tests/fs_watcher.rs` 파일 생성.
- 안정성 검증: 100회 연속 CI 실행 (또는 동등 stress run) 으로 flake rate < 1% 확인.
- AC-FW-3, AC-FW-5, AC-FW-6 통과.
- (운영 후속) CLAUDE.local.md §2.1 의 required contexts 에 `tmux-test (macOS)` + `tmux-test (Linux)` 추가 검토 (본 SPEC 외 운영 작업).

---

## 10. USER-DECISION 게이트

### 10.1 USER-DECISION-FW-A — 테스트 결정성 패턴 (MS-2 진입)

**결정 (2026-04-27): RESOLVED → 옵션 (a) A3 polling with bounded retry. deterministic upper bound 5초 + 50~100ms 폴링 간격. 코드 변경 ~10 LOC, 신규 dependency 0. typical case 100ms 내 성공.**

질문: "moai-fs file watcher 테스트의 결정성 확보 패턴은?"

옵션:
- (a) **권장: A3 (polling with bounded retry)** — 단일 큰 deadline (예: 5초) 안에서 짧은 폴링 간격 (50~100ms) 으로 이벤트 수신 시도. 첫 매칭 즉시 성공. 코드 변경 ~10 LOC. 신규 dependency 0. deterministic upper bound 명시. typical case 는 100ms 이내 성공.
- (b) A2 (retry loop with exponential backoff) — `100ms, 200ms, 400ms, ...` 간격으로 재시도. 일정 횟수 (예: 5회) 후 fail. CI 부하 변동에 적응. worst-case ~3초. 구현 복잡도 약간 증가.
- (c) A1 (sleep 시간 증가) — `200ms → 1000ms`, `500ms → 2000ms` 의 단순 patch. 외부 의존 0. 근본 원인 미해소, CI 부하가 마진을 초과하면 여전히 flaky.
- (d) A4 (notify backend mock) — 신규 trait 도입으로 테스트에서 mock backend 주입. 100% deterministic (1ms 안). 단 production 의 진짜 OS event 검증 못함 (false confidence). 공개 API 변경 위험.

영향 범위: REQ-FW-001, REQ-FW-002, MS-2 산출. 본 SPEC 의 핵심 결정.

### 10.2 USER-DECISION-FW-B — CI 버킷 격리 방식 (MS-3 진입)

**결정 (2026-04-27): RESOLVED → 옵션 (a) B1 cargo test name filter. tmux-test job: `cargo test ... -- --ignored 'tmux'` substring filter. file watcher 는 신규 step `cargo test -p moai-fs --all-targets -- --ignored` 로 분리. 코드 변경 0, CI yaml 수정만. MS-1 의 tmux 테스트 enumeration 단계 (T1) 에서 prefix 컨벤션 확인 필수 (R-FW-3 완화).**

질문: "`tmux-test` 의 `--ignored` 버킷에서 file watcher 테스트를 격리하는 방식은?"

옵션:
- (a) **권장: B1 (cargo test name filter)** — `cargo test ... -- --ignored 'tmux'` (또는 동등 substring) 로 narrow. file watcher 테스트는 별도 step `cargo test -p moai-fs --all-targets -- --ignored` 로 실행. 코드 변경 0, CI yaml 만 수정. tmux 테스트가 prefix 컨벤션 (예: `tmux_*`, `integration_tmux_*`) 을 따르는지 MS-1 enumeration 으로 확인 필요.
- (b) B2 (cfg feature flag) — `#[cfg(feature = "tmux-tests")]` + `#[cfg(feature = "fs-watcher-tests")]` 로 분리. 명시적이고 type-safe. Cargo.toml feature 추가 + 모든 ignored 테스트에 cfg 추가 필요. 빌드 multiplication.
- (c) B3 (separate test binary) — `crates/moai-fs/tests/fs_watcher.rs` integration test 로 이동. cargo 표준 패턴 준수. 단 super::* private API 접근 시 추가 노출 필요할 수 있음 (G5 위반 위험 — out-of-scope 항목).
- (d) B4 (두 별도 CI job) — `tmux-test` job 은 narrow + 신설 `fs-watcher-test` job. 가장 명확한 분리. 단 CI yaml 가장 큰 변경 + job duplication.

영향 범위: REQ-FW-010, REQ-FW-011, REQ-FW-013, MS-3 산출.

### 10.3 USER-DECISION-FW-C — notify-debouncer 도입 (선택, MS-2 진입 시)

**결정 (2026-04-27): OMITTED. research §4 권고에 따라 본 SPEC 은 (a) notify v7 유지 default 가정. notify-debouncer 도입은 별 SPEC 으로 분리 검토.**

질문: "notify v7 을 그대로 유지할 것인가, notify-debouncer 를 도입할 것인가?"

옵션:
- (a) **권장: notify v7 유지** — 본 SPEC 의 결정성 + 격리 문제 해소에 debouncer 가 본질적 해결책 아님 (debouncer 의 timeout 도 결국 deadline 필요). 신규 dependency 0. production behavior 무변경.
- (b) notify-debouncer-mini 도입 — minimal API. event batching. timeout 명시적 → 테스트 결정성 개선 부수 효과. 신규 dependency. production 의 burst event 처리 개선.
- (c) notify-debouncer-full 도입 — file rename 추적 + cookie 매칭 등 풍부한 기능. 신규 dependency 가장 무거움. 본 SPEC 범위 초과 가능성.

영향 범위: G6 (Cargo.toml `[dependencies]` 추가 금지) 의 예외 가능. (b) 또는 (c) 결정 시 별 SPEC 분리 권고 — 본 SPEC 에서 동시 진행 시 RG-FW 외 새로운 RG (production behavior 변경) 추가 필요.

[INFO] 본 게이트는 omit 가능. research §4 의 권고는 별 SPEC 분리, 본 SPEC 은 (a) default 가정.

---

## 11. 위험 (Risk Register)

| ID | 위험 | 영향 | 완화 |
|----|------|------|------|
| R-FW-1 | USER-DECISION-FW-A (a) A3 의 5초 deadline 도 high-load CI 에서 부족 | AC-FW-3 의 99% PASS 미충족 | deadline 을 10초로 increase, 또는 환경변수 (`MOAI_FS_TEST_DEADLINE_MS`) 로 외부 조정 가능하게 + REQ-FW-004 의 worst-case 10초 한계 검토 |
| R-FW-2 | USER-DECISION-FW-B (a) B1 의 cargo test name filter 가 substring match 라 의도 외 테스트가 매칭 | tmux 테스트 누락 또는 file watcher 테스트 의도 외 실행 | MS-1 의 tmux 테스트 enumeration 단계에서 명시적 prefix 컨벤션 확정 (예: `tmux_*`, `fs_watcher_*`), 또는 USER-DECISION-FW-B (c)/(d) 로 escalate |
| R-FW-3 | tmux 테스트 이름이 `tmux::` 또는 통일 prefix 미준수 | B1 적용 불가능 → USER-DECISION-FW-B 재결정 | MS-1 의 first task 로 enumeration + 필요 시 모듈 reorganization (단 그 자체는 본 SPEC 의 RG-FW-2 범위 내) |
| R-FW-4 | Axis A 적용 후에도 일부 환경 (예: macOS Apple Silicon Rosetta, ARM Linux) 에서 flake 잔존 | AC-FW-3 부분 미충족 | 본 SPEC 무관 — 별 issue 로 escalate. 본 SPEC 의 검증 환경은 macos-14 + ubuntu-22.04 한정 |
| R-FW-5 | 100회 연속 CI 실행이 GitHub Actions billing 부담 | AC-FW-3 검증 지연 | 50회 또는 30회로 sample size 축소 + 통계 검정 변경 (예: < 2% flake rate), 또는 local stress test 로 부분 검증 |

---

## 12. 외부 인터페이스 (불변 약속)

본 SPEC 은 다음 인터페이스를 fix 한다. 후속 SPEC 또는 코드 변경은 이를 신뢰할 수 있다:

```rust
// crates/moai-fs/src/lib.rs — 본 SPEC 으로 변경 안됨

pub struct FsWatcher { /* opaque */ }

impl FsWatcher {
    pub fn new() -> Result<(Self, mpsc::Receiver<FsEvent>), FsWatcherError>;
    pub fn watch(&mut self, path: &Path) -> Result<(), FsWatcherError>;
    pub fn unwatch(&mut self, path: &Path) -> Result<(), FsWatcherError>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum FsEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Removed(PathBuf),
}

#[derive(Debug, Error)]
pub enum FsWatcherError {
    WatchError(notify::Error),
    ChannelClosed,
}

pub use watcher::{FsEventBus, WorkspaceEvent, WorkspaceKey};
pub use workspace_watcher::WorkspaceWatcher;
```

본 SPEC 으로 변경 가능:
- 두 `#[ignore]` 테스트 (`test_detect_file_creation`, `test_unwatch_stops_events`) 의 내부 로직 (timing 패턴).
- `.github/workflows/ci-v3-pane.yml:240~313` 의 `tmux-test` job 명령 + 신규 step/job.
- (USER-DECISION-FW-B (b)/(c) 결정 시) `crates/moai-fs/Cargo.toml` 의 `[features]` 또는 `crates/moai-fs/tests/fs_watcher.rs` 신설.

본 SPEC 으로 변경 안됨:
- `FsWatcher::new` / `watch` / `unwatch` 의 시그니처 + 동작.
- `FsEvent` enum variant.
- `FsWatcherError` enum variant.
- `notify` v7 dependency 자체 (USER-DECISION-FW-C (a) default).
- `crates/moai-fs/src/{tree_watcher, watcher, workspace_watcher}.rs` 의 모든 코드.
- `tmux-test` job 의 runner / matrix / tmux 설치 step.

---

## 13. 추적성

### 13.1 CLAUDE.local.md ↔ 본 SPEC

| CLAUDE.local.md 섹션 | 본 SPEC 매핑 |
|---------------------|--------------|
| §2.1 main 의 required contexts (현재 7 개) | 본 SPEC 완료 후 `tmux-test (macOS)` + `tmux-test (Linux)` 추가 가능 |
| §2.1 "Excluded (별개 이슈, 추후 SPEC 으로 fix 후 추가)" | 본 SPEC 이 그 SPEC. file watcher 항목 해소 |
| §2.3 develop 의 required contexts | 본 SPEC 완료 후 동일 추가 가능 |
| §9 영어 주석 정책 | 본 SPEC 으로 변경되는 테스트 코드는 신규 수정이므로 영어 주석 적용 필요 (CLAUDE.local.md §9.2 touch-on-modify) |

### 13.2 코드베이스 변경 범위

| 파일 | 변경 종류 | RG 매핑 |
|------|----------|---------|
| `crates/moai-fs/src/lib.rs` (line 162~237) | 두 테스트의 내부 로직 변경 | RG-FW-1 |
| `.github/workflows/ci-v3-pane.yml` (line 240~313) | `tmux-test` job 명령 + 신규 step/job | RG-FW-2 |
| `crates/moai-fs/Cargo.toml` (옵션) | `[features]` 추가 (USER-DECISION-FW-B (b) 결정 시만) | RG-FW-2 (REQ-FW-013) |
| `crates/moai-fs/tests/fs_watcher.rs` (옵션) | 신규 파일 (USER-DECISION-FW-B (c) 결정 시만) | RG-FW-2 (REQ-FW-013) |

위 외 파일은 본 SPEC 으로 변경되지 않는다 (G5, N1, N2 carry).

---

## 14. 용어 정의

| 용어 | 정의 |
|------|------|
| flaky 테스트 | 동일 코드 + 동일 환경에서도 실행마다 PASS/FAIL 이 비결정적으로 변하는 테스트. 본 SPEC 의 두 테스트가 그 예. |
| `--ignored` 버킷 | `cargo test ... -- --ignored` 명령으로 한꺼번에 실행되는 모든 `#[ignore]` 테스트의 집합. |
| deterministic upper bound | 최악의 경우에도 결과가 정해진 시간 안에 결정되는 보장. 본 SPEC 의 A3 패턴이 이를 명시 (예: 5초 deadline). |
| polling with bounded retry | 짧은 간격 (예: 50ms) 으로 반복 시도하되 전체 deadline (예: 5초) 안에서만 시도. 첫 매칭 즉시 성공. |
| settle delay | unwatch 직후 system 이 안정 상태에 도달할 때까지의 대기 시간. 본 SPEC 의 `test_unwatch_stops_events` 가 이 개념을 다룸. |
| substring match | `cargo test pattern` 의 default 매칭. 함수 full path (예: `moai_fs::tests::test_detect_file_creation`) 의 임의 부분과 substring 매칭. |
| inotify | Linux kernel 의 file system event 알림 메커니즘. notify crate 의 Linux backend. |
| FSEvents | macOS 의 file system event 알림 API. notify crate 의 macOS backend. |
| polling backend | inotify/FSEvents 가용하지 않을 때 notify 가 사용하는 fallback. `with_poll_interval` 로 주기 설정. |
| branch protection required contexts | GitHub 의 branch protection rule 에서 PR 머지 전 PASS 요구되는 CI status check 목록. CLAUDE.local.md §2.1 정의. |

---

## 15. 변경 이력 정책

본 spec.md 는 추가 revision 누적 시 `## 16. Sprint Contract Revisions` section 을 신설하고 `### 16.1 / 16.2 / ...` 로 누적한다 (SPEC-V3-011 §15 패턴 따름).

---

작성 종료. 본 spec.md 는 plan.md (구현 milestone × task) + research.md (배경 분석) 와 함께 SPEC-V3-FS-WATCHER-001 implement 진입의 입력이다. implement 는 별도 feature 브랜치 (`feature/SPEC-V3-FS-WATCHER-001-bucket-isolation` 또는 동등) 에서 USER-DECISION-FW-A + FW-B 결정 후 시작한다.
