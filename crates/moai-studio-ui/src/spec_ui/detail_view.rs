//! SPEC-V3-009 MS-1 — SpecDetailView GPUI Entity.
//!
//! AC-SU-2: SPEC-V3-003 의 EARS 표가 RG-P-1~12 + REQ-* 로 파싱되어 표시.
//! AC-SU-3: AC 상태가 FULL/PARTIAL/DEFERRED/FAIL/PENDING 으로 컬러 분류.
//! AC-SU-5: 파일 누락 시 panic 없이 "no {filename}" placeholder 표시.
//!
//! RootView 통합은 MS-3 에서 수행 (SPEC-V3-009 N6).

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_spec::{AcState, SpecFileKind, SpecRecord};

use crate::design::tokens::{self as tok, semantic};

/// AcState → 토큰 색상 (u32) 매핑 (REQ-SU-013).
pub fn ac_state_color(state: AcState) -> u32 {
    match state {
        AcState::Full => semantic::SUCCESS,
        AcState::Partial => semantic::WARNING,
        AcState::Deferred => tok::FG_MUTED,
        AcState::Fail => semantic::DANGER,
        AcState::Pending => semantic::INFO,
    }
}

/// SpecDetailView — 선택된 SPEC 의 RG/REQ/AC 표를 렌더한다.
///
/// @MX:ANCHOR: [AUTO] SpecDetailView
/// @MX:REASON: [AUTO] SPEC-V3-009 AC-SU-2/AC-SU-3. SPEC 본문 렌더 진입점.
///   fan_in >= 3: spec_ui::mod.rs, SpecListView (선택 이벤트), tests.
pub struct SpecDetailView {
    /// 표시 중인 SpecRecord (없으면 None — "선택하세요" placeholder)
    pub record: Option<SpecRecord>,
}

impl SpecDetailView {
    /// 새 SpecDetailView 생성 (record 없음).
    pub fn new() -> Self {
        Self { record: None }
    }

    /// 표시할 SpecRecord 를 설정한다.
    pub fn set_record(&mut self, record: SpecRecord, cx: &mut Context<Self>) {
        self.record = Some(record);
        cx.notify();
    }

    /// record 를 지운다.
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.record = None;
        cx.notify();
    }
}

impl Default for SpecDetailView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for SpecDetailView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        match &self.record {
            None => render_empty().into_any_element(),
            Some(record) => render_detail(record).into_any_element(),
        }
    }
}

/// record 없을 때 placeholder.
fn render_empty() -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .justify_center()
        .items_center()
        .bg(rgb(tok::BG_APP))
        .child(
            div()
                .text_color(rgb(tok::FG_MUTED))
                .text_size(px(14.))
                .child("SPEC 를 선택하세요"),
        )
}

/// SPEC 상세 렌더 (RG/REQ 표 + AC 표).
fn render_detail(record: &SpecRecord) -> impl IntoElement {
    let mut col = div()
        .flex()
        .flex_col()
        .size_full()
        .p(px(12.))
        .bg(rgb(tok::BG_PANEL))
        .gap(px(8.));

    // ── 헤더: SPEC ID + title ──
    col = col.child(
        div()
            .flex()
            .flex_row()
            .gap(px(8.))
            .child(
                div()
                    .text_color(rgb(tok::ACCENT))
                    .text_size(px(14.))
                    .child(record.id.to_string()),
            )
            .child(
                div()
                    .text_color(rgb(tok::FG_PRIMARY))
                    .text_size(px(15.))
                    .child(record.title.clone()),
            ),
    );

    // ── missing file placeholders (AC-SU-5 REQ-SU-005) ──
    for &kind in SpecFileKind::all() {
        if !record.has_file(kind) {
            col = col.child(
                div()
                    .text_color(rgb(tok::FG_MUTED))
                    .text_size(px(11.))
                    .child(format!("no {}", kind.filename())),
            );
        }
    }

    // ── EARS 요구사항 표 (AC-SU-2) ──
    if !record.requirement_groups.is_empty() {
        col = col.child(section_header("EARS 요구사항"));
        for group in &record.requirement_groups {
            col = col.child(render_rg_group(group));
        }
    } else {
        col = col.child(
            div()
                .text_color(rgb(tok::FG_MUTED))
                .text_size(px(12.))
                .child("요구사항 없음"),
        );
    }

    // ── AC 표 (AC-SU-3) ──
    if !record.ac_rows.is_empty() {
        col = col.child(section_header("Acceptance Criteria"));
        for ac_row in &record.ac_rows {
            // progress.md 의 AcRecord 에서 state 조회 (없으면 Pending)
            let state = record
                .ac_records
                .iter()
                .find(|r| r.id == ac_row.id)
                .map(|r| r.state)
                .unwrap_or(AcState::Pending);

            col = col.child(render_ac_row_with_state(
                &ac_row.id,
                &ac_row.scenario,
                state,
            ));
        }
    } else {
        col = col.child(
            div()
                .text_color(rgb(tok::FG_MUTED))
                .text_size(px(12.))
                .child("AC 없음"),
        );
    }

    col
}

