//! SPEC-V0-3-0-WORKSPACE-COLOR-001 — Workspace color preset palette.
//!
//! Provides a 12-preset color palette for workspace tags. New workspaces
//! receive a round-robin color from this palette via [`next_color`].
//! ColorPickerModal logic-level state lives in `crate::workspace_menu`.

/// 12-preset color palette for workspace tags (Radix Colors 600 step).
///
/// Each entry is a packed `0xRRGGBB` u32 compatible with `gpui::rgb`.
/// Hue is distributed across the wheel for visual distinction; saturation
/// is harmonized to avoid clashing with the dark/light brand chrome.
pub const WORKSPACE_COLOR_PALETTE: [u32; 12] = [
    0xE5484D, // red
    0xF76808, // orange
    0xFFB224, // amber
    0x30A46C, // grass / green
    0x12A594, // teal
    0x05A2C2, // cyan
    0x0091FF, // blue
    0x3E63DD, // indigo
    0x8E4EC6, // purple
    0xD6409F, // pink
    0xE93D82, // crimson
    0x687076, // slate / gray
];

/// SPEC-V0-3-0-WORKSPACE-COLOR-001 (REQ-WC-003): Round-robin color picker.
///
/// Returns the palette entry at `existing_count % 12`. Used by
/// `RootView::handle_add_workspace` so each new workspace gets a distinct
/// color until the palette wraps.
pub fn next_color(existing_count: usize) -> u32 {
    WORKSPACE_COLOR_PALETTE[existing_count % WORKSPACE_COLOR_PALETTE.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// AC-WC-1: palette length is 12 and all entries are distinct u32 values.
    #[test]
    fn workspace_color_palette_has_twelve_distinct_entries() {
        assert_eq!(WORKSPACE_COLOR_PALETTE.len(), 12);
        let unique: HashSet<u32> = WORKSPACE_COLOR_PALETTE.iter().copied().collect();
        assert_eq!(unique.len(), 12, "all palette entries must be distinct");
    }

    /// AC-WC-2: next_color round-robins through the palette using modulo.
    #[test]
    fn next_color_round_robins_through_palette() {
        assert_eq!(next_color(0), WORKSPACE_COLOR_PALETTE[0]);
        assert_eq!(next_color(1), WORKSPACE_COLOR_PALETTE[1]);
        assert_eq!(next_color(11), WORKSPACE_COLOR_PALETTE[11]);
        assert_eq!(next_color(12), WORKSPACE_COLOR_PALETTE[0], "wraps after 12");
        assert_eq!(next_color(13), WORKSPACE_COLOR_PALETTE[1]);
        assert_eq!(next_color(24), WORKSPACE_COLOR_PALETTE[0]);
    }
}
