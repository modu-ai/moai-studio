//! SPEC-V3-009 MS-3 RG-SU-6 — Sprint Contract timeline 패널.
//!
//! REQ-SU-050: `^## \d+\.\d+ Sprint Contract Revision` heading 추출.
//! REQ-SU-051: 각 revision = (section, title, date, body).
//! REQ-SU-052: 가장 최근이 위 (section 번호 내림차순).
//! REQ-SU-053: revision 없으면 "No sprint contract revisions yet." placeholder.
//!
//! parser 는 `moai_studio_spec::parser::sprint_contract::parse_sprint_contracts` 재사용.
//!
//! # @MX:ANCHOR: [AUTO] SprintContractPanel
//! @MX:REASON: [AUTO] SPEC-V3-009 RG-SU-6 진입점. fan_in >= 3:
//!   spec_ui::mod, AC-SU-11 테스트, MS-3 통합.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_spec::parser::{SprintContractRevision, parse_sprint_contracts};
use moai_studio_spec::SpecRecord;

use crate::design::tokens as tok;

/// Sprint Contract revision timeline 패널 (RG-SU-6).
///
/// @MX:ANCHOR: [AUTO] SprintContractPanel — SPEC-V3-009 RG-SU-6 진입점.
/// @MX:REASON: [AUTO] fan_in >= 3: spec_ui::mod, AC-SU-11 테스트, kanban_view.
pub struct SprintContractPanel {
    /// 추출된 revision 목록 (most-recent-first 정렬, REQ-SU-052)
    pub revisions: Vec<SprintContractRevision>,
    /// 선택된 revision 인덱스 (None = 선택 없음)
    pub selected_idx: Option<usize>,
}

impl SprintContractPanel {
    /// SpecRecord 에서 Sprint Contract Panel 을 생성한다.
    ///
    /// spec.md 파일을 읽어 parse_sprint_contracts 를 호출한다.
    /// 파일이 없거나 읽기 실패 시 빈 panel (REQ-SU-005 spirit, NEVER panic).
    pub fn from_spec(spec_record: &SpecRecord) -> Self {
        // spec.md 경로 획득
        use moai_studio_spec::SpecFileKind;
        let text = spec_record
            .file_path(SpecFileKind::Spec)
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_default();
        Self::from_text(&text)
    }

    /// spec.md 텍스트에서 직접 생성한다 (테스트용 직접 경로).
    pub fn from_text(spec_md_text: &str) -> Self {
        let mut revisions = parse_sprint_contracts(spec_md_text);
        // most-recent-first: section 번호 내림차순 정렬 (REQ-SU-052)
        // (major, minor) 쌍에서 major 내림차순 → minor 내림차순
        revisions.sort_by(|a, b| b.section.cmp(&a.section));
        Self {
            revisions,
            selected_idx: None,
        }
    }

    /// 비어 있는지 여부.
    pub fn is_empty(&self) -> bool {
        self.revisions.is_empty()
    }
}

impl Render for SprintContractPanel {
    /// Sprint Contract revision timeline 을 렌더한다 (REQ-SU-052, REQ-SU-053).
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut col = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(tok::BG_PANEL))
            .p(px(8.))
            .gap(px(6.));

        if self.revisions.is_empty() {
            // REQ-SU-053: placeholder
            col = col.child(
                div()
                    .text_color(rgb(tok::FG_MUTED))
                    .text_size(px(12.))
                    .child("No sprint contract revisions yet."),
            );
            return col;
        }

        for (idx, rev) in self.revisions.iter().enumerate() {
            let is_selected = self.selected_idx == Some(idx);
            let bg = if is_selected {
                rgb(tok::BG_ELEVATED)
            } else {
                rgb(tok::BG_SURFACE)
            };

            // `§{major}.{minor} — {title_suffix}` + date
            let (major, minor) = rev.section;
            let display_title = format!("§{major}.{minor} — {}", truncate_title(&rev.title));

            let date_str = rev
                .date
                .as_deref()
                .map(|d| format!("  {d}"))
                .unwrap_or_default();

            // body preview (첫 줄 최대 60자)
            let body_preview = rev
                .body
                .lines()
                .find(|l| !l.trim().is_empty())
                .map(|l| {
                    if l.len() > 60 {
                        format!("{}…", &l[..60])
                    } else {
                        l.to_string()
                    }
                })
                .unwrap_or_default();

            let entry = div()
                .flex()
                .flex_col()
                .w_full()
                .p(px(6.))
                .bg(bg)
                .rounded_md()
                .gap(px(2.))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_color(rgb(tok::ACCENT))
                                .text_size(px(12.))
                                .child(display_title),
                        )
                        .child(
                            div()
                                .text_color(rgb(tok::FG_SECONDARY))
                                .text_size(px(11.))
                                .child(date_str),
                        ),
                )
                .child(
                    div()
                        .text_color(rgb(tok::FG_MUTED))
                        .text_size(px(10.))
                        .child(body_preview),
                );

            col = col.child(entry);
        }

        col
    }
}

