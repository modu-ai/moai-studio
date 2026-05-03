//! Onboarding environment detection — read-only toolchain probe.
//!
//! SPEC-V0-2-0-ONBOARDING-ENV-001 MS-1 (audit Top 8 #6, v0.2.0 cycle Sprint 10).
//!
//! Probes the user's local environment for 6 tools that MoAI Studio depends on
//! or works well with: shell, tmux, node, python, rust (cargo), git.
//!
//! The detection is split into a `CommandRunner` trait + concrete
//! `RealCommandRunner` so the entire pipeline is unit-testable with a mock
//! runner. ProjectWizard or a future onboarding screen calls
//! `detect_with_runner(&RealCommandRunner)` to get a fresh `EnvironmentReport`.
//!
//! Frozen zone (REQ-OE-008 implicit): no other crate is touched. lib.rs
//! receives a single `pub mod onboarding;` line addition.
//!
//! macOS / Linux only — Windows is out of scope (POSIX assumptions on
//! `std::env::var("SHELL")` and `Command::new(name)` PATH lookup).

use std::io;
use std::path::Path;
use std::process::Command;

// ============================================================
// Tool — REQ-OE-001 / REQ-OE-002
// ============================================================

/// Toolchain identifier — a fixed set of 6 tools the onboarding flow probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tool {
    /// User's interactive shell (zsh / bash / fish / nu — basename of `$SHELL`).
    Shell,
    /// Terminal multiplexer (`tmux`).
    Tmux,
    /// Node.js runtime (`node`).
    Node,
    /// Python interpreter (`python3` preferred; falls back to `python`).
    Python,
    /// Rust toolchain (`cargo` as the proxy for a working installation).
    Rust,
    /// Git client (`git`).
    Git,
}

impl Tool {
    /// All 6 tools in canonical order (matches the rendered list order).
    /// REQ-OE-001.
    pub fn all() -> [Tool; 6] {
        [
            Tool::Shell,
            Tool::Tmux,
            Tool::Node,
            Tool::Python,
            Tool::Rust,
            Tool::Git,
        ]
    }

    /// Returns the executable name to invoke for the version probe.
    ///
    /// `Shell` resolves at call time from `$SHELL`'s basename. The fallback
    /// when `$SHELL` is unset or empty is `"sh"` — every POSIX system has it.
    /// REQ-OE-002.
    pub fn executable(self) -> String {
        match self {
            Tool::Shell => shell_basename_from_env().unwrap_or_else(|| "sh".to_string()),
            Tool::Tmux => "tmux".to_string(),
            Tool::Node => "node".to_string(),
            Tool::Python => "python3".to_string(),
            Tool::Rust => "cargo".to_string(),
            Tool::Git => "git".to_string(),
        }
    }

    /// Human-readable display name for the report UI.
    /// REQ-OE-002.
    pub fn display_name(self) -> &'static str {
        match self {
            Tool::Shell => "Shell",
            Tool::Tmux => "tmux",
            Tool::Node => "Node.js",
            Tool::Python => "Python",
            Tool::Rust => "Rust (cargo)",
            Tool::Git => "Git",
        }
    }

    /// CLI flag that prints the version. tmux uses `-V` (uppercase, no dashes
    /// for "version"); the rest use the GNU-style `--version`.
    /// REQ-OE-002.
    pub fn version_arg(self) -> &'static str {
        match self {
            Tool::Tmux => "-V",
            _ => "--version",
        }
    }
}

/// Extract the basename of `$SHELL` if set and non-empty.
fn shell_basename_from_env() -> Option<String> {
    let raw = std::env::var("SHELL").ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    Path::new(trimmed)
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}

// ============================================================
// ToolStatus — REQ-OE-003
// ============================================================

/// Per-tool detection result.
///
/// `Available` carries a (best-effort) version string. `NotFound` is the
/// "PATH lookup failed" outcome. `Error` is everything else (permission
/// denied, exit non-zero with stderr, IO failure, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
    /// Tool is on PATH and the version probe succeeded.
    Available {
        /// First non-empty trimmed line of the version output.
        version: String,
    },
    /// Tool is not on PATH.
    NotFound,
    /// Tool was found but the probe failed.
    Error {
        /// Human-readable failure description.
        message: String,
    },
}

impl ToolStatus {
    /// True when the tool is `Available`.
    pub fn is_available(&self) -> bool {
        matches!(self, ToolStatus::Available { .. })
    }
}

// ============================================================
// EnvironmentReport — REQ-OE-004
// ============================================================

