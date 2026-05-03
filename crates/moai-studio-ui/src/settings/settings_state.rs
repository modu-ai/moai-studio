//! SettingsViewState — SettingsModal 의 transient UI 상태.
//!
//! 영속화 대상이 아닌 순수 런타임 뷰 상태만 보유한다.
//! UserSettings 영속화 + ActiveTheme 런타임 적용은 MS-3 단계.

use serde::{Deserialize, Serialize};

// ============================================================
// Section 열거형 (6 sections)
// ============================================================

/// SettingsModal 의 10개 section 식별자.
///
/// MS-4a (SPEC-V3-013, v0.1.2 Task 9a): `Hooks` variant 추가 (audit G-1).
/// MS-4b (SPEC-V3-013, v0.1.2 Task 9b): `Mcp` variant 추가 (audit G-1).
/// MS-4c (SPEC-V3-013, v0.1.2 Task 9c): `Skills` variant 추가 (audit G-1).
/// MS-4d (SPEC-V3-013, v0.1.2 Task 9d): `Rules` variant 추가 (audit G-1).
///
/// @MX:ANCHOR: [AUTO] settings-section-enum
/// @MX:REASON: [AUTO] sidebar row 렌더, main pane swap, selected_section 상태 전이의 공통 타입.
///   fan_in >= 3: settings_state.rs, settings_modal.rs, panes/*.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    /// 외모 설정 (theme/density/accent/font_size)
    Appearance,
    /// 키보드 단축키 설정
    Keyboard,
    /// 에디터 설정 (skeleton)
    Editor,
    /// 터미널 설정 (skeleton)
    Terminal,
    /// 에이전트 설정 (skeleton)
    Agent,
    /// Claude Code Hooks 설정 (read-only skeleton, MS-4a)
    Hooks,
    /// MCP servers 설정 (read-only skeleton, MS-4b)
    Mcp,
    /// Claude Code Skills 설정 (read-only skeleton, MS-4c)
    Skills,
    /// Claude Code Rules 설정 (read-only skeleton, MS-4d)
    Rules,
    /// 고급 설정 (skeleton)
    Advanced,
    /// Claude Code Plugins (read-only skeleton, SPEC-V0-2-0-PLUGIN-MGR-001 MS-1)
    Plugins,
}

impl SettingsSection {
    /// sidebar 에 표시할 레이블 문자열.
    pub fn label(self) -> &'static str {
        match self {
            Self::Appearance => "Appearance",
            Self::Keyboard => "Keyboard",
            Self::Editor => "Editor",
            Self::Terminal => "Terminal",
            Self::Agent => "Agent",
            Self::Hooks => "Hooks",
            Self::Mcp => "MCP",
            Self::Skills => "Skills",
            Self::Rules => "Rules",
            Self::Advanced => "Advanced",
            Self::Plugins => "Plugins",
        }
    }

    /// 11개 section 을 정해진 순서대로 반환한다 (REQ-V13-010 + MS-4a/4b/4c/4d + SPEC-V0-2-0-PLUGIN-MGR-001 MS-1).
    ///
    /// 순서는 sidebar 표시 순서와 일치한다. 새 카탈로그 패널들 (Hooks/MCP/
    /// Skills/Rules) 은 Agent 다음 위치에 인접 그룹으로 배치한다. Plugins 는
    /// enum schema 호환성을 위해 enum 끝 (Advanced 다음 = 11번째) 에 배치한다.
    pub fn all() -> [SettingsSection; 11] {
        [
            Self::Appearance,
            Self::Keyboard,
            Self::Editor,
            Self::Terminal,
            Self::Agent,
            Self::Hooks,
            Self::Mcp,
            Self::Skills,
            Self::Rules,
            Self::Advanced,
            Self::Plugins,
        ]
    }
}

// ============================================================
// AppearanceState — in-memory (MS-3 이전 영속화 없음)
// ============================================================

/// 테마 선택 (dark/light/system).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    /// 다크 테마 (기본값)
    #[default]
    Dark,
    /// 라이트 테마
    Light,
    /// 시스템 설정 따름
    System,
}

/// 밀도 선택 (compact/comfortable).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Density {
    /// 컴팩트 — 패딩/행 높이 0.85x 축소
    Compact,
    /// 컴포터블 — 기본 간격 (기본값)
    #[default]
    Comfortable,
}

/// 액센트 색상 선택 (4종).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AccentColor {
    /// 틸 청록 (브랜드 기본 — 0x144a46) (기본값)
    #[default]
    Teal,
    /// 블루 (0x2563eb)
    Blue,
    /// 바이올렛 (0x6a4cc7)
    Violet,
    /// 시안 (0x06b6d4)
    Cyan,
}

impl AccentColor {
    /// design::tokens 의 ide_accent 상수를 반환한다.
    pub fn hex_value(self) -> u32 {
        use crate::design::tokens::ide_accent;
        match self {
            Self::Teal => ide_accent::TEAL,
            Self::Blue => ide_accent::BLUE,
            Self::Violet => ide_accent::VIOLET,
            Self::Cyan => ide_accent::CYAN,
        }
    }
}

/// AppearancePane 의 in-memory 상태 (MS-1 범위 — 영속화 없음).
#[derive(Debug, Clone, PartialEq)]
pub struct AppearanceState {
    /// 테마 선택 (default: Dark)
    pub theme: ThemeMode,
    /// 밀도 선택 (default: Comfortable)
    pub density: Density,
    /// 액센트 색상 (default: Teal)
    pub accent: AccentColor,
    /// 폰트 크기 (12~18px, default: 14)
    pub font_size_px: u8,
}

impl Default for AppearanceState {
    fn default() -> Self {
        Self {
            theme: ThemeMode::default(),
            density: Density::default(),
            accent: AccentColor::default(),
            font_size_px: 14,
        }
    }
}

impl AppearanceState {
    /// font_size_px 를 설정한다. 12~18 범위 외는 무시한다 (REQ-V13-025).
    pub fn set_font_size(&mut self, px: u8) -> bool {
        if (12..=18).contains(&px) {
            self.font_size_px = px;
            true
        } else {
            false
        }
    }

