//! PaletteView Entity — palette input + list + 키보드 nav state machine.
//!
//! @MX:ANCHOR: [AUTO] PaletteView — fan_in 대상 (3 variant 가 공유).
//! @MX:REASON: [AUTO] 키보드 nav state machine + 레이아웃 계약 (AC-PL-3~5).
//! @MX:SPEC: SPEC-V3-012

// ============================================================
// PaletteView 레이아웃 상수
// (tokens.json round2_component.palette.container / row / input / list)
// ============================================================

/// Palette container 너비 (tokens.json: `round2_component.palette.container.width_px` = 600).
pub const PALETTE_WIDTH: f32 = 600.0;

/// 행 높이 (tokens.json: `round2_component.palette.row.height_px` = 32).
pub const ROW_HEIGHT: f32 = 32.0;

/// Input 폰트 크기 (tokens.json: `round2_component.palette.input.font_size_px` = 14).
pub const INPUT_FONT_SIZE: f32 = 14.0;

/// 목록 최대 높이 (tokens.json: `round2_component.palette.list.max_height_px` = 320).
pub const LIST_MAX_HEIGHT: f32 = 320.0;

/// Highlight accent alpha (tokens.json: `round2_component.palette.highlight.alpha` = 0.20).
pub const HIGHLIGHT_ALPHA: f32 = 0.20;

// ============================================================
// PaletteItem — 목록 항목
// ============================================================

/// PaletteView 목록 항목.
///
/// MS-1 단계에서는 간단한 레이블만 보유. MS-2 에서 variant-specific 필드 추가.
#[derive(Debug, Clone, PartialEq)]
pub struct PaletteItem {
    /// 목록에 표시되는 레이블.
    pub label: String,
    /// 항목 식별자 (이벤트 페이로드로 사용).
    pub id: String,
}

impl PaletteItem {
    /// 새 PaletteItem 을 생성한다.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

// ============================================================
// PaletteEvent — PaletteView 이벤트
// ============================================================

/// PaletteView 에서 발생하는 이벤트.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteEvent {
    /// 항목 선택 확정 — Enter 키 (AC-PL-4).
    ItemSelected(PaletteItem),
    /// Palette 닫기 요청 — Esc 키 (AC-PL-4).
    DismissRequested,
}

// ============================================================
// NavState — 키보드 nav state machine
// ============================================================

/// PaletteView 키보드 nav state.
///
/// items 가 비어있으면 selected_index 는 None.
/// items 가 비어있지 않으면 selected_index 는 0..items.len() 범위.
#[derive(Debug, Clone)]
pub struct NavState {
    /// 현재 선택된 인덱스.
    pub selected_index: Option<usize>,
    /// 표시 중인 항목 수.
    pub item_count: usize,
}

impl NavState {
    /// 새 NavState 를 생성한다.
    pub fn new(item_count: usize) -> Self {
        let selected_index = if item_count > 0 { Some(0) } else { None };
        Self {
            selected_index,
            item_count,
        }
    }

    /// 항목 수가 변경되었을 때 state 를 갱신한다.
    pub fn update_item_count(&mut self, item_count: usize) {
        self.item_count = item_count;
        if item_count == 0 {
            self.selected_index = None;
        } else {
            // 기존 index 가 유효하면 유지, 아니면 0 으로 리셋.
            self.selected_index = Some(
                self.selected_index
                    .map(|i| i.min(item_count - 1))
                    .unwrap_or(0),
            );
        }
    }

    /// ArrowDown: 다음 항목으로 이동 (끝에서 처음으로 wrap) (AC-PL-4 RG-PL-8).
    pub fn move_down(&mut self) {
        if self.item_count == 0 {
            return;
        }
        self.selected_index = Some(match self.selected_index {
            None => 0,
            Some(i) => (i + 1) % self.item_count,
        });
    }

    /// ArrowUp: 이전 항목으로 이동 (처음에서 끝으로 wrap) (AC-PL-4 RG-PL-9).
    pub fn move_up(&mut self) {
        if self.item_count == 0 {
            return;
        }
        self.selected_index = Some(match self.selected_index {
            None => 0,
            Some(0) => self.item_count - 1,
            Some(i) => i - 1,
        });
    }
}

// ============================================================
// FocusState — input 포커스 추적
// ============================================================

/// PaletteView input 필드 포커스 상태.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    /// input 포커스 없음.
    Unfocused,
    /// input 에 포커스 있음 — 텍스트 입력 라우팅 활성.
    Focused,
}

// ============================================================
// PaletteView — 키보드 nav + 레이아웃 state machine
// ============================================================

/// PaletteView — variant-agnostic palette core state machine.
///
/// - container width: PALETTE_WIDTH (600px)
/// - row height: ROW_HEIGHT (32px)
/// - input font-size: INPUT_FONT_SIZE (14px)
/// - list max-height: LIST_MAX_HEIGHT (320px)
/// - input 포커스: 열릴 때 자동 focus (AC-PL-5)
/// - 키보드: ↑↓ navigate, Enter select, Esc dismiss (AC-PL-4)
#[derive(Debug)]
pub struct PaletteView {
    /// 현재 입력 텍스트.
    pub query: String,
    /// 표시 중인 항목 목록.
    pub items: Vec<PaletteItem>,
    /// 키보드 nav state.
    pub nav: NavState,
    /// input 포커스 상태.
    pub focus: FocusState,
    /// 마지막으로 발생한 이벤트 (테스트 검증용).
    pub last_event: Option<PaletteEvent>,
}

