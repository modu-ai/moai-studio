//! Shell registry — SPEC-V0-2-0-MULTI-SHELL-001.
//!
//! Provides a typed `Shell` enum covering 8 shell variants, along with
//! helper methods for executable names, display labels, argument lists,
//! per-platform availability lists, and runtime detection via `which`.
//!
//! # Design notes
//!
//! - `detect_available` spawns a `which <exe>` subprocess per variant on
//!   Unix.  The call is synchronous and intentionally one-shot (Command
//!   Palette activation context).  Caching is deferred to v0.2.1.
//! - Windows shell detection is stubbed (`all_windows` / Windows path of
//!   `detect_available`) — v1 is Unix-only per N1 non-goal.

// @MX:ANCHOR: [AUTO] Shell — shell registry public API.
// @MX:REASON: [AUTO] fan_in >= 3: shell_picker::ShellPicker::new, UnixPty::spawn_with_shell,
//   RootView::handle_switch_shell, test suite.
// @MX:SPEC: SPEC-V0-2-0-MULTI-SHELL-001 REQ-MS-001 ~ REQ-MS-003

use std::process::Command;

// ============================================================
// Shell enum
// ============================================================

/// Typed representation of a supported shell.
///
/// 8 variants cover macOS / Linux / Windows possibilities. v1 only
/// detects and spawns Unix shells (Pwsh included when present on PATH).
/// Cmd and Dash are defined for completeness; Cmd is Windows-only,
/// Dash is Debian/Ubuntu default `/bin/dash`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shell {
    /// Z shell (`zsh`) — macOS default.
    Zsh,
    /// Bourne-again shell (`bash`).
    Bash,
    /// Friendly interactive shell (`fish`).
    Fish,
    /// Nushell (`nu`).
    Nu,
    /// Bourne shell (`sh`).
    Sh,
    /// PowerShell Core (`pwsh`) — cross-platform.
    Pwsh,
    /// Windows Command Prompt (`cmd`) — Windows only.
    Cmd,
    /// Debian Almquist shell (`dash`) — common on Debian/Ubuntu.
    Dash,
}