    /// density 에 따른 spacing multiplier 를 반환한다.
    pub fn spacing_multiplier(&self) -> f32 {
        match self.density {
            Density::Compact => 0.85,
            Density::Comfortable => 1.0,
        }
    }
}

// ============================================================
// KeyBinding — 키보드 단축키 바인딩 (MS-2)
// ============================================================

/// 단일 키보드 바인딩 엔트리.
///
/// @MX:ANCHOR: [AUTO] key-binding-struct
/// @MX:REASON: [AUTO] KeyboardPane 테이블 행, conflict_check, 기본값 목록의 공통 타입.
///   fan_in >= 3: keyboard.rs, settings_state.rs, settings_modal.rs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyBinding {
    /// 액션 식별자 (예: "command_palette", "open.settings").
    pub action_id: String,
    /// 사용자 표시용 레이블.
    pub label: String,
    /// 현재 할당된 단축키 (예: "Cmd+Shift+P", "Cmd+,").
    pub shortcut: String,
}

impl KeyBinding {
    /// 새 KeyBinding 을 생성한다.
    pub fn new(
        action_id: impl Into<String>,
        label: impl Into<String>,
        shortcut: impl Into<String>,
    ) -> Self {
        Self {
            action_id: action_id.into(),
            label: label.into(),
            shortcut: shortcut.into(),
        }
    }
}

/// 기본 키보드 바인딩 목록 (10개 이상, REQ-V13-030).
///
/// @MX:NOTE: [AUTO] default-key-bindings
/// 내부 충돌 없음 — 각 shortcut 이 유일하다. 초기화 시 conflict_check 로 검증.
pub fn default_key_bindings() -> Vec<KeyBinding> {
    vec![
        KeyBinding::new("open.command_palette", "명령 팔레트 열기", "Cmd+Shift+P"),
        KeyBinding::new("open.settings", "설정 열기", "Cmd+,"),
        KeyBinding::new("panes.split_horizontal", "수평 분할", "Cmd+\\"),
        KeyBinding::new("panes.split_vertical", "수직 분할", "Cmd+Shift+\\"),
        KeyBinding::new("panes.close", "패인 닫기", "Cmd+W"),
        KeyBinding::new("panes.focus_left", "왼쪽 패인 포커스", "Cmd+Left"),
        KeyBinding::new("panes.focus_right", "오른쪽 패인 포커스", "Cmd+Right"),
        KeyBinding::new("tabs.new", "새 탭", "Cmd+T"),
        KeyBinding::new("tabs.close", "탭 닫기", "Cmd+Shift+W"),
        KeyBinding::new("terminal.toggle", "터미널 토글", "Ctrl+`"),
        KeyBinding::new("agent.toggle", "에이전트 패널 토글", "Cmd+Shift+A"),
        KeyBinding::new("file.save", "파일 저장", "Cmd+S"),
    ]
}

// ============================================================
// KeyboardState — KeyboardPane in-memory 상태 (MS-2)
// ============================================================

/// KeyboardPane 의 편집 다이얼로그 상태.
#[derive(Debug, Clone, PartialEq)]
pub struct EditDialogState {
    /// 편집 중인 바인딩의 action_id.
    pub action_id: String,
    /// 현재 입력 중인 단축키 (아직 저장되지 않음).
    pub pending_shortcut: String,
    /// 충돌 정보 — Some(충돌_action_label) 이면 충돌 발생.
    pub conflict_error: Option<String>,
}

impl EditDialogState {
    /// 새 편집 다이얼로그 상태를 생성한다.
    pub fn new(action_id: impl Into<String>, current_shortcut: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            pending_shortcut: current_shortcut.into(),
            conflict_error: None,
        }
    }
}

/// KeyboardPane 의 in-memory 상태.
pub struct KeyboardState {
    /// 현재 모든 바인딩 목록 (기본 + 커스텀 포함).
    pub bindings: Vec<KeyBinding>,
    /// 편집 다이얼로그 표시 중인지 여부 (None = 닫힘).
    pub edit_dialog: Option<EditDialogState>,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            bindings: default_key_bindings(),
            edit_dialog: None,
        }
    }
}

impl KeyboardState {
    /// 새 KeyboardState 를 기본 바인딩으로 생성한다.
    pub fn new() -> Self {
        Self::default()
    }

    /// 충돌 검사 — 동일 shortcut 이 다른 action_id 에 이미 할당되어 있으면 Some(레이블) 반환.
    ///
    /// @MX:ANCHOR: [AUTO] conflict-check-fn
    /// @MX:REASON: [AUTO] KeyboardPane 의 핵심 안전 로직. AC-V13-8 의 pass/fail 양 케이스를 보증.
    ///   fan_in >= 3: keyboard.rs, settings_state.rs, settings_modal.rs (MS-2/MS-3).
    pub fn conflict_check(&self, action_id: &str, new_shortcut: &str) -> Option<String> {
        for binding in &self.bindings {
            if binding.action_id != action_id && binding.shortcut == new_shortcut {
                return Some(binding.label.clone());
            }
        }
        None
    }

    /// 바인딩을 업데이트한다. 충돌 시 Err(충돌_레이블), 성공 시 Ok(()) 반환.
    pub fn apply_binding(
        &mut self,
        action_id: &str,
        new_shortcut: impl Into<String>,
    ) -> Result<(), String> {
        let new_shortcut = new_shortcut.into();
        if let Some(conflict_label) = self.conflict_check(action_id, &new_shortcut) {
            return Err(conflict_label);
        }
        if let Some(binding) = self.bindings.iter_mut().find(|b| b.action_id == action_id) {
            binding.shortcut = new_shortcut;
            Ok(())
        } else {
            Err(format!("action_id '{action_id}' 를 찾을 수 없습니다"))
        }
    }

    /// 편집 다이얼로그를 열고 EditDialogState 를 초기화한다.
    pub fn open_edit_dialog(&mut self, action_id: &str) {
        if let Some(binding) = self.bindings.iter().find(|b| b.action_id == action_id) {
            self.edit_dialog = Some(EditDialogState::new(action_id, &binding.shortcut));
        }
    }

    /// 편집 다이얼로그를 닫는다.
    pub fn close_edit_dialog(&mut self) {
        self.edit_dialog = None;
    }