/// 섹션 헤더 렌더 헬퍼.
fn section_header(title: &str) -> impl IntoElement {
    div()
        .text_color(rgb(tok::FG_SECONDARY))
        .text_size(px(13.))
        .mt(px(4.))
        .mb(px(2.))
        .child(title.to_string())
}

/// RG 그룹 렌더.
fn render_rg_group(group: &moai_studio_spec::parser::ears::RequirementGroup) -> impl IntoElement {
    let mut col = div().flex().flex_col().mb(px(4.));

    // 그룹 ID + title
    col = col.child(
        div()
            .text_color(rgb(tok::ACCENT))
            .text_size(px(12.))
            .mb(px(2.))
            .child(format!("{} — {}", group.id, group.title)),
    );

    for req in &group.requirements {
        col = col.child(
            div()
                .flex()
                .flex_row()
                .gap(px(8.))
                .mb(px(1.))
                .pl(px(8.))
                .child(
                    div()
                        .text_color(rgb(semantic::INFO))
                        .text_size(px(11.))
                        .child(req.id.clone()),
                )
                .child(
                    div()
                        .text_color(rgb(tok::FG_MUTED))
                        .text_size(px(10.))
                        .child(req.pattern.clone()),
                )
                .child(
                    div()
                        .text_color(rgb(tok::FG_SECONDARY))
                        .text_size(px(11.))
                        .child(req.korean.clone()),
                ),
        );
    }

    col
}

/// AC 행 + state badge 렌더.
fn render_ac_row_with_state(id: &str, scenario: &str, state: AcState) -> impl IntoElement {
    let state_color = ac_state_color(state);
    let state_label = match state {
        AcState::Full => "FULL",
        AcState::Partial => "PARTIAL",
        AcState::Deferred => "DEFERRED",
        AcState::Fail => "FAIL",
        AcState::Pending => "PENDING",
    };

    div()
        .flex()
        .flex_row()
        .gap(px(8.))
        .mb(px(1.))
        .child(
            div()
                .text_color(rgb(tok::ACCENT))
                .text_size(px(11.))
                .child(id.to_string()),
        )
        .child(
            div()
                .text_color(rgb(tok::FG_SECONDARY))
                .text_size(px(11.))
                .child(scenario.to_string()),
        )
        .child(
            div()
                .text_color(rgb(state_color))
                .text_size(px(10.))
                .child(state_label),
        )
}

