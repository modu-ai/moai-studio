//! TerminalSurface — GPUI Terminal 렌더 컴포넌트 (SPEC-V3-002 RG-V3-002-4, RG-V3-002-5).
//!
//! @MX:ANCHOR: terminal-surface-render
//! @MX:REASON: GPUI 렌더 진입점 — TerminalSurface::render 는 moai-studio-ui 의 모든
//!   터미널 출력 경로가 수렴하는 지점이다. fan_in ≥ 3:
//!   content_area 분기 (lib.rs), PtyEvent handler (on_output), input.rs key dispatch (T5).

pub mod clipboard;
pub mod input;

use crate::design::tokens::{self as tok, ide_accent};
use gpui::{Context, IntoElement, Keystroke, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_terminal::link::ClickAction;

// ============================================================
// 폰트 메트릭 — pixel_to_cell 계산 기준
// ============================================================

/// 폰트 메트릭 — pixel_to_cell 계산 기준 (SPEC-V3-002 RG-V3-002-5).
///
/// @MX:NOTE: font-metric-coord-mapping
/// advance_width × col + line_height × row = pixel 기준점.
/// 기본값은 8px × 16px (Menlo 모노스페이스 1x DPI 기준).
/// 실제 값은 GPUI font system 에서 추출하여 set_font_metrics 로 갱신한다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    /// 글자 픽셀 폭 (monospace 기준 동일 폭, px)
    pub advance_width: f32,
    /// 행 픽셀 높이 (line-height, px)
    pub line_height: f32,
}

impl Default for FontMetrics {
    fn default() -> Self {
        // Menlo 12pt @ 1x DPI 기준 경험값
        Self {
            advance_width: 8.0,
            line_height: 16.0,
        }
    }
}

// ============================================================
// Selection — 마우스 드래그 선택 영역
// ============================================================

/// 마우스 드래그 선택 영역 (SPEC-V3-002 RG-V3-002-5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    /// 드래그 시작 셀 (row, col) — 0-indexed
    pub start: (u16, u16),
    /// 드래그 현재 종료 셀 (row, col) — 0-indexed
    pub end: (u16, u16),
}

impl Selection {
    /// 선택 영역을 정규화 (top-left, bottom-right) 로 반환.
    ///
    /// 역방향 드래그(end < start)에서도 올바른 사각형 경계를 반환한다.
    pub fn bounding_rect(&self) -> ((u16, u16), (u16, u16)) {
        let (r1, c1) = self.start;
        let (r2, c2) = self.end;
        ((r1.min(r2), c1.min(c2)), (r1.max(r2), c1.max(c2)))
    }

    /// 단일 셀 (start == end) 선택인지 확인.
    pub fn is_collapsed(&self) -> bool {
        self.start == self.end
    }
}

// ============================================================
// TerminalState — T3 완료 후 RenderSnapshot 으로 교체 예정
// ============================================================

/// 터미널 렌더 상태 — T3 완료 후 `moai_studio_terminal::libghostty_ffi::RenderSnapshot` 으로 교체.
///
/// @MX:TODO: T3 완료 후 terminal crate 의 RenderSnapshot 으로 교체.
///   교체 지점: TerminalSurface::snapshot 필드 타입 변경 + on_output 파라미터 변경.
#[derive(Debug, Clone, Default)]
pub struct TerminalState {
    /// 커서 행 (0-indexed)
    pub cursor_row: u16,
    /// 커서 열 (0-indexed)
    pub cursor_col: u16,
    /// 첫 번째 행 텍스트 (디버그 표시용)
    pub row0_text: String,
    /// 총 출력 바이트 수 (AC-T-9 검증용)
    pub total_bytes: usize,
}

// ============================================================
// TerminalSurface — GPUI 컴포넌트
// ============================================================

