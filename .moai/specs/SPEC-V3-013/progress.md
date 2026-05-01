# SPEC-V3-013 Progress

**Started**: 2026-04-26
**Branch**: main (direct commits via PRs #23, #25, #27)
**SPEC status**: FULLY IMPLEMENTED (all 3 milestones, 12 acceptance criteria)
**Completion date**: 2026-04-26

## Implementation Timeline

- 2026-04-26 PR #23 (`3f73331`): MS-1 SettingsModal + AppearancePane (AC-V13-1~6) — 951 LOC, 54 tests
- 2026-04-26 PR #25 (`71f8170`): MS-2 KeyboardPane + 4 sub-panes (AC-V13-7~9) — 1690 LOC, 90 new tests
- 2026-04-26 PR #27 (`254433e`): MS-3 UserSettings persistence + ActiveTheme + RootView (AC-V13-10~12) — 1142 LOC
- 2026-04-26 `b83fa84`: docs — SPEC-V3-012 + V3-013 status draft → implemented

## Milestone Status

- [x] MS-1: SettingsModal + AppearancePane — PR #23 (AC-V13-1~6)
- [x] MS-2: KeyboardPane + 4 sub-panes — PR #25 (AC-V13-7~9)
- [x] MS-3: UserSettings persistence + ActiveTheme + RootView integration — PR #27 (AC-V13-10~12)
- [x] MS-4a: HooksPane skeleton (audit G-1 첫 패널, v0.1.2 Task 9 sub-PR) — AC-V13-13~16
- [x] MS-4b: McpPane skeleton (audit G-1 두 번째 패널, v0.1.2 Task 9b sub-PR) — AC-V13-17~21
- [x] MS-4c: SkillsPane skeleton (audit G-1 세 번째 패널, v0.1.2 Task 9c sub-PR) — AC-V13-22~26
- [x] MS-4d: RulesPane skeleton (audit G-1 마지막 패널 — Task 9 4 sub-PR 완성, v0.1.2 Task 9d sub-PR) — AC-V13-27~31

### MS-4a Acceptance Criteria

| AC ID | Given | When | Then |
|-------|-------|------|------|
| AC-V13-13 | (메타데이터) | `HooksPane::title()` / `description()` 호출 | title == "Hooks", description 비어있지 않고 "27" 카탈로그 사이즈를 언급 |
| AC-V13-14 | (카탈로그) | `HooksPane::known_events()` 호출 | 정확히 27 개 hook event 반환, unique, 핵심 이벤트 (PreToolUse, PostToolUse, Stop, UserPromptSubmit, WorktreeCreate, WorktreeRemove) 포함 |
| AC-V13-15 | HooksPane(state.event_filter == "worktree") | `visible_events()` 호출 | filter 가 case-insensitive substring 매치, WorktreeCreate + WorktreeRemove 2 건 반환. 매치 없는 filter 는 빈 Vec |
| AC-V13-16 | event_filter 가 비어있지 않은 상태 | `clear_event_filter()` 호출 | event_filter 는 빈 문자열, visible_count == 27 (전체 카탈로그 복귀) |

#### MS-4a Frozen-zone (REQ-V13-MS4a-1)

- moai-studio-terminal/** 무변경
- moai-studio-workspace/** 무변경
- 기존 6 SettingsSection variant 의 동작/시그니처 무변경 (Hooks variant 추가, sections() return type 6 → 7, label/active_section_title match arm 만 확장)
- UserSettings 의 영속화 schema 무변경 (HooksState 는 in-memory only, MS-3 schema v1 carry)

### MS-4b Acceptance Criteria

| AC ID | Given | When | Then |
|-------|-------|------|------|
| AC-V13-17 | (메타데이터) | `McpPane::title()` / `description()` 호출 | title == "MCP", description 비어있지 않고 "MCP" 또는 "Model Context Protocol" 을 언급 |
| AC-V13-18 | 빈 server 목록 | `total_count()` / `visible_count()` / `set_servers([3개])` 호출 | 빈 상태에서 0/0, 주입 후 total=3, visible=3 (filter 빈) |
| AC-V13-19 | 3 sample servers (context7/playwright/github) | `set_server_filter("CONTEXT")` / `"npx"` / `"nonexistent"` 호출 | "CONTEXT" → context7 1건 (case-insensitive), "npx" → 2건 (command match), "nonexistent" → 0건 |
| AC-V13-20 | filter 가 활성 상태 (visible 1) | `clear_server_filter()` 호출 | filter == "" + visible_count == 3 (전체 복귀) |
| AC-V13-21 | enabled=2 + disabled=1 server 주입 | `visible_servers()` 검사 | enabled 와 disabled 모두 노출 (read-only viewer 의미) |

#### MS-4b Frozen-zone (REQ-V13-MS4b-1)

- moai-studio-terminal/** 무변경
- moai-studio-workspace/** 무변경
- 기존 7 SettingsSection variant (MS-4a 후) 의 동작/시그니처 무변경 (Mcp variant 추가, sections() return type 7 → 8, label/active_section_title match arm 만 확장)
- UserSettings 의 영속화 schema 무변경 (McpPaneState 는 in-memory only, .claude/settings.json 자동 로드 wiring 은 별 SPEC)

### MS-4c Acceptance Criteria

| AC ID | Given | When | Then |
|-------|-------|------|------|
| AC-V13-22 | (메타데이터) | `SkillsPane::title()` / `description()` 호출 | title == "Skills", description 비어있지 않고 "Skills" 키워드 포함 |
| AC-V13-23 | 빈 skill 목록 | `total_count()` / `set_skills([3개])` 호출 | 빈 상태 0/0, 주입 후 total=3 + visible=3 (filter 빈) |
| AC-V13-24 | sample skills 3개 (foundation/tdd/frontend) | `set_skill_filter("FOUNDATION")` / `"test-driven"` / `"nonexistent"` 호출 | "FOUNDATION" → 1건 (case-insensitive name), "test-driven" → 1건 (description match), "nonexistent" → 0건 |
| AC-V13-25 | filter 가 활성 (visible 1) | `clear_skill_filter()` 호출 | filter == "" + visible_count == 3 (전체 복귀) |
| AC-V13-26 | enabled=2 + disabled=1 skill 주입 | `visible_skills()` 검사 | enabled 와 disabled 모두 노출 (read-only viewer) |

#### MS-4c Frozen-zone (REQ-V13-MS4c-1)

- moai-studio-terminal/** 무변경
- moai-studio-workspace/** 무변경
- 기존 8 SettingsSection variant (MS-4b 후) 의 동작/시그니처 무변경 (Skills variant 추가, sections() return type 8 → 9, label/active_section_title match arm 만 확장)
- UserSettings 의 영속화 schema 무변경 (SkillsPaneState 는 in-memory only, ~/.claude/skills/ 자동 스캔 wiring 은 별 SPEC)

### MS-4d Acceptance Criteria

| AC ID | Given | When | Then |
|-------|-------|------|------|
| AC-V13-27 | (메타데이터) | `RulesPane::title()` / `description()` 호출 | title == "Rules", description 비어있지 않고 "Rules" 키워드 포함 |
| AC-V13-28 | 빈 rule 목록 | `total_count()` / `set_rules([3개])` 호출 | 빈 상태 0/0, 주입 후 total=3 + visible=3 (filter 빈) |
| AC-V13-29 | sample rules 3개 (constitution/spec-workflow/rust) | `set_rule_filter("CONSTITUTION")` / `"workflow"` / `"nonexistent"` 호출 | "CONSTITUTION" → 1건 (case-insensitive name), "workflow" → 1건 (description match), "nonexistent" → 0건 |
| AC-V13-30 | filter 가 활성 (visible 1) | `clear_rule_filter()` 호출 | filter == "" + visible_count == 3 (전체 복귀) |
| AC-V13-31 | enabled=2 + disabled=1 rule 주입 | `visible_rules()` 검사 | enabled 와 disabled 모두 노출 (read-only viewer) |

#### MS-4d Frozen-zone (REQ-V13-MS4d-1)

- moai-studio-terminal/** 무변경
- moai-studio-workspace/** 무변경
- 기존 9 SettingsSection variant (MS-4c 후) 의 동작/시그니처 무변경 (Rules variant 추가, sections() return type 9 → 10, label/active_section_title match arm 만 확장)
- UserSettings 의 영속화 schema 무변경 (RulesPaneState 는 in-memory only, .claude/rules/ 자동 스캔 wiring 은 별 SPEC)
- audit G-1 의 4 missing panes (Hooks/MCP/Skills/Rules) 모두 skeleton 추가 완료 — Task 9 의 4 sub-PR 완성 (4a~4d)

## Key Files Changed

### MS-1 — SettingsModal + AppearancePane (6 files, 951 LOC)

- `crates/moai-studio-ui/src/settings/settings_modal.rs`: 282 LOC — 880x640 container, 200px sidebar + 680px main pane (AC-V13-1)
- `crates/moai-studio-ui/src/settings/settings_state.rs`: 409 LOC — SettingsViewState mount/dismiss, font_size 12-18 range enforcement (REQ-V13-025)
- `crates/moai-studio-ui/src/settings/panes/appearance.rs`: 236 LOC — theme/density/accent(4)/font_size in-memory state (AC-V13-4~6)
- `crates/moai-studio-ui/src/settings/panes/mod.rs`: Pane module root
- `crates/moai-studio-ui/src/settings/mod.rs`: Settings module root
- `crates/moai-studio-ui/src/lib.rs`: `mod settings;` registration

### MS-2 — KeyboardPane + 4 Sub-panes (10 files, 1690 LOC)

- `crates/moai-studio-ui/src/settings/panes/keyboard.rs`: 339 LOC — 12 default bindings, edit dialog, conflict check (pass/fail cases)
- `crates/moai-studio-ui/src/settings/panes/editor.rs`: 152 LOC — tab_size NumericInput (2-8, default 4) skeleton
- `crates/moai-studio-ui/src/settings/panes/terminal.rs`: 152 LOC — scrollback_lines NumericInput (1000-100000, default 10000) skeleton
- `crates/moai-studio-ui/src/settings/panes/agent.rs`: 132 LOC — auto_approve Toggle (default false) skeleton
- `crates/moai-studio-ui/src/settings/panes/advanced.rs`: 113 LOC — experimental_flags read-only placeholder (default [])
- `crates/moai-studio-ui/src/settings/settings_state.rs`: 580 LOC — KeyBinding + KeyboardState + EditDialogState + 4 sub-pane state structs
- `crates/moai-studio-ui/src/settings/settings_modal.rs`: 188 LOC — 6 section routing (is_*_active, active_section_title)

### MS-3 — Persistence + ActiveTheme + RootView (8 files, 1142 LOC)

- `crates/moai-studio-ui/src/settings/user_settings.rs`: 590 LOC — UserSettings struct + serde JSON persistence (schema v1), atomic write (tempfile + rename), fail-soft load (.bak backup), dirs::config_dir() platform paths + temp_dir fallback. 13 unit tests (roundtrip, schema mismatch, corruption, parent dir auto-creation)
- `crates/moai-studio-ui/src/design/runtime.rs`: 254 LOC — ActiveTheme runtime dispatch wrapper. ThemeMode/AccentColor/Density to tokens constants (FROZEN R-V13-3). 15 unit tests (dark/light bg, 4 accent colors, spacing, font_size, System to Dark)
- `crates/moai-studio-ui/src/lib.rs`: 281 LOC — RootView MS-3 integration: user_settings + active_theme fields, `handle_settings_key_event` (Cmd+, on macOS / Ctrl+, on Linux/Win), `dismiss_settings_modal` (appearance sync + save_atomic + active_theme update), `render_settings_overlay` (scrim + 880x640 container). 8 unit tests (mount, double-press prevention, Esc dismiss, init consistency)
- `Cargo.toml`: `dirs = "5"` workspace dependency added
- `crates/moai-studio-ui/Cargo.toml`: dirs dependency usage
- `crates/moai-studio-ui/src/design/mod.rs`: `mod runtime;` registration
- `crates/moai-studio-ui/src/settings/mod.rs`: Module updates
- `crates/moai-studio-ui/src/settings/settings_state.rs`: ThemeMode/Density/AccentColor Serialize/Deserialize additions

## Test Coverage

- MS-1: 54 new tests
- MS-2: 90 new tests (584 total passing)
- MS-3: 536 tests (moai-studio-ui), entire workspace 0 failures
- Quality gates: clippy 0 warnings, rustfmt PASS, `cargo check --release` PASS on all milestones

## Known Limitations

- Skeleton panes: EditorPane, TerminalPane, AgentPane, AdvancedPane are functional skeletons with basic controls but limited real functionality beyond state management.
- Mock keybindings: KeyboardPane shows 12 default bindings but real keybinding reassignment is UI-only (no actual key remapping to GPUI event handlers).
- In-memory settings changes are persisted, but active runtime theme switching relies on the ActiveTheme wrapper — full visual theme change may require additional GPUI integration.
- No plan.md exists — planning was done inline in docs commit `ad44b83`. This is a known gap in SPEC artifact completeness.
