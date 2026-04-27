//! Project Wizard Modal — 5-step workspace creation flow (G-2).
//!
//! SPEC: G-2 project wizard implementation.
//! Step 1: Directory picker, Step 2: Name input, Step 3: SPEC selection,
//! Step 4: Color tag, Step 5: Confirm + create.

use crate::design::tokens as tok;
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