/// GPUI TerminalSurface 컴포넌트.
///
/// PTY worker 가 emit 한 출력 바이트를 수신하여 TerminalState 를 갱신하고
/// GPUI re-render 를 트리거한다 (cx.notify()).
///
/// T4 에서는 TerminalState 를 직접 관리하고, T3 완료 후 PtyEvent + RenderSnapshot
/// 으로 전환한다.
pub struct TerminalSurface {
    /// 최신 터미널 상태 (T3 완료 후 RenderSnapshot 으로 교체)
    pub state: TerminalState,
    /// 폰트 메트릭 — pixel_to_cell 계산용
    pub font: FontMetrics,
    /// 마우스 드래그 선택 영역 (없으면 None)
    pub selection: Option<Selection>,
    /// 커서 blink 표시 여부
    pub cursor_visible: bool,
    /// PTY stdin writer — T6(ghostty-spike) 에서 실제 PTY sender 로 교체.
    ///
    /// @MX:TODO: T6 에서 tokio::sync::mpsc::UnboundedSender<Vec<u8>> 로 교체.
    ///   현재는 pending 바이트를 버퍼에 쌓고 inspector 가 drain 한다.
    pub pending_input: Vec<u8>,
}

impl TerminalSurface {
    /// 기본 상태로 TerminalSurface 생성.
    pub fn new() -> Self {
        Self {
            state: TerminalState::default(),
            font: FontMetrics::default(),
            selection: None,
            cursor_visible: true,
            pending_input: Vec::new(),
        }
    }

    /// 폰트 메트릭을 갱신한다 (GPUI font system 에서 호출).
    pub fn set_font_metrics(&mut self, metrics: FontMetrics) {
        self.font = metrics;
    }

    /// PTY stdout 출력 바이트를 처리한다 (T3 완료 후 PtyEvent::Output 으로 전환).
    ///
    /// AC-T-6: cx.notify() 호출로 content_area re-render 트리거.
    pub fn on_output(&mut self, bytes: &[u8], cx: &mut Context<Self>) {
        // TODO(T5): VtTerminal::feed(bytes) → render_state() → snapshot 갱신
        self.state.total_bytes += bytes.len();
        // 첫 번째 줄 텍스트 업데이트 (ASCII 안전 부분만 표시)
        let ascii: String = bytes
            .iter()
            .filter(|b| b.is_ascii_graphic() || **b == b' ')
            .map(|b| *b as char)
            .collect();
        if !ascii.is_empty() {
            self.state.row0_text = ascii;
        }
        cx.notify();
    }

    /// shell 종료를 처리한다 (T3 완료 후 PtyEvent::ProcessExit 으로 전환).
    pub fn on_process_exit(&mut self, exit_code: i32) {
        tracing::info!(exit_code, "TerminalSurface: shell 종료");
    }

    /// 픽셀 좌표 → 셀 좌표 변환 (SPEC-V3-002 RG-V3-002-5).
    ///
    /// @MX:NOTE: font-metric-coord-mapping
    /// 반환값: (row: u16, col: u16) — 0-indexed.
    /// 음수 입력은 0 으로 처리된다.
    pub fn pixel_to_cell(&self, x: f32, y: f32) -> (u16, u16) {
        let col = if x <= 0.0 {
            0
        } else {
            (x / self.font.advance_width).floor() as u16
        };
        let row = if y <= 0.0 {
            0
        } else {
            (y / self.font.line_height).floor() as u16
        };
        (row, col)
    }

    /// 마우스 드래그 시작 — 선택 영역 초기화.
    pub fn begin_selection(&mut self, x: f32, y: f32) {
        let cell = self.pixel_to_cell(x, y);
        self.selection = Some(Selection {
            start: cell,
            end: cell,
        });
    }

    /// 마우스 드래그 업데이트 — 선택 종료 셀 갱신.
    pub fn update_selection(&mut self, x: f32, y: f32) {
        // pixel_to_cell 먼저 계산 후 &mut self.selection 획득 (borrow checker 충족)
        let cell = self.pixel_to_cell(x, y);
        if let Some(sel) = &mut self.selection {
            sel.end = cell;
        }
        // begin_selection 없이 update 는 selection 을 생성하지 않는다.
    }

    /// 마우스 버튼 업 — 선택 확정.
    pub fn end_selection(&mut self, x: f32, y: f32) {
        self.update_selection(x, y);
    }

