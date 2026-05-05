---
id: SPEC-V0-3-0-STATUS-BAR-WIRE-001
version: "1.0.0"
status: draft
created_at: 2026-05-05
updated_at: 2026-05-05
author: GOOS행님
priority: Medium
labels: [v0.3.0, sprint-2, lightweight, status-bar, audit-top16]
issue_number: null
---

# SPEC-V0-3-0-STATUS-BAR-WIRE-001 — StatusBar Functional Wire (agent-mode + workspace-switch git label + dispatch_command parity)

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-3-0-STATUS-BAR-WIRE-001 |
| **Title** | StatusBar carry — agent-mode setter / workspace-switch git label / `status.*` dispatch_command parity |
| **Status** | draft (Sprint 2 audit Top 16 follow-up) |
| **Priority** | Medium |
| **Revision** | 1.0 (lightweight) |
| **Dependencies** | SPEC-V3-006 MS-7 (audit F-4 — `status_bar` module 활성), SPEC-V0-3-0-MX-POPOVER-001 (audit Top 16 #11 동격 진입), SPEC-V0-3-0-PALETTE-POLISH-001 (CommandRegistry 확장 사례) |
| **Cycle** | v0.3.0 Sprint 2 (audit Top 16 follow-up — StatusBar functional wire) |
| **Milestones** | MS-1 |

## HISTORY

- 2026-05-05: 초안 작성. v0.3.0 Sprint 2 audit Top 16 follow-up. SPEC-V3-006 MS-7 (#? merged) 가 `crates/moai-studio-ui/src/status_bar.rs` 에 `StatusBarState` (agent_mode / git_branch / lsp_status 3 widget) + `render_status_bar` 를 state-bearing 으로 도입했지만, 실제 user-facing 진입점 (workspace 전환 시 git label 갱신 / `status.*` palette command / agent mode set/clear API 노출) 은 **모두 보류 상태** 다. RootView 의 `status_bar: StatusBarState` 필드 (lib.rs:275) 는 default 로만 초기화 (lib.rs:419) 되고, 어떤 호출 경로도 setter 를 호출하지 않는다. 본 SPEC 은 그 carry 누락분 중 가장 가벼운 3 함정 (workspace 전환 시 git label refresh / `status.*` dispatch routing 분기 / agent mode setter parity) 만 wire 한다. AC 수 (≤6) / milestones (≤2) 모두 lightweight 충족.

## 1. Purpose / 배경

`crates/moai-studio-ui/src/status_bar.rs` 의 `StatusBarState` 는 SPEC-V3-006 MS-7 에서 state-bearing 으로 활성되었으나, mutation API (`set_agent_mode` / `set_git_branch` / `set_lsp_status` 및 각 `clear_*`) 가 **외부에서 호출되지 않는다**. 결과적으로 status bar 는 항상 default 렌더 ("no git" + "moai-studio v{ver}" + "⌘K to search") 만 표시하고, command palette 의 `status.*` namespace 는 미정의 상태이며 (palette/registry.rs grep 결과 0건), `dispatch_command` 에도 `status.` 분기가 없다 (lib.rs grep 결과 0건). 또한 `handle_activate_workspace` (lib.rs:831~845) 가 active workspace 전환을 처리하지만 status bar git label 을 갱신하지 않는다.

본 SPEC 은 이 cold start 를 끊는 최소 단위 wire 만 수행한다. PANE-WIRE-001 / SURFACE-MENU-WIRE-001 의 cx-free helper + dispatch routing 패턴을 그대로 차용하여 audit Top 16 의 StatusBar follow-up 한 줄을 닫는다. 신규 widget UI 추가, LSP polling 통합, git poller 도입은 모두 본 SPEC 범위 밖.

## 2. Goals / 목표

- `dispatch_command` 가 `status.set_agent_mode` / `status.clear_agent_mode` / `status.refresh_git` 3 id 를 인식 (graceful degradation: 알 수 없는 `status.*` id 는 `false` 반환)
- `handle_activate_workspace` 가 workspace 전환 후 status bar git label 을 active workspace 의 식별자로 갱신 (혹은 git 정보 부재 시 clear)
- `RootView` 가 cx-bound `set_status_agent_mode` / `clear_status_agent_mode` / `refresh_status_git_label` 3 helper 를 노출하고, `cx.notify()` 호출로 즉시 재렌더 트리거
- cx-free helper `route_status_command_to_kind(&str) -> Option<StatusCommand>` 로 dispatch routing 분리 (PANE-WIRE-001 의 `route_pane_command_to_kind` 동일 패턴)
- cx-free helper `derive_status_git_label_from_workspace(&Workspace) -> Option<(String, bool)>` 로 workspace → (branch, dirty) 매핑 분리
- TRUST 5 gates (clippy / fmt / cargo test) ALL PASS, 기존 ui crate 1363+ tests 회귀 0 (additive only, +5~7 tests 신규)

## 3. Non-Goals / Exclusions

- `status_bar.rs` 모듈 내부 로직 변경 (FROZEN — 호출만, 무수정)
- LSP status chip 동기화 / lsp 백엔드 polling 통합 (별 SPEC, 본 SPEC 은 agent_mode + git label 만)
- 실제 git branch detection / `git2` crate 호출 (workspace 식별자 → label 매핑만, 실제 git 호출 X)
- StatusBar 내부 신규 widget 추가 (FROZEN — 3 widget 유지)
- `palette/registry.rs` 에 `status.*` 신규 entry 추가 (graceful degradation — 본 SPEC 은 `dispatch_command` routing 만 활성, palette 진입은 별 SPEC carry)
- StatusBar 클릭 인터랙션 / 우클릭 메뉴 / popover 진입 (별 SPEC, 본 SPEC 은 dispatch + setter wire 만)
- StatusBarState 의 새 필드 추가 (FROZEN — 3 widget skeleton 유지)
- Multi-window 의 StatusBar 동기화 (single-window 만 대상)

FROZEN (touch 금지):
- `crates/moai-studio-ui/src/status_bar.rs` (모듈 전체 read-only — `StatusBarState` API 호출만)
- `crates/moai-studio-terminal/**`
- `crates/moai-studio-workspace/**`
- 기존 `dispatch_command` 의 `settings.*` / `theme.*` / `workspace.*` / `pane.*` / `surface.*` 분기 동작
- 진행 중 SPEC (V3-004 / V3-005 / V3-014) 산출물

## 4. Touchpoints / 현행 scaffolding 위치

- `crates/moai-studio-ui/src/status_bar.rs:50` — `StatusBarState` 정의 (agent_mode / git_branch / lsp_status 3 Option)
- `crates/moai-studio-ui/src/status_bar.rs:70~103` — `set_agent_mode` / `clear_agent_mode` / `set_git_branch` / `clear_git_branch` / `set_lsp_status` / `clear_lsp_status` setter API (이미 존재, 호출자만 부재)
- `crates/moai-studio-ui/src/status_bar.rs:134` — `render_status_bar(&StatusBarState)` 자유 함수 (read-only)
- `crates/moai-studio-ui/src/lib.rs:275` — `RootView::status_bar: status_bar::StatusBarState` 필드
- `crates/moai-studio-ui/src/lib.rs:419` — `status_bar: status_bar::StatusBarState::default()` 초기화
- `crates/moai-studio-ui/src/lib.rs:831~845` — `handle_activate_workspace` (active workspace 전환 진입점, status bar git label 갱신 hook 위치)
- `crates/moai-studio-ui/src/lib.rs:966~` — `dispatch_command(&mut self, id: &str) -> bool` (palette command dispatch — 본 SPEC 에서 `status.*` 분기 추가 대상)
- `crates/moai-studio-ui/src/lib.rs:2488` — `render_status_bar(&self.status_bar)` 호출 (Render::render 내부, 무수정)

## 5. Requirements

- REQ-SBW-001: `RootView` 는 `set_status_agent_mode(&mut self, mode: impl Into<String>, cx: &mut Context<Self>)` cx-bound helper 를 가진다. `self.status_bar.set_agent_mode(mode)` 호출 후 `cx.notify()` 트리거.
- REQ-SBW-002: `RootView` 는 `clear_status_agent_mode(&mut self, cx: &mut Context<Self>)` cx-bound helper 를 가진다. `self.status_bar.clear_agent_mode()` 호출 후 `cx.notify()` 트리거.
- REQ-SBW-003: `RootView` 는 `refresh_status_git_label(&mut self, cx: &mut Context<Self>)` cx-bound helper 를 가진다. 현재 active workspace 가 존재하면 cx-free `derive_status_git_label_from_workspace` 호출 결과로 `set_git_branch` 또는 `clear_git_branch` 를 결정하여 적용한다. `cx.notify()` 트리거.
- REQ-SBW-004: `handle_activate_workspace` 는 store.touch 직후 REQ-SBW-003 helper 를 호출하여 status bar git label 을 갱신한다 (기존 동작 보존, additive only).
- REQ-SBW-005: cx-free helper `route_status_command_to_kind(&str) -> Option<StatusCommand>` 가 `status.set_agent_mode` / `status.clear_agent_mode` / `status.refresh_git` 3 id 를 각각 `Some(StatusCommand::SetAgentMode)` / `Some(StatusCommand::ClearAgentMode)` / `Some(StatusCommand::RefreshGit)` 로 매핑하고, 알 수 없는 id 는 `None` 반환. `StatusCommand` enum 은 `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` 만 부여 (lightweight).
- REQ-SBW-006: `dispatch_command` 는 `status.` prefix 분기를 통해 REQ-SBW-005 routing 결과로 helper 호출. agent mode 의 payload 는 별도 인자 (current dispatch_command signature 무변경) 를 받지 않으므로, `status.set_agent_mode` 분기는 placeholder 라벨 `"Plan"` 으로 하드코딩 호출 (실제 mode payload 전달은 carry-to). 알 수 없는 `status.*` id 는 `false` 반환 (graceful degradation).
- REQ-SBW-007: cx-free helper `derive_status_git_label_from_workspace(workspace_id: &str) -> Option<(String, bool)>` 는 빈 문자열은 `None`, 비어있지 않은 ID 는 `Some((id.to_string(), false))` 반환 (placeholder mapping — 실제 git 호출 carry-to). 향후 git2 crate 통합 시 본 helper 만 교체.

## 6. Acceptance Criteria

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-SBW-1 | `route_status_command_to_kind` cx-free helper | `route_status_command_to_kind("status.set_agent_mode")` 호출 | `Some(StatusCommand::SetAgentMode)` 반환 | unit test (`route_status_command_to_kind_set_agent_mode`) |
| AC-SBW-2 | `route_status_command_to_kind` cx-free helper | `route_status_command_to_kind("status.clear_agent_mode")` / `route_status_command_to_kind("status.refresh_git")` 각각 호출 | 각각 `Some(StatusCommand::ClearAgentMode)` / `Some(StatusCommand::RefreshGit)` 반환 | unit test (`route_status_command_to_kind_clear_and_refresh`) |
| AC-SBW-3 | `route_status_command_to_kind` cx-free helper + 알 수 없는 id (`status.unknown_xxx` / `status.` / `notstatus.set_agent_mode`) | 각각 호출 | 모두 `None` 반환 (graceful) | unit test (`route_status_command_to_kind_unknown_returns_none`) |
| AC-SBW-4 | `derive_status_git_label_from_workspace` cx-free helper + 비어있지 않은 workspace ID `"main-ws"` | 호출 | `Some(("main-ws".to_string(), false))` 반환 (placeholder mapping, dirty=false) | unit test (`derive_status_git_label_returns_workspace_id`) |
| AC-SBW-5 | `derive_status_git_label_from_workspace` cx-free helper + 빈 문자열 ID | 호출 | `None` 반환 (label clear 신호) | unit test (`derive_status_git_label_empty_id_returns_none`) |
| AC-SBW-6 | cargo build/clippy/fmt + ui crate test | run | ALL PASS, 기존 ui crate 1363+ tests 회귀 0 (additive only, +5 신규 tests T-SBW 블록) | CI |

(AC 합계: 6. lightweight 한도 ≤8 충족. 모두 cx-free helper 단위 검증.)

## 7. File Layout

| Path | Status | Note |
|------|--------|------|
| `crates/moai-studio-ui/src/lib.rs` | modified | 3 RootView helper (cx-bound), 2 cx-free helper (`route_status_command_to_kind` / `derive_status_git_label_from_workspace`), `StatusCommand` enum 노출, `dispatch_command` 의 `status.*` 분기 추가 (3 branch + unknown → false), `handle_activate_workspace` 가 `refresh_status_git_label` 호출, 신규 unit tests (T-SBW 블록 5개) |
| `.moai/specs/SPEC-V0-3-0-STATUS-BAR-WIRE-001/spec.md` | created | 본 문서 |
| `.moai/specs/SPEC-V0-3-0-STATUS-BAR-WIRE-001/progress.md` | created | run 진입 시 갱신 stub |

추가 파일 없음. `status_bar.rs` 는 read-only — 기존 `StatusBarState` setter/clearer API 호출만.

FROZEN (touch 금지):
- `crates/moai-studio-terminal/**`
- `crates/moai-studio-workspace/**`
- `crates/moai-studio-ui/src/status_bar.rs` (전체 read-only)
- `crates/moai-studio-ui/src/palette/registry.rs` (본 SPEC 무수정 — palette entry carry-to)
- 진행 중 SPEC (V3-004 / V3-005 / V3-014) 산출물

## 8. Test Strategy

ui crate `lib.rs::tests` 모듈에 신규 unit test 5개 추가 (T-SBW 블록).

- AC-SBW-1/2/3: `route_status_command_to_kind` cx-free 단위 검증 (정상 매칭 / 다중 분기 / unknown → None)
- AC-SBW-4/5: `derive_status_git_label_from_workspace` cx-free 단위 검증 (비어있지 않은 ID → Some, 빈 ID → None)
- AC-SBW-6: cargo gate (3-gate)

GPUI cx-bound 부분 (`set_status_agent_mode` / `clear_status_agent_mode` / `refresh_status_git_label`) 은 PANE-WIRE-001 / SURFACE-MENU-WIRE-001 에서 확립한 동일 정책 (cx-free helper 분리 + cx-bound wrapper 는 GPUI-level 검증 생략) 을 따른다. `dispatch_command` 의 `status.*` 분기는 routing helper 가 `Some` 반환하면 cx-bound helper 호출, `None` 이면 `false` 반환 (graceful) 의 form 이 되어 routing helper 단위 검증으로 분기 정확성을 100% 검증.

회귀 검증: 기존 ui crate 1363+ tests 무영향 (additive only, 기존 dispatch_command 분기 무수정).

본 SPEC run 단계에서 `cargo test -p moai-studio-ui --lib` + `cargo clippy -p moai-studio-ui --all-targets -- -D warnings` + `cargo fmt --all -- --check` 3 gate 통과 필수.

---

Version: 1.0.0 (lightweight)
Created: 2026-05-05
Cycle: v0.3.0 Sprint 2 (audit Top 16 follow-up — StatusBar functional wire)
Carry-from: SPEC-V3-006 MS-7 (status_bar 모듈 skeleton 도입), audit Top 16 (StatusBar 진입점 미구현 항목)
Carry-to: (a) palette/registry.rs 에 `status.*` 신규 entry, (b) `set_agent_mode` payload 전달 메커니즘 (현재 placeholder "Plan"), (c) 실제 git2 crate 통합 (`derive_status_git_label_from_workspace` 본체 교체), (d) LSP polling → `set_lsp_status` 호출 wire (별 SPEC)
