//! SPEC-V3-006 MS-3a: @MX Gutter 아이콘 + Popover (mock scanner).
//!
//! AC-MV-6 — gutter 에 ★/⚠/ℹ/☐ 아이콘 표시 + 클릭 시 popover.
//!
//! MS-3a 에서는 **scan stub** 만 구현한다:
//! - `MxTagScanner` trait: CodeViewer 가 @MX 태그를 얻는 추상 인터페이스
//! - `MockMxScanner`: 테스트용 seed 데이터 제공자
//! - `MxTag` struct: 태그 종류 + 본문 + 줄 번호
//! - `GutterIcon`: 렌더링 히트맵용 데이터 (줄 번호 → 아이콘/색상)
//! - `MxPopoverData`: popover 에 표시할 데이터 구조체
//!
//! ## MS-3b 마이그레이션 훅
//!
//! `MxTagScanner` trait 을 구현하는 `RealMxScanner` 를 추가하면 (regex 기반 in-memory scan,
//! REQ-MV-050) `CodeViewer::mx_scanner` 필드만 교체하면 된다.
//! fan_in 정적 분석 (SQLite cache) 은 별도 SPEC.

use crate::design::tokens::mx_tag;
use regex::Regex;
use std::time::Instant;

// @MX:ANCHOR: [AUTO] mx-tag-scanner-trait
// @MX:REASON: [AUTO] SPEC-V3-006 MS-3a AC-MV-6. MxTagScanner 는 mock (MS-3a) 과
//   real (MS-3b) 구현의 공통 인터페이스다.
//   fan_in >= 3: CodeViewer::load, MockMxScanner (테스트), RealMxScanner (MS-3b).

// ============================================================
// MxTagKind
// ============================================================

/// @MX 태그 종류 4가지 (REQ-MV-053, moai-constitution.md "MX Tag Quality Gates").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MxTagKind {
    /// 불변 계약 함수 — ★ (gold #d4a017)
    Anchor,
    /// 위험 구역 — ⚠ (amber #c47b2a)
    Warn,
    /// 맥락/의도 — ℹ (teal #2a8a8c)
    Note,
    /// 미완성 작업 — ☐ (violet #6a4cc7)
    Todo,
}

impl MxTagKind {
    /// 아이콘 문자 반환 (NFR-MV-11: 색상 외 형태로도 식별 가능).
    pub fn icon(self) -> &'static str {
        match self {
            MxTagKind::Anchor => "★",
            MxTagKind::Warn => "⚠",
            MxTagKind::Note => "ℹ",
            MxTagKind::Todo => "☐",
        }
    }

    /// 아이콘 색상 u32 (design tokens `mx_tag` 모듈, tokens.json v2.0.0).
    pub fn color_u32(self) -> u32 {
        match self {
            MxTagKind::Anchor => mx_tag::ANCHOR, // #d4a017
            MxTagKind::Warn => mx_tag::WARN,     // #c47b2a
            MxTagKind::Note => mx_tag::NOTE,     // #2a8a8c
            MxTagKind::Todo => mx_tag::TODO,     // #6a4cc7
        }
    }
}

// ============================================================
// MxTag
// ============================================================

/// 코드 소스에서 추출된 @MX 태그 항목 (REQ-MV-050 ~ MV-054).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MxTag {
    /// 태그 종류
    pub kind: MxTagKind,
    /// 태그 설명 본문 (`:` 이후 텍스트)
    pub body: String,
    /// 태그가 위치한 줄 번호 (0-indexed)
    pub line: usize,
    /// WARN 태그의 REASON 내용 (없으면 None — REQ-MV-051)
    pub reason: Option<String>,
    /// SPEC ID (body 에서 `SPEC-[A-Z0-9]+-[0-9]+` 패턴으로 추출, REQ-MV-056)
    pub spec_id: Option<String>,
}

// ============================================================
// GutterIcon
// ============================================================

/// 거터에 표시할 아이콘 데이터 (줄별, REQ-MV-053).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GutterIcon {
    /// 아이콘이 표시될 줄 번호 (0-indexed)
    pub line: usize,
    /// 아이콘 문자 (★/⚠/ℹ/☐)
    pub icon: &'static str,
    /// 아이콘 색상 u32
    pub color: u32,
    /// 원본 MxTag 인덱스 (popover 조회용)
    pub tag_index: usize,
}