    /// 선택 영역 초기화.
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// 현재 선택 영역의 텍스트를 반환한다 (클립보드 복사용).
    ///
    /// T5 에서: Grid<Cell> 을 순회하여 실제 텍스트를 추출한다.
    pub fn selection_text(&self) -> Option<String> {
        self.selection.as_ref().map(|_| {
            // TODO(T5): Grid 기반 텍스트 추출
            self.state.row0_text.clone()
        })
    }

    /// GPUI key down 이벤트 처리 — ANSI 인코딩 → pending_input 에 버퍼링.
    ///
    /// 우선순위:
    ///   1. 클립보드 복사 (Cmd+C / Ctrl+Shift+C) — arboard 복사 후 return
    ///   2. SIGINT (Ctrl+C, 선택 없음) — 0x03 버퍼링
    ///   3. 일반 키 — ANSI encoding → 버퍼링
    ///
    /// T6 에서: pending_input → PTY stdin writer 로 직접 전송.
    pub fn handle_key_down(&mut self, keystroke: &Keystroke, cx: &mut Context<Self>) {
        use crate::terminal::input::{is_clipboard_copy, keystroke_to_ansi_bytes};

        // 1. 클립보드 복사 단축키 (Cmd+C / Ctrl+Shift+C)
        if is_clipboard_copy(keystroke) {
            if let Some(text) = self.selection_text() {
                match clipboard::copy_to_clipboard(&text) {
                    Ok(()) => tracing::debug!("클립보드 복사 완료: {} chars", text.len()),
                    Err(e) => tracing::warn!("클립보드 복사 실패: {e}"),
                }
            }
            return; // PTY stdin 전송하지 않음
        }

        // 2~3. ANSI encoding → pending_input 버퍼
        if let Some(bytes) = keystroke_to_ansi_bytes(keystroke) {
            self.pending_input.extend_from_slice(&bytes);
            cx.notify();
        }
    }

    /// pending_input 버퍼를 drain 하고 반환한다 (PTY 연결 후 호출).
    ///
    /// T6 에서 PTY stdin writer 연결 시 이 메서드로 버퍼 drain.
    pub fn drain_pending_input(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.pending_input)
    }

    /// Handles mouse click events on terminal surface.
    ///
    /// AC-LK-4: FilePath span click → log OpenCodeViewer action
    /// AC-LK-5: URL span click → dispatch to browser via cx.open_url()
    ///
    /// @MX:NOTE: click-handler-entrypoint
    /// This is the entry point for GPUI click handling. The actual GPUI
    /// on_mouse_down wiring is in the Render impl. This helper contains
    /// the core logic for converting click position to ClickAction.
    pub fn handle_click(&mut self, _row: u16, col: u16, line_text: &str, cx: &mut Context<Self>) {
        // Convert column to byte offset (UTF-8 aware)
        let byte_offset = moai_studio_terminal::link::col_to_byte_offset(line_text, col as usize);

        // Resolve click action using link detection
        if let Some(action) = moai_studio_terminal::link::resolve_click(line_text, byte_offset) {
            match action {
                ClickAction::OpenCodeViewer(moai_studio_terminal::link::OpenCodeViewer {
                    path,
                    line,
                    col,
                }) => {
                    // AC-LK-4 PARTIAL: log the action (file opening deferred to viewer SPEC)
                    tracing::info!(
                        path = ?path,
                        line = ?line,
                        col = ?col,
                        "ClickAction::OpenCodeViewer"
                    );
                }
                ClickAction::OpenUrl(moai_studio_terminal::link::OpenUrl { url }) => {
                    // AC-LK-5: open URL in default browser
                    cx.open_url(&url);
                    tracing::debug!(url = %url, "Opened URL in browser");
                }
                ClickAction::OpenSpec(moai_studio_terminal::link::OpenSpec { spec_id }) => {
                    // B-4 feature: log SPEC ID click (panel opening deferred)
                    tracing::info!(spec_id = %spec_id, "ClickAction::OpenSpec");
                }
            }
        }
    }
}

