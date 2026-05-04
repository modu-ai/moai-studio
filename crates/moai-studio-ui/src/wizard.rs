//! Project Wizard Modal — 5-step workspace creation flow (G-2).
//!
//! SPEC: G-2 project wizard implementation.
//! Step 1: Directory picker, Step 2: Name input, Step 3: SPEC selection,
//! Step 4: Color tag, Step 5: Confirm + create.
//!
//! SPEC-V0-2-0-WIZARD-ENV-001 MS-1 (audit Top 8 #6 후속, v0.2.0 cycle Sprint 11+):
//! `env_report: Option<EnvironmentReport>` state binding 추가. 외부 caller 가
//! `detect_with_runner` 결과를 wizard 에 주입하면 후속 render PR 가 사용자에게
//! 환경 정보 (shell/tmux/node/python/rust/git 가용성) 를 표시한다. 본 SPEC 은
//! state 만 — UI render 는 carry per N2.

use crate::design::tokens as tok;
use crate::onboarding::{EnvironmentReport, Tool};
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb,
};

// ColorTag enum values (from moai_store)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorTag {
    Red,
    Orange,
    Yellow,
    Green,
    Blue,
    Purple,
    Pink,
    Gray,
}

impl ColorTag {
    pub const ALL: [ColorTag; 8] = [
        ColorTag::Red,
        ColorTag::Orange,
        ColorTag::Yellow,
        ColorTag::Green,
        ColorTag::Blue,
        ColorTag::Purple,
        ColorTag::Pink,
        ColorTag::Gray,
    ];
}

/// NewWorkspace parameters (simplified version).
pub struct NewWorkspace {
    pub name: String,
    pub project_path: String,
    pub spec_id: Option<String>,
    pub color_tag: Option<ColorTag>,
}

/// Wizard step enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardStep {
    Step1Directory,
    Step2Name,
    Step3Spec,
    Step4Color,
    Step5Confirm,
}

impl WizardStep {
    /// All wizard steps in order.
    pub const ALL: [WizardStep; 5] = [
        WizardStep::Step1Directory,
        WizardStep::Step2Name,
        WizardStep::Step3Spec,
        WizardStep::Step4Color,
        WizardStep::Step5Confirm,
    ];

    /// Step number (1-indexed).
    pub fn number(&self) -> usize {
        match self {
            WizardStep::Step1Directory => 1,
            WizardStep::Step2Name => 2,
            WizardStep::Step3Spec => 3,
            WizardStep::Step4Color => 4,
            WizardStep::Step5Confirm => 5,
        }
    }

    /// Step title for display.
    pub fn title(&self) -> &'static str {
        match self {
            WizardStep::Step1Directory => "Select Directory",
            WizardStep::Step2Name => "Project Name",
            WizardStep::Step3Spec => "SPEC (Optional)",
            WizardStep::Step4Color => "Color Tag",
            WizardStep::Step5Confirm => "Confirm",
        }
    }

    /// Returns the next step, if any.
    pub fn next(&self) -> Option<WizardStep> {
        match self {
            WizardStep::Step1Directory => Some(WizardStep::Step2Name),
            WizardStep::Step2Name => Some(WizardStep::Step3Spec),
            WizardStep::Step3Spec => Some(WizardStep::Step4Color),
            WizardStep::Step4Color => Some(WizardStep::Step5Confirm),
            WizardStep::Step5Confirm => None,
        }
    }

    /// Returns the previous step, if any.
    pub fn prev(&self) -> Option<WizardStep> {
        match self {
            WizardStep::Step1Directory => None,
            WizardStep::Step2Name => Some(WizardStep::Step1Directory),
            WizardStep::Step3Spec => Some(WizardStep::Step2Name),
            WizardStep::Step4Color => Some(WizardStep::Step3Spec),
            WizardStep::Step5Confirm => Some(WizardStep::Step4Color),
        }
    }
}

/// Project wizard modal state.
pub struct ProjectWizard {
    /// Current wizard step.
    current_step: WizardStep,
    /// Wizard visibility.
    visible: bool,

    // Step 1: Directory
    selected_directory: Option<String>,

    // Step 2: Name
    project_name: String,

    // Step 3: SPEC
    spec_id: Option<String>,

    // Step 4: Color tag
    selected_color: Option<ColorTag>,

    // SPEC-V0-2-0-WIZARD-ENV-001 MS-1 (REQ-WE-001): cached env detection report.
    // None = not yet probed. Populated by external caller via `set_env_report`,
    // cleared on `dismiss()` along with the rest of the wizard state.
    env_report: Option<EnvironmentReport>,
}

