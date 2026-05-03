# SPEC-V0-2-0-ONBOARDING-ENV-001 Progress

**Started**: 2026-05-04
**Branch**: feature/SPEC-V0-2-0-ONBOARDING-ENV-001
**SPEC status**: implemented (MS-1 complete)
**Completion date**: 2026-05-04
**audit reference**: feature-audit.md §3 Tier F line 224 (F-6) + §4 Top 8 #6 (⭐⭐⭐)
**Classification**: Lightweight SPEC fast-track (spec.md 9126 bytes ≤10KB, 1 MS, 7 REQ / 7 AC, no architectural impact)

## MS-1 (2026-05-04 sess 12+) — env detect module ✅

### Implementation

- `crates/moai-studio-ui/src/onboarding/mod.rs` (신규):
  - `pub mod env;`
  - re-exports of public types
- `crates/moai-studio-ui/src/onboarding/env.rs` (신규):
  - `Tool` enum (6 variant — Shell/Tmux/Node/Python/Rust/Git)
  - `Tool::all() / executable / display_name / version_arg` API
  - `ToolStatus` enum (Available { version } / NotFound / Error { message })
  - `EnvironmentReport` struct + `available_count / missing_tools / is_complete`
  - `CommandRunner` trait + `RealCommandRunner` impl
  - `detect_with_runner(&dyn CommandRunner) -> EnvironmentReport`
  - `parse_version_from_stdout` helper
  - 단위 테스트 ~10개
- `crates/moai-studio-ui/src/lib.rs`:
  - `pub mod onboarding;` 추가만 (R3)

### Acceptance Criteria

| AC | 내용 | 상태 |
|----|------|------|
| AC-OE-1 | Tool::all() 길이 6 + 순서 | ✅ |
| AC-OE-2 | 각 variant executable/display_name/version_arg | ✅ |
| AC-OE-3 | ToolStatus 3 variant exhaustive | ✅ |
| AC-OE-4 | EnvironmentReport available_count / missing / is_complete | ✅ |
| AC-OE-5 | mock runner all-Available → is_complete=true | ✅ |
| AC-OE-6 | mock runner 절반 NotFound → available_count=3 | ✅ |
| AC-OE-7 | parse_version_from_stdout 첫 줄 + 빈 입력 fallback | ✅ |

### Test count

- 신규: 13 (env.rs T-OE 블록)
  - tool_all_returns_six_in_canonical_order (AC-OE-1)
  - tool_metadata_is_non_empty_for_every_variant (AC-OE-2)
  - tool_version_arg_matches_each_tool_convention (AC-OE-2)
  - tool_executable_is_stable_for_non_shell_tools (AC-OE-2)
  - tool_status_three_variants_exhaustive (AC-OE-3)
  - environment_report_helpers (AC-OE-4)
  - detect_with_runner_all_available_yields_complete (AC-OE-5)
  - detect_with_runner_mixed_yields_half_complete (AC-OE-6)
  - detect_with_runner_other_errors_map_to_status_error (REQ-OE-007 negative)
  - parse_version_from_stdout_first_line_trimmed (AC-OE-7)
  - parse_version_from_stdout_empty_returns_unknown (AC-OE-7)
  - parse_version_from_stdout_strips_whitespace (AC-OE-7)
  - parse_version_from_stdout_skips_leading_blank_lines (AC-OE-7)
- moai-studio-ui crate tests: 1276 → 1289 (+13)
- 회귀 0 (agent 129, terminal 36, workspace 26)
- clippy 0 warning, fmt clean

### Public API surface (env::*)

- `Tool` enum (6 variant) + `all() / executable / display_name / version_arg`
- `ToolStatus` enum (3 variant) + `is_available`
- `EnvironmentReport` struct + `new / available_count / missing_tools / is_complete`
- `CommandRunner` trait + `RealCommandRunner` impl
- `detect_with_runner(&dyn CommandRunner) -> EnvironmentReport`
- `parse_version_from_stdout(&str) -> String`

### Carry to next PR

- ProjectWizard 통합 (env step 추가 또는 onboarding 화면)
- Interactive tour
- Tool installation guidance (brew/apt 추천)
- Version constraint validation (e.g. node >= 20)
- Windows 지원

### DoD ✅

외부 caller 가 `detect_with_runner(&RealCommandRunner)` 호출 → `EnvironmentReport` 받아
`available_count` / `missing_tools` / `is_complete` 사용 가능. ProjectWizard 또는 별
onboarding 화면이 본 API 를 직접 호출 (별 PR 통합).

audit Top 8 #6 F-6 진척: PARTIAL → 50% (env detect 완료, interactive tour + wizard
통합 carry).