impl Default for TerminalSurface {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// GPUI Render 구현
// ============================================================

impl Render for TerminalSurface {
    /// TerminalSurface 렌더 — T4 플레이스홀더.
    ///
    /// T5/T6 에서 실제 Grid<Cell> → Glyph 렌더로 교체한다.
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let cursor_info = format!(
            "cursor ({}, {}) | bytes={} | {}",
            self.state.cursor_row,
            self.state.cursor_col,
            self.state.total_bytes,
            self.state.row0_text,
        );

        let mut area = div()
            .size_full()
            .bg(rgb(tok::BG_APP)) // 터미널 배경 — tokens.json theme.dark.background.app
            .flex()
            .flex_col()
            .p_2();

        // 선택 영역 하이라이트 표시 (T6 에서 paint_quad 기반 반투명 렌더로 교체)
        if let Some(sel) = &self.selection {
            let ((r1, c1), (r2, c2)) = sel.bounding_rect();
            let sel_info = format!("sel ({r1},{c1})→({r2},{c2})");
            area = area.child(
                div()
                    .text_xs()
                    .text_color(rgb(ide_accent::CYAN))
                    .bg(rgb(ide_accent::BLUE))
                    .child(sel_info),
            );
        }

        // 커서 위치 + 첫 번째 행 텍스트 표시 (T5 에서 실제 셀 그리드로 교체)
        area.child(
            div()
                .text_sm()
                .text_color(rgb(tok::FG_SECONDARY))
                .child(cursor_info),
        )
        .child(
            // 커서 표시 (blink 는 T5 에서 GPUI timer 기반으로 구현)
            div().w(px(8.)).h(px(16.)).bg(rgb(if self.cursor_visible {
                tok::FG_SECONDARY
            } else {
                tok::BG_APP
            })),
        )
    }
}

