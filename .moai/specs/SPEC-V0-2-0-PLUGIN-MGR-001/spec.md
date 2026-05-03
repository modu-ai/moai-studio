---
id: SPEC-V0-2-0-PLUGIN-MGR-001
version: 1.0.0
status: draft
created_at: 2026-05-04
updated_at: 2026-05-04
author: MoAI (main session)
priority: Medium
issue_number: 0
depends_on: [SPEC-V3-013]
milestones: [MS-1]
language: ko
labels: [v0.2.0, ui, settings, plugin-manager]
revision: v1.0.0 (lightweight) — Lightweight SPEC fast-track per .claude/rules/moai/workflow/spec-workflow.md §Plan Phase
---

# SPEC-V0-2-0-PLUGIN-MGR-001: Plugin Manager UI — Settings Plugins pane skeleton

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-05-04 | 초안. v0.2.0 cycle Sprint 7 (audit Top 8 #3) — Plugin Manager UI skeleton. SPEC-V3-013 MS-4a/4b/4c/4d (Hooks/Mcp/Skills/Rules pane) 패턴을 mirror 하여 SettingsSection::Plugins 11번째 variant + PluginsPane skeleton + 6 default bundled plugin info entry 를 도입한다. Lightweight SPEC fast-track 적용 (spec.md ≤10KB / 1 MS / 7 AC). |

---

## 1. 목적

Settings Modal 의 sidebar 에 Plugins 항목을 추가하여, Claude Code marketplace 가 제공하는 plugin 의 read-only 카탈로그를 노출한다. v0.2.0 단계는 6 개 bundled plugin info 의 read-only list + search filter 만 제공하며, install/uninstall/enable 토글은 별 SPEC carry. 이는 SPEC-V3-013 의 Hooks/MCP/Skills/Rules pane 군 (10번째까지) 의 인접 그룹으로 11번째 variant 로 추가된다.

본 SPEC 은 **Lightweight SPEC fast-track** (spec-workflow.md §Plan Phase) 을 적용한다 — research.md / plan.md 생략, spec.md + progress.md 만으로 진행. 적격성: spec.md ≤ 10KB, AC = 7, milestones = 1, no architectural impact (settings_state.rs 의 enum 1 variant 추가 + 신규 pane 모듈 1 개), 단일 PR 예상 (~600 LOC).

---

## 2. 목표 (Goals)

- G1. `SettingsSection::Plugins` variant 추가 (11번째). `all()` / `label()` 확장. 기존 10 variant 동작 무변경.
- G2. `PluginsPane` skeleton 모듈 신규 (`crates/moai-studio-ui/src/settings/panes/plugins.rs`) — HooksPane 패턴 mirror.
- G3. 6 개 bundled plugin info 노출 (`PluginInfo` struct: name, source, version, description). v0.2.0 단계 카탈로그.
- G4. `PluginsState::plugin_filter` (search filter, case-insensitive substring) + `visible_plugins()` / `visible_count()` / `set_plugin_filter` / `clear_plugin_filter` API.
- G5. Settings Modal 의 `is_plugins_section` accessor + `selected_section` label/title 분기 추가.

---

## 3. Non-Goals / Exclusions

- N1. **Install / Uninstall / Enable toggle dispatch** — 별 SPEC (Plugin lifecycle management).
- N2. **Plugin marketplace HTTP fetch** — bundled hardcoded list 만, 네트워크 호출 없음.
- N3. **Plugin runtime sandbox 조정** — Claude Code 자체 plugin 시스템 위임.
- N4. **Detailed pane render (per-plugin detail view)** — list-only skeleton.
- N5. **Settings persistence schema 변경** — read-only catalog, no settings.json 영향.
- N6. **PluginsPane GPUI render side wire** — settings_modal main pane render (별 후속 PR, MS-2).

---

## 4. Requirements (EARS)

- **REQ-PM-001**: `SettingsSection::Plugins` 가 enum 의 11번째 variant 로 노출되어야 한다 (Advanced 다음 위치).
- **REQ-PM-002**: `SettingsSection::all()` 가 정확히 11 entry 를 반환해야 한다 (이전 10 → 11). 기존 10 entry 의 순서/identity 무변경.
- **REQ-PM-003**: `SettingsSection::Plugins.label()` 이 `"Plugins"` 를 반환해야 한다.
- **REQ-PM-004**: `PluginsPane::known_plugins()` 가 정확히 6 개의 `PluginInfo` 를 반환해야 한다 — name 은 모두 unique, source 는 비어있지 않음.
- **REQ-PM-005**: 빈 filter (default) 는 `visible_plugins()` 가 `known_plugins()` 와 동일한 길이를 반환해야 한다.
- **REQ-PM-006**: `PluginsPane::set_plugin_filter("X")` 적용 후 `visible_plugins()` 는 case-insensitive substring 매치로 필터링된 부분집합을 반환해야 한다 (name 또는 description 어느 한쪽 매치).
- **REQ-PM-007**: `SettingsViewState` 에 `plugins: PluginsState` 필드 추가, `Default::default()` 가 빈 filter 상태이어야 한다. 기존 9 개 state 필드 (appearance/keyboard/.../hooks) 무변경.

---

## 5. Acceptance Criteria

| AC ID | Requirement | Given | When | Then | 검증 수단 |
|-------|-------------|-------|------|------|-----------|
| AC-PM-1 | REQ-PM-001, REQ-PM-002 | `SettingsSection::all()` 호출 | 반환된 array 검사 | 길이 11, index 10 가 `Plugins`, 이전 0~9 entries 무변경 | unit test in settings_state.rs |
| AC-PM-2 | REQ-PM-003 | `SettingsSection::Plugins` | `.label()` 호출 | `"Plugins"` 반환 | unit test |
| AC-PM-3 | REQ-PM-004 | `PluginsPane::known_plugins()` 호출 | 반환된 slice 검사 | 길이 6, 모든 name unique, 모든 source 비어있지 않음, 모든 description 비어있지 않음 | unit test in plugins.rs |
| AC-PM-4 | REQ-PM-005 | 새 PluginsPane (default state) | `visible_count()` 호출 | 6 반환 (전체 노출) | unit test |
| AC-PM-5 | REQ-PM-006 | PluginsPane 에 `set_plugin_filter("git")` 후 | `visible_plugins()` 반환 | "git" 을 name 또는 description 에 (case-insensitive) 포함하는 entry 만 반환 | unit test |
| AC-PM-6 | REQ-PM-006 | PluginsPane 에 `set_plugin_filter("ZZZNonexistent")` 후 | `visible_count()` 반환 | 0 반환 | unit test |
| AC-PM-7 | REQ-PM-007 | `SettingsViewState::default()` | `view.plugins` 접근 | `PluginsState::default()` 와 동일 (빈 filter), 기존 9 state 필드 동작 무변경 (test_settings_view_state_default 무변경 GREEN) | unit test |

---

## 6. File Layout

### 6.1 신규

- `crates/moai-studio-ui/src/settings/panes/plugins.rs` — `PluginsPane` struct + `PluginInfo` struct + 6 bundled entries + 단위 테스트 ~10개.

### 6.2 수정

- `crates/moai-studio-ui/src/settings/settings_state.rs`:
  - `SettingsSection` enum 에 `Plugins` variant 추가 (Advanced 다음, 즉 11번째).
  - `SettingsSection::all()` 의 시그니처를 `[Self; 10]` → `[Self; 11]` 로 확장 + array 에 `Plugins` 추가.
  - `SettingsSection::label()` 에 `Plugins => "Plugins"` 추가.
  - `PluginsState` struct 신규 (`plugin_filter: String`) + `Default` impl + `filtered_plugins(...)` helper.
  - `SettingsViewState` 에 `plugins: PluginsState` 필드 추가 + `Default` 갱신.
- `crates/moai-studio-ui/src/settings/panes/mod.rs`:
  - `pub mod plugins;` 추가.
  - `pub use plugins::PluginsPane;` 추가.
- `crates/moai-studio-ui/src/settings/settings_modal.rs`:
  - `use` 라인에 `PluginsPane` 추가.
  - `is_plugins_section()` accessor 추가 (HooksPane 패턴 mirror).
  - `selected_section_label()` (또는 동등 함수) 의 match 에 `SettingsSection::Plugins => PluginsPane::title()` 추가.
  - `sections()` accessor 의 반환 타입 `[SettingsSection; 10]` → `[SettingsSection; 11]` 갱신.

### 6.3 변경 금지 (FROZEN)

- `crates/moai-studio-terminal/**` 전체.
- `crates/moai-studio-workspace/**` 전체.
- `SettingsSection` 의 기존 10 variant 의 ordinal / discriminant / label.
- `SettingsViewState` 의 기존 9 필드.
- `panes/{appearance,keyboard,editor,terminal,agent,hooks,mcp,skills,rules,advanced}.rs` 의 공개 API.
- 기존 settings_modal.rs 의 selected_section / sidebar 렌더 동작.

---

## 7. Test Strategy

- 단위 테스트 minimum **10개** 신규 (plugins.rs ~8 + settings_state.rs ~2):
  - PluginsPane title / description / known_plugins (count + uniqueness)
  - 빈 filter / case-insensitive match / no-match / clear_filter
  - SettingsSection::all() 길이 11 + index 10 = Plugins
  - SettingsSection::Plugins.label() == "Plugins"
- 회귀 0: 기존 settings 관련 테스트 (settings_state.rs / settings_modal.rs / panes/*) 전원 GREEN.
- 검증: `cargo test -p moai-studio-ui --lib --no-fail-fast` + `cargo clippy -p moai-studio-ui --lib --no-deps -- -D warnings` + `cargo fmt -p moai-studio-ui -- --check`.
- 시연: `cargo run -p moai-studio-app` → ⌘, → Settings Modal → sidebar 11번째 항목 "Plugins" 가시 (현재는 selected 시 main pane 빈 placeholder, render mount 는 MS-2 carry).

---

## 8. 6 Bundled Plugin Info (canonical seed list)

본 SPEC MS-1 의 `PluginsPane::known_plugins()` 가 반환하는 정렬된 6 entry:

| # | name | source | version | description |
|---|------|--------|---------|-------------|
| 1 | moai-adk | local-bundled | 0.1.2 | MoAI Agentic Development Kit — full SPEC workflow + DDD/TDD agents. |
| 2 | claude-code-skills | claude-code-marketplace | 1.0.0 | Official Claude Code skills bundle (update-config, simplify, fewer-permission-prompts, ...). |
| 3 | mermaid-diagrams | local-bundled | 0.3.0 | Mermaid diagram rendering for markdown viewer. |
| 4 | git-co-author | claude-code-marketplace | 0.2.1 | Auto-suggest co-author trailer in git commit messages. |
| 5 | nextra-docs | local-bundled | 0.4.0 | Nextra-style documentation site generator. |
| 6 | shadcn-ui-helper | claude-code-marketplace | 0.5.0 | shadcn/ui component scaffolding helper. |

순서는 MS-1 시점의 canonical seed — 향후 marketplace fetch 가 도입되면 동적 정렬로 대체. 본 SPEC 단계는 hardcoded constant.

---

## 9. 의존성 및 제약

- `crates/moai-studio-ui` (SPEC-V3-013 MS-4a/4b/4c/4d 완료) — Hooks/MCP/Skills/Rules pane 패턴 mirror.
- 외부 dependency 0 추가 — 기존 std + serde + design::tokens 만 사용.
- 신규 schema 0 — `PluginsState` 는 in-memory only, settings.json persistence 비대상.
- HARD: §6.3 FROZEN 항목 무변경 — 기존 10 variant 의 discriminant 가 바뀌면 settings.json 의 selected_section 호환성이 깨질 수 있음. 따라서 `Plugins` 는 enum 끝 (Advanced 다음 = 11번째) 에 추가 필수.

---

## 10. DoD

`cargo run -p moai-studio-app` → ⌘, → Settings Modal → sidebar 11번째 "Plugins" 가시 (selected main pane render mount 는 MS-2 carry). 단위 테스트 10개 GREEN + clippy/fmt clean.

---

Version: 1.0.0 (lightweight) | Source: SPEC-V0-2-0-PLUGIN-MGR-001 | 2026-05-04
