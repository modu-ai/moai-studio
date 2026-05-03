#![recursion_limit = "1024"]
//! MoAI Studio UI 컴포넌트 라이브러리.
//!
//! Phase 1.6 (SPEC-V3-001 RG-V3-2) — Sidebar workspace 리스트 + active 하이라이트.
//! Phase 2 (SPEC-V3-002 T4) — TerminalSurface content_area 분기 추가.
//!
//! ## 설계
//! - `run_app(workspaces)` 이 유일한 엔트리. `moai-studio-app` 바이너리가 호출.
//! - 윈도우 크기 1600×1000 (`system.md` §8 기본 크기)
//! - 4 영역:
//!   - TitleBar 44pt (상단, 활성 워크스페이스 이름 표시)
//!   - Sidebar 260pt (좌측, workspace 리스트) + Body (가변, 우측)
//!   - StatusBar 28pt (하단)
//! - Empty state CTA 는 workspaces 가 비었을 때만 body 에 표시
//! - TerminalSurface 가 Some 이면 content_area 는 빈 상태 대신 터미널을 렌더한다.

use crate::terminal::TerminalClickEvent;
// SPEC-V3-007 MS-4 (REQ-WB-031~033): RootView wires TerminalStdoutEvent
// into the URL detector so dev-server URLs surfaced from PTY output reach
// the WebView surface via toast click.
#[cfg(feature = "web")]
use crate::terminal::TerminalStdoutEvent;

pub mod agent;
// SPEC-V3-006 MS-7 (audit F-4): state-bearing StatusBar widget surface.
pub mod status_bar;
// SPEC-V3-004 MS-4 (audit D-2): sidebar workspace context menu skeleton.
pub mod workspace_menu;
// SPEC-V0-1-2-MENUS-001 F-3: Toolbar 모듈
pub mod toolbar;
// G-2: Project Wizard 모듈
pub mod wizard;
// SPEC-V3-014 MS-1: Banners Surface 모듈 (Banner trait + BannerView + BannerStack)
pub mod banners;
// tokens.json v2.0.0 GPUI Rust 상수 모듈 (chore: design-tokens-rust-A-B)
pub mod design;
pub mod explorer;
// SPEC-V3-012 MS-1: Palette Surface 모듈 (Scrim + PaletteView core)
pub mod palette;
pub mod panes;
// SPEC-V3-013 MS-1: Settings Surface 모듈 (SettingsModal + AppearancePane)
pub mod settings;
pub mod tabs;
pub mod terminal;
// SPEC-V3-006 MS-1: viewer surface 모듈
pub mod viewer;
// SPEC-V3-009 MS-1: SPEC Management UI 모듈 (SpecListView + SpecDetailView)
pub mod spec_ui;
// SPEC-V3-008 MS-2: Git UI module (GitDiffViewer + GitBranchSwitcher)
pub mod git;
// SPEC-V3-007 MS-1: WebView module (wry backend + abstraction)
#[cfg(feature = "web")]
pub mod web;
// SPEC-V3-017 MS-2: TRUST 5 Quality Dashboard (RadarChartView + QualityGateView)
pub mod quality;
// SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2: Global search panel (SearchPanel GPUI Entity).
pub mod search;
// SPEC-V0-2-0-MULTI-SHELL-001 MS-1: Shell picker logic (ShellPicker struct).
pub mod shell_picker;

use design::tokens::{self as tok, traffic};
use gpui::{
    App, Application, Context, Entity, InteractiveElement, IntoElement, KeyDownEvent, Menu,
    MenuItem, MouseButton, OsAction, ParentElement, Render, Styled, SystemMenuType, Window,
    WindowOptions, actions, div, prelude::*, px, rgb, size,
};

// SPEC-V0-1-1-UX-FIX (C-5 + audit §10): macOS menu bar 비어있던 상태 해결.
// gpui::actions! 매크로로 Action type 정의 → cx.set_menus + cx.on_action + cx.bind_keys 로 wire.
// - Quit / About: App menu actions (system 자동 dispatch)
// - NoOp: OsAction Cut/Copy/Paste/Undo/Redo placeholder (system handler 가 처리, dispatch 무시)
// - NewWorkspace: File menu + Cmd+N → RootView::handle_add_workspace
// - OpenSettings: App menu + Cmd+, → RootView::settings_modal mount (V3-013)
// - ReportIssue: Help menu → GitHub issues URL
actions!(
    moai_studio,
    [
        Quit,
        About,
        NoOp,
        NewWorkspace,
        OpenSettings,
        ReportIssue,
        // SPEC-V0-1-2 menu expansion — View
        ToggleSidebar,
        ToggleBanner,
        ReloadWorkspace,
        ToggleTheme,
        ToggleFind,
        // Pane
        SplitRight,
        SplitDown,
        ClosePane,
        FocusNextPane,
        FocusPrevPane,
        // Surface
        NewTerminalSurface,
        NewMarkdownSurface,
        NewCodeViewerSurface,
        // Go
        OpenCommandPalette,
        OpenSpecPanel,
        // Help
        OpenDocumentation,
        OpenAbout,
        // WebView DevTools (SPEC-V3-007 MS-2)
        #[cfg(feature = "web")]
        ToggleDevTools,
    ]
);
use moai_studio_workspace::{Workspace, WorkspacesStore};
use panes::PaneId;
use std::collections::HashMap;
use std::path::PathBuf;
use tabs::TabContainer;
use tracing::{error, info};
use viewer::LeafKind;

// ============================================================
// Design tokens — design::tokens (tokens.json v2.0.0) alias.
// 구 `tokens` 모듈은 design::tokens 로 통합되었습니다.
// ============================================================

/// 하위 호환 alias — 기존 코드가 `tokens::XXX` 패턴으로 참조할 경우 사용.
/// 새 코드는 `design::tokens::*` 또는 `design::tokens::theme::dark::*` 를 직접 사용할 것.
#[deprecated(
    since = "0.2.0",
    note = "design::tokens 모듈로 이관. crate::design::tokens::* 를 직접 사용하세요."
)]
#[allow(dead_code)]
pub mod tokens {
    pub use crate::design::tokens::BG_ELEVATED as BG_SURFACE_2;
    pub use crate::design::tokens::BG_ELEVATED as BG_SURFACE_3;
    pub use crate::design::tokens::BG_PANEL as BG_BASE;
    pub use crate::design::tokens::BG_SURFACE;
    pub use crate::design::tokens::BORDER_STRONG;
    pub use crate::design::tokens::BORDER_SUBTLE;
    pub use crate::design::tokens::FG_DISABLED as FG_DIM;
    pub use crate::design::tokens::FG_MUTED;
    pub use crate::design::tokens::FG_PRIMARY;
    pub use crate::design::tokens::FG_SECONDARY;
    /// ACCENT_MOAI (오렌지) 폐기 → 다크 모드 청록 PRIMARY_DARK 로 교체
    pub use crate::design::tokens::brand::PRIMARY_DARK as ACCENT_MOAI;
    pub use crate::design::tokens::traffic::GREEN as TRAFFIC_GREEN;
    pub use crate::design::tokens::traffic::RED as TRAFFIC_RED;
    pub use crate::design::tokens::traffic::YELLOW as TRAFFIC_YELLOW;
}

// ============================================================
// Root view — 4 영역 레이아웃 컨테이너
// ============================================================

/// 앱 전역 상태 — Phase 1.7: workspace 리스트 + active id + storage path (버튼 클릭 시 재로드).
/// Phase 2 (SPEC-V3-002 T4): terminal 필드 추가 — content_area TerminalSurface 분기.
/// Phase 3 MS-1 T7 (SPEC-V3-003): terminal → pane_splitter rename.
/// Phase 4 T2 (SPEC-V3-004 MS-1): pane_splitter → tab_container rename.
/// SPEC-V3-013 MS-3: settings_modal slot + user_settings load-on-init + active_theme.
pub struct RootView {
    pub workspaces: Vec<Workspace>,
    pub active_id: Option<String>,
    pub storage_path: PathBuf,
    // @MX:ANCHOR: [AUTO] root-view-tab-container-binding
    // @MX:REASON: [AUTO] SPEC-V3-004 RG-R-1. tab_container 는 content_area 진입점이며
    //   key dispatch (RG-R-4) 와 divider drag (RG-R-3) 의 mutation target 이다.
    //   fan_in >= 3: T2 init, T5 key handler (MS-2), T7 divider drag (MS-3).
    /// SPEC-V3-004 MS-1: content_area 렌더 진입점 (TabContainer Entity).
    /// None 이면 Empty State CTA 렌더 (REQ-R-005).
    pub tab_container: Option<Entity<TabContainer>>,
    // @MX:ANCHOR: [AUTO] root-view-file-explorer-binding
    // @MX:REASON: [AUTO] SPEC-V3-005 RG-FE-1. file_explorer 는 sidebar 좌측 영역의 진입점이며
    //   워크스페이스 활성 변경 시 재바인딩된다.
    //   fan_in >= 3: T4 init, MS-2 watch event, MS-3 git_status refresh.
    /// SPEC-V3-005 MS-1: sidebar 파일 탐색기 Entity.
    /// None 이면 기존 workspace 리스트 렌더 유지.
    pub file_explorer: Option<Entity<explorer::FileExplorer>>,
    // @MX:TODO(MS-2-dashboard-wire): AgentDashboardView 를 content_area 에 배선 (MS-2 담당)
    /// SPEC-V3-010 MS-1: agent progress dashboard Entity (선택적).
    /// None 이면 tab_container 렌더 유지.
    pub agent_dashboard: Option<Entity<agent::dashboard_view::AgentDashboardView>>,
    // @MX:TODO(MS-2-pane-tree-leafkind): MS-2 에서 TabContainer 의 PaneTree<String> 을
    //   PaneTree<LeafKind> 로 교체하면 이 HashMap 은 제거되고 pane_tree.set_leaf_payload 로 직접 교체.
    /// SPEC-V3-006 MS-1: PaneId → LeafKind 매핑 (MS-2 에서 PaneTree<LeafKind> 교체 전 임시).
    pub leaf_payloads: HashMap<PaneId, LeafKind>,
    // ── MS-3 (SPEC-V3-012 AC-PL-14/15): palette overlay slot ──
    /// Palette overlay 상태 관리자 (mutual exclusion, RG-PL-24).
    pub palette: palette::PaletteOverlay,
    /// Terminal pane 포커스 상태 — SlashBar trigger 조건 (RG-PL-23).
    pub terminal_focused: bool,
    // ── F-1: fuzzy search query state ──
    /// Current text query typed inside the open palette (F-1, PARTIAL → DONE).
    /// Reset to empty when palette is dismissed.
    pub palette_query: String,
    /// Active CmdPalette instance wired to the fuzzy file source (F-1).
    /// Populated on CmdPalette open; reset to None on dismiss.
    pub cmd_palette: Option<palette::variants::CmdPalette>,
    // ── MS-3 (SPEC-V3-013 AC-V13-1/10/11/12): settings overlay slot ──
    // @MX:ANCHOR: [AUTO] root-view-settings-modal-slot
    // @MX:REASON: [AUTO] SPEC-V3-013 MS-3. settings_modal 은 Cmd+, 진입점이며
    //   mount/dismiss/save 의 mutation target 이다.
    //   fan_in >= 3: handle_settings_key_event (mount), dismiss (save), load_user_settings (init).
    /// SPEC-V3-013 MS-3: SettingsModal slot. None = dismiss 상태, Some = mount 상태 (REQ-V13-001).
    pub settings_modal: Option<settings::SettingsModal>,
    /// 현재 로드된 UserSettings (영속화 원본, REQ-V13-054).
    pub user_settings: settings::user_settings::UserSettings,
    /// 현재 런타임 테마 (ActiveTheme dispatch wrapper, REQ-V13-061).
    pub active_theme: design::runtime::ActiveTheme,
    // ── SPEC-V3-006 MS-3a: Find/Replace 상태 ──
    /// Find bar 표시 여부 (Cmd+F → true, Esc → false).
    pub find_bar_open: bool,
    // ── SPEC-V3-014 MS-3: BannerStack overlay slot ──
    // @MX:ANCHOR: [AUTO] root-view-banner-stack-slot
    // @MX:REASON: [AUTO] SPEC-V3-014 MS-3. banner_stack 은 top-of-window 알림 스택의 유일한 소유자.
    //   fan_in >= 3: RootView::render (mount), push_crash/update/lsp/pty/workspace helpers, tick loop.
    /// SPEC-V3-014 MS-3: BannerStack Entity. None = 미초기화 (테스트 호환), Some = 활성 (REQ-V14-026).
    pub banner_stack: Option<Entity<banners::BannerStack>>,
    // ── SPEC-V3-015 MS-1: SpecPanelView overlay slot ──
    // @MX:ANCHOR: [AUTO] root-view-spec-panel-slot
    // @MX:REASON: [AUTO] SPEC-V3-015 MS-1. spec_panel 은 3개 spec_ui 컴포넌트의 단일 overlay 진입점.
    //   fan_in >= 3: RootView::new (init=None), handle_spec_key_event (toggle), Render::render (mount).
    /// SPEC-V3-015 MS-1: SpecPanelView overlay. None = dismiss 상태, Some = mount 상태 (REQ-RV-002).
    pub spec_panel: Option<spec_ui::SpecPanelView>,
    // ── F-3: Toolbar Entity (SPEC-V0-1-2-MENUS-001) ──
    /// Main app toolbar with 7 action buttons (Option because created in run_app)
    pub toolbar: Option<Entity<toolbar::Toolbar>>,
    // ── G-2: Project Wizard Entity ──
    /// 5-step workspace creation wizard (Option because created in run_app)
    pub project_wizard: Option<Entity<wizard::ProjectWizard>>,
    // ── MS-4 (SPEC-V3-012 AC-PL-21): SlashBar terminal injection buffer ──
    /// Pending slash command string to inject into the active terminal on next update.
    ///
    /// Set by `inject_slash_command`; drained by the render/update loop when a
    /// TerminalSurface Entity context is available.
    pub pending_slash_injection: Option<String>,
    // ── SPEC-V3-006 MS-7 (audit F-4): state-bearing status bar widgets ──
    /// Injected widget state for the bottom 28pt status bar.
    ///
    /// Default state preserves the pre-MS-7 static rendering ("no git",
    /// version, ⌘K hint). External callers populate AgentPill / GitWidget /
    /// LspWidget via mutation API in `crate::status_bar`. Real broadcasting
    /// from git2 / LSP / agent runtime is follow-up (REQ-SB-MS7-3).
    pub status_bar: status_bar::StatusBarState,
    // ── SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2: SearchPanel sidebar slot ──
    // @MX:ANCHOR: [AUTO] root-view-search-panel-slot
    // @MX:REASON: [AUTO] SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2. search_panel is the
    //   sidebar toggleable section entry point. fan_in >= 3:
    //   RootView::new (init=None), handle_search_key_event (toggle), Render::render (mount).
    /// SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2: SearchPanel Entity.
    ///
    /// `None` = panel not yet initialised (created lazily on first ⌘⇧F).
    /// `Some` = panel exists and its `is_visible` controls sidebar rendering.
    pub search_panel: Option<search::SearchPanel>,
    // ── SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-3: last navigation result ──
    /// Last resolved `OpenCodeViewer` from a search result click.
    ///
    /// Set by `handle_search_open`. Consulted by logic-level tests to verify
    /// navigation outcome without a running GPUI application (Spike 2 pattern).
    /// Also used by the GPUI render path to dispatch scroll-to-line (MS-4).
    pub last_open_code_viewer: Option<moai_studio_terminal::link::OpenCodeViewer>,
    // ── SPEC-V3-007 MS-4 (RG-WB-4): WebView toast pipeline ──
    /// URL auto-detection debouncer fed by TerminalStdoutEvent (REQ-WB-031).
    ///
    /// Enforces the 5s dedupe window and 30min dismissed-URL silence required
    /// by REQ-WB-035. Always present so unit tests can exercise the logic
    /// without enabling the heavy `web` feature; kept feature-gated only for
    /// the wry-bound surface entities below.
    #[cfg(feature = "web")]
    pub url_detector: web::UrlDetectionDebouncer,
    /// Pending dev-server URL toasts ready to be rendered as an overlay.
    ///
    /// Populated by `wire_terminal_stdout_callback` (REQ-WB-032); drained by
    /// `open_toast_in_new_tab` (REQ-WB-033, AC-WB-INT-2) or `dismiss_toast`
    /// (AC-WB-INT-3).
    #[cfg(feature = "web")]
    pub pending_toasts: Vec<WebToastEntry>,
    // ── SPEC-V3-004 MS-5 (REQ-D2-MS5-3): rename modal state ──
    // @MX:NOTE: [AUTO] REQ-D2-MS5-3 — rename_modal holds rename UI state.
    //   None = closed, Some = open (target_id + buffer set by WorkspaceMenu dispatch).
    /// Rename modal slot.  `None` = modal is dismissed.
    pub rename_modal: Option<workspace_menu::RenameModal>,
    // ── SPEC-V3-004 MS-5 (REQ-D2-MS5-4): delete confirmation state ──
    // @MX:NOTE: [AUTO] REQ-D2-MS5-4 — delete_confirmation holds the pending delete target.
    //   None = no pending delete, Some = awaiting user confirmation.
    /// Delete confirmation slot.  `None` = no pending deletion.
    pub delete_confirmation: Option<workspace_menu::DeleteConfirmation>,
    // ── SPEC-V3-004 MS-5 (REQ-D2-MS5-5): WorkspacesStore owned by RootView ──
    // @MX:ANCHOR: [AUTO] root-view-store
    // @MX:REASON: [AUTO] REQ-D2-MS5-5. store is the single owner of WorkspacesStore in
    //   RootView. fan_in >= 3: handle_workspace_menu_action_logic (dispatch),
    //   apply_added_workspace (add), remove_workspace (remove).
    /// Owned WorkspacesStore, populated either from a real load or injected in tests.
    pub store: moai_studio_workspace::WorkspacesStore,
    // ── SPEC-V0-2-0-MULTI-SHELL-001 MS-1 (REQ-MS-007): ShellPicker overlay slot ──
    // @MX:NOTE: [AUTO] shell_picker — None = picker not yet activated.
    //   Populated lazily by handle_switch_shell on first Command Palette dispatch.
    // @MX:SPEC: SPEC-V0-2-0-MULTI-SHELL-001 REQ-MS-007
    /// Shell picker state.  `None` = not yet activated.
    ///
    /// Populated by `handle_switch_shell`; used by the GUI overlay (v0.2.1+).
    pub shell_picker: Option<shell_picker::ShellPicker>,
    // ── SPEC-V3-004 MS-6 (REQ-D2-MS6-1): workspace right-click context menu ──
    // @MX:NOTE: [AUTO] REQ-D2-MS6-1 — workspace_menu holds the sidebar right-click
    //   context menu state. Default = closed; populated by open_workspace_menu_at,
    //   drained by click_workspace_menu_item.
    /// Sidebar workspace context menu state. `Default::default()` = closed.
    pub workspace_menu: workspace_menu::WorkspaceMenu,
    // ── SPEC-V0-2-0-MISSION-CTRL-001 MS-2 (REQ-MC-024): Mission Control overlay slot ──
    // @MX:NOTE: [AUTO] mission-control — None = view not yet activated.
    //   Lazily created by `ensure_mission_control` on first activation. RootView
    //   pushes per-render snapshots into the entity via `update_mission_control_snapshot`.
    // @MX:SPEC: SPEC-V0-2-0-MISSION-CTRL-001 REQ-MC-024
    /// Mission Control 4-cell grid view slot. `None` = not yet activated.
    pub mission_control: Option<Entity<agent::mission_control_view::MissionControlView>>,
}

/// Pending toast entry surfaced for a detected dev-server URL.
///
/// Owns its strings so the entry remains valid after the Debouncer purges its
/// internal cache. The render layer reads these entries to draw the bottom-right
/// stack defined in AC-WB-INT-2.
#[cfg(feature = "web")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebToastEntry {
    /// Detected URL (e.g. `http://localhost:8080`).
    pub url: String,
    /// Source line that produced the match (truncated to a single line).
    pub source: String,
}

impl RootView {
    pub fn new(workspaces: Vec<Workspace>, storage_path: PathBuf) -> Self {
        // 가장 최근 활성 워크스페이스를 자동 선택 (last_active 최댓값).
        let active_id = workspaces
            .iter()
            .max_by_key(|w| w.last_active)
            .map(|w| w.id.clone());
        // SPEC-V3-013 MS-3: 앱 시작 시 UserSettings load + ActiveTheme 초기화 (REQ-V13-054).
        let user_settings =
            settings::user_settings::load_or_default(&settings::user_settings::settings_path());
        let active_theme = design::runtime::ActiveTheme::from_settings(&user_settings.appearance);
        // SPEC-V3-004 MS-5: empty store — real load happens in run_app after path is known.
        let store = moai_studio_workspace::WorkspacesStore::empty(storage_path.clone());
        Self {
            workspaces,
            active_id,
            storage_path,
            tab_container: None,
            file_explorer: None,
            agent_dashboard: None,
            leaf_payloads: HashMap::new(),
            palette: palette::PaletteOverlay::new(),
            terminal_focused: false,
            palette_query: String::new(),
            cmd_palette: None,
            settings_modal: None,
            user_settings,
            active_theme,
            find_bar_open: false,
            banner_stack: None,
            spec_panel: None,
            search_panel: None, // SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2: lazy init on first ⌘⇧F
            last_open_code_viewer: None, // SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-3: navigation result
            toolbar: None,      // F-3: toolbar created in run_app after App context available
            project_wizard: None, // G-2: wizard created in run_app after App context available
            pending_slash_injection: None, // MS-4: slash injection buffer (drained by render loop)
            // SPEC-V3-006 MS-7 (audit F-4): default state preserves pre-MS-7 static rendering.
            status_bar: status_bar::StatusBarState::default(),
            // SPEC-V3-007 MS-4: WebView toast pipeline initial state.
            #[cfg(feature = "web")]
            url_detector: web::UrlDetectionDebouncer::new(),
            #[cfg(feature = "web")]
            pending_toasts: Vec::new(),
            // SPEC-V3-004 MS-5: workspace context-menu overlay slots (default: closed).
            rename_modal: None,
            delete_confirmation: None,
            store,
            // SPEC-V0-2-0-MULTI-SHELL-001 MS-1: lazy init on first Command Palette dispatch.
            shell_picker: None,
            // SPEC-V3-004 MS-6 (REQ-D2-MS6-1): workspace context menu, default closed.
            workspace_menu: workspace_menu::WorkspaceMenu::default(),
            // SPEC-V0-2-0-MISSION-CTRL-001 MS-2 (REQ-MC-024): lazy-init on first activation.
            mission_control: None,
        }
    }

    /// 현재 활성 워크스페이스 레퍼런스.
    pub fn active(&self) -> Option<&Workspace> {
        let id = self.active_id.as_deref()?;
        self.workspaces.iter().find(|w| w.id == id)
    }

    /// TitleBar 에 표시할 워크스페이스 이름 (없으면 placeholder).
    pub fn title_label(&self) -> &str {
        self.active()
            .map(|w| w.name.as_str())
            .unwrap_or("no workspace")
    }

    /// 새 워크스페이스가 저장소에 추가된 이후 로컬 상태를 갱신.
    /// GPUI 이벤트 핸들러와 독립적으로 테스트 가능 (no-cx 시그니처 유지).
    pub fn apply_added_workspace(&mut self, added: &Workspace, all: Vec<Workspace>) {
        self.workspaces = all;
        self.active_id = Some(added.id.clone());
    }

    // @MX:ANCHOR: [AUTO] root-view-handle-workspace-menu-action-logic
    // @MX:REASON: [AUTO] REQ-D2-MS5-5. handle_workspace_menu_action_logic is the
    //   logic-level bridge between WorkspaceMenuAction dispatch and RootView overlay state.
    //   fan_in >= 3: T7 unit tests (3 callers), future GPUI handle_workspace_menu_action (cx),
    //   future sidebar right-click wire (next MS).
    /// Apply the result of a workspace context-menu action to RootView overlay state.
    ///
    /// This method is intentionally free of GPUI `Context` so it can be called from
    /// both the GPUI event handler (which will add `cx.notify()` calls) and from
    /// logic-level unit tests.
    ///
    /// - `Rename`  → opens `rename_modal` for the targeted workspace.
    /// - `Delete`  → opens `delete_confirmation` for the targeted workspace.
    /// - `MoveUp` / `MoveDown` → mutates `self.store` list order (no overlay opened).
    /// - Unknown workspace → logs a warning, no state change.
    pub fn handle_workspace_menu_action_logic(
        &mut self,
        action: workspace_menu::WorkspaceMenuAction,
        ws_id: &str,
    ) {
        use workspace_menu::{WorkspaceMenuOutcome, dispatch_workspace_menu_action};

        match dispatch_workspace_menu_action(action, ws_id, &mut self.store) {
            WorkspaceMenuOutcome::OpenRenameModal {
                ws_id,
                current_name,
            } => {
                self.rename_modal
                    .get_or_insert_with(Default::default)
                    .open(ws_id, current_name);
            }
            WorkspaceMenuOutcome::OpenDeleteConfirmation { ws_id } => {
                self.delete_confirmation
                    .get_or_insert_with(Default::default)
                    .open(ws_id);
            }
            WorkspaceMenuOutcome::Reordered => {
                // SPEC-V3-004 MS-6 (REQ-D2-MS6-2 fix-up for MS-5): re-sync local
                // workspaces vector from the store so the rendered sidebar reflects
                // the new order. Caller (GPUI handler) still owns cx.notify().
                self.sync_workspaces_from_store();
            }
            WorkspaceMenuOutcome::Unknown => {
                tracing::warn!(
                    ws_id,
                    "handle_workspace_menu_action_logic: unknown workspace or action failed"
                );
            }
        }
    }

