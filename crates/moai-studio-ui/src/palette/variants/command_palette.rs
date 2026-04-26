//! CommandPalette variant — Cmd+Shift+P 커맨드 실행.
//!
//! @MX:NOTE: [AUTO] CommandPalette variant — mock 커맨드 레지스트리. 실제 소스는 후속 SPEC.
//! @MX:SPEC: SPEC-V3-012 MS-2 AC-PL-7

use crate::palette::fuzzy::fuzzy_match;
use crate::palette::palette_view::{PaletteEvent, PaletteItem, PaletteView};

// ============================================================
// 커맨드 레지스트리 항목
// ============================================================

/// 커맨드 레지스트리 항목 — id + label.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandEntry {
    /// 커맨드 식별자 (이벤트 페이로드).
    pub id: String,
    /// 목록에 표시되는 레이블.
    pub label: String,
}

impl CommandEntry {
    /// 새 CommandEntry 를 생성한다.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

// ============================================================
// mock 커맨드 레지스트리 (AC-PL-7)
// ============================================================

/// CommandPalette 기본 mock 커맨드 레지스트리.
///
/// N2 (Non-Goal): 실제 커맨드 레지스트리 + 사용자 정의 커맨드 등록은 후속 SPEC.
pub fn default_mock_commands() -> Vec<CommandEntry> {
    vec![
        CommandEntry::new("file.new", "New File"),
        CommandEntry::new("file.open", "Open File"),
        CommandEntry::new("file.save", "Save File"),
        CommandEntry::new("file.save_all", "Save All Files"),
        CommandEntry::new("editor.format", "Format Document"),
        CommandEntry::new("editor.toggle_comment", "Toggle Line Comment"),
        CommandEntry::new("view.toggle_sidebar", "Toggle Sidebar"),
        CommandEntry::new("view.toggle_terminal", "Toggle Terminal"),
        CommandEntry::new("palette.cmd", "Open File Quick Open (Cmd+P)"),
        CommandEntry::new("palette.command", "Open Command Palette (Cmd+Shift+P)"),
    ]
}

// ============================================================
// CommandPalette 이벤트
// ============================================================

/// CommandPalette 에서 발생하는 이벤트.
#[derive(Debug, Clone, PartialEq)]
pub enum CommandPaletteEvent {
    /// 커맨드 실행 확정 — Enter 시 선택된 커맨드 id.
    CommandTriggered(String),
    /// Palette 닫기 요청.
    DismissRequested,
}

// ============================================================
// CommandPalette — 커맨드 실행 variant
// ============================================================

/// CommandPalette — Cmd+Shift+P 커맨드 실행 variant.
///
/// - mock 커맨드 레지스트리에서 데이터 소스를 가져온다 (AC-PL-7).
/// - query 변경 시 fuzzy match 로 필터링한다.
/// - Enter 시 CommandTriggered(command_id) 이벤트를 발생시킨다.
/// - 활성화 keybinding: Cmd+Shift+P (데이터만 선언, 실제 설치는 MS-3).
pub struct CommandPalette {
    /// 공유 PaletteView core.
    pub view: PaletteView,
    /// 전체 커맨드 레지스트리 (필터링 원본).
    command_registry: Vec<CommandEntry>,
    /// 마지막 이벤트.
    pub last_event: Option<CommandPaletteEvent>,
    /// 활성화 keybinding 정보.
    pub activation_key: &'static str,
}

impl CommandPalette {
    /// 기본 mock 커맨드 레지스트리로 새 CommandPalette 를 생성한다.
    pub fn new() -> Self {
        let registry = default_mock_commands();
        let items = commands_to_items(&registry);
        Self {
            view: PaletteView::with_items(items),
            command_registry: registry,
            last_event: None,
            activation_key: "cmd+shift+p",
        }
    }

    /// 커스텀 커맨드 레지스트리로 CommandPalette 를 생성한다 (테스트용).
    pub fn with_commands(commands: Vec<CommandEntry>) -> Self {
        let items = commands_to_items(&commands);
        Self {
            view: PaletteView::with_items(items),
            command_registry: commands,
            last_event: None,
            activation_key: "cmd+shift+p",
        }
    }

    /// query 를 갱신하고 커맨드 레지스트리를 fuzzy 필터링한다 (AC-PL-7).
    pub fn set_query(&mut self, query: String) {
        self.view.set_query(query.clone());
        let filtered = filter_commands(&self.command_registry, &query);
        self.view.set_items(filtered);
    }

    /// 현재 query 로 필터링된 항목 수를 반환한다.
    pub fn filtered_count(&self) -> usize {
        self.view.items.len()
    }

