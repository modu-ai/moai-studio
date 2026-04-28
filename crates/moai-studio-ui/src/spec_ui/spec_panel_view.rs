//! SPEC-V3-015 MS-1 — SpecPanelView: List/Kanban/Sprint 3-mode 컨테이너.
//!
//! RootView overlay 패턴으로 mount/dismiss.
//! Cmd+Shift+S 단축키로 toggle.
//! USER-DECISION-SU-RV-A = (a) overlay.

use std::path::PathBuf;

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_spec::SpecId;

use crate::design::tokens as tok;
use crate::spec_ui::kanban_view::KanbanBoardView;
use crate::spec_ui::list_view::SpecListView;
use crate::spec_ui::sprint_panel::SprintContractPanel;

/// SpecPanelView 의 활성 view mode (3-mode cycling).
///
/// @MX:NOTE: [AUTO] SpecPanelMode — Tab 키로 순환되는 3-mode 상태기.
/// List → Kanban → Sprint → List 순환 (cycle_mode 메서드 참조).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecPanelMode {
    List,
    Kanban,
    Sprint,
}

/// SpecPanelView — RootView overlay 컨테이너 (REQ-RV-1, REQ-RV-2).
///
/// # @MX:ANCHOR: [AUTO] spec-panel-view
/// @MX:REASON: [AUTO] SPEC-V3-015 MS-1. SpecPanelView 는 3개 spec_ui 컴포넌트의
///   단일 진입점이며 RootView 가 slot으로 소유, key dispatch 에서 toggle, Render impl 에서 렌더.
///   fan_in >= 3: RootView::new (init), handle_spec_key_event (key), Render::render (mount).
pub struct SpecPanelView {
    /// `.moai/specs/` 베이스 디렉터리
    pub specs_dir: PathBuf,
    /// 현재 활성 view mode
    pub mode: SpecPanelMode,
    /// List mode 컴포넌트
    pub list: SpecListView,
    /// Kanban mode 컴포넌트
    pub kanban: KanbanBoardView,
    /// Sprint mode 컴포넌트 (None = SPEC 미선택 상태)
    pub sprint: Option<SprintContractPanel>,
}

impl SpecPanelView {
    /// 새 SpecPanelView 를 생성한다. 기본 mode = List (REQ-RV-4).
    pub fn new(specs_dir: PathBuf) -> Self {
        let list = SpecListView::new(specs_dir.clone());
        let kanban = KanbanBoardView::new(specs_dir.clone());
        Self {
            specs_dir,
            mode: SpecPanelMode::List,
            list,
            kanban,
            sprint: None,
        }
    }

    /// mode 를 직접 전환한다 (AC-RV-3 explicit selector).
    pub fn set_mode(&mut self, mode: SpecPanelMode) {
        self.mode = mode;
    }

    /// Tab 키 입력 시 mode 를 순환한다 (List → Kanban → Sprint → List).
    ///
    /// @MX:NOTE: [AUTO] cycle-mode — 3-mode 순환 로직 (List→Kanban→Sprint→List).
    /// Tab 키 dispatch 시 호출되며 mode 전환 후 notify 필요 없음 (호출자 책임).
    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            SpecPanelMode::List => SpecPanelMode::Kanban,
            SpecPanelMode::Kanban => SpecPanelMode::Sprint,
            SpecPanelMode::Sprint => SpecPanelMode::List,
        };
    }

    /// SPEC 카드 선택 시 SprintContractPanel 을 초기화한다 (AC-RV-4).
    ///
    /// `spec_id` 가 list index 에 없으면 noop (graceful).
    pub fn select_spec(&mut self, spec_id: SpecId) {
        let record = self.list.index.find(&spec_id);
        if let Some(record) = record {
            self.sprint = Some(SprintContractPanel::from_spec(record));
            self.list.selected_id = Some(spec_id);
        }
        // 없는 id 는 noop — sprint = 기존 상태 유지 (AC-RV-4 graceful)
    }

    /// MS-2: 현재 로드된 SPEC 목록의 이름들을 반환한다.
    ///
    /// 각 SPEC의 ID 문자열을 포함하는 Vec를 반환한다.
    pub fn spec_names(&self) -> Vec<String> {
        self.list
            .index
            .records
            .iter()
            .map(|r| r.id.to_string())
            .collect()
    }

    /// MS-2: SPEC 목록이 비어있는지 확인한다.
    ///
    /// 로드된 SPEC이 하나도 없으면 true를 반환한다.
    pub fn is_empty(&self) -> bool {
        self.list.index.is_empty()
    }
}

