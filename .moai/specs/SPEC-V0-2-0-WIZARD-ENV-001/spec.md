---
id: SPEC-V0-2-0-WIZARD-ENV-001
version: 1.1.0
status: draft
created_at: 2026-05-04
updated_at: 2026-05-04
author: MoAI (main session)
priority: Medium
issue_number: 0
depends_on: [SPEC-V0-2-0-ONBOARDING-ENV-001]
milestones: [MS-1, MS-2]
language: ko
labels: [v0.2.0, ui, onboarding, wizard, audit-top-8, lightweight]
revision: v1.1.0 (lightweight + MS-2 amendment) — Lightweight SPEC fast-track per .claude/rules/moai/workflow/spec-workflow.md §Plan Phase
---

# SPEC-V0-2-0-WIZARD-ENV-001: ProjectWizard env report state binding (audit F-6 후속)

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-05-04 | 초안. v0.2.0 cycle Sprint 11+ (audit Top 8 #6 후속). SPEC-V0-2-0-ONBOARDING-ENV-001 (env detect API) 의 첫 consumer 도입 — ProjectWizard 가 EnvironmentReport state 를 보유하고 외부에서 set 가능하도록 확장. 기존 5-step navigation / 시그니처 무변경 (R3 새 필드만). UI render 통합은 별 PR carry. Lightweight SPEC fast-track 적용. |
| 1.1.0-draft | 2026-05-04 | MS-2 amendment 추가. v0.2.0 cycle Sprint 13 (audit Top 8 #6 후속 — F-6 100% 목표). MS-1 의 state binding 을 GPUI render + RootView lifecycle 와 wire: (a) ProjectWizard::render 가 env_report Some 시 banner UI (available count + missing tools), None 시 "Detecting environment..." placeholder 표시. (b) RootView::handle_add_workspace 가 wizard.mount() 직후 cx.spawn + cx.background_executor 로 detect_with_runner(&RealCommandRunner) 비동기 실행 → 완료 시 wizard.set_env_report(report). main thread freeze 회피. Lightweight SPEC fast-track 7번째 적용 (PLUGIN-MGR / TOOLBAR-WIRE / ONBOARDING-ENV / OSC8-LIFECYCLE MS-1 / WIZARD-ENV MS-1 / OSC8-LIFECYCLE MS-2 이후). |

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

## MS-2 — Wizard render env banner + RootView async auto-detect

### MS-2.1 목적

MS-1 이 wizard.rs 에 추가한 env_report Option 필드 + 3 setter/getter 메서드를 GPUI render path + RootView lifecycle 와 wire 한다. (a) ProjectWizard::render 가 env_report 상태에 따라 banner UI 또는 "Detecting..." placeholder 를 표시. (b) RootView::handle_add_workspace 가 wizard.mount() 직후 비동기 detect 를 spawn 하여 결과를 wizard 에 주입. 600ms 의 std::process::Command 호출 chain 으로 인한 main thread freeze 를 회피하기 위해 cx.spawn + cx.background_executor 패턴 사용.

### MS-2.2 목표 (Goals)

- G2-1. `ProjectWizard::render` 본문 (현재 line 261~352) 의 wizard panel children chain 에 신규 env section 한 줄 삽입 — Header / Progress bar / Step title / Step content / Navigation 사이의 적절한 위치 (권장: Header 와 Progress bar 사이, 또는 Step title 위).
- G2-2. 신규 `pub(crate) fn render_env_section(&self) -> gpui::Div` (또는 `impl ProjectWizard` block 내 method) 가 env_report 상태에 따라 분기:
  - `None` → small text "Detecting environment..." (`tok::FG_TERTIARY` 또는 `FG_SECONDARY`, `text_xs`)
  - `Some(report)` → banner with "Environment: {N}/6 tools available" 헤드라인 + missing tools list (있을 때 — `report.missing_tools()` iterate, 각 tool display_name 한 줄씩, `tok::FG_DANGER` 또는 accent 색상)
  - `Some(report)` 가 `is_complete()` true → green check + "All tools detected" 단일 라인
- G2-3. 신규 pure helper `format_env_summary(report: &EnvironmentReport) -> String` — "5/6 tools available" 형식. cx 의존 없음, 단위 테스트 가능.
- G2-4. 신규 pure helper `format_missing_tools_label(report: &EnvironmentReport) -> Option<String>` — 미발견 tool display_name 들을 ", " 로 join. 모두 발견 시 `None` 반환. 단위 테스트 가능.
- G2-5. `RootView::handle_add_workspace` (lib.rs:1884) 본문 확장:
  - 기존 `wizard.mount()` 호출 직후 (cx.notify() 전 또는 후)
  - `let bg_task = cx.background_executor().spawn(async move { detect_with_runner(&RealCommandRunner) });`
  - `cx.spawn(|this, mut cx| async move { let report = bg_task.await; this.update(&mut cx, |rv, cx| { if let Some(wizard) = &rv.project_wizard { wizard.update(cx, |w, cx| { w.set_env_report(report); cx.notify(); }); } }).ok(); }).detach();`
  - 단, 정확한 GPUI 0.2.2 API 이름 (`cx.background_executor()` vs `cx.background_spawn()`) 은 implementation 단계에서 확인 — fallback path 는 동기 detect (덜 선호, MVP).
- G2-6. RootView 측 신규 helper `pub(crate) fn spawn_env_detect_into_wizard(&mut self, cx: &mut Context<Self>)` 또는 `fn trigger_env_detect(&mut self, cx: &mut Context<Self>)` — handle_add_workspace 외에서도 재사용 가능 (testing seam).
- G2-7. 비동기 detect 의 단위 테스트 회피: pure helper (G2-3, G2-4) 와 sync injection path (`wizard.set_env_report(mock_report)` → render 검증) 만 단위 테스트. 실제 cx.spawn 은 manual smoke test 에 의존.
- G2-8. `ProjectWizard` struct 에 new state 필드 추가 없음 (MS-1 의 5+1 필드 무변경, R3 보존). "Detecting" 상태는 env_report None 으로 표현 (no Option<Result<>>, no separate is_detecting flag).

### MS-2.3 Non-Goals (MS-2 scope)

- M2-N1. **Refresh button** — 사용자가 wizard 안에서 재감지 트리거 — 별 PR (Lightweight 후보).
- M2-N2. **Tool 별 install 가이드 deep-link** — 예: "tmux 가 없습니다 — `brew install tmux`" 같은 install command suggestion — 별 SPEC.
- M2-N3. **Tool icon / svg** — display_name 옆에 시각적 아이콘 — 별 SPEC (UI design tokens 확장 필요).
- M2-N4. **WizardStep::Step0Env 신규 step** — 5-step navigation 변경 — 별 SPEC.
- M2-N5. **env_report → workspace metadata 저장** — 새 workspace 생성 시 환경 정보 저장 — 별 SPEC.
- M2-N6. **Detect timeout / cancellation** — 비동기 detect 의 timeout 처리 또는 wizard dismiss 시 task cancel — 별 SPEC. 본 MS-2 는 detect 가 항상 완료된다고 가정 (wizard dismissed 후 결과 도착 시 wizard.update 가 wizard 자체에 영향 없음 — visible=false).
- M2-N7. **Error state UI** — detect_with_runner 가 panic 또는 RealCommandRunner 가 process spawn 실패 시 — std::process::Command 가 자체 ToolStatus::Error 로 매핑하므로 banner 가 그 결과를 그대로 표시 (별도 error UI 불필요).

### MS-2.4 Requirements (EARS)

- **REQ-WE-007**: `ProjectWizard::render` 의 wizard panel 이 env section 한 줄을 child 로 포함한다 (현재 Header / Progress bar / Step title / Step content / Navigation 다섯 children 의 chain 에 추가). 위치는 Header 직후 (Progress bar 앞) 권장.
- **REQ-WE-008**: env section 이 `env_report() == None` 시 "Detecting environment..." 1 line text (`text_xs`, `FG_SECONDARY` 또는 `FG_TERTIARY`) 표시. 시각적으로 비파괴적 (wizard 의 다른 element 와 layout 충돌 없음).
- **REQ-WE-009**: env section 이 `env_report() == Some(report)` 일 때 (a) "Environment: {available_count}/6 tools available" 헤드라인 (`text_sm`, `FG_PRIMARY`) (b) report.is_complete() == false 시 missing tools display_name 을 ", " 로 join 한 라인 (`text_xs`, `FG_DANGER` 또는 accent), (c) report.is_complete() == true 시 "All tools detected" 단일 라인 (`text_xs`, `FG_SUCCESS` 또는 accent).
- **REQ-WE-010**: 신규 pure helper `format_env_summary(&EnvironmentReport) -> String` 가 `format!("{}/6 tools available", report.available_count())` 반환 (포맷 정확히 일치).
- **REQ-WE-011**: 신규 pure helper `format_missing_tools_label(&EnvironmentReport) -> Option<String>` 가 (a) `report.missing_tools().is_empty()` 시 `None` 반환, (b) 그 외 missing tool 의 `display_name()` 들을 ", " 로 join 한 `Some(String)` 반환.
- **REQ-WE-012**: `RootView::handle_add_workspace` 가 기존 `wizard.mount()` + `cx.notify()` 호출에 추가로 비동기 detect path 를 trigger 한다 — `cx.background_executor().spawn(...)` (또는 GPUI 0.2.2 의 동등 API) 로 `detect_with_runner(&RealCommandRunner)` 호출, `cx.spawn` 으로 결과 await + `wizard.set_env_report(report)` + `cx.notify()`. 메인 thread freeze 없음. 동기 fallback (덜 선호, ~600ms freeze 수용) 은 GPUI API 가용성 검증 후 implementation 단계 결정.
- **REQ-WE-013**: 신규 RootView 메서드 `pub(crate) fn trigger_env_detect(&mut self, cx: &mut Context<Self>)` 또는 동등 helper 가 비동기 detect path 를 캡슐화한다. handle_add_workspace 가 이를 호출. 외부에서 reuse 가능 (예: 향후 refresh button 에서 호출).
- **REQ-WE-014**: ProjectWizard struct 에 new field 추가 없음. MS-1 의 6 필드 (current_step / visible / selected_directory / project_name / spec_id / selected_color / env_report) 무변경. WizardStep enum 5 variant + ALL/next/prev 무변경. set_env_report / env_report / clear_env_report 시그니처 + body 무변경. mount / dismiss / reset / next_step / prev_step / can_go_next / can_go_back / build_workspace 무변경.

### MS-2.5 Acceptance Criteria

| AC ID | Requirement | Given | When | Then | 검증 수단 |
|-------|-------------|-------|------|------|-----------|
| AC-WE-7 | REQ-WE-010 | `EnvironmentReport::new(vec![(Tool::Shell, ToolStatus::Available { version: "5.9".into() }), (Tool::Tmux, ToolStatus::NotFound), (Tool::Node, ToolStatus::Available { version: "20".into() })])` (3 entries, 2 available) | `format_env_summary(&report)` | `"2/6 tools available"` 정확 일치 | unit test |
| AC-WE-8 | REQ-WE-011 (negative) | 6 tools 모두 Available 인 report | `format_missing_tools_label(&report)` | `None` 반환 | unit test |
| AC-WE-9 | REQ-WE-011 (positive) | Tmux + Python NotFound, 나머지 4 Available 인 report | `format_missing_tools_label(&report)` | `Some("tmux, python")` (display_name 순서, ", " 구분, lowercase 정확 일치 — Tool::display_name() 출력 그대로) | unit test |
| AC-WE-10 | REQ-WE-008/009/014 | `ProjectWizard::new()` | mount() + render 호출 | render 가 panic 없이 div 반환 (env_report None 분기 진입). render 는 GPUI 의존이므로 직접 test 어려우나 helper 직접 호출로 대체 가능 — `wizard.env_report() == None` 검증 + helper text "Detecting environment..." 가 코드에 존재 (string literal grep 검증) | unit test (env_report None 검증) + 코드 inspection |
| AC-WE-11 | REQ-WE-009 (Some + incomplete) | wizard.set_env_report(report with Tmux NotFound, others Available) | render 호출 (또는 helper 직접) | format_env_summary(&report) == "5/6 tools available" + format_missing_tools_label(&report) == Some("tmux") | unit test (2개 helper 직접 호출 검증) |
| AC-WE-12 | REQ-WE-009 (Some + complete) | wizard.set_env_report(report with all 6 Available) | render 호출 (또는 helper 직접) | format_env_summary == "6/6 tools available" + format_missing_tools_label == None | unit test |
| AC-WE-13 | REQ-WE-012/013 | `RootView::trigger_env_detect` 메서드 존재 + `RootView::handle_add_workspace` 본문이 mount 외에 detect path 를 trigger | 코드 inspection | `trigger_env_detect` 정의 존재 + handle_add_workspace 본문에 호출 또는 동등 spawn block 존재 | unit test (메서드 존재 컴파일 검증) + 코드 inspection |

### MS-2.6 File Layout

#### 수정

- `crates/moai-studio-ui/src/wizard.rs`:
  - `impl ProjectWizard` block (line 355+ render_navigation 근처) 에 신규 method `render_env_section(&self) -> gpui::Div` 추가 (env_report 분기, "Detecting..." vs banner)
  - 신규 free function `pub(crate) fn format_env_summary(report: &EnvironmentReport) -> String` (REQ-WE-010, AC-WE-7)
  - 신규 free function `pub(crate) fn format_missing_tools_label(report: &EnvironmentReport) -> Option<String>` (REQ-WE-011, AC-WE-8/9)
  - `Render::render` 본문 (line 261~352) 의 wizard panel children chain 에 `render_env_section` 호출 한 줄 삽입 (Header 다음, Progress bar 앞)
  - `use crate::onboarding::{EnvironmentReport, Tool}` (Tool 추가 import — display_name() 호출용)
  - 단위 테스트 ~5개 추가 (T-WE-MS2 블록 — 4 helper test + 1 wizard env_report None mount test)

- `crates/moai-studio-ui/src/lib.rs`:
  - `use crate::onboarding::{detect_with_runner, RealCommandRunner};` (line 32 wizard mod 선언 근처)
  - `RootView::trigger_env_detect(&mut self, cx: &mut Context<Self>)` 신규 메서드 (REQ-WE-013) — body: cx.background_executor + cx.spawn + wizard.update(set_env_report)
  - `handle_add_workspace` 본문에 `self.trigger_env_detect(cx);` 한 줄 추가 (mount 직후, cx.notify 전)
  - 단위 테스트 ~2개 추가 (trigger_env_detect 메서드 존재 컴파일 검증, handle_add_workspace 본문에 trigger_env_detect 호출 존재 — 가능하면 RootView::new() 후 handle_add_workspace 호출 → wizard mount 검증, env_report 는 async 이므로 즉시 검증 안 됨)

#### 변경 금지 (MS-2 FROZEN)

- `crates/moai-studio-ui/src/wizard.rs` 의 `WizardStep` enum + ALL/next/prev/number/title 무변경
- `ProjectWizard` 의 6 필드 (current_step/visible/selected_directory/project_name/spec_id/selected_color/env_report) 무변경 (R3 새 필드 추가 없음)
- `ProjectWizard::new / mount / dismiss / reset / next_step / prev_step / can_go_next / can_go_back / build_workspace / set_env_report / env_report / clear_env_report / is_visible` 시그니처 + body 무변경
- `NewWorkspace`, `ColorTag` 무변경
- `crates/moai-studio-ui/src/onboarding/**` 무변경 (consumer 만 추가)
- `crates/moai-studio-ui/src/lib.rs` 의 RootView 다른 필드 + 다른 메서드 무변경 (trigger_env_detect 한 메서드 추가 + handle_add_workspace 본문 1 line 추가만)

### MS-2.7 Test Strategy

- 단위 테스트 minimum **7개**:
  - `wizard.rs` (~5):
    - `format_env_summary_partial` (AC-WE-7) — 3 entries (2 available) → "2/6 tools available"
    - `format_env_summary_complete` (AC-WE-12 보조) — 6 all available → "6/6 tools available"
    - `format_missing_tools_label_none` (AC-WE-8) — all available → None
    - `format_missing_tools_label_some` (AC-WE-9) — Tmux + Python missing → Some("tmux, python")
    - `wizard_render_with_env_report_none_does_not_panic` (AC-WE-10) — `let mut w = ProjectWizard::new(); w.mount();` + `w.env_report()` 가 None 인지 검증 (render 자체는 GPUI 의존이라 cx 필요, 직접 호출 회피)
  - `lib.rs` (~2):
    - `trigger_env_detect_method_exists_and_compiles` — `let _: fn(&mut RootView, &mut Context<RootView>) = RootView::trigger_env_detect;` 컴파일 검증
    - `handle_add_workspace_after_call_wizard_mounted` — RootView 초기화 (가능한 한도 내) + handle_add_workspace 호출 → wizard.is_visible() == true 검증 (env_report 는 async path 이므로 검증 skip — 별 manual smoke)
- 회귀: ui crate 1305 → ~1312 (+7) GREEN. 기존 T-WE 6 tests (MS-1) 무변경.
- clippy 0 warning, fmt clean.

### MS-2.8 DoD

본 MS-2 PASS 시점에 사용자가:
1. RootView 의 "+ New Workspace" 버튼 또는 empty state CTA 클릭
2. → wizard 즉시 mount (visible) — UI freeze 없음
3. → wizard 가 "Detecting environment..." 텍스트 표시 (한 줄, Header 직후)
4. → ~600ms 후 env detect 완료 — wizard 가 "Environment: 5/6 tools available" + missing tools list (또는 "All tools detected") 표시
5. → 사용자가 wizard step 1~5 진행 (env detect 와 무관, 기존 navigation 동작)

audit Top 8 #6 F-6 진척: 65% → **100%** (state binding + render + 자동 detect 모두 완료, F-6 GA).

### MS-2.9 후속 carry (별 SPEC)

본 MS-2 GA 후 잔존 가능 enhancement:
- Refresh button (Tool re-detect on demand) — 별 PR
- Tool 별 install 가이드 deep-link — 별 SPEC
- Detect timeout / cancellation — 별 SPEC
- Tool icon visualization — 별 SPEC (UI design tokens 확장)
- env_report 를 workspace metadata 에 저장 — 별 SPEC

---

Version: 1.1.0 (lightweight, MS-1 GA + MS-2 amendment) | Source: SPEC-V0-2-0-WIZARD-ENV-001 | 2026-05-04