    /// Enter 키 처리 — CommandTriggered 이벤트 발생 (AC-PL-7).
    pub fn on_enter(&mut self) -> Option<CommandPaletteEvent> {
        let event = match self.view.on_enter() {
            Some(PaletteEvent::ItemSelected(item)) => {
                Some(CommandPaletteEvent::CommandTriggered(item.id.clone()))
            }
            _ => None,
        };
        self.last_event = event.clone();
        event
    }

    /// Esc 키 처리 — DismissRequested 이벤트 발생.
    pub fn on_escape(&mut self) -> CommandPaletteEvent {
        self.view.on_escape();
        let event = CommandPaletteEvent::DismissRequested;
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

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 헬퍼
// ============================================================

fn commands_to_items(commands: &[CommandEntry]) -> Vec<PaletteItem> {
    commands
        .iter()
        .map(|c| PaletteItem::new(c.id.clone(), c.label.clone()))
        .collect()
}

fn filter_commands(commands: &[CommandEntry], query: &str) -> Vec<PaletteItem> {
    if query.is_empty() {
        return commands_to_items(commands);
    }
    let mut results: Vec<(i32, PaletteItem)> = commands
        .iter()
        .filter_map(|c| {
            fuzzy_match(query, &c.label)
                .map(|(score, _)| (score, PaletteItem::new(c.id.clone(), c.label.clone())))
        })
        .collect();
    results.sort_by(|a, b| b.0.cmp(&a.0));
    results.into_iter().map(|(_, item)| item).collect()
}

// ============================================================
// 단위 테스트 — AC-PL-7
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_palette() -> CommandPalette {
        CommandPalette::with_commands(vec![
            CommandEntry::new("file.new", "New File"),
            CommandEntry::new("file.open", "Open File"),
            CommandEntry::new("file.save", "Save File"),
            CommandEntry::new("editor.format", "Format Document"),
            CommandEntry::new("view.sidebar", "Toggle Sidebar"),
        ])
    }

    /// 초기 상태에서 전체 커맨드 목록이 표시된다.
    #[test]
    fn initial_shows_all_commands() {
        let palette = make_palette();
        assert_eq!(palette.filtered_count(), 5, "초기 전체 커맨드 표시");
    }

    /// query 로 필터링 시 매칭 커맨드만 표시된다.
    #[test]
    fn filters_by_query() {
        let mut palette = make_palette();
        palette.set_query("file".to_string());
        let count = palette.filtered_count();
        assert!(count > 0, "file 쿼리는 최소 1개 이상 결과");
        assert!(count < 5, "file 쿼리는 전체보다 적은 결과");
    }

    /// AC-PL-7: Enter 시 CommandTriggered 이벤트에 커맨드 id 가 담긴다.
    #[test]
    fn enter_dispatches_command() {
        let mut palette = make_palette();
        // 첫 번째 항목 선택.
        let event = palette.on_enter();
        assert!(event.is_some(), "Enter 시 이벤트 발생");
        if let Some(CommandPaletteEvent::CommandTriggered(id)) = event {
            assert!(!id.is_empty(), "커맨드 id 가 비어있으면 안 됨");
        } else {
            panic!("CommandTriggered 이벤트가 아님");
        }
    }

    /// 특정 커맨드를 검색하고 Enter 로 올바른 id 가 반환된다.
    #[test]
    fn search_and_enter_returns_correct_id() {
        let mut palette = make_palette();
        palette.set_query("format".to_string());
        assert!(palette.filtered_count() > 0, "format 쿼리 결과 있어야 함");
        let event = palette.on_enter();
        if let Some(CommandPaletteEvent::CommandTriggered(id)) = event {
            assert_eq!(id, "editor.format", "format 검색 → editor.format id");
        } else {
            panic!("CommandTriggered 이벤트가 아님");
        }
    }

    /// Esc 시 DismissRequested 이벤트 발생.
    #[test]
    fn escape_emits_dismiss() {
        let mut palette = make_palette();
        let event = palette.on_escape();
        assert_eq!(event, CommandPaletteEvent::DismissRequested);
    }

    /// activation_key 가 "cmd+shift+p" 이다.
    #[test]
    fn activation_key_is_cmd_shift_p() {
        let palette = CommandPalette::new();
        assert_eq!(palette.activation_key, "cmd+shift+p");
    }

    /// 매칭 없는 쿼리 → 빈 목록.
    #[test]
    fn no_match_returns_empty() {
        let mut palette = make_palette();
        palette.set_query("zzz_no_match".to_string());
        assert_eq!(palette.filtered_count(), 0);
    }

    /// 빈 쿼리로 초기화하면 전체 목록 복원.
    #[test]
    fn empty_query_restores_all() {
        let mut palette = make_palette();
        palette.set_query("format".to_string());
        palette.set_query("".to_string());
        assert_eq!(palette.filtered_count(), 5, "빈 쿼리 → 전체 커맨드 복원");
    }
}