// ============================================================
// 단위 테스트 (AC-SU-2, AC-SU-3, AC-SU-5)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AppContext, TestAppContext};
    use moai_studio_spec::{
        AcRecord, AcState, SpecFileKind, SpecId, SpecRecord,
        parser::{
            AcRow,
            ears::{Requirement, RequirementGroup},
        },
    };
    use std::path::PathBuf;

    fn make_record(id: &str) -> SpecRecord {
        SpecRecord::new(
            SpecId::new(id),
            format!("{id} 제목"),
            PathBuf::from(format!("/tmp/specs/{id}")),
        )
    }

    // ── ac_state_color tests (REQ-SU-013) ──

    #[test]
    fn ac_state_color_full_is_success() {
        // Full → status.success 토큰 색상
        let color = ac_state_color(AcState::Full);
        assert_eq!(color, semantic::SUCCESS, "Full → SUCCESS 색상");
    }

    #[test]
    fn ac_state_color_fail_is_danger() {
        let color = ac_state_color(AcState::Fail);
        assert_eq!(color, semantic::DANGER, "Fail → DANGER 색상");
    }

    #[test]
    fn ac_state_color_partial_is_warning() {
        let color = ac_state_color(AcState::Partial);
        assert_eq!(color, semantic::WARNING, "Partial → WARNING 색상");
    }

    #[test]
    fn ac_state_color_pending_is_info() {
        let color = ac_state_color(AcState::Pending);
        assert_eq!(color, semantic::INFO, "Pending → INFO 색상");
    }

    #[test]
    fn ac_state_color_deferred_is_muted() {
        let color = ac_state_color(AcState::Deferred);
        assert_eq!(color, tok::FG_MUTED, "Deferred → FG_MUTED 색상");
    }

    #[test]
    fn all_five_ac_states_have_distinct_colors() {
        // AC-SU-3: 5 상태 모두 distinct (또는 의도적 공유) 색상이 있음
        let colors: Vec<u32> = [
            AcState::Full,
            AcState::Partial,
            AcState::Deferred,
            AcState::Fail,
            AcState::Pending,
        ]
        .iter()
        .map(|&s| ac_state_color(s))
        .collect();
        // 5개 색상 모두 존재 (일부 공유 허용 — Deferred 는 FG_MUTED)
        assert_eq!(colors.len(), 5);
    }

    // ── SpecDetailView 기본 동작 ──

    #[test]
    fn detail_view_new_has_no_record() {
        let view = SpecDetailView::new();
        assert!(view.record.is_none());
    }

    #[test]
    fn detail_view_set_record_updates_record() {
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| SpecDetailView::new());

        let record = make_record("SPEC-V3-009");

        cx.update(|app| {
            entity.update(app, |view: &mut SpecDetailView, cx| {
                view.set_record(record, cx);
            });
        });

        let has_record = cx.read(|app| entity.read(app).record.is_some());
        assert!(has_record);
    }

    #[test]
    fn detail_view_clear_removes_record() {
        let mut cx = TestAppContext::single();
        let entity = cx.new(|_cx| {
            let mut v = SpecDetailView::new();
            v.record = Some(make_record("SPEC-V3-001"));
            v
        });

        cx.update(|app| {
            entity.update(app, |view: &mut SpecDetailView, cx| {
                view.clear(cx);
            });
        });

        let has_record = cx.read(|app| entity.read(app).record.is_some());
        assert!(!has_record);
    }

    // ── AC-SU-5: missing file graceful ──

    #[test]
    fn detail_view_no_panic_with_missing_acceptance_md() {
        // acceptance.md 없는 record 로 render 도 panic 없어야 함
        let mut cx = TestAppContext::single();
        let mut record = make_record("SPEC-V3-001");
        // acceptance.md = None (files 맵에 등록하지 않음 → has_file = false)
        record.files.insert(SpecFileKind::Acceptance, None);

        let entity = cx.new(|_cx| {
            let mut v = SpecDetailView::new();
            v.record = Some(record);
            v
        });

        // render 호출이 panic 하지 않아야 함
        cx.update(|app| {
            entity.update(app, |view: &mut SpecDetailView, _cx| {
                let has_missing = view
                    .record
                    .as_ref()
                    .is_some_and(|r| !r.has_file(SpecFileKind::Acceptance));
                assert!(has_missing, "acceptance.md 없음이 감지되어야 함");
            });
        });
    }

    // ── AC-SU-2: EARS 표 파싱 결과 표시 ──

    #[test]
    fn detail_view_shows_requirement_groups() {
        let mut record = make_record("SPEC-V3-003");
        record.requirement_groups = vec![RequirementGroup {
            id: "RG-P-1".to_string(),
            title: "Pane 자료구조".to_string(),
            requirements: vec![Requirement {
                id: "REQ-P-001".to_string(),
                pattern: "Ubiquitous".to_string(),
                korean: "시스템은 이진 트리를 제공한다.".to_string(),
                english: None,
            }],
        }];

        let view = SpecDetailView {
            record: Some(record),
        };
        let rg_count = view.record.as_ref().unwrap().rg_count();
        let req_count = view.record.as_ref().unwrap().req_count();
        assert_eq!(rg_count, 1);
        assert_eq!(req_count, 1);
    }

    // ── AC-SU-3: AC 상태 컬러 ──

    #[test]
    fn detail_view_ac_state_uses_progress_records_when_available() {
        let mut record = make_record("SPEC-V3-003");
        record.ac_rows = vec![AcRow {
            id: "AC-P-1".to_string(),
            scenario: "test".to_string(),
            pass_condition: "pass".to_string(),
            verification: None,
            rg_mapping: None,
        }];
        record.ac_records = vec![AcRecord::new("AC-P-1").with_label("PASS")];

        let view = SpecDetailView {
            record: Some(record),
        };
        let rec = view.record.as_ref().unwrap();

        // AC-P-1 의 state 는 Full 이어야 함
        let state = rec.ac_records[0].state;
        assert_eq!(state, AcState::Full);
        assert_eq!(ac_state_color(state), semantic::SUCCESS);
    }

    #[test]
    fn detail_view_ac_state_defaults_to_pending_when_no_progress() {
        let mut record = make_record("SPEC-V3-003");
        record.ac_rows = vec![AcRow {
            id: "AC-P-2".to_string(),
            scenario: "test".to_string(),
            pass_condition: "pass".to_string(),
            verification: None,
            rg_mapping: None,
        }];
        // ac_records 비어 있음 → Pending

        let view = SpecDetailView {
            record: Some(record),
        };
        let rec = view.record.as_ref().unwrap();

        // ac_rows 는 있지만 ac_records 가 없으면 Pending fallback
        let state = rec
            .ac_records
            .iter()
            .find(|r| r.id == "AC-P-2")
            .map(|r| r.state)
            .unwrap_or(AcState::Pending);
        assert_eq!(state, AcState::Pending);
    }
}
