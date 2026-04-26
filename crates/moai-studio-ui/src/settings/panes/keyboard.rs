//! KeyboardPane — 키보드 단축키 바인딩 테이블 + 편집 다이얼로그 + 충돌 검출.
//!
//! SPEC-V3-013 MS-2: AC-V13-7 ~ AC-V13-8 구현.
//! MS-2 단계: in-memory 상태 관리. RootView keymap rebuild + 영속화는 MS-3.

use crate::settings::settings_state::{EditDialogState, KeyBinding, KeyboardState};

// ============================================================
// KeyboardPane 레이아웃 상수
// ============================================================

/// 편집 다이얼로그 너비 (400px) — REQ-V13-031.
pub const EDIT_DIALOG_WIDTH: f32 = 400.0;
/// 편집 다이얼로그 높이 (200px) — REQ-V13-031.
pub const EDIT_DIALOG_HEIGHT: f32 = 200.0;

// ============================================================
// KeyboardPane
// ============================================================

/// KeyboardPane — 바인딩 테이블 + 편집 다이얼로그 보유.
///
/// @MX:ANCHOR: [AUTO] keyboard-pane-struct
/// @MX:REASON: [AUTO] SPEC-V3-013 MS-2. AC-V13-7/AC-V13-8 진입점.
///   fan_in >= 3: settings_modal.rs (section routing), MS-3 lib.rs (keymap rebuild), test suite.
pub struct KeyboardPane {
    /// KeyboardPane 이 소유하는 in-memory 상태.
    pub state: KeyboardState,
}

impl KeyboardPane {
    /// 기본 바인딩으로 새 KeyboardPane 을 생성한다.
    pub fn new() -> Self {
        Self {
            state: KeyboardState::new(),
        }
    }

    /// 지정 상태로 KeyboardPane 을 생성한다 (테스트 편의).
    pub fn with_state(state: KeyboardState) -> Self {
        Self { state }
    }

    // ---- section 메타데이터 ----

    /// section 타이틀.
    pub fn title() -> &'static str {
        "Keyboard"
    }

    /// section 설명.
    pub fn description() -> &'static str {
        "키보드 단축키를 설정합니다. Edit 버튼으로 단축키를 변경하고 충돌을 확인합니다."
    }

    // ---- 바인딩 테이블 조회 ----

    /// 현재 모든 바인딩을 슬라이스로 반환한다 (AC-V13-7).
    pub fn bindings(&self) -> &[KeyBinding] {
        &self.state.bindings
    }

    /// 바인딩 개수를 반환한다.
    pub fn binding_count(&self) -> usize {
        self.state.bindings.len()
    }

    // ---- 편집 다이얼로그 ----

    /// 지정 action_id 의 편집 다이얼로그를 연다 (REQ-V13-031).
    pub fn open_edit(&mut self, action_id: &str) {
        self.state.open_edit_dialog(action_id);
    }

    /// 편집 다이얼로그를 닫는다 (REQ-V13-035).
    pub fn cancel_edit(&mut self) {
        self.state.close_edit_dialog();
    }

    /// 편집 다이얼로그가 열려 있는지 여부를 반환한다.
    pub fn is_edit_open(&self) -> bool {
        self.state.edit_dialog.is_some()
    }

    /// 편집 다이얼로그 상태에 대한 불변 참조를 반환한다.
    pub fn edit_dialog(&self) -> Option<&EditDialogState> {
        self.state.edit_dialog.as_ref()
    }

    /// 편집 다이얼로그 상태에 대한 가변 참조를 반환한다.
    pub fn edit_dialog_mut(&mut self) -> Option<&mut EditDialogState> {
        self.state.edit_dialog.as_mut()
    }

    // ---- 충돌 검사 + 저장 ----

    /// 충돌 검사를 수행한다 (REQ-V13-032).
    /// 충돌 없으면 None, 충돌 있으면 Some(충돌_레이블) 반환.
    pub fn conflict_check(&self, action_id: &str, new_shortcut: &str) -> Option<String> {
        self.state.conflict_check(action_id, new_shortcut)
    }

    /// 편집 다이얼로그의 pending_shortcut 을 저장 시도 (REQ-V13-033 / REQ-V13-034).
    /// 성공 시 true, 충돌 시 false (dialog 에 오류 유지).
    pub fn save_edit(&mut self) -> bool {
        self.state.save_edit_dialog()
    }

    /// 편집 다이얼로그 레이아웃 상수 — 너비.
    pub fn dialog_width() -> f32 {
        EDIT_DIALOG_WIDTH
    }

    /// 편집 다이얼로그 레이아웃 상수 — 높이.
    pub fn dialog_height() -> f32 {
        EDIT_DIALOG_HEIGHT
    }
}