// ============================================================
// MxPopoverData
// ============================================================

/// 거터 아이콘 클릭 시 표시할 popover 데이터 (REQ-MV-054).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MxPopoverData {
    /// 태그 종류
    pub kind: MxTagKind,
    /// 태그 본문
    pub body: String,
    /// WARN 태그의 REASON (없으면 None; WARN 이고 None 이면 "REASON required" 경고 표시)
    pub reason: Option<String>,
    /// ANCHOR 의 fan_in 카운트 (v1.0.0 은 "N/A" — 정적 분석 미지원, REQ-MV-054)
    pub fan_in: String,
    /// SPEC ID 링크 (있으면 "Jump to SPEC" 버튼 표시)
    pub spec_id: Option<String>,
}

impl MxPopoverData {
    /// `MxTag` 에서 변환한다.
    pub fn from_tag(tag: &MxTag) -> Self {
        Self {
            kind: tag.kind,
            body: tag.body.clone(),
            reason: tag.reason.clone(),
            fan_in: "N/A".to_string(), // v1.0.0 정적 분석 미지원 (REQ-MV-054)
            spec_id: tag.spec_id.clone(),
        }
    }

    /// WARN 태그에 REASON 이 누락된 경우 true (REQ-MV-055).
    pub fn warn_missing_reason(&self) -> bool {
        self.kind == MxTagKind::Warn && self.reason.is_none()
    }
}

// ============================================================
// MxTagScanner trait
// ============================================================

/// @MX 태그 스캐너 추상 인터페이스.
///
/// MS-3a: `MockMxScanner` 가 이 trait 을 구현한다.
/// MS-3b: `RealMxScanner` (regex 기반 in-memory) 가 이 trait 을 구현한다.
pub trait MxTagScanner {
    /// 소스 코드에서 @MX 태그를 스캔하여 목록을 반환한다 (REQ-MV-050).
    fn scan(&self, source: &str) -> Vec<MxTag>;

    /// 스캔 결과에서 거터 아이콘 목록을 계산한다 (REQ-MV-053).
    fn gutter_icons(&self, tags: &[MxTag]) -> Vec<GutterIcon> {
        tags.iter()
            .enumerate()
            .map(|(idx, tag)| GutterIcon {
                line: tag.line,
                icon: tag.kind.icon(),
                color: tag.kind.color_u32(),
                tag_index: idx,
            })
            .collect()
    }
}

// ============================================================
// MockMxScanner
// ============================================================

/// 테스트 및 MS-3a 개발 중 사용하는 mock @MX 태그 스캐너.
///
/// 생성 시 seed 태그 목록을 제공하거나, `scan()` 을 stub 으로 호출하면
/// seed 값을 소스 무관하게 반환한다.
pub struct MockMxScanner {
    seed_tags: Vec<MxTag>,
}

impl MockMxScanner {
    /// 빈 seed 로 생성한다 (scan 시 빈 목록 반환).
    pub fn new() -> Self {
        Self {
            seed_tags: Vec::new(),
        }
    }

    /// seed 태그 목록으로 생성한다.
    pub fn with_tags(tags: Vec<MxTag>) -> Self {
        Self { seed_tags: tags }
    }
}

impl Default for MockMxScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl MxTagScanner for MockMxScanner {
    /// seed 태그 목록을 그대로 반환한다 (소스 내용 무시).
    fn scan(&self, _source: &str) -> Vec<MxTag> {
        self.seed_tags.clone()
    }
}

// ============================================================
// RealMxScanner (MS-3b 선행 구현 — 실제 regex 스캔)
// ============================================================

/// 실제 regex 기반 @MX 태그 스캐너 (REQ-MV-050 ~ MV-051).
///
/// 라인 단위 정규식으로 4종 태그를 추출한다.
/// MS-3a 에서 이미 구현하여 MockMxScanner 와 교체 가능 상태로 제공한다.
pub struct RealMxScanner;