impl PaletteView {
    /// 새 PaletteView 를 생성한다.
    ///
    /// 생성 시 input 이 자동으로 focused 상태가 된다 (AC-PL-5).
    pub fn new() -> Self {
        Self {
            query: String::new(),
            items: Vec::new(),
            nav: NavState::new(0),
            focus: FocusState::Focused, // AC-PL-5: 열릴 때 input focus
            last_event: None,
        }
    }

    /// 초기 항목을 설정하여 PaletteView 를 생성한다.
    pub fn with_items(items: Vec<PaletteItem>) -> Self {
        let item_count = items.len();
        Self {
            query: String::new(),
            items,
            nav: NavState::new(item_count),
            focus: FocusState::Focused,
            last_event: None,
        }
    }

    /// container width 를 반환한다 (AC-PL-3).
    pub fn container_width(&self) -> f32 {
        PALETTE_WIDTH
    }

    /// row height 를 반환한다 (AC-PL-3).
    pub fn row_height(&self) -> f32 {
        ROW_HEIGHT
    }

    /// input font-size 를 반환한다 (AC-PL-3).
    pub fn input_font_size(&self) -> f32 {
        INPUT_FONT_SIZE
    }

    /// list max-height 를 반환한다 (AC-PL-3).
    pub fn list_max_height(&self) -> f32 {
        LIST_MAX_HEIGHT
    }

    /// 현재 선택된 항목을 반환한다.
    pub fn selected_item(&self) -> Option<&PaletteItem> {
        self.nav.selected_index.and_then(|i| self.items.get(i))
    }

    /// ArrowDown 키 처리 — 다음 항목으로 이동 (AC-PL-4).
    pub fn on_arrow_down(&mut self) {
        self.nav.move_down();
    }

    /// ArrowUp 키 처리 — 이전 항목으로 이동 (AC-PL-4).
    pub fn on_arrow_up(&mut self) {
        self.nav.move_up();
    }

    /// Enter 키 처리 — 선택된 항목 확정, ItemSelected 이벤트 발생 (AC-PL-4).
    ///
    /// 반환값: 이벤트 발생 여부 (선택된 항목이 없으면 None).
    pub fn on_enter(&mut self) -> Option<PaletteEvent> {
        let event = self
            .selected_item()
            .cloned()
            .map(PaletteEvent::ItemSelected);
        self.last_event = event.clone();
        event
    }

    /// Esc 키 처리 — DismissRequested 이벤트 발생 (AC-PL-4).
    pub fn on_escape(&mut self) -> PaletteEvent {
        let event = PaletteEvent::DismissRequested;
        self.last_event = Some(event.clone());
        event
    }

    /// query 를 갱신하고 항목 목록을 업데이트한다.
    pub fn set_query(&mut self, query: String) {
        self.query = query;
    }

    /// 항목 목록을 교체하고 nav state 를 갱신한다.
    pub fn set_items(&mut self, items: Vec<PaletteItem>) {
        let count = items.len();
        self.items = items;
        self.nav.update_item_count(count);
    }

    /// input 포커스 여부를 반환한다 (AC-PL-5).
    pub fn is_input_focused(&self) -> bool {
        self.focus == FocusState::Focused
    }
}

impl Default for PaletteView {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — AC-PL-3, AC-PL-4, AC-PL-5
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_items(n: usize) -> Vec<PaletteItem> {
        (0..n)
            .map(|i| PaletteItem::new(format!("id-{i}"), format!("item {i}")))
            .collect()
    }

    // ----------------------------------------------------------
    // AC-PL-3: 레이아웃 치수 검증
    // ----------------------------------------------------------

    /// AC-PL-3: PaletteView 치수가 SPEC 명세값과 일치한다 (1px tolerance).
    #[test]
    fn dimensions_match_spec() {
        let view = PaletteView::new();
        assert!(
            (view.container_width() - 600.0).abs() <= 1.0,
            "container width 불일치: {}",
            view.container_width()
        );
        assert!(
            (view.row_height() - 32.0).abs() <= 1.0,
            "row height 불일치: {}",
            view.row_height()
        );
        assert!(
            (view.input_font_size() - 14.0).abs() <= 1.0,
            "input font-size 불일치: {}",
            view.input_font_size()
        );
        assert!(
            (view.list_max_height() - 320.0).abs() <= 1.0,
            "list max-height 불일치: {}",
            view.list_max_height()
        );
    }

    // ----------------------------------------------------------
    // AC-PL-4: 키보드 nav state machine 검증
    // ----------------------------------------------------------