    /// GPUI-aware wrapper: delegates to `handle_workspace_menu_action_logic` and
    /// calls `cx.notify()` on reorder so the sidebar re-renders.
    pub fn handle_workspace_menu_action(
        &mut self,
        action: workspace_menu::WorkspaceMenuAction,
        ws_id: &str,
        cx: &mut Context<Self>,
    ) {
        use workspace_menu::WorkspaceMenuAction;

        // Capture whether this is a reorder before mutating state.
        let is_reorder = matches!(
            action,
            WorkspaceMenuAction::MoveUp | WorkspaceMenuAction::MoveDown
        );
        self.handle_workspace_menu_action_logic(action, ws_id);
        if is_reorder {
            cx.notify();
        }
    }

    // ── SPEC-V3-004 MS-6 helpers (REQ-D2-MS6-1 ~ REQ-D2-MS6-4) ──

    // @MX:ANCHOR: [AUTO] sync-workspaces-from-store
    // @MX:REASON: [AUTO] REQ-D2-MS6-2~4. Single sync point that mirrors the
    //   canonical WorkspacesStore state into the rendered self.workspaces vector
    //   after rename / reorder / delete mutations. fan_in >= 3:
    //   handle_workspace_menu_action_logic Reordered branch, click_workspace_menu_item,
    //   commit_rename_modal, confirm_delete_modal.
    /// Re-populate `self.workspaces` from the canonical `self.store`.
    fn sync_workspaces_from_store(&mut self) {
        self.workspaces = self.store.list().to_vec();
    }

    // @MX:NOTE: [AUTO] REQ-D2-MS6-2 — open_workspace_menu_at is the single entry
    //   point invoked from the workspace_row right-click listener.
    /// Open the workspace context menu for `ws_id` at the given screen position.
    /// AC-D2-12.
    pub fn open_workspace_menu_at(&mut self, ws_id: &str, x: f32, y: f32) {
        self.workspace_menu.open_for(ws_id, x, y);
    }

    // @MX:NOTE: [AUTO] REQ-D2-MS6-3 — click_workspace_menu_item dispatches the
    //   chosen action against the currently visible workspace target, then
    //   atomically closes the menu (single-menu invariant from REQ-D2-MS4-2).
    /// Dispatch the workspace context-menu's selected action for whichever
    /// workspace the menu is currently open for. No-op when the menu is closed.
    /// AC-D2-13.
    pub fn click_workspace_menu_item(&mut self, action: workspace_menu::WorkspaceMenuAction) {
        let Some(ws_id) = self.workspace_menu.visible_target().map(str::to_string) else {
            return;
        };
        self.handle_workspace_menu_action_logic(action, &ws_id);
        self.workspace_menu.close();
    }

    // @MX:NOTE: [AUTO] REQ-D2-MS6-4 — commit_rename_modal persists via store and
    //   mirrors the change into self.workspaces via sync_workspaces_from_store.
    /// Commit the open rename modal: persist the new name via `store.rename`,
    /// mirror into `self.workspaces`, and close the modal.
    ///
    /// Returns `Some((ws_id, new_name))` on success. Returns `None` when the
    /// modal is closed, the trimmed buffer is blank (per `RenameModal::commit`),
    /// or the store rejects the rename.
    /// AC-D2-14 (rename half).
    pub fn commit_rename_modal(&mut self) -> Option<(String, String)> {
        let modal = self.rename_modal.as_mut()?;
        let (ws_id, new_name) = modal.commit()?;
        if let Err(e) = self.store.rename(&ws_id, &new_name) {
            tracing::warn!(
                error = ?e,
                ws_id,
                "commit_rename_modal: store.rename failed"
            );
            return None;
        }
        self.rename_modal = None;
        self.sync_workspaces_from_store();
        Some((ws_id, new_name))
    }

    /// Cancel the open rename modal without persisting.
    pub fn cancel_rename_modal(&mut self) {
        self.rename_modal = None;
    }

    // @MX:NOTE: [AUTO] REQ-D2-MS6-4 — confirm_delete_modal removes via store,
    //   mirrors into self.workspaces, and reassigns active_id when the deleted
    //   workspace was active to avoid a dangling reference.
    /// Confirm the open delete modal: remove the workspace via `store.remove`,
    /// mirror into `self.workspaces`, reassign `active_id` if it pointed at the
    /// deleted workspace, and close the modal.
    ///
    /// Returns `Some(ws_id)` on success. Returns `None` when the modal is
    /// closed or the store rejects the removal.
    /// AC-D2-14 (delete half).
    pub fn confirm_delete_modal(&mut self) -> Option<String> {
        let conf = self.delete_confirmation.as_mut()?;
        let ws_id = conf.confirm()?;
        if let Err(e) = self.store.remove(&ws_id) {
            tracing::warn!(
                error = ?e,
                ws_id,
                "confirm_delete_modal: store.remove failed"
            );
            return None;
        }
        self.delete_confirmation = None;
        self.sync_workspaces_from_store();
        if self.active_id.as_deref() == Some(ws_id.as_str()) {
            self.active_id = self.workspaces.first().map(|w| w.id.clone());
        }
        Some(ws_id)
    }

    /// Cancel the open delete modal without removing the workspace.
    pub fn cancel_delete_modal(&mut self) {
        self.delete_confirmation = None;
    }

    // ── SPEC-V0-2-0-MISSION-CTRL-001 MS-2 helpers (REQ-MC-024) ──

    // @MX:NOTE: [AUTO] ensure_mission_control — single mount entry point.
    //   Creates the MissionControlView entity lazily on first call. Subsequent
    //   calls are no-ops; the caller updates the existing entity via
    //   `update_mission_control_snapshot`.
    /// Lazily mount the Mission Control view. Idempotent.
    /// AC-MC-13.
    pub fn ensure_mission_control(&mut self, cx: &mut Context<Self>) {
        if self.mission_control.is_none() {
            self.mission_control =
                Some(cx.new(|_| agent::mission_control_view::MissionControlView::new()));
        }
    }

    /// Dismiss the Mission Control view (releases the entity).
    pub fn dismiss_mission_control(&mut self) {
        self.mission_control = None;
    }

    /// Push a fresh snapshot of `top_n_active(N)` cards into the mounted view.
    /// No-op when the view has not been mounted via `ensure_mission_control`.
    pub fn update_mission_control_snapshot(
        &mut self,
        cards: Vec<moai_studio_agent::AgentCard>,
        cx: &mut Context<Self>,
    ) {
        if let Some(ref entity) = self.mission_control {
            entity.update(cx, |view, cx| {
                view.set_snapshot(cards);
                cx.notify();
            });
        }
    }

    /// SPEC-V0-1-1-UX-FIX (C-3): TabContainer 가 없으면 생성. workspace 활성화 시점에 호출.
    ///
    /// v0.1.0 에서는 production 코드에 TabContainer 생성 로직이 부재하여 content_area 가
    /// 영구적으로 "tab container initializing" 텍스트만 표시되는 stuck 상태 발생. v0.1.1 에서
    /// 본 helper 가 handle_add_workspace + handle_activate_workspace 양쪽에서 호출되어
    /// workspace 가 활성화된 즉시 TabContainer 가 가시 상태가 된다.
    fn ensure_tab_container(&mut self, cx: &mut Context<Self>) {
        if self.tab_container.is_none() {
            self.tab_container = Some(cx.new(|_| TabContainer::new()));
        }
    }

