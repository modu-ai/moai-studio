# SPEC-V0-2-0-OSC8-LIFECYCLE-001 Progress

**Started**: 2026-05-04
**Branch**: feature/SPEC-V0-2-0-OSC8-LIFECYCLE-001
**SPEC status**: implemented (MS-1 complete)
**Completion date**: 2026-05-04
**Predecessor**: SPEC-V3-LINK-001 (Smart Link Detection)
**audit reference**: feature-audit.md §3 Tier B line 53 (B-1) + §4 Top 8 #7 (⭐⭐⭐)
**Classification**: Lightweight SPEC fast-track (spec.md 8627 bytes ≤10KB, 1 MS, 7 REQ / 7 AC, no architectural impact)

## MS-1 (2026-05-04 sess 12+) — VisitedLinkRegistry + CopyUrl ClickAction ✅

### Implementation

- `crates/moai-studio-terminal/src/link.rs`:
  - VisitedLinkRegistry struct + Default + mark_visited / is_visited / clear / count
  - ClickAction enum 에 CopyUrl(OpenUrl) variant 추가 (enum 끝)
  - resolve_click_for_copy / resolve_click_for_copy_from_spans helpers
  - use std::collections::HashSet
  - 단위 테스트 ~10개

### Acceptance Criteria

| AC | 내용 | 상태 |
|----|------|------|
| AC-OL-1 | default registry empty | pending |
| AC-OL-2 | mark_visited idempotent | pending |
| AC-OL-3 | is_visited true/false | pending |
| AC-OL-4 | clear() empties | pending |
| AC-OL-5 | ClickAction::CopyUrl variant + 기존 3 보존 | pending |
| AC-OL-6 | resolve_click_for_copy URL → CopyUrl | pending |
| AC-OL-7 | resolve_click_for_copy FilePath → None | ✅ |

### Implementation summary

- VisitedLinkRegistry (HashSet<String>) + Default + new / mark_visited (idempotent) / is_visited / clear / count / is_empty
- ClickAction::CopyUrl(OpenUrl) variant 추가 (enum 끝, 기존 3 variant 무변경)
- resolve_click_for_copy + resolve_click_for_copy_from_spans helpers
- terminal/mod.rs match 에 CopyUrl arm 추가 (현재는 log only — clipboard wire carry per N1/N2)

### Test count

- 신규 11 (link.rs T-OL 블록):
  - visited_registry_default_is_empty (AC-OL-1)
  - visited_registry_mark_is_idempotent (AC-OL-2)
  - visited_registry_is_visited_distinguishes_entries (AC-OL-3)
  - visited_registry_clear_empties_set (AC-OL-4)
  - click_action_copy_url_variant_matches_exhaustively (AC-OL-5)
  - resolve_click_for_copy_url_returns_copy_url (AC-OL-6)
  - resolve_click_for_copy_osc8_returns_copy_url
  - resolve_click_for_copy_filepath_returns_none (AC-OL-7)
  - resolve_click_for_copy_spec_id_returns_none
  - resolve_click_for_copy_no_span_returns_none
  - resolve_click_for_copy_from_spans_matches_auto_variant (REQ-OL-007)
- moai-studio-terminal: 36 → 47 (+11)
- moai-studio-ui: 1289 GREEN (회귀 0)
- agent 129, workspace 26 GREEN
- clippy 0 warning, fmt clean

### Carry to next PR

- TerminalSurface 우클릭 → CopyUrl dispatch (terminal/mod.rs)
- arboard::Clipboard::set_text 실 복사
- visited state 색상 렌더 (link span color override)
- Hover params tooltip
- Visited state persistence / TTL
