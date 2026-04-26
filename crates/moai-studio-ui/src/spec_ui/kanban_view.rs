//! SPEC-V3-009 MS-2 — KanbanBoardView GPUI Entity.
//!
//! AC-SU-6: 4 lane (TODO/IN-PROGRESS/REVIEW/DONE) 에 모든 SPEC 이 정확히 1개 lane 에 배치.
//! AC-SU-7: Enter 키로 stage 전환 시 sidecar persist + 재로드 후 동일 stage 복원.
//!
//! USER-DECISION-SU-B = (a) keyboard-only. ↑↓ + ←→ + Enter 로 조작.
//! N9: 새 design token 추가 없음 — 기존 `crate::design::tokens` 재사용.
//! N6: terminal/panes/tabs core 무변경.

use std::path::PathBuf;

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_spec::{KanbanStage, SpecIndex, SpecRecord, write_stage};

use crate::design::tokens as tok;

/// Kanban board 의 포커스 위치.
/// lane = KanbanStage, idx = lane 내 인덱스 (0-based).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KanbanFocus {
    pub stage: KanbanStage,
    pub idx: usize,
}

/// KanbanBoardView — 4 lane Kanban board (REQ-SU-020).
///
/// # @MX:ANCHOR: [AUTO] KanbanBoardView
/// @MX:REASON: [AUTO] SPEC-V3-009 MS-2 진입점.
///   fan_in >= 3: spec_ui::mod.rs, integration tests, AC-SU-6/7 테스트.
pub struct KanbanBoardView {
    /// 스캔된 SPEC 인덱스
    pub index: SpecIndex,
    /// `.moai/specs/` 베이스 디렉터리
    pub specs_dir: PathBuf,
    /// 현재 포커스 (lane + lane 내 인덱스). 없으면 None.
    pub focused: Option<KanbanFocus>,
}

impl KanbanBoardView {
    /// 새 KanbanBoardView 를 생성하고 초기 스캔을 수행한다.
    ///
    /// sidecar 는 `SpecIndex::scan` 내부에서 `kanban_persist::read_stage` 로 로드된다.
    pub fn new(specs_dir: PathBuf) -> Self {
        let mut index = SpecIndex::new();
        index.scan(&specs_dir);
        Self {
            index,
            specs_dir,
            focused: None,
        }
    }

    /// 지정 stage 의 records 참조를 반환한다 (AC-SU-6 unit-testable).
    ///
    /// 모든 SpecRecord 는 정확히 1개 lane 에만 나타난다 — kanban_stage 필드 기준.
    ///
    /// @MX:NOTE: [AUTO] lane_records — AC-SU-6 의 핵심 분류 함수.
    pub fn lane_records(&self, stage: KanbanStage) -> Vec<&SpecRecord> {
        self.index
            .records
            .iter()
            .filter(|r| r.kanban_stage == stage)
            .collect()
    }

    /// ↓ 키 — lane 내 포커스를 한 칸 아래로 이동한다.
    ///
    /// lane 끝에서는 이동하지 않는다 (stop at end, no wrap — 단순성 우선).
    /// sidecar 변경 없음 (REQ-SU-023).
    pub fn handle_arrow_down(&mut self) {
        let focus = match self.focused {
            Some(f) => f,
            None => return,
        };
        let lane_len = self.lane_records(focus.stage).len();
        if focus.idx + 1 < lane_len {
            self.focused = Some(KanbanFocus {
                stage: focus.stage,
                idx: focus.idx + 1,
            });
        }
    }

    /// ↑ 키 — lane 내 포커스를 한 칸 위로 이동한다.
    ///
    /// lane 시작(0)에서는 이동하지 않는다. sidecar 변경 없음 (REQ-SU-023).
    pub fn handle_arrow_up(&mut self) {
        let focus = match self.focused {
            Some(f) => f,
            None => return,
        };
        if focus.idx > 0 {
            self.focused = Some(KanbanFocus {
                stage: focus.stage,
                idx: focus.idx - 1,
            });
        }
    }