impl Render for SpecPanelView {
    /// mode 에 따라 헤더 + 활성 서브컴포넌트를 렌더한다 (AC-RV-4).
    ///
    /// 헤더: [List] [Kanban] [Sprint] — 활성 mode 강조 (ACCENT 색상).
    /// 본문: mode 별 컴포넌트 placeholder (컴포넌트 자체 Render 위임).
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mode = self.mode;

        // mode selector 헤더
        let header = div()
            .flex()
            .flex_row()
            .gap_1()
            .px_3()
            .py_2()
            .bg(rgb(tok::BG_SURFACE))
            .border_b_1()
            .border_color(rgb(tok::BORDER_SUBTLE))
            .child(mode_tab_label("List", mode == SpecPanelMode::List))
            .child(mode_tab_label("Kanban", mode == SpecPanelMode::Kanban))
            .child(mode_tab_label("Sprint", mode == SpecPanelMode::Sprint));

        // 본문: 활성 mode 컴포넌트 placeholder
        let body_label = match mode {
            SpecPanelMode::List => "SpecListView",
            SpecPanelMode::Kanban => "KanbanBoardView",
            SpecPanelMode::Sprint => "SprintContractPanel",
        };

        div()
            .flex()
            .flex_col()
            .w(px(640.))
            .h(px(480.))
            .bg(rgb(tok::BG_PANEL))
            .rounded_lg()
            .overflow_hidden()
            .child(header)
            .child(
                div().flex().flex_col().flex_grow().p_3().child(
                    div()
                        .text_sm()
                        .text_color(rgb(tok::FG_SECONDARY))
                        .child(body_label),
                ),
            )
    }
}

/// mode 탭 라벨 — 활성 시 ACCENT 강조, 비활성 시 MUTED.
fn mode_tab_label(label: &'static str, active: bool) -> impl IntoElement {
    let fg = if active { tok::ACCENT } else { tok::FG_MUTED };
    div()
        .px_2()
        .py_1()
        .text_sm()
        .text_color(rgb(fg))
        .child(label)
}

