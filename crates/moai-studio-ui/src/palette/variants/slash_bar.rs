//! SlashBar variant — `/moai *` slash command 런처.
//!
//! @MX:NOTE: [AUTO] SlashBar variant — mock /moai * 서브커맨드 목록. 실제 소스는 후속 SPEC.
//! @MX:SPEC: SPEC-V3-012 MS-2 AC-PL-8

use crate::palette::fuzzy::fuzzy_match;
use crate::palette::palette_view::{PaletteEvent, PaletteItem, PaletteView};

// ============================================================
// mock slash command 목록 (AC-PL-8, SPEC §14 Q1)
// ============================================================

/// SlashBar 기본 `/moai *` 서브커맨드 목록.
///
/// Q1 (Open Question 해소): CLAUDE.md §3 기준 `/moai plan`, `/moai run`, `/moai sync`,
/// `/moai project`, `/moai fix`, `/moai design` 6개 기본 노출.
pub const MOCK_SLASH_COMMANDS: &[(&str, &str)] = &[
    ("moai_plan", "/moai plan"),
    ("moai_run", "/moai run"),
    ("moai_sync", "/moai sync"),
    ("moai_project", "/moai project"),
    ("moai_fix", "/moai fix"),
    ("moai_design", "/moai design"),
    ("moai_review", "/moai review"),
    ("moai_loop", "/moai loop"),
];

// ============================================================
// SlashBar 이벤트
// ============================================================

/// SlashBar 에서 발생하는 이벤트.
#[derive(Debug, Clone, PartialEq)]
pub enum SlashBarEvent {
    /// Slash command 실행 확정 — Enter 시 선택된 서브커맨드 이름.
    SlashInvoked(String),
    /// Palette 닫기 요청.
    DismissRequested,
}

// ============================================================
// SlashBar — slash command 런처 variant
// ============================================================

/// SlashBar — terminal pane 에서 `/moai *` 서브커맨드를 선택하는 variant.
///
/// - mock MOCK_SLASH_COMMANDS 에서 데이터 소스를 가져온다 (AC-PL-8).
/// - query 변경 시 fuzzy match 로 필터링한다.
/// - Enter 시 SlashInvoked(subcommand_name) 이벤트를 발생시킨다.
/// - 활성화 keybinding: terminal pane 에서 `/` (데이터만 선언, 실제 설치는 MS-3).
pub struct SlashBar {
    /// 공유 PaletteView core.
    pub view: PaletteView,
    /// 전체 slash command 목록 (필터링 원본).
    slash_commands: Vec<(String, String)>, // (id, label)
    /// 마지막 이벤트.
    pub last_event: Option<SlashBarEvent>,
    /// 활성화 keybinding 정보.
    pub activation_key: &'static str,
}

impl SlashBar {
    /// 기본 mock slash command 목록으로 새 SlashBar 를 생성한다.
    pub fn new() -> Self {
        let slash_commands: Vec<(String, String)> = MOCK_SLASH_COMMANDS
            .iter()
            .map(|(id, label)| (id.to_string(), label.to_string()))
            .collect();
        let items = slash_commands_to_items(&slash_commands);
        Self {
            view: PaletteView::with_items(items),
            slash_commands,
            last_event: None,
            activation_key: "/",
        }
    }

    /// 커스텀 slash command 목록으로 SlashBar 를 생성한다 (테스트용).
    pub fn with_commands(commands: Vec<(String, String)>) -> Self {
        let items = slash_commands_to_items(&commands);
        Self {
            view: PaletteView::with_items(items),
            slash_commands: commands,
            last_event: None,
            activation_key: "/",
        }
    }

    /// query 를 갱신하고 slash command 목록을 fuzzy 필터링한다 (AC-PL-8).
    ///
    /// 쿼리는 `/moai ` prefix 없이 서브커맨드 이름으로 검색한다.
    /// 예: "pl" → "/moai plan" 매칭.
    pub fn set_query(&mut self, query: String) {
        self.view.set_query(query.clone());
        let filtered = filter_slash_commands(&self.slash_commands, &query);
        self.view.set_items(filtered);
    }