impl RealMxScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealMxScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl MxTagScanner for RealMxScanner {
    fn scan(&self, source: &str) -> Vec<MxTag> {
        // @MX:TYPE: body 패턴 (// 또는 # 주석 형태 모두 지원)
        // 형식: `@MX:(ANCHOR|WARN|NOTE|TODO)[: ]*(.*)` — `[AUTO]` prefix 포함
        let tag_re = Regex::new(r"(?m)@MX:(ANCHOR|WARN|NOTE|TODO)(?:\s*:\s*|\s+)(.*)$")
            .expect("valid regex");

        // REASON sub-line 패턴: `@MX:REASON:` 또는 `[REASON: ...]`
        let reason_inline_re = Regex::new(r"\[REASON:\s*([^\]]*)\]").expect("valid regex");
        let reason_line_re = Regex::new(r"@MX:REASON:\s*(.*)$").expect("valid regex");

        // SPEC ID 패턴: `SPEC-[A-Z0-9]+-[0-9]+`
        let spec_re = Regex::new(r"SPEC-[A-Z0-9]+-[0-9]+").expect("valid regex");

        let lines: Vec<&str> = source.lines().collect();
        let mut tags: Vec<MxTag> = Vec::new();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i];
            if let Some(cap) = tag_re.captures(line) {
                let kind_str = &cap[1];
                let body = cap[2].trim().to_string();
                let kind = match kind_str {
                    "ANCHOR" => MxTagKind::Anchor,
                    "WARN" => MxTagKind::Warn,
                    "NOTE" => MxTagKind::Note,
                    "TODO" => MxTagKind::Todo,
                    _ => unreachable!(),
                };

                // REASON 추출: 같은 줄 inline [REASON: ...] 우선
                let reason = if kind == MxTagKind::Warn {
                    // 먼저 inline REASON 시도
                    let inline = reason_inline_re
                        .captures(&body)
                        .map(|c| c[1].trim().to_string());
                    if inline.is_some() {
                        inline
                    } else {
                        // 다음 줄에 @MX:REASON: 이 있으면 수집
                        lines.get(i + 1).and_then(|next| {
                            reason_line_re
                                .captures(next)
                                .map(|c| c[1].trim().to_string())
                        })
                    }
                } else {
                    None
                };

                // SPEC ID 추출
                let spec_id = spec_re.find(&body).map(|m| m.as_str().to_string());

                tags.push(MxTag {
                    kind,
                    body,
                    line: i,
                    reason,
                    spec_id,
                });
            }
            i += 1;
        }

        tags
    }
}

// ============================================================
// SPEC-V0-3-0-MX-POPOVER-001 MS-1: hover detection + popover FSM
// ============================================================

// @MX:ANCHOR: [AUTO] mx-popover-fsm
// @MX:REASON: [AUTO] SPEC-V0-3-0-MX-POPOVER-001 MS-1 AC-MXP-1~6.
//   Pure-data state machine isolated from GPUI for unit-test coverage.
//   fan_in >= 3: viewer mouse-move handler, viewer escape handler, viewer tick.

/// Hover debounce before promoting Hovering -> Open (REQ-MXP-003).
pub const MX_HOVER_DEBOUNCE_MS: u128 = 200;

/// Hit-test gutter pixel-y to a line index (REQ-MXP-001).
///
/// `viewport_y` is the mouse y coordinate relative to the gutter top edge.
/// Returns `None` when the coordinate falls outside the gutter band or the
/// line height is non-positive.
pub fn hit_test_gutter(viewport_y: f32, line_height: f32, num_lines: usize) -> Option<usize> {
    if viewport_y < 0.0 || line_height <= 0.0 {
        return None;
    }
    let idx = (viewport_y / line_height).floor() as usize;
    if idx < num_lines { Some(idx) } else { None }
}

/// Decide whether the popover should flip to the anchor's left side (REQ-MXP-006).
///
/// When the available width on the anchor's right side is smaller than the
/// popover width, the popover flips to the anchor's left.
pub fn should_flip_left(anchor_x: f32, popover_width: f32, viewport_width: f32) -> bool {
    let available_right = viewport_width - anchor_x;
    available_right < popover_width
}

/// Render popover content as plain text (REQ-MXP-004).
///
/// Used as the unit-testable surface for popover content composition. The
/// GPUI render layer reads the same `MxPopoverData` and produces the visual
/// element; the text rendering preserves identical token order.
pub fn render_popover_text(data: &MxPopoverData) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(4);
    parts.push(data.kind.icon().to_string());
    parts.push(data.body.clone());
    if let Some(reason) = &data.reason {
        parts.push(reason.clone());
    }
    if let Some(spec_id) = &data.spec_id {
        parts.push(spec_id.clone());
    }
    parts.join(" | ")
}