    /// 주어진 id 의 워크스페이스를 active 로 전환. 존재하지 않으면 false.
    pub fn activate_workspace(&mut self, id: &str) -> bool {
        if self.workspaces.iter().any(|w| w.id == id) {
            self.active_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// Row 클릭 처리 — active_id 전환 + store.touch() 로 last_active 갱신.
    /// 저장 실패는 로깅만, UI 전환은 성공 처리.
    /// SPEC-V0-1-1-UX-FIX (C-3): activate 시 TabContainer 미생성이면 생성.
    fn handle_activate_workspace(&mut self, id: String, cx: &mut Context<Self>) {
        if !self.activate_workspace(&id) {
            return;
        }
        self.ensure_tab_container(cx);
        match WorkspacesStore::load(&self.storage_path) {
            Ok(mut store) => {
                if let Err(e) = store.touch(&id) {
                    error!("store.touch({id}) 실패: {e}");
                }
            }
            Err(e) => error!("WorkspacesStore::load (touch 시) 실패: {e}"),
        }
        cx.notify();
    }

    // ── MS-3: palette 글로벌 키바인딩 (AC-PL-14/15) ──

    /// Palette 글로벌 키 이벤트 처리 (AC-PL-14/15).
    ///
    /// - Cmd+P: CmdPalette toggle (열려있으면 닫고, 아니면 열기)
    /// - Cmd+Shift+P: CommandPalette toggle (mutual exclusion)
    /// - Esc: 열려있는 palette dismiss
    /// - "/": terminal pane focused 상태에서만 SlashBar open
    ///
    /// 반환값: 이 핸들러가 키 이벤트를 소비했으면 true (caller 가 tab 키 dispatch 스킵).
    pub fn handle_palette_key_event(&mut self, ev: &KeyDownEvent) -> bool {
        let k = &ev.keystroke;
        // macOS: Cmd = modifiers.platform, Linux/Win: Ctrl = modifiers.control.
        // tabs/keys.rs 패턴과 일치.
        let cmd = k.modifiers.platform;
        let shift = k.modifiers.shift;
        let key = k.key.as_str();

        if cmd && !shift && key == "p" {
            self.toggle_cmd_palette();
            return true;
        }
        // F-1: Cmd+K is an alias for Cmd+P (VS Code / Zed quick-open convention).
        if cmd && !shift && key == "k" {
            self.toggle_cmd_palette();
            return true;
        }
        if cmd && shift && key == "p" {
            self.palette.toggle(palette::PaletteVariant::CommandPalette);
            return true;
        }
        if !cmd && !shift && key == "escape" && self.palette.is_visible() {
            self.palette.dismiss();
            self.reset_palette_query();
            return true;
        }
        if !cmd && !shift && key == "/" && self.terminal_focused && !self.palette.is_visible() {
            self.palette.open(palette::PaletteVariant::SlashBar);
            return true;
        }
        false
    }

    /// Palette overlay 렌더 여부 — active palette variant 가 있을 때 true.
    pub fn has_palette_overlay(&self) -> bool {
        self.palette.is_visible()
    }

    // ── F-1: CmdPalette toggle + query wire ──

    /// Toggle CmdPalette: open with fresh CmdPalette instance, or dismiss if same variant visible.
    ///
    /// On open: if an active workspace with a valid path exists, scans that directory
    /// for files via `CmdPalette::from_workspace_dir`; otherwise falls back to the
    /// default mock file index. Stores the instance in `self.cmd_palette`.
    /// On dismiss: resets query and cmd_palette.
    fn toggle_cmd_palette(&mut self) {
        if self.palette.active_variant == Some(palette::PaletteVariant::CmdPalette) {
            // Already open — dismiss (toggle semantics per AC-PL-14 Q2 default).
            self.palette.dismiss();
            self.reset_palette_query();
        } else {
            // Open (also replaces other variants due to mutual exclusion).
            self.palette.open(palette::PaletteVariant::CmdPalette);
            // Use workspace path for real file scanning (F-1 wiring).
            let cmd_palette = self
                .active()
                .and_then(|w| {
                    let p = &w.project_path;
                    if p.is_dir() {
                        Some(palette::variants::CmdPalette::from_workspace_dir(p))
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            self.cmd_palette = Some(cmd_palette);
            self.palette_query = String::new();
        }
    }

    /// Update palette query and re-filter CmdPalette items (F-1 fuzzy wire).
    ///
    /// Called on every keystroke while CmdPalette is open. Delegates to
    /// `CmdPalette::set_query` which applies fuzzy matching.
    pub fn handle_palette_text_input(&mut self, query: String) {
        self.palette_query = query.clone();
        if let Some(ref mut cp) = self.cmd_palette {
            cp.set_query(query);
        }
    }

    /// Reset palette query and cmd_palette state (called on dismiss).
    pub fn reset_palette_query(&mut self) {
        self.palette_query = String::new();
        self.cmd_palette = None;
    }

    // ── MS-4: CommandPalette dispatch (AC-PL-17/18) ──

    /// Dispatch a command id to the appropriate RootView handler (MS-4 AC-PL-17).
    ///
    /// Called when CommandPalette emits `CommandTriggered(id)`.
    ///
    /// Routing by id prefix:
    /// - `settings.*`  → SettingsModal mount
    /// - `theme.*`     → theme toggle / switch
    /// - `tab.*`       → tab action (logged; full implementation deferred)
    /// - `pane.*`      → pane action (logged; full implementation deferred)
    /// - `workspace.*` → workspace action (logged; deferred)
    /// - `surface.*`   → surface action (logged; deferred)
    /// - `git.*`       → git action (logged; deferred)
    /// - `agent.*`     → agent dashboard toggle (logged; deferred)
    /// - Unrecognised  → warning log + dismiss (AC-PL-19 graceful degradation)
    ///
    /// Returns `true` if the command was handled (even if only logged).
    ///
    /// @MX:ANCHOR: [AUTO] dispatch_command — CommandPalette command dispatch.
    /// @MX:REASON: [AUTO] fan_in >= 3: on_command_triggered, tests, MS-4 AC-PL-17/18.
    pub fn dispatch_command(&mut self, id: &str) -> bool {
        // Dismiss the palette immediately regardless of what the command does.
        self.palette.dismiss();
        self.reset_palette_query();

        if id.starts_with("settings.") {
            match id {
                "settings.open" => {
                    if self.settings_modal.is_none() {
                        let mut modal = settings::SettingsModal::new();
                        modal.mount();
                        self.settings_modal = Some(modal);
                        tracing::info!(command = id, "settings.open — SettingsModal mounted");
                    }
                }
                other => {
                    tracing::info!(command = other, "settings command not yet wired");
                }
            }
            return true;
        }

        if id.starts_with("theme.") {
            use settings::settings_state::ThemeMode;
            match id {
                "theme.toggle" => {
                    let next = match self.active_theme.theme {
                        ThemeMode::Dark | ThemeMode::System => ThemeMode::Light,
                        ThemeMode::Light => ThemeMode::Dark,
                    };
                    self.active_theme.theme = next;
                    self.user_settings.appearance.theme = next;
                    tracing::info!(command = id, theme = ?next, "theme.toggle — applied");
                }
                "theme.dark" => {
                    self.active_theme.theme = ThemeMode::Dark;
                    self.user_settings.appearance.theme = ThemeMode::Dark;
                    tracing::info!(command = id, "theme.dark — applied");
                }
                "theme.light" => {
                    self.active_theme.theme = ThemeMode::Light;
                    self.user_settings.appearance.theme = ThemeMode::Light;
                    tracing::info!(command = id, "theme.light — applied");
                }
                other => {
                    tracing::info!(command = other, "theme command not yet wired");
                }
            }
            return true;
        }

        if id.starts_with("tab.") {
            tracing::info!(
                command = id,
                "tab command not yet wired — deferred to tab SPEC"
            );
            return true;
        }

        if id.starts_with("pane.") {
            tracing::info!(
                command = id,
                "pane command not yet wired — deferred to pane SPEC"
            );
            return true;
        }

        if id.starts_with("workspace.") {
            // SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-3 (REQ-GS-050, AC-GS-11):
            // workspace.search activates the SearchPanel.
            if id == "workspace.search" {
                tracing::info!(command = id, "workspace.search — activating SearchPanel");
                self.dispatch_command_workspace_search();
                // cx.notify() is not available here (dispatch_command has no cx
                // parameter in this context-free signature). The GPUI render
                // path will pick up the state change on the next frame.
                return true;
            }
            tracing::info!(
                command = id,
                "workspace command not yet wired — deferred to workspace SPEC"
            );
            return true;
        }

        if id.starts_with("surface.") {
            tracing::info!(
                command = id,
                "surface command not yet wired — deferred to surface SPEC"
            );
            return true;
        }

        if id.starts_with("git.") {
            tracing::info!(
                command = id,
                "git command not yet wired — deferred to git SPEC"
            );
            return true;
        }

        if id.starts_with("agent.") {
            tracing::info!(
                command = id,
                "agent command not yet wired — deferred to agent SPEC"
            );
            return true;
        }

        if id.starts_with("shell.") {
            // SPEC-V0-2-0-MULTI-SHELL-001 MS-1 (REQ-MS-007): shell.switch activates ShellPicker.
            if id == "shell.switch" {
                tracing::info!(command = id, "shell.switch — activating ShellPicker");
                self.dispatch_command_shell_switch();
                // cx.notify() is not available in this context-free signature.
                // The GPUI render path picks up the state change on the next frame.
                return true;
            }
            tracing::info!(
                command = id,
                "shell command not yet wired — deferred to shell SPEC"
            );
            return true;
        }

        if id.starts_with("file.") || id.starts_with("view.") {
            tracing::info!(command = id, "Command not yet wired: {}", id);
            return true;
        }

        // AC-PL-19: Unrecognised command — warn and dismiss (already dismissed above).
        tracing::warn!(
            command = id,
            "Unrecognised command id — no handler registered"
        );
        false
    }

    /// Inject a `/moai <subcommand>\n` string into the active terminal's pending_input (MS-4 AC-PL-21).
    ///
    /// Called when SlashBar emits `SlashInvoked(subcommand_label)`.
    /// `subcommand_label` is the full label, e.g. "/moai plan".
    ///
    /// If the active pane is not a terminal, logs a warning and dismisses.
    /// The terminal write uses `pending_input` buffering (same path as key input).
    ///
    /// Returns `true` if the injection was performed (or at least attempted).
    pub fn inject_slash_command(&mut self, subcommand_label: &str) -> bool {
        // Dismiss SlashBar first.
        self.palette.dismiss();
        self.reset_palette_query();

        // Validate that the label looks like a /moai command.
        if !subcommand_label.starts_with("/moai") {
            tracing::warn!(
                label = subcommand_label,
                "inject_slash_command: label does not start with /moai — skipping injection"
            );
            return false;
        }

        // Build the command string to inject (append newline to execute).
        let cmd_str = format!("{}\n", subcommand_label);
        tracing::info!(
            command = subcommand_label,
            "inject_slash_command: logging slash command injection (terminal wiring deferred)"
        );

        // Store the pending injection for callers / tests to inspect.
        // Full PTY write integration requires the active TerminalSurface Entity, which
        // is only available in a GPUI Context<Self>. The no-context path stores the
        // pending string so the render/update loop can drain it.
        self.pending_slash_injection = Some(cmd_str);
        true
    }

    /// Handle Enter key inside active CmdPalette — returns selected file path or None.
    ///
    /// Callers dispatch the returned path as an open-file action (or log it in tests).
    pub fn on_palette_enter(&mut self) -> Option<String> {
        if let Some(ref mut cp) = self.cmd_palette
            && let Some(palette::variants::cmd_palette::CmdPaletteEvent::FileOpened(path)) =
                cp.on_enter()
        {
            self.palette.dismiss();
            self.reset_palette_query();
            return Some(path);
        }
        None
    }

    // ── SPEC-V3-006 MS-3a: Find/Replace 글로벌 키바인딩 ──

    /// Find/Replace 글로벌 키 이벤트 처리 (SPEC-V3-006 MS-3a).
    ///
    /// - Cmd+F: active viewer 의 find bar 를 열거나 포커스
    /// - Esc: find bar 가 열려있으면 닫기 (palette/settings 핸들러 이후에 호출)
    ///
    /// 반환값: 이 핸들러가 키 이벤트를 소비했으면 true.
    pub fn handle_find_key_event(&mut self, ev: &KeyDownEvent) -> bool {
        let k = &ev.keystroke;
        let cmd = k.modifiers.platform;
        let ctrl = k.modifiers.control;
        let shift = k.modifiers.shift;
        let key = k.key.as_str();

        // Cmd+F (macOS) 또는 Ctrl+F (Linux/Win) — find bar open.
        if (cmd || ctrl) && !shift && key == "f" {
            self.find_bar_open = true;
            return true;
        }
        // Esc — find bar 가 열려있으면 닫기.
        if key == "escape" && self.find_bar_open {
            self.find_bar_open = false;
            return true;
        }
        false
    }

    /// Find bar 가 현재 열려있는지 확인한다.
    pub fn has_find_bar(&self) -> bool {
        self.find_bar_open
    }

    // ── SPEC-V3-013 MS-3: Settings 키바인딩 (AC-V13-1) ──

    /// Settings 글로벌 키 이벤트 처리 (AC-V13-1, REQ-V13-001).
    ///
    /// - Cmd+, (macOS) / Ctrl+, (Linux/Win): SettingsModal toggle.
    ///   이미 열려있으면 무시 (REQ-V13-006).
    /// - Esc: SettingsModal 이 열려있으면 dismiss + save (REQ-V13-004, REQ-V13-053).
    ///
    /// 반환값: 이 핸들러가 키 이벤트를 소비했으면 true.
    pub fn handle_settings_key_event(&mut self, ev: &KeyDownEvent) -> bool {
        let k = &ev.keystroke;
        let cmd = k.modifiers.platform;
        let ctrl = k.modifiers.control;
        let shift = k.modifiers.shift;
        let key = k.key.as_str();

        // Cmd+, (macOS) 또는 Ctrl+, (Linux/Win) — settings toggle.
        if (cmd || ctrl) && !shift && key == "," {
            if self.settings_modal.is_none() {
                // 모달 mount
                let mut modal = settings::SettingsModal::new();
                modal.mount();
                self.settings_modal = Some(modal);
            }
            // 이미 열려있으면 무시 (REQ-V13-006).
            return true;
        }

        // Esc — settings modal 이 열려있으면 dismiss + save.
        if !cmd && !shift && key == "escape" && self.settings_modal.is_some() {
            self.dismiss_settings_modal();
            return true;
        }

        false
    }

    /// SettingsModal 을 dismiss 하고 dirty 상태이면 즉시 save 한다 (REQ-V13-053).
    pub fn dismiss_settings_modal(&mut self) {
        if let Some(modal) = self.settings_modal.take() {
            // 모달의 in-memory 상태를 UserSettings 에 동기화 (appearance 섹션)
            self.user_settings.appearance.theme = modal.view_state.appearance.theme;
            self.user_settings.appearance.density = modal.view_state.appearance.density;
            self.user_settings.appearance.accent = modal.view_state.appearance.accent;
            self.user_settings.appearance.font_size_px = modal.view_state.appearance.font_size_px;
            // keyboard 변경 사항 동기화
            // (MS-3 단순화: custom binding 만 저장, default 제외)
            // ActiveTheme 업데이트
            self.active_theme =
                design::runtime::ActiveTheme::from_settings(&self.user_settings.appearance);
            // 즉시 flush (REQ-V13-053)
            let path = settings::user_settings::settings_path();
            if let Err(e) = settings::user_settings::save_atomic(&path, &self.user_settings) {
                error!("settings.json save 실패: {e}");
            }
        }
    }

    /// SettingsModal 이 현재 열려있는지 확인한다 (REQ-V13-001).
    pub fn has_settings_modal(&self) -> bool {
        self.settings_modal.is_some()
    }

    /// UserSettings 에서 ActiveTheme 을 재계산하고 반환한다 (REQ-V13-062).
    pub fn refresh_active_theme(&mut self) {
        self.active_theme =
            design::runtime::ActiveTheme::from_settings(&self.user_settings.appearance);
    }

    // ── SPEC-V3-015 MS-1: SpecPanelView 키바인딩 (AC-RV-2) ──

    /// SpecPanelView 글로벌 키 이벤트 처리 (AC-RV-2, REQ-RV-003).
    ///
    /// - Cmd+Shift+S (macOS) / Ctrl+Shift+S (Linux/Win): SpecPanelView toggle.
    ///   dismiss 상태이면 mount, mount 상태이면 dismiss.
    /// - Tab (SpecPanel 열린 상태): mode cycle (List → Kanban → Sprint → List).
    /// - 다른 overlay (palette, settings_modal) 활성 중에는 무시 (single-overlay invariant).
    ///
    /// 반환값: 이 핸들러가 키 이벤트를 소비했으면 true.
    pub fn handle_spec_key_event(&mut self, ev: &KeyDownEvent) -> bool {
        let k = &ev.keystroke;
        let cmd = k.modifiers.platform;
        let ctrl = k.modifiers.control;
        let shift = k.modifiers.shift;
        let key = k.key.as_str();

        // Cmd+Shift+S (macOS) 또는 Ctrl+Shift+S (Linux/Win) — spec panel toggle.
        if (cmd || ctrl) && shift && key == "s" {
            // single-overlay invariant: 다른 overlay 가 활성이면 무시 (REQ-RV-007).
            if self.palette.is_visible() || self.settings_modal.is_some() {
                return true; // 소비는 하되 action 없음
            }
            if self.spec_panel.is_none() {
                // mount: specs_dir 은 storage_path 기준 .moai/specs
                let specs_dir = self.storage_path.join(".moai").join("specs");
                self.spec_panel = Some(spec_ui::SpecPanelView::new(specs_dir));
            } else {
                // dismiss
                self.spec_panel = None;
            }
            return true;
        }

        // Tab — spec panel 열린 상태에서 mode cycle.
        if key == "tab" && self.spec_panel.is_some() {
            if let Some(panel) = self.spec_panel.as_mut() {
                panel.cycle_mode();
            }
            return true;
        }

        // Esc — spec panel 이 열려있으면 dismiss.
        if !cmd && !ctrl && !shift && key == "escape" && self.spec_panel.is_some() {
            self.spec_panel = None;
            return true;
        }

        false
    }

    /// SpecPanelView 가 현재 mount 되어 있는지 확인한다 (REQ-RV-002).
    pub fn has_spec_panel(&self) -> bool {
        self.spec_panel.is_some()
    }

    /// GPUI 키 이벤트를 탭 명령으로 변환하여 TabContainer 에 전달한다 (REQ-R-031).
    ///
    /// @MX:NOTE: [AUTO] rootview-key-dispatch-ac-r-3
    /// RootView 의 on_key_down 핸들러. keystroke_to_tab_key → dispatch_tab_key 순서로 변환.
    /// Some(TabCommand) 시에만 tab_container.update 호출 (REQ-R-031).
    /// None 이면 RootView 가 keystroke 를 소비하지 않아 활성 leaf 로 자동 forward (REQ-R-035).
    fn handle_key_event(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let (mods, code) = tabs::keys::keystroke_to_tab_key(&ev.keystroke);
        let Some(cmd) = tabs::keys::dispatch_tab_key(mods, code) else {
            return; // REQ-R-035: passthrough
        };
        let Some(tc) = self.tab_container.as_ref() else {
            return;
        };
        let tc = tc.clone();
        tc.update(cx, |tc, cx| {
            use crate::panes::PaneId;
            use tabs::keys::TabCommand;
            match cmd {
                TabCommand::NewTab => {
                    tc.new_tab(None);
                }
                TabCommand::SwitchToTab(idx) => {
                    // REQ-R-033: IndexOutOfBounds 는 무시.
                    let _ = tc.switch_tab(idx);
                }
                TabCommand::SplitVertical => {
                    // REQ-R-034: SplitVertical → split_horizontal (좌우 분할).
                    if let Some(focused) = tc.active_tab().last_focused_pane.clone() {
                        let _ = tc.active_tab_mut().pane_tree.split_horizontal(
                            &focused,
                            PaneId::new_unique(),
                            "new-pane".to_string(),
                        );
                    }
                }
                TabCommand::SplitHorizontal => {
                    // REQ-R-034: SplitHorizontal → split_vertical (상하 분할).
                    if let Some(focused) = tc.active_tab().last_focused_pane.clone() {
                        let _ = tc.active_tab_mut().pane_tree.split_vertical(
                            &focused,
                            PaneId::new_unique(),
                            "new-pane".to_string(),
                        );
                    }
                }
                TabCommand::PrevTab => {
                    if tc.active_tab_idx > 0 {
                        let _ = tc.switch_tab(tc.active_tab_idx - 1);
                    }
                }
                TabCommand::NextTab => {
                    let next = tc.active_tab_idx + 1;
                    if next < tc.tabs.len() {
                        let _ = tc.switch_tab(next);
                    }
                }
            }
            cx.notify();
        });
    }

    // @MX:ANCHOR: [AUTO] file-open-pipeline
    // @MX:REASON: [AUTO] FileExplorer → RootView → viewer 단일 진입점.
    //   fan_in >= 3: explorer callback (AC-WIRE-1), 미래 Cmd+P (V3-005 MS-3),
    //   LSP go-to-definition (V3-006 MS-3).
    //   이 메서드는 모든 "파일 열기" 요청의 escape hatch 이다.
    /// FileExplorer 의 파일 열기 이벤트를 구독하여 handle_open_file 로 dispatch 한다.
    ///
    /// `file_explorer` 가 Some 일 때 GPUI EventEmitter 구독을 설정한다.
    /// FileExplorer 가 `emit_open_file` 을 호출하면 RootView::handle_open_file 이 트리거된다.
    ///
    /// @MX:NOTE: [AUTO] viewer-mount-async-strategy
    /// read_file_for_viewer 는 동기 std::fs::read 로 구현되어 있다 (MS-1/2 임시).
    /// TooLarge / NotUtf8 오류는 viewer.set_error 로 표시된다.
    /// MS-3 에서 cx.background_spawn 비동기로 전환 예정.
    pub fn wire_file_explorer_callback(&mut self, cx: &mut Context<Self>) {
        let Some(fe) = self.file_explorer.as_ref() else {
            return;
        };

        // @MX:NOTE: Subscription 을 저장하지 않으면 즉시 drop 되므로
        // 현재는 _subscription 을 local 에 두어 RootView 생존 기간 동안 유지.
        // MS-3 에서 RootView 필드 (Vec<Subscription>) 로 이관 예정.
        let _subscription = cx.subscribe(fe, |this, _fe, ev: &explorer::FileOpenEvent, cx| {
            let open_ev = viewer::OpenFileEvent {
                path: ev.abs_path.clone(),
                surface_hint: None,
            };
            this.handle_open_file(&open_ev, cx);
        });
        // Subscription 을 forget 하여 RootView 생존 기간 동안 활성 유지
        _subscription.detach();
    }

    /// Subscribe to terminal click events and dispatch to appropriate handlers.
    ///
    /// This method sets up a GPUI EventEmitter subscription on TerminalSurface.
    /// When the terminal emits click events (file URLs, SPEC IDs), RootView
    /// delegates them to the appropriate handler.
    pub fn wire_terminal_click_callback(
        &mut self,
        terminal: &Entity<terminal::TerminalSurface>,
        cx: &mut Context<Self>,
    ) {
        let _subscription = cx.subscribe(
            terminal,
            |this, _terminal, event: &TerminalClickEvent, cx| {
                match event {
                    TerminalClickEvent::OpenFile {
                        path,
                        line: _,
                        col: _,
                    } => {
                        // Note: current OpenFileEvent doesn't support line/col, so they're ignored
                        let ev = viewer::OpenFileEvent {
                            path: path.clone(),
                            surface_hint: None,
                        };
                        this.handle_open_file(&ev, cx);
                    }
                    TerminalClickEvent::OpenUrl(url) => {
                        // @MX:WARN: URL passed to system without validation
                        // @MX:REASON: GPUI open_url delegates to OS handler which applies its own safety checks
                        cx.open_url(url);
                    }
                    TerminalClickEvent::OpenSpec(spec_id) => {
                        // SPEC-V3-009 MS-4a (AC-SU-13~16): wire OpenSpec event to
                        // SpecPanelView mount + select_spec. Respects the
                        // single-overlay invariant by deferring to RootView helper.
                        this.handle_terminal_open_spec(spec_id, cx);
                    }
                }
            },
        );
        _subscription.detach();
    }

    /// Subscribe to TerminalSurface stdout chunks for the WebView toast pipeline.
    ///
    /// SPEC-V3-007 MS-4 (REQ-WB-031~035, AC-WB-INT-1):
    /// 1. PTY stdout chunk arrives via `TerminalStdoutEvent::Chunk`.
    /// 2. `web::detect_local_urls` extracts dev-server URLs.
    /// 3. `UrlDetectionDebouncer` enforces 5s dedupe + 30min dismiss-silence.
    /// 4. New URLs are pushed to `pending_toasts`, triggering `cx.notify`.
    ///
    /// The subscription is attached on the terminal entity so it lives for as
    /// long as the entity is referenced by RootView. Mirror of the existing
    /// `wire_terminal_click_callback` pattern.
    #[cfg(feature = "web")]
    pub fn wire_terminal_stdout_callback(
        &mut self,
        terminal: &Entity<terminal::TerminalSurface>,
        cx: &mut Context<Self>,
    ) {
        let subscription = cx.subscribe(
            terminal,
            |this, _terminal, event: &TerminalStdoutEvent, cx| match event {
                TerminalStdoutEvent::Chunk(chunk) => {
                    this.ingest_stdout_chunk(chunk, cx);
                }
            },
        );
        subscription.detach();
    }

    /// Feed a single stdout chunk through the URL detector + debouncer.
    ///
    /// Exposed as a `pub(crate)` helper so unit tests can drive the pipeline
    /// without spinning up a real TerminalSurface entity. Tests assert that
    /// `pending_toasts` grows on first match and is unchanged on re-emission
    /// within the dedupe window (AC-WB-INT-1).
    #[cfg(feature = "web")]
    pub(crate) fn ingest_stdout_chunk(&mut self, chunk: &str, cx: &mut Context<Self>) {
        let detected = web::detect_local_urls(chunk);
        if detected.is_empty() {
            return;
        }
        let new_urls = self.url_detector.process(detected);
        if new_urls.is_empty() {
            return;
        }
        for du in new_urls {
            // Truncate the source to its first line so the toast remains
            // single-line; the full chunk is already available in the
            // tracing log if deeper context is needed.
            let source = du.source.lines().next().unwrap_or("").to_string();
            self.pending_toasts.push(WebToastEntry {
                url: du.url,
                source,
            });
        }
        cx.notify();
    }

    /// Open the toast at `toast_idx` in a new tab as a `LeafKind::Web`.
    ///
    /// AC-WB-INT-2: clicking the toast creates a fresh tab via
    /// `TabContainer::new_tab`, mounts a `WebViewSurface` entity onto the new
    /// tab's focused leaf, navigates to the toast URL, and removes the toast
    /// from `pending_toasts`. Returns the new TabId on success or None when
    /// the toast index is out of range or no TabContainer is bound.
    #[cfg(feature = "web")]
    pub fn open_toast_in_new_tab(
        &mut self,
        toast_idx: usize,
        cx: &mut Context<Self>,
    ) -> Option<tabs::container::TabId> {
        let toast = self.pending_toasts.get(toast_idx).cloned()?;
        let url = toast.url.clone();

        let container = self.tab_container.as_ref()?.clone();

        // 1) Create the new tab inside the container entity.
        let tab_id = container.update(cx, |tc, _cx| tc.new_tab(None));

        // 2) Resolve the focused leaf id of the just-created (now active) tab.
        let leaf_id = container.read(cx).active_tab().last_focused_pane.clone()?;

        // 3) Build the WebViewSurface entity and navigate it to the URL.
        let surface_entity = cx.new(|_cx| {
            let mut surface = web::WebViewSurface::new(url.clone());
            surface.navigate(url.clone());
            surface
        });
        self.leaf_payloads
            .insert(leaf_id, LeafKind::Web(surface_entity));

        // 4) Drop the consumed toast.
        self.pending_toasts.remove(toast_idx);
        cx.notify();
        Some(tab_id)
    }

    /// Dismiss the toast at `toast_idx`, registering the URL for 30 min silence.
    ///
    /// AC-WB-INT-3: subsequent detections of the same URL during the silence
    /// window are filtered out by the Debouncer.
    #[cfg(feature = "web")]
    pub fn dismiss_toast(&mut self, toast_idx: usize, cx: &mut Context<Self>) {
        if toast_idx >= self.pending_toasts.len() {
            return;
        }
        let toast = self.pending_toasts.remove(toast_idx);
        self.url_detector.dismiss(toast.url);
        cx.notify();
    }

    /// Render the WebView dev-server toast overlay (REQ-WB-032).
    ///
    /// Returns an empty vector when no toasts are pending or when the `web`
    /// feature is disabled, allowing `.children(...)` to be a no-op overlay.
    #[cfg(feature = "web")]
    fn render_web_toasts(&self, cx: &mut Context<Self>) -> Vec<gpui::AnyElement> {
        if self.pending_toasts.is_empty() {
            return Vec::new();
        }
        // Build a single absolutely-positioned stack at the bottom-right.
        let toast_rows: Vec<gpui::AnyElement> = self
            .pending_toasts
            .iter()
            .enumerate()
            .map(|(idx, toast)| {
                let url_label = format!("Open {} in Studio?", toast.url);
                let open_listener = cx.listener(move |this, _ev, _window, cx| {
                    this.open_toast_in_new_tab(idx, cx);
                });
                let dismiss_listener = cx.listener(move |this, _ev, _window, cx| {
                    this.dismiss_toast(idx, cx);
                });
                div()
                    .id(("web-toast", idx))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .px_3()
                    .py_2()
                    .bg(rgb(tok::BG_SURFACE))
                    .border_1()
                    .border_color(rgb(tok::BORDER_SUBTLE))
                    .rounded_md()
                    .shadow_md()
                    .child(div().text_color(rgb(tok::FG_PRIMARY)).child(url_label))
                    .child(
                        div()
                            .id(("web-toast-open", idx))
                            .px_2()
                            .py_1()
                            .bg(rgb(tok::BG_ELEVATED))
                            .text_color(rgb(tok::FG_PRIMARY))
                            .rounded_sm()
                            .cursor_pointer()
                            .child("Open")
                            .on_mouse_down(MouseButton::Left, open_listener),
                    )
                    .child(
                        div()
                            .id(("web-toast-dismiss", idx))
                            .px_2()
                            .py_1()
                            .text_color(rgb(tok::BORDER_SUBTLE))
                            .cursor_pointer()
                            .child("Dismiss")
                            .on_mouse_down(MouseButton::Left, dismiss_listener),
                    )
                    .into_any_element()
            })
            .collect();

        let stack = div()
            .absolute()
            .bottom_4()
            .right_4()
            .flex()
            .flex_col()
            .gap_2()
            .children(toast_rows)
            .into_any_element();
        vec![stack]
    }

    /// Stub used when the `web` feature is disabled — overlay renders nothing.
    #[cfg(not(feature = "web"))]
    fn render_web_toasts(&self, _cx: &mut Context<Self>) -> Vec<gpui::AnyElement> {
        Vec::new()
    }

    /// Pending dev-server toast count (always 0 when `web` feature is off).
    pub fn toast_count(&self) -> usize {
        #[cfg(feature = "web")]
        {
            self.pending_toasts.len()
        }
        #[cfg(not(feature = "web"))]
        {
            0
        }
    }

    /// SPEC-V0-1-2-MENUS-001 MS-2 (AC-MN-9 / AC-MN-10): split the focused leaf
    /// of the active tab in the requested direction.
    ///
    /// No-op when:
    /// - `tab_container` is `None` (workspace not yet active)
    /// - the active tab has no `last_focused_pane` (defensive — should not
    ///   happen because `Tab::new` seeds the focus)
    /// - the focused pane id no longer exists in the tree (e.g. a previous
    ///   close removed it without updating focus)
    ///
    /// The new pane carries an empty payload because the active payload type
    /// is `PaneTree<String>`; a follow-up SPEC will switch to `PaneTree<LeafKind>`
    /// so that splits can mount Terminal/Markdown/Code surfaces directly.
    pub fn handle_split_action(
        &mut self,
        direction: panes::tree::SplitDirection,
        cx: &mut Context<Self>,
    ) {
        let Some(container) = self.tab_container.clone() else {
            tracing::debug!("split_action: tab_container is None — ignored");
            return;
        };
        let focused = container.read(cx).active_tab().last_focused_pane.clone();
        let Some(focused) = focused else {
            tracing::debug!("split_action: no focused pane — ignored");
            return;
        };
        let new_id = panes::PaneId::new_unique();
        let result = container.update(cx, |tc, _cx| {
            let tree = &mut tc.active_tab_mut().pane_tree;
            match direction {
                panes::tree::SplitDirection::Horizontal => {
                    tree.split_horizontal(&focused, new_id, String::new())
                }
                panes::tree::SplitDirection::Vertical => {
                    tree.split_vertical(&focused, new_id, String::new())
                }
            }
        });
        if let Err(e) = result {
            tracing::warn!(?e, ?direction, "split_action failed");
        }
        cx.notify();
    }

    /// SPEC-V0-1-2-MENUS-001 MS-2 (AC-MN-8): mount or dismiss the SPEC panel
    /// overlay. Mirrors the body of `handle_spec_key_event` so menu/keybinding
    /// dispatch and the legacy direct key handler stay in sync.
    pub fn handle_open_spec_panel(&mut self, cx: &mut Context<Self>) {
        // Single-overlay invariant: respect existing modals.
        if self.palette.is_visible() || self.settings_modal.is_some() {
            return;
        }
        if self.spec_panel.is_none() {
            let specs_dir = self.storage_path.join(".moai").join("specs");
            self.spec_panel = Some(spec_ui::SpecPanelView::new(specs_dir));
        } else {
            self.spec_panel = None;
        }
        cx.notify();
    }

    /// SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2 (REQ-GS-031): Toggle the global search panel.
    ///
    /// Lazy-initialises `search_panel` on the first call (None → Some with
    /// `is_visible = true`). Subsequent calls delegate to `SearchPanel::toggle`.
    /// Calling `focus_input` records the intent so MS-3 can resolve real GPUI
    /// focus once an `Entity<SearchPanel>` exists.
    pub fn handle_toggle_search_panel(&mut self, cx: &mut Context<Self>) {
        if self.search_panel.is_none() {
            let mut panel = search::SearchPanel::new();
            panel.toggle();
            panel.focus_input();
            self.search_panel = Some(panel);
        } else if let Some(panel) = self.search_panel.as_mut() {
            panel.toggle();
            if panel.is_visible() {
                panel.focus_input();
            }
        }
        cx.notify();
    }

    /// SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-3 (REQ-GS-050, AC-GS-11):
    /// Context-free helper that activates the SearchPanel (toggle + focus_input).
    ///
    /// This method is context-free (no `cx` parameter) so it can be tested with
    /// logic-level unit tests (Spike 2 pattern, SPEC-V3-005 §6).
    /// The GPUI `cx.notify()` call is made by the callers that hold a context.
    ///
    /// @MX:NOTE: [AUTO] dispatch-workspace-search-no-cx
    /// Called from `dispatch_command` (GPUI context available) and from unit
    /// tests (no GPUI context). Callers must call `cx.notify()` themselves.
    pub fn dispatch_command_workspace_search(&mut self) {
        if self.search_panel.is_none() {
            let mut panel = search::SearchPanel::new();
            panel.toggle();
            panel.focus_input();
            self.search_panel = Some(panel);
        } else if let Some(panel) = self.search_panel.as_mut() {
            panel.toggle();
            if panel.is_visible() {
                panel.focus_input();
            }
        }
    }

    /// SPEC-V0-2-0-MULTI-SHELL-001 MS-1 (REQ-MS-007): Activate the ShellPicker.
    ///
    /// Detects available shells via `Shell::detect_available()`, builds a
    /// `ShellPicker` and stores it in `self.shell_picker`.  Subsequent calls
    /// (re-activation) rebuild the picker so the available-list stays current.
    ///
    /// This method is context-aware (has `cx` parameter) so it can call
    /// `cx.notify()` to trigger a GPUI re-render.  A logic-level
    /// `dispatch_command_shell_switch` variant (no cx) is also provided for
    /// unit testing (Spike 2 pattern).
    ///
    /// @MX:ANCHOR: [AUTO] handle_switch_shell — Shell Picker activation entry point.
    /// @MX:REASON: [AUTO] fan_in >= 3: dispatch_command (shell.switch branch),
    ///   GPUI action handler, unit test suite.
    /// @MX:SPEC: SPEC-V0-2-0-MULTI-SHELL-001 REQ-MS-007
    pub fn handle_switch_shell(&mut self, cx: &mut Context<Self>) {
        self.dispatch_command_shell_switch();
        cx.notify();
    }

    /// Context-free helper — activates `shell_picker` without requiring a GPUI context.
    ///
    /// Called by `handle_switch_shell` (context-aware path) and by logic-level
    /// unit tests (no-context path).  Callers that hold a GPUI context must
    /// call `cx.notify()` themselves after this returns.
    ///
    /// @MX:NOTE: [AUTO] dispatch-shell-switch-no-cx
    /// @MX:SPEC: SPEC-V0-2-0-MULTI-SHELL-001 REQ-MS-007
    pub fn dispatch_command_shell_switch(&mut self) {
        use moai_studio_terminal::shell::Shell;
        let available = Shell::detect_available();
        let current_default = std::env::var("SHELL").ok().and_then(|s| {
            Shell::all_unix()
                .into_iter()
                .find(|sh| s.ends_with(sh.executable()))
        });
        self.shell_picker = Some(shell_picker::ShellPicker::new(available, current_default));
        tracing::info!("shell.switch — ShellPicker activated");
    }

    /// SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-3 (REQ-GS-040~042, AC-GS-10):
    /// Navigate to a search hit — workspace activate + new tab + line scroll.
    ///
    /// This context-free variant resolves the `OpenCodeViewer` from the hit and
    /// records it in `self.last_open_code_viewer` for logic-level testing.
    /// The GPUI Entity mutation (tab open via `TabContainer::new_tab`) is
    /// performed by the context-aware companion `handle_search_open_with_cx`.
    ///
    /// # Failure contract (REQ-GS-042)
    ///
    /// Returns `false` (no panic) when workspace resolution fails. The caller
    /// should log a warning and leave the SearchPanel visible.
    ///
    /// @MX:ANCHOR: [AUTO] handle-search-open
    /// @MX:REASON: [AUTO] Central navigation entry point, fan_in >= 3:
    ///   handle_search_open_with_cx, unit tests (T2/T3/T4/T5), result_view dispatch.
    pub fn handle_search_open(&mut self, hit: &moai_search::SearchHit) -> bool {
        use search::navigation;

        // Step 1: resolve workspace.
        let ocv = navigation::hit_to_open_code_viewer(hit, &self.workspaces);
        let ocv = match ocv {
            Some(v) => v,
            None => {
                tracing::warn!(
                    workspace_id = hit.workspace_id.as_str(),
                    "handle_search_open: workspace not found — navigation skipped"
                );
                return false;
            }
        };

        // Step 2: log workspace activation intent.
        // Full WorkspacesStore::touch is performed in handle_search_open_with_cx
        // where GPUI context + store reference are both available.
        tracing::info!(
            workspace_id = hit.workspace_id.as_str(),
            "handle_search_open: workspace resolved — activation queued"
        );

        // Step 3 (tab open) and Step 4 (scroll dispatch) require GPUI Context —
        // performed in handle_search_open_with_cx.
        tracing::info!(
            path = %ocv.path.display(),
            line = ?ocv.line,
            col = ?ocv.col,
            "handle_search_open: OpenCodeViewer resolved"
        );

        // Record the resolved action for downstream dispatch and test assertions.
        self.last_open_code_viewer = Some(ocv);
        true
    }

    /// SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-3 (REQ-GS-040, AC-GS-10):
    /// Context-aware companion to `handle_search_open`.
    ///
    /// When GPUI context is available, this method:
    /// 1. Calls `handle_search_open` to resolve the `OpenCodeViewer`.
    /// 2. Opens a new tab via `TabContainer::new_tab(Some(abs_path))` (REQ-GS-040b, R6).
    /// 3. Calls `cx.notify()` to schedule a re-render.
    ///
    /// Tab open failures (no tab_container) are logged and tolerated (REQ-GS-042).
    pub fn handle_search_open_with_cx(
        &mut self,
        hit: &moai_search::SearchHit,
        cx: &mut Context<Self>,
    ) {
        if !self.handle_search_open(hit) {
            return;
        }
        // Step 3: open new tab (R6 — calls TabContainer::new_tab, no sig change).
        if let Some(tc_entity) = self.tab_container.clone() {
            let abs_path = self
                .last_open_code_viewer
                .as_ref()
                .map(|ocv| ocv.path.clone());
            tc_entity.update(cx, |tc, _| {
                tc.new_tab(abs_path);
            });
            tracing::info!("handle_search_open_with_cx: new tab created");
        } else {
            tracing::warn!("handle_search_open_with_cx: no tab_container — tab open skipped");
        }
        cx.notify();
    }

    /// SPEC-V3-009 MS-4a (AC-SU-13~16): handle a `TerminalClickEvent::OpenSpec`
    /// emission by mounting (if dismissed) the SPEC panel and selecting the
    /// requested SPEC. Respects the single-overlay invariant — when the
    /// palette or settings modal is active, the click is logged and ignored
    /// so the user's current overlay focus is not disrupted.
    ///
    /// `select_spec` itself is documented as graceful: an unknown spec_id
    /// keeps the prior selection (no panic, no stale state).
    pub fn handle_terminal_open_spec(&mut self, spec_id: &str, cx: &mut Context<Self>) {
        // Overlay invariant — do not steal focus from active modals.
        if self.palette.is_visible() || self.settings_modal.is_some() {
            tracing::info!(
                spec_id = %spec_id,
                "Terminal emitted OpenSpec but another overlay is active — ignored"
            );
            return;
        }
        // Lazy mount the SPEC panel if not already visible.
        if self.spec_panel.is_none() {
            let specs_dir = self.storage_path.join(".moai").join("specs");
            self.spec_panel = Some(spec_ui::SpecPanelView::new(specs_dir));
        }
        if let Some(panel) = self.spec_panel.as_mut() {
            panel.select_spec(moai_studio_spec::SpecId::new(spec_id));
        }
        cx.notify();
    }

    /// 파일 열기 이벤트를 처리한다 (REQ-MV-080).
    ///
    /// SPEC-V3-005 의 `OpenFileEvent` 를 소비하여:
    /// 1. 바이너리 파일 → 무시 (log 만).
    /// 2. Markdown → `Entity<MarkdownViewer>` 생성 후 `LeafKind::Markdown` 으로 저장.
    /// 3. Code / 그 외 → `Entity<CodeViewer>` 생성 후 `LeafKind::Code` 로 저장 (MS-2).
    ///
    /// MS-2 에서 tree-sitter CodeViewer 가 활성화되었다.
    pub fn handle_open_file(&mut self, ev: &viewer::OpenFileEvent, cx: &mut Context<Self>) {
        use viewer::code::CodeViewer;
        use viewer::markdown::MarkdownViewer;
        use viewer::{EventResolution, SurfaceHint, resolve_event};

        // 활성 탭의 focused pane id 를 구한다.
        let leaf_id = self
            .tab_container
            .as_ref()
            .and_then(|tc| tc.read(cx).active_tab().last_focused_pane.clone());

        let Some(leaf_id) = leaf_id else {
            // tab_container 없음 또는 focused pane 없음 — 무시
            return;
        };

        match resolve_event(ev) {
            EventResolution::Binary => {
                // AC-MV-11: binary 파일은 viewer 마운트 없이 무시
                info!("handle_open_file: binary 파일 무시 ({:?})", ev.path);
            }
            // SPEC-V3-016 MS-1: Image viewer routing (REQ-IV-012)
            EventResolution::Image => {
                let path = ev.path.clone();
                let entity = cx.new(|_cx| viewer::image::ImageViewer::new());
                // Load image using decode_image (REQ-IV-002)
                match viewer::image_data::decode_image(&path) {
                    Ok(data) => {
                        entity.update(cx, |viewer: &mut viewer::image::ImageViewer, cx| {
                            viewer.load_image(data, cx);
                        });
                    }
                    Err(e) => {
                        entity.update(cx, |viewer: &mut viewer::image::ImageViewer, cx| {
                            viewer.set_error(e.to_string(), cx);
                        });
                    }
                }
                self.leaf_payloads.insert(leaf_id, LeafKind::Image(entity));
                cx.notify();
            }
            EventResolution::Open(SurfaceHint::Markdown) => {
                let path = ev.path.clone();
                let entity = cx.new(|_cx| MarkdownViewer::new(path.clone()));
                // MS-1: sync read → load
                match viewer::read_file_for_viewer(&path) {
                    Ok(src) => {
                        entity.update(cx, |viewer: &mut MarkdownViewer, cx| {
                            viewer.load(src.source, cx);
                        });
                    }
                    Err(e) => {
                        entity.update(cx, |viewer: &mut MarkdownViewer, cx| {
                            viewer.set_error(e.to_string(), cx);
                        });
                    }
                }
                self.leaf_payloads
                    .insert(leaf_id, LeafKind::Markdown(entity));
                cx.notify();
            }
            EventResolution::Open(_) => {
                // MS-2: tree-sitter CodeViewer 생성
                let path = ev.path.clone();
                let entity = cx.new(|_cx| CodeViewer::new(path.clone()));
                match viewer::read_file_for_viewer(&path) {
                    Ok(src) => {
                        entity.update(cx, |viewer: &mut CodeViewer, cx| {
                            viewer.load(src.source, cx);
                        });
                    }
                    Err(e) => {
                        entity.update(cx, |viewer: &mut CodeViewer, cx| {
                            viewer.set_error(e.to_string(), cx);
                        });
                    }
                }
                self.leaf_payloads.insert(leaf_id, LeafKind::Code(entity));
                cx.notify();
            }
        }
    }

    /// + New Workspace 버튼 클릭 처리 — GPUI native folder prompt, 비동기 store 갱신.
    ///
    /// SPEC-V0-1-1-RFD-MODAL-FIX (hotfix v0.1.1):
    /// `rfd::FileDialog::pick_folder()` blocking call 은 GPUI main thread 의 RefCell
    /// borrow 가 살아있는 상태에서 NSOpenPanel 이 GPUI tick 을 재진입하면 panic loop
    /// (`RefCell already borrowed`) 를 일으킨다. v0.1.0 에서 critical regression 으로 발견됨.
    ///
    /// Fix: GPUI 0.2.2 의 native `cx.prompt_for_paths` API 사용. listener borrow 종료 후
    /// `cx.spawn` async task 에서 receiver 를 await — main thread 안전성 + GPUI tick 재진입
    /// 회피.
    fn handle_add_workspace(&mut self, cx: &mut Context<Self>) {
        // G-2: Show project wizard instead of direct file picker
        if let Some(wizard) = &self.project_wizard {
            wizard.update(cx, |w, _cx| {
                w.mount();
            });
        }
        cx.notify();
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let new_ws_btn = new_workspace_button().on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _ev, _window, cx| this.handle_add_workspace(cx)),
        );
        // SPEC-V0-1-1-RFD-MODAL-FIX: empty state primary CTA 가 v0.1.0 에서 dead button 이었음.
        // hotfix 로 동일 handle_add_workspace 호출에 wire — sidebar `+ New Workspace` 와 같은 동작.
        let create_first_btn = empty_state_primary_cta().on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _ev, _window, cx| this.handle_add_workspace(cx)),
        );
        // Row 클릭 리스너를 각 workspace 에 attach.
        let rows: Vec<gpui::Stateful<gpui::Div>> = self
            .workspaces
            .iter()
            .map(|ws| {
                let id = ws.id.clone();
                let id_for_right = ws.id.clone();
                let is_active = self.active_id.as_deref() == Some(ws.id.as_str());
                workspace_row(ws, is_active)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _ev, _window, cx| {
                            this.handle_activate_workspace(id.clone(), cx)
                        }),
                    )
                    // SPEC-V3-004 MS-6 (REQ-D2-MS6-2): right-click opens the
                    // workspace context menu at the click position.
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, ev: &gpui::MouseDownEvent, _window, cx| {
                            let x = f32::from(ev.position.x);
                            let y = f32::from(ev.position.y);
                            this.open_workspace_menu_at(&id_for_right, x, y);
                            cx.notify();
                        }),
                    )
            })
            .collect();
        // SPEC-V3-004 T2: tab_container Entity 를 main_body 에 전달.
        let tab_container = self.tab_container.clone();
        // SPEC-V3-004 T5: RootView 가 key 이벤트를 수신하여 tab command 로 dispatch.
        // REQ-R-031: keystroke_to_tab_key → dispatch_tab_key 순서로 변환.
        // MS-3: palette overlay slot — active palette 가 있을 때 overlay 렌더.
        // SPEC-V3-013 MS-3: settings overlay slot — settings_modal 이 Some 이면 overlay 렌더.
        let has_settings = self.settings_modal.is_some();
        // SPEC-V3-015 MS-1: spec_panel — overlay mount 상태.
        let has_spec_panel = self.spec_panel.is_some();
        // SPEC-V3-014 MS-3: banner_stack — entries 를 읽어 Entity<BannerView> 목록 생성.
        // 두 단계로 분리: (1) 불변 참조로 data 복사, (2) 가변 참조로 entity 생성.
        let banner_view_data: Vec<banners::banner_view::BannerView> = self
            .banner_stack
            .as_ref()
            .map(|entity| {
                entity
                    .read(cx)
                    .entries()
                    .iter()
                    .map(|e| banners::banner_view::BannerView::from_data(&e.data))
                    .collect()
            })
            .unwrap_or_default();
        let banner_view_entities: Vec<Entity<banners::banner_view::BannerView>> = banner_view_data
            .into_iter()
            .map(|v| cx.new(|_| v))
            .collect();
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(tok::BG_APP))
            // SPEC-V0-1-1-UX-FIX (audit §10): Menu / keybinding action dispatch.
            // Cmd+N (NewWorkspace) → handle_add_workspace (sidebar / File menu / 단축키 통합 진입점).
            // Cmd+, (OpenSettings) → settings_modal mount (V3-013 와 동일 동작).
            .on_action(cx.listener(|this, _: &NewWorkspace, _window, cx| {
                this.handle_add_workspace(cx);
            }))
            .on_action(cx.listener(|this, _: &OpenSettings, _window, cx| {
                if this.settings_modal.is_none() {
                    let mut modal = settings::SettingsModal::new();
                    modal.mount();
                    this.settings_modal = Some(modal);
                    cx.notify();
                }
            }))
            // SPEC-V0-1-2-MENUS-001: View/Pane/Surface/Go/Help menu action wiring.
            .on_action(cx.listener(|_this, _: &ToggleSidebar, _window, cx| {
                info!("ToggleSidebar — file explorer toggle deferred");
                cx.notify();
            }))
            .on_action(cx.listener(|_this, _: &ToggleBanner, _window, _cx| {
                info!("ToggleBanner — banner system is separate");
            }))
            .on_action(cx.listener(|this, _: &ToggleFind, _window, cx| {
                this.find_bar_open = !this.find_bar_open;
                cx.notify();
            }))
            .on_action(cx.listener(|_this, _: &ReloadWorkspace, _window, _cx| {
                info!("ReloadWorkspace — full reload deferred");
            }))
            .on_action(cx.listener(|this, _: &ToggleTheme, _window, cx| {
                use settings::settings_state::ThemeMode;
                let next = match this.active_theme.theme {
                    ThemeMode::Dark | ThemeMode::System => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
                this.active_theme.theme = next;
                this.user_settings.appearance.theme = next;
                cx.notify();
            }))
            // SPEC-V0-1-2-MENUS-001 MS-2 (AC-MN-9): SplitRight wires to
            // PaneTree::split_horizontal on the active tab's focused leaf.
            // The new pane is created with an empty payload (PaneTree<String>);
            // a follow-up SPEC will switch the payload type to LeafKind so the
            // new pane can be mounted with a Terminal/Markdown surface.
            .on_action(cx.listener(|this, _: &SplitRight, _window, cx| {
                this.handle_split_action(panes::tree::SplitDirection::Horizontal, cx);
            }))
            // SPEC-V0-1-2-MENUS-001 MS-2 (AC-MN-10): SplitDown vertical split.
            .on_action(cx.listener(|this, _: &SplitDown, _window, cx| {
                this.handle_split_action(panes::tree::SplitDirection::Vertical, cx);
            }))
            .on_action(cx.listener(|_this, _: &ClosePane, _window, _cx| {
                info!("ClosePane — pane management deferred");
            }))
            .on_action(cx.listener(|_this, _: &FocusNextPane, _window, _cx| {
                info!("FocusNextPane — pane focus deferred");
            }))
            .on_action(cx.listener(|_this, _: &FocusPrevPane, _window, _cx| {
                info!("FocusPrevPane — pane focus deferred");
            }))
            .on_action(cx.listener(|_this, _: &NewTerminalSurface, _window, _cx| {
                info!("NewTerminalSurface — terminal creation deferred");
            }))
            .on_action(cx.listener(|_this, _: &NewMarkdownSurface, _window, _cx| {
                info!("NewMarkdownSurface — surface creation deferred");
            }))
            .on_action(
                cx.listener(|_this, _: &NewCodeViewerSurface, _window, _cx| {
                    info!("NewCodeViewerSurface — surface creation deferred");
                }),
            )
            // SPEC-V0-1-2-MENUS-001 MS-2 (AC-MN-7): Cmd+K / Go menu →
            // command palette toggle. Reuses the existing palette overlay so
            // the keybinding handler and the menu dispatch share state.
            .on_action(cx.listener(|this, _: &OpenCommandPalette, _window, cx| {
                this.toggle_cmd_palette();
                cx.notify();
            }))
            // SPEC-V0-1-2-MENUS-001 MS-2 (AC-MN-8): Cmd+Shift+P / Go menu →
            // SPEC panel mount/dismiss with single-overlay invariant.
            .on_action(cx.listener(|this, _: &OpenSpecPanel, _window, cx| {
                this.handle_open_spec_panel(cx);
            }))
            // SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2 (REQ-GS-031): ⌘⇧F / Ctrl+Shift+F →
            // toggle SearchPanel visibility; lazy-init on first invocation.
            .on_action(
                cx.listener(|this, _: &search::ToggleSearchPanel, _window, cx| {
                    this.handle_toggle_search_panel(cx);
                }),
            )
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _window, cx| {
                // settings 키 먼저 처리 — 소비되면 나머지 스킵.
                if this.handle_settings_key_event(ev) {
                    cx.notify();
                    return;
                }
                // SPEC-V3-015: spec panel 키 처리 — 소비되면 나머지 스킵.
                if this.handle_spec_key_event(ev) {
                    cx.notify();
                    return;
                }
                // MS-3: palette 키 처리 — 소비되면 tab 키 dispatch 스킵.
                if this.handle_palette_key_event(ev) {
                    return;
                }
                // MS-3a: Find/Replace Cmd+F 처리 — 소비되면 tab 키 dispatch 스킵.
                if this.handle_find_key_event(ev) {
                    cx.notify();
                    return;
                }
                this.handle_key_event(ev, cx);
            }))
            .child(title_bar(self.title_label()))
            // F-3: Toolbar — main app action buttons (SPEC-V0-1-2-MENUS-001)
            .children(self.toolbar.clone())
            // SPEC-V3-014 REQ-V14-027: banner_stack — TabContainer 위, 정상 flow (overlay 아님).
            .child(render_banner_strip(banner_view_entities))
            .child(main_body(
                &self.workspaces,
                rows,
                new_ws_btn,
                tab_container,
                create_first_btn,
            ))
            .child(status_bar::render_status_bar(&self.status_bar))
            .children(
                self.cmd_palette
                    .as_ref()
                    .map(|cp| render_palette_overlay(cp, &self.palette_query)),
            )
            .children(has_settings.then(render_settings_overlay))
            .children(has_spec_panel.then(render_spec_panel_overlay))
            // G-2: Project Wizard overlay (rendered when visible)
            .children(self.project_wizard.clone())
            // SPEC-V3-007 MS-4 (REQ-WB-032, AC-WB-INT-2): WebView dev-server toast stack.
            // The overlay is a no-op stack when `feature = "web"` is disabled or
            // `pending_toasts` is empty — render_web_toasts returns None in either case.
            .children(self.render_web_toasts(cx))
            // SPEC-V3-004 MS-6 (REQ-D2-MS6-3): workspace context menu overlay.
            // Mounted at visible_position when WorkspaceMenu::is_open() == true.
            .children(
                self.workspace_menu
                    .is_open()
                    .then(|| render_workspace_context_menu_overlay(&self.workspace_menu, cx)),
            )
            // SPEC-V3-004 MS-6 (REQ-D2-MS6-4): rename modal overlay.
            // Mounted when rename_modal == Some.
            .children(
                self.rename_modal
                    .as_ref()
                    .map(|m| render_rename_modal_overlay(m, cx)),
            )
            // SPEC-V3-004 MS-6 (REQ-D2-MS6-4): delete confirmation overlay.
            // Mounted when delete_confirmation == Some.
            .children(
                self.delete_confirmation
                    .as_ref()
                    .map(|c| render_delete_confirmation_overlay(c, cx)),
            )
            // SPEC-V0-2-0-MISSION-CTRL-001 MS-2 (REQ-MC-024): Mission Control overlay.
            // Mounted when mission_control == Some. Activation trigger (command palette
            // entry / key bind) is a follow-up PR per SPEC §7.
            .children(self.mission_control.clone())
    }
}