    /// 현재 query 로 필터링된 항목 수를 반환한다.
    pub fn filtered_count(&self) -> usize {
        self.view.items.len()
    }

    /// Enter 키 처리 — SlashInvoked 이벤트 발생 (AC-PL-8).
    pub fn on_enter(&mut self) -> Option<SlashBarEvent> {
        let event = match self.view.on_enter() {
            Some(PaletteEvent::ItemSelected(item)) => {
                Some(SlashBarEvent::SlashInvoked(item.label.clone()))
            }
            _ => None,
        };
        self.last_event = event.clone();
        event
    }

    /// Esc 키 처리 — DismissRequested 이벤트 발생.
    pub fn on_escape(&mut self) -> SlashBarEvent {
        self.view.on_escape();
        let event = SlashBarEvent::DismissRequested;
        self.last_event = Some(event.clone());
        event
    }

    /// ArrowDown 처리.
    pub fn on_arrow_down(&mut self) {
        self.view.on_arrow_down();
    }

    /// ArrowUp 처리.
    pub fn on_arrow_up(&mut self) {
        self.view.on_arrow_up();
    }
}

impl Default for SlashBar {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 헬퍼
// ============================================================

fn slash_commands_to_items(commands: &[(String, String)]) -> Vec<PaletteItem> {
    commands
        .iter()
        .map(|(id, label)| PaletteItem::new(id.clone(), label.clone()))
        .collect()
}

fn filter_slash_commands(commands: &[(String, String)], query: &str) -> Vec<PaletteItem> {
    if query.is_empty() {
        return slash_commands_to_items(commands);
    }
    let mut results: Vec<(i32, PaletteItem)> = commands
        .iter()
        .filter_map(|(id, label)| {
            // label 전체에서 검색 (예: "pl" → "/moai plan").
            fuzzy_match(query, label)
                .map(|(score, _)| (score, PaletteItem::new(id.clone(), label.clone())))
        })
        .collect();
    results.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    results.into_iter().map(|(_, item)| item).collect()
}

// ============================================================
// 단위 테스트 — AC-PL-8
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// 초기 상태에서 전체 slash command 목록이 표시된다.
    #[test]
    fn initial_shows_all_commands() {
        let slash_bar = SlashBar::new();
        assert_eq!(
            slash_bar.filtered_count(),
            MOCK_SLASH_COMMANDS.len(),
            "초기 상태 전체 커맨드 표시"
        );
    }

    /// AC-PL-8: "pl" 쿼리로 "/moai plan" 이 포함된 결과가 반환된다 (subsequence match).
    #[test]
    fn filters_moai_commands() {
        let mut slash_bar = SlashBar::new();
        slash_bar.set_query("pl".to_string());
        let count = slash_bar.filtered_count();
        assert!(count > 0, "pl 쿼리는 최소 1개 이상 결과");
        // "/moai plan" 이 결과에 포함되어야 함.
        let has_plan = slash_bar
            .view
            .items
            .iter()
            .any(|item| item.label.contains("plan"));
        assert!(has_plan, "/moai plan 이 결과에 포함되어야 함");
    }

    /// "run" 쿼리로 "/moai run" 이 포함된 결과가 반환된다.
    #[test]
    fn filters_run_command() {
        let mut slash_bar = SlashBar::new();
        slash_bar.set_query("run".to_string());
        let has_run = slash_bar
            .view
            .items
            .iter()
            .any(|item| item.label.contains("run"));
        assert!(has_run, "/moai run 이 결과에 포함되어야 함");
    }