impl Default for KeyboardPane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — RED-GREEN phase (SPEC-V3-013 MS-2 KeyboardPane)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 바인딩 테이블 tests ----

    #[test]
    /// KeyboardPane 기본 바인딩이 10개 이상 존재한다 (AC-V13-7).
    fn keyboard_pane_default_has_ten_or_more_bindings() {
        let pane = KeyboardPane::new();
        assert!(
            pane.binding_count() >= 10,
            "기본 바인딩 10개 이상 필요, 실제: {}",
            pane.binding_count()
        );
    }

    #[test]
    /// bindings() 가 전체 바인딩 슬라이스를 반환한다.
    fn keyboard_pane_bindings_returns_all() {
        let pane = KeyboardPane::new();
        assert_eq!(pane.bindings().len(), pane.binding_count());
    }

    #[test]
    /// 기본 바인딩에 "open.command_palette" 가 포함된다.
    fn keyboard_pane_has_command_palette_binding() {
        let pane = KeyboardPane::new();
        let found = pane
            .bindings()
            .iter()
            .any(|b| b.action_id == "open.command_palette");
        assert!(found, "open.command_palette 바인딩이 있어야 함");
    }

    #[test]
    /// 기본 바인딩에 "open.settings" 가 포함된다.
    fn keyboard_pane_has_open_settings_binding() {
        let pane = KeyboardPane::new();
        let found = pane
            .bindings()
            .iter()
            .any(|b| b.action_id == "open.settings");
        assert!(found, "open.settings 바인딩이 있어야 함");
    }

    #[test]
    /// 편집 다이얼로그 너비가 400px 이다 (REQ-V13-031).
    fn edit_dialog_width_is_400() {
        assert!((KeyboardPane::dialog_width() - 400.0).abs() < f32::EPSILON);
    }

    #[test]
    /// 편집 다이얼로그 높이가 200px 이다 (REQ-V13-031).
    fn edit_dialog_height_is_200() {
        assert!((KeyboardPane::dialog_height() - 200.0).abs() < f32::EPSILON);
    }

    // ---- 편집 다이얼로그 open/close tests ----

    #[test]
    /// open_edit() 로 다이얼로그가 열린다 (REQ-V13-031).
    fn open_edit_opens_dialog() {
        let mut pane = KeyboardPane::new();
        assert!(!pane.is_edit_open());
        pane.open_edit("open.settings");
        assert!(pane.is_edit_open());
    }

    #[test]
    /// open_edit() 후 dialog action_id 와 pending_shortcut 이 올바르다.
    fn open_edit_sets_correct_action_and_shortcut() {
        let mut pane = KeyboardPane::new();
        pane.open_edit("open.settings");
        let dialog = pane.edit_dialog().unwrap();
        assert_eq!(dialog.action_id, "open.settings");
        assert_eq!(dialog.pending_shortcut, "Cmd+,");
        assert!(dialog.conflict_error.is_none());
    }

    #[test]
    /// cancel_edit() 로 다이얼로그가 닫힌다 (REQ-V13-035).
    fn cancel_edit_closes_dialog() {
        let mut pane = KeyboardPane::new();
        pane.open_edit("open.settings");
        pane.cancel_edit();
        assert!(!pane.is_edit_open());
    }

    #[test]
    /// cancel_edit() 는 바인딩을 변경하지 않는다.
    fn cancel_edit_does_not_change_binding() {
        let mut pane = KeyboardPane::new();
        pane.open_edit("open.settings");
        pane.edit_dialog_mut().unwrap().pending_shortcut = "Cmd+Option+Z".to_string();
        pane.cancel_edit();
        let binding = pane
            .bindings()
            .iter()
            .find(|b| b.action_id == "open.settings")
            .unwrap();
        assert_eq!(binding.shortcut, "Cmd+,", "취소 후 원래 단축키 유지");
    }

    // ---- conflict_check tests ----

    #[test]
    /// conflict_check — 미사용 shortcut 은 None 반환 (AC-V13-8 pass case).
    fn conflict_check_unused_shortcut_pass() {
        let pane = KeyboardPane::new();
        let result = pane.conflict_check("open.settings", "Cmd+Option+X");
        assert!(result.is_none());
    }

    #[test]
    /// conflict_check — 다른 action 에 이미 사용된 shortcut 은 Some 반환 (AC-V13-8 fail case).
    fn conflict_check_used_shortcut_fail() {
        let pane = KeyboardPane::new();
        // "Cmd+T" 는 tabs.new 에 이미 할당됨
        let result = pane.conflict_check("open.settings", "Cmd+T");
        assert!(result.is_some(), "이미 사용된 shortcut은 충돌 반환");
    }

    #[test]
    /// conflict_check — 자기 자신 재할당은 충돌 아님.
    fn conflict_check_self_reassignment_no_conflict() {
        let pane = KeyboardPane::new();
        // open.settings 에 이미 할당된 "Cmd+," 를 open.settings 에 재할당 시도 → 충돌 아님
        let result = pane.conflict_check("open.settings", "Cmd+,");
        assert!(result.is_none(), "자기 자신 재할당은 충돌 아님");
    }

    // ---- save_edit tests ----

    #[test]
    /// save_edit — 미사용 shortcut 저장 성공 (REQ-V13-033).
    fn save_edit_success_no_conflict() {
        let mut pane = KeyboardPane::new();
        pane.open_edit("open.settings");
        pane.edit_dialog_mut().unwrap().pending_shortcut = "Cmd+Option+S".to_string();
        let ok = pane.save_edit();
        assert!(ok, "저장 성공");
        assert!(!pane.is_edit_open(), "저장 후 다이얼로그 닫힘");
        let binding = pane
            .bindings()
            .iter()
            .find(|b| b.action_id == "open.settings")
            .unwrap();
        assert_eq!(binding.shortcut, "Cmd+Option+S");
    }

    #[test]
    /// save_edit — 충돌 단축키 저장 실패 + inline 오류 설정 (REQ-V13-034).
    fn save_edit_conflict_sets_error_and_fails() {
        let mut pane = KeyboardPane::new();
        pane.open_edit("open.settings");
        // tabs.new 의 "Cmd+T" 로 충돌 시도
        pane.edit_dialog_mut().unwrap().pending_shortcut = "Cmd+T".to_string();
        let ok = pane.save_edit();
        assert!(!ok, "충돌 시 저장 실패");
        assert!(pane.is_edit_open(), "충돌 후 다이얼로그 열려 있음");
        let error = pane.edit_dialog().unwrap().conflict_error.as_ref();
        assert!(error.is_some(), "충돌 오류 메시지 설정됨");
    }

    #[test]
    /// save_edit — 충돌 후 기존 바인딩 변경 없음.
    fn save_edit_conflict_preserves_original_binding() {
        let mut pane = KeyboardPane::new();
        pane.open_edit("open.settings");
        pane.edit_dialog_mut().unwrap().pending_shortcut = "Cmd+T".to_string();
        pane.save_edit();
        let binding = pane
            .bindings()
            .iter()
            .find(|b| b.action_id == "open.settings")
            .unwrap();
        assert_eq!(binding.shortcut, "Cmd+,", "충돌 후 원래 단축키 유지");
    }

    #[test]
    /// 연속 편집: 첫 편집 성공 후 두 번째 편집도 정상 동작.
    fn sequential_edits_work_correctly() {
        let mut pane = KeyboardPane::new();

        // 첫 번째 편집
        pane.open_edit("open.settings");
        pane.edit_dialog_mut().unwrap().pending_shortcut = "Cmd+Option+S".to_string();
        assert!(pane.save_edit());

        // 두 번째 편집
        pane.open_edit("panes.close");
        pane.edit_dialog_mut().unwrap().pending_shortcut = "Cmd+Option+W".to_string();
        assert!(pane.save_edit());

        let settings_binding = pane
            .bindings()
            .iter()
            .find(|b| b.action_id == "open.settings")
            .unwrap();
        assert_eq!(settings_binding.shortcut, "Cmd+Option+S");

        let close_binding = pane
            .bindings()
            .iter()
            .find(|b| b.action_id == "panes.close")
            .unwrap();
        assert_eq!(close_binding.shortcut, "Cmd+Option+W");
    }
}