/// FSM state for the @MX gutter hover popover (REQ-MXP-002, MXP-003, MXP-005).
#[derive(Debug, Clone, PartialEq)]
pub enum MxPopoverState {
    /// No hover in progress.
    Closed,
    /// Mouse is hovering a gutter icon but the debounce has not elapsed yet.
    Hovering {
        line: usize,
        tag_index: usize,
        started_at: Instant,
    },
    /// Popover is open with content derived from the hovered tag.
    Open {
        tag_index: usize,
        data: MxPopoverData,
        anchor_line: usize,
    },
}

/// Hover state machine that drives popover open/close (REQ-MXP-002~005).
///
/// Pure-data; no GPUI dependency. The viewer feeds mouse events / ticks /
/// escape into the FSM and reads `state()` to decide rendering.
#[derive(Debug, Clone)]
pub struct MxHoverFsm {
    state: MxPopoverState,
    mouse_in_popover: bool,
}

impl MxHoverFsm {
    pub fn new() -> Self {
        Self {
            state: MxPopoverState::Closed,
            mouse_in_popover: false,
        }
    }

    /// MouseMove over a gutter line. If the line carries a tag, enters
    /// `Hovering` (resetting the timer when the line changes).
    pub fn on_gutter_hover(&mut self, line: usize, icons: &[GutterIcon], now: Instant) {
        let icon = icons.iter().find(|ic| ic.line == line);
        match (self.state.clone(), icon) {
            (_, None) => {
                // No tag at this line. Close unless the user has crossed into
                // the popover area (which keeps the popover alive).
                if !self.mouse_in_popover {
                    self.state = MxPopoverState::Closed;
                }
            }
            (MxPopoverState::Closed, Some(icon)) => {
                self.state = MxPopoverState::Hovering {
                    line,
                    tag_index: icon.tag_index,
                    started_at: now,
                };
            }
            (
                MxPopoverState::Hovering {
                    line: prev_line, ..
                },
                Some(icon),
            ) if prev_line != line => {
                self.state = MxPopoverState::Hovering {
                    line,
                    tag_index: icon.tag_index,
                    started_at: now,
                };
            }
            (MxPopoverState::Hovering { .. }, Some(_)) => {
                // Same line: keep timer.
            }
            (MxPopoverState::Open { .. }, _) => {
                // Already open; movement does not change state until dismissed.
            }
        }
    }

    /// Set whether the mouse is currently inside the popover area. Toggling to
    /// `true` while a popover is open prevents the dismiss-on-leave path.
    pub fn set_mouse_in_popover(&mut self, inside: bool) {
        self.mouse_in_popover = inside;
    }

    /// Mouse left both gutter and popover (REQ-MXP-005).
    pub fn on_mouse_leave_all(&mut self) {
        self.mouse_in_popover = false;
        self.state = MxPopoverState::Closed;
    }

    /// Escape pressed (REQ-MXP-005).
    pub fn on_escape(&mut self) {
        self.mouse_in_popover = false;
        self.state = MxPopoverState::Closed;
    }

    /// Frame tick: promote `Hovering` -> `Open` once the debounce elapses
    /// (REQ-MXP-003).
    pub fn tick(&mut self, now: Instant, tags: &[MxTag]) {
        if let MxPopoverState::Hovering {
            line,
            tag_index,
            started_at,
        } = self.state.clone()
            && now.saturating_duration_since(started_at).as_millis() >= MX_HOVER_DEBOUNCE_MS
            && let Some(tag) = tags.get(tag_index)
        {
            self.state = MxPopoverState::Open {
                tag_index,
                data: MxPopoverData::from_tag(tag),
                anchor_line: line,
            };
        }
    }

    pub fn state(&self) -> &MxPopoverState {
        &self.state
    }

    pub fn is_open(&self) -> bool {
        matches!(self.state, MxPopoverState::Open { .. })
    }

    pub fn open_data(&self) -> Option<&MxPopoverData> {
        match &self.state {
            MxPopoverState::Open { data, .. } => Some(data),
            _ => None,
        }
    }
}

impl Default for MxHoverFsm {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 (MS-3a TDD — RED → GREEN)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── T1: MxTagKind 아이콘 & 색상 ──

    #[test]
    fn mx_tag_kind_anchor_icon_is_star() {
        assert_eq!(MxTagKind::Anchor.icon(), "★");
    }