    /// Enter 시 SlashInvoked 이벤트에 서브커맨드 이름이 담긴다.
    #[test]
    fn enter_emits_slash_invoked() {
        let mut slash_bar = SlashBar::new();
        // 첫 번째 항목 선택.
        let event = slash_bar.on_enter();
        assert!(event.is_some(), "Enter 시 이벤트 발생");
        if let Some(SlashBarEvent::SlashInvoked(name)) = event {
            assert!(!name.is_empty(), "서브커맨드 이름이 비어있으면 안 됨");
            // /moai 로 시작해야 함.
            assert!(
                name.starts_with("/moai"),
                "서브커맨드 이름은 /moai 로 시작해야 함: {name}"
            );
        } else {
            panic!("SlashInvoked 이벤트가 아님");
        }
    }

    /// 특정 커맨드 검색 후 Enter 로 올바른 이름 반환.
    #[test]
    fn search_plan_and_enter_returns_correct_name() {
        let mut slash_bar = SlashBar::new();
        slash_bar.set_query("plan".to_string());
        assert!(slash_bar.filtered_count() > 0, "plan 쿼리 결과 있어야 함");
        let event = slash_bar.on_enter();
        if let Some(SlashBarEvent::SlashInvoked(name)) = event {
            assert_eq!(name, "/moai plan", "plan 검색 → /moai plan");
        } else {
            panic!("SlashInvoked 이벤트가 아님");
        }
    }

    /// Esc 시 DismissRequested 이벤트 발생.
    #[test]
    fn escape_emits_dismiss() {
        let mut slash_bar = SlashBar::new();
        let event = slash_bar.on_escape();
        assert_eq!(event, SlashBarEvent::DismissRequested);
    }

    /// activation_key 가 "/" 이다.
    #[test]
    fn activation_key_is_slash() {
        let slash_bar = SlashBar::new();
        assert_eq!(slash_bar.activation_key, "/");
    }

    /// 매칭 없는 쿼리 → 빈 목록.
    #[test]
    fn no_match_returns_empty() {
        let mut slash_bar = SlashBar::new();
        slash_bar.set_query("zzz_no_match".to_string());
        assert_eq!(slash_bar.filtered_count(), 0);
    }

    /// 빈 쿼리로 초기화하면 전체 목록 복원.
    #[test]
    fn empty_query_restores_all() {
        let mut slash_bar = SlashBar::new();
        slash_bar.set_query("plan".to_string());
        slash_bar.set_query("".to_string());
        assert_eq!(
            slash_bar.filtered_count(),
            MOCK_SLASH_COMMANDS.len(),
            "빈 쿼리 → 전체 목록 복원"
        );
    }

    // ── MS-4: SlashInvoked dispatch correctness (AC-PL-21) ──

    /// AC-PL-21: SlashInvoked payload starts with "/moai" for all default commands.
    #[test]
    fn all_default_commands_invoke_with_moai_prefix() {
        for (id, label) in MOCK_SLASH_COMMANDS {
            let mut bar = SlashBar::with_commands(vec![(id.to_string(), label.to_string())]);
            if let Some(SlashBarEvent::SlashInvoked(name)) = bar.on_enter() {
                assert!(
                    name.starts_with("/moai"),
                    "SlashInvoked must start with /moai for command {id}: got {name}"
                );
            } else {
                panic!("Expected SlashInvoked for command {id}");
            }
        }
    }

    /// AC-PL-21: SlashInvoked subcommand matches /moai plan label.
    #[test]
    fn slash_invoked_plan_dispatches_correct_label() {
        let mut bar =
            SlashBar::with_commands(vec![("moai_plan".to_string(), "/moai plan".to_string())]);
        let ev = bar.on_enter();
        assert_eq!(
            ev,
            Some(SlashBarEvent::SlashInvoked("/moai plan".to_string()))
        );
    }

    /// MS-4: subcommand count expands from 8 to cover all /moai subcommands in mock.
    #[test]
    fn mock_slash_commands_covers_essential_subcommands() {
        let essential = ["/moai plan", "/moai run", "/moai sync", "/moai fix"];
        for expected in &essential {
            let has = MOCK_SLASH_COMMANDS
                .iter()
                .any(|(_, label)| label == expected);
            assert!(has, "MOCK_SLASH_COMMANDS must include '{expected}'");
        }
    }
}