impl Shell {
    /// The executable name used to launch this shell.
    ///
    /// This is the value passed to `which` for detection and to the PTY
    /// spawn call.
    pub fn executable(self) -> &'static str {
        match self {
            Shell::Zsh => "zsh",
            Shell::Bash => "bash",
            Shell::Fish => "fish",
            Shell::Nu => "nu",
            Shell::Sh => "sh",
            Shell::Pwsh => "pwsh",
            Shell::Cmd => "cmd",
            Shell::Dash => "dash",
        }
    }

    /// Human-readable display name shown in the Shell Picker UI.
    pub fn display_name(self) -> &'static str {
        match self {
            Shell::Zsh => "Zsh",
            Shell::Bash => "Bash",
            Shell::Fish => "Fish",
            Shell::Nu => "Nushell",
            Shell::Sh => "Sh",
            Shell::Pwsh => "PowerShell",
            Shell::Cmd => "Command Prompt",
            Shell::Dash => "Dash",
        }
    }

    /// Default launch arguments for this shell.
    ///
    /// Most interactive shells require no explicit arguments when spawned
    /// as a PTY child.  `fish` in particular benefits from `-i` (already
    /// the default when stdin is a TTY), but we leave the args empty to
    /// respect the user's shell configuration.
    pub fn default_args(self) -> Vec<&'static str> {
        // v1: no extra args for any variant.  Carry-over to v0.2.1 if
        // specific shells prove to need explicit flags.
        match self {
            Shell::Zsh
            | Shell::Bash
            | Shell::Fish
            | Shell::Nu
            | Shell::Sh
            | Shell::Pwsh
            | Shell::Cmd
            | Shell::Dash => vec![],
        }
    }

    // ── Platform lists ──

    /// Shells to probe on Unix (macOS + Linux).
    ///
    /// Returns 6 variants: Zsh, Bash, Fish, Nu, Sh, Pwsh.
    /// Cmd is Windows-only (omitted). Dash is Linux-specific but included
    /// here since macOS ships `/bin/dash` on some systems; `detect_available`
    /// will filter it out when `which dash` fails.
    ///
    /// @MX:NOTE: [AUTO] all_unix — canonical Unix shell ordering (most common first).
    /// @MX:SPEC: SPEC-V0-2-0-MULTI-SHELL-001 REQ-MS-002
    pub fn all_unix() -> Vec<Shell> {
        vec![
            Shell::Zsh,
            Shell::Bash,
            Shell::Fish,
            Shell::Nu,
            Shell::Sh,
            Shell::Pwsh,
        ]
    }

    /// Shells to probe on Windows (stub — v1 not implemented, N1 non-goal).
    ///
    /// Pwsh and Cmd are included in the enum definition for future use.
    /// v1 returns this list but `detect_available` on Windows returns `[]`.
    pub fn all_windows() -> Vec<Shell> {
        vec![Shell::Pwsh, Shell::Cmd]
    }

    /// Probe which shells from `all_unix()` are available on the current system.
    ///
    /// On Unix, runs `which <executable>` for each candidate and keeps those
    /// where the process exits with status 0 (shell found on PATH).
    ///
    /// On Windows (v1 stub), always returns an empty `Vec`.
    ///
    /// # Performance note
    ///
    /// Spawns one subprocess per candidate (~6 in v1).  Expect ~5 ms per
    /// call on a warm system.  Results are not cached in v1; caching is a
    /// v0.2.1 carry-over item.
    ///
    /// @MX:NOTE: [AUTO] detect_available — spawns `which` subprocesses synchronously.
    /// @MX:REASON: [AUTO] One-shot call at Command Palette activation; acceptable latency (~30 ms total).
    /// @MX:SPEC: SPEC-V0-2-0-MULTI-SHELL-001 REQ-MS-003
    pub fn detect_available() -> Vec<Shell> {
        #[cfg(unix)]
        {
            Shell::all_unix()
                .into_iter()
                .filter(|shell| {
                    Command::new("which")
                        .arg(shell.executable())
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false)
                })
                .collect()
        }
        #[cfg(not(unix))]
        {
            // v1: Windows shell detection is a N1 non-goal.
            vec![]
        }
    }
}

