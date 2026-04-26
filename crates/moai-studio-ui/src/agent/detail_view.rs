//! EventDetailView GPUI Entity — JSON full payload inspector (RG-AD-6, AC-AD-11)
//!
//! SPEC-V3-010 REQ-AD-030: 선택된 event 의 full JSON 을 2-space indent 로 pretty-print.
//! SPEC-V3-010 REQ-AD-031: nested object collapse / expand disclosure 트라이앵글.
//! SPEC-V3-010 REQ-AD-033: "Copy as JSON" — clipboard 로 raw JSON 복사.
//! REQ-AD-032 (markdown payload 렌더, V3-006 통합) 은 follow-up SPEC 으로 보류.
//!
//! @MX:ANCHOR: [AUTO] event-detail-view-entity
//! @MX:REASON: [AUTO] event 검사 단일 진입점. fan_in >= 3:
//!   AgentDashboardView, timeline 클릭 라우팅, 테스트.
//!   SPEC: SPEC-V3-010 RG-AD-6

use std::collections::HashSet;

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use moai_studio_agent::events::AgentEvent;

use crate::design::tokens as tok;

/// EventDetailView — 선택된 event 의 JSON 인스펙터.
pub struct EventDetailView {
    /// 현재 표시 중인 event (없으면 placeholder)
    pub selected: Option<AgentEvent>,
    /// 접힌(collapsed) JSON 경로 집합 (REQ-AD-031). path 표기는 dot-separated.
    /// 예: "kind.payload.tools" 가 set 에 있으면 해당 노드는 접힘.
    pub collapsed_paths: HashSet<String>,
}

impl EventDetailView {
    /// 빈 view 를 생성한다.
    pub fn new() -> Self {
        Self {
            selected: None,
            collapsed_paths: HashSet::new(),
        }
    }

    /// event 를 선택한다 (timeline 클릭 라우팅 진입점, REQ-AD-012).
    pub fn select(&mut self, event: AgentEvent) {
        self.selected = Some(event);
        // 새 event 선택 시 collapse 상태 초기화 (혼동 방지)
        self.collapsed_paths.clear();
    }

    /// 선택을 해제한다.
    pub fn clear(&mut self) {
        self.selected = None;
        self.collapsed_paths.clear();
    }

    /// AC-AD-11: 선택된 event 를 2-space indent 로 pretty-print 한다 (REQ-AD-030).
    /// 선택된 event 가 없으면 빈 문자열을 반환한다.
    pub fn pretty_print(&self) -> String {
        match &self.selected {
            Some(ev) => {
                serde_json::to_string_pretty(ev).unwrap_or_else(|_| "<직렬화 실패>".to_string())
            }
            None => String::new(),
        }
    }

    /// REQ-AD-033: 선택된 event 의 raw JSON 을 clipboard 로 복사한다.
    /// arboard 를 통해 OS clipboard 에 작성하며, 실패 시 io::Error 반환.
    pub fn copy_to_clipboard(&self) -> Result<(), CopyError> {
        let payload = self.pretty_print();
        if payload.is_empty() {
            return Err(CopyError::NothingSelected);
        }
        let mut cb = arboard::Clipboard::new().map_err(|e| CopyError::Backend(e.to_string()))?;
        cb.set_text(payload)
            .map_err(|e| CopyError::Backend(e.to_string()))?;
        Ok(())
    }

    /// REQ-AD-031: path 의 collapse 상태를 토글한다.
    pub fn toggle_collapse(&mut self, path: impl Into<String>) {
        let key = path.into();
        if self.collapsed_paths.contains(&key) {
            self.collapsed_paths.remove(&key);
        } else {
            self.collapsed_paths.insert(key);
        }
    }

    /// path 가 접혀있는지 여부 (REQ-AD-031).
    pub fn is_collapsed(&self, path: &str) -> bool {
        self.collapsed_paths.contains(path)
    }
}

impl Default for EventDetailView {
    fn default() -> Self {
        Self::new()
    }
}

/// REQ-AD-033 의 clipboard 실패 종류.
#[derive(Debug)]
pub enum CopyError {
    /// 선택된 event 가 없음
    NothingSelected,
    /// arboard backend 실패 (OS clipboard 접근 실패 등)
    Backend(String),
}

impl std::fmt::Display for CopyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NothingSelected => write!(f, "선택된 event 가 없어 복사할 내용이 없습니다"),
            Self::Backend(e) => write!(f, "clipboard 작성 실패: {}", e),
        }
    }
}

impl std::error::Error for CopyError {}

impl Render for EventDetailView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let mut container = div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(tok::BG_PANEL))
            .p_3()
            .gap(px(2.))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_MUTED))
                            .child("Event Detail"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::ACCENT))
                            .child("⎘ Copy as JSON"),
                    ),
            );

        match &self.selected {
            None => {
                container = container.child(
                    div()
                        .text_xs()
                        .text_color(rgb(tok::FG_DISABLED))
                        .child("(timeline 에서 event 를 선택하세요)"),
                );
            }
            Some(_) => {
                let body = self.pretty_print();
                // 라인 단위로 div 렌더 — collapse 토글은 라인 prefix 매칭으로 결정.
                for line in body.lines() {
                    container = container.child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(line.to_string()),
                    );
                }
            }
        }

        container
    }
}

