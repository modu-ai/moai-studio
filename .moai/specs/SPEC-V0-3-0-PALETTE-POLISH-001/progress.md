# SPEC-V0-3-0-PALETTE-POLISH-001 — Progress

## Status

- 2026-05-04: 초안 작성 + MS-1 구현 완료. 8 AC ALL ✅. ui crate 1326 → 1332 (+6). clippy/fmt clean.

## Milestone Tracker

### MS-1 — Registry expansion to 60+ entries (✅ DONE)

| AC | Status | Note |
|----|--------|------|
| AC-PP-1 | ✅ | entries.len() = 69 (>= 60) |
| AC-PP-2 | ✅ | 4 신규 카테고리 (Plugin/Layout/Help/Spec) 모두 포함 |
| AC-PP-3 | ✅ | file.recent_1~5 + file.duplicate + file.rename (7 추가) |
| AC-PP-4 | ✅ | Plugin 5 entries (list/refresh/install/disable/enable) |
| AC-PP-5 | ✅ | Layout 4 entries (center/zoom_in/zoom_out/reset_zoom) |
| AC-PP-6 | ✅ | Help 3 + Spec 3 entries |
| AC-PP-7 | ✅ | 모든 ids unique + namespaced (기존 tests 통과) |
| AC-PP-8 | ✅ | cargo build/clippy/fmt + 1332 ui tests PASS, 회귀 0 |

### Implementation summary

- `palette/registry.rs`:
  - `CATEGORIES` const: 11 → 15 (+ Plugin / Layout / Help / Spec)
  - `default_entries()` : 44 → **69 entries** (+25)
    - File 5 → 12 (+7: 5 recent slots + duplicate + rename)
    - Workspace 5 → 8 (+3: recent / add_existing / show_in_finder)
    - Plugin 0 → 5 (신규)
    - Layout 0 → 4 (신규)
    - Help 0 → 3 (신규)
    - Spec 0 → 3 (신규)
  - 기존 entries 모두 frozen (id/category/label 무수정 — R5 호환성)
- 기존 tests 갱신: `default_registry_has_at_least_30_entries` → `_60_entries`, `all_expected_categories_present` 의 expected 리스트 +4
- 6 신규 tests (T-PP):
  - registry_exposes_five_file_recent_slots
  - registry_exposes_five_plugin_entries
  - registry_exposes_four_layout_entries
  - registry_exposes_help_and_spec_entries
  - categories_const_includes_four_new_categories
  - legacy_entries_remain_unchanged

## Carry-Forward (별 SPEC)

- `palette/fuzzy.rs` 매칭 weight 조정 (category 우선순위, recent 부스트) — 차후 SPEC
- Command Palette GPUI render 측 category 그룹화 헤더 / divider — 차후 SPEC
- recent files placeholder labels → 실제 RecentFilesProvider wiring — 차후 SPEC
- 신규 commands 의 dispatch_command 활성화 (현재 surface./pane./plugin./layout./help./spec. namespace 모두 stub) — Surface/Pane SPEC + 신규 namespace SPECs

## Notes

- Lightweight SPEC fast-track 10번째 적용 (이전 9번째 = SPEC-V0-3-0-WORKSPACE-COLOR-001).
- main 세션 직접 fallback 유지.
- v0.3.0 cycle Sprint 1 #1 (마지막 task) — **Sprint 1 100% 종료** (4/4: clippy fix + menu wire + color tags + palette polish).
- audit Top 16 진척: F-1 (Palette polish 60+) GA. fuzzy weight + GPUI category render carry.
