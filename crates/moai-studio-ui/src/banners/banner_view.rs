//! BannerView — 개별 배너 UI (SPEC-V3-014 MS-1).
//!
//! REQ-V14-006 ~ REQ-V14-010 구현.
//! 36px height, full-width, severity bg color, icon + body + actions + dismiss × 레이아웃.

use gpui::{
    InteractiveElement, IntoElement, MouseButton, ParentElement, Render, Styled, Window, div, px,
    rgb,
};

use crate::banners::{
    ActionButton, BANNER_ELEMENT_GAP_PX, BANNER_HEIGHT_PX, BANNER_ICON_SIZE_PX,
    BANNER_PADDING_X_PX, BannerData, BannerId, Severity,
};
use crate::design::tokens;

// ============================================================
// severity → 색상 매핑 (REQ-V14-008)
// ============================================================

/// severity 에 대응하는 배경색 hex — semantic 토큰 참조 (REQ-V14-008).
pub fn severity_bg_color(severity: Severity) -> u32 {
    use crate::design::tokens::semantic;
    match severity {
        Severity::Critical | Severity::Error => semantic::DANGER,
        Severity::Warning => semantic::WARNING,
        Severity::Info => semantic::INFO,
        Severity::Success => semantic::SUCCESS,
    }
}

/// severity 에 대응하는 아이콘 텍스트 (placeholder — MS-1 에서는 ASCII/emoji 사용).
pub fn severity_icon(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical => "⚠",
        Severity::Error => "✖",
        Severity::Warning => "⚠",
        Severity::Info => "ℹ",
        Severity::Success => "✓",
    }
}

// ============================================================
// BannerViewState — BannerView 를 위한 순수 데이터 뷰 모델
// ============================================================

/// BannerView 렌더를 위한 뷰 모델 — GPUI Entity 없이 테스트 가능한 순수 구조체.
pub struct BannerViewState {
    /// 배너 고유 id
    pub id: BannerId,
    /// 심각도
    pub severity: Severity,
    /// 주 메시지
    pub message: String,
    /// 보조 메시지 (옵션)
    pub meta: Option<String>,
    /// 액션 버튼 목록
    pub actions: Vec<ActionButton>,
}

impl BannerViewState {
    /// BannerData 로부터 BannerViewState 생성.
    pub fn from_data(data: &BannerData) -> Self {
        Self {
            id: data.id.clone(),
            severity: data.severity,
            message: data.message.clone(),
            meta: data.meta.clone(),
            actions: data.actions.clone(),
        }
    }
}

// ============================================================
// BannerView — GPUI Render 구현
// ============================================================

/// 개별 배너 UI 컴포넌트 (REQ-V14-006 ~ REQ-V14-010).
///
/// 레이아웃: [icon] [body: message + meta] [actions...] [× dismiss]
/// 높이: 36px (BANNER_HEIGHT_PX), 전체 너비, border-radius 없음.
pub struct BannerView {
    /// 뷰 상태
    pub state: BannerViewState,
    /// dismiss 핸들러 — dismiss × 버튼 클릭 시 호출 (banner id 전달).
    /// MS-1 에서는 None 허용 (mock). MS-3 에서 BannerStack Entity 와 연결.
    #[allow(clippy::type_complexity)]
    pub on_dismiss: Option<Box<dyn Fn(&BannerId) + Send + Sync>>,
}

impl BannerView {
    /// 새 BannerView 생성.
    pub fn new(state: BannerViewState) -> Self {
        Self {
            state,
            on_dismiss: None,
        }
    }

    /// BannerData 로부터 BannerView 생성 (편의 메서드).
    pub fn from_data(data: &BannerData) -> Self {
        Self::new(BannerViewState::from_data(data))
    }

    /// dismiss 핸들러 설정.
    pub fn with_dismiss(mut self, handler: impl Fn(&BannerId) + Send + Sync + 'static) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }

    // ── 렌더 헬퍼 — 테스트 가능한 순수 메서드 ──

    /// 배경색 반환 (severity → semantic token).
    pub fn bg_color(&self) -> u32 {
        severity_bg_color(self.state.severity)
    }

    /// 아이콘 텍스트 반환.
    pub fn icon_text(&self) -> &'static str {
        severity_icon(self.state.severity)
    }

    /// 액션 버튼 수.
    pub fn action_count(&self) -> usize {
        self.state.actions.len()
    }