/// Aggregated probe result for all tools.
///
/// @MX:NOTE: [AUTO] environment-report-snapshot
/// @MX:SPEC: SPEC-V0-2-0-ONBOARDING-ENV-001 REQ-MC-004
/// `EnvironmentReport` is a flat `Vec` rather than a `HashMap` so the entry
/// order matches `Tool::all()` and the renderer can iterate without sorting.
#[derive(Debug, Clone)]
pub struct EnvironmentReport {
    /// (Tool, status) pairs in the canonical order of `Tool::all()`.
    pub entries: Vec<(Tool, ToolStatus)>,
}

impl EnvironmentReport {
    /// Construct from raw entries (test convenience).
    pub fn new(entries: Vec<(Tool, ToolStatus)>) -> Self {
        Self { entries }
    }

    /// Number of tools whose status is `Available`.
    pub fn available_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|(_, s)| s.is_available())
            .count()
    }

    /// Tools whose status is NOT `Available`.
    pub fn missing_tools(&self) -> Vec<Tool> {
        self.entries
            .iter()
            .filter(|(_, s)| !s.is_available())
            .map(|(t, _)| *t)
            .collect()
    }

    /// True when every probed tool is `Available`.
    pub fn is_complete(&self) -> bool {
        self.available_count() == self.entries.len()
    }
}

// ============================================================
// CommandRunner — REQ-OE-005 / REQ-OE-006
// ============================================================

/// Abstraction over `std::process::Command` execution.
///
/// Production code uses `RealCommandRunner`; tests use a mock that returns
/// canned strings without touching the filesystem.
pub trait CommandRunner {
    /// Run `executable args...` and return the captured stdout (UTF-8 lossy)
    /// on success. Returns `io::Error` on spawn / wait / non-zero exit.
    fn run(&self, executable: &str, args: &[&str]) -> io::Result<String>;
}

/// Real `std::process::Command` runner.
pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, executable: &str, args: &[&str]) -> io::Result<String> {
        let output = Command::new(executable).args(args).output()?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let message = if stderr.is_empty() {
                format!(
                    "{} {} exited with status {}",
                    executable,
                    args.join(" "),
                    output.status
                )
            } else {
                stderr
            };
            Err(io::Error::other(message))
        }
    }
}

// ============================================================
// Detection pipeline — REQ-OE-007
// ============================================================

/// Probe every `Tool::all()` entry through `runner` and collect the result.
///
/// This is the public entry point. ProjectWizard or the onboarding screen
/// calls `detect_with_runner(&RealCommandRunner)` to get a fresh report.
pub fn detect_with_runner(runner: &dyn CommandRunner) -> EnvironmentReport {
    let entries = Tool::all()
        .iter()
        .map(|tool| {
            let executable = tool.executable();
            let arg = tool.version_arg();
            let status = match runner.run(&executable, &[arg]) {
                Ok(stdout) => ToolStatus::Available {
                    version: parse_version_from_stdout(&stdout),
                },
                Err(e) if io_error_is_not_found(&e) => ToolStatus::NotFound,
                Err(e) => ToolStatus::Error {
                    message: e.to_string(),
                },
            };
            (*tool, status)
        })
        .collect();
    EnvironmentReport::new(entries)
}

/// Heuristic: classify "executable not found" errors so we surface
/// `ToolStatus::NotFound` instead of the generic `Error`.
fn io_error_is_not_found(e: &io::Error) -> bool {
    matches!(
        e.kind(),
        io::ErrorKind::NotFound | io::ErrorKind::PermissionDenied
    )
}

/// Extract the first non-empty trimmed line from `stdout` as a version label.
///
/// Returns `"(unknown)"` when stdout has no usable line — keeps the UI from
/// rendering an empty pill.
pub fn parse_version_from_stdout(stdout: &str) -> String {
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "(unknown)".to_string()
}