    /// ← 키 — 이전 lane 으로 포커스 이동 (파생 요구사항, REQ-SU-023 지원).
    ///
    /// Todo 에서는 이동하지 않는다. 이동 후 idx 는 새 lane 의 길이에 clamp.
    pub fn handle_arrow_left(&mut self) {
        let focus = match self.focused {
            Some(f) => f,
            None => return,
        };
        if focus.stage == KanbanStage::Todo {
            return;
        }
        let prev = match focus.stage {
            KanbanStage::InProgress => KanbanStage::Todo,
            KanbanStage::Review => KanbanStage::InProgress,
            KanbanStage::Done => KanbanStage::Review,
            KanbanStage::Todo => return, // 도달 불가
        };
        let prev_len = self.lane_records(prev).len();
        self.focused = Some(KanbanFocus {
            stage: prev,
            idx: focus.idx.min(prev_len.saturating_sub(1)),
        });
    }

    /// → 키 — 다음 lane 으로 포커스 이동 (파생 요구사항, REQ-SU-023 지원).
    ///
    /// Done 에서는 이동하지 않는다. 이동 후 idx 는 새 lane 의 길이에 clamp.
    pub fn handle_arrow_right(&mut self) {
        let focus = match self.focused {
            Some(f) => f,
            None => return,
        };
        if focus.stage == KanbanStage::Done {
            return;
        }
        let next = match focus.stage {
            KanbanStage::Todo => KanbanStage::InProgress,
            KanbanStage::InProgress => KanbanStage::Review,
            KanbanStage::Review => KanbanStage::Done,
            KanbanStage::Done => return, // 도달 불가
        };
        let next_len = self.lane_records(next).len();
        self.focused = Some(KanbanFocus {
            stage: next,
            idx: focus.idx.min(next_len.saturating_sub(1)),
        });
    }

    /// Enter 키 — 포커스된 카드를 다음 stage 로 전환하고 sidecar 에 즉시 write (REQ-SU-022).
    ///
    /// 포커스 없으면 noop (AC-SU-7).
    /// 메모리의 SpecRecord.kanban_stage 도 갱신 후 새 lane 에서 포커스 재계산.
    pub fn handle_enter(&mut self) -> std::io::Result<()> {
        let focus = match self.focused {
            Some(f) => f,
            None => return Ok(()), // 포커스 없음 → noop
        };

        // 포커스된 record 의 dir_path 와 새 stage 를 미리 계산
        let (spec_dir, new_stage) = {
            let lane = self.lane_records(focus.stage);
            let record = match lane.get(focus.idx) {
                Some(r) => *r,
                None => return Ok(()),
            };
            (record.dir_path.clone(), record.kanban_stage.next())
        };

        // sidecar 파일에 write (REQ-SU-022)
        write_stage(&spec_dir, new_stage)?;

        // 메모리 상태 갱신 — dir_path 기준으로 record 탐색
        for record in &mut self.index.records {
            if record.dir_path == spec_dir {
                record.kanban_stage = new_stage;
                break;
            }
        }

        // 새 lane 내에서 포커스 재조정 (이동된 카드가 새 lane 의 마지막에 추가)
        let new_lane_len = self.lane_records(new_stage).len();
        self.focused = Some(KanbanFocus {
            stage: new_stage,
            idx: new_lane_len.saturating_sub(1),
        });

        Ok(())
    }

    /// specs_dir 를 재스캔한다 (테스트 및 debounce 이후 호출용).
    pub fn rescan(&mut self) {
        let dir = self.specs_dir.clone();
        self.index.scan(&dir);
        // 포커스 초기화 (재스캔 후 lane 구조 변경 가능)
        self.focused = None;
    }
}

// ── GPUI Render ────────────────────────────────────────────────────────────────