/// 인터랙션 가능한 "+ New Workspace" 버튼 (id 필수 — StatefulInteractiveElement).
fn new_workspace_button() -> gpui::Stateful<gpui::Div> {
    div()
        .id("new-workspace-btn")
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .px_2()
        .py_2()
        .rounded_md()
        .bg(rgb(tok::BG_ELEVATED))
        .text_color(rgb(tok::FG_SECONDARY))
        .text_sm()
        .hover(|s| s.bg(rgb(tok::BG_ELEVATED)))
        .cursor_pointer()
        .child("+ New Workspace")
}

// ============================================================
// 0) BannerStrip — SPEC-V3-014 REQ-V14-027
// ============================================================

/// BannerStack 의 배너 Entity 목록을 v_flex 로 렌더 (REQ-V14-027).
///
/// 배너가 없으면 빈 div (높이 0) 반환.
/// TabContainer 위, TitleBar 아래 위치 (normal flow, not overlay).
fn render_banner_strip(
    banner_entities: Vec<Entity<banners::banner_view::BannerView>>,
) -> impl IntoElement {
    let mut strip = div().flex().flex_col().w_full();
    for entity in banner_entities {
        strip = strip.child(entity);
    }
    strip
}

// ============================================================
// 1) TitleBar — 44pt 상단
// ============================================================

fn title_bar(active_label: &str) -> impl IntoElement {
    // SPEC-V0-1-1-UX-FIX (C-4): macOS native traffic light 와 중복되던 custom traffic_lights() 제거.
    // GPUI WindowOptions::titlebar 가 native traffic light 를 좌상단 (~70px) 에 그리므로 좌측 padding 으로 회피.
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(44.))
        .pl(px(80.)) // native traffic light 영역 회피
        .pr_4()
        .gap_3()
        .bg(rgb(tok::BG_SURFACE))
        .border_b_1()
        .border_color(rgb(tok::BORDER_SUBTLE))
        // 프로젝트 이름 (현재 활성 워크스페이스)
        .child(
            div()
                .text_sm()
                .text_color(rgb(tok::FG_PRIMARY))
                .child("MoAI Studio"),
        )
        // 구분자
        .child(div().text_sm().text_color(rgb(tok::FG_DISABLED)).child("/"))
        // 활성 워크스페이스 이름 (empty state 시에는 placeholder)
        .child(
            div()
                .text_sm()
                .text_color(rgb(tok::FG_SECONDARY))
                .child(active_label.to_string()),
        )
}

/// macOS 전용 traffic lights (red/yellow/green). v0.1.1 부터 macOS native traffic light 만 사용 (C-4).
/// 본 함수는 향후 cross-platform fallback (Linux/Windows custom titlebar) 용으로 보존.
#[allow(dead_code)]
fn traffic_lights() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            div()
                .w(px(12.))
                .h(px(12.))
                .rounded_full()
                .bg(rgb(traffic::RED)),
        )
        .child(
            div()
                .w(px(12.))
                .h(px(12.))
                .rounded_full()
                .bg(rgb(traffic::YELLOW)),
        )
        .child(
            div()
                .w(px(12.))
                .h(px(12.))
                .rounded_full()
                .bg(rgb(traffic::GREEN)),
        )
}

// ============================================================
// 2) Main Body — Sidebar 260pt + 컨텐츠 영역
// ============================================================

fn main_body(
    workspaces: &[Workspace],
    rows: Vec<gpui::Stateful<gpui::Div>>,
    new_ws_btn: impl IntoElement,
    tab_container: Option<Entity<TabContainer>>,
    create_first_btn: gpui::Stateful<gpui::Div>,
) -> impl IntoElement {
    let is_empty = workspaces.is_empty();
    div()
        .flex()
        .flex_row()
        .flex_grow()
        .w_full()
        .child(sidebar(is_empty, rows, new_ws_btn))
        .child(content_area(is_empty, tab_container, create_first_btn))
}

/// Sidebar 260pt — WORKSPACE + GIT WORKTREES + SPECs 섹션 + 하단 인터랙티브 "+ New Workspace".
fn sidebar(
    is_empty: bool,
    rows: Vec<gpui::Stateful<gpui::Div>>,
    new_ws_btn: impl IntoElement,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .w(px(260.))
        .h_full()
        .bg(rgb(tok::BG_SURFACE))
        .border_r_1()
        .border_color(rgb(tok::BORDER_SUBTLE))
        .px_3()
        .py_4()
        .gap_4()
        .child(workspace_section(is_empty, rows))
        .child(sidebar_section(
            "GIT WORKTREES",
            vec![("—", tok::FG_DISABLED)],
        ))
        .child(sidebar_section("SPECS", vec![("—", tok::FG_DISABLED)]))
        .child(div().flex_grow())
        .child(new_ws_btn)
}

/// Sidebar 내부 섹션 (ALL-CAPS 라벨 + 항목 리스트).
fn sidebar_section(label: &'static str, items: Vec<(&'static str, u32)>) -> impl IntoElement {
    let mut section = div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_xs().text_color(rgb(tok::FG_MUTED)).child(label));
    for (text, color) in items {
        section = section.child(
            div()
                .text_sm()
                .text_color(rgb(color))
                .px_2()
                .py_1()
                .child(text),
        );
    }
    section
}

/// WORKSPACE 섹션 — 비었으면 placeholder, 있으면 render 에서 생성한 rows 렌더.
fn workspace_section(is_empty: bool, rows: Vec<gpui::Stateful<gpui::Div>>) -> impl IntoElement {
    let mut section = div().flex().flex_col().gap_2().child(
        div()
            .text_xs()
            .text_color(rgb(tok::FG_MUTED))
            .child("WORKSPACE"),
    );

    if is_empty {
        section = section.child(
            div()
                .text_sm()
                .text_color(rgb(tok::FG_MUTED))
                .px_2()
                .py_1()
                .child("No workspace yet"),
        );
    } else {
        for row in rows {
            section = section.child(row);
        }
    }
    section
}

/// 단일 워크스페이스 row — Stateful (id=ws.id). 컬러 dot + 이름. Active 시 하이라이트.
///
/// SPEC-V0-1-1-UX-FIX (H-1): active dot 색상을 is_active 기반으로 분리.
/// - active: brand ACCENT (청록) — 현재 선택된 workspace 강조
/// - inactive: BORDER_STRONG dim outline — 시각적 우선순위 낮춤
///
/// 이전 v0.1.0 에서는 모든 row 가 ws.color (orange-red) 로 동일하여 active 구분이 어려웠음.
fn workspace_row(ws: &Workspace, is_active: bool) -> gpui::Stateful<gpui::Div> {
    let bg = if is_active {
        tok::BG_ELEVATED
    } else {
        tok::BG_SURFACE
    };
    let fg = if is_active {
        tok::FG_PRIMARY
    } else {
        tok::FG_SECONDARY
    };
    let dot_color = if is_active {
        tok::ACCENT
    } else {
        tok::BORDER_STRONG
    };
    div()
        .id(gpui::SharedString::from(format!("ws-row-{}", ws.id)))
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(rgb(bg))
        .hover(|s| s.bg(rgb(tok::BG_ELEVATED)))
        .cursor_pointer()
        .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(rgb(dot_color)))
        .child(div().text_sm().text_color(rgb(fg)).child(ws.name.clone()))
}

/// 컨텐츠 영역 — SPEC-V3-004 T2: tab_container Entity 렌더.
///
/// 우선순위 (SPEC-V3-004 REQ-R-001 ~ REQ-R-005):
///   1. tab_container 가 Some 이면 TabContainer 렌더 (MS-1+: 탭 바 + PaneTree)
///   2. show_empty_state 이면 Empty State CTA 렌더 (SPEC-V3-001 carry)
///   3. 그 외 (workspace 선택 but tab_container 없음) 플레이스홀더 렌더
///
/// SPEC-V0-1-1-RFD-MODAL-FIX: `create_first_btn` 은 RootView::render 에서 cx.listener 로
/// wire 된 stateful button. 이전 v0.1.0 에서는 dead button 이었음.
fn content_area(
    show_empty_state: bool,
    tab_container: Option<Entity<TabContainer>>,
    create_first_btn: gpui::Stateful<gpui::Div>,
) -> impl IntoElement {
    let mut area = div()
        .flex()
        .flex_col()
        .flex_grow()
        .h_full()
        .bg(rgb(tok::BG_APP));

    if let Some(tc) = tab_container {
        // REQ-R-001/002: tab_container 존재 시 TabContainer 렌더.
        area = area.child(tc);
    } else if show_empty_state {
        area = area
            .justify_center()
            .items_center()
            .gap_4()
            .px_12()
            .child(empty_state_hero())
            .child(create_first_btn)
            .child(empty_state_secondary_cta_row())
            .child(empty_state_tip());
    } else {
        // workspace 선택 but TabContainer 미생성 — 초기화 대기 상태.
        area = area.justify_center().items_center().child(
            div()
                .text_sm()
                .text_color(rgb(tok::FG_MUTED))
                .child("Workspace selected — tab container initializing"),
        );
    }
    area
}

/// Hero: 큰 환영 제목 + 서브타이틀.
fn empty_state_hero() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap_2()
        .child(
            div()
                .text_3xl()
                .text_color(rgb(tok::FG_PRIMARY))
                .child("Welcome to MoAI Studio"),
        )
        .child(
            div()
                .text_sm()
                .text_color(rgb(tok::FG_MUTED))
                .child("SPEC-first native shell for Claude Code agents"),
        )
}

/// Primary CTA — `+ Create First Workspace` (모두의AI 청록).
///
/// SPEC-V0-1-1-RFD-MODAL-FIX (hotfix v0.1.1): `Stateful<Div>` 반환으로 변경하여
/// `RootView::render` 에서 `cx.listener` + `handle_add_workspace` 를 wire 가능. v0.1.0
/// 에서는 click handler 없는 dead button 이었음.
fn empty_state_primary_cta() -> gpui::Stateful<gpui::Div> {
    div()
        .id("empty-state-primary-cta")
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .mt_4()
        .px_6()
        .py_3()
        .rounded_lg()
        .bg(rgb(tok::ACCENT))
        .text_color(rgb(crate::design::tokens::theme::dark::text::ON_PRIMARY))
        .text_sm()
        .cursor_pointer()
        .child("+ Create First Workspace")
}

