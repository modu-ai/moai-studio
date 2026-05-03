---
id: SPEC-V0-2-0-OSC8-LIFECYCLE-001
version: 1.1.0
status: draft
created_at: 2026-05-04
updated_at: 2026-05-04
author: MoAI (main session)
priority: Medium
issue_number: 0
depends_on: [SPEC-V3-LINK-001]
milestones: [MS-1, MS-2]
language: ko
labels: [v0.2.0, terminal, link, osc8, audit-top-8, lightweight]
revision: v1.1.0 (lightweight + MS-2 amendment) — Lightweight SPEC fast-track per .claude/rules/moai/workflow/spec-workflow.md §Plan Phase
---

# SPEC-V0-2-0-OSC8-LIFECYCLE-001: OSC 8 hyperlink lifecycle — VisitedLinkRegistry + CopyUrl ClickAction

## HISTORY

| 버전 | 날짜 | 변경 |
|------|------|------|
| 1.0.0-draft | 2026-05-04 | 초안. v0.2.0 cycle Sprint 11 (audit Top 8 #7 ⭐⭐⭐). audit feature-audit.md §3 Tier B line 53 / §4 #7 의 "B-1 OSC 8 lifecycle: visited state, copy URL, hover params" 의 logic-only 부분 해소. SPEC-V3-LINK-001 의 link.rs 확장 — VisitedLinkRegistry 신규 + ClickAction::CopyUrl variant + resolve_click_for_copy helper. GPUI 우클릭 메뉴 wire 는 별 PR carry. Lightweight SPEC fast-track 적용. |
| 1.1.0-draft | 2026-05-04 | MS-2 amendment 추가. v0.2.0 cycle Sprint 12 (audit Top 8 #7 ⭐⭐⭐ 후속). MS-1 의 logic API 를 GPUI wire 로 연결: TerminalSurface 우클릭 → CopyUrl dispatch + 기존 `copy_to_clipboard` 재사용 + ClipboardWriter trait 추상화 + visited tracking (mark_visited 호출). visited URL span 색상 렌더는 MS-3 (T5/T6 cell-grid render path 의존) carry. Lightweight SPEC fast-track 6번째 적용 (PLUGIN-MGR / TOOLBAR-WIRE / ONBOARDING-ENV / OSC8-LIFECYCLE MS-1 / WIZARD-ENV 이후). |

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

## MS-2 — Right-click dispatch + arboard wire + visited tracking

### MS-2.1 목적

MS-1 이 link.rs 에 추가한 `VisitedLinkRegistry` + `ClickAction::CopyUrl` + `resolve_click_for_copy*` 의 logic API 를 TerminalSurface (moai-studio-ui) 의 GPUI 클릭 경로와 시스템 클립보드(arboard)로 wire 한다. visited URL 의 색상 분기 렌더는 cell-grid render path 가 placeholder 상태이므로 별 SPEC (MS-3) 으로 분리한다.

### MS-2.2 목표 (Goals)

- G2-1. `ClipboardWriter` trait 신규 (`Send + Sync`) — 단일 메서드 `write(&self, &str) -> Result<(), arboard::Error>`. 현재 `terminal/clipboard.rs` 의 free function `copy_to_clipboard` 와 공존 (free function 무변경, trait 은 polymorphic injection 통로).
- G2-2. `ArboardClipboardWriter` struct 신규 (production) — `ClipboardWriter` impl 이 기존 `copy_to_clipboard` 호출. SPEC-V3-002 RG-V3-002-4 의 selection-text copy 동작 무변경 (회귀 0).
- G2-3. `MockClipboardWriter` struct 신규 (test) — `Arc<Mutex<Vec<String>>>` capture, `contents()` accessor 로 호출 history 검증. 모든 호출 `Ok(())`.
- G2-4. `TerminalSurface` 에 두 필드 추가: `visited_links: VisitedLinkRegistry` (default empty) + `clipboard_writer: Box<dyn ClipboardWriter + Send + Sync>` (default `ArboardClipboardWriter`).
- G2-5. `TerminalSurface::new()` 두 필드 default 초기화. 신규 builder helper `with_clipboard_writer(writer)` (테스트 inject 용).
- G2-6. `TerminalSurface::handle_click_for_copy(_row, col, line_text, _cx)` 메서드 신규 — pure logic helper `copy_url_at(line, byte_offset, &mut visited, &dyn ClipboardWriter) -> Option<String>` 로 분리하여 cx 의존 없는 단위 테스트 가능. CopyUrl 시 ClipboardWriter::write + mark_visited.
- G2-7. `TerminalSurface::render()` 의 `on_mouse_down(MouseButton::Left, ...)` 다음에 `on_mouse_down(MouseButton::Right, ...)` 분기 추가 — listener 본문은 `pixel_to_cell` → `handle_click_for_copy` 호출.
- G2-8. 기존 `handle_click` 의 `ClickAction::OpenUrl` arm (mod.rs:336) 에 `self.visited_links.mark_visited(url.clone())` 한 줄 추가. 좌클릭 OpenUrl + 우클릭 CopyUrl 양쪽 모두 visited 기록.

### MS-2.3 Non-Goals (MS-2 scope)

- M2-N1. **visited URL span 색상 렌더** — TerminalSurface::render() 의 cell-grid render path 가 placeholder (cursor_info 단일 텍스트만 출력) 상태. URL span 자체가 색상 렌더되지 않으므로 visited override 추가 무의미. T5/T6 cell-grid render 도착 후 별 SPEC MS-3 carry.
- M2-N2. **Hover params tooltip** — OSC 8 parameter 표시 — 별 SPEC.
- M2-N3. **Visited persistence / TTL** — disk save/load, 시간 기반 expiration — 별 SPEC.
- M2-N4. **OSC 52 원격 클립보드** — `copy_to_clipboard` 의 SPEC-V3-002 §6 exclusion 따름 (parser silently ignore).
- M2-N5. **modifier+click 기반 copy** (Cmd+Click etc) — 본 MS-2 는 우클릭 단일 패턴. modifier 결합 UX 는 별 PR.

### MS-2.4 Requirements (EARS)

- **REQ-OL-008**: `ClipboardWriter` trait 정의 (`crates/moai-studio-ui/src/terminal/clipboard.rs`). bound: `Send + Sync`. method: `fn write(&self, text: &str) -> Result<(), arboard::Error>`.
- **REQ-OL-009**: `ArboardClipboardWriter` struct + `Default` impl + `ClipboardWriter` impl. impl body 는 기존 `copy_to_clipboard(text)` 호출. 기존 `copy_to_clipboard` 함수 시그니처 + 호출 동작 무변경 (FROZEN).
- **REQ-OL-010**: `MockClipboardWriter` struct + `Default` impl + `ClipboardWriter` impl. 내부 `Arc<Mutex<Vec<String>>>` 에 push, `contents() -> Vec<String>` 로 snapshot 반환. 모든 `write` 호출 `Ok(())`. `Send + Sync` 만족.
- **REQ-OL-011**: `TerminalSurface` struct 에 두 필드 추가 — `visited_links: VisitedLinkRegistry` + `clipboard_writer: Box<dyn ClipboardWriter + Send + Sync>`. `new()` default = empty registry + `ArboardClipboardWriter::default()`. 신규 `with_clipboard_writer(writer)` builder (테스트 용, builder pattern 으로 chain 가능 또는 직접 setter).
- **REQ-OL-012**: pure helper `copy_url_at(line_text: &str, byte_offset: usize, visited: &mut VisitedLinkRegistry, writer: &(dyn ClipboardWriter + Send + Sync)) -> Option<String>`. (a) `resolve_click_for_copy(line_text, byte_offset)` 호출. (b) `Some(ClickAction::CopyUrl(OpenUrl { url }))` 시 `writer.write(&url)` 호출 (실패 시 `tracing::warn!` log, panic 금지) + `visited.mark_visited(url.clone())` + `Some(url)` 반환. (c) `None` 또는 다른 variant 시 `None` 반환. cx 의존성 없음 (단위 테스트 가능).
- **REQ-OL-013**: `TerminalSurface::handle_click_for_copy(_row: u16, col: u16, line_text: &str, _cx: &mut Context<Self>)` 메서드. body 는 `byte_offset = col_to_byte_offset(line_text, col as usize)` → `copy_url_at(line_text, byte_offset, &mut self.visited_links, self.clipboard_writer.as_ref())` 호출 → `Some(url)` 시 `tracing::info!(url, "ClickAction::CopyUrl wrote to clipboard")`.
- **REQ-OL-014**: `TerminalSurface::render()` 의 `on_mouse_down` listener 에 `MouseButton::Right` 분기 추가. listener body 는 `pixel_to_cell(x, y)` → `handle_click_for_copy(row, col, &line, cx)` 호출. 기존 `MouseButton::Left` 분기 무변경.
- **REQ-OL-015**: 기존 `TerminalSurface::handle_click` 의 `ClickAction::OpenUrl` arm 본문에 `self.visited_links.mark_visited(url.clone())` 한 줄 추가 (`cx.open_url(&url)` 직후, `cx.emit(...)` 직전). 기존 `OpenCodeViewer / OpenSpec / CopyUrl` arm 무변경.

### MS-2.5 Acceptance Criteria

| AC ID | Requirement | Given | When | Then | 검증 수단 |
|-------|-------------|-------|------|------|-----------|
| AC-OL-8 | REQ-OL-008/010 | `let _: Box<dyn ClipboardWriter + Send + Sync> = Box::new(MockClipboardWriter::default());` | 컴파일 | 성공 (trait + bound 만족) | unit test (`#[test]` 컴파일만 검증) |
| AC-OL-9 | REQ-OL-010 | `MockClipboardWriter::default()` | `write("a")` → `write("b")` 순차 호출 | `contents() == vec!["a", "b"]` (순서 보존) + 두 호출 모두 `Ok(())` | unit test |
| AC-OL-10 | REQ-OL-009 | `ArboardClipboardWriter::default()` 인스턴스화 | trait method 호출 가능성 검사 | trait object 로 box 가능 + `write` 호출 path 가 `copy_to_clipboard` 로 라우팅 (compile-time) | unit test (실제 arboard 호출은 headless CI 환경 의존성으로 인해 검증 skip 가능; 인스턴스화 + trait method dispatch 만 보장) |
| AC-OL-11 | REQ-OL-012 (positive) | URL "https://example.com/foo" 포함 line_text + empty visited + MockClipboardWriter | `copy_url_at(line, byte_in_url, &mut visited, &mock)` | 반환 `Some("https://example.com/foo")`, `mock.contents() == vec!["https://example.com/foo"]`, `visited.is_visited("https://example.com/foo") == true` | unit test |
| AC-OL-12 | REQ-OL-012 (negative) | FilePath/SpecId/plain 포함 line_text + empty visited + MockClipboardWriter | `copy_url_at(line, byte, &mut visited, &mock)` | 반환 `None`, `mock.contents().is_empty()`, `visited.count() == 0` | unit test (3 case parameterize 또는 3 분리 test) |
| AC-OL-13 | REQ-OL-011 | `TerminalSurface::new()` | 직후 검사 | `visited_links.count() == 0` + `clipboard_writer` 가 ArboardClipboardWriter (trait dispatch 검증) | unit test (visited count 직접, clipboard_writer 는 with_clipboard_writer 인젝션 후 mock contents 으로 간접 검증) |

### MS-2.6 File Layout

#### 수정

- `crates/moai-studio-ui/src/terminal/clipboard.rs`:
  - `ClipboardWriter` trait 신규 (`Send + Sync`, `fn write(&self, &str) -> Result<(), arboard::Error>`)
  - `ArboardClipboardWriter` struct + `Default` impl + `ClipboardWriter` impl (body: `copy_to_clipboard(text)` 호출)
  - `MockClipboardWriter` struct (`Arc<Mutex<Vec<String>>>` 보유) + `Default` impl + `ClipboardWriter` impl + `contents() -> Vec<String>` + `clear()` (선택)
  - `use std::sync::{Arc, Mutex};` 추가
  - 기존 `copy_to_clipboard(text)` 함수 + `sigint_bytes()` 무변경 (FROZEN)
  - 단위 테스트 ~5개 추가 (trait object 컴파일, mock capture single/multi, ArboardClipboardWriter instantiate)

- `crates/moai-studio-ui/src/terminal/mod.rs`:
  - `use moai_studio_terminal::link::{ClickAction, OpenUrl, VisitedLinkRegistry};` (OpenUrl + VisitedLinkRegistry 추가)
  - `use crate::terminal::clipboard::{ArboardClipboardWriter, ClipboardWriter};` 추가
  - `TerminalSurface` struct 에 두 필드 추가:
    - `visited_links: VisitedLinkRegistry`
    - `clipboard_writer: Box<dyn ClipboardWriter + Send + Sync>`
  - `TerminalSurface::new()` 두 필드 default 초기화
  - 신규 메서드 `with_clipboard_writer(mut self, writer: Box<dyn ClipboardWriter + Send + Sync>) -> Self` (builder)
  - 신규 free function (또는 `pub(crate)` impl-side) `copy_url_at(line_text, byte_offset, visited, writer) -> Option<String>` (REQ-OL-012 logic)
  - 신규 메서드 `TerminalSurface::handle_click_for_copy(_row, col, line_text, _cx)` (REQ-OL-013)
  - 기존 `handle_click` 의 `ClickAction::OpenUrl` arm 본문에 `self.visited_links.mark_visited(url.clone())` 한 줄 추가 (REQ-OL-015)
  - `Render::render()` 의 `on_mouse_down(MouseButton::Left, ...)` 다음에 `on_mouse_down(MouseButton::Right, ...)` 분기 추가 (REQ-OL-014)
  - 단위 테스트 ~5개 추가 (copy_url_at positive/negative, handle_click left-click visited tracking, with_clipboard_writer inject, default visited empty)

#### 변경 금지 (MS-2 FROZEN)

- `crates/moai-studio-terminal/src/link.rs` 전체 (MS-1 결과물, MS-2 는 import 만 추가)
- `crates/moai-studio-ui/src/terminal/clipboard.rs` 의 `copy_to_clipboard(text)` + `sigint_bytes()` 시그니처 + body
- `crates/moai-studio-ui/src/terminal/mod.rs` 의 기존 `pixel_to_cell`, `Selection`, `FontMetrics`, `TerminalClickEvent`, `TerminalStdoutEvent`, `set_font_metrics`, `selection_text`, `begin_selection / update_selection / end_selection`
- `crates/moai-studio-ui/src/terminal/mod.rs` 의 `handle_click` 내 `OpenCodeViewer / OpenSpec / CopyUrl` arm (CopyUrl arm 은 left-click 경로에서 도달 안 함 — 우클릭 전용이므로 left-click handle_click 에서는 trace log 유지)
- `crates/moai-studio-ui/src/terminal/input.rs` 전체
- `crates/moai-studio-workspace/**`, `crates/moai-studio-agent/**`, 기타 ui crate 모듈

### MS-2.7 Test Strategy

- 단위 테스트 minimum **10개** (clipboard.rs ~5 + mod.rs ~5):
  - `clipboard.rs`:
    - `clipboard_writer_trait_object_compiles` (AC-OL-8) — `let _: Box<dyn ClipboardWriter + Send + Sync> = Box::new(MockClipboardWriter::default());`
    - `mock_clipboard_writer_captures_single_write` (AC-OL-9 부분)
    - `mock_clipboard_writer_captures_multiple_writes_in_order` (AC-OL-9)
    - `mock_clipboard_writer_default_contents_empty`
    - `arboard_clipboard_writer_can_be_instantiated` (AC-OL-10) — instantiation 만, 실 write 는 cfg-gate 또는 Result 무시
  - `mod.rs`:
    - `copy_url_at_url_returns_url_and_marks_visited` (AC-OL-11)
    - `copy_url_at_file_path_returns_none` (AC-OL-12 case 1)
    - `copy_url_at_spec_id_returns_none` (AC-OL-12 case 2)
    - `copy_url_at_plain_text_returns_none` (AC-OL-12 case 3)
    - `terminal_surface_new_default_visited_links_empty` (AC-OL-13 부분)
    - `terminal_surface_with_clipboard_writer_inject_then_copy_url_at` (통합) — TerminalSurface::new().with_clipboard_writer(mock) → manual `copy_url_at` 호출 (cx 회피) → mock.contents() 검증
- 회귀: ui crate 1295 → ~1305 GREEN (+~10) / terminal crate 47 GREEN (무변경, link.rs FROZEN) / agent 129 + workspace 26 GREEN. 기존 4개 click handler 테스트 (`handle_click_on_*_returns_action`) 무변경 (resolve_click 직접 호출 패턴 보존).
- clippy 0 warning, fmt clean.
- `cargo test -p moai-studio-ui` + `cargo test -p moai-studio-terminal` GREEN 확인 후 PR.

### MS-2.8 DoD

본 MS-2 PASS 시점에 사용자가:
1. 터미널에 출력된 URL 위에서 우클릭 → 시스템 클립보드에 URL 복사됨 (production: arboard, headless CI: log warn 로 fail-soft) + `tracing::info!` 로그
2. 터미널에 출력된 URL 위에서 좌클릭 → (기존) 브라우저 열림 + (신규) `visited_links` 에 URL 기록
3. 우클릭으로 복사한 URL 도 `visited_links` 에 기록됨 (좌/우 클릭 모두 visited 추적)
4. 외부 (테스트 caller) 가 `MockClipboardWriter` inject 후 `TerminalSurface` 의 클립보드 호출 history 검증 가능

audit Top 8 #7 B-1 진척: 60% → **85%** (logic GA + GPUI wire + clipboard + visited tracking 완료, visited 색상 렌더는 MS-3 carry).

### MS-2.9 MS-3 carry (별 SPEC, T5/T6 의존)

T5/T6 cell-grid render path 가 도착해 URL span 이 개별 GPUI element 로 렌더되는 시점에:
- visited URL span 의 foreground color override (e.g., `tok::FG_TERTIARY` 또는 `theme.url_visited`)
- B-1 100% 달성

MS-3 은 본 SPEC 내 추가 milestone (`milestones: [MS-1, MS-2, MS-3]`) 으로 promote 시 lightweight 한도 (≤ 2 milestones) 초과 — 별 SPEC `SPEC-V0-2-0-OSC8-VISITED-RENDER-001` 으로 분리하거나, T5/T6 SPEC 통합 시 거기서 함께 처리.

---

Version: 1.1.0 (lightweight, MS-1 GA + MS-2 amendment) | Source: SPEC-V0-2-0-OSC8-LIFECYCLE-001 | 2026-05-04