    /// pending_shortcut 을 저장 시도한다. 충돌 시 dialog 에 오류 설정.
    pub fn save_edit_dialog(&mut self) -> bool {
        if let Some(dialog) = self.edit_dialog.take() {
            let result = self.apply_binding(&dialog.action_id, &dialog.pending_shortcut);
            match result {
                Ok(()) => true,
                Err(conflict_label) => {
                    // 오류 상태로 다이얼로그를 다시 열어둠
                    self.edit_dialog = Some(EditDialogState {
                        conflict_error: Some(conflict_label),
                        ..dialog
                    });
                    false
                }
            }
        } else {
            false
        }
    }
}

// ============================================================
// EditorState — EditorPane in-memory 상태 (MS-2 skeleton)
// ============================================================

/// EditorPane 의 in-memory 상태 (v0.1.0 skeleton — 1 setting).
#[derive(Debug, Clone, PartialEq)]
pub struct EditorState {
    /// 탭 크기 (2~8, default 4) — REQ-V13-040.
    pub tab_size: u8,
}

impl Default for EditorState {
    fn default() -> Self {
        Self { tab_size: 4 }
    }
}

impl EditorState {
    /// tab_size 를 설정한다. 2~8 범위 외는 무시.
    pub fn set_tab_size(&mut self, size: u8) -> bool {
        if (2..=8).contains(&size) {
            self.tab_size = size;
            true
        } else {
            false
        }
    }
}

// ============================================================
// TerminalState — TerminalPane in-memory 상태 (MS-2 skeleton)
// ============================================================

/// TerminalPane 의 in-memory 상태 (v0.1.0 skeleton — 1 setting).
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalState {
    /// 스크롤백 줄 수 (1000~100000, default 10000) — REQ-V13-041.
    pub scrollback_lines: u32,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            scrollback_lines: 10_000,
        }
    }
}

impl TerminalState {
    /// scrollback_lines 를 설정한다. 1000~100000 범위 외는 무시.
    pub fn set_scrollback_lines(&mut self, lines: u32) -> bool {
        if (1_000..=100_000).contains(&lines) {
            self.scrollback_lines = lines;
            true
        } else {
            false
        }
    }
}

// ============================================================
// AgentState — AgentPane in-memory 상태 (MS-2 skeleton)
// ============================================================

/// AgentPane 의 in-memory 상태 (v0.1.0 skeleton — 1 setting).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AgentState {
    /// 자동 승인 여부 (default false) — REQ-V13-042.
    pub auto_approve: bool,
}

impl AgentState {
    /// auto_approve 를 토글한다.
    pub fn toggle_auto_approve(&mut self) {
        self.auto_approve = !self.auto_approve;
    }
}

// ============================================================
// PluginsState — PluginsPane in-memory state (SPEC-V0-2-0-PLUGIN-MGR-001 MS-1)
// ============================================================

/// PluginsPane 의 read-only in-memory 상태 (REQ-PM-007).
///
/// audit Top 8 #3 (Plugin Manager UI) skeleton. v0.2.0 단계는 6 개 bundled
/// plugin info 를 read-only 로 노출만 하며, install / uninstall / enable
/// 토글과 marketplace fetch 는 별 SPEC carry.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PluginsState {
    /// User-entered search query — empty string surfaces all entries.
    pub plugin_filter: String,
}

impl PluginsState {
    /// Returns the entries whose `name` or `description` (case-insensitive)
    /// contain `plugin_filter`. Empty filter returns all entries unchanged.
    pub fn filtered_plugins<'a, T, F>(&self, entries: &'a [T], project: F) -> Vec<&'a T>
    where
        F: Fn(&T) -> (&str, &str),
    {
        if self.plugin_filter.is_empty() {
            return entries.iter().collect();
        }
        let needle = self.plugin_filter.to_ascii_lowercase();
        entries
            .iter()
            .filter(|e| {
                let (name, desc) = project(e);
                name.to_ascii_lowercase().contains(&needle)
                    || desc.to_ascii_lowercase().contains(&needle)
            })
            .collect()
    }
}

// ============================================================
// HooksState — HooksPane in-memory state (MS-4a skeleton)
// ============================================================

/// HooksPane 의 read-only in-memory 상태 (v0.1.2 Task 9 / MS-4a).
///
/// audit G-1 (Settings Hooks pane) skeleton. v0.1.2 단계에서는 Claude Code
/// hook event 카탈로그를 read-only 로 노출만 하며, enable/disable 토글과
/// hook script 편집은 향후 SPEC 으로 carry.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HooksState {
    /// 사용자 입력 search query — 빈 문자열이면 전체 노출.
    pub event_filter: String,
}

impl HooksState {
    /// `event_filter` 에 매치되는 known hook event 만 반환한다.
    ///
    /// 매치는 case-insensitive substring. 빈 filter 는 전체를 반환한다.
    /// `'static` 보장은 입력 slice 의 element 들이 `&'static str` 인 데서 온다.
    pub fn filtered_events(&self, events: &[&'static str]) -> Vec<&'static str> {
        if self.event_filter.is_empty() {
            return events.to_vec();
        }
        let needle = self.event_filter.to_ascii_lowercase();
        events
            .iter()
            .copied()
            .filter(|name| name.to_ascii_lowercase().contains(&needle))
            .collect()
    }
}

// ============================================================
// McpServer + McpPaneState — McpPane in-memory state (MS-4b skeleton)
// ============================================================

/// 단일 MCP server 의 read-only 메타데이터.
///
/// SPEC-V3-013 MS-4b (audit G-1) — settings.json `mcpServers` 항목과
/// 호환되는 최소 필드만 노출한다. 실제 server 활성화 / 편집은 후속 SPEC.
#[derive(Debug, Clone, PartialEq)]
pub struct McpServer {
    /// settings.json 의 server 키 (e.g. "context7").
    pub name: String,
    /// 실행 명령 (e.g. "npx").
    pub command: String,
    /// 명령 인자 목록 (e.g. ["-y", "@upstash/context7-mcp"]).
    pub args: Vec<String>,
    /// transport 종류 (stdio/http/sse 등). 미상이면 "stdio" 기본값.
    pub transport: String,
    /// settings.json 또는 mcp.json 에서 활성화된 상태인지 여부.
    pub enabled: bool,
}

impl McpServer {
    /// 새 McpServer 를 생성한다.
    pub fn new(
        name: impl Into<String>,
        command: impl Into<String>,
        args: Vec<String>,
        transport: impl Into<String>,
        enabled: bool,
    ) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            args,
            transport: transport.into(),
            enabled,
        }
    }
}