    /// primary action button 색상 (brand.primary 기준, MS-1 에서는 다크 모드 기본).
    pub fn primary_action_color(&self) -> u32 {
        tokens::brand::PRIMARY_DARK
    }
}

impl Render for BannerView {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let bg = self.bg_color();
        let icon = self.icon_text();
        let message = self.state.message.clone();
        let meta = self.state.meta.clone();
        let actions = self.state.actions.clone();
        let banner_id = self.state.id.clone();

        // 루트 배너 행
        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .h(px(BANNER_HEIGHT_PX))
            .px(px(BANNER_PADDING_X_PX))
            .gap(px(BANNER_ELEMENT_GAP_PX))
            .bg(rgb(bg));

        // [icon] 16×16
        row = row.child(
            div()
                .w(px(BANNER_ICON_SIZE_PX))
                .h(px(BANNER_ICON_SIZE_PX))
                .flex()
                .items_center()
                .justify_center()
                .text_xs()
                .text_color(rgb(tokens::FG_PRIMARY))
                .child(icon),
        );

        // [body: message + optional meta]
        let mut body = div().flex().flex_col().flex_grow().child(
            div()
                .text_sm()
                .text_color(rgb(tokens::FG_PRIMARY))
                .child(message),
        );
        if let Some(m) = meta {
            body = body.child(div().text_xs().text_color(rgb(tokens::FG_MUTED)).child(m));
        }
        row = row.child(body);

        // [actions]
        for action in &actions {
            let action_id_str = action.action_id.clone();
            let label = action.label.clone();
            let is_primary = action.primary;
            let fg_color = if is_primary {
                tokens::theme::dark::text::ON_PRIMARY
            } else {
                tokens::FG_PRIMARY
            };
            let bg_color = if is_primary {
                tokens::brand::PRIMARY_DARK
            } else {
                tokens::BG_ELEVATED
            };
            row = row.child(
                div()
                    .id(gpui::SharedString::from(format!(
                        "banner-action-{}-{}",
                        banner_id.as_str(),
                        &action_id_str
                    )))
                    .px(px(8.0))
                    .py(px(4.0))
                    .rounded(px(4.0))
                    .bg(rgb(bg_color))
                    .text_xs()
                    .text_color(rgb(fg_color))
                    .cursor_pointer()
                    .on_mouse_down(MouseButton::Left, {
                        let id_str = action_id_str.clone();
                        cx.listener(move |_this, _ev, _window, _cx| {
                            tracing::info!("banner action: {id_str}");
                        })
                    })
                    .child(label),
            );
        }

        // [× dismiss 버튼]
        let dismiss_id = banner_id.clone();
        row = row.child(
            div()
                .id(gpui::SharedString::from(format!(
                    "banner-dismiss-{}",
                    banner_id.as_str()
                )))
                .w(px(20.0))
                .h(px(20.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(4.0))
                .text_xs()
                .text_color(rgb(tokens::FG_MUTED))
                .cursor_pointer()
                .hover(|s: gpui::StyleRefinement| s.text_color(rgb(tokens::FG_PRIMARY)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _ev, _window, _cx| {
                        if let Some(ref handler) = this.on_dismiss {
                            handler(&dismiss_id);
                        }
                    }),
                )
                .child("×"),
        );

        row
    }
}