    #[test]
    fn mx_tag_kind_warn_icon_is_warning() {
        assert_eq!(MxTagKind::Warn.icon(), "⚠");
    }

    #[test]
    fn mx_tag_kind_note_icon_is_info() {
        assert_eq!(MxTagKind::Note.icon(), "ℹ");
    }

    #[test]
    fn mx_tag_kind_todo_icon_is_checkbox() {
        assert_eq!(MxTagKind::Todo.icon(), "☐");
    }

    #[test]
    fn mx_tag_kind_anchor_color_is_gold() {
        assert_eq!(MxTagKind::Anchor.color_u32(), 0xd4a017);
    }

    #[test]
    fn mx_tag_kind_warn_color_is_amber() {
        assert_eq!(MxTagKind::Warn.color_u32(), 0xc47b2a);
    }

    #[test]
    fn mx_tag_kind_note_color_is_teal() {
        assert_eq!(MxTagKind::Note.color_u32(), 0x2a8a8c);
    }

    #[test]
    fn mx_tag_kind_todo_color_is_violet() {
        assert_eq!(MxTagKind::Todo.color_u32(), 0x6a4cc7);
    }

    // ── T2: MockMxScanner ──

    #[test]
    fn mock_scanner_empty_returns_no_tags() {
        let scanner = MockMxScanner::new();
        let tags = scanner.scan("fn main() {}");
        assert!(tags.is_empty());
    }

