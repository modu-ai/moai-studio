# SPEC-V0-2-0-PLUGIN-MGR-001 Progress

**Started**: 2026-05-04
**Branch**: feature/SPEC-V0-2-0-PLUGIN-MGR-001
**SPEC status**: implemented (MS-1 complete)
**Completion date**: 2026-05-04
**Classification**: Lightweight SPEC fast-track (spec.md ≤10KB, 1 MS, 7 AC, no architectural impact)

## MS-1 (2026-05-04 sess 12) — Plugin Manager UI skeleton (audit Top 8 #3) ✅

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
| AC-PM-1 | SettingsSection::all() 길이 11, index 10 = Plugins | ✅ |
| AC-PM-2 | SettingsSection::Plugins.label() == "Plugins" | ✅ |
| AC-PM-3 | known_plugins() 6 entries, all unique name, non-empty source/description | ✅ |
| AC-PM-4 | empty filter → visible_count == 6 | ✅ |
| AC-PM-5 | filter "git" → case-insensitive name/desc match 결과 부분집합 | ✅ |
| AC-PM-6 | filter "ZZZNonexistent" → visible_count == 0 | ✅ |
| AC-PM-7 | SettingsViewState::default() 의 plugins 필드 = PluginsState::default() | ✅ |

### Test count

- 신규: 15 (plugins.rs 11 + settings_state.rs 2 갱신/신규 + settings_modal.rs 2)
  - plugins.rs (T-PM 블록): title / description / known_plugins count / unique names / non-empty metadata / empty filter / case-insensitive substring / uppercase match / no-match / clear filter / with_state preserves
  - settings_state.rs: section_all_returns_eleven_in_order (10→11 갱신) / section_labels_are_correct (+ Plugins) / settings_view_state_default_initializes_plugins_state / plugins_state_filtered_plugins_respects_filter
  - settings_modal.rs: sections_returns_eleven (10→11 갱신) / only_one_section_active_at_a_time (+ Plugins 행 추가) / plugins_pane_title_via_modal_routing / plugins_section_selected_routes_title
- moai-studio-ui lib tests: 1231 → 1246 (+15)
- clippy 0 warning, fmt clean, 회귀 0

### Implementation files

신규:
- `crates/moai-studio-ui/src/settings/panes/plugins.rs` (11130 bytes)
  - `PluginInfo` struct + `PluginsPane` struct + `known_plugins()` 6 entries (canonical seed § 8) + 단위 테스트 11개
- `.moai/specs/SPEC-V0-2-0-PLUGIN-MGR-001/spec.md` (9972 bytes — Lightweight 기준 ≤10KB 충족)
- `.moai/specs/SPEC-V0-2-0-PLUGIN-MGR-001/progress.md`

수정:
- `crates/moai-studio-ui/src/settings/settings_state.rs`:
  - SettingsSection::Plugins variant 추가 (11번째)
  - all() [Self;10] → [Self;11]
  - label() match 에 Plugins 추가
  - PluginsState struct + filtered_plugins helper 신규
  - SettingsViewState::plugins 필드 추가 + Default 갱신
- `crates/moai-studio-ui/src/settings/settings_modal.rs`:
  - use 라인에 PluginsPane 추가
  - sections() 반환 타입 [_; 10] → [_; 11]
  - is_plugins_active() accessor 추가
  - active_section_title match 에 Plugins => PluginsPane::title() 추가
- `crates/moai-studio-ui/src/settings/panes/mod.rs`:
  - pub mod plugins; + pub use plugins::PluginsPane;

### Carry (별 SPEC 또는 별 PR)

- MS-2: PluginsPane GPUI render side mount (settings_modal main pane).
- Plugin install/uninstall/enable toggle dispatch — 별 SPEC.
- Plugin marketplace HTTP fetch — 별 SPEC.

### DoD ✅

- `cargo test -p moai-studio-ui --lib --no-fail-fast` → 1246 passed (1231 → +15)
- `cargo clippy -p moai-studio-ui --lib --no-deps -- -D warnings` → 0 warning
- `cargo fmt -p moai-studio-ui -- --check` → clean
- 회귀 0
- spec.md 9972 bytes (< 10 KB), AC 7 (≤ 8), milestones 1 (≤ 2) — Lightweight SPEC 적격성 유지
