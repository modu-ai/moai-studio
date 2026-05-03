---
id: SPEC-V0-2-0-OSC8-LIFECYCLE-001
version: 1.0.0
status: draft
created_at: 2026-05-04
updated_at: 2026-05-04
author: MoAI (main session)
priority: Medium
issue_number: 0
depends_on: [SPEC-V3-LINK-001]
milestones: [MS-1]
language: ko
labels: [v0.2.0, terminal, link, osc8, audit-top-8, lightweight]
revision: v1.0.0 (lightweight) — Lightweight SPEC fast-track per .claude/rules/moai/workflow/spec-workflow.md §Plan Phase
---

# SPEC-V0-2-0-OSC8-LIFECYCLE-001: OSC 8 hyperlink lifecycle — VisitedLinkRegistry + CopyUrl ClickAction

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-05-04 | 초안. v0.2.0 cycle Sprint 11 (audit Top 8 #7 ⭐⭐⭐). audit feature-audit.md §3 Tier B line 53 / §4 #7 의 "B-1 OSC 8 lifecycle: visited state, copy URL, hover params" 의 logic-only 부분 해소. SPEC-V3-LINK-001 의 link.rs 확장 — VisitedLinkRegistry 신규 + ClickAction::CopyUrl variant + resolve_click_for_copy helper. GPUI 우클릭 메뉴 wire 는 별 PR carry. Lightweight SPEC fast-track 적용. |

---

## 1. 목적

SPEC-V3-LINK-001 의 link.rs 가 link detection + click resolution 까지 끝낸 상태에서, OSC 8 (그리고 일반 URL) hyperlink 의 lifecycle 추적 logic 을 추가한다. 구체적으로:
- **Visited state tracking**: 사용자가 클릭한 URL 을 기억해 후속 render 에서 다른 색으로 표시 (별 PR GPUI wire) 가능하도록.
- **Copy URL action**: 사용자가 우클릭 또는 modifier+click 시 URL 만 클립보드에 복사 (브라우저 열지 않음) 가능하도록 ClickAction 확장.
- **Helper**: `resolve_click_for_copy` — 동일 byte_offset 에서 OpenUrl 대신 CopyUrl variant 반환.

본 SPEC scope 는 **logic only** — 클립보드 호출, GPUI 우클릭 메뉴, visited state 의 색상 렌더는 모두 carry. link.rs 에 VisitedLinkRegistry struct + ClickAction::CopyUrl variant + 1 helper 만 추가.

audit feature-audit.md §4 line 304~306: "B-1 OSC 8 click full lifecycle ⭐⭐⭐ — Why: B-2/B-3 와 함께 USP 완성. visited state, copy URL, hover params. Scope: terminal/mod.rs 확장. 추정 ~300 LOC."

**Lightweight SPEC fast-track** 적격성:
- spec.md ≤ 10 KB ✅
- AC 7 (≤ 8) ✅
- milestones 1 (≤ 2) ✅
- no architectural impact (기존 link.rs 확장 + 1 enum variant 추가 + 1 신규 struct, 외부 dep 0) ✅
- 단일 PR (~250 LOC) ✅

---

## 2. 목표 (Goals)

- G1. 신규 `VisitedLinkRegistry` struct — `HashSet<String>` 보유, `mark_visited / is_visited / clear / count` API.
- G2. `ClickAction` enum 에 `CopyUrl(OpenUrl)` variant 추가 (R3 새 variant 만, 기존 3 variant 동작 무변경).
- G3. 신규 helper `resolve_click_for_copy(text, byte_offset) -> Option<ClickAction>` — URL/OSC 8 span 에 대해 `CopyUrl` 반환, 다른 kind 는 None.
- G4. 신규 helper `resolve_click_for_copy_from_spans(spans, byte_offset) -> Option<ClickAction>` — 위 함수의 spans-injected variant.
- G5. SPEC-V3-LINK-001 의 기존 `resolve_click` / `resolve_click_from_spans` 동작 무변경 (회귀 0).
- G6. moai-studio-ui crate 의 기존 terminal click 핸들러 (terminal/mod.rs:336~341) 무변경.

---

## 3. Non-Goals / Exclusions

- N1. **GPUI 우클릭 메뉴 wire** — TerminalSurface 우클릭 → CopyUrl dispatch — 별 PR.
- N2. **클립보드 실 복사** — `arboard::Clipboard::set_text` 호출 — 별 PR (또는 별 SPEC-V0-2-0-OSC8-CLIPBOARD-001).
- N3. **Visited state 의 GPUI 색상 렌더** — link span 의 visited 여부에 따라 다른 색 렌더 — 별 PR.
- N4. **Hover params 표시** — OSC 8 의 `id=...:foo=bar:baz=qux` parameter tooltip — 별 SPEC.
- N5. **Visited state persistence** — disk save / load — 별 SPEC.
- N6. **Visited state TTL / expiration** — 시간 기반 자동 만료 — 별 SPEC.
- N7. **Open vs Copy modifier 결정** (Cmd+Click vs 우클릭) — UX 결정은 별 PR. 본 SPEC 은 helper 만 노출.

---

## 4. Requirements (EARS)

- **REQ-OL-001**: `VisitedLinkRegistry::default()` 가 빈 set 으로 초기화한다. `count() == 0`, `is_visited(_) == false`.
- **REQ-OL-002**: `VisitedLinkRegistry::mark_visited(url)` 가 url 을 set 에 추가한다. 동일 url 두 번 mark 시 idempotent (count 증가 1회만).
- **REQ-OL-003**: `VisitedLinkRegistry::is_visited(url)` 가 mark 된 url 에 대해 true, 아니면 false 반환.
- **REQ-OL-004**: `VisitedLinkRegistry::clear()` 가 모든 entry 제거. count == 0.
- **REQ-OL-005**: `ClickAction` 에 `CopyUrl(OpenUrl)` variant 추가. 기존 `OpenCodeViewer / OpenUrl / OpenSpec` variant 의 discriminant 위치는 유지 (CopyUrl 은 enum 끝에 추가).
- **REQ-OL-006**: `resolve_click_for_copy(text, byte_offset)` 가 (a) URL/Osc8 span 에 대해 `Some(ClickAction::CopyUrl(OpenUrl { url }))` 반환, (b) FilePath/SpecId span 또는 span 없는 위치는 `None` 반환.
- **REQ-OL-007**: `resolve_click_for_copy_from_spans(spans, byte_offset)` 가 위 함수의 pre-computed spans 변형. 동일 동작.

---

## 5. Acceptance Criteria

| AC ID | Requirement | Given | When | Then | 검증 수단 |
|-------|-------------|-------|------|------|-----------|
| AC-OL-1 | REQ-OL-001 | `VisitedLinkRegistry::default()` 인스턴스 | 직후 검사 | count==0, is_visited("x")==false | unit test |
| AC-OL-2 | REQ-OL-002 | empty registry | mark_visited("https://a") 두 번 | count==1 (idempotent) | unit test |
| AC-OL-3 | REQ-OL-003 | mark_visited("a", "b") 적용 후 | is_visited("a") / is_visited("c") | true / false | unit test |
| AC-OL-4 | REQ-OL-004 | 3 entries 보유 registry | clear() | count==0, 모든 is_visited==false | unit test |
| AC-OL-5 | REQ-OL-005 | `ClickAction::CopyUrl(OpenUrl { ... })` 인스턴스 | match exhaustive | CopyUrl arm 매치 + payload 접근. 기존 3 variant 모두 보존 | unit test |
| AC-OL-6 | REQ-OL-006 | URL "https://example.com" 포함 텍스트 | resolve_click_for_copy(text, byte_in_url) | Some(CopyUrl(OpenUrl { url == "https://example.com" })) | unit test |
| AC-OL-7 | REQ-OL-006 | FilePath "src/main.rs" 포함 텍스트 | resolve_click_for_copy(text, byte_in_path) | None (FilePath span 은 copy 대상 아님) | unit test |

---

## 6. File Layout

### 6.1 수정

- `crates/moai-studio-terminal/src/link.rs`:
  - `VisitedLinkRegistry` struct 신규 + `Default` impl + 4 method (mark_visited / is_visited / clear / count)
  - `ClickAction` enum 에 `CopyUrl(OpenUrl)` variant 추가 (enum 끝에)
  - `resolve_click_for_copy(text, byte_offset)` 신규 helper
  - `resolve_click_for_copy_from_spans(spans, byte_offset)` 신규 helper
  - `use std::collections::HashSet;` 추가
  - 단위 테스트 ~10개 추가

### 6.2 변경 금지 (FROZEN)

- `crates/moai-studio-terminal/src/link.rs` 의 기존 ClickAction match arms (3 variant 동작) — REQ-OL-005 invariant
- `crates/moai-studio-terminal/src/link.rs` 의 `resolve_click` / `resolve_click_from_spans` 시그니처 + 본문
- `crates/moai-studio-ui/src/terminal/mod.rs` 의 기존 click handler — terminal click → ClickAction::OpenUrl 처리는 무변경 (CopyUrl 은 별 PR 에서 우클릭 wire 추가)
- `crates/moai-studio-workspace/**`, `crates/moai-studio-agent/**`, `crates/moai-studio-ui/**` 전체 (toolbar / lib.rs 등 무관)

---

## 7. Test Strategy

- 단위 테스트 minimum **10개** (link.rs 의 #[cfg(test)] mod tests):
  - VisitedLinkRegistry default empty / mark idempotent / is_visited true/false / clear (4 tests, AC-OL-1~4)
  - ClickAction::CopyUrl variant exhaustive match (AC-OL-5)
  - 기존 3 variant (OpenCodeViewer / OpenUrl / OpenSpec) 보존 (AC-OL-5 mirror)
  - resolve_click_for_copy URL → CopyUrl (AC-OL-6)
  - resolve_click_for_copy Osc8 → CopyUrl
  - resolve_click_for_copy FilePath → None (AC-OL-7)
  - resolve_click_for_copy SpecId → None
  - resolve_click_for_copy_from_spans pre-computed variant (REQ-OL-007)
- 회귀: 기존 link.rs tests 전원 GREEN, ui crate 전체 1289 tests GREEN.

---

## 8. DoD

본 SPEC PASS 시점에 외부 caller 가:
1. `VisitedLinkRegistry::default()` → `mark_visited(url)` → `is_visited(url) == true` 사이클 사용 가능
2. `resolve_click_for_copy(text, offset)` 호출 시 URL/Osc8 span 에 대해 `CopyUrl(OpenUrl { ... })` 반환 받아 별 PR 의 우클릭 wire 가 클립보드 복사 dispatch 가능

GPUI 우클릭 메뉴 / 클립보드 실 복사 / visited 색상 렌더는 모두 carry — 본 SPEC 은 logic API 만.

audit Top 8 #7 B-1 진척: PARTIAL (open URL 까지) → 60% (visited tracking + copy logic 완료, GPUI wire carry).

---

Version: 1.0.0 (lightweight) | Source: SPEC-V0-2-0-OSC8-LIFECYCLE-001 | 2026-05-04