// ============================================================
// Unit tests — AC-MS-1 / AC-MS-2 / AC-MS-3
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    // ── T1: Shell enum + mapping methods ──

    /// AC-MS-1 (REQ-MS-001): Shell enum has exactly 8 variants.
    #[test]
    fn test_shell_enum_variants() {
        let all = [
            Shell::Zsh,
            Shell::Bash,
            Shell::Fish,
            Shell::Nu,
            Shell::Sh,
            Shell::Pwsh,
            Shell::Cmd,
            Shell::Dash,
        ];
        assert_eq!(all.len(), 8, "Shell enum must have exactly 8 variants");
        // All variants must be distinct.
        let unique: HashSet<String> = all.iter().map(|s| format!("{:?}", s)).collect();
        assert_eq!(unique.len(), 8, "all Shell variants must be distinct");
    }

    /// AC-MS-1: executable() returns a non-empty string for every variant.
    #[test]
    fn test_shell_executable_mapping() {
        let cases = [
            (Shell::Zsh, "zsh"),
            (Shell::Bash, "bash"),
            (Shell::Fish, "fish"),
            (Shell::Nu, "nu"),
            (Shell::Sh, "sh"),
            (Shell::Pwsh, "pwsh"),
            (Shell::Cmd, "cmd"),
            (Shell::Dash, "dash"),
        ];
        for (shell, expected) in &cases {
            assert_eq!(
                shell.executable(),
                *expected,
                "{:?}.executable() must be {:?}",
                shell,
                expected
            );
        }
    }

    /// AC-MS-1: display_name() returns a non-empty string for every variant.
    #[test]
    fn test_shell_display_name() {
        let shells = [
            Shell::Zsh,
            Shell::Bash,
            Shell::Fish,
            Shell::Nu,
            Shell::Sh,
            Shell::Pwsh,
            Shell::Cmd,
            Shell::Dash,
        ];
        for shell in &shells {
            let name = shell.display_name();
            assert!(
                !name.is_empty(),
                "{:?}.display_name() must not be empty",
                shell
            );
        }
        // Spot-check a few values.
        assert_eq!(Shell::Zsh.display_name(), "Zsh");
        assert_eq!(Shell::Nu.display_name(), "Nushell");
        assert_eq!(Shell::Pwsh.display_name(), "PowerShell");
        assert_eq!(Shell::Cmd.display_name(), "Command Prompt");
    }

    /// AC-MS-1: default_args() returns an empty Vec for all v1 variants.
    #[test]
    fn test_shell_default_args_empty_for_most() {
        let shells = [
            Shell::Zsh,
            Shell::Bash,
            Shell::Fish,
            Shell::Nu,
            Shell::Sh,
            Shell::Pwsh,
            Shell::Cmd,
            Shell::Dash,
        ];
        for shell in &shells {
            assert!(
                shell.default_args().is_empty(),
                "{:?}.default_args() must be empty in v1",
                shell
            );
        }
    }

    // ── T2: all_unix / all_windows ──

    /// AC-MS-2 (REQ-MS-002): all_unix() returns exactly 6 variants.
    #[test]
    fn test_shell_all_unix_returns_six_variants() {
        let unix = Shell::all_unix();
        assert_eq!(
            unix.len(),
            6,
            "all_unix() must return exactly 6 variants, got {}",
            unix.len()
        );
        // Must include the common five + pwsh.
        let ids: HashSet<String> = unix.iter().map(|s| format!("{:?}", s)).collect();
        for expected in &["Zsh", "Bash", "Fish", "Nu", "Sh", "Pwsh"] {
            assert!(
                ids.contains(*expected),
                "all_unix() must include {:?}",
                expected
            );
        }
    }

    /// all_windows() returns exactly 2 variants (Pwsh + Cmd).
    #[test]
    fn test_shell_all_windows_returns_two_variants() {
        let windows = Shell::all_windows();
        assert_eq!(
            windows.len(),
            2,
            "all_windows() must return exactly 2 variants, got {}",
            windows.len()
        );
        let ids: HashSet<String> = windows.iter().map(|s| format!("{:?}", s)).collect();
        assert!(ids.contains("Pwsh"), "all_windows() must include Pwsh");
        assert!(ids.contains("Cmd"), "all_windows() must include Cmd");
    }

    // ── T3: detect_available ──

    /// AC-MS-3 (REQ-MS-003): detect_available() finds at least one shell on Unix.
    ///
    /// CI runners (macOS / Linux) always have `sh` or `bash` on PATH.
    /// On Windows the result is an empty Vec (v1 stub), so we skip the
    /// assertion there.
    #[test]
    #[cfg(unix)]
    fn test_detect_available_returns_at_least_one_on_unix() {
        let available = Shell::detect_available();
        assert!(
            !available.is_empty(),
            "detect_available() must return at least one shell on Unix (sh / bash guaranteed)"
        );
    }

    /// detect_available() returns only shells from all_unix() on Unix.
    #[test]
    #[cfg(unix)]
    fn test_detect_available_subset_of_all_unix() {
        let available = Shell::detect_available();
        let unix_set: HashSet<String> = Shell::all_unix()
            .iter()
            .map(|s| format!("{:?}", s))
            .collect();
        for shell in &available {
            assert!(
                unix_set.contains(&format!("{:?}", shell)),
                "detect_available() returned {:?} which is not in all_unix()",
                shell
            );
        }
    }
}
