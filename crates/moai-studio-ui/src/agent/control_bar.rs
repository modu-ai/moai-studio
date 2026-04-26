//! AgentControlBar GPUI Entity — pause / resume / kill (RG-AD-5, AC-AD-9/10)
//!
//! SPEC-V3-010 REQ-AD-024: 3 button (pause/resume/kill) toggled per AgentRunStatus.
//! SPEC-V3-010 REQ-AD-025/026/027: stdin envelope 작성 (USER-DECISION-AD-C C1).
//! SPEC-V3-010 REQ-AD-029: kill 은 confirm dialog 없이 즉시 실행 금지.
//!
//! @MX:ANCHOR: [AUTO] agent-control-bar-entity
//! @MX:REASON: [AUTO] agent control UI 단일 진입점. fan_in >= 3:
//!   AgentDashboardView, integration test, future supervisor 연결.
//!   SPEC: SPEC-V3-010 RG-AD-5

use std::io::{self, Write};

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_agent::{AgentRunId, AgentRunStatus, ControlEnvelope, write_envelope};

use crate::design::tokens as tok;

/// AgentControlBar — pause/resume/kill 3 button + confirm modal 상태 (REQ-AD-024/027).
pub struct AgentControlBar {
    /// 현재 대상 run (없으면 모든 button disabled)
    pub run_id: Option<AgentRunId>,
    /// 현재 run 의 진행 상태
    pub status: AgentRunStatus,
    /// kill confirm modal 표시 여부 (REQ-AD-027/029)
    pub kill_confirm_open: bool,
    /// 마지막으로 작성된 envelope (테스트 검증용)
    pub last_envelope: Option<ControlEnvelope>,
}

impl AgentControlBar {
    /// 빈 상태로 새 control bar 를 생성한다 (run 미할당).
    pub fn new() -> Self {
        Self {
            run_id: None,
            status: AgentRunStatus::Completed,
            kill_confirm_open: false,
            last_envelope: None,
        }
    }

    /// 새 run 을 attach 하고 status 를 Running 으로 초기화한다.
    pub fn attach_run(&mut self, run_id: AgentRunId) {
        self.run_id = Some(run_id);
        self.status = AgentRunStatus::Running;
        self.kill_confirm_open = false;
    }

    /// 외부 hook ack 로 status 를 갱신한다 (REQ-AD-028).
    pub fn update_status(&mut self, status: AgentRunStatus) {
        self.status = status;
        if status.is_terminal() {
            self.kill_confirm_open = false;
        }
    }

    /// pause 버튼이 활성화되어 있는지 (REQ-AD-024).
    pub fn pause_enabled(&self) -> bool {
        self.run_id.is_some() && self.status.allows_pause()
    }

    /// resume 버튼이 활성화되어 있는지 (REQ-AD-024).
    pub fn resume_enabled(&self) -> bool {
        self.run_id.is_some() && self.status.allows_resume()
    }

    /// kill 버튼이 활성화되어 있는지 (REQ-AD-024).
    pub fn kill_enabled(&self) -> bool {
        self.run_id.is_some() && self.status.allows_kill()
    }

    /// pause 버튼 클릭 핸들러 (REQ-AD-025).
    /// 비활성 상태에서는 noop, 활성 상태에서는 envelope 작성.
    pub fn click_pause<W: Write>(&mut self, writer: &mut W) -> io::Result<bool> {
        if !self.pause_enabled() {
            return Ok(false);
        }
        let run_id = self.run_id.clone().expect("pause_enabled 시 run_id 보장");
        let env = ControlEnvelope::pause(run_id);
        write_envelope(writer, &env)?;
        self.last_envelope = Some(env);
        // status 는 hook ack 를 기다린다 (낙관적 변경 금지).
        Ok(true)
    }

    /// resume 버튼 클릭 핸들러 (REQ-AD-026).
    pub fn click_resume<W: Write>(&mut self, writer: &mut W) -> io::Result<bool> {
        if !self.resume_enabled() {
            return Ok(false);
        }
        let run_id = self.run_id.clone().expect("resume_enabled 시 run_id 보장");
        let env = ControlEnvelope::resume(run_id);
        write_envelope(writer, &env)?;
        self.last_envelope = Some(env);
        Ok(true)
    }

    /// kill 버튼 클릭 핸들러 — confirm modal 만 연다 (REQ-AD-027/029).
    /// envelope 는 `confirm_kill` 에서만 작성된다.
    pub fn click_kill_open_confirm(&mut self) -> bool {
        if !self.kill_enabled() {
            return false;
        }
        self.kill_confirm_open = true;
        true
    }

    /// kill confirm modal 의 OK 버튼 핸들러 — envelope 작성 (REQ-AD-027).
    pub fn confirm_kill<W: Write>(&mut self, writer: &mut W) -> io::Result<bool> {
        if !self.kill_confirm_open || !self.kill_enabled() {
            return Ok(false);
        }
        let run_id = self.run_id.clone().expect("kill_enabled 시 run_id 보장");
        let env = ControlEnvelope::kill(run_id);
        write_envelope(writer, &env)?;
        self.last_envelope = Some(env);
        self.kill_confirm_open = false;
        Ok(true)
    }

    /// kill confirm modal 의 Cancel 버튼 핸들러 — modal 만 닫는다 (envelope 미작성).
    pub fn cancel_kill(&mut self) {
        self.kill_confirm_open = false;
    }
}