/// McpPane 의 read-only in-memory 상태 (v0.1.2 Task 9b / MS-4b).
///
/// 외부 (lib.rs) 가 .claude/settings.json 또는 ~/.claude/mcp.json 을 파싱하여
/// `set_servers` 로 주입한다. v0.1.2 단계에서는 자동 로드 wiring 미포함.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct McpPaneState {
    /// 사용자 입력 search query — 빈 문자열이면 전체 노출.
    pub server_filter: String,
    /// 외부에서 주입된 server 목록.
    pub servers: Vec<McpServer>,
}

impl McpPaneState {
    /// `server_filter` 에 매치되는 server 만 반환한다.
    ///
    /// 매치는 case-insensitive substring 으로 name + command 양쪽을 검사.
    /// 빈 filter 는 전체를 반환한다.
    pub fn filtered_servers(&self) -> Vec<&McpServer> {
        if self.server_filter.is_empty() {
            return self.servers.iter().collect();
        }
        let needle = self.server_filter.to_ascii_lowercase();
        self.servers
            .iter()
            .filter(|s| {
                s.name.to_ascii_lowercase().contains(&needle)
                    || s.command.to_ascii_lowercase().contains(&needle)
            })
            .collect()
    }
}

// ============================================================
// Skill + SkillsPaneState — SkillsPane in-memory state (MS-4c skeleton)
// ============================================================

/// 단일 Claude Code Skill 의 read-only 메타데이터.
///
/// SPEC-V3-013 MS-4c (audit G-1) — `~/.claude/skills/*` 또는 plugin-bundled
/// skills 의 frontmatter 와 호환되는 최소 필드만 노출한다. 실제 토글 /
/// 편집은 후속 SPEC.
#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    /// skill 의 고유 이름 (frontmatter `name:` 또는 디렉터리 이름).
    pub name: String,
    /// frontmatter `description:` (없으면 빈 문자열).
    pub description: String,
    /// 등록된 source — "user" / "project" / 플러그인 namespace.
    pub source: String,
    /// 활성화 여부 (settings.json `disabledSkills` 미포함이면 true).
    pub enabled: bool,
}

impl Skill {
    /// 새 Skill 메타데이터를 생성한다.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        source: impl Into<String>,
        enabled: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            source: source.into(),
            enabled,
        }
    }
}

/// SkillsPane 의 read-only in-memory 상태 (v0.1.2 Task 9c / MS-4c).
///
/// 외부 (lib.rs) 가 ~/.claude/skills/ 또는 plugin manifest 를 스캔하여
/// `set_skills` 로 주입한다. v0.1.2 단계에서는 자동 로드 wiring 미포함.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SkillsPaneState {
    /// 사용자 입력 search query — 빈 문자열이면 전체 노출.
    pub skill_filter: String,
    /// 외부에서 주입된 skill 목록.
    pub skills: Vec<Skill>,
}

impl SkillsPaneState {
    /// `skill_filter` 에 매치되는 skill 만 반환한다.
    ///
    /// 매치는 case-insensitive substring 으로 name + description 양쪽을 검사.
    /// 빈 filter 는 전체를 반환한다.
    pub fn filtered_skills(&self) -> Vec<&Skill> {
        if self.skill_filter.is_empty() {
            return self.skills.iter().collect();
        }
        let needle = self.skill_filter.to_ascii_lowercase();
        self.skills
            .iter()
            .filter(|s| {
                s.name.to_ascii_lowercase().contains(&needle)
                    || s.description.to_ascii_lowercase().contains(&needle)
            })
            .collect()
    }
}

// ============================================================
// Rule + RulesPaneState — RulesPane in-memory state (MS-4d skeleton)
// ============================================================

/// 단일 Claude Code Rule 의 read-only 메타데이터.
///
/// SPEC-V3-013 MS-4d (audit G-1) — `.claude/rules/**/*.md` 의 frontmatter
/// 와 호환되는 최소 필드만 노출한다. 실제 토글 / 편집은 후속 SPEC.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    /// rule 의 표시 이름 (frontmatter `name:` 또는 파일 이름).
    pub name: String,
    /// 1-line 요약 (frontmatter `description:` 또는 빈 문자열).
    pub description: String,
    /// scope — "user" / "project" / "plugin:<namespace>".
    pub scope: String,
    /// 활성화 여부 (paths frontmatter 매치 OR 사용자 토글 결과, default true).
    pub enabled: bool,
}

impl Rule {
    /// 새 Rule 메타데이터를 생성한다.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        scope: impl Into<String>,
        enabled: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            scope: scope.into(),
            enabled,
        }
    }
}

/// RulesPane 의 read-only in-memory 상태 (v0.1.2 Task 9d / MS-4d).
///
/// 외부 (lib.rs) 가 .claude/rules/ 또는 plugin-bundled rules 를 스캔하여
/// `set_rules` 로 주입한다. v0.1.2 단계에서는 자동 로드 wiring 미포함.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RulesPaneState {
    /// 사용자 입력 search query — 빈 문자열이면 전체 노출.
    pub rule_filter: String,
    /// 외부에서 주입된 rule 목록.
    pub rules: Vec<Rule>,
}

impl RulesPaneState {
    /// `rule_filter` 에 매치되는 rule 만 반환한다.
    ///
    /// 매치는 case-insensitive substring 으로 name + description 양쪽을 검사.
    /// 빈 filter 는 전체를 반환한다.
    pub fn filtered_rules(&self) -> Vec<&Rule> {
        if self.rule_filter.is_empty() {
            return self.rules.iter().collect();
        }
        let needle = self.rule_filter.to_ascii_lowercase();
        self.rules
            .iter()
            .filter(|r| {
                r.name.to_ascii_lowercase().contains(&needle)
                    || r.description.to_ascii_lowercase().contains(&needle)
            })
            .collect()
    }
}