impl ProjectWizard {
    /// Create a new wizard in hidden state.
    pub fn new() -> Self {
        Self {
            current_step: WizardStep::Step1Directory,
            visible: false,
            selected_directory: None,
            project_name: String::new(),
            spec_id: None,
            selected_color: None,
            // SPEC-V0-2-0-WIZARD-ENV-001 MS-1 (REQ-WE-001): no env probe by default.
            env_report: None,
        }
    }

    /// Show the wizard (mount).
    pub fn mount(&mut self) {
        self.visible = true;
        self.current_step = WizardStep::Step1Directory;
    }

    /// Hide the wizard (dismiss).
    pub fn dismiss(&mut self) {
        self.visible = false;
        self.reset();
    }

    /// Reset wizard state.
    fn reset(&mut self) {
        self.current_step = WizardStep::Step1Directory;
        self.selected_directory = None;
        self.project_name = String::new();
        self.spec_id = None;
        self.selected_color = None;
        // SPEC-V0-2-0-WIZARD-ENV-001 MS-1 (REQ-WE-005): drop the cached env probe
        // so the next mount starts with a fresh slate.
        self.env_report = None;
    }

    // ── SPEC-V0-2-0-WIZARD-ENV-001 MS-1 — env_report state binding ──

    /// Inject the latest `EnvironmentReport` (typically from
    /// `crate::onboarding::detect_with_runner`).
    /// REQ-WE-002.
    pub fn set_env_report(&mut self, report: EnvironmentReport) {
        self.env_report = Some(report);
    }

    /// Returns the cached environment report, or `None` if no probe has been
    /// injected since the wizard was last constructed or dismissed.
    /// REQ-WE-003.
    pub fn env_report(&self) -> Option<&EnvironmentReport> {
        self.env_report.as_ref()
    }

    /// Drop the cached environment report without affecting the rest of the
    /// wizard state (different from `dismiss()` which resets everything).
    /// REQ-WE-004.
    pub fn clear_env_report(&mut self) {
        self.env_report = None;
    }

    /// Check if wizard is visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Navigate to next step.
    pub fn next_step(&mut self) {
        if let Some(step) = self.current_step.next() {
            self.current_step = step;
        }
    }

    /// Navigate to previous step.
    pub fn prev_step(&mut self) {
        if let Some(step) = self.current_step.prev() {
            self.current_step = step;
        }
    }

    /// Check if can go next.
    pub fn can_go_next(&self) -> bool {
        match self.current_step {
            WizardStep::Step1Directory => self.selected_directory.is_some(),
            WizardStep::Step2Name => !self.project_name.is_empty(),
            WizardStep::Step3Spec => true,  // Optional
            WizardStep::Step4Color => true, // Optional
            WizardStep::Step5Confirm => false,
        }
    }

    /// Check if can go back.
    pub fn can_go_back(&self) -> bool {
        self.current_step.prev().is_some()
    }

    /// Build NewWorkspace from collected data.
    pub fn build_workspace(&self) -> Option<NewWorkspace> {
        if self.selected_directory.is_none() || self.project_name.is_empty() {
            return None;
        }
        Some(NewWorkspace {
            name: self.project_name.clone(),
            project_path: self.selected_directory.clone().unwrap(),
            spec_id: self.spec_id.clone(),
            color_tag: self.selected_color,
        })
    }
}

impl Default for ProjectWizard {
    fn default() -> Self {
        Self::new()
    }
}

// SPEC-V0-2-0-WIZARD-ENV-001 MS-2 — pure formatters for the env section.
// These free functions stay outside `impl ProjectWizard` so unit tests can
// exercise them without constructing the wizard or touching GPUI.

/// SPEC-V0-2-0-WIZARD-ENV-001 MS-2 (REQ-WE-010, AC-WE-7): formatted env headline.
///
/// Returns a string of the form "{available}/6 tools available" using the
/// canonical 6-tool baseline from `Tool::all()`.
pub(crate) fn format_env_summary(report: &EnvironmentReport) -> String {
    format!("{}/6 tools available", report.available_count())
}