impl Render for KanbanBoardView {
    /// 4 lane Kanban board 를 가로 flex 로 렌더한다 (REQ-SU-020, REQ-SU-024, REQ-SU-025).
    ///
    /// 시각적 smoke 는 MS-3+ e2e 로 위임. 본 구현은 구조적 정합성만 보장.
    /// (AC-SU-6 spec: "e2e (visual smoke)" 는 MS-3+ followup)
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let stages = [
            KanbanStage::Todo,
            KanbanStage::InProgress,
            KanbanStage::Review,
            KanbanStage::Done,
        ];

        let focused = self.focused;

        // 각 lane 의 records 를 미리 수집 (borrow checker 회피)
        // LaneCard: (id_str, title, ac_text, branch_text)
        type LaneCard = (String, String, String, String);
        let lanes: Vec<(KanbanStage, Vec<LaneCard>)> = stages
            .iter()
            .map(|&stage| {
                let cards: Vec<LaneCard> = self
                    .index
                    .records
                    .iter()
                    .filter(|r| r.kanban_stage == stage)
                    .map(|r| {
                        let id_str = r.id.to_string();
                        let title = r.title.clone();
                        let ac_text = r.ac_summary().display();
                        // MS-3 에서 실제 branch 파싱 구현 예정 — 현재는 placeholder
                        let branch_text = r
                            .branch_hint()
                            .map(|b| format!("▶ {b}"))
                            .unwrap_or_else(|| "(no branch)".to_string());
                        (id_str, title, ac_text, branch_text)
                    })
                    .collect();
                (stage, cards)
            })
            .collect();

        let mut board = div()
            .flex()
            .flex_row()
            .size_full()
            .bg(rgb(tok::BG_APP))
            .p(px(8.))
            .gap(px(8.));

        for (stage, cards) in &lanes {
            let lane_count = cards.len();
            let lane_label = format!("{} ({})", stage.label(), lane_count);

            let mut lane_col = div()
                .flex()
                .flex_col()
                .w(px(200.))
                .bg(rgb(tok::BG_PANEL))
                .p(px(6.))
                .rounded_md()
                .gap(px(4.));

            // lane header
            lane_col = lane_col.child(
                div()
                    .text_color(rgb(tok::ACCENT))
                    .text_size(px(11.))
                    .mb(px(4.))
                    .child(lane_label),
            );

            // cards
            for (card_idx, (id_str, title, ac_text, branch_text)) in cards.iter().enumerate() {
                let is_focused = focused.is_some_and(|f| f.stage == *stage && f.idx == card_idx);

                let card_bg = if is_focused {
                    rgb(tok::BG_ELEVATED)
                } else {
                    rgb(tok::BG_SURFACE)
                };

                let card = div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .p(px(6.))
                    .bg(card_bg)
                    .rounded_sm()
                    .gap(px(2.))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(4.))
                            .child(
                                div()
                                    .text_color(rgb(tok::ACCENT))
                                    .text_size(px(10.))
                                    .child(id_str.clone()),
                            )
                            .child(
                                div()
                                    .text_color(rgb(tok::FG_PRIMARY))
                                    .text_size(px(11.))
                                    .child(title.clone()),
                            ),
                    )
                    .child(
                        div()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .text_size(px(10.))
                            .child(ac_text.clone()),
                    )
                    .child(
                        div()
                            .text_color(rgb(tok::FG_MUTED))
                            .text_size(px(9.))
                            .child(branch_text.clone()),
                    );

                lane_col = lane_col.child(card);
            }

            board = board.child(lane_col);
        }

        board
    }
}

// ── SpecRecord 확장 트레이트 (branch placeholder) ────────────────────────────

/// MS-3 에서 실제 구현 예정인 branch hint 임시 인터페이스.
trait SpecRecordExt {
    /// 현재 branch 힌트 (MS-3 이전에는 None).
    fn branch_hint(&self) -> Option<&str>;
}

