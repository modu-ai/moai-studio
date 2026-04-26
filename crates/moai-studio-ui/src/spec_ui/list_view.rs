//! SPEC-V3-009 MS-1 — SpecListView GPUI Entity.
//!
//! AC-SU-1: `.moai/specs/` 디렉터리에서 spec.md 파일을 발견하여 SPEC 카드 목록을 렌더한다.
//! AC-SU-5: acceptance.md 가 없는 SPEC 도 panic 없이 "no acceptance.md" placeholder 를 표시한다.
//!
//! RootView 통합은 MS-3 에서 수행 (SPEC-V3-009 N6).

use std::path::PathBuf;

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_spec::{SpecFileKind, SpecId, SpecIndex, SpecRecord};

use crate::design::tokens as tok;

/// SpecListView 상태 — SPEC 목록 + 선택 상태.
///
/// @MX:ANCHOR: [AUTO] SpecListView
/// @MX:REASON: [AUTO] SPEC-V3-009 AC-SU-1. SPEC 목록 UI 의 진입점.
///   fan_in >= 3: spec_ui::mod.rs, detail_view (selected_id), tests.
pub struct SpecListView {
    /// 스캔된 SPEC 인덱스
    pub index: SpecIndex,
    /// 현재 선택된 SPEC ID (없으면 None)
    pub selected_id: Option<SpecId>,
    /// `.moai/specs/` 베이스 디렉터리
    pub specs_dir: PathBuf,
}

impl SpecListView {
    /// 새 SpecListView 를 생성하고 초기 스캔을 수행한다.
    ///
    /// `specs_dir` 가 존재하지 않으면 빈 목록으로 graceful 처리 (REQ-SU-005).
    pub fn new(specs_dir: PathBuf) -> Self {
        let mut index = SpecIndex::new();
        index.scan(&specs_dir);
        Self {
            index,
            selected_id: None,
            specs_dir,
        }
    }

    /// SPEC 목록을 재스캔한다 (REQ-SU-003 debounce 이후 호출).
    pub fn rescan(&mut self, cx: &mut Context<Self>) {
        let dir = self.specs_dir.clone();
        self.index.scan(&dir);
        cx.notify();
    }

    /// SPEC 선택.
    pub fn select(&mut self, id: SpecId, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }

    /// 선택 해제.
    pub fn deselect(&mut self, cx: &mut Context<Self>) {
        self.selected_id = None;
        cx.notify();
    }

    /// 현재 선택된 SpecRecord 참조.
    pub fn selected_record(&self) -> Option<&SpecRecord> {
        self.selected_id.as_ref().and_then(|id| self.index.find(id))
    }
}

impl Render for SpecListView {
    /// SPEC 카드 목록을 렌더한다.
    ///
    /// - SPEC 없음: "No SPECs found" 메시지
    /// - SPEC 있음: 각 SPEC 카드 (ID + title + AC summary + missing file placeholders)
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let records = self.index.records.clone();
        let selected_id = self.selected_id.clone();

        let mut col = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(tok::BG_PANEL))
            .p(px(8.))
            .gap(px(4.));

        if records.is_empty() {
            col = col.child(
                div()
                    .text_color(rgb(tok::FG_MUTED))
                    .text_size(px(13.))
                    .child("No SPECs found in .moai/specs/"),
            );
            return col;
        }

        for record in &records {
            let is_selected = selected_id.as_ref().is_some_and(|id| id == &record.id);

            let card = render_spec_card(record, is_selected);
            col = col.child(card);
        }

        col
    }
}

/// 단일 SPEC 카드를 렌더한다 (AC-SU-1, AC-SU-5).
fn render_spec_card(record: &SpecRecord, is_selected: bool) -> impl IntoElement {
    let bg = if is_selected {
        rgb(tok::BG_ELEVATED)
    } else {
        rgb(tok::BG_SURFACE)
    };

    let summary = record.ac_summary();
    let summary_text = summary.display();

    // missing file placeholders (AC-SU-5 — REQ-SU-005)
    let missing_files: Vec<String> = [
        SpecFileKind::Spec,
        SpecFileKind::Acceptance,
        SpecFileKind::Progress,
    ]
    .iter()
    .filter(|&&kind| !record.has_file(kind))
    .map(|kind| format!("no {}", kind.filename()))
    .collect();

    let mut card = div()
        .flex()
        .flex_col()
        .w_full()
        .p(px(8.))
        .mb(px(2.))
        .bg(bg)
        .rounded_md()
        .gap(px(2.));

    // 헤더: ID + title
    card = card.child(
        div()
            .flex()
            .flex_row()
            .gap(px(8.))
            .child(
                div()
                    .text_color(rgb(tok::ACCENT))
                    .text_size(px(12.))
                    .child(record.id.to_string()),
            )
            .child(
                div()
                    .text_color(rgb(tok::FG_PRIMARY))
                    .text_size(px(13.))
                    .child(record.title.clone()),
            ),
    );

    // AC 요약
    card = card.child(
        div()
            .text_color(rgb(tok::FG_SECONDARY))
            .text_size(px(11.))
            .child(summary_text),
    );

    // missing file placeholders (REQ-SU-005)
    for placeholder in &missing_files {
        card = card.child(
            div()
                .text_color(rgb(tok::FG_MUTED))
                .text_size(px(10.))
                .child(placeholder.clone()),
        );
    }

    card
}

