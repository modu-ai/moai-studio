---
id: SPEC-V0-2-0-ONBOARDING-ENV-001
version: 1.0.0
status: draft
created_at: 2026-05-04
updated_at: 2026-05-04
author: MoAI (main session)
priority: Medium
issue_number: 0
depends_on: []
milestones: [MS-1]
language: ko
labels: [v0.2.0, ui, onboarding, env-detect, audit-top-8, lightweight]
revision: v1.0.0 (lightweight) — Lightweight SPEC fast-track per .claude/rules/moai/workflow/spec-workflow.md §Plan Phase
---

# SPEC-V0-2-0-ONBOARDING-ENV-001: Onboarding environment detection module (audit F-6)

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-05-04 | 초안. v0.2.0 cycle Sprint 10 (audit Top 8 #6 ⭐⭐⭐). audit feature-audit.md §3 Tier F line 224 의 "F-6 Onboarding tour — wizard.rs 12.7KB 5-step structure 있음. 환경 감지 (shell/tmux/node/python/rust 자동 detect) + interactive tour 추가" 의 첫 building block (환경 감지). interactive tour + ProjectWizard 통합은 별 PR carry. Lightweight SPEC fast-track 적용. |

---

## 1. 목적

신규 모듈 `crates/moai-studio-ui/src/onboarding/env.rs` 도입 — 사용자 환경의 6 tool (shell / tmux / node / python / rust / git) 가용성과 버전을 감지하는 read-only API. ProjectWizard 또는 onboarding tour 가 본 모듈을 호출해 환경 정보를 얻고, 사용자에게 missing tool 안내 / version warning 등을 제공할 수 있게 한다.

본 SPEC scope 는 **detection logic only** — UI 통합 (ProjectWizard 의 새 step 또는 별 onboarding 화면) 은 별 PR carry. 이는 detection 이 single-file pure module 이며 mock runner 로 단위 테스트가 결정적인 반면, UI 통합은 wizard step 추가 + 사용자 흐름 결정이 필요한 별개의 작업이기 때문.

audit feature-audit.md §3 Tier F line 224: "F-6 Onboarding tour — wizard.rs 12.7KB 5-step structure 있음. 환경 감지 (shell/tmux/node/python/rust 자동 detect) + interactive tour 추가." 의 첫 절반 해소.

**Lightweight SPEC fast-track** 적격성:
- spec.md ≤ 10 KB ✅
- AC 7 (≤ 8) ✅
- milestones 1 (≤ 2) ✅
- no architectural impact (단일 신규 모듈, 외부 dependency 0, 기존 모듈 수정 없음) ✅
- 단일 PR (~450 LOC) ✅

---

## 2. 목표 (Goals)

- G1. 신규 `Tool` enum — 6 variant: `Shell`, `Tmux`, `Node`, `Python`, `Rust`, `Git`. display name + executable name + version flag (`--version` etc.) 노출.
- G2. 신규 `ToolStatus` enum — `Available { version: String }` / `NotFound` / `Error { message: String }` 3 variant.
- G3. 신규 `EnvironmentReport` struct — `Vec<(Tool, ToolStatus)>` carrier + 카운트 / 분류 helpers (`available_count`, `missing_tools`, `is_complete`).
- G4. 신규 `CommandRunner` trait — `run(executable, args) -> Result<String, std::io::Error>` 추상화. testability + mock runner.
- G5. 신규 `RealCommandRunner` — `std::process::Command` 기반 실 구현.
- G6. 신규 `detect_with_runner(runner) -> EnvironmentReport` pure function — 6 tool 모두 검사 후 report 반환.
- G7. version 파싱 helper — `--version` stdout 의 첫 줄 trim + 빈 값 처리.

---

## 3. Non-Goals / Exclusions

- N1. **ProjectWizard 통합** — wizard step 추가 또는 별 onboarding 화면 — 별 PR.
- N2. **Interactive tour** — step-by-step 가이드 — 별 SPEC.
- N3. **Tool installation guidance** — missing tool 시 brew/apt/pacman 명령 추천 — 별 SPEC.
- N4. **Version constraint validation** — "node >= 20 필요" 등 정책 — 별 SPEC.
- N5. **Windows 지원** — POSIX (macOS / Linux) 전용. v0.2.0 Windows scope 외.
- N6. **PATH 외 위치 검색** — `Command::new(name)` 가 PATH 만 검색. /opt /usr/local 직접 스캔 안 함.
- N7. **Async / concurrent detection** — 6 tool 순차 실행. 6 × ~50ms ≈ 300ms 허용.
- N8. **Cache** — 매 호출 detect. 호출자가 caching 책임.

---

## 4. Requirements (EARS)

- **REQ-OE-001**: `Tool` enum 이 6 variant 노출 — `Shell`, `Tmux`, `Node`, `Python`, `Rust`, `Git`. `Tool::all()` 가 정확히 6 entry 를 정해진 순서로 반환한다.
- **REQ-OE-002**: 각 `Tool` 가 `executable() -> &'static str` (e.g., "node"), `display_name() -> &'static str` (e.g., "Node.js"), `version_arg() -> &'static str` (e.g., "--version") 를 노출한다. Shell 의 executable 은 `std::env::var("SHELL")` 의 basename, 실패 시 "sh".
- **REQ-OE-003**: `ToolStatus` enum 이 3 variant — `Available { version: String }`, `NotFound`, `Error { message: String }`.
- **REQ-OE-004**: `EnvironmentReport` 가 `entries: Vec<(Tool, ToolStatus)>` 를 보유하며 `available_count() -> usize`, `missing_tools() -> Vec<Tool>`, `is_complete() -> bool` (모든 tool Available) 를 제공한다.
- **REQ-OE-005**: `CommandRunner` trait 이 `fn run(&self, executable: &str, args: &[&str]) -> std::io::Result<String>` 시그니처를 노출한다. Ok 는 stdout 의 UTF-8 lossy String, Err 은 spawn / wait 실패.
- **REQ-OE-006**: `RealCommandRunner` 가 `std::process::Command::new(executable).args(args).output()` 으로 실 명령을 실행하고 stdout 을 반환한다. exit code 비-0 이면 stderr 메시지를 io::Error 로 반환.
- **REQ-OE-007**: `detect_with_runner(runner) -> EnvironmentReport` 가 `Tool::all()` 를 순회하며 각 tool 에 대해 (a) runner.run(executable, &[version_arg]) 호출, (b) 성공 시 `parse_version_from_stdout` 으로 첫 줄 trim 한 결과를 `Available { version }` 로, (c) 실패가 NotFound (executable 없음) 패턴이면 `NotFound`, (d) 그 외 실패는 `Error { message }` 로 매핑한다. 결과를 entries 순서대로 collect 후 EnvironmentReport 반환.

---

## 5. Acceptance Criteria

| AC ID | Requirement | Given | When | Then | 검증 수단 |
|-------|-------------|-------|------|------|-----------|
| AC-OE-1 | REQ-OE-001 | `Tool::all()` 호출 | 결과 array 검사 | 정확히 6 entry, 순서 = [Shell, Tmux, Node, Python, Rust, Git] | unit test |
| AC-OE-2 | REQ-OE-002 | 각 `Tool` variant | `executable()` / `display_name()` / `version_arg()` 호출 | 비어있지 않은 string. node→"node"/"Node.js"/"--version", git→"git"/"Git"/"--version", tmux→"tmux"/"tmux"/"-V" | unit test (각 variant) |
| AC-OE-3 | REQ-OE-003 | `ToolStatus::Available { version: "1.0".into() }` / `ToolStatus::NotFound` / `ToolStatus::Error { ... }` | match exhaustive | 3 variant 모두 매치, payload 접근 가능 | unit test |
| AC-OE-4 | REQ-OE-004 | mock report (3 Available + 2 NotFound + 1 Error) | available_count / missing_tools / is_complete | 3 / 3 missing entries / false | unit test |
| AC-OE-5 | REQ-OE-005, REQ-OE-007 | mock runner (모든 tool 에 대해 "v1.2.3" 반환) | detect_with_runner | 6 entries 모두 Available { version: "v1.2.3" }, is_complete = true | unit test |
| AC-OE-6 | REQ-OE-007 | mock runner (절반 NotFound, 절반 Available) | detect_with_runner | available_count == 3, missing_tools.len() == 3 | unit test |
| AC-OE-7 | REQ-OE-007 (parse_version_from_stdout) | runner stdout = "node v20.10.0\n  more text" | parse_version_from_stdout | 첫 줄 trim 한 "node v20.10.0" 반환. 빈 입력 → "(unknown)" | unit test |

---

## 6. File Layout

### 6.1 신규

- `crates/moai-studio-ui/src/onboarding/mod.rs` — `pub mod env;` + `pub use env::{Tool, ToolStatus, EnvironmentReport, CommandRunner, RealCommandRunner, detect_with_runner};`
- `crates/moai-studio-ui/src/onboarding/env.rs` — 본 SPEC 의 단일 파일 구현 (~400-450 LOC + ~10 단위 테스트).

### 6.2 수정

- `crates/moai-studio-ui/src/lib.rs` — `pub mod onboarding;` 추가만.

### 6.3 변경 금지 (FROZEN)

- `crates/moai-studio-terminal/**` 전체.
- `crates/moai-studio-workspace/**` 전체.
- `crates/moai-studio-agent/**` 전체.
- `crates/moai-studio-ui/src/wizard.rs` (별 PR 통합).
- `crates/moai-studio-ui/src/lib.rs` 의 다른 모든 영역 (R3 — `pub mod onboarding;` 한 줄만 추가).

---

## 7. Test Strategy

- 단위 테스트 minimum **10개**:
  - `Tool::all()` 길이 + 순서 (AC-OE-1)
  - 각 6 variant 의 executable/display_name/version_arg 검증 (AC-OE-2)
  - `ToolStatus` 3 variant 매치 + payload (AC-OE-3)
  - `EnvironmentReport` available_count / missing_tools / is_complete (AC-OE-4)
  - mock runner 모든 Available → 6 Available, is_complete=true (AC-OE-5)
  - mock runner 절반 NotFound → available_count == 3 (AC-OE-6)
  - parse_version_from_stdout 첫 줄 + 빈 입력 fallback (AC-OE-7)
- 통합 테스트 (실 PATH 검사) 는 `#[ignore]` 로 작성 — `cargo test --ignored` 명시 호출 시 실행. CI 비대상.
- 회귀: `cargo test -p moai-studio-ui --lib` 1276 tests 모두 GREEN 유지.

---

## 8. DoD

본 SPEC PASS 시점에 외부 caller 가 `detect_with_runner(&RealCommandRunner)` 호출 →
`EnvironmentReport` 받아 `available_count` / `missing_tools` / `is_complete` 사용 가능.
ProjectWizard 또는 별 onboarding 화면이 본 API 를 직접 호출 (또는 별 PR 에서 통합).

audit Top 8 #6 F-6 진척: PARTIAL → 50% (env detect 완료, interactive tour + wizard 통합 carry).

---

Version: 1.0.0 (lightweight) | Source: SPEC-V0-2-0-ONBOARDING-ENV-001 | 2026-05-04
