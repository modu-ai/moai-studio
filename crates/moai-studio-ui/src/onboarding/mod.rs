//! Onboarding helpers — environment detection, tour state, etc.
//!
//! SPEC-V0-2-0-ONBOARDING-ENV-001 MS-1 (audit Top 8 #6, v0.2.0 cycle Sprint 10):
//! `env` module provides read-only detection of the user's local toolchain
//! (shell / tmux / node / python / rust / git). ProjectWizard or a future
//! interactive tour consumes the report to guide the user through setup.

pub mod env;

pub use env::{
    CommandRunner, EnvironmentReport, RealCommandRunner, Tool, ToolStatus, detect_with_runner,
    parse_version_from_stdout,
};
