//! McpPane — MCP servers read-only viewer (skeleton).
//!
//! SPEC-V3-013 MS-4b (audit G-1, v0.1.2 Task 9b): Settings panel 의 MCP
//! section. v0.1.2 단계는 외부에서 주입된 MCP server 목록을 read-only 로
//! 노출하고 search filter 만 제공한다. server enable/disable 토글, 신규
//! server 추가, settings.json 자동 로드/저장은 후속 SPEC 으로 carry.
//!
//! Frozen zone (REQ-V13-MS4b-1):
//! - moai-studio-terminal/** 무변경
//! - moai-studio-workspace/** 무변경
//! - settings_state.rs 의 다른 SettingsSection variant 동작 무변경
//!   (Mcp variant 추가 + McpPaneState 새로 노출)

use crate::settings::settings_state::{McpPaneState, McpServer};

// ============================================================
// McpPane
// ============================================================

/// McpPane — read-only MCP server 목록 + search filter.
///
/// @MX:NOTE: [AUTO] mcp-pane-skeleton
/// v0.1.2: 외부 주입된 server 목록의 read-only list + filter. CRUD 는 별 SPEC.
pub struct McpPane {
    /// McpPane 이 소유하는 in-memory 상태 (server list + filter).
    pub state: McpPaneState,
}

impl McpPane {
    /// 빈 server 목록과 빈 filter 로 새 McpPane 을 생성한다.
    pub fn new() -> Self {
        Self {
            state: McpPaneState::default(),
        }
    }

    /// 지정 상태로 McpPane 을 생성한다 (테스트 / lib.rs 자동 로드 편의).
    pub fn with_state(state: McpPaneState) -> Self {
        Self { state }
    }

    // ---- section 메타데이터 ----

    /// section 타이틀.
    pub fn title() -> &'static str {
        "MCP"
    }

    /// section 설명.
    pub fn description() -> &'static str {
        "Model Context Protocol 서버 목록을 표시합니다. 서버 활성화 토글 및 추가 / 편집은 향후 버전에서 제공됩니다."
    }

    // ---- server 관리 ----

    /// 외부 (lib.rs) 가 .claude/settings.json 의 mcpServers 를 파싱하여 주입한다.
    pub fn set_servers(&mut self, servers: Vec<McpServer>) {
        self.state.servers = servers;
    }

    /// 현재 등록된 server 의 총 개수 (filter 무시).
    pub fn total_count(&self) -> usize {
        self.state.servers.len()
    }

    /// 현재 filter 가 매치하는 server 만 반환한다.
    pub fn visible_servers(&self) -> Vec<&McpServer> {
        self.state.filtered_servers()
    }

    /// 현재 filter 가 매치하는 server 의 개수.
    pub fn visible_count(&self) -> usize {
        self.state.filtered_servers().len()
    }

    // ---- filter API ----

    /// search filter 를 갱신한다.
    pub fn set_server_filter(&mut self, filter: impl Into<String>) {
        self.state.server_filter = filter.into();
    }

    /// 현재 search filter 를 반환한다.
    pub fn server_filter(&self) -> &str {
        &self.state.server_filter
    }

    /// search filter 를 비운다 (전체 노출 상태로 복귀).
    pub fn clear_server_filter(&mut self) {
        self.state.server_filter.clear();
    }
}

impl Default for McpPane {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — SPEC-V3-013 MS-4b McpPane skeleton
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_servers() -> Vec<McpServer> {
        vec![
            McpServer::new(
                "context7",
                "npx",
                vec!["-y".into(), "@upstash/context7-mcp".into()],
                "stdio",
                true,
            ),
            McpServer::new(
                "playwright",
                "npx",
                vec!["-y".into(), "@playwright/mcp".into()],
                "stdio",
                true,
            ),
            McpServer::new(
                "github",
                "docker",
                vec![
                    "run".into(),
                    "-i".into(),
                    "ghcr.io/github/github-mcp".into(),
                ],
                "stdio",
                false,
            ),
        ]
    }