    /// AC-PL-4: ArrowDown N+1 회 누르면 선택이 index 0 으로 wrap.
    #[test]
    fn nav_wraps_down() {
        let mut view = PaletteView::with_items(make_items(3));
        // 0 → 1 → 2 → 0 (wrap)
        view.on_arrow_down(); // 1
        view.on_arrow_down(); // 2
        view.on_arrow_down(); // 0 (wrap)
        assert_eq!(
            view.nav.selected_index,
            Some(0),
            "N+1 down 후 index 0 이어야 함"
        );
    }

    /// AC-PL-4: index 0 에서 ArrowUp 누르면 선택이 N-1 로 wrap.
    #[test]
    fn nav_wraps_up() {
        let mut view = PaletteView::with_items(make_items(3));
        // 초기 index = 0
        view.on_arrow_up(); // N-1 = 2 (wrap)
        assert_eq!(
            view.nav.selected_index,
            Some(2),
            "index 0에서 up 후 N-1 이어야 함"
        );
    }

    /// AC-PL-4: Enter 키 시 선택된 항목의 ItemSelected 이벤트가 발생한다.
    #[test]
    fn enter_emits_selected() {
        let mut view = PaletteView::with_items(make_items(3));
        view.on_arrow_down(); // index 1
        let event = view.on_enter();
        assert_eq!(
            event,
            Some(PaletteEvent::ItemSelected(PaletteItem::new(
                "id-1", "item 1"
            ))),
            "Enter 이벤트 불일치"
        );
    }

    /// AC-PL-4: Esc 키 시 DismissRequested 이벤트가 발생한다.
    #[test]
    fn escape_emits_dismiss() {
        let mut view = PaletteView::with_items(make_items(3));
        let event = view.on_escape();
        assert_eq!(event, PaletteEvent::DismissRequested);
    }

    // ----------------------------------------------------------
    // AC-PL-5: input 포커스 관리 검증
    // ----------------------------------------------------------

    /// AC-PL-5: 새로 생성된 PaletteView 의 input 은 focused 상태이다.
    #[test]
    fn input_focused_on_open() {
        let view = PaletteView::new();
        assert!(
            view.is_input_focused(),
            "PaletteView 생성 즉시 input 이 focused 상태이어야 함 (AC-PL-5)"
        );
    }

    // ----------------------------------------------------------
    // 추가 nav 엣지 케이스
    // ----------------------------------------------------------

    /// 항목이 없을 때 nav 동작이 안전하다.
    #[test]
    fn nav_empty_items_is_safe() {
        let mut view = PaletteView::new();
        view.on_arrow_down();
        view.on_arrow_up();
        assert_eq!(view.nav.selected_index, None);
    }

    /// 항목이 1개일 때 wrap 이 자기 자신으로 돌아온다.
    #[test]
    fn nav_single_item_wraps_to_self() {
        let mut view = PaletteView::with_items(make_items(1));
        view.on_arrow_down();
        assert_eq!(view.nav.selected_index, Some(0));
        view.on_arrow_up();
        assert_eq!(view.nav.selected_index, Some(0));
    }

    /// 선택 항목 없을 때 Enter 는 None 을 반환한다.
    #[test]
    fn enter_no_selection_returns_none() {
        let mut view = PaletteView::new(); // items 비어있음
        let event = view.on_enter();
        assert_eq!(event, None);
    }

    /// set_items 로 항목 목록을 교체하면 nav index 가 초기화된다.
    #[test]
    fn set_items_resets_nav() {
        let mut view = PaletteView::with_items(make_items(5));
        view.on_arrow_down();
        view.on_arrow_down(); // index 2
        view.set_items(make_items(2));
        assert_eq!(view.nav.item_count, 2);
        assert!(
            view.nav.selected_index.is_some_and(|i| i < 2),
            "set_items 후 index 가 새 범위 안이어야 함"
        );
    }

    /// query 갱신이 동작한다.
    #[test]
    fn set_query_updates_query() {
        let mut view = PaletteView::new();
        view.set_query("hello".to_string());
        assert_eq!(view.query, "hello");
    }

    /// ArrowDown 연속으로 전체 순환이 가능하다.
    #[test]
    fn nav_full_cycle() {
        let n = 4;
        let mut view = PaletteView::with_items(make_items(n));
        for expected in 1..n {
            view.on_arrow_down();
            assert_eq!(view.nav.selected_index, Some(expected));
        }
        view.on_arrow_down(); // wrap back to 0
        assert_eq!(view.nav.selected_index, Some(0));
    }

    /// 상수 값이 SPEC 명세와 일치한다.
    #[test]
    fn constants_match_spec() {
        assert!((PALETTE_WIDTH - 600.0).abs() < f32::EPSILON);
        assert!((ROW_HEIGHT - 32.0).abs() < f32::EPSILON);
        assert!((INPUT_FONT_SIZE - 14.0).abs() < f32::EPSILON);
        assert!((LIST_MAX_HEIGHT - 320.0).abs() < f32::EPSILON);
        assert!((HIGHLIGHT_ALPHA - 0.20).abs() < f32::EPSILON);
    }
}