// ============================================================
// 단위 테스트 (RED → GREEN)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn dummy_specs_dir() -> PathBuf {
        PathBuf::from("/nonexistent/specs")
    }

    /// AC-RV-1 / REQ-RV-4: 기본 mode = List.
    #[test]
    fn new_initializes_with_list_mode() {
        let view = SpecPanelView::new(dummy_specs_dir());
        assert_eq!(
            view.mode,
            SpecPanelMode::List,
            "기본 mode 는 List 이어야 한다"
        );
    }

    /// AC-RV-1: specs_dir 이 올바르게 저장된다.
    #[test]
    fn new_stores_specs_dir() {
        let dir = PathBuf::from("/some/specs");
        let view = SpecPanelView::new(dir.clone());
        assert_eq!(view.specs_dir, dir);
    }

    /// AC-RV-3: Tab 키 cycle — List → Kanban → Sprint → List.
    #[test]
    fn cycle_mode_advances_through_3_modes() {
        let mut view = SpecPanelView::new(dummy_specs_dir());
        assert_eq!(view.mode, SpecPanelMode::List);

        view.cycle_mode();
        assert_eq!(view.mode, SpecPanelMode::Kanban, "List → Kanban");

        view.cycle_mode();
        assert_eq!(view.mode, SpecPanelMode::Sprint, "Kanban → Sprint");

        view.cycle_mode();
        assert_eq!(view.mode, SpecPanelMode::List, "Sprint → List (순환)");
    }

    /// AC-RV-3: set_mode 로 직접 전환 (List → Sprint).
    #[test]
    fn set_mode_switches_directly() {
        let mut view = SpecPanelView::new(dummy_specs_dir());
        assert_eq!(view.mode, SpecPanelMode::List);

        view.set_mode(SpecPanelMode::Sprint);
        assert_eq!(view.mode, SpecPanelMode::Sprint, "직접 Sprint 로 전환");
    }

    /// AC-RV-4: 초기 sprint = None.
    #[test]
    fn sprint_panel_initially_none() {
        let view = SpecPanelView::new(dummy_specs_dir());
        assert!(
            view.sprint.is_none(),
            "초기 sprint panel 은 None 이어야 한다"
        );
    }

    /// AC-RV-4: select_spec 으로 없는 id 전달 시 sprint 는 None 유지 (graceful).
    #[test]
    fn select_spec_with_invalid_id_is_noop() {
        let mut view = SpecPanelView::new(dummy_specs_dir());
        assert!(view.sprint.is_none());

        // 존재하지 않는 SPEC ID → noop, panic 없음
        let fake_id = SpecId::new("SPEC-FAKE-999");
        view.select_spec(fake_id);

        assert!(
            view.sprint.is_none(),
            "없는 SPEC ID 는 sprint 를 변경하지 않아야 한다"
        );
    }

    /// AC-RV-3: set_mode 는 다른 필드에 영향 없음.
    #[test]
    fn set_mode_does_not_affect_sprint() {
        let mut view = SpecPanelView::new(dummy_specs_dir());
        view.set_mode(SpecPanelMode::Sprint);
        // sprint panel 은 set_mode 로 초기화되지 않는다 — select_spec 만 초기화.
        assert!(
            view.sprint.is_none(),
            "set_mode 는 sprint panel 을 초기화하지 않는다"
        );
    }

    // ============================================================
    // MS-2 Tests - Real SPEC data loading
    // ============================================================

    use std::fs;
    use tempfile::TempDir;

    /// Create a temporary specs directory with test SPEC folders
    fn create_test_specs_dir() -> TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let specs_dir = tmp.path().join(".moai").join("specs");

        // Create test SPEC directories
        let spec1 = specs_dir.join("SPEC-V3-001");
        let spec2 = specs_dir.join("SPEC-V3-002");
        let spec3 = specs_dir.join("SPEC-M2-002");

        fs::create_dir_all(&spec1).unwrap();
        fs::create_dir_all(&spec2).unwrap();
        fs::create_dir_all(&spec3).unwrap();

        // Add minimal spec.md files
        fs::write(
            spec1.join("spec.md"),
            "---\nid: SPEC-V3-001\ntitle: Test Spec 1\n---\n",
        )
        .unwrap();
        fs::write(
            spec2.join("spec.md"),
            "---\nid: SPEC-V3-002\ntitle: Test Spec 2\n---\n",
        )
        .unwrap();
        fs::write(
            spec3.join("spec.md"),
            "---\nid: SPEC-M2-002\ntitle: Test Spec M2\n---\n",
        )
        .unwrap();

        tmp
    }

    /// MS-2: SpecPanel should load actual SPEC names from directory
    #[test]
    fn test_spec_panel_loads_real_specs() {
        let tmp = create_test_specs_dir();
        let specs_dir = tmp.path().join(".moai").join("specs");
        let panel = SpecPanelView::new(specs_dir);

        let names = panel.spec_names();

        // Should find 3 SPEC directories
        assert_eq!(names.len(), 3, "Should load 3 SPECs from directory");

        // Should contain the SPEC names
        assert!(names.contains(&"SPEC-V3-001".to_string()));
        assert!(names.contains(&"SPEC-V3-002".to_string()));
        assert!(names.contains(&"SPEC-M2-002".to_string()));
    }

    /// MS-2: SpecPanel should handle empty directory gracefully
    #[test]
    fn test_spec_panel_empty_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let specs_dir = tmp.path().join(".moai").join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let panel = SpecPanelView::new(specs_dir);

        assert!(panel.is_empty(), "Should be empty when no specs exist");
        assert_eq!(panel.spec_names().len(), 0);
    }

    /// MS-2: SpecPanel should handle non-existent directory gracefully
    #[test]
    fn test_spec_panel_nonexistent_directory() {
        let specs_dir = PathBuf::from("/nonexistent/.moai/specs");
        let panel = SpecPanelView::new(specs_dir);

        assert!(panel.is_empty(), "Should be empty when directory doesn't exist");
        assert_eq!(panel.spec_names().len(), 0);
    }

    /// MS-2: SpecPanel should provide access to SpecIndex for details
    #[test]
    fn test_spec_panel_index_access() {
        let tmp = create_test_specs_dir();
        let specs_dir = tmp.path().join(".moai").join("specs");
        let panel = SpecPanelView::new(specs_dir);

        // Should be able to access the underlying index
        assert!(!panel.list.index.is_empty());
        assert_eq!(panel.list.index.len(), 3);

        // Should be able to find specific specs
        let id = SpecId::new("SPEC-V3-001");
        let record = panel.list.index.find(&id);
        assert!(record.is_some(), "Should find SPEC-V3-001");
        assert_eq!(record.unwrap().id.as_str(), "SPEC-V3-001");
    }

    /// MS-2: select_spec should work with real loaded specs
    #[test]
    fn test_select_spec_with_real_spec() {
        let tmp = create_test_specs_dir();
        let specs_dir = tmp.path().join(".moai").join("specs");
        let mut view = SpecPanelView::new(specs_dir);

        // Select a real spec
        let id = SpecId::new("SPEC-V3-001");
        view.select_spec(id.clone());

        // Sprint panel should be initialized
        assert!(view.sprint.is_some(), "Sprint panel should be initialized");
        assert_eq!(view.list.selected_id, Some(id));
    }
}