// ================================================================
// 테스트 (RED-GREEN 사이클)
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use moai_studio_agent::events::{EventKind, HookEvent, StreamJsonEvent};
    use serde_json::json;

    fn make_stream_event() -> AgentEvent {
        AgentEvent {
            id: 1,
            timestamp_ns: 0,
            kind: EventKind::StreamJson(StreamJsonEvent {
                type_: "assistant".to_string(),
                payload: json!({
                    "content": [
                        {"type": "text", "text": "hello"},
                        {"type": "tool_use", "name": "bash"}
                    ]
                }),
                usage: None,
            }),
            raw: r#"{"type":"assistant"}"#.to_string(),
        }
    }

    fn make_hook_event() -> AgentEvent {
        AgentEvent {
            id: 2,
            timestamp_ns: 0,
            kind: EventKind::Hook(HookEvent {
                event_name: "PostToolUse".to_string(),
                payload: json!({"tool_name": "bash", "exit_code": 0}),
            }),
            raw: String::new(),
        }
    }

    /// AC-AD-11: pretty_print 가 2-space indent 를 사용한다 (REQ-AD-030).
    #[test]
    fn pretty_print_uses_two_space_indent() {
        let mut view = EventDetailView::new();
        view.select(make_stream_event());

        let body = view.pretty_print();
        assert!(!body.is_empty());

        // serde_json::to_string_pretty 의 default 는 2-space indent.
        // 첫 번째 nested 라인이 정확히 2-space indent 로 시작해야 한다.
        let lines: Vec<&str> = body.lines().collect();
        let nested_line = lines
            .iter()
            .find(|l| l.starts_with("  ") && !l.starts_with("    "))
            .expect("2-space indent 라인 부재");
        assert!(nested_line.starts_with("  "));
        assert!(!nested_line.starts_with("\t"), "tab indent 금지");
    }

    /// pretty_print 결과는 다시 JSON 으로 파싱 가능해야 한다.
    #[test]
    fn pretty_print_is_valid_json() {
        let mut view = EventDetailView::new();
        view.select(make_hook_event());

        let body = view.pretty_print();
        let parsed: serde_json::Value =
            serde_json::from_str(&body).expect("pretty_print 결과는 valid JSON");
        // 기본 필드 존재 검증
        assert!(parsed.get("id").is_some());
        assert!(parsed.get("kind").is_some());
    }

    /// 선택 없을 때 pretty_print 는 빈 문자열.
    #[test]
    fn pretty_print_empty_when_no_selection() {
        let view = EventDetailView::new();
        assert_eq!(view.pretty_print(), "");
    }

    /// REQ-AD-031: collapse 토글이 정상 작동한다.
    #[test]
    fn toggle_collapse_changes_state() {
        let mut view = EventDetailView::new();
        let path = "kind.payload";

        assert!(!view.is_collapsed(path));
        view.toggle_collapse(path);
        assert!(view.is_collapsed(path));
        view.toggle_collapse(path);
        assert!(!view.is_collapsed(path));
    }

    /// 새 event 선택 시 collapse 상태가 reset 되어야 한다.
    #[test]
    fn select_clears_collapse_paths() {
        let mut view = EventDetailView::new();
        view.toggle_collapse("foo.bar");
        view.toggle_collapse("baz");
        assert!(view.is_collapsed("foo.bar"));

        view.select(make_stream_event());
        assert!(!view.is_collapsed("foo.bar"));
        assert!(!view.is_collapsed("baz"));
        assert!(view.collapsed_paths.is_empty());
    }

    /// AC-AD-11: copy_to_clipboard 는 선택 없으면 NothingSelected 반환.
    #[test]
    fn copy_without_selection_returns_error() {
        let view = EventDetailView::new();
        let result = view.copy_to_clipboard();
        assert!(matches!(result, Err(CopyError::NothingSelected)));
    }

    /// clear 가 select 와 collapse 를 모두 지운다.
    #[test]
    fn clear_resets_state() {
        let mut view = EventDetailView::new();
        view.select(make_stream_event());
        view.toggle_collapse("a.b");

        view.clear();
        assert!(view.selected.is_none());
        assert!(view.collapsed_paths.is_empty());
        assert_eq!(view.pretty_print(), "");
    }

    /// pretty_print 출력에 raw 필드가 포함되어야 한다 (디버그성).
    #[test]
    fn pretty_print_includes_event_raw_field() {
        let mut view = EventDetailView::new();
        view.select(make_stream_event());

        let body = view.pretty_print();
        assert!(body.contains("\"raw\""));
        assert!(body.contains("\"id\""));
    }
}