// ============================================================
// 단위 테스트 — BannerView 렌더 계약 (AC-V14-10)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banners::{ActionButton, BannerData, BannerId, Severity};

    fn make_view_state(
        severity: Severity,
        actions: Vec<ActionButton>,
        meta: Option<String>,
    ) -> BannerViewState {
        BannerViewState {
            id: BannerId::new("test:banner"),
            severity,
            message: "Test banner message".to_string(),
            meta,
            actions,
        }
    }

    // ── severity_bg_color (REQ-V14-008) ──

    #[test]
    fn severity_bg_critical_is_danger() {
        assert_eq!(
            severity_bg_color(Severity::Critical),
            crate::design::tokens::semantic::DANGER
        );
    }

    #[test]
    fn severity_bg_error_is_danger() {
        assert_eq!(
            severity_bg_color(Severity::Error),
            crate::design::tokens::semantic::DANGER
        );
    }

    #[test]
    fn severity_bg_warning_is_warning_token() {
        assert_eq!(
            severity_bg_color(Severity::Warning),
            crate::design::tokens::semantic::WARNING
        );
    }

    #[test]
    fn severity_bg_info_is_info_token() {
        assert_eq!(
            severity_bg_color(Severity::Info),
            crate::design::tokens::semantic::INFO
        );
    }

    #[test]
    fn severity_bg_success_is_success_token() {
        assert_eq!(
            severity_bg_color(Severity::Success),
            crate::design::tokens::semantic::SUCCESS
        );
    }

    // ── BannerView 생성 / 속성 ──

    #[test]
    fn banner_view_bg_color_matches_severity() {
        let view = BannerView::new(make_view_state(Severity::Critical, vec![], None));
        assert_eq!(view.bg_color(), crate::design::tokens::semantic::DANGER);
    }

    #[test]
    fn banner_view_icon_text_critical() {
        let view = BannerView::new(make_view_state(Severity::Critical, vec![], None));
        assert_eq!(view.icon_text(), "⚠");
    }

    #[test]
    fn banner_view_icon_text_success() {
        let view = BannerView::new(make_view_state(Severity::Success, vec![], None));
        assert_eq!(view.icon_text(), "✓");
    }

    #[test]
    fn banner_view_icon_text_info() {
        let view = BannerView::new(make_view_state(Severity::Info, vec![], None));
        assert_eq!(view.icon_text(), "ℹ");
    }

    /// 0개 action 배너.
    #[test]
    fn banner_view_zero_actions() {
        let view = BannerView::new(make_view_state(Severity::Info, vec![], None));
        assert_eq!(view.action_count(), 0);
    }

    /// 1개 action 배너.
    #[test]
    fn banner_view_one_action() {
        let view = BannerView::new(make_view_state(
            Severity::Warning,
            vec![ActionButton::new("Configure", "lsp:configure", true)],
            None,
        ));
        assert_eq!(view.action_count(), 1);
    }

    /// 2개 action 배너 (표준 패턴).
    #[test]
    fn banner_view_two_actions() {
        let view = BannerView::new(make_view_state(
            Severity::Critical,
            vec![
                ActionButton::new("Reopen", "crash:reopen", true),
                ActionButton::new("Dismiss", "crash:dismiss", false),
            ],
            None,
        ));
        assert_eq!(view.action_count(), 2, "배너는 2개 액션을 가져야 함");
        assert_eq!(view.state.actions[0].label, "Reopen");
        assert!(view.state.actions[0].primary);
        assert_eq!(view.state.actions[1].label, "Dismiss");
        assert!(!view.state.actions[1].primary);
    }

    /// from_data 편의 생성자.
    #[test]
    fn banner_view_from_data_preserves_fields() {
        let data = BannerData::new(
            BannerId::new("lsp:rust-analyzer"),
            Severity::Warning,
            "rust-analyzer failed to start".to_string(),
            Some("spawn error: not found".to_string()),
            vec![ActionButton::new("Configure", "lsp:configure", true)],
        );
        let view = BannerView::from_data(&data);
        assert_eq!(view.state.severity, Severity::Warning);
        assert_eq!(view.state.message, "rust-analyzer failed to start");
        assert_eq!(view.state.meta, Some("spawn error: not found".to_string()));
        assert_eq!(view.action_count(), 1);
    }

    /// primary_action_color 는 brand::PRIMARY_DARK 이다.
    #[test]
    fn banner_view_primary_action_color() {
        let view = BannerView::new(make_view_state(Severity::Info, vec![], None));
        assert_eq!(
            view.primary_action_color(),
            crate::design::tokens::brand::PRIMARY_DARK
        );
    }

    // ── BANNER_HEIGHT_PX / BANNER_ICON_SIZE_PX / constants ──

    #[test]
    fn banner_height_is_36() {
        assert!((BANNER_HEIGHT_PX - 36.0).abs() < f32::EPSILON);
    }

    #[test]
    fn banner_icon_size_is_16() {
        assert!((BANNER_ICON_SIZE_PX - 16.0).abs() < f32::EPSILON);
    }

    #[test]
    fn banner_padding_x_is_12() {
        assert!((BANNER_PADDING_X_PX - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn banner_element_gap_is_8() {
        assert!((BANNER_ELEMENT_GAP_PX - 8.0).abs() < f32::EPSILON);
    }
}