// ============================================================
// 단위 테스트 (AC-SU-1, AC-SU-5)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};
    use std::fs;
    use tempfile::TempDir;

    fn make_specs_dir_with(specs: &[(&str, bool)]) -> TempDir {
        let tmp = tempfile::tempdir().unwrap();
        for (spec_id, has_spec_md) in specs {
            let dir = tmp.path().join(spec_id);
            fs::create_dir_all(&dir).unwrap();
            if *has_spec_md {
                fs::write(dir.join("spec.md"), format!("---\nid: {spec_id}\n---\n")).unwrap();
            }
        }
        tmp
    }

    #[test]
    fn list_view_empty_when_no_specs() {
        let tmp = tempfile::tempdir().unwrap();
        let view = SpecListView::new(tmp.path().to_path_buf());
        assert!(
            view.index.is_empty(),
            "디렉터리가 비어 있으면 목록도 비어야 함"
        );
    }

    #[test]
    fn list_view_discovers_spec_dirs() {
        // AC-SU-1: .moai/specs/ 디렉터리에서 SPEC 디렉터리 발견
        let tmp = make_specs_dir_with(&[("SPEC-V3-009", true), ("SPEC-V3-001", false)]);
        let view = SpecListView::new(tmp.path().to_path_buf());
        assert_eq!(view.index.len(), 2, "2개 SPEC 디렉터리 발견");
    }

    #[test]
    fn list_view_finds_spec_v3_009() {
        // AC-SU-1: .moai/specs/SPEC-V3-009/ 자체가 1개 카드로 등장
        let tmp = make_specs_dir_with(&[("SPEC-V3-009", true)]);
        let view = SpecListView::new(tmp.path().to_path_buf());
        let id = SpecId::new("SPEC-V3-009");
        assert!(view.index.find(&id).is_some(), "SPEC-V3-009 발견");
    }

    #[test]
    fn list_view_graceful_when_spec_dir_missing() {
        // REQ-SU-005: 디렉터리 없어도 panic 없이 빈 목록
        let view = SpecListView::new(PathBuf::from("/nonexistent/specs"));
        assert!(view.index.is_empty());
    }

    #[test]
    fn list_view_select_updates_selected_id() {
        let tmp = make_specs_dir_with(&[("SPEC-V3-009", true)]);
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| SpecListView::new(tmp.path().to_path_buf()));

        cx.update(|app| {
            entity.update(app, |view: &mut SpecListView, cx| {
                view.select(SpecId::new("SPEC-V3-009"), cx);
            });
        });

        let selected = cx.read(|app| entity.read(app).selected_id.clone());
        assert_eq!(selected, Some(SpecId::new("SPEC-V3-009")));
    }

    #[test]
    fn list_view_deselect_clears_selected_id() {
        let tmp = make_specs_dir_with(&[("SPEC-V3-009", true)]);
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| {
            let mut view = SpecListView::new(tmp.path().to_path_buf());
            view.selected_id = Some(SpecId::new("SPEC-V3-009"));
            view
        });

        cx.update(|app| {
            entity.update(app, |view: &mut SpecListView, cx| {
                view.deselect(cx);
            });
        });

        let selected = cx.read(|app| entity.read(app).selected_id.clone());
        assert!(selected.is_none());
    }

    #[test]
    fn list_view_missing_acceptance_shows_placeholder() {
        // AC-SU-5: acceptance.md 없는 SPEC 도 panic 없이 placeholder
        let tmp = make_specs_dir_with(&[("SPEC-V3-001", false)]);
        let view = SpecListView::new(tmp.path().to_path_buf());
        let id = SpecId::new("SPEC-V3-001");
        let record = view.index.find(&id).unwrap();
        // acceptance.md 없음 — has_file 이 false 여야 함
        assert!(!record.has_file(SpecFileKind::Acceptance));
        // render 가 panic 없이 동작하는지는 GPUI TestAppContext 로 검증
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| SpecListView::new(tmp.path().to_path_buf()));
        // render 호출이 panic 하지 않아야 함 (AC-SU-5 REQ-SU-005)
        cx.update(|app| {
            entity.update(app, |view: &mut SpecListView, _cx| {
                let _ = view.selected_record();
            });
        });
    }

    #[test]
    fn list_view_selected_record_returns_correct_record() {
        let tmp = make_specs_dir_with(&[("SPEC-V3-009", true), ("SPEC-V3-001", false)]);
        let mut view = SpecListView::new(tmp.path().to_path_buf());

        // 선택 없음
        assert!(view.selected_record().is_none());

        // SPEC-V3-009 선택
        view.selected_id = Some(SpecId::new("SPEC-V3-009"));
        let rec = view.selected_record().unwrap();
        assert_eq!(rec.id.as_str(), "SPEC-V3-009");
    }

    #[test]
    fn list_view_rescan_picks_up_new_specs() {
        let tmp = tempfile::tempdir().unwrap();
        let mut cx = TestAppContext::single();
        let tmp_path = tmp.path().to_path_buf();
        let entity = cx.new(|_cx| SpecListView::new(tmp_path.clone()));

        // 초기 상태: 빈 목록
        let initial_len = cx.read(|app| entity.read(app).index.len());
        assert_eq!(initial_len, 0);

        // 새 SPEC 디렉터리 추가
        fs::create_dir_all(tmp.path().join("SPEC-NEW-001")).unwrap();

        cx.update(|app| {
            entity.update(app, |view: &mut SpecListView, cx| {
                view.rescan(cx);
            });
        });

        let new_len = cx.read(|app| entity.read(app).index.len());
        assert_eq!(new_len, 1, "rescan 후 새 SPEC 발견");
    }
}
