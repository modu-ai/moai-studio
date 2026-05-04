# SPEC-V0-3-0-PALETTE-POLISH-001 — Command Palette polish to 60+ entries

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-3-0-PALETTE-POLISH-001 |
| **Title** | CommandRegistry 확장 60+ entries (recent files / plugin actions / layout / help / spec) |
| **Status** | merged (PR #102, 2026-05-04 main 9dd0c69) — MS-1 100% (44→69 entries / 11→15 categories). fuzzy weight + GPUI category render carry. |
| **Priority** | Low (P3 polish) |
| **Revision** | 1.0 (lightweight) |
| **Dependencies** | SPEC-V3-012 MS-4 (CommandRegistry baseline) |
| **Cycle** | v0.3.0 (audit Top 16 #12 / F-1) |
| **Milestones** | MS-1 |

## HISTORY

- 2026-05-04: 초안 작성. v0.3.0 cycle Sprint 1 #1 (마지막) 진입. audit feature-audit.md §4 #12 F-1 carry. Lightweight 한도 (≤10KB / ≤8 AC / 1 MS) 충족.

## 1. Purpose

`palette/registry.rs` 의 `CommandRegistry::default_registry()` 가 v0.2.0 시점 44 entries / 11 categories 로 정체되어 있다. audit Top 16 #12 F-1 은 "60+ commands" 를 polish 목표로 지정 — recent files / plugin actions / layout / help / spec 카테고리를 신설하고 25 entries 를 추가하여 60+ 도달. fuzzy 매칭 weight 조정 / category 그룹화 GPUI render 측 변경은 차후 SPEC carry.

## 2. Goals

- CommandRegistry default entries 60+ 도달 (44 → 69)
- 4 신규 카테고리: `Plugin` / `Layout` / `Help` / `Spec`
- `Plugin` 카테고리: 5 actions (list / refresh / install / disable / enable)
- `Layout` 카테고리: 4 actions (center / zoom in / zoom out / reset zoom)
- `Help` 카테고리: 3 actions (open docs / report issue / shortcuts)
- `Spec` 카테고리: 3 actions (open panel / new spec / refresh)
- `File` 확장: recent files 5 slot + duplicate + rename (7 entries 추가)
- `Workspace` 확장: recent / add existing / show in finder (3 entries 추가)
- 기존 entries / ids / categories 모두 frozen (R5 — 기존 호환성 보존)
- TRUST 5 gates ALL PASS

## 3. Non-Goals / Exclusions

- fuzzy.rs 매칭 weight 조정 (차후 SPEC carry)
- Command Palette GPUI render 측 category 그룹화 헤더 / divider (차후 SPEC carry)
- recent files 의 실제 데이터 소스 wiring (현재는 placeholder labels)
- 실제 dispatch 로직 변경 (registry 만 확장; dispatch_command 의 namespace stub 은 그대로)

## 4. Requirements

- REQ-PP-001: `default_entries()` 는 ≥ 60 entries 를 반환한다.
- REQ-PP-002: `CATEGORIES` const 는 4 신규 ("Plugin", "Layout", "Help", "Spec") 를 포함한다.
- REQ-PP-003: 5 file recent slot — id "file.recent_1" ~ "file.recent_5", category "File", label "Recent File 1" ~ "Recent File 5".
- REQ-PP-004: Plugin 카테고리 5 entries — `plugin.list` / `plugin.refresh` / `plugin.install` / `plugin.disable` / `plugin.enable`.
- REQ-PP-005: Layout 카테고리 4 entries — `layout.center` / `layout.zoom_in` / `layout.zoom_out` / `layout.reset_zoom`.
- REQ-PP-006: Help 카테고리 3 entries — `help.open_docs` / `help.report_issue` / `help.shortcuts`.
- REQ-PP-007: Spec 카테고리 3 entries — `spec.open_panel` / `spec.new_spec` / `spec.refresh`.
- REQ-PP-008: 기존 44 entries 의 id / category / label 무수정 (R5 — 호환성).

## 5. Acceptance Criteria

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-PP-1 | default_registry | `entries.len()` | `>= 60` | unit test |
| AC-PP-2 | default_registry | `categories.contains` | "Plugin"/"Layout"/"Help"/"Spec" 모두 포함 | unit test |
| AC-PP-3 | default_registry | recent files | id "file.recent_1".."file.recent_5" 5 entries 존재, category "File" | unit test |
| AC-PP-4 | default_registry | Plugin entries | 5 entries (list/refresh/install/disable/enable), 모두 category "Plugin" | unit test |
| AC-PP-5 | default_registry | Layout entries | 4 entries (center/zoom_in/zoom_out/reset_zoom), 모두 category "Layout" | unit test |
| AC-PP-6 | default_registry | Help/Spec entries | Help 3 + Spec 3, 모두 category 일치 | unit test |
| AC-PP-7 | default_registry | 모든 entries | id 모두 unique + namespaced (`.` 포함) | 기존 tests 통과 |
| AC-PP-8 | cargo build/clippy/fmt + ui tests | run | ALL PASS, 회귀 0 | CI |

## 6. File Layout

| Path | Status | Note |
|------|--------|------|
| `crates/moai-studio-ui/src/palette/registry.rs` | modified | default_entries +25, CATEGORIES +4, tests 갱신 |
| `.moai/specs/SPEC-V0-3-0-PALETTE-POLISH-001/spec.md` | created | 본 문서 |
| `.moai/specs/SPEC-V0-3-0-PALETTE-POLISH-001/progress.md` | created | 구현 후 갱신 |

FROZEN:
- 기존 44 entries 의 id / category / label
- `CommandRegistry` struct 시그니처 / `default_registry()` 시그니처
- `CommandEntry::new` 시그니처

## 7. Test Strategy

기존 registry tests (현재 ~14) 의 `>= 30` assertion 을 `>= 60` 으로 갱신. 신규 카테고리 / 신규 entries 검증 6 추가 tests. 회귀 0.

---

Version: 1.0.0 (lightweight)
Created: 2026-05-04
Cycle: v0.3.0 Sprint 1 #1 (last task)
Carry-to: fuzzy 가중치 조정 / GPUI category header render — 차후 SPEC