impl Default for AgentControlBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for AgentControlBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // 상태 라벨
        let status_label = match self.status {
            AgentRunStatus::Running => "Running",
            AgentRunStatus::Paused => "Paused",
            AgentRunStatus::Completed => "Completed",
            AgentRunStatus::Failed => "Failed",
            AgentRunStatus::Killed => "Killed",
        };

        let button_color = |enabled: bool| -> u32 {
            if enabled {
                tok::FG_PRIMARY
            } else {
                tok::FG_DISABLED
            }
        };

        let mut bar = div()
            .flex()
            .flex_row()
            .gap(px(8.))
            .py(px(4.))
            .px(px(8.))
            .bg(rgb(tok::BG_ELEVATED))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(tok::FG_MUTED))
                    .child(format!("Status: {}", status_label)),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(button_color(self.pause_enabled())))
                    .child("⏸ pause"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(button_color(self.resume_enabled())))
                    .child("▶ resume"),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(button_color(self.kill_enabled())))
                    .child("⏹ kill"),
            );

        if self.kill_confirm_open {
            bar = bar.child(
                div()
                    .ml(px(12.))
                    .px(px(8.))
                    .py(px(2.))
                    .bg(rgb(tok::BG_SURFACE))
                    .text_xs()
                    .text_color(rgb(tok::FG_PRIMARY))
                    .child("Kill agent run? — [OK] / [Cancel]"),
            );
        }

        bar
    }
}

// ================================================================
// 테스트 (RED-GREEN 사이클)
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn run(s: &str) -> AgentRunId {
        AgentRunId(s.to_string())
    }

    /// AC-AD-9: pause envelope 가 정상적으로 작성된다.
    #[test]
    fn pause_writes_envelope_when_running() {
        let mut bar = AgentControlBar::new();
        bar.attach_run(run("r1"));

        let mut buf: Vec<u8> = Vec::new();
        let written = bar.click_pause(&mut buf).expect("write 성공");
        assert!(written, "pause click 시 envelope 작성됨");

        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains(r#""action":"pause""#));
        assert!(s.contains(r#""run_id":"r1""#));
    }

    /// REQ-AD-024: 비활성 상태에서 pause click 은 noop 이어야 한다.
    #[test]
    fn pause_noop_when_disabled() {
        let mut bar = AgentControlBar::new();
        bar.attach_run(run("r1"));
        bar.update_status(AgentRunStatus::Completed);

        let mut buf: Vec<u8> = Vec::new();
        let written = bar.click_pause(&mut buf).unwrap();
        assert!(!written, "Completed 상태에서 pause 는 noop");
        assert!(buf.is_empty());
    }

    /// REQ-AD-024: button enable 매트릭스 검증.
    #[test]
    fn button_enable_matrix() {
        let mut bar = AgentControlBar::new();

        // run 미할당: 모두 비활성
        assert!(!bar.pause_enabled());
        assert!(!bar.resume_enabled());
        assert!(!bar.kill_enabled());

        bar.attach_run(run("r1")); // → Running
        assert!(bar.pause_enabled());
        assert!(!bar.resume_enabled());
        assert!(bar.kill_enabled());

        bar.update_status(AgentRunStatus::Paused);
        assert!(!bar.pause_enabled());
        assert!(bar.resume_enabled());
        assert!(bar.kill_enabled());

        bar.update_status(AgentRunStatus::Killed);
        assert!(!bar.pause_enabled());
        assert!(!bar.resume_enabled());
        assert!(!bar.kill_enabled());
    }

    /// AC-AD-10: kill click 은 confirm modal 만 열고 envelope 는 작성하지 않아야 한다 (REQ-AD-029).
    #[test]
    fn kill_click_opens_confirm_only() {
        let mut bar = AgentControlBar::new();
        bar.attach_run(run("r1"));

        let opened = bar.click_kill_open_confirm();
        assert!(opened);
        assert!(bar.kill_confirm_open);
        assert!(
            bar.last_envelope.is_none(),
            "confirm 전에 envelope 작성 금지"
        );
    }

    /// AC-AD-10: confirm_kill 호출 시에만 envelope 가 작성된다.
    #[test]
    fn confirm_kill_writes_envelope() {
        let mut bar = AgentControlBar::new();
        bar.attach_run(run("r-kill"));
        bar.click_kill_open_confirm();

        let mut buf: Vec<u8> = Vec::new();
        let written = bar.confirm_kill(&mut buf).unwrap();
        assert!(written);

        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains(r#""action":"kill""#));
        assert!(s.contains(r#""run_id":"r-kill""#));
        assert!(!bar.kill_confirm_open, "confirm 후 modal 닫힘");
    }

    /// AC-AD-10: cancel_kill 은 envelope 작성하지 않고 modal 만 닫는다.
    #[test]
    fn cancel_kill_does_not_write_envelope() {
        let mut bar = AgentControlBar::new();
        bar.attach_run(run("r1"));
        bar.click_kill_open_confirm();

        bar.cancel_kill();
        assert!(!bar.kill_confirm_open);
        assert!(bar.last_envelope.is_none());
    }

    /// REQ-AD-028: terminal status 도착 시 confirm modal 이 강제 닫혀야 한다.
    #[test]
    fn terminal_status_closes_confirm_modal() {
        let mut bar = AgentControlBar::new();
        bar.attach_run(run("r1"));
        bar.click_kill_open_confirm();
        assert!(bar.kill_confirm_open);

        bar.update_status(AgentRunStatus::Completed);
        assert!(!bar.kill_confirm_open);
    }

    /// pause/resume 후 status 는 자동 변경되지 않아야 한다 (낙관적 갱신 금지, hook ack 대기).
    #[test]
    fn click_does_not_optimistically_update_status() {
        let mut bar = AgentControlBar::new();
        bar.attach_run(run("r1"));
        let before = bar.status;

        let mut buf: Vec<u8> = Vec::new();
        bar.click_pause(&mut buf).unwrap();
        assert_eq!(bar.status, before, "click 만으로 status 변경 금지");
    }
}