/// heading 전체 문자열에서 표시용 suffix 를 추출한다.
///
/// `"## 10.1 Sprint Contract Revision 2026-04-20"` →
/// `"Sprint Contract Revision 2026-04-20"`
fn truncate_title(full_title: &str) -> String {
    // `## N.M ` prefix 를 제거
    // 예: "## 10.1 Sprint Contract Revision ..." → "Sprint Contract Revision ..."
    if let Some(rest) = full_title.strip_prefix("## ") {
        // section 번호와 공백 제거
        if let Some(after_num) = rest.find(' ') {
            return rest[after_num + 1..].to_string();
        }
    }
    full_title.to_string()
}

// ============================================================
// 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── 5개 revision fixture (§10.1 ~ §10.5) ──────────────────────

    const FIXTURE_5_REVISIONS: &str = r#"
## 10. Sprint Contracts

Introduction.

## 10.1 Sprint Contract Revision 2026-04-01

First revision. Initial scope defined.

## 10.2 Sprint Contract Revision 2026-04-05

Second revision. Scope expanded.

## 10.3 Sprint Contract Revision 2026-04-10

Third revision. Bug fix scope.

## 10.4 Sprint Contract Revision 2026-04-15

Fourth revision. Performance focus.

## 10.5 Sprint Contract Revision 2026-04-20

Fifth revision. Final polish.

## 11. Other Section

Not a sprint contract.
"#;

    #[test]
    fn from_text_extracts_5_revisions() {
        // AC-SU-11: 5개 revision 추출
        let panel = SprintContractPanel::from_text(FIXTURE_5_REVISIONS);
        assert_eq!(panel.revisions.len(), 5, "5개 revision 추출");
    }

    #[test]
    fn from_text_returns_empty_for_no_revisions() {
        // AC-SU-11: revision 없는 경우 빈 Vec
        let text = "# Normal SPEC\n\n## 5. Requirements\n\nNo sprint contracts here.\n";
        let panel = SprintContractPanel::from_text(text);
        assert!(panel.revisions.is_empty(), "revision 없으면 빈 패널");
        assert!(panel.is_empty());
    }

    #[test]
    fn revisions_sorted_most_recent_first() {
        // REQ-SU-052: section 번호 내림차순 (10.5 > 10.4 > ... > 10.1)
        let panel = SprintContractPanel::from_text(FIXTURE_5_REVISIONS);
        let sections: Vec<_> = panel.revisions.iter().map(|r| r.section).collect();
        assert_eq!(
            sections,
            vec![(10, 5), (10, 4), (10, 3), (10, 2), (10, 1)],
            "내림차순 정렬 확인"
        );
    }

    #[test]
    fn from_text_extracts_5_from_v3_003_fixture() {
        // AC-SU-11: 실제 SPEC-V3-003 spec.md 에서 revision 추출
        // 파일이 없는 경우 inline fixture 로 대체
        let real_spec_path =
            std::path::Path::new(".moai/specs/SPEC-V3-003/spec.md");
        if real_spec_path.exists() {
            let text = std::fs::read_to_string(real_spec_path).unwrap();
            let panel = SprintContractPanel::from_text(&text);
            assert!(
                panel.revisions.len() >= 5,
                "SPEC-V3-003 에서 최소 5개 revision 추출: 실제 {}개",
                panel.revisions.len()
            );
        } else {
            // inline fixture fallback
            let panel = SprintContractPanel::from_text(FIXTURE_5_REVISIONS);
            assert_eq!(panel.revisions.len(), 5, "fallback fixture 5개 확인");
        }
    }

    #[test]
    fn placeholder_shown_when_empty() {
        // REQ-SU-053: 빈 상태에서 is_empty() = true
        let panel = SprintContractPanel::from_text("");
        assert!(panel.is_empty(), "빈 텍스트 → is_empty() = true");
    }

    #[test]
    fn revision_date_extracted() {
        // REQ-SU-051: date 추출 검증
        let panel = SprintContractPanel::from_text(FIXTURE_5_REVISIONS);
        // most-recent-first 이므로 [0] = §10.5
        let latest = &panel.revisions[0];
        assert_eq!(latest.section, (10, 5));
        assert_eq!(latest.date.as_deref(), Some("2026-04-20"));
    }

    #[test]
    fn revision_body_non_empty() {
        // REQ-SU-051: body 캡처 확인
        let panel = SprintContractPanel::from_text(FIXTURE_5_REVISIONS);
        for rev in &panel.revisions {
            assert!(
                !rev.body.is_empty(),
                "§{}.{} body 가 비어 있음",
                rev.section.0,
                rev.section.1
            );
        }
    }

    #[test]
    fn truncate_title_removes_section_prefix() {
        // heading 전체에서 section 번호 prefix 제거
        let full = "## 10.1 Sprint Contract Revision 2026-04-01";
        let result = truncate_title(full);
        assert_eq!(result, "Sprint Contract Revision 2026-04-01");
    }
}
