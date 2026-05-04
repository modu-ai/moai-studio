# SPEC-V0-3-0-MX-POPOVER-001 — @MX Gutter Hover Popover wiring

| Field | Value |
|-------|-------|
| **ID** | SPEC-V0-3-0-MX-POPOVER-001 |
| **Title** | MXGutterLine hover detection + MXPopover render/dismiss + content fetch from MxAnnotation |
| **Status** | in_progress |
| **Priority** | Low (P3 polish) |
| **Revision** | 1.0 (lightweight) |
| **Dependencies** | SPEC-V3-006 MS-3a (MxTagScanner / MxPopoverData / GutterIcon 자료구조 — 이미 land) |
| **Cycle** | v0.3.0 (audit Top 16 #11 / E-3 carry) |
| **Milestones** | MS-1 |

## HISTORY

- 2026-05-04: 초안 작성. v0.3.0 cycle Sprint 2 #1 진입. audit feature-audit.md §4 #11 E-3 PARTIAL carry. Lightweight 한도 (≤10KB / ≤8 AC / 1 MS) 충족.

## 1. Purpose

`viewer/mx_gutter.rs` 는 SPEC-V3-006 MS-3a 기준 `MxTagKind` / `MxTag` / `GutterIcon` / `MxPopoverData` / `MxTagScanner` (Mock + Real) 자료구조 + RealMxScanner regex 스캔까지 모두 land 되어 있다. audit Top 16 #11 E-3 는 PARTIAL — gutter 표시는 작동하나 **hover 감지 / popover 위치 / content fetch 결합** 이 미구현 상태다. 본 SPEC 은 MxAnnotation 소스 (이미 gutter 가 사용 중) 에서 popover 컨텐츠를 끌어와 hover → popover open → mouse leave/Escape → popover close 의 최소 사이클을 완성한다. 키보드 navigation / ARIA / click-to-pin 은 carry-out.

## 2. Goals

- Gutter line hit-test: 마우스 좌표 → 해당 줄 인덱스 매핑 (MouseMove)
- Hover state: line index + 진입 시각 (debounce ~200ms) 보유
- Popover open: hover 유지 시 `MxPopoverData` 를 anchor (gutter 줄 우측) 위치에 render
- Popover content: kind / icon / body / WARN 의 REASON 1 줄 / SPEC ID 표시 — `MxPopoverData::from_tag` 결과 그대로
- Popover dismiss: mouse leave (gutter + popover 영역 모두 벗어남) 또는 Escape 키
- 충돌 회피 basic: viewport 우측 경계를 넘으면 anchor 좌측으로 flip
- FROZEN: 기존 `MxTagKind` / `MxTag` / `GutterIcon` / `MxPopoverData` / `MxTagScanner` 시그니처 무수정
- TRUST 5 gates ALL PASS

## 3. Non-Goals / Exclusions

- 키보드 navigation (Tab / Arrow keys 로 gutter icon 순회) — carry-to MS-2
- ARIA 속성 (role="tooltip", aria-describedby) — carry-to MS-2
- Click-to-pin (popover 를 클릭으로 고정 / `Jump to SPEC` 버튼 동작) — carry-to MS-2
- fan_in 정적 분석 (현재 "N/A" 표시 유지) — 별도 SPEC
- WARN missing-REASON 에 대한 인라인 경고 UI — 차후 SPEC
- MxPopover 의 multi-tag 줄 (한 줄에 2개 이상 태그) 처리 — carry-to MS-2 (현재는 첫 태그만 표시)
- Hover delay / popover fade 애니메이션 튜닝 — 200ms / no animation 고정

## 4. Requirements

- REQ-MXP-001: `viewer/mx_gutter.rs` (또는 형제 모듈) 는 `(viewport_y: Pixels, line_height: Pixels) → Option<usize>` 형태의 hit-test helper 를 제공한다. 좌표가 gutter 영역 밖이면 `None`.
- REQ-MXP-002: WHEN MouseMove 가 gutter 위에서 발생하고 hit-test 결과가 `MxTagScanner::gutter_icons()` 결과 중 한 항목과 매칭되면, viewer 는 hover state (`hovered_line: usize`, `hover_started_at: Instant`) 를 갱신한다.
- REQ-MXP-003: WHILE hover 가 디바운스 (200ms) 를 초과하여 유지되면 viewer 는 해당 `tag_index` 의 `MxPopoverData` 를 popover 로 열어야 한다. Popover 는 gutter 줄 우측에 anchor.
- REQ-MXP-004: Popover render 는 `kind.icon()` + `body` (1 line ellipsis 허용) + `reason` 이 `Some` 이면 그 1 line + `spec_id` 가 `Some` 이면 ID 표시를 포함한다. content fetch 소스는 viewer 가 이미 보유한 `Vec<MxTag>` (gutter 표시에 사용 중인 것과 동일) — 새 fetch 경로 추가 금지.
- REQ-MXP-005: WHEN mouse 가 gutter 영역 + popover 영역 모두를 벗어나거나 Escape 키가 눌리면, viewer 는 hover state 와 popover 를 동시에 닫는다.
- REQ-MXP-006: WHERE popover anchor (gutter 줄 우측) 의 화면 우측 경계까지 거리가 popover 너비보다 작으면, popover 를 anchor 좌측으로 flip 한다 (수직 위치는 유지).
- REQ-MXP-007: 기존 `MxTagKind` / `MxTag` / `GutterIcon` / `MxPopoverData` / `MxTagScanner` (Mock + Real) 자료구조 시그니처와 기존 RealMxScanner regex 동작은 무수정 (R5 — 호환성 보존, FROZEN). 신규 코드는 viewer 측 hover/popover wiring 로 한정.

## 5. Acceptance Criteria

| AC ID | Given | When | Then | Verification |
|-------|-------|------|------|--------------|
| AC-MXP-1 | viewer 에 5 lines / line 2,4 에 MxTag (Anchor, Warn) | hit-test (y=line2 중앙) | `Some(2)` 반환, anchor tag_index=0 매칭 | unit test |
| AC-MXP-2 | viewer 에 hovered_line=2, hover_started_at = now-201ms | tick → popover_open() | popover 가 line 2 의 MxPopoverData 와 함께 open 상태 | unit test |
| AC-MXP-3 | popover open / WARN tag with reason="no cancel prop" + spec_id=Some("SPEC-V3-006") | render content | 출력에 "⚠" + body + "no cancel prop" + "SPEC-V3-006" 모두 포함 | unit test |
| AC-MXP-4 | popover open / hovered_line=2 | mouse 가 gutter+popover 영역 밖으로 이동 | popover dismissed, hovered_line=None | unit test |
| AC-MXP-5 | popover open | Escape 키 입력 | popover dismissed | unit test |
| AC-MXP-6 | viewport_width=800 / popover_width=300 / anchor_x=600 | flip 판정 | anchor 우측 공간(200) < popover_width → 좌측 flip | unit test |
| AC-MXP-7 | 기존 mx_gutter tests (현재 24) | run | ALL PASS, 회귀 0 | existing tests |
| AC-MXP-8 | cargo build/clippy/fmt + ui 전체 tests | run | ALL PASS, ui 1332 + N → 1332+N PASS | CI |

## 6. File Layout

| Path | Status | Note |
|------|--------|------|
| `crates/moai-studio-ui/src/viewer/mx_gutter.rs` | modified | hit-test helper + hover state + popover open/close FSM (≤200 LOC 추가). 기존 자료구조 / RealMxScanner FROZEN. |
| `crates/moai-studio-ui/src/viewer/mod.rs` | modified (potentially) | viewer 통합 시 mx_gutter hover state 노출 (있을 경우) |
| `.moai/specs/SPEC-V0-3-0-MX-POPOVER-001/spec.md` | created | 본 문서 |
| `.moai/specs/SPEC-V0-3-0-MX-POPOVER-001/progress.md` | created | 구현 후 갱신 |

FROZEN:
- `MxTagKind` enum + `icon()` / `color_u32()`
- `MxTag` struct fields
- `GutterIcon` struct fields
- `MxPopoverData` struct + `from_tag()` / `warn_missing_reason()`
- `MxTagScanner` trait + 기본 `gutter_icons()` 구현
- `MockMxScanner` / `RealMxScanner` 시그니처 + RealMxScanner 의 regex 패턴
- 기존 24 tests 동작

Carry-out (MS-2 후보):
- 키보드 nav (Tab / Arrow)
- ARIA role/속성
- Click-to-pin + Jump to SPEC 버튼 활성
- 한 줄 multi-tag 처리 (현재 첫 태그만)
- WARN missing-REASON 인라인 경고 UI

## 7. Test Strategy

신규 hit-test / hover FSM / popover open/close / flip 판정에 대해 단위 테스트 ~6개 추가 (AC-MXP-1 ~ AC-MXP-6 각각 1개). 기존 24 tests 는 변경 없이 통과 (AC-MXP-7). ui 전체 1332 → 1338 목표. 회귀 0. GPUI render 통합 단계 (실제 popover element 그리기) 는 unit test 가 닿지 않는 영역이므로 mod-level FSM 만 검증하고 시각 검증은 manual smoke 으로 카바.

---

Version: 1.0.0 (lightweight)
Created: 2026-05-04
Cycle: v0.3.0 Sprint 2 #1 (audit Top 16 #11 E-3 PARTIAL closure)
Carry-to: 키보드 nav / ARIA / click-to-pin / multi-tag 줄 / WARN missing-REASON UI — MS-2 또는 차후 SPEC