impl SpecRecordExt for SpecRecord {
    fn branch_hint(&self) -> Option<&str> {
        // MS-3: branch.rs 파서 구현 후 실제 git branch 반환
        // 현재는 None (AC-SU-8 는 MS-3 범위)
        None
    }
}

// ============================================================
// 단위 테스트 (RED phase 에서 먼저 작성)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_spec::KanbanStage;
    use std::fs;
    use tempfile::TempDir;

    // ── 테스트 헬퍼 ──────────────────────────────────────────────

    /// 지정 stage 의 SpecRecord 들을 가진 KanbanBoardView 를 생성한다.
    fn make_view_with_stages(stages: &[KanbanStage]) -> (KanbanBoardView, TempDir) {
        let tmp = tempfile::tempdir().unwrap();
        for (i, &stage) in stages.iter().enumerate() {
            let spec_id = format!("SPEC-T-{:03}", i + 1);
            let spec_dir = tmp.path().join(&spec_id);
            fs::create_dir_all(&spec_dir).unwrap();
            // sidecar 기록 (kanban_persist::write_stage 사용)
            moai_studio_spec::write_stage(&spec_dir, stage).unwrap();
        }
        let view = KanbanBoardView::new(tmp.path().to_path_buf());
        (view, tmp)
    }

    // ── AC-SU-6 단위 테스트 ──────────────────────────────────────

    #[test]
    fn lane_records_classifies_all_specs_into_exactly_one_lane() {
        // AC-SU-6: 모든 SpecRecord 가 정확히 1개 lane 에만 배치
        let stages = [
            KanbanStage::Todo,
            KanbanStage::InProgress,
            KanbanStage::InProgress,
            KanbanStage::Review,
            KanbanStage::Done,
        ];
        let (view, _tmp) = make_view_with_stages(&stages);

        let total: usize = [
            KanbanStage::Todo,
            KanbanStage::InProgress,
            KanbanStage::Review,
            KanbanStage::Done,
        ]
        .iter()
        .map(|&s| view.lane_records(s).len())
        .sum();

        assert_eq!(
            total,
            view.index.len(),
            "모든 record 가 lane 합계에 포함되어야 함"
        );

        // 각 record 는 정확히 1개 lane 에만 나타남
        for record in &view.index.records {
            let count: usize = [
                KanbanStage::Todo,
                KanbanStage::InProgress,
                KanbanStage::Review,
                KanbanStage::Done,
            ]
            .iter()
            .map(|&s| {
                view.lane_records(s)
                    .iter()
                    .filter(|r| r.id == record.id)
                    .count()
            })
            .sum();
            assert_eq!(
                count, 1,
                "record {} 가 정확히 1개 lane 에만 있어야 함",
                record.id
            );
        }
    }

    #[test]
    fn lane_records_returns_empty_for_unused_stage() {
        // 아무 record 도 없는 stage 는 빈 Vec 반환
        let (view, _tmp) = make_view_with_stages(&[KanbanStage::Todo]);
        assert!(view.lane_records(KanbanStage::InProgress).is_empty());
        assert!(view.lane_records(KanbanStage::Review).is_empty());
        assert!(view.lane_records(KanbanStage::Done).is_empty());
    }

    // ── 화살표 이동 테스트 ────────────────────────────────────────

    #[test]
    fn arrow_down_moves_focus_within_lane() {
        let (mut view, _tmp) = make_view_with_stages(&[KanbanStage::Todo, KanbanStage::Todo]);
        view.focused = Some(KanbanFocus {
            stage: KanbanStage::Todo,
            idx: 0,
        });
        view.handle_arrow_down();
        assert_eq!(
            view.focused,
            Some(KanbanFocus {
                stage: KanbanStage::Todo,
                idx: 1
            })
        );
    }

    #[test]
    fn arrow_down_at_lane_end_stops() {
        // lane 끝에서 ↓ 는 이동하지 않는다 (no wrap)
        let (mut view, _tmp) = make_view_with_stages(&[KanbanStage::Todo, KanbanStage::Todo]);
        view.focused = Some(KanbanFocus {
            stage: KanbanStage::Todo,
            idx: 1,
        });
        view.handle_arrow_down();
        assert_eq!(
            view.focused,
            Some(KanbanFocus {
                stage: KanbanStage::Todo,
                idx: 1
            }),
            "lane 끝에서 이동 없음"
        );
    }

    #[test]
    fn arrow_up_at_lane_start_stops() {
        let (mut view, _tmp) = make_view_with_stages(&[KanbanStage::Todo]);
        view.focused = Some(KanbanFocus {
            stage: KanbanStage::Todo,
            idx: 0,
        });
        view.handle_arrow_up();
        assert_eq!(
            view.focused,
            Some(KanbanFocus {
                stage: KanbanStage::Todo,
                idx: 0
            }),
            "lane 시작에서 이동 없음"
        );
    }

    #[test]
    fn arrow_up_moves_focus_up() {
        let (mut view, _tmp) = make_view_with_stages(&[KanbanStage::Todo, KanbanStage::Todo]);
        view.focused = Some(KanbanFocus {
            stage: KanbanStage::Todo,
            idx: 1,
        });
        view.handle_arrow_up();
        assert_eq!(
            view.focused,
            Some(KanbanFocus {
                stage: KanbanStage::Todo,
                idx: 0
            })
        );
    }

    #[test]
    fn arrow_left_right_switches_lane_preserving_index_clamp() {
        // [Todo:2개, InProgress:1개] 설정
        let (mut view, _tmp) = make_view_with_stages(&[
            KanbanStage::Todo,
            KanbanStage::Todo,
            KanbanStage::InProgress,
        ]);
        // Todo lane 의 idx=1 에 포커스
        view.focused = Some(KanbanFocus {
            stage: KanbanStage::Todo,
            idx: 1,
        });
        // → 오른쪽: InProgress (1개) → idx clamp → 0
        view.handle_arrow_right();
        assert_eq!(
            view.focused,
            Some(KanbanFocus {
                stage: KanbanStage::InProgress,
                idx: 0
            }),
            "오른쪽 이동 후 clamp"
        );
        // ← 왼쪽: Todo (2개) → idx 0 그대로
        view.handle_arrow_left();
        assert_eq!(
            view.focused,
            Some(KanbanFocus {
                stage: KanbanStage::Todo,
                idx: 0
            }),
            "왼쪽 이동 후 유지"
        );
    }

    #[test]
    fn arrow_right_stops_at_done() {
        let (mut view, _tmp) = make_view_with_stages(&[KanbanStage::Done]);
        view.focused = Some(KanbanFocus {
            stage: KanbanStage::Done,
            idx: 0,
        });
        view.handle_arrow_right();
        assert_eq!(
            view.focused,
            Some(KanbanFocus {
                stage: KanbanStage::Done,
                idx: 0
            }),
            "Done 에서 오른쪽 이동 없음"
        );
    }

    #[test]
    fn arrow_left_stops_at_todo() {
        let (mut view, _tmp) = make_view_with_stages(&[KanbanStage::Todo]);
        view.focused = Some(KanbanFocus {
            stage: KanbanStage::Todo,
            idx: 0,
        });
        view.handle_arrow_left();
        assert_eq!(
            view.focused,
            Some(KanbanFocus {
                stage: KanbanStage::Todo,
                idx: 0
            }),
            "Todo 에서 왼쪽 이동 없음"
        );
    }

    // ── Enter 테스트 (AC-SU-7) ────────────────────────────────────

    #[test]
    fn enter_when_no_focus_is_noop() {
        // 포커스 없으면 enter 는 아무것도 하지 않는다
        let (mut view, _tmp) = make_view_with_stages(&[KanbanStage::Todo]);
        view.focused = None;
        let result = view.handle_enter();
        assert!(result.is_ok());
        assert!(view.lane_records(KanbanStage::Todo).len() == 1);
    }

    #[test]
    fn enter_advances_stage_writes_sidecar_updates_in_memory_and_relocates_focus() {
        // AC-SU-7: Todo → InProgress, sidecar write + 메모리 갱신 + focus 이동
        let (mut view, _tmp) = make_view_with_stages(&[KanbanStage::Todo]);
        view.focused = Some(KanbanFocus {
            stage: KanbanStage::Todo,
            idx: 0,
        });

        let result = view.handle_enter();
        assert!(result.is_ok(), "enter 오류: {:?}", result);

        // 메모리 상태 확인
        assert_eq!(
            view.lane_records(KanbanStage::Todo).len(),
            0,
            "Todo lane 비어야 함"
        );
        assert_eq!(
            view.lane_records(KanbanStage::InProgress).len(),
            1,
            "InProgress lane 에 1개 있어야 함"
        );

        // 포커스가 InProgress 로 이동했는지
        assert!(view.focused.is_some());
        assert_eq!(view.focused.unwrap().stage, KanbanStage::InProgress);

        // sidecar 파일 확인 (실제 파일 I/O)
        let record = &view.index.records[0];
        let sidecar = record.dir_path.join(".kanban-stage");
        assert!(sidecar.exists(), "sidecar 파일이 생성되어야 함");
        let content = fs::read_to_string(&sidecar).unwrap();
        assert_eq!(content.trim(), "in-progress");
    }

    #[test]
    fn reload_after_enter_persists_stage() {
        // AC-SU-7: enter 후 view 를 재생성하면 동일 stage 가 복원된다
        let tmp = tempfile::tempdir().unwrap();
        let spec_dir = tmp.path().join("SPEC-R-001");
        fs::create_dir_all(&spec_dir).unwrap();
        // 초기: Todo
        moai_studio_spec::write_stage(&spec_dir, KanbanStage::Todo).unwrap();

        let mut view = KanbanBoardView::new(tmp.path().to_path_buf());
        view.focused = Some(KanbanFocus {
            stage: KanbanStage::Todo,
            idx: 0,
        });
        view.handle_enter().unwrap(); // Todo → InProgress

        // view 를 drop 하고 새로 생성 (재로드 시뮬레이션)
        drop(view);
        let new_view = KanbanBoardView::new(tmp.path().to_path_buf());

        assert_eq!(
            new_view.lane_records(KanbanStage::InProgress).len(),
            1,
            "재로드 후 InProgress 에 1개 있어야 함"
        );
        assert_eq!(
            new_view.index.records[0].kanban_stage,
            KanbanStage::InProgress,
            "재로드 후 stage 가 InProgress 로 복원되어야 함"
        );
    }

    // ── 포커스 없음 상태에서 화살표 noop ─────────────────────────

    #[test]
    fn arrow_keys_with_no_focus_are_noop() {
        let (mut view, _tmp) = make_view_with_stages(&[KanbanStage::Todo]);
        view.focused = None;
        view.handle_arrow_down();
        view.handle_arrow_up();
        view.handle_arrow_left();
        view.handle_arrow_right();
        assert!(view.focused.is_none(), "포커스 없으면 arrow 는 noop");
    }

    // ── lane_records 와 index.len() 정합 ──────────────────────────

    #[test]
    fn lane_records_sum_equals_total_record_count() {
        let stages = vec![
            KanbanStage::Todo,
            KanbanStage::InProgress,
            KanbanStage::Review,
            KanbanStage::Done,
            KanbanStage::Done,
        ];
        let (view, _tmp) = make_view_with_stages(&stages);
        let lane_sum: usize = [
            KanbanStage::Todo,
            KanbanStage::InProgress,
            KanbanStage::Review,
            KanbanStage::Done,
        ]
        .iter()
        .map(|&s| view.lane_records(s).len())
        .sum();
        assert_eq!(lane_sum, view.index.len());
    }
}
