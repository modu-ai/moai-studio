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

pub mod agent;
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

use design::tokens::{self as tok, traffic};
use gpui::{
    App, Application, Context, Entity, IntoElement, KeyDownEvent, MouseButton, ParentElement,
    Render, Styled, Window, WindowOptions, div, prelude::*, px, rgb, size,
};
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
            settings_modal: None,
            user_settings,
            active_theme,
            find_bar_open: false,
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
    /// GPUI 이벤트 핸들러와 독립적으로 테스트 가능.
    pub fn apply_added_workspace(&mut self, added: &Workspace, all: Vec<Workspace>) {
        self.workspaces = all;
        self.active_id = Some(added.id.clone());
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
    fn handle_activate_workspace(&mut self, id: String, cx: &mut Context<Self>) {
        if !self.activate_workspace(&id) {
            return;
        }
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
            self.palette.toggle(palette::PaletteVariant::CmdPalette);
            return true;
        }
        if cmd && shift && key == "p" {
            self.palette.toggle(palette::PaletteVariant::CommandPalette);
            return true;
        }
        if !cmd && !shift && key == "escape" && self.palette.is_visible() {
            self.palette.dismiss();
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

    /// + New Workspace 버튼 클릭 처리 — store 재로드, 네이티브 picker, 상태 갱신.
    ///   사용자가 취소하거나 로드/저장이 실패하면 상태 유지.
    fn handle_add_workspace(&mut self, cx: &mut Context<Self>) {
        let mut store = match WorkspacesStore::load(&self.storage_path) {
            Ok(s) => s,
            Err(e) => {
                error!("WorkspacesStore::load 실패: {e}");
                return;
            }
        };
        match moai_studio_workspace::pick_and_save(&mut store) {
            Ok(Some(ws)) => {
                let all = store.list().to_vec();
                self.apply_added_workspace(&ws, all);
                cx.notify();
            }
            Ok(None) => info!("pick_and_save: 사용자 취소"),
            Err(e) => error!("pick_and_save 실패: {e}"),
        }
    }
}

impl Render for RootView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let new_ws_btn = new_workspace_button().on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _ev, _window, cx| this.handle_add_workspace(cx)),
        );
        // Row 클릭 리스너를 각 workspace 에 attach.
        let rows: Vec<gpui::Stateful<gpui::Div>> = self
            .workspaces
            .iter()
            .map(|ws| {
                let id = ws.id.clone();
                let is_active = self.active_id.as_deref() == Some(ws.id.as_str());
                workspace_row(ws, is_active).on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _ev, _window, cx| {
                        this.handle_activate_workspace(id.clone(), cx)
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
        let active_palette = self.palette.active_variant;
        let has_settings = self.settings_modal.is_some();
        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(tok::BG_APP))
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _window, cx| {
                // settings 키 먼저 처리 — 소비되면 나머지 스킵.
                if this.handle_settings_key_event(ev) {
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
            .child(main_body(&self.workspaces, rows, new_ws_btn, tab_container))
            .child(status_bar())
            .children(active_palette.map(|_v| render_palette_overlay()))
            .children(has_settings.then(render_settings_overlay))
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
// 1) TitleBar — 44pt 상단
// ============================================================

fn title_bar(active_label: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(44.))
        .px_4()
        .gap_3()
        .bg(rgb(tok::BG_SURFACE))
        .border_b_1()
        .border_color(rgb(tok::BORDER_SUBTLE))
        // 좌측 — traffic lights placeholder (native 윈도우 chrome 사용 시 숨김 가능)
        .child(traffic_lights())
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

/// macOS 전용 traffic lights (red/yellow/green). GPUI 자체 타이틀바 사용 시 생략 가능하나
/// 브랜드 일관성을 위해 인라인 렌더링.
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
) -> impl IntoElement {
    let is_empty = workspaces.is_empty();
    div()
        .flex()
        .flex_row()
        .flex_grow()
        .w_full()
        .child(sidebar(is_empty, rows, new_ws_btn))
        .child(content_area(is_empty, tab_container))
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
        .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(rgb(ws.color)))
        .child(div().text_sm().text_color(rgb(fg)).child(ws.name.clone()))
}

/// 컨텐츠 영역 — SPEC-V3-004 T2: tab_container Entity 렌더.
///
/// 우선순위 (SPEC-V3-004 REQ-R-001 ~ REQ-R-005):
///   1. tab_container 가 Some 이면 TabContainer 렌더 (MS-1+: 탭 바 + PaneTree)
///   2. show_empty_state 이면 Empty State CTA 렌더 (SPEC-V3-001 carry)
///   3. 그 외 (workspace 선택 but tab_container 없음) 플레이스홀더 렌더
fn content_area(
    show_empty_state: bool,
    tab_container: Option<Entity<TabContainer>>,
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
            .child(empty_state_primary_cta())
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

/// Primary CTA — `+ Create First Workspace` (MoAI 오렌지).
fn empty_state_primary_cta() -> impl IntoElement {
    div()
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

/// Palette overlay 렌더 — Scrim 위에 variant placeholder 를 표시한다 (AC-PL-14/15).
///
/// MS-3 에서는 Scrim (반투명 backdrop) + 중앙 정렬 컨테이너 placeholder 를 렌더한다.
/// 실제 CmdPalette/CommandPalette/SlashBar variant 컴포넌트는 follow-up 에서 연결.
fn render_palette_overlay() -> impl IntoElement {
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
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(tok::FG_SECONDARY))
                        .child("palette"),
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

// ============================================================
// 3) StatusBar — 28pt 하단
// ============================================================

fn status_bar() -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(28.))
        .px_3()
        .gap_3()
        .bg(rgb(tok::BG_ELEVATED))
        .border_t_1()
        .border_color(rgb(tok::BORDER_SUBTLE))
        .child(
            div()
                .text_xs()
                .text_color(rgb(tok::FG_MUTED))
                .child("no git"),
        )
        .child(div().text_xs().text_color(rgb(tok::FG_DISABLED)).child("·"))
        .child(
            div()
                .text_xs()
                .text_color(rgb(tok::FG_MUTED))
                .child("moai-studio v0.1.0"),
        )
        .child(div().flex_grow())
        // 우측 — ⌘K 힌트 (Command Palette 발견성)
        .child(
            div()
                .text_xs()
                .text_color(rgb(tok::FG_MUTED))
                .child("⌘K to search"),
        )
}

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
            cx.new(|_cx| RootView::new(ws, path))
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
}