// ============================================================
// AdvancedState — AdvancedPane in-memory 상태 (MS-2 skeleton)
// ============================================================

/// AdvancedPane 의 in-memory 상태 (v0.1.0 skeleton — 1 setting).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AdvancedState {
    /// 실험적 플래그 목록 (read-only, default 빈 목록) — REQ-V13-043.
    pub experimental_flags: Vec<String>,
}

// ============================================================
// SettingsViewState — SettingsModal transient 상태
// ============================================================

/// SettingsModal 의 런타임 전용 뷰 상태.
///
/// @MX:NOTE: [AUTO] settings-view-state-transient
/// 이 상태는 영속화되지 않는다 — UserSettings 영속화는 MS-3.
/// SettingsModal mount 시 항상 default (Appearance section 활성) 로 시작한다.
pub struct SettingsViewState {
    /// 현재 선택된 section (default: Appearance).
    pub selected_section: SettingsSection,
    /// AppearancePane 의 in-memory 상태.
    pub appearance: AppearanceState,
    /// KeyboardPane 의 in-memory 상태.
    pub keyboard: KeyboardState,
    /// EditorPane 의 in-memory 상태.
    pub editor: EditorState,
    /// TerminalPane 의 in-memory 상태.
    pub terminal: TerminalState,
    /// AgentPane 의 in-memory 상태.
    pub agent: AgentState,
    /// HooksPane 의 in-memory 상태 (MS-4a, audit G-1).
    pub hooks: HooksState,
    /// McpPane 의 in-memory 상태 (MS-4b, audit G-1).
    pub mcp: McpPaneState,
    /// SkillsPane 의 in-memory 상태 (MS-4c, audit G-1).
    pub skills: SkillsPaneState,
    /// RulesPane 의 in-memory 상태 (MS-4d, audit G-1).
    pub rules: RulesPaneState,
    /// AdvancedPane 의 in-memory 상태.
    pub advanced: AdvancedState,
    /// PluginsPane 의 in-memory 상태 (SPEC-V0-2-0-PLUGIN-MGR-001 MS-1).
    pub plugins: PluginsState,
    /// SettingsModal 이 표시 중인지 여부 (mount/dismiss 상태).
    pub is_visible: bool,
}

impl Default for SettingsViewState {
    fn default() -> Self {
        Self {
            selected_section: SettingsSection::Appearance,
            appearance: AppearanceState::default(),
            keyboard: KeyboardState::default(),
            editor: EditorState::default(),
            terminal: TerminalState::default(),
            agent: AgentState::default(),
            hooks: HooksState::default(),
            mcp: McpPaneState::default(),
            skills: SkillsPaneState::default(),
            rules: RulesPaneState::default(),
            advanced: AdvancedState::default(),
            plugins: PluginsState::default(),
            is_visible: false,
        }
    }
}

impl SettingsViewState {
    /// 새 SettingsViewState 를 생성한다 (default: Appearance 섹션, 숨김 상태).
    pub fn new() -> Self {
        Self::default()
    }

    /// 지정 section 을 선택한다.
    pub fn select_section(&mut self, section: SettingsSection) {
        self.selected_section = section;
    }

    /// SettingsModal 을 mount 상태로 전환한다.
    pub fn show(&mut self) {
        self.is_visible = true;
    }

    /// SettingsModal 을 dismiss 상태로 전환한다.
    pub fn hide(&mut self) {
        self.is_visible = false;
    }
}