    #[test]
    fn mock_scanner_seed_returns_seed_tags() {
        let seed = vec![MxTag {
            kind: MxTagKind::Anchor,
            body: "test anchor".to_string(),
            line: 5,
            reason: None,
            spec_id: None,
        }];
        let scanner = MockMxScanner::with_tags(seed.clone());
        let tags = scanner.scan("any source");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].body, "test anchor");
        assert_eq!(tags[0].line, 5);
    }

    // ── T3: GutterIcon 생성 ──

    #[test]
    fn gutter_icons_from_tags_maps_correctly() {
        let tags = vec![
            MxTag {
                kind: MxTagKind::Anchor,
                body: String::new(),
                line: 10,
                reason: None,
                spec_id: None,
            },
            MxTag {
                kind: MxTagKind::Warn,
                body: String::new(),
                line: 20,
                reason: Some("danger".to_string()),
                spec_id: None,
            },
        ];
        let scanner = MockMxScanner::with_tags(tags.clone());
        let icons = scanner.gutter_icons(&tags);

        assert_eq!(icons.len(), 2);
        assert_eq!(icons[0].line, 10);
        assert_eq!(icons[0].icon, "★");
        assert_eq!(icons[0].color, 0xd4a017);
        assert_eq!(icons[1].line, 20);
        assert_eq!(icons[1].icon, "⚠");
        assert_eq!(icons[1].color, 0xc47b2a);
    }

    #[test]
    fn gutter_icons_tag_index_is_correct() {
        let tags = vec![
            MxTag {
                kind: MxTagKind::Note,
                body: String::new(),
                line: 1,
                reason: None,
                spec_id: None,
            },
            MxTag {
                kind: MxTagKind::Todo,
                body: String::new(),
                line: 2,
                reason: None,
                spec_id: None,
            },
        ];
        let scanner = MockMxScanner::with_tags(tags.clone());
        let icons = scanner.gutter_icons(&tags);
        assert_eq!(icons[0].tag_index, 0);
        assert_eq!(icons[1].tag_index, 1);
    }

    // ── T4: MxPopoverData ──

    #[test]
    fn popover_data_from_anchor_tag() {
        let tag = MxTag {
            kind: MxTagKind::Anchor,
            body: "root-view-binding SPEC-V3-004".to_string(),
            line: 42,
            reason: None,
            spec_id: Some("SPEC-V3-004".to_string()),
        };
        let popover = MxPopoverData::from_tag(&tag);
        assert_eq!(popover.kind, MxTagKind::Anchor);
        assert_eq!(popover.fan_in, "N/A");
        assert_eq!(popover.spec_id, Some("SPEC-V3-004".to_string()));
        assert!(!popover.warn_missing_reason());
    }

    #[test]
    fn popover_data_warn_with_reason() {
        let tag = MxTag {
            kind: MxTagKind::Warn,
            body: "goroutine without context".to_string(),
            line: 7,
            reason: Some("no cancel propagation".to_string()),
            spec_id: None,
        };
        let popover = MxPopoverData::from_tag(&tag);
        assert_eq!(popover.reason, Some("no cancel propagation".to_string()));
        assert!(!popover.warn_missing_reason());
    }

    #[test]
    fn popover_data_warn_missing_reason_is_flagged() {
        let tag = MxTag {
            kind: MxTagKind::Warn,
            body: "some warn without reason".to_string(),
            line: 3,
            reason: None,
            spec_id: None,
        };
        let popover = MxPopoverData::from_tag(&tag);
        assert!(
            popover.warn_missing_reason(),
            "WARN + reason=None → flagged"
        );
    }

    #[test]
    fn popover_data_non_warn_missing_reason_is_not_flagged() {
        let tag = MxTag {
            kind: MxTagKind::Note,
            body: "context".to_string(),
            line: 1,
            reason: None,
            spec_id: None,
        };
        let popover = MxPopoverData::from_tag(&tag);
        assert!(!popover.warn_missing_reason());
    }

    // ── T5: RealMxScanner ──

    #[test]
    fn real_scanner_detects_anchor_tag() {
        let source = "// @MX:ANCHOR: root-view-binding\n// more code";
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].kind, MxTagKind::Anchor);
        assert_eq!(tags[0].line, 0);
    }

    #[test]
    fn real_scanner_detects_warn_with_reason_subline() {
        let source = "// @MX:WARN: goroutine without context\n// @MX:REASON: no cancel prop";
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].kind, MxTagKind::Warn);
        assert_eq!(tags[0].reason, Some("no cancel prop".to_string()));
    }

    #[test]
    fn real_scanner_detects_warn_with_inline_reason() {
        let source = "// @MX:WARN: danger zone [REASON: very risky]";
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].reason.as_deref(), Some("very risky"));
    }

    #[test]
    fn real_scanner_detects_note_tag() {
        let source = "# @MX:NOTE: this is a note\nsome_code()";
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].kind, MxTagKind::Note);
    }

    #[test]
    fn real_scanner_detects_todo_tag() {
        let source = "// @MX:TODO: implement this later";
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].kind, MxTagKind::Todo);
    }

    #[test]
    fn real_scanner_detects_multiple_tags() {
        let source = concat!(
            "// @MX:ANCHOR: anchor1\n",
            "fn foo() {}\n",
            "// @MX:WARN: warn1\n",
            "// @MX:REASON: reason1\n",
            "// @MX:NOTE: note1\n",
            "// @MX:TODO: todo1\n",
        );
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags.len(), 4, "4개 태그 감지되어야 한다");
    }

    #[test]
    fn real_scanner_extracts_spec_id_from_body() {
        let source = "// @MX:ANCHOR: root-view SPEC-V3-004 binding";
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].spec_id, Some("SPEC-V3-004".to_string()));
    }

    #[test]
    fn real_scanner_no_spec_id_is_none() {
        let source = "// @MX:NOTE: no spec here";
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags[0].spec_id, None);
    }

    #[test]
    fn real_scanner_line_numbers_are_correct() {
        let source = "line0\n// @MX:ANCHOR: anchor\nline2\n// @MX:TODO: todo\nline4";
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].line, 1, "anchor 는 1번째 줄");
        assert_eq!(tags[1].line, 3, "todo 는 3번째 줄");
    }

    #[test]
    fn real_scanner_warn_without_reason_has_none_reason() {
        let source = "// @MX:WARN: dangerous code\nsome_code()";
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags[0].reason, None, "다음 줄이 REASON 이 아니면 None");
    }

    #[test]
    fn real_scanner_auto_prefix_is_handled() {
        // [AUTO] prefix 가 있는 실제 코드 패턴
        let source = "// @MX:ANCHOR: [AUTO] root-view-tab-container-binding";
        let scanner = RealMxScanner::new();
        let tags = scanner.scan(source);
        assert_eq!(tags.len(), 1);
        assert!(tags[0].body.contains("[AUTO]"), "body 에 [AUTO] 포함");
    }

    // ============================================================
    // T6: SPEC-V0-3-0-MX-POPOVER-001 MS-1 — hover/popover FSM
    // (AC-MXP-1 ~ AC-MXP-6)
    // ============================================================

    use std::time::Duration;

    fn sample_icons(tags: &[MxTag]) -> Vec<GutterIcon> {
        let scanner = MockMxScanner::with_tags(tags.to_vec());
        scanner.gutter_icons(tags)
    }

    // ── AC-MXP-1: hit-test gutter ──

    #[test]
    fn ac_mxp_1_hit_test_returns_line_index_within_band() {
        // line_height=20, viewport_y=50 → floor(50/20) = 2.
        assert_eq!(hit_test_gutter(50.0, 20.0, 5), Some(2));
        // Top of line 0.
        assert_eq!(hit_test_gutter(0.0, 20.0, 5), Some(0));
        // Just below top of line 4.
        assert_eq!(hit_test_gutter(80.5, 20.0, 5), Some(4));
    }

    #[test]
    fn ac_mxp_1_hit_test_returns_none_when_outside_band() {
        // 5 lines * 20 = 100 px band; y=150 is below.
        assert_eq!(hit_test_gutter(150.0, 20.0, 5), None);
        // Negative y.
        assert_eq!(hit_test_gutter(-1.0, 20.0, 5), None);
        // Non-positive line_height.
        assert_eq!(hit_test_gutter(50.0, 0.0, 5), None);
        assert_eq!(hit_test_gutter(50.0, -20.0, 5), None);
    }

    // ── AC-MXP-2: hover → debounce → open ──

    #[test]
    fn ac_mxp_2_hover_promotes_to_open_after_debounce() {
        let tags = vec![MxTag {
            kind: MxTagKind::Anchor,
            body: "anchored".into(),
            line: 2,
            reason: None,
            spec_id: None,
        }];
        let icons = sample_icons(&tags);

        let mut fsm = MxHoverFsm::new();
        let t0 = Instant::now();
        fsm.on_gutter_hover(2, &icons, t0);
        assert!(matches!(
            fsm.state(),
            MxPopoverState::Hovering {
                line: 2,
                tag_index: 0,
                ..
            }
        ));

        // Tick before debounce -> still hovering.
        fsm.tick(t0 + Duration::from_millis(100), &tags);
        assert!(
            !fsm.is_open(),
            "100ms < 200ms debounce, must remain hovering"
        );

        // Tick after debounce -> open.
        fsm.tick(t0 + Duration::from_millis(201), &tags);
        assert!(fsm.is_open(), "201ms >= 200ms debounce, must open");
        assert_eq!(fsm.open_data().map(|d| d.kind), Some(MxTagKind::Anchor));
    }

    #[test]
    fn ac_mxp_2_changing_line_resets_debounce_timer() {
        let tags = vec![
            MxTag {
                kind: MxTagKind::Anchor,
                body: "a".into(),
                line: 1,
                reason: None,
                spec_id: None,
            },
            MxTag {
                kind: MxTagKind::Note,
                body: "b".into(),
                line: 4,
                reason: None,
                spec_id: None,
            },
        ];
        let icons = sample_icons(&tags);

        let mut fsm = MxHoverFsm::new();
        let t0 = Instant::now();
        fsm.on_gutter_hover(1, &icons, t0);
        // After 150ms move to line 4 -> timer resets, started_at = t0+150.
        fsm.on_gutter_hover(4, &icons, t0 + Duration::from_millis(150));
        // Tick at t0+250 -> only 100ms on line 4, must remain hovering.
        fsm.tick(t0 + Duration::from_millis(250), &tags);
        assert!(!fsm.is_open());
        // Tick at t0+360 -> 210ms on line 4, opens with the line-4 tag.
        fsm.tick(t0 + Duration::from_millis(360), &tags);
        assert!(fsm.is_open());
        assert_eq!(fsm.open_data().map(|d| d.kind), Some(MxTagKind::Note));
    }

    // ── AC-MXP-3: popover content includes icon + body + reason + spec_id ──

    #[test]
    fn ac_mxp_3_render_text_includes_warn_icon_body_reason_spec_id() {
        let data = MxPopoverData {
            kind: MxTagKind::Warn,
            body: "goroutine without context".into(),
            reason: Some("no cancel prop".into()),
            fan_in: "N/A".into(),
            spec_id: Some("SPEC-V3-006".into()),
        };
        let rendered = render_popover_text(&data);
        assert!(rendered.contains("⚠"), "warn icon present");
        assert!(
            rendered.contains("goroutine without context"),
            "body present"
        );
        assert!(rendered.contains("no cancel prop"), "reason present");
        assert!(rendered.contains("SPEC-V3-006"), "spec id present");
    }

    #[test]
    fn ac_mxp_3_render_text_omits_optional_fields_when_none() {
        let data = MxPopoverData {
            kind: MxTagKind::Note,
            body: "context note".into(),
            reason: None,
            fan_in: "N/A".into(),
            spec_id: None,
        };
        let rendered = render_popover_text(&data);
        assert!(rendered.contains("ℹ"));
        assert!(rendered.contains("context note"));
        // No spurious empty separators; reason / spec id absent.
        assert!(!rendered.contains("None"));
    }

    // ── AC-MXP-4: mouse-leave-all dismisses ──

    #[test]
    fn ac_mxp_4_mouse_leave_all_closes_open_popover() {
        let tags = vec![MxTag {
            kind: MxTagKind::Todo,
            body: "todo".into(),
            line: 0,
            reason: None,
            spec_id: None,
        }];
        let icons = sample_icons(&tags);

        let mut fsm = MxHoverFsm::new();
        let t0 = Instant::now();
        fsm.on_gutter_hover(0, &icons, t0);
        fsm.tick(t0 + Duration::from_millis(201), &tags);
        assert!(fsm.is_open());

        fsm.on_mouse_leave_all();
        assert!(matches!(fsm.state(), MxPopoverState::Closed));
        assert!(fsm.open_data().is_none());
    }

    #[test]
    fn ac_mxp_4_mouse_in_popover_keeps_state_when_gutter_has_no_tag() {
        let tags = vec![MxTag {
            kind: MxTagKind::Anchor,
            body: "a".into(),
            line: 1,
            reason: None,
            spec_id: None,
        }];
        let icons = sample_icons(&tags);

        let mut fsm = MxHoverFsm::new();
        let t0 = Instant::now();
        fsm.on_gutter_hover(1, &icons, t0);
        fsm.tick(t0 + Duration::from_millis(201), &tags);
        assert!(fsm.is_open());

        // Mouse moves into popover area, then over a gutter line without a tag.
        fsm.set_mouse_in_popover(true);
        fsm.on_gutter_hover(99, &icons, t0 + Duration::from_millis(300));
        assert!(fsm.is_open(), "popover stays open while mouse is inside it");
    }

    // ── AC-MXP-5: Escape dismisses ──

    #[test]
    fn ac_mxp_5_escape_closes_open_popover() {
        let tags = vec![MxTag {
            kind: MxTagKind::Note,
            body: "n".into(),
            line: 3,
            reason: None,
            spec_id: None,
        }];
        let icons = sample_icons(&tags);

        let mut fsm = MxHoverFsm::new();
        let t0 = Instant::now();
        fsm.on_gutter_hover(3, &icons, t0);
        fsm.tick(t0 + Duration::from_millis(201), &tags);
        assert!(fsm.is_open());

        fsm.on_escape();
        assert!(matches!(fsm.state(), MxPopoverState::Closed));
    }

    #[test]
    fn ac_mxp_5_escape_closes_hovering_state_too() {
        let tags = vec![MxTag {
            kind: MxTagKind::Note,
            body: "n".into(),
            line: 0,
            reason: None,
            spec_id: None,
        }];
        let icons = sample_icons(&tags);

        let mut fsm = MxHoverFsm::new();
        let t0 = Instant::now();
        fsm.on_gutter_hover(0, &icons, t0);
        // Don't tick — stay in Hovering.
        fsm.on_escape();
        assert!(matches!(fsm.state(), MxPopoverState::Closed));
    }

    // ── AC-MXP-6: flip when right space < popover width ──

    #[test]
    fn ac_mxp_6_flip_to_left_when_right_space_insufficient() {
        // viewport=800, popover=300, anchor_x=600 → right=200 < 300 → flip.
        assert!(should_flip_left(600.0, 300.0, 800.0));
    }

    #[test]
    fn ac_mxp_6_no_flip_when_right_space_sufficient() {
        // viewport=800, popover=300, anchor_x=400 → right=400 ≥ 300 → no flip.
        assert!(!should_flip_left(400.0, 300.0, 800.0));
        // Edge case: exactly equal → fits, no flip.
        assert!(!should_flip_left(500.0, 300.0, 800.0));
    }
}