// ============================================================
// Unit tests — SPEC-V0-2-0-ONBOARDING-ENV-001 MS-1 (AC-OE-1 ~ AC-OE-7)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// Mock runner that returns canned outputs keyed by executable name.
    /// Missing keys produce `io::ErrorKind::NotFound`.
    struct MockRunner {
        canned: RefCell<HashMap<String, io::Result<String>>>,
    }

    impl MockRunner {
        fn new() -> Self {
            Self {
                canned: RefCell::new(HashMap::new()),
            }
        }

        fn add(mut self, executable: &str, output: &str) -> Self {
            self.canned
                .get_mut()
                .insert(executable.to_string(), Ok(output.to_string()));
            self
        }

        fn add_not_found(mut self, executable: &str) -> Self {
            self.canned.get_mut().insert(
                executable.to_string(),
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("{executable} not on PATH"),
                )),
            );
            self
        }

        fn add_error(mut self, executable: &str, message: &str) -> Self {
            self.canned.get_mut().insert(
                executable.to_string(),
                Err(io::Error::other(message.to_string())),
            );
            self
        }
    }

    impl CommandRunner for MockRunner {
        fn run(&self, executable: &str, _args: &[&str]) -> io::Result<String> {
            // Re-create the io::Error since it isn't Clone.
            match self.canned.borrow().get(executable) {
                Some(Ok(s)) => Ok(s.clone()),
                Some(Err(e)) => Err(io::Error::new(e.kind(), e.to_string())),
                None => Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("(no canned response for {executable})"),
                )),
            }
        }
    }

    /// AC-OE-1 (REQ-OE-001): Tool::all() is the canonical 6-entry list.
    #[test]
    fn tool_all_returns_six_in_canonical_order() {
        let all = Tool::all();
        assert_eq!(all.len(), 6);
        assert_eq!(all[0], Tool::Shell);
        assert_eq!(all[1], Tool::Tmux);
        assert_eq!(all[2], Tool::Node);
        assert_eq!(all[3], Tool::Python);
        assert_eq!(all[4], Tool::Rust);
        assert_eq!(all[5], Tool::Git);
    }

    /// AC-OE-2 (REQ-OE-002): each variant exposes non-empty metadata.
    #[test]
    fn tool_metadata_is_non_empty_for_every_variant() {
        for tool in Tool::all() {
            assert!(!tool.executable().is_empty(), "{tool:?} executable empty");
            assert!(
                !tool.display_name().is_empty(),
                "{tool:?} display_name empty"
            );
            assert!(!tool.version_arg().is_empty(), "{tool:?} version_arg empty");
        }
    }

    /// AC-OE-2 (REQ-OE-002): tmux uses `-V`, others use `--version`.
    #[test]
    fn tool_version_arg_matches_each_tool_convention() {
        assert_eq!(Tool::Tmux.version_arg(), "-V");
        assert_eq!(Tool::Node.version_arg(), "--version");
        assert_eq!(Tool::Python.version_arg(), "--version");
        assert_eq!(Tool::Rust.version_arg(), "--version");
        assert_eq!(Tool::Git.version_arg(), "--version");
        assert_eq!(Tool::Shell.version_arg(), "--version");
    }

    /// AC-OE-2 (REQ-OE-002): non-shell tools have stable executables.
    #[test]
    fn tool_executable_is_stable_for_non_shell_tools() {
        assert_eq!(Tool::Tmux.executable(), "tmux");
        assert_eq!(Tool::Node.executable(), "node");
        assert_eq!(Tool::Python.executable(), "python3");
        assert_eq!(Tool::Rust.executable(), "cargo");
        assert_eq!(Tool::Git.executable(), "git");
    }

    /// AC-OE-3 (REQ-OE-003): ToolStatus 3 variants pattern-match exhaustively.
    #[test]
    fn tool_status_three_variants_exhaustive() {
        for st in [
            ToolStatus::Available {
                version: "1.0".to_string(),
            },
            ToolStatus::NotFound,
            ToolStatus::Error {
                message: "boom".to_string(),
            },
        ] {
            match &st {
                ToolStatus::Available { version } => assert!(!version.is_empty()),
                ToolStatus::NotFound => {}
                ToolStatus::Error { message } => assert!(!message.is_empty()),
            }
        }
        assert!(
            ToolStatus::Available {
                version: "x".to_string()
            }
            .is_available()
        );
        assert!(!ToolStatus::NotFound.is_available());
    }

    /// AC-OE-4 (REQ-OE-004): EnvironmentReport helpers.
    #[test]
    fn environment_report_helpers() {
        let report = EnvironmentReport::new(vec![
            (
                Tool::Shell,
                ToolStatus::Available {
                    version: "zsh 5.9".to_string(),
                },
            ),
            (
                Tool::Tmux,
                ToolStatus::Available {
                    version: "tmux 3.3a".to_string(),
                },
            ),
            (Tool::Node, ToolStatus::NotFound),
            (Tool::Python, ToolStatus::NotFound),
            (
                Tool::Rust,
                ToolStatus::Available {
                    version: "cargo 1.92".to_string(),
                },
            ),
            (
                Tool::Git,
                ToolStatus::Error {
                    message: "permission denied".to_string(),
                },
            ),
        ]);

        assert_eq!(report.available_count(), 3);
        let missing = report.missing_tools();
        assert_eq!(missing.len(), 3);
        assert!(missing.contains(&Tool::Node));
        assert!(missing.contains(&Tool::Python));
        assert!(missing.contains(&Tool::Git));
        assert!(!report.is_complete());
    }

    /// AC-OE-5 (REQ-OE-005, REQ-OE-007): all-Available mock yields complete report.
    #[test]
    fn detect_with_runner_all_available_yields_complete() {
        let shell_exec = Tool::Shell.executable();
        let runner = MockRunner::new()
            .add(&shell_exec, "zsh 5.9 (x86_64-apple-darwin23.0)")
            .add("tmux", "tmux 3.3a")
            .add("node", "v20.10.0")
            .add("python3", "Python 3.12.1")
            .add("cargo", "cargo 1.92.0")
            .add("git", "git version 2.43.0");

        let report = detect_with_runner(&runner);
        assert_eq!(report.entries.len(), 6);
        assert!(report.is_complete(), "report must be complete");
        assert_eq!(report.available_count(), 6);
        // Spot-check one entry's parsed version.
        let node_entry = report
            .entries
            .iter()
            .find(|(t, _)| *t == Tool::Node)
            .unwrap();
        match &node_entry.1 {
            ToolStatus::Available { version } => assert_eq!(version, "v20.10.0"),
            other => panic!("expected Available, got {other:?}"),
        }
    }

    /// AC-OE-6 (REQ-OE-007): mixed runner yields half-Available report.
    #[test]
    fn detect_with_runner_mixed_yields_half_complete() {
        let shell_exec = Tool::Shell.executable();
        let runner = MockRunner::new()
            .add(&shell_exec, "zsh 5.9")
            .add_not_found("tmux")
            .add("node", "v20.10.0")
            .add_not_found("python3")
            .add("cargo", "cargo 1.92.0")
            .add_not_found("git");

        let report = detect_with_runner(&runner);
        assert_eq!(report.entries.len(), 6);
        assert_eq!(report.available_count(), 3);
        let missing = report.missing_tools();
        assert_eq!(missing.len(), 3);
        assert!(missing.contains(&Tool::Tmux));
        assert!(missing.contains(&Tool::Python));
        assert!(missing.contains(&Tool::Git));
        assert!(!report.is_complete());
    }

    /// REQ-OE-007 negative: non-NotFound errors map to ToolStatus::Error.
    #[test]
    fn detect_with_runner_other_errors_map_to_status_error() {
        let shell_exec = Tool::Shell.executable();
        let runner = MockRunner::new()
            .add(&shell_exec, "zsh 5.9")
            .add("tmux", "tmux 3.3a")
            .add("node", "v20.10.0")
            .add("python3", "Python 3.12.1")
            .add("cargo", "cargo 1.92.0")
            .add_error("git", "exited with status 1");

        let report = detect_with_runner(&runner);
        let git_status = &report
            .entries
            .iter()
            .find(|(t, _)| *t == Tool::Git)
            .unwrap()
            .1;
        match git_status {
            ToolStatus::Error { message } => assert!(message.contains("exited with status 1")),
            other => panic!("expected Error, got {other:?}"),
        }
        // Other 5 tools are still Available.
        assert_eq!(report.available_count(), 5);
        assert!(!report.is_complete());
    }

    /// AC-OE-7 (REQ-OE-007): parse_version_from_stdout returns first non-empty trimmed line.
    #[test]
    fn parse_version_from_stdout_first_line_trimmed() {
        let stdout = "node v20.10.0\n  more text\n";
        assert_eq!(parse_version_from_stdout(stdout), "node v20.10.0");
    }

    /// AC-OE-7: empty / whitespace-only stdout returns "(unknown)".
    #[test]
    fn parse_version_from_stdout_empty_returns_unknown() {
        assert_eq!(parse_version_from_stdout(""), "(unknown)");
        assert_eq!(parse_version_from_stdout("\n\n   \n"), "(unknown)");
    }

    /// AC-OE-7: leading/trailing whitespace is stripped.
    #[test]
    fn parse_version_from_stdout_strips_whitespace() {
        assert_eq!(
            parse_version_from_stdout("  Python 3.12.1\nextra"),
            "Python 3.12.1"
        );
    }

    /// AC-OE-7: leading blank lines are skipped to find the first content line.
    #[test]
    fn parse_version_from_stdout_skips_leading_blank_lines() {
        let stdout = "\n\n  cargo 1.92.0  \nrest";
        assert_eq!(parse_version_from_stdout(stdout), "cargo 1.92.0");
    }
}