/// SPEC-V0-2-0-WIZARD-ENV-001 MS-2 (REQ-WE-011, AC-WE-8/9):
/// joined missing-tool display names, or `None` when every tool resolved.
///
/// Display names follow `Tool::display_name()` verbatim — Tmux maps to
/// "tmux" (lowercase) while Python and Git use title case.
pub(crate) fn format_missing_tools_label(report: &EnvironmentReport) -> Option<String> {
    let missing = report.missing_tools();
    if missing.is_empty() {
        None
    } else {
        Some(
            missing
                .iter()
                .map(|t: &Tool| t.display_name())
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

impl Render for ProjectWizard {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if !self.visible {
            return div(); // Hidden
        }

        // Wizard layout constants
        const WIZARD_WIDTH: f32 = 600.0;
        const WIZARD_HEIGHT: f32 = 500.0;

        div()
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgb(0x00000080)) // Semi-transparent scrim
            .child(
                div()
                    .w(px(WIZARD_WIDTH))
                    .h(px(WIZARD_HEIGHT))
                    .bg(rgb(tok::BG_ELEVATED))
                    .rounded_lg()
                    .border_1()
                    .border_color(rgb(tok::BORDER_SUBTLE))
                    .flex()
                    .flex_col()
                    .p_4()
                    .gap_4()
                    // Header
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(tok::FG_PRIMARY))
                                    .child("Create New Workspace"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(tok::FG_SECONDARY))
                                    .child(format!("Step {} of 5", self.current_step.number())),
                            ),
                    )
                    // SPEC-V0-2-0-WIZARD-ENV-001 MS-2 (REQ-WE-007): env section
                    // between header and progress bar — auto-detect status row.
                    .child(self.render_env_section())
                    // Progress bar
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_1()
                            .children(WizardStep::ALL.iter().map(|step| {
                                let is_active = *step == self.current_step;
                                let is_past = step.number() < self.current_step.number();
                                let bg_color = if is_active || is_past {
                                    rgb(tok::ACCENT)
                                } else {
                                    rgb(tok::BG_PANEL)
                                };
                                div().flex_1().h(px(4.)).rounded_full().bg(bg_color)
                            })),
                    )
                    // Step title
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(tok::FG_PRIMARY))
                            .child(self.current_step.title()),
                    )
                    // Step content placeholder
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_sm()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(self.render_step_content()),
                    )
                    // Navigation buttons
                    .child(self.render_navigation()),
            )
    }
}

impl ProjectWizard {
    /// SPEC-V0-2-0-WIZARD-ENV-001 MS-2 (REQ-WE-007/008/009): env section row.
    ///
    /// Branches on `env_report` state:
    /// - `None` → "Detecting environment..." muted placeholder.
    /// - `Some(report)` complete → headline + "All tools detected" accent line.
    /// - `Some(report)` partial → headline + missing tools (joined by ", ").
    ///
    /// Layout is a single column block intended for insertion between the
    /// header and the progress bar inside `Render::render`.
    fn render_env_section(&self) -> gpui::Div {
        match self.env_report.as_ref() {
            None => div()
                .text_xs()
                .text_color(rgb(tok::FG_MUTED))
                .child("Detecting environment..."),
            Some(report) => {
                let headline = format_env_summary(report);
                let mut col = div().flex().flex_col().gap_1().child(
                    div()
                        .text_sm()
                        .text_color(rgb(tok::FG_PRIMARY))
                        .child(format!("Environment: {headline}")),
                );
                if report.is_complete() {
                    col = col.child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::ACCENT))
                            .child("All tools detected"),
                    );
                } else if let Some(missing) = format_missing_tools_label(report) {
                    col = col.child(
                        div()
                            .text_xs()
                            .text_color(rgb(tok::FG_SECONDARY))
                            .child(format!("Missing: {missing}")),
                    );
                }
                col
            }
        }
    }

    fn render_navigation(&self) -> gpui::Div {
        let mut nav = div().flex().flex_row().justify_between();

        // Back button (conditional)
        if self.can_go_back() {
            nav = nav.child(
                div()
                    .px(px(16.))
                    .py(px(8.))
                    .rounded_md()
                    .bg(rgb(tok::BG_PANEL))
                    .text_color(rgb(tok::FG_PRIMARY))
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.8))
                    .child("Back"),
            );
        } else {
            nav = nav.child(div().w(px(80.))); // Spacer
        }

        // Next/Create button (conditional)
        if self.current_step == WizardStep::Step5Confirm && self.can_go_next() {
            nav = nav.child(
                div()
                    .px(px(16.))
                    .py(px(8.))
                    .rounded_md()
                    .bg(rgb(tok::ACCENT))
                    .text_color(rgb(0xFFFFFF))
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.9))
                    .child("Create"),
            );
        } else if self.can_go_next() {
            nav = nav.child(
                div()
                    .px(px(16.))
                    .py(px(8.))
                    .rounded_md()
                    .bg(rgb(tok::ACCENT))
                    .text_color(rgb(0xFFFFFF))
                    .cursor_pointer()
                    .hover(|s| s.opacity(0.9))
                    .child("Next"),
            );
        } else {
            nav = nav.child(div().w(px(80.))); // Spacer
        }

        nav
    }
}

