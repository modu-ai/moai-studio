---
id: SPEC-V0-2-0-WIZARD-ENV-001
version: 1.0.0
status: draft
created_at: 2026-05-04
updated_at: 2026-05-04
author: MoAI (main session)
priority: Medium
issue_number: 0
depends_on: [SPEC-V0-2-0-ONBOARDING-ENV-001]
milestones: [MS-1]
language: ko
labels: [v0.2.0, ui, onboarding, wizard, audit-top-8, lightweight]
revision: v1.0.0 (lightweight) — Lightweight SPEC fast-track per .claude/rules/moai/workflow/spec-workflow.md §Plan Phase
---

# SPEC-V0-2-0-WIZARD-ENV-001: ProjectWizard env report state binding (audit F-6 후속)

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-05-04 | 초안. v0.2.0 cycle Sprint 11+ (audit Top 8 #6 후속). SPEC-V0-2-0-ONBOARDING-ENV-001 (env detect API) 의 첫 consumer 도입 — ProjectWizard 가 EnvironmentReport state 를 보유하고 외부에서 set 가능하도록 확장. 기존 5-step navigation / 시그니처 무변경 (R3 새 필드만). UI render 통합은 별 PR carry. Lightweight SPEC fast-track 적용. |

---

## 1. 목적

`ProjectWizard` (wizard.rs, 408 LOC) 가 SPEC-V0-2-0-ONBOARDING-ENV-001 의 `EnvironmentReport` 를 state 로 보유하도록 확장한다. 외부 (RootView 또는 wizard mount caller) 가 `detect_with_runner(&RealCommandRunner)` 결과를 wizard 에 주입하고, wizard 가 후속 render (별 PR) 에서 사용자에게 환경 정보를 제공할 수 있도록 한다.

본 SPEC scope 는 **state binding only** — ProjectWizard 의 5-step navigation 은 그대로, 신규 step 추가 없음. EnvironmentReport 가 Some 일 때 어떤 UI 를 보여줄지는 별 PR (Lightweight 후보 — F-6 wizard render 통합).

audit feature-audit.md §3 Tier F line 224 / §4 #6 의 "F-6 Onboarding tour ... 환경 감지 (shell/tmux/node/python/rust 자동 detect) + interactive tour" 의 wizard 측 first hook.

**Lightweight SPEC fast-track** 적격성:
- spec.md ≤ 10 KB ✅
- AC 6 (≤ 8) ✅
- milestones 1 (≤ 2) ✅
- no architectural impact (wizard.rs 단일 파일 + R3 새 필드만 + onboarding 모듈 의존성 1개 추가) ✅
- 단일 PR (~150 LOC) ✅

---

## 2. 목표 (Goals)

- G1. `ProjectWizard` 에 `env_report: Option<EnvironmentReport>` field 추가 (R3 새 필드, 기존 5 필드 무변경).
- G2. `ProjectWizard::new()` 가 `env_report = None` 으로 초기화한다 (lazy injection).
- G3. `ProjectWizard::set_env_report(report)` setter — 외부 caller 가 detect 결과 주입.
- G4. `ProjectWizard::env_report()` getter — `Option<&EnvironmentReport>` 반환.
- G5. `ProjectWizard::clear_env_report()` — None 으로 reset.
- G6. `ProjectWizard::mount()` 와 `dismiss()` 의 기존 동작 무변경. dismiss 시 reset() 가 env_report 도 None 으로 초기화.
- G7. WizardStep enum 무변경 (5 variant 그대로). step navigation 무변경. build_workspace 무변경.

---

## 3. Non-Goals / Exclusions

- N1. **WizardStep::Step0Env enum variant 추가** — 5-step navigation 변경은 별 SPEC 또는 별 PR.
- N2. **Render 통합** — env_report 가 Some 일 때 wizard UI 에 표시 — 별 PR.
- N3. **자동 detect 호출** — ProjectWizard 내부에서 `RealCommandRunner` 를 invoke 하지 않음. 외부 caller 가 detect 후 주입.
- N4. **RootView 통합** — RootView 가 wizard mount 시 자동 detect 호출 — 별 PR.
- N5. **Missing tool 안내 UI** — "tmux 가 없습니다 — `brew install tmux` 권장" 같은 가이드 — 별 SPEC.
- N6. **Refresh button** — 사용자가 wizard 안에서 재감지 트리거 — 별 PR.
- N7. **env_report → workspace metadata 저장** — 새 workspace 생성 시 환경 정보 저장 — 별 SPEC.

---

## 4. Requirements (EARS)

- **REQ-WE-001**: `ProjectWizard::new()` 가 `env_report = None` 으로 초기화한다. 다른 5 필드 (current_step / visible / selected_directory / project_name / spec_id / selected_color) 의 초기값 무변경.
- **REQ-WE-002**: `ProjectWizard::set_env_report(report: EnvironmentReport)` 가 field 를 `Some(report)` 로 갱신한다.
- **REQ-WE-003**: `ProjectWizard::env_report() -> Option<&EnvironmentReport>` 가 현재 field 를 반환한다.
- **REQ-WE-004**: `ProjectWizard::clear_env_report()` 가 field 를 `None` 으로 reset 한다.
- **REQ-WE-005**: `ProjectWizard::dismiss()` 호출 시 내부 `reset()` 이 env_report 도 None 으로 초기화한다 (기존 5 step state reset 과 동일 lifecycle).
- **REQ-WE-006**: `ProjectWizard::mount()` / `next_step()` / `prev_step()` / `can_go_next()` / `can_go_back()` / `build_workspace()` 의 동작 무변경. `WizardStep::all()` / `next()` / `prev()` 무변경.

---

## 5. Acceptance Criteria

| AC ID | Requirement | Given | When | Then | 검증 수단 |
|-------|-------------|-------|------|------|-----------|
| AC-WE-1 | REQ-WE-001 | `ProjectWizard::new()` 인스턴스 | env_report() 호출 | None 반환. 기존 visible / current_step 등 무변경 | unit test |
| AC-WE-2 | REQ-WE-002 / 003 | empty wizard | mock EnvironmentReport (3 entries) set + env_report() | Some(report), entries.len() == 3 | unit test |
| AC-WE-3 | REQ-WE-004 | wizard 가 env_report Some 보유 | clear_env_report() | None 반환 | unit test |
| AC-WE-4 | REQ-WE-005 | wizard 가 env_report Some + step Step3Spec + 다른 state 보유 | dismiss() | env_report None + 모든 5 step state reset (current_step Step1Directory, visible false, ...) | unit test |
| AC-WE-5 | REQ-WE-006 | env_report Some 인 상태 | next_step() / prev_step() / can_go_next() / can_go_back() | 동작이 env_report 와 무관하게 기존과 동일. WizardStep::ALL 5 entry 그대로 | unit test |
| AC-WE-6 | REQ-WE-006 | wizard.set_env_report + selected_directory + project_name 모두 set | build_workspace() | env_report 와 무관하게 NewWorkspace { name, project_path, spec_id, color_tag } 반환 (env 정보는 NewWorkspace 에 포함되지 않음 — N7) | unit test |

---

## 6. File Layout

### 6.1 수정

- `crates/moai-studio-ui/src/wizard.rs`:
  - `use crate::onboarding::EnvironmentReport;`
  - `ProjectWizard` 에 `env_report: Option<EnvironmentReport>` field 추가 (R3, 마지막에 배치)
  - `new()` 에 `env_report: None` 초기화
  - 신규 메서드 3개: `set_env_report` / `env_report` (getter) / `clear_env_report`
  - `reset()` 에 `self.env_report = None;` 추가 (REQ-WE-005)
  - 단위 테스트 ~6개

### 6.2 변경 금지 (FROZEN)

- `crates/moai-studio-ui/src/wizard.rs` 의 `WizardStep` enum 5 variant + `next` / `prev` / `ALL` 무변경
- `ProjectWizard::next_step / prev_step / can_go_next / can_go_back / mount / dismiss / is_visible / build_workspace` 시그니처 + 본문 무변경 (단, `reset` 은 새 필드 reset 만 추가)
- `NewWorkspace` struct 무변경
- `ColorTag` enum 무변경
- `crates/moai-studio-ui/src/onboarding/**` 무변경 (consumer 만 추가)
- 다른 모든 crate 무변경

---

## 7. Test Strategy

- 단위 테스트 minimum **6개** (wizard.rs `#[cfg(test)] mod tests` 신규 또는 확장):
  - new() 직후 env_report() == None (AC-WE-1)
  - set_env_report → env_report() == Some (AC-WE-2)
  - clear_env_report → None (AC-WE-3)
  - dismiss() → env_report None + 다른 state reset (AC-WE-4)
  - next_step / prev_step / can_go_next / can_go_back env_report 무관 (AC-WE-5)
  - build_workspace env_report 와 무관, NewWorkspace 출력 무변경 (AC-WE-6)
- 회귀: 기존 ui crate 1289 tests 모두 GREEN 유지.

---

## 8. DoD

본 SPEC PASS 시점에 외부 caller (예: RootView 또는 별 PR 의 wizard mount handler) 가:
1. `let report = detect_with_runner(&RealCommandRunner);`
2. `wizard.set_env_report(report);`
3. `wizard.mount();`

후 wizard render (별 PR) 가 `wizard.env_report()` 를 읽어 사용자에게 환경 정보 표시 가능. 본 SPEC 은 state binding 만 — render 는 carry.

audit Top 8 #6 F-6 진척: 50% (env detect logic) → **65%** (wizard state binding 완료, render + 자동 detect 호출 carry).

---

Version: 1.0.0 (lightweight) | Source: SPEC-V0-2-0-WIZARD-ENV-001 | 2026-05-04
