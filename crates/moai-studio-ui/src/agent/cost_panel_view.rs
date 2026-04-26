//! CostPanelView GPUI Entity — session/daily/weekly USD 비용 표시 (RG-AD-3, AC-AD-5/6)
//!
//! SPEC-V3-010 REQ-AD-014: API self-report cost_usd 만 표시.
//! SPEC-V3-010 REQ-AD-017: 로컬 계산 금지.
//!
//! @MX:ANCHOR: [AUTO] cost-panel-view-entity
//! @MX:REASON: [AUTO] 비용 UI 단일 진실 원천. fan_in >= 3:
//!   AgentDashboardView, 테스트, 미래 통합 뷰.
//!   SPEC: SPEC-V3-010 RG-AD-3, AC-AD-5/6

// @MX:NOTE: [AUTO] cost-display-api-only
// @MX:SPEC: SPEC-V3-010 REQ-AD-017
// 표시값은 반드시 CostTracker 를 통해 API 제공 cost_usd 에서만 온다.
// "$%.4f" 포맷으로 소수점 4자리 표시.

use std::time::SystemTime;

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_agent::{AgentRunId, CostSnapshot, CostTracker, cost::unix_secs_to_system_time};

use crate::design::tokens as tok;

/// CostPanelView GPUI Entity — 비용 집계 표시 (AC-AD-5/6).
pub struct CostPanelView {
    /// 비용 집계기
    pub tracker: CostTracker,
    /// 현재 run (session_total 기준)
    pub current_run: Option<AgentRunId>,
}

impl CostPanelView {
    /// 새 CostPanelView 를 생성한다.
    pub fn new() -> Self {
        Self {
            tracker: CostTracker::new(),
            current_run: None,
        }
    }

    /// 스냅샷을 기록한다. run_id 를 current_run 으로 설정한다 (AC-AD-5).
    pub fn record_snapshot(&mut self, snap: CostSnapshot) {
        self.current_run = Some(snap.run_id.clone());
        self.tracker.record(snap);
    }

    /// session USD 포맷 문자열 반환 (예: "$0.0500").
    pub fn display_session_usd(&self) -> String {
        let total = self
            .current_run
            .as_ref()
            .map(|id| self.tracker.session_total(id))
            .unwrap_or(0.0);
        format!("${:.4}", total)
    }

    /// 오늘(UTC) USD 포맷 문자열 반환.
    pub fn display_daily_usd(&self) -> String {
        let total = self.tracker.daily_total(SystemTime::now());
        format!("${:.4}", total)
    }

    /// 이번 주(UTC) USD 포맷 문자열 반환.
    ///
    /// 현재 시각을 기준으로 직전 월요일 00:00 UTC 를 week_start 로 사용한다.
    pub fn display_weekly_usd(&self) -> String {
        let week_start = current_week_start();
        let total = self.tracker.weekly_total(week_start);
        format!("${:.4}", total)
    }
}

impl Default for CostPanelView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for CostPanelView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .bg(rgb(tok::BG_ELEVATED))
            .p_3()
            .gap(px(4.))
            .child(div().text_xs().text_color(rgb(tok::FG_MUTED)).child("Cost"))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(format!("Session {}", self.display_session_usd())),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(format!("Daily {}", self.display_daily_usd())),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(format!("Weekly {}", self.display_weekly_usd())),
                    ),
            )
    }
}

// ----------------------------------------------------------------
// 내부 헬퍼
// ----------------------------------------------------------------

/// 현재 시각의 주 시작(월요일 00:00 UTC) 을 반환한다.
fn current_week_start() -> SystemTime {
    let now_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // 1970-01-01 은 목요일이므로 +3일 오프셋으로 월요일 기준
    let days_since_epoch = now_secs / 86400;
    let weekday = (days_since_epoch + 3) % 7; // 0=월요일
    let monday_secs = (days_since_epoch - weekday) * 86400;
    unix_secs_to_system_time(monday_secs)
}

// ================================================================
// 테스트 (RED-GREEN 사이클)
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_agent::cost::unix_secs_to_system_time;

    fn run_id(s: &str) -> AgentRunId {
        AgentRunId(s.to_string())
    }

    fn make_snap(usd: f64, run: &str, ts_secs: u64) -> CostSnapshot {
        CostSnapshot {
            timestamp: unix_secs_to_system_time(ts_secs),
            usd,
            run_id: run_id(run),
        }
    }

    /// 스냅샷 없을 때 session 은 "$0.0000" 반환
    #[test]
    fn display_session_zero_when_empty() {
        let view = CostPanelView::new();
        assert_eq!(view.display_session_usd(), "$0.0000");
    }

    /// 스냅샷 기록 후 session 총계 표시
    #[test]
    fn display_session_after_records() {
        let mut view = CostPanelView::new();
        view.record_snapshot(make_snap(0.01, "r1", 0));
        view.record_snapshot(make_snap(0.04, "r1", 1));
        assert_eq!(view.display_session_usd(), "$0.0500");
    }

    /// 당일 집계 표시 — 동일 날짜 스냅샷 합산
    #[test]
    fn display_daily_aggregates() {
        let mut view = CostPanelView::new();
        // 2024-01-01 UTC
        view.record_snapshot(make_snap(0.02, "r1", 1_704_067_200 + 3600));
        view.record_snapshot(make_snap(0.03, "r1", 1_704_067_200 + 7200));

        let daily = view
            .tracker
            .daily_total(unix_secs_to_system_time(1_704_067_200 + 3600));
        let formatted = format!("${:.4}", daily);
        assert_eq!(formatted, "$0.0500");
    }

    /// record_snapshot 이 current_run 을 업데이트해야 한다
    #[test]
    fn record_snapshot_updates_current_run() {
        let mut view = CostPanelView::new();
        assert!(view.current_run.is_none());

        view.record_snapshot(make_snap(0.01, "r42", 0));
        assert_eq!(view.current_run, Some(run_id("r42")));
    }
}
