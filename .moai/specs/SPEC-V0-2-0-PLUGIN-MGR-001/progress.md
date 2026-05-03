# SPEC-V0-2-0-PLUGIN-MGR-001 Progress

**Started**: 2026-05-04
**Branch**: feature/SPEC-V0-2-0-PLUGIN-MGR-001
**SPEC status**: draft → MS-1 in-progress
**Classification**: Lightweight SPEC fast-track (spec.md ≤10KB, 1 MS, 7 AC, no architectural impact)

## MS-1 (2026-05-04 sess 12) — Plugin Manager UI skeleton (audit Top 8 #3)

### Implementation (planned)

- `crates/moai-studio-ui/src/settings/panes/plugins.rs` (신규):
  - `PluginInfo` struct (name + source + version + description)
  - `PluginsPane` struct (state-bearing, HooksPane mirror)
  - `known_plugins() -> &'static [PluginInfo]` — 6 bundled entries
  - `visible_plugins / visible_count / set_plugin_filter / clear_plugin_filter / plugin_filter` API
  - title/description metadata
  - 단위 테스트 ~8개 (AC-PM-3~6 커버)
- `crates/moai-studio-ui/src/settings/settings_state.rs`:
  - `SettingsSection` enum 에 `Plugins` variant 추가 (11번째, Advanced 다음)
  - `all()` 시그니처 `[Self; 10]` → `[Self; 11]`
  - `label()` 에 `Plugins => "Plugins"` 추가
  - `PluginsState` struct + `Default` impl + `filtered_plugins(...)` helper
  - `SettingsViewState` 에 `plugins: PluginsState` 필드 추가
  - 단위 테스트 ~2개 (AC-PM-1/2/7 커버)
- `crates/moai-studio-ui/src/settings/panes/mod.rs`:
  - `pub mod plugins;` + `pub use plugins::PluginsPane;`
- `crates/moai-studio-ui/src/settings/settings_modal.rs`:
  - `use` 라인 + `is_plugins_section()` accessor + `sections()` 반환 타입 갱신 + selected_section_label 분기 갱신

### Acceptance Criteria

| AC | 내용 | 상태 |
|----|------|------|
| AC-PM-1 | SettingsSection::all() 길이 11, index 10 = Plugins | pending |
| AC-PM-2 | SettingsSection::Plugins.label() == "Plugins" | pending |
| AC-PM-3 | known_plugins() 6 entries, all unique name, non-empty source/description | pending |
| AC-PM-4 | empty filter → visible_count == 6 | pending |
| AC-PM-5 | filter "git" → case-insensitive name/desc match 결과 부분집합 | pending |
| AC-PM-6 | filter "ZZZNonexistent" → visible_count == 0 | pending |
| AC-PM-7 | SettingsViewState::default() 의 plugins 필드 = PluginsState::default() | pending |

### Test count (planned)

- 신규: ~10 (plugins.rs 8 + settings_state.rs 2)
- moai-studio-ui lib tests: 1231 → ~1241 (+10)
- 회귀 0 (기존 settings 테스트 무변경 GREEN)

### Carry (별 SPEC 또는 별 PR)

- MS-2: PluginsPane GPUI render side mount (settings_modal main pane).
- Plugin install/uninstall/enable toggle dispatch — 별 SPEC.
- Plugin marketplace HTTP fetch — 별 SPEC.