    /// AC-V13-17: McpPane 타이틀이 "MCP" 이다.
    #[test]
    fn mcp_pane_title_is_mcp() {
        assert_eq!(McpPane::title(), "MCP");
    }

    /// AC-V13-17: 설명이 비어있지 않고 "MCP" 또는 "Model Context Protocol" 을 언급.
    #[test]
    fn mcp_pane_description_mentions_mcp() {
        let desc = McpPane::description();
        assert!(!desc.is_empty(), "description must not be empty");
        assert!(
            desc.contains("MCP") || desc.contains("Model Context Protocol"),
            "description should mention MCP: {desc}"
        );
    }

    /// AC-V13-18: 빈 server 목록의 total/visible/filter 는 모두 정합.
    #[test]
    fn mcp_pane_default_is_empty() {
        let pane = McpPane::new();
        assert_eq!(pane.total_count(), 0);
        assert_eq!(pane.visible_count(), 0);
        assert_eq!(pane.server_filter(), "");
    }

    /// AC-V13-18: set_servers 로 주입한 목록이 그대로 노출된다 (빈 filter).
    #[test]
    fn mcp_pane_set_servers_reflects_total() {
        let mut pane = McpPane::new();
        pane.set_servers(sample_servers());
        assert_eq!(pane.total_count(), 3);
        assert_eq!(pane.visible_count(), 3);
    }

    /// AC-V13-19: filter 는 case-insensitive substring 으로 name 을 매치한다.
    #[test]
    fn mcp_pane_filter_matches_name_case_insensitive() {
        let mut pane = McpPane::with_state(McpPaneState {
            server_filter: String::new(),
            servers: sample_servers(),
        });
        pane.set_server_filter("CONTEXT");
        let visible = pane.visible_servers();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].name, "context7");
    }

    /// AC-V13-19: filter 는 command 도 매치한다 (npx).
    #[test]
    fn mcp_pane_filter_matches_command() {
        let mut pane = McpPane::new();
        pane.set_servers(sample_servers());
        pane.set_server_filter("npx");
        let visible = pane.visible_servers();
        assert_eq!(visible.len(), 2, "context7 + playwright use npx");
        assert!(visible.iter().any(|s| s.name == "context7"));
        assert!(visible.iter().any(|s| s.name == "playwright"));
    }

    /// AC-V13-19: 매치 없는 filter 는 빈 결과.
    #[test]
    fn mcp_pane_filter_no_match_returns_empty() {
        let mut pane = McpPane::new();
        pane.set_servers(sample_servers());
        pane.set_server_filter("nonexistent-server-zzz");
        assert_eq!(pane.visible_count(), 0);
    }

    /// AC-V13-20: clear_server_filter 가 전체 목록을 복원한다.
    #[test]
    fn mcp_pane_clear_filter_restores_full_list() {
        let mut pane = McpPane::new();
        pane.set_servers(sample_servers());
        pane.set_server_filter("github");
        assert_eq!(pane.visible_count(), 1);
        pane.clear_server_filter();
        assert_eq!(pane.server_filter(), "");
        assert_eq!(pane.visible_count(), 3);
    }

    /// AC-V13-21: enabled 와 disabled server 모두 노출된다 (read-only).
    #[test]
    fn mcp_pane_includes_disabled_servers() {
        let mut pane = McpPane::new();
        pane.set_servers(sample_servers());
        let visible = pane.visible_servers();
        let disabled_count = visible.iter().filter(|s| !s.enabled).count();
        let enabled_count = visible.iter().filter(|s| s.enabled).count();
        assert_eq!(enabled_count, 2);
        assert_eq!(disabled_count, 1, "github server is disabled");
    }

    /// with_state 생성자가 servers + filter 를 모두 보존한다.
    #[test]
    fn mcp_pane_with_state_preserves_both_fields() {
        let state = McpPaneState {
            server_filter: "play".to_string(),
            servers: sample_servers(),
        };
        let pane = McpPane::with_state(state);
        assert_eq!(pane.server_filter(), "play");
        assert_eq!(pane.total_count(), 3);
        assert_eq!(pane.visible_count(), 1, "only playwright matches 'play'");
    }
}