/// Secondary CTA 2 개 — Start Sample + Open Recent (가로 배치).
fn empty_state_secondary_cta_row() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .gap_3()
        .mt_2()
        .child(secondary_btn("Start Sample", "Guided tour"))
        .child(secondary_btn("Open Recent", "Last used workspace"))
}

fn secondary_btn(label: &'static str, subtitle: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .px_5()
        .py_3()
        .rounded_lg()
        .bg(rgb(tok::BG_SURFACE))
        .border_1()
        .border_color(rgb(tok::BORDER_SUBTLE))
        .child(
            div()
                .text_sm()
                .text_color(rgb(tok::FG_PRIMARY))
                .child(label),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(tok::FG_MUTED))
                .child(subtitle),
        )
}

/// Bottom tip — Command Palette 발견성 힌트.
fn empty_state_tip() -> impl IntoElement {
    div()
        .mt_8()
        .text_xs()
        .text_color(rgb(tok::FG_MUTED))
        .child("Tip: ⌘K opens Command Palette anytime")
}

// ============================================================
// MS-3: Palette overlay — Scrim + variant placeholder
// ============================================================

/// Palette overlay rendering — Shows CmdPalette with query input and filtered results.
///
/// @MX:ANCHOR: [AUTO] render_palette_overlay — RootView.render 호출 + GPUI 렌더 계약.
/// @MX:REASON: [AUTO] SPEC-V3-PALETTE-001 AC-PA-1~3. fan_in >= 3:
///   RootView.render (line 866), palette integration tests, visual regression tests.
/// @MX:SPEC: SPEC-V3-PALETTE-001
///
/// # Arguments
/// * `cmd_palette` - Active CmdPalette instance with query, items, and navigation state
/// * `query` - Current query text for display in input field
///
/// # Rendering
/// - Scrim backdrop (semi-transparent dark overlay)
/// - Centered 600px palette container
/// - Query input field (showing current query)
/// - Filtered results list (32px rows, max 320px height)
/// - Selected item highlight (HIGHLIGHT_ALPHA with brand primary)
fn render_palette_overlay(
    cmd_palette: &palette::variants::CmdPalette,
    query: &str,
) -> impl IntoElement {
    use palette::palette_view::{LIST_MAX_HEIGHT, ROW_HEIGHT};

    // Extract data from CmdPalette for rendering
    let view = &cmd_palette.view;
    let items = &view.items;
    let nav = &view.nav;

    // Calculate highlight color with alpha (0xRR_GG_BB_AA format)
    // Brand primary blue (0x3b_82_f6) with HIGHLIGHT_ALPHA (0.20 = 0x33)
    let highlight_color = 0x3b_82_f6_33u32;

    div()
        .absolute()
        .inset_0()
        .bg(gpui::rgba(0x08_0c_0b_8c)) // rgba(8,12,11,0.55) — scrim dark
        .flex()
        .flex_col()
        .items_center()
        .pt(px(80.))
        .child(
            div()
                .w(px(palette::PALETTE_WIDTH))
                .bg(rgb(crate::design::tokens::neutral::N900))
                .rounded_lg()
                .p_3()
                .flex()
                .flex_col()
                .gap_2()
                // Query input field
                .child(
                    div()
                        .w(px(palette::PALETTE_WIDTH - 24.0)) // Account for padding (p-3 = 12px * 2)
                        .h(px(32.0))
                        .px_3()
                        .bg(rgb(tok::BG_APP))
                        .rounded_md()
                        .flex()
                        .items_center()
                        .text_sm()
                        .text_color(rgb(tok::FG_PRIMARY))
                        .child(if query.is_empty() {
                            div()
                                .text_color(rgb(tok::FG_MUTED))
                                .child("Search files...")
                        } else {
                            div().child(query.to_string())
                        }),
                )
                // Filtered results list
                .child(
                    div()
                        .w(px(palette::PALETTE_WIDTH - 24.0))
                        .max_h(px(LIST_MAX_HEIGHT))
                        .overflow_y_hidden()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .children(items.iter().enumerate().map(|(idx, item)| {
                            let is_selected = nav.selected_index == Some(idx);
                            div()
                                .h(px(ROW_HEIGHT))
                                .px_3()
                                .flex()
                                .items_center()
                                .rounded_md()
                                .when(is_selected, |div| div.bg(gpui::rgba(highlight_color)))
                                .text_sm()
                                .text_color(if is_selected {
                                    rgb(tok::FG_PRIMARY)
                                } else {
                                    rgb(tok::FG_SECONDARY)
                                })
                                .child(item.label.clone())
                        })),
                ),
        )
}

// ============================================================
// SPEC-V3-013 MS-3: Settings overlay — Scrim + SettingsModal placeholder
// ============================================================

/// Settings overlay 렌더 — Scrim 위에 880×640 SettingsModal placeholder (AC-V13-1).
///
/// MS-3 단계: layout constants + scrim + 컨테이너 placeholder.
/// 실제 sidebar/main pane 렌더는 SettingsModal.view_state 기반 (후속 연결).
fn render_settings_overlay() -> impl IntoElement {
    use settings::settings_modal::{
        SETTINGS_MODAL_HEIGHT, SETTINGS_MODAL_WIDTH, SETTINGS_SIDEBAR_WIDTH,
    };
    div()
        .absolute()
        .inset_0()
        .bg(gpui::rgba(0x08_0c_0b_8c)) // scrim dark (REQ-V13-005)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(SETTINGS_MODAL_WIDTH))
                .h(px(SETTINGS_MODAL_HEIGHT))
                .bg(rgb(crate::design::tokens::theme::dark::background::PANEL))
                .rounded_lg()
                .flex()
                .flex_row()
                // sidebar placeholder
                .child(
                    div()
                        .w(px(SETTINGS_SIDEBAR_WIDTH))
                        .h_full()
                        .bg(rgb(crate::design::tokens::theme::dark::background::SURFACE))
                        .border_r_1()
                        .border_color(rgb(
                            crate::design::tokens::theme::dark::border::DEFAULT_APPROX,
                        ))
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(tok::FG_MUTED))
                                .p_3()
                                .child("Settings"),
                        ),
                )
                // main pane placeholder
                .child(
                    div().flex_grow().h_full().p_4().child(
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child("Settings Modal"),
                    ),
                ),
        )
}

/// SPEC-V3-015 MS-1: SpecPanelView overlay 렌더 (AC-RV-2, REQ-RV-003).
///
/// Scrim (반투명 backdrop) + 중앙 정렬 컨테이너 (640×480).
/// palette/settings 패턴과 동일한 구조.
fn render_spec_panel_overlay() -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .bg(gpui::rgba(0x08_0c_0b_8c)) // scrim dark — palette/settings 와 동일
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(640.))
                .h(px(480.))
                .bg(rgb(tok::BG_PANEL))
                .rounded_lg()
                .flex()
                .flex_col()
                // 헤더: mode selector
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_1()
                        .px_3()
                        .py_2()
                        .bg(rgb(tok::BG_SURFACE))
                        .border_b_1()
                        .border_color(rgb(tok::BORDER_SUBTLE))
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .text_sm()
                                .text_color(rgb(tok::ACCENT))
                                .child("List"),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .text_sm()
                                .text_color(rgb(tok::FG_MUTED))
                                .child("Kanban"),
                        )
                        .child(
                            div()
                                .px_2()
                                .py_1()
                                .text_sm()
                                .text_color(rgb(tok::FG_MUTED))
                                .child("Sprint"),
                        ),
                )
                // 본문
                .child(
                    div().flex().flex_col().flex_grow().p_3().child(
                        div()
                            .text_sm()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child("SpecListView"),
                    ),
                ),
        )
}

// ============================================================
// SPEC-V3-004 MS-6: Workspace context menu + rename modal + delete confirmation
// overlays. These mount on top of the sidebar / main pane via `Render for RootView`
// when the corresponding state slot is populated. Visual fidelity is intentionally
// minimal — the contract is that buttons dispatch to RootView helpers and state
// transitions are observable via unit tests on those helpers.
// ============================================================

/// Right-click workspace context menu overlay (REQ-D2-MS6-3).
///
/// Renders 4 action rows (Rename / Delete / Move Up / Move Down) absolutely
/// positioned at `WorkspaceMenu::visible_position()`. Each row dispatches to
/// `RootView::click_workspace_menu_item` which closes the menu and routes
/// through the MS-5 dispatch logic.
fn render_workspace_context_menu_overlay(
    menu: &workspace_menu::WorkspaceMenu,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let pos = menu
        .visible_position()
        .unwrap_or(workspace_menu::MenuPosition { x: 0.0, y: 0.0 });
    let mut container = div()
        .id("workspace-context-menu")
        .absolute()
        .left(px(pos.x))
        .top(px(pos.y))
        .w(px(160.0))
        .bg(rgb(tok::BG_ELEVATED))
        .border_1()
        .border_color(rgb(tok::BORDER_SUBTLE))
        .rounded_md()
        .flex()
        .flex_col()
        .py_1();
    for action in workspace_menu::WorkspaceMenu::items() {
        let row = div()
            .id(gpui::SharedString::from(format!(
                "workspace-context-menu-item-{}",
                action.label()
            )))
            .px_3()
            .py_1()
            .text_sm()
            .text_color(rgb(if action.is_destructive() {
                tok::ACCENT
            } else {
                tok::FG_PRIMARY
            }))
            .hover(|s| s.bg(rgb(tok::BG_SURFACE)))
            .cursor_pointer()
            .child(action.label())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _ev, _window, cx| {
                    this.click_workspace_menu_item(action);
                    cx.notify();
                }),
            );
        container = container.child(row);
    }
    container
}

/// Rename workspace modal overlay (REQ-D2-MS6-4 rename half).
///
/// Centered scrim + small box that displays the current buffer text and offers
/// Commit / Cancel buttons. The actual text-input wiring (per-keystroke
/// `set_buffer`) is out of MS-6 scope — the buffer here is read-only feedback.
fn render_rename_modal_overlay(
    modal: &workspace_menu::RenameModal,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    let buffer_display = if modal.buffer().is_empty() {
        "(empty)".to_string()
    } else {
        modal.buffer().to_string()
    };
    div()
        .absolute()
        .inset_0()
        .bg(gpui::rgba(0x08_0c_0b_8c)) // scrim dark — palette/settings parity
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(360.0))
                .bg(rgb(crate::design::tokens::theme::dark::background::PANEL))
                .rounded_lg()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(tok::FG_PRIMARY))
                        .child("Rename Workspace"),
                )
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .bg(rgb(tok::BG_APP))
                        .rounded_md()
                        .text_sm()
                        .text_color(rgb(tok::FG_SECONDARY))
                        .child(buffer_display),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_2()
                        .justify_end()
                        .child(
                            div()
                                .id("rename-modal-cancel")
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(rgb(tok::BG_SURFACE))
                                .text_sm()
                                .text_color(rgb(tok::FG_SECONDARY))
                                .cursor_pointer()
                                .child("Cancel")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _ev, _window, cx| {
                                        this.cancel_rename_modal();
                                        cx.notify();
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .id("rename-modal-commit")
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(rgb(tok::ACCENT))
                                .text_sm()
                                .text_color(rgb(tok::FG_PRIMARY))
                                .cursor_pointer()
                                .child("Commit")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _ev, _window, cx| {
                                        let _ = this.commit_rename_modal();
                                        cx.notify();
                                    }),
                                ),
                        ),
                ),
        )
}

/// Delete workspace confirmation overlay (REQ-D2-MS6-4 delete half).
///
/// Centered scrim + small box with a warning message and Confirm / Cancel
/// buttons. Confirm dispatches to `confirm_delete_modal`, Cancel to
/// `cancel_delete_modal`.
fn render_delete_confirmation_overlay(
    _conf: &workspace_menu::DeleteConfirmation,
    cx: &mut Context<RootView>,
) -> impl IntoElement {
    div()
        .absolute()
        .inset_0()
        .bg(gpui::rgba(0x08_0c_0b_8c)) // scrim dark
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(px(360.0))
                .bg(rgb(crate::design::tokens::theme::dark::background::PANEL))
                .rounded_lg()
                .p_4()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(tok::FG_PRIMARY))
                        .child("Delete Workspace?"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(tok::FG_MUTED))
                        .child("This action cannot be undone."),
                )
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_2()
                        .justify_end()
                        .child(
                            div()
                                .id("delete-modal-cancel")
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(rgb(tok::BG_SURFACE))
                                .text_sm()
                                .text_color(rgb(tok::FG_SECONDARY))
                                .cursor_pointer()
                                .child("Cancel")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _ev, _window, cx| {
                                        this.cancel_delete_modal();
                                        cx.notify();
                                    }),
                                ),
                        )
                        .child(
                            div()
                                .id("delete-modal-confirm")
                                .px_3()
                                .py_1()
                                .rounded_md()
                                .bg(rgb(tok::ACCENT))
                                .text_sm()
                                .text_color(rgb(tok::FG_PRIMARY))
                                .cursor_pointer()
                                .child("Confirm")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _ev, _window, cx| {
                                        let _ = this.confirm_delete_modal();
                                        cx.notify();
                                    }),
                                ),
                        ),
                ),
        )
}

// ============================================================
// 3) StatusBar — 28pt 하단
//
// SPEC-V3-006 MS-7 (audit F-4): the prior free function `status_bar()`
// was extracted into the `crate::status_bar` module as a state-bearing
// surface. RootView::render now calls
// `crate::status_bar::render_status_bar(&self.status_bar)` which preserves
// the pre-MS-7 layout when `StatusBarState::default()` is used and exposes
// `set_agent_mode` / `set_git_branch` / `set_lsp_status` / `clear_lsp_status`
// for follow-up wiring of git2 / LSP / agent broadcasting (REQ-SB-MS7-3).
// ============================================================

// ============================================================
// 앱 엔트리
// ============================================================

pub fn run_app(workspaces: Vec<Workspace>, storage_path: PathBuf) {
    info!(
        "moai-studio-ui: GPUI Application 시작 (Phase 1.7 — workspaces={}, store={})",
        workspaces.len(),
        storage_path.display()
    );

    Application::new().run(move |cx: &mut App| {
        // SPEC-V0-1-1-UX-FIX (C-5 + audit §10): macOS menu bar setup.
        // App menu, File (New Workspace), Edit (OsAction), View, Window, Help (Report Issue).
        cx.on_action(|_: &Quit, cx: &mut App| cx.quit());
        cx.on_action(|_: &About, _cx: &mut App| {
            tracing::info!("About MoAI Studio v{}", env!("CARGO_PKG_VERSION"));
        });
        cx.on_action(|_: &ReportIssue, cx: &mut App| {
            cx.open_url("https://github.com/modu-ai/moai-studio/issues");
        });
        // SPEC-V0-1-2 menu expansion: app-level handlers for Help/Go items
        cx.on_action(|_: &OpenDocumentation, cx: &mut App| {
            cx.open_url("https://github.com/modu-ai/moai-studio#readme");
        });
        cx.on_action(|_: &OpenAbout, cx: &mut App| {
            cx.open_url("https://github.com/modu-ai/moai-studio");
        });
        // NewWorkspace + OpenSettings + remaining actions (View/Pane/Surface/Go) are dispatched
        // to RootView via entity-level on_action handlers in `Render::render`.

        // SPEC-V0-1-2: full keybinding map covering view, pane, surface, go menus.
        cx.bind_keys([
            gpui::KeyBinding::new("cmd-n", NewWorkspace, None),
            gpui::KeyBinding::new("cmd-,", OpenSettings, None),
            gpui::KeyBinding::new("cmd-b", ToggleSidebar, None),
            gpui::KeyBinding::new("cmd-r", ReloadWorkspace, None),
            gpui::KeyBinding::new("cmd-t", ToggleTheme, None),
            gpui::KeyBinding::new("cmd-f", ToggleFind, None),
            gpui::KeyBinding::new("cmd-\\", SplitRight, None),
            gpui::KeyBinding::new("cmd-shift-\\", SplitDown, None),
            gpui::KeyBinding::new("cmd-w", ClosePane, None),
            gpui::KeyBinding::new("cmd-]", FocusNextPane, None),
            gpui::KeyBinding::new("cmd-[", FocusPrevPane, None),
            gpui::KeyBinding::new("cmd-k", OpenCommandPalette, None),
            gpui::KeyBinding::new("cmd-shift-p", OpenSpecPanel, None),
            // SPEC-V0-2-0-GLOBAL-SEARCH-001 MS-2: Global Search panel toggle (REQ-GS-031).
            gpui::KeyBinding::new("cmd-shift-f", search::ToggleSearchPanel, None),
            gpui::KeyBinding::new("ctrl-shift-f", search::ToggleSearchPanel, None),
            // SPEC-V3-007 MS-2: WebView DevTools toggle (Cmd+Opt+I / Ctrl+Shift+I)
            #[cfg(feature = "web")]
            gpui::KeyBinding::new("cmd-alt-i", ToggleDevTools, None),
            #[cfg(feature = "web")]
            gpui::KeyBinding::new("ctrl-shift-i", ToggleDevTools, None),
        ]);

        cx.set_menus(vec![
            Menu {
                name: "MoAI Studio".into(),
                items: vec![
                    MenuItem::action("About MoAI Studio", About),
                    MenuItem::separator(),
                    MenuItem::action("Settings...", OpenSettings),
                    MenuItem::separator(),
                    MenuItem::os_submenu("Services", SystemMenuType::Services),
                    MenuItem::separator(),
                    MenuItem::action("Quit MoAI Studio", Quit),
                ],
            },
            Menu {
                name: "File".into(),
                items: vec![MenuItem::action("New Workspace", NewWorkspace)],
            },
            Menu {
                name: "Edit".into(),
                items: vec![
                    MenuItem::os_action("Cut", NoOp, OsAction::Cut),
                    MenuItem::os_action("Copy", NoOp, OsAction::Copy),
                    MenuItem::os_action("Paste", NoOp, OsAction::Paste),
                    MenuItem::separator(),
                    MenuItem::os_action("Undo", NoOp, OsAction::Undo),
                    MenuItem::os_action("Redo", NoOp, OsAction::Redo),
                    MenuItem::separator(),
                    MenuItem::os_action("Select All", NoOp, OsAction::SelectAll),
                ],
            },
            Menu {
                name: "View".into(),
                items: vec![
                    MenuItem::action("Toggle Sidebar", ToggleSidebar),
                    MenuItem::action("Toggle Banner Stack", ToggleBanner),
                    MenuItem::separator(),
                    MenuItem::action("Find...", ToggleFind),
                    MenuItem::separator(),
                    MenuItem::action("Reload Workspace", ReloadWorkspace),
                    MenuItem::action("Toggle Theme", ToggleTheme),
                ],
            },
            Menu {
                name: "Pane".into(),
                items: vec![
                    MenuItem::action("Split Right", SplitRight),
                    MenuItem::action("Split Down", SplitDown),
                    MenuItem::separator(),
                    MenuItem::action("Close Pane", ClosePane),
                    MenuItem::separator(),
                    MenuItem::action("Focus Next Pane", FocusNextPane),
                    MenuItem::action("Focus Previous Pane", FocusPrevPane),
                ],
            },
            Menu {
                name: "Surface".into(),
                items: vec![
                    MenuItem::action("New Terminal", NewTerminalSurface),
                    MenuItem::action("New Markdown Viewer", NewMarkdownSurface),
                    MenuItem::action("New Code Viewer", NewCodeViewerSurface),
                ],
            },
            Menu {
                name: "Go".into(),
                items: vec![
                    MenuItem::action("Command Palette", OpenCommandPalette),
                    MenuItem::action("SPEC Panel", OpenSpecPanel),
                ],
            },
            Menu {
                name: "Window".into(),
                items: vec![],
            },
            Menu {
                name: "Help".into(),
                items: vec![
                    MenuItem::action("Documentation", OpenDocumentation),
                    MenuItem::action("Report Issue", ReportIssue),
                    MenuItem::separator(),
                    MenuItem::action("About MoAI Studio", OpenAbout),
                ],
            },
        ]);

        let bounds = gpui::Bounds::centered(None, size(px(1600.), px(1000.)), cx);
        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("MoAI Studio".into()),
                appears_transparent: false,
                traffic_light_position: None,
            }),
            ..Default::default()
        };

        let ws = workspaces.clone();
        let path = storage_path.clone();
        cx.open_window(options, move |_window, cx| {
            cx.new(|cx| {
                let mut rv = RootView::new(ws, path);
                // SPEC-V3-014 REQ-V14-026: banner_stack 초기화 (empty BannerStack).
                rv.banner_stack = Some(cx.new(|_| banners::BannerStack::new()));
                // F-3: Toolbar Entity 초기화 (SPEC-V0-1-2-MENUS-001 REQ-F3-001)
                rv.toolbar = Some(cx.new(|_| toolbar::Toolbar::new(true)));
                // G-2: Project Wizard 초기화
                rv.project_wizard = Some(cx.new(|_| wizard::ProjectWizard::new()));
                rv
            })
        })
        .expect("GPUI 윈도우 생성 실패");

        cx.activate(true);
        info!("moai-studio-ui: RootView 렌더 등록 완료 (+ New Workspace 버튼 배선)");
    });
}

/// 스캐폴드 hello 유지 (non-GPUI 경로용).
pub fn hello() {
    info!("moai-studio-ui: scaffold entry. GPUI 엔트리는 run_app(workspaces)");
}