// ============================================================
// 유닛 테스트 — TerminalSurface 상태 로직 (GPUI 렌더 제외)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- pixel_to_cell 테스트 (SPEC-V3-002 RG-V3-002-5, plan.md T5 §예상산출) ---

    #[test]
    fn pixel_to_cell_origin_maps_to_row0_col0() {
        let surface = TerminalSurface::new();
        assert_eq!(surface.pixel_to_cell(0.0, 0.0), (0, 0));
    }

    #[test]
    fn pixel_to_cell_within_first_cell_stays_at_0_0() {
        let surface = TerminalSurface::new();
        // advance_width=8, line_height=16 — (7.9, 15.9) 는 여전히 (0, 0)
        assert_eq!(surface.pixel_to_cell(7.9, 15.9), (0, 0));
    }

    #[test]
    fn pixel_to_cell_at_column_boundary_advances_column() {
        let surface = TerminalSurface::new();
        // x=8.0 → col=1 (정확히 경계)
        assert_eq!(surface.pixel_to_cell(8.0, 0.0), (0, 1));
    }

    #[test]
    fn pixel_to_cell_at_row_boundary_advances_row() {
        let surface = TerminalSurface::new();
        // y=16.0 → row=1
        assert_eq!(surface.pixel_to_cell(0.0, 16.0), (1, 0));
    }

    #[test]
    fn pixel_to_cell_multi_cell_mapping() {
        let surface = TerminalSurface::new();
        // (80.0, 160.0) → col=10, row=10
        assert_eq!(surface.pixel_to_cell(80.0, 160.0), (10, 10));
    }

    #[test]
    fn pixel_to_cell_fractional_within_cell() {
        let surface = TerminalSurface::new();
        // (8.5, 16.5) → col=1, row=1 (소수점 버림)
        assert_eq!(surface.pixel_to_cell(8.5, 16.5), (1, 1));
    }

    #[test]
    fn pixel_to_cell_custom_font_metrics() {
        let mut surface = TerminalSurface::new();
        surface.font = FontMetrics {
            advance_width: 10.0,
            line_height: 20.0,
        };
        // (25.0, 45.0) → col=2 (25/10=2.5→2), row=2 (45/20=2.25→2)
        assert_eq!(surface.pixel_to_cell(25.0, 45.0), (2, 2));
    }

    #[test]
    fn pixel_to_cell_negative_x_clamps_to_0() {
        let surface = TerminalSurface::new();
        assert_eq!(surface.pixel_to_cell(-5.0, 0.0), (0, 0));
    }

    #[test]
    fn pixel_to_cell_negative_y_clamps_to_0() {
        let surface = TerminalSurface::new();
        assert_eq!(surface.pixel_to_cell(0.0, -8.0), (0, 0));
    }

    // --- Selection 테스트 ---

    #[test]
    fn selection_bounding_rect_normal_direction() {
        let sel = Selection {
            start: (1, 2),
            end: (3, 5),
        };
        assert_eq!(sel.bounding_rect(), ((1, 2), (3, 5)));
    }

    #[test]
    fn selection_bounding_rect_reverse_direction() {
        // 역방향 드래그 — 정규화 확인
        let sel = Selection {
            start: (3, 5),
            end: (1, 2),
        };
        assert_eq!(sel.bounding_rect(), ((1, 2), (3, 5)));
    }

    #[test]
    fn selection_bounding_rect_same_row_different_col() {
        let sel = Selection {
            start: (2, 5),
            end: (2, 1),
        };
        assert_eq!(sel.bounding_rect(), ((2, 1), (2, 5)));
    }

    #[test]
    fn selection_is_collapsed_when_start_equals_end() {
        let sel = Selection {
            start: (2, 4),
            end: (2, 4),
        };
        assert!(sel.is_collapsed());
        assert_eq!(sel.bounding_rect(), ((2, 4), (2, 4)));
    }

    #[test]
    fn selection_not_collapsed_when_different() {
        let sel = Selection {
            start: (0, 0),
            end: (1, 1),
        };
        assert!(!sel.is_collapsed());
    }

    // --- TerminalSurface 상태 전환 테스트 ---

    #[test]
    fn terminal_surface_initial_state_no_selection() {
        let surface = TerminalSurface::new();
        assert!(surface.selection.is_none());
        assert!(surface.cursor_visible);
        assert_eq!(surface.state.total_bytes, 0);
    }

    #[test]
    fn terminal_surface_begin_selection_creates_collapsed_selection() {
        let mut surface = TerminalSurface::new();
        // (16.0, 32.0) → font 8×16 → col=2, row=2
        surface.begin_selection(16.0, 32.0);
        let sel = surface.selection.as_ref().unwrap();
        assert_eq!(sel.start, (2, 2));
        assert_eq!(sel.end, (2, 2));
        assert!(sel.is_collapsed());
    }

    #[test]
    fn terminal_surface_update_selection_extends_end_cell() {
        let mut surface = TerminalSurface::new();
        surface.begin_selection(0.0, 0.0); // (0, 0)
        surface.update_selection(40.0, 16.0); // col=5, row=1
        let sel = surface.selection.as_ref().unwrap();
        assert_eq!(sel.start, (0, 0));
        assert_eq!(sel.end, (1, 5));
    }

    #[test]
    fn terminal_surface_clear_selection_removes_selection() {
        let mut surface = TerminalSurface::new();
        surface.begin_selection(0.0, 0.0);
        surface.clear_selection();
        assert!(surface.selection.is_none());
    }

    #[test]
    fn terminal_surface_update_without_begin_is_noop() {
        let mut surface = TerminalSurface::new();
        // begin_selection 없이 update → selection 생성 금지
        surface.update_selection(100.0, 100.0);
        assert!(surface.selection.is_none());
    }

    #[test]
    fn terminal_surface_end_selection_finalizes() {
        let mut surface = TerminalSurface::new();
        surface.begin_selection(0.0, 0.0);
        surface.end_selection(24.0, 32.0); // col=3, row=2
        let sel = surface.selection.as_ref().unwrap();
        assert_eq!(sel.start, (0, 0));
        assert_eq!(sel.end, (2, 3));
    }

    #[test]
    fn terminal_surface_default_font_metrics_are_8x16() {
        let surface = TerminalSurface::new();
        assert_eq!(surface.font.advance_width, 8.0);
        assert_eq!(surface.font.line_height, 16.0);
    }

    #[test]
    fn terminal_surface_set_font_metrics_updates_pixel_mapping() {
        let mut surface = TerminalSurface::new();
        surface.set_font_metrics(FontMetrics {
            advance_width: 12.0,
            line_height: 24.0,
        });
        // (24.0, 48.0) → col=2, row=2
        assert_eq!(surface.pixel_to_cell(24.0, 48.0), (2, 2));
    }

    // --- AC-LK-4/5: Click handler 테스트 ---

    #[test]
    fn handle_click_on_file_path_returns_action() {
        // Test that clicking on a file path resolves to OpenCodeViewer
        let line_text = "error at src/main.rs:42:10 here";
        let col = line_text.find("main").unwrap() as u16;

        // Verify link detection works
        let byte_offset = moai_studio_terminal::link::col_to_byte_offset(line_text, col as usize);
        let action = moai_studio_terminal::link::resolve_click(line_text, byte_offset);

        assert!(action.is_some(), "Should resolve click on file path");
        match action.unwrap() {
            ClickAction::OpenCodeViewer(moai_studio_terminal::link::OpenCodeViewer {
                path,
                line,
                col,
            }) => {
                assert_eq!(path, std::path::PathBuf::from("src/main.rs"));
                assert_eq!(line, Some(42));
                assert_eq!(col, Some(10));
            }
            other => panic!("Expected OpenCodeViewer, got {:?}", other),
        }
    }

    #[test]
    fn handle_click_on_url_returns_action() {
        // Test that clicking on a URL resolves to OpenUrl
        let line_text = "see https://example.com/foo for details";
        let col = line_text.find("example").unwrap() as u16;

        // Verify link detection works
        let byte_offset = moai_studio_terminal::link::col_to_byte_offset(line_text, col as usize);
        let action = moai_studio_terminal::link::resolve_click(line_text, byte_offset);

        assert!(action.is_some(), "Should resolve click on URL");
        match action.unwrap() {
            ClickAction::OpenUrl(moai_studio_terminal::link::OpenUrl { url }) => {
                assert_eq!(url, "https://example.com/foo");
            }
            other => panic!("Expected OpenUrl, got {:?}", other),
        }
    }

    #[test]
    fn handle_click_on_spec_id_returns_action() {
        // Test that clicking on a SPEC-ID resolves to OpenSpec
        let line_text = "Working on SPEC-V3-001 today";
        let col = line_text.find("V3").unwrap() as u16;

        // Verify link detection works
        let byte_offset = moai_studio_terminal::link::col_to_byte_offset(line_text, col as usize);
        let action = moai_studio_terminal::link::resolve_click(line_text, byte_offset);

        assert!(action.is_some(), "Should resolve click on SPEC-ID");
        match action.unwrap() {
            ClickAction::OpenSpec(moai_studio_terminal::link::OpenSpec { spec_id }) => {
                assert_eq!(spec_id, "SPEC-V3-001");
            }
            other => panic!("Expected OpenSpec, got {:?}", other),
        }
    }

    #[test]
    fn handle_click_on_plain_text_returns_none() {
        // Test that clicking on plain text returns no action
        let line_text = "plain text no links here";
        let col = 5;

        // Verify link detection returns None
        let byte_offset = moai_studio_terminal::link::col_to_byte_offset(line_text, col as usize);
        let action = moai_studio_terminal::link::resolve_click(line_text, byte_offset);

        assert!(action.is_none(), "Should not resolve click on plain text");
    }

    #[test]
    fn col_to_byte_offset_handles_utf8_correctly() {
        // Test UTF-8 byte offset calculation (Korean text)
        let line_text = "에러 src/main.rs:42";
        // '에' is 3 bytes, '러' is 3 bytes, space is 1 byte = 7 bytes
        // "src/main.rs" starts at byte offset 7, which is character index 3
        let col = 3; // Character index 3 (after "에러 ") is 's' in "src"

        let byte_offset = moai_studio_terminal::link::col_to_byte_offset(line_text, col);
        assert_eq!(
            byte_offset, 7,
            "UTF-8 byte offset should be calculated correctly"
        );
    }
}