impl ProjectWizard {
    fn render_step_content(&self) -> String {
        match self.current_step {
            WizardStep::Step1Directory => {
                if let Some(dir) = &self.selected_directory {
                    format!("Selected: {}", dir)
                } else {
                    "Click to select project directory...".to_string()
                }
            }
            WizardStep::Step2Name => {
                if self.project_name.is_empty() {
                    "Enter project name...".to_string()
                } else {
                    self.project_name.clone()
                }
            }
            WizardStep::Step3Spec => {
                if let Some(spec) = &self.spec_id {
                    format!("SPEC: {}", spec)
                } else {
                    "No SPEC selected (optional)".to_string()
                }
            }
            WizardStep::Step4Color => {
                if let Some(color) = self.selected_color {
                    format!("Color: {:?}", color)
                } else {
                    "No color selected".to_string()
                }
            }
            WizardStep::Step5Confirm => {
                format!(
                    "Name: {}\nDir: {:?}\nSPEC: {:?}\nColor: {:?}",
                    self.project_name, self.selected_directory, self.spec_id, self.selected_color
                )
            }
        }
    }
}

// ============================================================
// Unit tests — SPEC-V0-2-0-WIZARD-ENV-001 MS-1 (AC-WE-1 ~ AC-WE-6)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::onboarding::{EnvironmentReport, Tool, ToolStatus};

    fn mk_report() -> EnvironmentReport {
        EnvironmentReport::new(vec![
            (
                Tool::Shell,
                ToolStatus::Available {
                    version: "zsh 5.9".to_string(),
                },
            ),
            (Tool::Tmux, ToolStatus::NotFound),
            (
                Tool::Git,
                ToolStatus::Available {
                    version: "git 2.43".to_string(),
                },
            ),
        ])
    }

    /// AC-WE-1 (REQ-WE-001): new() initializes env_report to None.
    #[test]
    fn project_wizard_new_initializes_env_report_to_none() {
        let wiz = ProjectWizard::new();
        assert!(wiz.env_report().is_none());
        // Sanity: existing fields remain at default.
        assert!(!wiz.is_visible());
    }

    /// AC-WE-2 (REQ-WE-002 / 003): set_env_report stores the report.
    #[test]
    fn project_wizard_set_env_report_stores_value() {
        let mut wiz = ProjectWizard::new();
        let report = mk_report();
        wiz.set_env_report(report);
        let got = wiz.env_report().expect("env_report must be Some");
        assert_eq!(got.entries.len(), 3);
        assert_eq!(got.available_count(), 2);
    }

    /// AC-WE-3 (REQ-WE-004): clear_env_report resets to None.
    #[test]
    fn project_wizard_clear_env_report_resets() {
        let mut wiz = ProjectWizard::new();
        wiz.set_env_report(mk_report());
        assert!(wiz.env_report().is_some());
        wiz.clear_env_report();
        assert!(wiz.env_report().is_none());
    }

    /// AC-WE-4 (REQ-WE-005): dismiss() clears env_report along with the rest.
    #[test]
    fn project_wizard_dismiss_clears_env_report_and_state() {
        let mut wiz = ProjectWizard::new();
        wiz.mount();
        wiz.set_env_report(mk_report());
        // Advance a step so reset is observable.
        wiz.next_step(); // Step1 → Step2 (next() returns None for Step1 only when can_go_next false)
        assert!(wiz.is_visible());
        assert!(wiz.env_report().is_some());

        wiz.dismiss();

        assert!(!wiz.is_visible());
        assert!(wiz.env_report().is_none(), "dismiss must clear env_report");
        // build_workspace returns None because dir / name reset.
        assert!(wiz.build_workspace().is_none());
    }

    /// AC-WE-5 (REQ-WE-006): step navigation is independent of env_report.
    #[test]
    fn project_wizard_navigation_is_independent_of_env_report() {
        let mut wiz_with_env = ProjectWizard::new();
        wiz_with_env.set_env_report(mk_report());
        let wiz_without_env = ProjectWizard::new();

        // can_go_next at Step1 depends on selected_directory (None for both),
        // so both must agree.
        assert_eq!(
            wiz_with_env.can_go_next(),
            wiz_without_env.can_go_next(),
            "can_go_next must not depend on env_report"
        );
        assert_eq!(wiz_with_env.can_go_back(), wiz_without_env.can_go_back());

        // WizardStep enum still exposes 5 variants in canonical order.
        assert_eq!(WizardStep::ALL.len(), 5);
    }

    /// AC-WE-6 (REQ-WE-006): build_workspace ignores env_report.
    #[test]
    fn project_wizard_build_workspace_ignores_env_report() {
        let mut wiz = ProjectWizard::new();
        wiz.selected_directory = Some("/tmp/proj".to_string());
        wiz.project_name = "Demo".to_string();
        wiz.set_env_report(mk_report());
        let ws = wiz.build_workspace().expect("must build");
        assert_eq!(ws.name, "Demo");
        assert_eq!(ws.project_path, "/tmp/proj");
        assert_eq!(ws.spec_id, None);
        assert_eq!(ws.color_tag, None);
        // env_report is intentionally NOT carried into NewWorkspace (N7).
    }

    // ============================================================
    // SPEC-V0-2-0-WIZARD-ENV-001 MS-2 — env render helpers + render section
    // (AC-WE-7 ~ AC-WE-12)
    // ============================================================

    fn mk_complete_report() -> EnvironmentReport {
        EnvironmentReport::new(vec![
            (
                Tool::Shell,
                ToolStatus::Available {
                    version: "zsh 5.9".to_string(),
                },
            ),
            (
                Tool::Tmux,
                ToolStatus::Available {
                    version: "tmux 3.5".to_string(),
                },
            ),
            (
                Tool::Node,
                ToolStatus::Available {
                    version: "v20".to_string(),
                },
            ),
            (
                Tool::Python,
                ToolStatus::Available {
                    version: "Python 3.12".to_string(),
                },
            ),
            (
                Tool::Rust,
                ToolStatus::Available {
                    version: "cargo 1.92".to_string(),
                },
            ),
            (
                Tool::Git,
                ToolStatus::Available {
                    version: "git 2.43".to_string(),
                },
            ),
        ])
    }

    fn mk_partial_report_missing_tmux_python() -> EnvironmentReport {
        EnvironmentReport::new(vec![
            (
                Tool::Shell,
                ToolStatus::Available {
                    version: "zsh 5.9".to_string(),
                },
            ),
            (Tool::Tmux, ToolStatus::NotFound),
            (
                Tool::Node,
                ToolStatus::Available {
                    version: "v20".to_string(),
                },
            ),
            (Tool::Python, ToolStatus::NotFound),
            (
                Tool::Rust,
                ToolStatus::Available {
                    version: "cargo 1.92".to_string(),
                },
            ),
            (
                Tool::Git,
                ToolStatus::Available {
                    version: "git 2.43".to_string(),
                },
            ),
        ])
    }

    /// AC-WE-7 (REQ-WE-010): partial report — `2/6 tools available`.
    #[test]
    fn format_env_summary_partial_returns_two_of_six() {
        let report = mk_report(); // 3 entries, Shell + Git available, Tmux missing → 2/6
        assert_eq!(format_env_summary(&report), "2/6 tools available");
    }

    /// AC-WE-12 부분: complete report — `6/6 tools available`.
    #[test]
    fn format_env_summary_complete_returns_six_of_six() {
        let report = mk_complete_report();
        assert_eq!(format_env_summary(&report), "6/6 tools available");
    }

    /// AC-WE-8 (REQ-WE-011 negative): all available — `None`.
    #[test]
    fn format_missing_tools_label_complete_returns_none() {
        let report = mk_complete_report();
        assert!(format_missing_tools_label(&report).is_none());
    }

    /// AC-WE-9 (REQ-WE-011 positive): Tmux + Python missing → `"tmux, Python"`.
    ///
    /// Display name comes from `Tool::display_name()` verbatim:
    /// Tmux → "tmux" (lowercase), Python → "Python".
    #[test]
    fn format_missing_tools_label_partial_joins_display_names() {
        let report = mk_partial_report_missing_tmux_python();
        let label = format_missing_tools_label(&report).expect("must be Some");
        assert_eq!(label, "tmux, Python");
    }

    /// AC-WE-10 (REQ-WE-008/014): mount + render_env_section produce a Div
    /// without panicking when env_report is None (Detecting branch). Verifies
    /// the helper compiles and the wizard remains in a consistent state
    /// after mount() (env_report stays None until external set_env_report).
    #[test]
    fn wizard_render_env_section_with_none_compiles_and_state_stable() {
        let mut wiz = ProjectWizard::new();
        wiz.mount();
        assert!(wiz.is_visible());
        assert!(wiz.env_report().is_none());
        // Helper returns a Div — calling it must not panic.
        let _section: gpui::Div = wiz.render_env_section();
    }
}