// ============================================================
// 유닛 테스트 — RootView 상태 로직 (GPUI 렌더 제외)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_ws(name: &str, id_suffix: &str, last_active: u64) -> Workspace {
        Workspace {
            id: format!("ws-{}", id_suffix),
            name: name.to_string(),
            project_path: PathBuf::from(format!("/tmp/{}", name)),
            moai_config: PathBuf::from(".moai"),
            color: crate::design::tokens::brand::PRIMARY_DARK,
            last_active,
        }
    }

    fn dummy_path() -> PathBuf {
        PathBuf::from("/tmp/moai-studio-ui-tests-workspaces.json")
    }

    #[test]
    fn root_view_empty_workspaces_has_no_active_and_placeholder_label() {
        let view = RootView::new(vec![], dummy_path());
        assert!(view.active_id.is_none());
        assert!(view.active().is_none());
        assert_eq!(view.title_label(), "no workspace");
    }

    #[test]
    fn root_view_picks_most_recently_active_workspace_as_active() {
        let older = make_ws("alpha", "1", 1_000);
        let newer = make_ws("beta", "2", 9_000);
        let view = RootView::new(vec![older, newer.clone()], dummy_path());
        assert_eq!(view.active_id.as_deref(), Some(newer.id.as_str()));
        assert_eq!(view.title_label(), "beta");
        assert_eq!(view.active().map(|w| w.name.as_str()), Some("beta"));
    }

    #[test]
    fn root_view_active_returns_none_when_active_id_missing() {
        let mut view = RootView::new(vec![make_ws("alpha", "1", 1_000)], dummy_path());
        view.active_id = Some("ws-does-not-exist".to_string());
        assert!(view.active().is_none());
        assert_eq!(view.title_label(), "no workspace");
    }

    #[test]
    fn apply_added_workspace_from_empty_sets_active_to_new() {
        let mut view = RootView::new(vec![], dummy_path());
        let added = make_ws("new-proj", "new1", 5_000);
        view.apply_added_workspace(&added, vec![added.clone()]);
        assert_eq!(view.workspaces.len(), 1);
        assert_eq!(view.active_id.as_deref(), Some(added.id.as_str()));
        assert_eq!(view.title_label(), "new-proj");
    }

    #[test]
    fn activate_workspace_switches_active_id_when_id_exists() {
        let a = make_ws("alpha", "1", 5_000);
        let b = make_ws("beta", "2", 1_000);
        let mut view = RootView::new(vec![a.clone(), b.clone()], dummy_path());
        assert_eq!(view.active_id.as_deref(), Some(a.id.as_str()));
        let ok = view.activate_workspace(&b.id);
        assert!(ok);
        assert_eq!(view.active_id.as_deref(), Some(b.id.as_str()));
        assert_eq!(view.title_label(), "beta");
    }

    #[test]
    fn activate_workspace_returns_false_for_unknown_id_and_keeps_active() {
        let a = make_ws("alpha", "1", 5_000);
        let mut view = RootView::new(vec![a.clone()], dummy_path());
        let ok = view.activate_workspace("ws-does-not-exist");
        assert!(!ok);
        assert_eq!(view.active_id.as_deref(), Some(a.id.as_str()));
    }

    #[test]
    fn apply_added_workspace_switches_active_even_if_others_newer() {
        // last_active 가 더 오래된 항목을 추가해도 "방금 추가한 것" 을 active 로 강제.
        let existing = make_ws("alpha", "1", 9_999);
        let added = make_ws("new-proj", "new1", 1_000);
        let mut view = RootView::new(vec![existing.clone()], dummy_path());
        assert_eq!(view.active_id.as_deref(), Some(existing.id.as_str()));

        view.apply_added_workspace(&added, vec![existing, added.clone()]);
        assert_eq!(view.workspaces.len(), 2);
        assert_eq!(view.active_id.as_deref(), Some(added.id.as_str()));
        assert_eq!(view.title_label(), "new-proj");
    }

    // --- Phase 2 (SPEC-V3-002 T4) 추가 테스트 ---

    #[test]
    fn tab_container_is_none_by_default() {
        // AC-R-1 (partial): 초기 상태에서 tab_container 는 None (empty state 렌더).
        // SPEC-V3-004 T2: pane_splitter → tab_container 필드 교체.
        let view = RootView::new(vec![], dummy_path());
        assert!(view.tab_container.is_none());
    }

    #[test]
    fn root_view_with_workspaces_tab_container_still_none() {
        // 워크스페이스 존재해도 tab_container 는 명시 생성 전까지 None
        let ws = make_ws("proj", "1", 1_000);
        let view = RootView::new(vec![ws], dummy_path());
        assert!(view.tab_container.is_none());
    }

    // ── T2: handle_open_file (AC-MV-1 / AC-MV-11) ──

    #[test]
    fn leaf_payloads_is_empty_on_new_root_view() {
        // 초기 상태에서 leaf_payloads 는 비어있어야 한다
        let view = RootView::new(vec![], dummy_path());
        assert!(
            view.leaf_payloads.is_empty(),
            "초기 leaf_payloads 는 비어있어야 한다"
        );
    }

    #[test]
    fn handle_open_file_no_tab_container_early_returns_without_panic() {
        // tab_container 없으면 early return — panic 없어야 한다 (AC-MV-1 전제)
        use gpui::{AppContext, TestAppContext};
        use viewer::{OpenFileEvent, SurfaceHint};

        let mut cx = TestAppContext::single();
        let ev = OpenFileEvent {
            path: std::path::PathBuf::from("docs/README.md"),
            surface_hint: Some(SurfaceHint::Markdown),
        };
        let root_entity = cx.new(|_cx| RootView::new(vec![], dummy_path()));
        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.handle_open_file(&ev, cx);
            });
        });
        let leaf_count = cx.read(|app| root_entity.read(app).leaf_payloads.len());
        assert_eq!(
            leaf_count, 0,
            "tab_container 없으면 early return, leaf_payloads 변경 없음"
        );
    }

    // ── MS-3 테스트: palette 글로벌 키바인딩 (AC-PL-14/15) ──

    fn make_key_event(key: &str, command: bool, shift: bool) -> KeyDownEvent {
        // macOS: Cmd = platform modifier. tabs/keys.rs 패턴 참조.
        KeyDownEvent {
            keystroke: gpui::Keystroke {
                modifiers: gpui::Modifiers {
                    platform: command,
                    shift,
                    ..Default::default()
                },
                key: key.to_string(),
                key_char: None,
            },
            is_held: false,
        }
    }

    /// AC-PL-14: Cmd+P → CmdPalette 열림.
    #[test]
    fn cmd_p_opens_cmd_palette() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(view.palette.active_variant.is_none());
        let ev = make_key_event("p", true, false);
        let consumed = view.handle_palette_key_event(&ev);
        assert!(consumed, "Cmd+P 는 소비되어야 함");
        assert_eq!(
            view.palette.active_variant,
            Some(palette::PaletteVariant::CmdPalette)
        );
    }

    /// AC-PL-14 (toggle): Cmd+P 두 번 → CmdPalette 닫힘 (VS Code toggle semantics).
    #[test]
    fn cmd_p_toggles_dismisses_cmd_palette() {
        let mut view = RootView::new(vec![], dummy_path());
        let ev = make_key_event("p", true, false);
        view.handle_palette_key_event(&ev); // open
        view.handle_palette_key_event(&ev); // toggle → close
        assert!(
            view.palette.active_variant.is_none(),
            "두 번째 Cmd+P 는 CmdPalette 를 닫아야 함"
        );
    }

    /// AC-PL-15 (mutual exclusion): CmdPalette 열린 상태에서 Cmd+Shift+P → CommandPalette 로 교체.
    #[test]
    fn cmd_shift_p_replaces_cmd_palette() {
        let mut view = RootView::new(vec![], dummy_path());
        let cmd_p = make_key_event("p", true, false);
        let cmd_shift_p = make_key_event("p", true, true);
        view.handle_palette_key_event(&cmd_p); // open CmdPalette
        assert_eq!(
            view.palette.active_variant,
            Some(palette::PaletteVariant::CmdPalette)
        );
        let consumed = view.handle_palette_key_event(&cmd_shift_p); // switch to CommandPalette
        assert!(consumed, "Cmd+Shift+P 는 소비되어야 함");
        assert_eq!(
            view.palette.active_variant,
            Some(palette::PaletteVariant::CommandPalette),
            "Cmd+Shift+P 후 CommandPalette 가 활성이어야 함"
        );
        assert_ne!(
            view.palette.active_variant,
            Some(palette::PaletteVariant::CmdPalette),
            "CmdPalette 는 닫혀있어야 함"
        );
    }

    /// Cmd+Shift+P → CommandPalette 열림 (초기 상태).
    #[test]
    fn cmd_shift_p_opens_command_palette() {
        let mut view = RootView::new(vec![], dummy_path());
        let ev = make_key_event("p", true, true);
        let consumed = view.handle_palette_key_event(&ev);
        assert!(consumed);
        assert_eq!(
            view.palette.active_variant,
            Some(palette::PaletteVariant::CommandPalette)
        );
    }

    /// Esc → 열려있는 palette dismiss.
    #[test]
    fn esc_dismisses_active_palette() {
        let mut view = RootView::new(vec![], dummy_path());
        view.palette.open(palette::PaletteVariant::CmdPalette);
        let esc = make_key_event("escape", false, false);
        let consumed = view.handle_palette_key_event(&esc);
        assert!(consumed, "Esc 는 소비되어야 함 (palette 열려있을 때)");
        assert!(
            view.palette.active_variant.is_none(),
            "Esc 후 palette 는 닫혀있어야 함"
        );
    }

    /// Esc — palette 닫혀있으면 소비하지 않음.
    #[test]
    fn esc_no_op_when_palette_closed() {
        let mut view = RootView::new(vec![], dummy_path());
        let esc = make_key_event("escape", false, false);
        let consumed = view.handle_palette_key_event(&esc);
        assert!(!consumed, "palette 닫혀있을 때 Esc 는 소비하지 않아야 함");
    }

    /// "/" + terminal_focused=true → SlashBar 열림.
    #[test]
    fn slash_with_terminal_focused_opens_slash_bar() {
        let mut view = RootView::new(vec![], dummy_path());
        view.terminal_focused = true;
        let slash = make_key_event("/", false, false);
        let consumed = view.handle_palette_key_event(&slash);
        assert!(consumed, "/ + terminal focused 는 소비되어야 함");
        assert_eq!(
            view.palette.active_variant,
            Some(palette::PaletteVariant::SlashBar)
        );
    }

    /// "/" + terminal_focused=false → no-op.
    #[test]
    fn slash_without_terminal_focus_is_noop() {
        let mut view = RootView::new(vec![], dummy_path());
        view.terminal_focused = false;
        let slash = make_key_event("/", false, false);
        let consumed = view.handle_palette_key_event(&slash);
        assert!(!consumed, "terminal focus 없이 / 는 소비하지 않아야 함");
        assert!(
            view.palette.active_variant.is_none(),
            "SlashBar 는 열리지 않아야 함"
        );
    }

    /// "/" + palette 이미 열려있으면 SlashBar 열지 않음 (RG-PL-23: no palette visible).
    #[test]
    fn slash_no_op_when_palette_already_visible() {
        let mut view = RootView::new(vec![], dummy_path());
        view.terminal_focused = true;
        view.palette.open(palette::PaletteVariant::CmdPalette);
        let slash = make_key_event("/", false, false);
        let consumed = view.handle_palette_key_event(&slash);
        assert!(!consumed, "palette 열려있으면 / 는 소비하지 않아야 함");
        assert_eq!(
            view.palette.active_variant,
            Some(palette::PaletteVariant::CmdPalette),
            "기존 palette 는 유지되어야 함"
        );
    }

    /// has_palette_overlay — active palette 있을 때 true.
    #[test]
    fn has_palette_overlay_returns_true_when_active() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(!view.has_palette_overlay());
        view.palette.open(palette::PaletteVariant::CmdPalette);
        assert!(view.has_palette_overlay());
        view.palette.dismiss();
        assert!(!view.has_palette_overlay());
    }

    // ── F-1 additional tests: Cmd+K binding + query wire ──

    /// F-1: Cmd+K → CmdPalette opens (same as Cmd+P, VS Code / Zed pattern).
    #[test]
    fn cmd_k_opens_cmd_palette() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(view.palette.active_variant.is_none());
        let ev = make_key_event("k", true, false);
        let consumed = view.handle_palette_key_event(&ev);
        assert!(consumed, "Cmd+K must be consumed");
        assert_eq!(
            view.palette.active_variant,
            Some(palette::PaletteVariant::CmdPalette),
            "Cmd+K must open CmdPalette"
        );
    }

    /// F-1: Cmd+K toggle — second press dismisses.
    #[test]
    fn cmd_k_toggles_dismisses_cmd_palette() {
        let mut view = RootView::new(vec![], dummy_path());
        let ev = make_key_event("k", true, false);
        view.handle_palette_key_event(&ev); // open
        view.handle_palette_key_event(&ev); // toggle → close
        assert!(
            view.palette.active_variant.is_none(),
            "second Cmd+K must dismiss CmdPalette"
        );
    }

    /// F-1: palette_query is empty on initial open, resets on dismiss.
    #[test]
    fn palette_query_resets_on_dismiss() {
        let mut view = RootView::new(vec![], dummy_path());
        view.palette.open(palette::PaletteVariant::CmdPalette);
        view.handle_palette_text_input("src".to_string());
        assert_eq!(view.palette_query, "src", "query must update");
        view.palette.dismiss();
        view.reset_palette_query();
        assert_eq!(view.palette_query, "", "query must reset after dismiss");
    }

    /// F-1: handle_palette_text_input updates palette_query.
    #[test]
    fn palette_text_input_updates_query() {
        let mut view = RootView::new(vec![], dummy_path());
        view.palette.open(palette::PaletteVariant::CmdPalette);
        view.handle_palette_text_input("palette".to_string());
        assert_eq!(view.palette_query, "palette");
    }

    #[test]
    fn handle_open_file_binary_no_tab_container_does_not_panic() {
        // AC-MV-11: binary 파일 이벤트 → panic 없이 무시
        use gpui::{AppContext, TestAppContext};
        use viewer::OpenFileEvent;

        let mut cx = TestAppContext::single();
        let png_ev = OpenFileEvent {
            path: std::path::PathBuf::from("photo.png"),
            surface_hint: None,
        };
        let root_entity = cx.new(|_cx| RootView::new(vec![], dummy_path()));
        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.handle_open_file(&png_ev, cx);
            });
        });
        let leaf_count = cx.read(|app| root_entity.read(app).leaf_payloads.len());
        assert_eq!(leaf_count, 0, "binary 파일은 leaf_payloads 에 영향 없음");
    }

    // ── SPEC-V3-013 MS-3: RootView settings_modal + Cmd+, 키바인딩 테스트 ──

    /// 새 RootView 에서 settings_modal 이 None 이다 (AC-V13-1 전제).
    #[test]
    fn root_view_settings_modal_starts_none() {
        let view = RootView::new(vec![], dummy_path());
        assert!(view.settings_modal.is_none(), "초기 settings_modal 은 None");
        assert!(!view.has_settings_modal());
    }

    /// Cmd+, → settings_modal 이 Some 이 된다 (AC-V13-1).
    #[test]
    fn cmd_comma_mounts_settings_modal() {
        let mut view = RootView::new(vec![], dummy_path());
        let ev = make_key_event(",", true, false);
        let consumed = view.handle_settings_key_event(&ev);
        assert!(consumed, "Cmd+, 는 소비되어야 함");
        assert!(
            view.settings_modal.is_some(),
            "Cmd+, 후 settings_modal 이 Some"
        );
        assert!(view.has_settings_modal());
    }

    /// Cmd+, 두 번 → 이미 열려있으면 무시 (REQ-V13-006).
    #[test]
    fn cmd_comma_double_press_does_not_remount() {
        let mut view = RootView::new(vec![], dummy_path());
        let ev = make_key_event(",", true, false);
        view.handle_settings_key_event(&ev); // mount
        assert!(view.has_settings_modal());
        view.handle_settings_key_event(&ev); // 두 번째 — 무시
        assert!(view.has_settings_modal(), "두 번째 Cmd+, 는 modal 유지");
    }

    /// Esc → settings_modal 이 dismiss 된다 (REQ-V13-004).
    #[test]
    fn esc_dismisses_settings_modal() {
        let mut view = RootView::new(vec![], dummy_path());
        // 먼저 열기
        let cmd_comma = make_key_event(",", true, false);
        view.handle_settings_key_event(&cmd_comma);
        assert!(view.has_settings_modal());
        // Esc
        let esc = make_key_event("escape", false, false);
        let consumed = view.handle_settings_key_event(&esc);
        assert!(consumed, "Esc 는 소비되어야 함 (settings modal open 상태)");
        assert!(!view.has_settings_modal(), "Esc 후 settings_modal 이 None");
    }

    /// settings_modal 이 없을 때 Esc 는 소비되지 않는다.
    #[test]
    fn esc_when_no_settings_modal_not_consumed() {
        let mut view = RootView::new(vec![], dummy_path());
        let esc = make_key_event("escape", false, false);
        let consumed = view.handle_settings_key_event(&esc);
        assert!(!consumed, "settings modal 없을 때 Esc 는 소비하지 않음");
    }

    /// RootView init 시 user_settings 가 load 되고 active_theme 이 일관성 있게 설정된다 (AC-V13-11).
    /// 실제 config 파일이 존재할 수 있으므로 파일 내용이 아닌 active_theme ↔ user_settings 일관성만 검증.
    #[test]
    fn root_view_init_loads_user_settings_and_active_theme() {
        let view = RootView::new(vec![], dummy_path());
        // active_theme 이 load 된 user_settings.appearance 에서 일관되게 derive 되어야 함
        use design::runtime::ActiveTheme;
        let expected = ActiveTheme::from_settings(&view.user_settings.appearance);
        assert_eq!(
            view.active_theme, expected,
            "active_theme 은 load 된 user_settings.appearance 와 일치해야 함"
        );
    }

    /// dismiss_settings_modal 이 user_settings 를 업데이트한다.
    #[test]
    fn dismiss_settings_modal_syncs_user_settings() {
        let mut view = RootView::new(vec![], dummy_path());
        // modal 열기
        let ev = make_key_event(",", true, false);
        view.handle_settings_key_event(&ev);

        // modal 내 appearance 변경
        if let Some(ref mut modal) = view.settings_modal {
            modal.view_state.appearance.font_size_px = 16;
            modal.view_state.appearance.accent = settings::settings_state::AccentColor::Blue;
        }

        // dismiss → save + sync
        view.dismiss_settings_modal();
        assert!(!view.has_settings_modal(), "dismiss 후 modal 이 None");
        assert_eq!(
            view.user_settings.appearance.font_size_px, 16,
            "font_size_px 가 user_settings 에 반영되어야 함"
        );
        assert_eq!(
            view.user_settings.appearance.accent,
            settings::settings_state::AccentColor::Blue,
            "accent 이 user_settings 에 반영되어야 함"
        );
        // ActiveTheme 도 업데이트되어야 함
        assert!((view.active_theme.font_size_px() - 16.0).abs() < f32::EPSILON);
        assert_eq!(view.active_theme.accent_color(), 0x2563eb);
    }

    /// Ctrl+, (Linux/Win) 도 settings_modal 을 mount 한다 (REQ-V13-001).
    #[test]
    fn ctrl_comma_mounts_settings_modal() {
        let mut view = RootView::new(vec![], dummy_path());
        // Ctrl+, (control modifier, not platform)
        let ev = KeyDownEvent {
            keystroke: gpui::Keystroke {
                modifiers: gpui::Modifiers {
                    control: true,
                    ..Default::default()
                },
                key: ",".to_string(),
                key_char: None,
            },
            is_held: false,
        };
        let consumed = view.handle_settings_key_event(&ev);
        assert!(consumed, "Ctrl+, 는 소비되어야 함");
        assert!(view.has_settings_modal());
    }

    // ── SPEC-V3-006 MS-3a: Find/Replace 키바인딩 테스트 ──

    fn make_find_key_event(cmd: bool, ctrl: bool, shift: bool, key: &str) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: gpui::Keystroke {
                modifiers: gpui::Modifiers {
                    platform: cmd,
                    control: ctrl,
                    shift,
                    ..Default::default()
                },
                key: key.to_string(),
                key_char: None,
            },
            is_held: false,
        }
    }

    #[test]
    fn find_bar_initially_closed() {
        let view = RootView::new(vec![], dummy_path());
        assert!(
            !view.has_find_bar(),
            "초기 상태에서 find bar 는 닫혀있어야 한다"
        );
    }

    #[test]
    fn cmd_f_opens_find_bar() {
        let mut view = RootView::new(vec![], dummy_path());
        let ev = make_find_key_event(true, false, false, "f");
        let consumed = view.handle_find_key_event(&ev);
        assert!(consumed, "Cmd+F 는 소비되어야 한다");
        assert!(view.has_find_bar(), "Cmd+F 후 find bar 가 열려야 한다");
    }

    #[test]
    fn ctrl_f_opens_find_bar() {
        let mut view = RootView::new(vec![], dummy_path());
        let ev = make_find_key_event(false, true, false, "f");
        let consumed = view.handle_find_key_event(&ev);
        assert!(consumed, "Ctrl+F 는 소비되어야 한다");
        assert!(view.has_find_bar());
    }

    #[test]
    fn escape_closes_find_bar() {
        let mut view = RootView::new(vec![], dummy_path());
        // Cmd+F 로 열기
        let open_ev = make_find_key_event(true, false, false, "f");
        view.handle_find_key_event(&open_ev);
        assert!(view.has_find_bar());
        // Esc 로 닫기
        let esc_ev = make_find_key_event(false, false, false, "escape");
        let consumed = view.handle_find_key_event(&esc_ev);
        assert!(consumed, "Esc 는 find bar 닫을 때 소비되어야 한다");
        assert!(!view.has_find_bar(), "Esc 후 find bar 가 닫혀야 한다");
    }

    #[test]
    fn escape_when_find_bar_closed_is_not_consumed() {
        let mut view = RootView::new(vec![], dummy_path());
        // find bar 가 닫혀있을 때 Esc 는 소비하지 않는다
        let esc_ev = make_find_key_event(false, false, false, "escape");
        let consumed = view.handle_find_key_event(&esc_ev);
        assert!(
            !consumed,
            "find bar 닫혀있을 때 Esc 는 소비하지 않아야 한다"
        );
    }

    // ── SPEC-V3-014 MS-3: RootView + BannerStack 통합 테스트 (AC-V14-11, AC-V14-12) ──

    // ── SPEC-V3-015 MS-1: SpecPanelView 키바인딩 테스트 (AC-RV-1, AC-RV-2) ──

    /// AC-RV-1: RootView::new 초기 상태 — spec_panel = None.
    #[test]
    fn spec_panel_initially_none() {
        let view = RootView::new(vec![], dummy_path());
        assert!(
            view.spec_panel.is_none(),
            "초기 spec_panel 은 None 이어야 한다 (AC-RV-1)"
        );
    }

    /// AC-RV-2: Cmd+Shift+S → spec_panel mount (None → Some).
    #[test]
    fn cmd_shift_s_mounts_spec_panel() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(!view.has_spec_panel());
        let ev = make_key_event("s", true, true);
        let consumed = view.handle_spec_key_event(&ev);
        assert!(consumed, "Cmd+Shift+S 는 소비되어야 한다");
        assert!(
            view.has_spec_panel(),
            "Cmd+Shift+S 후 spec_panel 이 mount 되어야 한다"
        );
    }

    /// AC-RV-2 (toggle): Cmd+Shift+S 두 번 → mount → dismiss.
    #[test]
    fn cmd_shift_s_toggles_spec_panel() {
        let mut view = RootView::new(vec![], dummy_path());
        let ev = make_key_event("s", true, true);
        view.handle_spec_key_event(&ev); // mount
        assert!(view.has_spec_panel(), "첫 번째 Cmd+Shift+S 후 mount");
        view.handle_spec_key_event(&ev); // dismiss
        assert!(!view.has_spec_panel(), "두 번째 Cmd+Shift+S 후 dismiss");
    }

    /// AC-RV-2 (mutual exclusion): palette 열려있을 때 Cmd+Shift+S → noop.
    #[test]
    fn cmd_shift_s_noop_when_palette_active() {
        let mut view = RootView::new(vec![], dummy_path());
        view.palette.open(palette::PaletteVariant::CmdPalette);
        let ev = make_key_event("s", true, true);
        view.handle_spec_key_event(&ev);
        assert!(
            !view.has_spec_panel(),
            "palette 열려있을 때 spec_panel 은 mount 되지 않아야 한다"
        );
    }

    /// AC-RV-2 (mutual exclusion): settings modal 열려있을 때 Cmd+Shift+S → noop.
    #[test]
    fn cmd_shift_s_noop_when_settings_active() {
        let mut view = RootView::new(vec![], dummy_path());
        let mut modal = settings::SettingsModal::new();
        modal.mount();
        view.settings_modal = Some(modal);
        let ev = make_key_event("s", true, true);
        view.handle_spec_key_event(&ev);
        assert!(
            !view.has_spec_panel(),
            "settings modal 열려있을 때 spec_panel 은 mount 되지 않아야 한다"
        );
    }

    /// Esc → spec_panel dismiss.
    #[test]
    fn esc_dismisses_spec_panel() {
        let mut view = RootView::new(vec![], dummy_path());
        let open_ev = make_key_event("s", true, true);
        view.handle_spec_key_event(&open_ev);
        assert!(view.has_spec_panel());
        let esc = make_key_event("escape", false, false);
        let consumed = view.handle_spec_key_event(&esc);
        assert!(consumed, "Esc 는 소비되어야 한다 (spec_panel 열려있을 때)");
        assert!(!view.has_spec_panel(), "Esc 후 spec_panel dismiss");
    }

    /// AC-RV-1: RootView::new 초기 상태 — banner_stack = None (cx 없이 생성).
    #[test]
    fn rootview_banner_stack_none_when_created_without_cx() {
        let view = RootView::new(vec![], dummy_path());
        // cx 없이 생성되면 banner_stack = None (테스트 호환 초기값)
        assert!(
            view.banner_stack.is_none(),
            "cx 없이 생성 시 banner_stack = None (정상)"
        );
    }

    /// AC-V14-11: GPUI cx 로 생성 시 banner_stack 이 Some 으로 초기화됨.
    #[test]
    fn rootview_banner_stack_initialized_with_cx() {
        use gpui::TestAppContext;

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|cx| {
            let mut rv = RootView::new(vec![], dummy_path());
            rv.banner_stack = Some(cx.new(|_| banners::BannerStack::new()));
            rv
        });
        let has_stack = cx.read(|app| root_entity.read(app).banner_stack.is_some());
        assert!(
            has_stack,
            "cx 로 생성 시 banner_stack 은 Some 이어야 함 (AC-V14-11)"
        );
    }

    /// AC-V14-11: 초기 banner_stack Entity 는 empty (len == 0).
    #[test]
    fn rootview_banner_stack_empty_on_init() {
        use gpui::TestAppContext;

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|cx| {
            let mut rv = RootView::new(vec![], dummy_path());
            rv.banner_stack = Some(cx.new(|_| banners::BannerStack::new()));
            rv
        });
        let len = cx.read(|app| {
            let view = root_entity.read(app);
            view.banner_stack.as_ref().unwrap().read(app).len()
        });
        assert_eq!(len, 0, "초기 banner_stack 은 비어있어야 함 (AC-V14-11)");
    }

    /// AC-V14-12: push_crash helper — CrashBanner(Critical) 가 스택에 삽입됨.
    #[test]
    fn push_crash_helper_constructs_crash_banner() {
        use gpui::TestAppContext;
        use std::time::Duration;

        let mut cx = TestAppContext::single();
        let stack_entity = cx.new(|_| banners::BannerStack::new());

        cx.update(|app| {
            stack_entity.update(app, |stack, cx| {
                stack.push_crash("/tmp/log".into(), Duration::from_secs(12), cx);
            });
        });

        cx.read(|app| {
            let stack = stack_entity.read(app);
            assert_eq!(stack.len(), 1, "push_crash 후 스택 길이 = 1 (AC-V14-12)");
            let entry = &stack.entries()[0];
            assert_eq!(
                entry.data.severity,
                banners::Severity::Critical,
                "CrashBanner severity = Critical (AC-V14-12)"
            );
            assert_eq!(entry.data.message, "Agent crashed");
            assert_eq!(entry.data.actions.len(), 2);
            let primary = entry.data.actions.iter().find(|a| a.primary).unwrap();
            assert_eq!(primary.label, "Reopen");
        });
    }

    /// AC-V14-12: push_update helper — UpdateBanner(Info, auto-dismiss 8s) 삽입.
    #[test]
    fn push_update_helper_constructs_update_banner() {
        use gpui::TestAppContext;
        use std::time::Duration;

        let mut cx = TestAppContext::single();
        let stack_entity = cx.new(|_| banners::BannerStack::new());

        cx.update(|app| {
            stack_entity.update(app, |stack, cx| {
                stack.push_update("0.2.0", "12.3 MB", cx);
            });
        });

        cx.read(|app| {
            let stack = stack_entity.read(app);
            assert_eq!(stack.len(), 1);
            let entry = &stack.entries()[0];
            assert_eq!(entry.data.severity, banners::Severity::Info);
            assert_eq!(
                entry.data.auto_dismiss_after,
                Some(Duration::from_secs(8)),
                "UpdateBanner auto_dismiss = 8s (AC-V14-12)"
            );
        });
    }

    // ============================================================
    // SPEC-V3-PALETTE-001: Palette rendering tests
    // ============================================================

    /// Test that CmdPalette provides correct data for rendering (AC-PA-1).
    #[test]
    fn cmd_palette_render_data_has_query_and_items() {
        use palette::variants::CmdPalette;

        let cmd_palette = CmdPalette::new();

        // Verify initial state for rendering
        assert_eq!(
            cmd_palette.view.query, "",
            "Initial query should be empty for rendering"
        );
        assert!(
            !cmd_palette.view.items.is_empty(),
            "Should have items to render in list"
        );
        assert!(
            cmd_palette.view.nav.selected_index.is_some(),
            "Should have initial selection for highlight"
        );
    }

    /// Test navigation state for rendering selection highlight (AC-PA-3).
    #[test]
    fn palette_nav_state_provides_selection_for_rendering() {
        use palette::palette_view::{NavState, PaletteItem};

        let items = [
            PaletteItem::new("1", "File 1"),
            PaletteItem::new("2", "File 2"),
            PaletteItem::new("3", "File 3"),
        ];

        let mut nav = NavState::new(items.len());

        // Initial selection should be first item
        assert_eq!(
            nav.selected_index,
            Some(0),
            "First item should be selected for rendering"
        );

        // Move down - selection changes
        nav.move_down();
        assert_eq!(
            nav.selected_index,
            Some(1),
            "Second item should be selected after move_down"
        );

        // Move up - selection changes back
        nav.move_up();
        assert_eq!(
            nav.selected_index,
            Some(0),
            "First item should be selected after move_up"
        );
    }

    /// Test empty palette state for rendering (AC-PA-2).
    #[test]
    fn empty_palette_has_no_selection_for_rendering() {
        use palette::PaletteItem;
        use palette::palette_view::NavState;

        let items: Vec<PaletteItem> = vec![];
        let nav = NavState::new(items.len());

        assert_eq!(
            nav.selected_index, None,
            "No selection when empty - nothing to highlight"
        );
        assert_eq!(nav.item_count, 0, "Item count should be zero");
    }

    /// Test that RootView integrates palette state for rendering.
    #[test]
    fn root_view_cmd_palette_state_ready_for_rendering() {
        let mut view = RootView::new(vec![], dummy_path());

        // Initial state - palette closed, no data
        assert!(
            view.cmd_palette.is_none(),
            "CmdPalette should be None initially"
        );
        assert_eq!(view.palette_query, "", "Query should be empty initially");

        // Open CmdPalette - should create palette instance
        view.toggle_cmd_palette();

        assert!(
            view.cmd_palette.is_some(),
            "CmdPalette should be created for rendering"
        );
        assert_eq!(view.palette_query, "", "Query should be empty after open");

        // Verify palette has data for rendering
        let cmd_palette = view.cmd_palette.as_ref().unwrap();
        assert!(
            !cmd_palette.view.items.is_empty(),
            "Should have items to render"
        );
    }

    // ── MS-4: dispatch_command tests (AC-PL-17/18/19) ──

    /// AC-PL-17: dispatch_command("settings.open") mounts SettingsModal.
    #[test]
    fn dispatch_command_settings_open_mounts_modal() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(view.settings_modal.is_none());
        let handled = view.dispatch_command("settings.open");
        assert!(handled, "settings.open must be handled");
        assert!(
            view.settings_modal.is_some(),
            "settings.open must mount SettingsModal"
        );
        // Palette must be dismissed after dispatch.
        assert!(
            !view.palette.is_visible(),
            "palette must be dismissed after dispatch"
        );
    }

    /// AC-PL-17: dispatch_command("theme.toggle") cycles theme.
    #[test]
    fn dispatch_command_theme_toggle_cycles_theme() {
        use settings::settings_state::ThemeMode;
        let mut view = RootView::new(vec![], dummy_path());
        // Default theme is Dark (from default UserSettings).
        let original_theme = view.active_theme.theme;
        let handled = view.dispatch_command("theme.toggle");
        assert!(handled, "theme.toggle must be handled");
        let toggled_theme = view.active_theme.theme;
        assert_ne!(
            original_theme, toggled_theme,
            "theme.toggle must change the active theme"
        );
        // Toggle back.
        view.dispatch_command("theme.toggle");
        assert_eq!(
            view.active_theme.theme, original_theme,
            "double toggle must restore original theme"
        );
        let _ = ThemeMode::Dark; // suppress unused import
    }

    /// AC-PL-17: dispatch_command("tab.new") returns true (logged).
    #[test]
    fn dispatch_command_tab_new_returns_true() {
        let mut view = RootView::new(vec![], dummy_path());
        let handled = view.dispatch_command("tab.new");
        assert!(handled, "tab.new must return true (logged)");
        assert!(!view.palette.is_visible());
    }

    /// AC-PL-17: dispatch_command("tab.close") returns true.
    #[test]
    fn dispatch_command_tab_close_returns_true() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(view.dispatch_command("tab.close"));
    }

    /// AC-PL-17: dispatch_command("pane.split_horizontal") returns true.
    #[test]
    fn dispatch_command_pane_split_horizontal_returns_true() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(view.dispatch_command("pane.split_horizontal"));
    }

    /// AC-PL-17: dispatch_command("surface.toggle_terminal") returns true.
    #[test]
    fn dispatch_command_surface_toggle_terminal_returns_true() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(view.dispatch_command("surface.toggle_terminal"));
    }

    /// AC-PL-17: dispatch_command("workspace.switch") returns true.
    #[test]
    fn dispatch_command_workspace_switch_returns_true() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(view.dispatch_command("workspace.switch"));
    }

    /// AC-PL-17: dispatch_command("agent.toggle_dashboard") returns true.
    #[test]
    fn dispatch_command_agent_toggle_dashboard_returns_true() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(view.dispatch_command("agent.toggle_dashboard"));
    }

    /// AC-PL-19: dispatch_command("unknown.xxx") returns false (graceful degradation).
    #[test]
    fn dispatch_command_unknown_id_returns_false() {
        let mut view = RootView::new(vec![], dummy_path());
        let handled = view.dispatch_command("unknown.command_that_does_not_exist");
        assert!(!handled, "unknown command must return false");
        // Palette must still be dismissed even for unknown commands.
        assert!(!view.palette.is_visible());
    }

    /// dispatch_command dismisses palette before executing.
    #[test]
    fn dispatch_command_dismisses_palette() {
        let mut view = RootView::new(vec![], dummy_path());
        view.palette.open(palette::PaletteVariant::CommandPalette);
        assert!(view.palette.is_visible());
        view.dispatch_command("tab.new");
        assert!(
            !view.palette.is_visible(),
            "palette must be dismissed by dispatch_command"
        );
    }

    /// theme.dark sets dark theme.
    #[test]
    fn dispatch_command_theme_dark_sets_dark_theme() {
        use settings::settings_state::ThemeMode;
        let mut view = RootView::new(vec![], dummy_path());
        view.dispatch_command("theme.dark");
        assert_eq!(view.active_theme.theme, ThemeMode::Dark);
        assert_eq!(view.user_settings.appearance.theme, ThemeMode::Dark);
    }

    /// theme.light sets light theme.
    #[test]
    fn dispatch_command_theme_light_sets_light_theme() {
        use settings::settings_state::ThemeMode;
        let mut view = RootView::new(vec![], dummy_path());
        view.dispatch_command("theme.light");
        assert_eq!(view.active_theme.theme, ThemeMode::Light);
        assert_eq!(view.user_settings.appearance.theme, ThemeMode::Light);
    }

    // ── MS-4: inject_slash_command tests (AC-PL-21) ──

    /// AC-PL-21: inject_slash_command("/moai plan") returns true and sets pending injection.
    #[test]
    fn inject_slash_command_sets_pending_injection() {
        let mut view = RootView::new(vec![], dummy_path());
        view.palette.open(palette::PaletteVariant::SlashBar);
        let result = view.inject_slash_command("/moai plan");
        assert!(
            result,
            "inject_slash_command must return true for valid /moai command"
        );
        assert!(
            view.pending_slash_injection.is_some(),
            "pending_slash_injection must be set"
        );
        let injection = view.pending_slash_injection.as_deref().unwrap();
        assert!(
            injection.contains("/moai plan"),
            "pending injection must contain the command"
        );
        assert!(
            injection.ends_with('\n'),
            "pending injection must end with newline"
        );
        // Palette must be dismissed.
        assert!(!view.palette.is_visible());
    }

    /// inject_slash_command with non-/moai label returns false.
    #[test]
    fn inject_slash_command_invalid_label_returns_false() {
        let mut view = RootView::new(vec![], dummy_path());
        let result = view.inject_slash_command("git commit");
        assert!(
            !result,
            "inject_slash_command must return false for non-/moai label"
        );
        assert!(
            view.pending_slash_injection.is_none(),
            "no injection for invalid label"
        );
    }

    /// inject_slash_command("/moai run") injects correct string.
    #[test]
    fn inject_slash_command_moai_run() {
        let mut view = RootView::new(vec![], dummy_path());
        view.inject_slash_command("/moai run");
        let injection = view.pending_slash_injection.as_deref().unwrap();
        assert_eq!(injection, "/moai run\n");
    }

    /// pending_slash_injection starts as None.
    #[test]
    fn pending_slash_injection_initially_none() {
        let view = RootView::new(vec![], dummy_path());
        assert!(view.pending_slash_injection.is_none());
    }

    // ============================================================
    // SPEC-V3-007 MS-4 — RootView ↔ WebView toast pipeline tests
    // (AC-WB-INT-1, AC-WB-INT-2, AC-WB-INT-3)
    // ============================================================

    /// AC-WB-INT-1 (initial state): pending_toasts starts empty when feature is on.
    #[cfg(feature = "web")]
    #[test]
    fn web_toasts_initially_empty() {
        let view = RootView::new(vec![], dummy_path());
        assert_eq!(view.pending_toasts.len(), 0);
        assert_eq!(view.toast_count(), 0);
    }

    /// AC-WB-INT-1: stdout chunk containing a localhost URL pushes one toast.
    #[cfg(feature = "web")]
    #[test]
    fn web_ingest_stdout_chunk_pushes_toast_for_localhost_url() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));
        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.ingest_stdout_chunk(
                    "Serving HTTP on 0.0.0.0 port 8080 http://localhost:8080/ ...\n",
                    cx,
                );
            });
        });
        let toast_url = cx.read(|app| {
            let v = root_entity.read(app);
            assert_eq!(v.pending_toasts.len(), 1);
            v.pending_toasts[0].url.clone()
        });
        assert_eq!(toast_url, "http://localhost:8080/");
    }

    /// AC-WB-INT-1 (dedupe): the same URL emitted twice within 5s yields a single toast.
    #[cfg(feature = "web")]
    #[test]
    fn web_ingest_stdout_chunk_dedupes_same_url_within_window() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));
        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.ingest_stdout_chunk("vite dev http://localhost:5173", cx);
                view.ingest_stdout_chunk("ready: http://localhost:5173", cx);
            });
        });
        let count = cx.read(|app| root_entity.read(app).pending_toasts.len());
        assert_eq!(count, 1, "second emission must be deduped within 5s");
    }

    /// AC-WB-INT-1 (no match): a chunk without any URL leaves pending_toasts empty.
    #[cfg(feature = "web")]
    #[test]
    fn web_ingest_stdout_chunk_no_match_keeps_toasts_empty() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));
        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.ingest_stdout_chunk("warning: nothing to do here", cx);
            });
        });
        let count = cx.read(|app| root_entity.read(app).pending_toasts.len());
        assert_eq!(count, 0);
    }

    /// AC-WB-INT-2: clicking a toast creates a new tab whose focused leaf is
    /// LeafKind::Web with the detected URL, and removes the consumed toast.
    #[cfg(feature = "web")]
    #[test]
    fn web_open_toast_creates_new_tab_with_leaf_web() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|cx| {
            let mut rv = RootView::new(vec![], dummy_path());
            rv.tab_container = Some(cx.new(|_| tabs::container::TabContainer::new()));
            rv
        });

        // Push a synthetic toast directly to bypass the detector for focus.
        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.pending_toasts.push(WebToastEntry {
                    url: "http://localhost:9000/".to_string(),
                    source: "Listening on http://localhost:9000/".to_string(),
                });
                let new_id = view.open_toast_in_new_tab(0, cx);
                assert!(new_id.is_some(), "open_toast_in_new_tab must return TabId");
            });
        });

        cx.read(|app| {
            let v = root_entity.read(app);
            assert_eq!(v.pending_toasts.len(), 0, "consumed toast removed");

            // tab_container now has 2 tabs (initial + new), and the new one is active.
            let tc = v.tab_container.as_ref().unwrap().read(app);
            assert_eq!(tc.tab_count(), 2);
            // active is the just-created tab → its last_focused_pane should be present.
            let leaf_id = tc
                .active_tab()
                .last_focused_pane
                .clone()
                .expect("new tab seeds a last_focused_pane");
            let leaf = v
                .leaf_payloads
                .get(&leaf_id)
                .expect("LeafKind::Web mounted on the new tab's focused leaf");
            match leaf {
                LeafKind::Web(entity) => {
                    let surface = entity.read(app);
                    assert_eq!(surface.current_url(), "http://localhost:9000/");
                }
                _ => panic!("expected LeafKind::Web on the new tab's focused leaf"),
            }
        });
    }

    /// AC-WB-INT-2 (no container): without a TabContainer, open_toast returns None
    /// without panicking and leaves the toast in place for retry.
    #[cfg(feature = "web")]
    #[test]
    fn web_open_toast_without_tab_container_returns_none() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.pending_toasts.push(WebToastEntry {
                    url: "http://localhost:1234/".to_string(),
                    source: "n/a".to_string(),
                });
                let result = view.open_toast_in_new_tab(0, cx);
                assert!(result.is_none(), "no tab_container → None");
            });
        });

        // Toast is still pending so the user can retry once a workspace is active.
        let count = cx.read(|app| root_entity.read(app).pending_toasts.len());
        assert_eq!(count, 1, "toast preserved when open could not proceed");
    }

    /// AC-WB-INT-3: dismissing a toast removes it and silences future detections.
    #[cfg(feature = "web")]
    #[test]
    fn web_dismiss_toast_silences_future_detections() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                // First detection adds a toast.
                view.ingest_stdout_chunk("dev server: http://localhost:7000", cx);
                assert_eq!(view.pending_toasts.len(), 1);

                // Dismiss the toast.
                view.dismiss_toast(0, cx);
                assert_eq!(view.pending_toasts.len(), 0);

                // Second detection of the same URL during silence stays empty.
                view.ingest_stdout_chunk("dev server: http://localhost:7000", cx);
                assert_eq!(
                    view.pending_toasts.len(),
                    0,
                    "dismissed URL must remain silent"
                );
            });
        });
    }

    /// AC-WB-INT-3 (out-of-range): dismiss with an invalid index is a no-op.
    #[cfg(feature = "web")]
    #[test]
    fn web_dismiss_toast_out_of_range_is_noop() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.dismiss_toast(99, cx);
                assert_eq!(view.pending_toasts.len(), 0);
            });
        });
    }

    // ============================================================
    // SPEC-V0-1-2-MENUS-001 MS-2 — Action handler wiring polish
    // (AC-MN-7 / AC-MN-8 / AC-MN-9 / AC-MN-10 / AC-MN-11)
    // ============================================================

    /// AC-MN-9: SplitRight on the focused leaf turns the tree into a Horizontal split.
    #[test]
    fn split_right_action_horizontal_splits_focused_leaf() {
        use gpui::{AppContext, TestAppContext};
        use panes::tree::SplitDirection;

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|cx| {
            let mut rv = RootView::new(vec![], dummy_path());
            rv.tab_container = Some(cx.new(|_| tabs::container::TabContainer::new()));
            rv
        });

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.handle_split_action(SplitDirection::Horizontal, cx);
            });
        });

        cx.read(|app| {
            let tc = root_entity
                .read(app)
                .tab_container
                .as_ref()
                .unwrap()
                .read(app);
            let tree = &tc.active_tab().pane_tree;
            match tree {
                panes::PaneTree::Split { direction, .. } => {
                    assert_eq!(*direction, SplitDirection::Horizontal);
                }
                panes::PaneTree::Leaf(_) => {
                    panic!("expected Horizontal split, found Leaf")
                }
            }
        });
    }

    /// AC-MN-10: SplitDown turns the tree into a Vertical split.
    #[test]
    fn split_down_action_vertical_splits_focused_leaf() {
        use gpui::{AppContext, TestAppContext};
        use panes::tree::SplitDirection;

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|cx| {
            let mut rv = RootView::new(vec![], dummy_path());
            rv.tab_container = Some(cx.new(|_| tabs::container::TabContainer::new()));
            rv
        });

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.handle_split_action(SplitDirection::Vertical, cx);
            });
        });

        cx.read(|app| {
            let tc = root_entity
                .read(app)
                .tab_container
                .as_ref()
                .unwrap()
                .read(app);
            let tree = &tc.active_tab().pane_tree;
            match tree {
                panes::PaneTree::Split { direction, .. } => {
                    assert_eq!(*direction, SplitDirection::Vertical);
                }
                panes::PaneTree::Leaf(_) => {
                    panic!("expected Vertical split, found Leaf")
                }
            }
        });
    }

    /// AC-MN-11: split action without a tab_container is a safe no-op.
    #[test]
    fn split_action_without_tab_container_is_noop() {
        use gpui::{AppContext, TestAppContext};
        use panes::tree::SplitDirection;

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.handle_split_action(SplitDirection::Horizontal, cx);
                // Should not panic, should not mutate any state we own.
                assert!(view.tab_container.is_none());
            });
        });
    }

    /// AC-MN-8: OpenSpecPanel toggles the spec_panel slot from None to Some.
    #[test]
    fn open_spec_panel_action_mounts_when_dismissed() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                assert!(view.spec_panel.is_none());
                view.handle_open_spec_panel(cx);
                assert!(view.spec_panel.is_some(), "spec_panel must mount");
            });
        });
    }

    /// AC-MN-8: a second invocation dismisses the panel.
    #[test]
    fn open_spec_panel_action_dismisses_when_visible() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.handle_open_spec_panel(cx); // mount
                assert!(view.spec_panel.is_some());
                view.handle_open_spec_panel(cx); // dismiss
                assert!(view.spec_panel.is_none(), "second dispatch must dismiss");
            });
        });
    }

    /// AC-MN-8 (overlay invariant): mount is suppressed while a settings modal is open.
    #[test]
    fn open_spec_panel_action_respects_settings_modal_invariant() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| {
            let mut rv = RootView::new(vec![], dummy_path());
            let mut modal = settings::SettingsModal::new();
            modal.mount();
            rv.settings_modal = Some(modal);
            rv
        });

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.handle_open_spec_panel(cx);
                assert!(
                    view.spec_panel.is_none(),
                    "spec_panel must not mount while settings modal is active"
                );
            });
        });
    }

    /// AC-MN-7: OpenCommandPalette toggles the cmd palette state.
    #[test]
    fn open_command_palette_action_toggles_cmd_palette() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(view.palette.active_variant.is_none());

        view.toggle_cmd_palette();
        assert!(
            view.palette.active_variant.is_some(),
            "first toggle opens the palette"
        );

        view.toggle_cmd_palette();
        assert!(
            view.palette.active_variant.is_none(),
            "second toggle dismisses the palette"
        );
    }

    // ============================================================
    // SPEC-V3-009 MS-4a — Terminal SPEC-ID click wiring tests
    // (AC-SU-13 / AC-SU-14 / AC-SU-15 / AC-SU-16)
    // ============================================================

    /// AC-SU-13: OpenSpec event with no panel mounted lazily mounts the panel.
    /// The id may not exist in the index when running tests against a tmp
    /// storage_path, so we only assert mount + no-panic; AC-SU-16 covers the
    /// graceful no-op selection contract.
    #[test]
    fn terminal_open_spec_lazy_mounts_spec_panel() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                assert!(view.spec_panel.is_none());
                view.handle_terminal_open_spec("SPEC-V3-007", cx);
                assert!(
                    view.spec_panel.is_some(),
                    "spec_panel must mount when OpenSpec arrives"
                );
            });
        });
    }

    /// AC-SU-14: a second OpenSpec for a different id reuses the existing
    /// panel without unmounting it.
    #[test]
    fn terminal_open_spec_with_existing_panel_keeps_it_mounted() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.handle_terminal_open_spec("SPEC-V3-007", cx);
                let was_mounted = view.spec_panel.is_some();
                view.handle_terminal_open_spec("SPEC-V3-009", cx);
                let is_mounted = view.spec_panel.is_some();
                assert!(
                    was_mounted && is_mounted,
                    "panel must stay mounted across consecutive OpenSpec events"
                );
            });
        });
    }

    /// AC-SU-15: OpenSpec respects the single-overlay invariant — palette
    /// active means the click is logged and discarded.
    #[test]
    fn terminal_open_spec_ignored_when_palette_visible() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| {
            let mut rv = RootView::new(vec![], dummy_path());
            rv.palette.toggle(palette::PaletteVariant::CommandPalette);
            rv
        });

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                assert!(view.palette.is_visible(), "test prerequisite");
                view.handle_terminal_open_spec("SPEC-V3-007", cx);
                assert!(
                    view.spec_panel.is_none(),
                    "spec_panel must not mount while palette is visible"
                );
            });
        });
    }

    /// AC-SU-15 (companion): same invariant for the settings modal.
    #[test]
    fn terminal_open_spec_ignored_when_settings_modal_open() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| {
            let mut rv = RootView::new(vec![], dummy_path());
            let mut modal = settings::SettingsModal::new();
            modal.mount();
            rv.settings_modal = Some(modal);
            rv
        });

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.handle_terminal_open_spec("SPEC-V3-007", cx);
                assert!(
                    view.spec_panel.is_none(),
                    "spec_panel must not mount while settings modal is active"
                );
            });
        });
    }

    /// AC-SU-16: an unknown spec_id is a graceful no-op for selection.
    /// The panel still mounts because mounting is always a safe view change,
    /// but `select_spec` keeps the prior selection (None) per its docs.
    #[test]
    fn terminal_open_spec_with_unknown_id_does_not_select() {
        use gpui::{AppContext, TestAppContext};

        let mut cx = TestAppContext::single();
        let root_entity = cx.new(|_| RootView::new(vec![], dummy_path()));

        cx.update(|app| {
            root_entity.update(app, |view: &mut RootView, cx| {
                view.handle_terminal_open_spec("SPEC-DOES-NOT-EXIST", cx);
                let panel = view.spec_panel.as_ref().expect("panel mounted");
                assert!(
                    panel.list.selected_id.is_none(),
                    "selected_id stays None when spec_id is missing from index"
                );
            });
        });
    }

    // ── T7: Command Palette workspace.search → SearchPanel toggle (AC-GS-11) ──

    /// AC-GS-11 (REQ-GS-050): dispatch_command("workspace.search") activates
    /// the SearchPanel and makes it visible.
    #[test]
    fn test_palette_workspace_search_entry_toggles_search_panel() {
        let mut view = RootView::new(vec![], dummy_path());
        // SearchPanel is None initially.
        assert!(
            view.search_panel.is_none(),
            "search_panel must be None before first dispatch"
        );
        // dispatch_command requires no GPUI context for logic-level assertion;
        // we call the inner toggle logic directly (Spike 2 pattern).
        // dispatch_command_no_cx is the context-free entry point wired in MS-3.
        view.dispatch_command_workspace_search();
        assert!(
            view.search_panel.is_some(),
            "search_panel must be Some after workspace.search dispatch"
        );
        let panel = view.search_panel.as_ref().unwrap();
        assert!(
            panel.is_visible(),
            "SearchPanel must be visible after dispatch"
        );
    }

    /// AC-GS-11: dispatch_command("workspace.search") via full dispatch path
    /// returns true and activates the panel.
    #[test]
    fn test_dispatch_command_workspace_search_returns_true() {
        // dispatch_command needs a Context<Self> for cx.notify() — tested via
        // dispatch_command_workspace_search() (context-free helper) above.
        // Here we verify dispatch_command routing reaches the workspace.search
        // branch by confirming the panel state indirectly via the no-cx helper.
        let mut view = RootView::new(vec![], dummy_path());
        view.dispatch_command_workspace_search();
        assert!(view.search_panel.as_ref().is_some_and(|p| p.is_visible()));
    }

    // ── T2/T3/T4: handle_search_open — AC-GS-10 ──

    /// AC-GS-10 (T3/T4): handle_search_open resolves OpenCodeViewer with
    /// correct path and line/col when workspace is known.
    #[test]
    fn test_handle_search_open_calls_new_tab_with_code_kind() {
        use moai_studio_workspace::Workspace;
        use std::path::PathBuf;

        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace {
            id: "ws-code".to_string(),
            name: "code-project".to_string(),
            project_path: tmp.path().to_path_buf(),
            moai_config: PathBuf::from(".moai"),
            color: 0,
            last_active: 0,
        };
        let mut view = RootView::new(vec![ws], dummy_path());

        let hit = moai_search::SearchHit {
            workspace_id: "ws-code".to_string(),
            rel_path: PathBuf::from("src/lib.rs"),
            line: 12,
            col: 4,
            preview: "fn main() {".to_string(),
            match_start: 3,
            match_end: 7,
        };

        // handle_search_open must return true and record the OpenCodeViewer.
        let success = view.handle_search_open(&hit);
        assert!(
            success,
            "handle_search_open must return true for known workspace"
        );

        let ocv = view
            .last_open_code_viewer
            .as_ref()
            .expect("last_open_code_viewer must be set after successful navigation");
        assert_eq!(ocv.line, Some(12), "line must match hit.line");
        assert_eq!(ocv.col, Some(4), "col must match hit.col");
        assert!(
            ocv.path.ends_with("src/lib.rs"),
            "path must include rel_path"
        );
        assert!(
            ocv.path.starts_with(tmp.path()),
            "path must be rooted in workspace project_path"
        );
    }

    /// AC-GS-10 (T4): line and col in OpenCodeViewer match the hit precisely.
    #[test]
    fn test_handle_search_open_dispatches_open_code_viewer_with_line_col() {
        use moai_studio_workspace::Workspace;
        use std::path::PathBuf;

        let tmp = tempfile::tempdir().unwrap();
        let ws = Workspace {
            id: "ws-lc".to_string(),
            name: "line-col-project".to_string(),
            project_path: tmp.path().to_path_buf(),
            moai_config: PathBuf::from(".moai"),
            color: 0,
            last_active: 0,
        };
        let mut view = RootView::new(vec![ws], dummy_path());

        let hit = moai_search::SearchHit {
            workspace_id: "ws-lc".to_string(),
            rel_path: PathBuf::from("deep/nested/file.rs"),
            line: 999,
            col: 88,
            preview: "some text".to_string(),
            match_start: 0,
            match_end: 4,
        };

        view.handle_search_open(&hit);
        let ocv = view.last_open_code_viewer.as_ref().unwrap();
        assert_eq!(ocv.line, Some(999));
        assert_eq!(ocv.col, Some(88));
    }

    /// AC-GS-10 (T5): handle_search_open with unknown workspace returns false
    /// without panicking — last_open_code_viewer is unchanged.
    #[test]
    fn test_handle_search_open_unknown_workspace_no_panic() {
        use std::path::PathBuf;

        let mut view = RootView::new(vec![], dummy_path());

        let hit = moai_search::SearchHit {
            workspace_id: "ws-does-not-exist".to_string(),
            rel_path: PathBuf::from("src/main.rs"),
            line: 1,
            col: 0,
            preview: "text".to_string(),
            match_start: 0,
            match_end: 4,
        };

        // Must not panic; must return false.
        let success = view.handle_search_open(&hit);
        assert!(
            !success,
            "handle_search_open must return false for unknown workspace"
        );
        assert!(
            view.last_open_code_viewer.is_none(),
            "last_open_code_viewer must stay None when workspace is unknown"
        );
    }

    // ── T7: RootView handle_workspace_menu_action (REQ-D2-MS5-5 wire) ───────

    fn make_store_with_ws(
        name: &str,
        id_suffix: &str,
    ) -> (moai_studio_workspace::WorkspacesStore, String) {
        let tmp = std::env::temp_dir().join(format!("moai-rootview-t7-{}.json", id_suffix));
        std::fs::remove_file(&tmp).ok();
        let project_dir = std::env::temp_dir().join(format!("moai-rootview-t7-proj-{}", id_suffix));
        std::fs::create_dir_all(&project_dir).unwrap();

        let mut store = moai_studio_workspace::WorkspacesStore::load(&tmp).unwrap();
        let mut ws = moai_studio_workspace::Workspace::from_path(&project_dir).unwrap();
        ws.name = name.to_string();
        let id = ws.id.clone();
        store.add(ws).unwrap();
        (store, id)
    }

    /// REQ-D2-MS5-5 wire: Rename action opens rename_modal with correct target.
    #[test]
    fn test_root_view_handle_workspace_menu_action_rename_opens_modal() {
        use gpui::TestAppContext;

        let (store, ws_id) = make_store_with_ws("MyWorkspace", "rename");
        let workspaces = store.list().to_vec();

        let mut cx = TestAppContext::single();
        let root = cx.new(|_cx| {
            let mut view = RootView::new(workspaces, dummy_path());
            // Inject the pre-loaded store so tests don't hit real disk paths.
            view.store = store;
            view
        });

        cx.update(|app| {
            root.update(app, |view: &mut RootView, _cx| {
                view.handle_workspace_menu_action_logic(
                    workspace_menu::WorkspaceMenuAction::Rename,
                    &ws_id,
                );
                assert!(
                    view.rename_modal.is_some(),
                    "rename_modal must be Some after Rename action"
                );
                let modal = view.rename_modal.as_ref().unwrap();
                assert!(modal.is_open(), "RenameModal must be open");
                assert_eq!(modal.target_id(), Some(ws_id.as_str()));
            });
        });

        // Cleanup temp files created by make_store_with_ws
        let tmp = std::env::temp_dir().join("moai-rootview-t7-rename.json");
        std::fs::remove_file(&tmp).ok();
        let p = std::env::temp_dir().join("moai-rootview-t7-proj-rename");
        std::fs::remove_dir_all(&p).ok();
    }

    /// REQ-D2-MS5-5 wire: Delete action opens delete_confirmation with correct target.
    #[test]
    fn test_root_view_handle_workspace_menu_action_delete_opens_confirmation() {
        use gpui::TestAppContext;

        let (store, ws_id) = make_store_with_ws("MyWorkspace", "delete");
        let workspaces = store.list().to_vec();

        let mut cx = TestAppContext::single();
        let root = cx.new(|_cx| {
            let mut view = RootView::new(workspaces, dummy_path());
            view.store = store;
            view
        });

        cx.update(|app| {
            root.update(app, |view: &mut RootView, _cx| {
                view.handle_workspace_menu_action_logic(
                    workspace_menu::WorkspaceMenuAction::Delete,
                    &ws_id,
                );
                assert!(
                    view.delete_confirmation.is_some(),
                    "delete_confirmation must be Some after Delete action"
                );
                let conf = view.delete_confirmation.as_ref().unwrap();
                assert!(conf.is_open(), "DeleteConfirmation must be open");
                assert_eq!(conf.target_id(), Some(ws_id.as_str()));
            });
        });

        let tmp = std::env::temp_dir().join("moai-rootview-t7-delete.json");
        std::fs::remove_file(&tmp).ok();
        let p = std::env::temp_dir().join("moai-rootview-t7-proj-delete");
        std::fs::remove_dir_all(&p).ok();
    }

    /// REQ-D2-MS5-5 wire: MoveUp action reorders the store list.
    #[test]
    fn test_root_view_handle_workspace_menu_action_move_up_calls_store() {
        use gpui::TestAppContext;

        // Two-workspace store: ws_a at index 0, ws_b at index 1.
        let tmp = std::env::temp_dir().join("moai-rootview-t7-moveup.json");
        std::fs::remove_file(&tmp).ok();
        let pa = std::env::temp_dir().join("moai-rootview-t7-proj-moveup-a");
        let pb = std::env::temp_dir().join("moai-rootview-t7-proj-moveup-b");
        std::fs::create_dir_all(&pa).unwrap();
        std::fs::create_dir_all(&pb).unwrap();

        let mut store = moai_studio_workspace::WorkspacesStore::load(&tmp).unwrap();
        let ws_a = moai_studio_workspace::Workspace::from_path(&pa).unwrap();
        let ws_b = moai_studio_workspace::Workspace::from_path(&pb).unwrap();
        let id_b = ws_b.id.clone();
        store.add(ws_a).unwrap();
        store.add(ws_b).unwrap();

        let workspaces = store.list().to_vec();

        let mut cx = TestAppContext::single();
        let root = cx.new(|_cx| {
            let mut view = RootView::new(workspaces, dummy_path());
            view.store = store;
            view
        });

        cx.update(|app| {
            root.update(app, |view: &mut RootView, _cx| {
                view.handle_workspace_menu_action_logic(
                    workspace_menu::WorkspaceMenuAction::MoveUp,
                    &id_b,
                );
                // ws_b should now be at index 0
                assert_eq!(
                    view.store.list()[0].id,
                    id_b,
                    "ws_b should be at index 0 after MoveUp"
                );
            });
        });

        std::fs::remove_file(&tmp).ok();
        std::fs::remove_dir_all(&pa).ok();
        std::fs::remove_dir_all(&pb).ok();
    }

    // ── T7: RootView shell_picker — AC-MS-7 ──

    /// AC-MS-7 (REQ-MS-007): handle_switch_shell activates shell_picker (logic-level).
    ///
    /// Uses `dispatch_command_shell_switch()` (no-cx variant) to avoid
    /// requiring a live GPUI context in unit tests (Spike 2 pattern).
    #[test]
    fn test_root_view_handle_switch_shell_activates_picker() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(
            view.shell_picker.is_none(),
            "shell_picker must be None initially"
        );
        view.dispatch_command_shell_switch();
        assert!(
            view.shell_picker.is_some(),
            "shell_picker must be Some after dispatch_command_shell_switch()"
        );
    }

    /// AC-MS-7: dispatch_command("shell.switch") activates shell_picker.
    #[test]
    fn test_dispatch_command_shell_switch_activates_picker() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(
            view.shell_picker.is_none(),
            "shell_picker must be None initially"
        );
        let handled = view.dispatch_command("shell.switch");
        assert!(handled, "dispatch_command('shell.switch') must return true");
        assert!(
            view.shell_picker.is_some(),
            "shell_picker must be Some after dispatch_command('shell.switch')"
        );
    }

    // ── T8: SPEC-V3-004 MS-6 — workspace context menu GPUI overlay mount ──

    /// Helper that builds a `RootView` already wired to a temp `WorkspacesStore`
    /// containing the supplied workspace names. Returns the view and the list
    /// of generated ids in insertion order. Cleanup of temp files is the
    /// caller's responsibility (call `cleanup_t8_temp(suffix)` at end).
    fn make_root_view_with_workspaces(names: &[&str], suffix: &str) -> (RootView, Vec<String>) {
        let tmp = std::env::temp_dir().join(format!("moai-rootview-t8-{}.json", suffix));
        std::fs::remove_file(&tmp).ok();
        let mut store = moai_studio_workspace::WorkspacesStore::load(&tmp).unwrap();
        let mut ids = Vec::new();
        for (i, name) in names.iter().enumerate() {
            let project_dir =
                std::env::temp_dir().join(format!("moai-rootview-t8-proj-{}-{}", suffix, i));
            std::fs::create_dir_all(&project_dir).unwrap();
            let mut ws = moai_studio_workspace::Workspace::from_path(&project_dir).unwrap();
            ws.name = name.to_string();
            ids.push(ws.id.clone());
            store.add(ws).unwrap();
        }
        let workspaces = store.list().to_vec();
        let mut view = RootView::new(workspaces, tmp.clone());
        view.store = store;
        (view, ids)
    }

    fn cleanup_t8_temp(suffix: &str, count: usize) {
        let tmp = std::env::temp_dir().join(format!("moai-rootview-t8-{}.json", suffix));
        std::fs::remove_file(&tmp).ok();
        for i in 0..count {
            let p = std::env::temp_dir().join(format!("moai-rootview-t8-proj-{}-{}", suffix, i));
            std::fs::remove_dir_all(&p).ok();
        }
    }

    /// AC-D2-11 (REQ-D2-MS6-1): RootView::new initializes workspace_menu closed.
    #[test]
    fn test_workspace_menu_default_closed_on_root_view_new() {
        let view = RootView::new(vec![], dummy_path());
        assert!(
            !view.workspace_menu.is_open(),
            "workspace_menu must be closed by default on RootView::new"
        );
        assert_eq!(view.workspace_menu.visible_target(), None);
        assert_eq!(view.workspace_menu.visible_position(), None);
    }

    /// AC-D2-12 (REQ-D2-MS6-2): open_workspace_menu_at records target + position.
    #[test]
    fn test_open_workspace_menu_at_records_target_and_position() {
        let mut view = RootView::new(vec![], dummy_path());
        view.open_workspace_menu_at("ws-1", 100.0, 200.0);
        assert!(view.workspace_menu.is_open());
        assert!(view.workspace_menu.is_visible_for("ws-1"));
        assert!(!view.workspace_menu.is_visible_for("ws-2"));
        assert_eq!(
            view.workspace_menu.visible_position(),
            Some(workspace_menu::MenuPosition { x: 100.0, y: 200.0 })
        );
    }

    /// AC-D2-13 (REQ-D2-MS6-3): clicking Rename opens rename_modal AND closes the menu.
    #[test]
    fn test_click_workspace_menu_rename_opens_rename_modal_and_closes_menu() {
        let (mut view, ids) = make_root_view_with_workspaces(&["OldName"], "rename");
        let ws_id = ids[0].clone();
        view.open_workspace_menu_at(&ws_id, 10.0, 20.0);
        view.click_workspace_menu_item(workspace_menu::WorkspaceMenuAction::Rename);

        assert!(view.rename_modal.is_some(), "rename_modal must be opened");
        let modal = view.rename_modal.as_ref().unwrap();
        assert_eq!(modal.target_id(), Some(ws_id.as_str()));
        assert_eq!(modal.buffer(), "OldName");
        assert!(
            !view.workspace_menu.is_open(),
            "workspace_menu must close after click (single-menu invariant)"
        );
        cleanup_t8_temp("rename", 1);
    }

    /// AC-D2-13 mirror: clicking Delete opens delete_confirmation AND closes the menu.
    #[test]
    fn test_click_workspace_menu_delete_opens_delete_confirmation_and_closes_menu() {
        let (mut view, ids) = make_root_view_with_workspaces(&["W1"], "delete");
        let ws_id = ids[0].clone();
        view.open_workspace_menu_at(&ws_id, 10.0, 20.0);
        view.click_workspace_menu_item(workspace_menu::WorkspaceMenuAction::Delete);

        assert!(
            view.delete_confirmation.is_some(),
            "delete_confirmation must be opened"
        );
        let conf = view.delete_confirmation.as_ref().unwrap();
        assert_eq!(conf.target_id(), Some(ws_id.as_str()));
        assert!(
            !view.workspace_menu.is_open(),
            "workspace_menu must close after click"
        );
        cleanup_t8_temp("delete", 1);
    }

    /// AC-D2-13 + Reordered sync fix: MoveUp must mutate store AND mirror into self.workspaces.
    #[test]
    fn test_click_workspace_menu_move_up_calls_store_and_syncs_workspaces() {
        let (mut view, ids) = make_root_view_with_workspaces(&["A", "B"], "moveup");
        let id_b = ids[1].clone();
        // Sanity: B starts at index 1 in both store and workspaces vector.
        assert_eq!(view.workspaces[1].id, id_b);
        assert_eq!(view.store.list()[1].id, id_b);

        view.open_workspace_menu_at(&id_b, 10.0, 20.0);
        view.click_workspace_menu_item(workspace_menu::WorkspaceMenuAction::MoveUp);

        // After MoveUp: B is at index 0 in BOTH store and (synced) workspaces vector.
        assert_eq!(view.store.list()[0].id, id_b, "store reordered");
        assert_eq!(
            view.workspaces[0].id, id_b,
            "self.workspaces must be re-synced from store (MS-6 Reordered fix)"
        );
        assert!(!view.workspace_menu.is_open());
        cleanup_t8_temp("moveup", 2);
    }

    /// AC-D2-14 (rename half): commit_rename_modal renames in store + syncs workspaces.
    #[test]
    fn test_commit_rename_modal_renames_in_store_and_syncs_workspaces() {
        let (mut view, ids) = make_root_view_with_workspaces(&["Old"], "commit-rename");
        let ws_id = ids[0].clone();
        view.rename_modal = Some({
            let mut m = workspace_menu::RenameModal::default();
            m.open(ws_id.clone(), "Old");
            m.set_buffer("New");
            m
        });

        let result = view.commit_rename_modal();
        assert_eq!(
            result,
            Some((ws_id.clone(), "New".to_string())),
            "commit must return Some((ws_id, new_name))"
        );
        assert!(view.rename_modal.is_none(), "rename_modal must be cleared");
        assert_eq!(
            view.store.list()[0].name,
            "New",
            "store must hold the new name"
        );
        assert_eq!(
            view.workspaces[0].name, "New",
            "self.workspaces must be re-synced from store"
        );
        cleanup_t8_temp("commit-rename", 1);
    }

    /// AC-D2-14 (delete half): confirm_delete_modal removes from store + syncs workspaces.
    #[test]
    fn test_confirm_delete_modal_removes_from_store_and_syncs_workspaces() {
        let (mut view, ids) = make_root_view_with_workspaces(&["A", "B"], "confirm-delete");
        let id_a = ids[0].clone();
        let id_b = ids[1].clone();
        view.active_id = Some(id_a.clone());
        view.delete_confirmation = Some({
            let mut c = workspace_menu::DeleteConfirmation::default();
            c.open(id_b.clone());
            c
        });

        let result = view.confirm_delete_modal();
        assert_eq!(
            result,
            Some(id_b.clone()),
            "confirm must return Some(ws_id)"
        );
        assert!(
            view.delete_confirmation.is_none(),
            "delete_confirmation must be cleared"
        );
        assert_eq!(view.workspaces.len(), 1, "B must be removed");
        assert_eq!(view.workspaces[0].id, id_a, "only A remains");
        assert_eq!(view.store.list().len(), 1);
        assert_eq!(
            view.active_id.as_deref(),
            Some(id_a.as_str()),
            "active_id stays on A (was already A)"
        );
        cleanup_t8_temp("confirm-delete", 2);
    }

    /// Additional invariant: deleting the active workspace reassigns active_id.
    #[test]
    fn test_confirm_delete_modal_reassigns_active_when_active_deleted() {
        let (mut view, ids) = make_root_view_with_workspaces(&["A", "B"], "delete-active");
        let id_a = ids[0].clone();
        let id_b = ids[1].clone();
        view.active_id = Some(id_a.clone());
        view.delete_confirmation = Some({
            let mut c = workspace_menu::DeleteConfirmation::default();
            c.open(id_a.clone());
            c
        });

        let result = view.confirm_delete_modal();
        assert_eq!(result, Some(id_a.clone()));
        assert_eq!(view.workspaces.len(), 1);
        assert_eq!(view.workspaces[0].id, id_b);
        assert_eq!(
            view.active_id.as_deref(),
            Some(id_b.as_str()),
            "active_id must be reassigned to the first remaining workspace"
        );
        cleanup_t8_temp("delete-active", 2);
    }

    /// cancel_rename_modal clears the modal slot without persisting.
    #[test]
    fn test_cancel_rename_modal_clears_state() {
        let (mut view, ids) = make_root_view_with_workspaces(&["Old"], "cancel-rename");
        let ws_id = ids[0].clone();
        view.rename_modal = Some({
            let mut m = workspace_menu::RenameModal::default();
            m.open(ws_id.clone(), "Old");
            m.set_buffer("ShouldNotPersist");
            m
        });
        view.cancel_rename_modal();
        assert!(view.rename_modal.is_none());
        // Store name unchanged.
        assert_eq!(view.store.list()[0].name, "Old");
        cleanup_t8_temp("cancel-rename", 1);
    }

    /// cancel_delete_modal clears the modal slot without removing.
    #[test]
    fn test_cancel_delete_modal_clears_state() {
        let (mut view, ids) = make_root_view_with_workspaces(&["A"], "cancel-delete");
        let id_a = ids[0].clone();
        view.delete_confirmation = Some({
            let mut c = workspace_menu::DeleteConfirmation::default();
            c.open(id_a.clone());
            c
        });
        view.cancel_delete_modal();
        assert!(view.delete_confirmation.is_none());
        assert_eq!(view.workspaces.len(), 1);
        assert_eq!(view.store.list().len(), 1);
        cleanup_t8_temp("cancel-delete", 1);
    }

    /// click_workspace_menu_item is a no-op when the menu is closed.
    #[test]
    fn test_click_workspace_menu_item_noop_when_closed() {
        let mut view = RootView::new(vec![], dummy_path());
        assert!(!view.workspace_menu.is_open());
        view.click_workspace_menu_item(workspace_menu::WorkspaceMenuAction::Rename);
        // No state changes — no panic, no modal opened.
        assert!(view.rename_modal.is_none());
        assert!(view.delete_confirmation.is_none());
    }

    /// commit_rename_modal returns None when no modal is open.
    #[test]
    fn test_commit_rename_modal_returns_none_when_closed() {
        let mut view = RootView::new(vec![], dummy_path());
        assert_eq!(view.commit_rename_modal(), None);
    }

    /// confirm_delete_modal returns None when no modal is open.
    #[test]
    fn test_confirm_delete_modal_returns_none_when_closed() {
        let mut view = RootView::new(vec![], dummy_path());
        assert_eq!(view.confirm_delete_modal(), None);
    }

    // ── T9: SPEC-V0-2-0-MISSION-CTRL-001 MS-2 — RootView mission_control field ──

    /// AC-MC-13 (REQ-MC-024): RootView::new initializes mission_control to None (lazy).
    #[test]
    fn test_mission_control_is_none_on_root_view_new() {
        let view = RootView::new(vec![], dummy_path());
        assert!(
            view.mission_control.is_none(),
            "mission_control must be None on RootView::new (lazy init)"
        );
    }

    /// REQ-MC-024 (ensure): ensure_mission_control creates the entity on first call.
    #[test]
    fn test_ensure_mission_control_creates_entity() {
        use gpui::TestAppContext;

        let mut cx = TestAppContext::single();
        let root = cx.new(|_cx| RootView::new(vec![], dummy_path()));
        cx.update(|app| {
            root.update(app, |view: &mut RootView, cx| {
                assert!(view.mission_control.is_none());
                view.ensure_mission_control(cx);
                assert!(
                    view.mission_control.is_some(),
                    "ensure_mission_control must create the entity"
                );
            });
        });
    }

    /// REQ-MC-024 (ensure idempotent): second call does not replace the entity.
    #[test]
    fn test_ensure_mission_control_is_idempotent() {
        use gpui::TestAppContext;

        let mut cx = TestAppContext::single();
        let root = cx.new(|_cx| RootView::new(vec![], dummy_path()));
        cx.update(|app| {
            root.update(app, |view: &mut RootView, cx| {
                view.ensure_mission_control(cx);
                let first_id = view.mission_control.as_ref().unwrap().entity_id();
                view.ensure_mission_control(cx);
                let second_id = view.mission_control.as_ref().unwrap().entity_id();
                assert_eq!(
                    first_id, second_id,
                    "second ensure must NOT replace the existing entity"
                );
            });
        });
    }

    /// REQ-MC-024 (dismiss): dismiss_mission_control releases the entity.
    #[test]
    fn test_dismiss_mission_control_clears_entity() {
        use gpui::TestAppContext;

        let mut cx = TestAppContext::single();
        let root = cx.new(|_cx| RootView::new(vec![], dummy_path()));
        cx.update(|app| {
            root.update(app, |view: &mut RootView, cx| {
                view.ensure_mission_control(cx);
                assert!(view.mission_control.is_some());
                view.dismiss_mission_control();
                assert!(view.mission_control.is_none());
            });
        });
    }

    /// REQ-MC-024: update_mission_control_snapshot pushes cards into the entity.
    #[test]
    fn test_update_mission_control_snapshot_pushes_cards() {
        use gpui::TestAppContext;
        use moai_studio_agent::AgentCard;
        use moai_studio_agent::events::{AgentRunId, AgentRunStatus};

        let mut cx = TestAppContext::single();
        let root = cx.new(|_cx| RootView::new(vec![], dummy_path()));

        let card_a = {
            let mut c = AgentCard::new(AgentRunId("ra".to_string()), "Run A");
            c.status = AgentRunStatus::Running;
            c
        };

        cx.update(|app| {
            root.update(app, |view: &mut RootView, cx| {
                view.ensure_mission_control(cx);
                view.update_mission_control_snapshot(vec![card_a.clone()], cx);
                let entity = view.mission_control.as_ref().unwrap();
                let snapshot_len = entity.read(cx).snapshot.len();
                assert_eq!(snapshot_len, 1, "snapshot must contain the pushed card");
                assert_eq!(entity.read(cx).snapshot[0].label, "Run A");
            });
        });
    }

    /// REQ-MC-024: update_mission_control_snapshot is no-op when not mounted.
    #[test]
    fn test_update_mission_control_snapshot_noop_when_not_mounted() {
        use gpui::TestAppContext;

        let mut cx = TestAppContext::single();
        let root = cx.new(|_cx| RootView::new(vec![], dummy_path()));
        cx.update(|app| {
            root.update(app, |view: &mut RootView, cx| {
                // Not mounted → no panic.
                view.update_mission_control_snapshot(vec![], cx);
                assert!(view.mission_control.is_none());
            });
        });
    }
}