// ============================================================
// 단위 테스트 — RED phase (SPEC-V3-013 MS-1)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- SettingsSection tests ----

    #[test]
    /// SettingsSection::all() 이 11개를 정해진 순서로 반환한다 (REQ-V13-010 + MS-4a/4b/4c/4d + REQ-PM-001/002).
    fn section_all_returns_eleven_in_order() {
        let all = SettingsSection::all();
        assert_eq!(all.len(), 11);
        assert_eq!(all[0], SettingsSection::Appearance);
        assert_eq!(all[1], SettingsSection::Keyboard);
        assert_eq!(all[2], SettingsSection::Editor);
        assert_eq!(all[3], SettingsSection::Terminal);
        assert_eq!(all[4], SettingsSection::Agent);
        assert_eq!(all[5], SettingsSection::Hooks);
        assert_eq!(all[6], SettingsSection::Mcp);
        assert_eq!(all[7], SettingsSection::Skills);
        assert_eq!(all[8], SettingsSection::Rules);
        assert_eq!(all[9], SettingsSection::Advanced);
        assert_eq!(all[10], SettingsSection::Plugins);
    }

    #[test]
    /// 각 section 의 label() 이 올바른 문자열을 반환한다 (REQ-PM-003 포함).
    fn section_labels_are_correct() {
        assert_eq!(SettingsSection::Appearance.label(), "Appearance");
        assert_eq!(SettingsSection::Keyboard.label(), "Keyboard");
        assert_eq!(SettingsSection::Editor.label(), "Editor");
        assert_eq!(SettingsSection::Terminal.label(), "Terminal");
        assert_eq!(SettingsSection::Agent.label(), "Agent");
        assert_eq!(SettingsSection::Hooks.label(), "Hooks");
        assert_eq!(SettingsSection::Mcp.label(), "MCP");
        assert_eq!(SettingsSection::Skills.label(), "Skills");
        assert_eq!(SettingsSection::Rules.label(), "Rules");
        assert_eq!(SettingsSection::Advanced.label(), "Advanced");
        assert_eq!(SettingsSection::Plugins.label(), "Plugins");
    }

    /// AC-PM-7 (REQ-PM-007): SettingsViewState::default() initializes plugins to PluginsState::default().
    #[test]
    fn settings_view_state_default_initializes_plugins_state() {
        let view = SettingsViewState::default();
        assert_eq!(view.plugins, PluginsState::default());
        assert_eq!(view.plugins.plugin_filter, "");
    }

    /// REQ-PM-007 mirror: PluginsState filtered_plugins respects empty filter / case-insensitive match.
    #[test]
    fn plugins_state_filtered_plugins_respects_filter() {
        let state = PluginsState::default();
        let entries = ["alpha", "Beta", "Gamma"];
        let visible = state.filtered_plugins(&entries, |s| (s, ""));
        assert_eq!(visible.len(), 3, "empty filter must surface all");

        let state = PluginsState {
            plugin_filter: "BET".to_string(),
        };
        let visible = state.filtered_plugins(&entries, |s| (s, ""));
        assert_eq!(visible.len(), 1);
        assert_eq!(*visible[0], "Beta");
    }

    // ---- AppearanceState tests ----

    #[test]
    /// AppearanceState 기본값이 SPEC 명세와 일치한다.
    fn appearance_state_default_values() {
        let s = AppearanceState::default();
        assert_eq!(s.theme, ThemeMode::Dark);
        assert_eq!(s.density, Density::Comfortable);
        assert_eq!(s.accent, AccentColor::Teal);
        assert_eq!(s.font_size_px, 14);
    }

    #[test]
    /// font_size 12~18 범위 내 설정이 성공한다 (REQ-V13-024).
    fn font_size_valid_range_accepted() {
        let mut s = AppearanceState::default();
        assert!(s.set_font_size(12));
        assert_eq!(s.font_size_px, 12);
        assert!(s.set_font_size(18));
        assert_eq!(s.font_size_px, 18);
        assert!(s.set_font_size(14));
        assert_eq!(s.font_size_px, 14);
    }

    #[test]
    /// font_size 11 (범위 하한 미만) 은 적용되지 않는다 (AC-V13-6).
    fn font_size_below_min_rejected() {
        let mut s = AppearanceState::default();
        let original = s.font_size_px;
        assert!(!s.set_font_size(11));
        assert_eq!(s.font_size_px, original, "범위 외 값 11은 적용되면 안 됨");
    }

    #[test]
    /// font_size 19 (범위 상한 초과) 는 적용되지 않는다 (AC-V13-6).
    fn font_size_above_max_rejected() {
        let mut s = AppearanceState::default();
        let original = s.font_size_px;
        assert!(!s.set_font_size(19));
        assert_eq!(s.font_size_px, original, "범위 외 값 19는 적용되면 안 됨");
    }

    #[test]
    /// font_size 경계값 0 은 적용되지 않는다.
    fn font_size_zero_rejected() {
        let mut s = AppearanceState::default();
        assert!(!s.set_font_size(0));
        assert_eq!(s.font_size_px, 14);
    }

    #[test]
    /// density Compact 시 spacing_multiplier 가 0.85 이다 (REQ-V13-022).
    fn density_compact_multiplier_is_0_85() {
        let s = AppearanceState {
            density: Density::Compact,
            ..Default::default()
        };
        let m = s.spacing_multiplier();
        assert!(
            (m - 0.85).abs() < f32::EPSILON,
            "compact multiplier = 0.85, got {m}"
        );
    }

    #[test]
    /// density Comfortable 시 spacing_multiplier 가 1.0 이다 (REQ-V13-022).
    fn density_comfortable_multiplier_is_1_0() {
        let s = AppearanceState::default();
        assert_eq!(s.density, Density::Comfortable);
        let m = s.spacing_multiplier();
        assert!(
            (m - 1.0).abs() < f32::EPSILON,
            "comfortable multiplier = 1.0, got {m}"
        );
    }

    #[test]
    /// AccentColor::Violet 의 hex_value 가 0x6a4cc7 이다 (AC-V13-5).
    fn accent_violet_hex_is_correct() {
        assert_eq!(AccentColor::Violet.hex_value(), 0x6a4cc7);
    }

    #[test]
    /// AccentColor::Teal 의 hex_value 가 design::tokens::ide_accent::TEAL 과 일치한다.
    fn accent_teal_hex_matches_token() {
        use crate::design::tokens::ide_accent;
        assert_eq!(AccentColor::Teal.hex_value(), ide_accent::TEAL);
    }

    #[test]
    /// AccentColor::Blue hex_value 가 0x2563eb 이다.
    fn accent_blue_hex_is_correct() {
        assert_eq!(AccentColor::Blue.hex_value(), 0x2563eb);
    }

    #[test]
    /// AccentColor::Cyan hex_value 가 0x06b6d4 이다.
    fn accent_cyan_hex_is_correct() {
        assert_eq!(AccentColor::Cyan.hex_value(), 0x06b6d4);
    }

    #[test]
    /// 4개 accent 색상이 모두 다른 값이다.
    fn accent_four_colors_are_distinct() {
        let values = [
            AccentColor::Teal.hex_value(),
            AccentColor::Blue.hex_value(),
            AccentColor::Violet.hex_value(),
            AccentColor::Cyan.hex_value(),
        ];
        // 중복 없이 4개 고유 값
        let mut unique = values.to_vec();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), 4, "4개 accent 색상이 모두 달라야 함");
    }

    // ---- SettingsViewState tests ----

    #[test]
    /// SettingsViewState 기본값 — Appearance 선택, 숨김 상태.
    fn view_state_default_is_appearance_hidden() {
        let state = SettingsViewState::new();
        assert_eq!(state.selected_section, SettingsSection::Appearance);
        assert!(!state.is_visible);
    }

    #[test]
    /// show() 호출 시 is_visible 이 true 가 된다 (AC-V13-1).
    fn view_state_show_makes_visible() {
        let mut state = SettingsViewState::new();
        state.show();
        assert!(state.is_visible);
    }

    #[test]
    /// hide() 호출 시 is_visible 이 false 가 된다 (REQ-V13-004).
    fn view_state_hide_makes_hidden() {
        let mut state = SettingsViewState::new();
        state.show();
        state.hide();
        assert!(!state.is_visible);
    }

    #[test]
    /// select_section() 이 selected_section 을 업데이트한다 (AC-V13-3).
    fn view_state_select_section_updates_state() {
        let mut state = SettingsViewState::new();
        state.select_section(SettingsSection::Keyboard);
        assert_eq!(state.selected_section, SettingsSection::Keyboard);
        state.select_section(SettingsSection::Advanced);
        assert_eq!(state.selected_section, SettingsSection::Advanced);
    }

    #[test]
    /// AppearanceState theme 변경이 SettingsViewState 에 반영된다 (AC-V13-4 in-memory).
    fn view_state_appearance_theme_mutation() {
        let mut state = SettingsViewState::new();
        assert_eq!(state.appearance.theme, ThemeMode::Dark);
        state.appearance.theme = ThemeMode::Light;
        assert_eq!(state.appearance.theme, ThemeMode::Light);
    }

    #[test]
    /// AppearanceState accent 변경이 반영된다 (AC-V13-5 in-memory).
    fn view_state_appearance_accent_mutation() {
        let mut state = SettingsViewState::new();
        state.appearance.accent = AccentColor::Violet;
        assert_eq!(state.appearance.accent, AccentColor::Violet);
        assert_eq!(state.appearance.accent.hex_value(), 0x6a4cc7);
    }

    // ---- MS-2: KeyBinding tests ----

    #[test]
    /// default_key_bindings() 가 10개 이상의 바인딩을 반환한다 (AC-V13-7).
    fn default_key_bindings_has_ten_or_more() {
        let bindings = default_key_bindings();
        assert!(
            bindings.len() >= 10,
            "기본 바인딩은 10개 이상이어야 함, 실제: {}",
            bindings.len()
        );
    }

    #[test]
    /// 기본 바인딩 중 내부 충돌 (동일 shortcut) 이 없다 (R-V13-5).
    fn default_key_bindings_no_internal_conflicts() {
        let bindings = default_key_bindings();
        let mut seen_shortcuts = std::collections::HashSet::new();
        for b in &bindings {
            assert!(
                seen_shortcuts.insert(b.shortcut.clone()),
                "중복 shortcut 발견: '{}' (action: '{}')",
                b.shortcut,
                b.action_id
            );
        }
    }

    #[test]
    /// 기본 바인딩 중 action_id 가 모두 유일하다.
    fn default_key_bindings_unique_action_ids() {
        let bindings = default_key_bindings();
        let mut seen_ids = std::collections::HashSet::new();
        for b in &bindings {
            assert!(
                seen_ids.insert(b.action_id.clone()),
                "중복 action_id 발견: '{}'",
                b.action_id
            );
        }
    }

    #[test]
    /// KeyBinding::new() 생성자가 필드를 올바르게 설정한다.
    fn key_binding_new_sets_fields() {
        let b = KeyBinding::new("open.settings", "설정 열기", "Cmd+,");
        assert_eq!(b.action_id, "open.settings");
        assert_eq!(b.label, "설정 열기");
        assert_eq!(b.shortcut, "Cmd+,");
    }

    // ---- MS-2: KeyboardState tests ----

    #[test]
    /// KeyboardState::new() 가 기본 바인딩으로 초기화된다.
    fn keyboard_state_default_has_bindings() {
        let ks = KeyboardState::new();
        assert!(ks.bindings.len() >= 10);
        assert!(ks.edit_dialog.is_none());
    }

    #[test]
    /// conflict_check — 미사용 단축키는 None 반환 (AC-V13-8 pass case).
    fn conflict_check_unused_shortcut_returns_none() {
        let ks = KeyboardState::new();
        let result = ks.conflict_check("open.settings", "Cmd+Option+X");
        assert!(result.is_none(), "미사용 단축키는 충돌 없음");
    }

    #[test]
    /// conflict_check — 같은 action_id 의 단축키는 충돌 아님 (자기 자신 재할당).
    fn conflict_check_same_action_id_no_conflict() {
        let ks = KeyboardState::new();
        // open.settings 의 기본 shortcut = "Cmd+,"
        let result = ks.conflict_check("open.settings", "Cmd+,");
        assert!(result.is_none(), "같은 action에 같은 shortcut은 충돌 아님");
    }

    #[test]
    /// conflict_check — 다른 action_id 에 할당된 단축키는 충돌 반환 (AC-V13-8 fail case).
    fn conflict_check_occupied_shortcut_returns_conflict() {
        let ks = KeyboardState::new();
        // "Cmd+," 은 이미 open.settings 에 할당됨
        let result = ks.conflict_check("panes.close", "Cmd+,");
        assert!(result.is_some(), "이미 할당된 shortcut은 충돌 반환");
        let conflict_label = result.unwrap();
        assert_eq!(conflict_label, "설정 열기", "충돌 레이블이 일치해야 함");
    }

    #[test]
    /// apply_binding — 미사용 단축키 적용 성공.
    fn apply_binding_unused_shortcut_succeeds() {
        let mut ks = KeyboardState::new();
        let result = ks.apply_binding("open.settings", "Cmd+Option+S");
        assert!(result.is_ok());
        let binding = ks
            .bindings
            .iter()
            .find(|b| b.action_id == "open.settings")
            .unwrap();
        assert_eq!(binding.shortcut, "Cmd+Option+S");
    }

    #[test]
    /// apply_binding — 충돌 단축키 적용 시 Err 반환하고 기존 값 유지.
    fn apply_binding_conflict_returns_err_and_preserves_original() {
        let mut ks = KeyboardState::new();
        // "Cmd+T" 는 tabs.new 에 할당됨
        let result = ks.apply_binding("open.settings", "Cmd+T");
        assert!(result.is_err(), "충돌 단축키는 Err 반환");
        // open.settings 는 여전히 원래 shortcut
        let binding = ks
            .bindings
            .iter()
            .find(|b| b.action_id == "open.settings")
            .unwrap();
        assert_eq!(binding.shortcut, "Cmd+,");
    }

    #[test]
    /// open_edit_dialog — 존재하는 action_id 로 다이얼로그를 연다.
    fn open_edit_dialog_sets_dialog_state() {
        let mut ks = KeyboardState::new();
        ks.open_edit_dialog("open.settings");
        assert!(ks.edit_dialog.is_some());
        let dialog = ks.edit_dialog.as_ref().unwrap();
        assert_eq!(dialog.action_id, "open.settings");
        assert_eq!(dialog.pending_shortcut, "Cmd+,");
        assert!(dialog.conflict_error.is_none());
    }

    #[test]
    /// close_edit_dialog — 다이얼로그를 닫는다.
    fn close_edit_dialog_clears_dialog() {
        let mut ks = KeyboardState::new();
        ks.open_edit_dialog("open.settings");
        assert!(ks.edit_dialog.is_some());
        ks.close_edit_dialog();
        assert!(ks.edit_dialog.is_none());
    }

    #[test]
    /// save_edit_dialog — 미사용 단축키로 저장 성공.
    fn save_edit_dialog_success() {
        let mut ks = KeyboardState::new();
        ks.open_edit_dialog("open.settings");
        ks.edit_dialog.as_mut().unwrap().pending_shortcut = "Cmd+Option+S".to_string();
        let ok = ks.save_edit_dialog();
        assert!(ok, "저장 성공");
        assert!(ks.edit_dialog.is_none(), "저장 후 다이얼로그 닫힘");
        let binding = ks
            .bindings
            .iter()
            .find(|b| b.action_id == "open.settings")
            .unwrap();
        assert_eq!(binding.shortcut, "Cmd+Option+S");
    }

    #[test]
    /// save_edit_dialog — 충돌 단축키로 저장 실패 시 dialog 에 오류 설정.
    fn save_edit_dialog_conflict_sets_error() {
        let mut ks = KeyboardState::new();
        ks.open_edit_dialog("open.settings");
        // tabs.new 의 단축키 "Cmd+T" 로 충돌 시도
        ks.edit_dialog.as_mut().unwrap().pending_shortcut = "Cmd+T".to_string();
        let ok = ks.save_edit_dialog();
        assert!(!ok, "충돌 시 저장 실패");
        // 다이얼로그가 열려 있고 오류가 설정됨
        assert!(ks.edit_dialog.is_some());
        assert!(ks.edit_dialog.as_ref().unwrap().conflict_error.is_some());
    }

    // ---- MS-2: EditorState tests ----

    #[test]
    /// EditorState 기본값이 tab_size = 4 이다 (REQ-V13-040).
    fn editor_state_default_tab_size_is_4() {
        let s = EditorState::default();
        assert_eq!(s.tab_size, 4);
    }

    #[test]
    /// tab_size 2~8 범위 내 설정 성공.
    fn editor_state_set_tab_size_valid_range() {
        let mut s = EditorState::default();
        assert!(s.set_tab_size(2));
        assert_eq!(s.tab_size, 2);
        assert!(s.set_tab_size(8));
        assert_eq!(s.tab_size, 8);
    }

    #[test]
    /// tab_size 1 (범위 하한 미만) 거부.
    fn editor_state_set_tab_size_below_min_rejected() {
        let mut s = EditorState::default();
        assert!(!s.set_tab_size(1));
        assert_eq!(s.tab_size, 4);
    }

    #[test]
    /// tab_size 9 (범위 상한 초과) 거부.
    fn editor_state_set_tab_size_above_max_rejected() {
        let mut s = EditorState::default();
        assert!(!s.set_tab_size(9));
        assert_eq!(s.tab_size, 4);
    }

    // ---- MS-2: TerminalState tests ----

    #[test]
    /// TerminalState 기본값이 scrollback_lines = 10000 이다 (REQ-V13-041).
    fn terminal_state_default_scrollback_is_10000() {
        let s = TerminalState::default();
        assert_eq!(s.scrollback_lines, 10_000);
    }

    #[test]
    /// scrollback_lines 1000~100000 범위 내 설정 성공.
    fn terminal_state_set_scrollback_valid() {
        let mut s = TerminalState::default();
        assert!(s.set_scrollback_lines(1_000));
        assert_eq!(s.scrollback_lines, 1_000);
        assert!(s.set_scrollback_lines(100_000));
        assert_eq!(s.scrollback_lines, 100_000);
    }

    #[test]
    /// scrollback_lines 999 거부.
    fn terminal_state_set_scrollback_below_min_rejected() {
        let mut s = TerminalState::default();
        assert!(!s.set_scrollback_lines(999));
        assert_eq!(s.scrollback_lines, 10_000);
    }

    #[test]
    /// scrollback_lines 100001 거부.
    fn terminal_state_set_scrollback_above_max_rejected() {
        let mut s = TerminalState::default();
        assert!(!s.set_scrollback_lines(100_001));
        assert_eq!(s.scrollback_lines, 10_000);
    }

    // ---- MS-2: AgentState tests ----

    #[test]
    /// AgentState 기본값이 auto_approve = false 이다 (REQ-V13-042).
    fn agent_state_default_auto_approve_is_false() {
        let s = AgentState::default();
        assert!(!s.auto_approve);
    }

    #[test]
    /// toggle_auto_approve() 가 상태를 반전한다.
    fn agent_state_toggle_auto_approve() {
        let mut s = AgentState::default();
        s.toggle_auto_approve();
        assert!(s.auto_approve);
        s.toggle_auto_approve();
        assert!(!s.auto_approve);
    }

    // ---- MS-2: AdvancedState tests ----

    #[test]
    /// AdvancedState 기본값이 experimental_flags = [] 이다 (REQ-V13-043).
    fn advanced_state_default_flags_empty() {
        let s = AdvancedState::default();
        assert!(s.experimental_flags.is_empty());
    }

    // ---- MS-2: SettingsViewState 확장 tests ----

    #[test]
    /// SettingsViewState 기본 keyboard state 가 기본 바인딩을 포함한다.
    fn view_state_keyboard_has_default_bindings() {
        let state = SettingsViewState::new();
        assert!(state.keyboard.bindings.len() >= 10);
    }

    #[test]
    /// SettingsViewState 기본 editor state 가 tab_size = 4 이다.
    fn view_state_editor_default_tab_size() {
        let state = SettingsViewState::new();
        assert_eq!(state.editor.tab_size, 4);
    }

    #[test]
    /// SettingsViewState 기본 terminal state 가 scrollback = 10000 이다.
    fn view_state_terminal_default_scrollback() {
        let state = SettingsViewState::new();
        assert_eq!(state.terminal.scrollback_lines, 10_000);
    }

    #[test]
    /// SettingsViewState 기본 agent state 가 auto_approve = false 이다.
    fn view_state_agent_default_auto_approve() {
        let state = SettingsViewState::new();
        assert!(!state.agent.auto_approve);
    }

    #[test]
    /// SettingsViewState 기본 advanced state 가 flags = [] 이다.
    fn view_state_advanced_default_flags_empty() {
        let state = SettingsViewState::new();
        assert!(state.advanced.experimental_flags.is_empty());
    }
}
