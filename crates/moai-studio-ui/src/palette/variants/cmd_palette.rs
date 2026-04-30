//! CmdPalette variant — Cmd+P 파일 quick open + @mention / # issue mode (MS-4).
//!
//! @MX:NOTE: [AUTO] CmdPalette variant — file source (F-1) + @symbol / #issue mode switching (MS-4).
//! @MX:SPEC: SPEC-V3-012 MS-2 AC-PL-6, MS-4 AC-PL-20

use crate::palette::fuzzy::fuzzy_match;
use crate::palette::palette_view::{PaletteEvent, PaletteItem, PaletteView};
use std::path::Path;

// ============================================================
// mock 파일 인덱스 (AC-PL-6: 5+ 항목)
// ============================================================

/// CmdPalette 기본 mock 파일 인덱스.
///
/// N1 (Non-Goal): 실제 파일 인덱스 데이터 레이어는 후속 SPEC 에서 구현.
pub const MOCK_FILE_INDEX: &[&str] = &[
    "src/main.rs",
    "src/lib.rs",
    "src/palette/mod.rs",
    "src/palette/fuzzy.rs",
    "src/palette/palette_view.rs",
    "src/palette/scrim.rs",
    "Cargo.toml",
    "README.md",
];

// ============================================================
// PaletteMode — @mention / # issue mode switching (MS-4 AC-PL-20)
// ============================================================

/// Active search mode for CmdPalette.
///
/// Determined by the leading character of the query:
/// - No prefix or `@` not at start → `File` (default)
/// - `@` prefix → `Symbol` (mock data for now)
/// - `#` prefix → `Issue` (mock data for now)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteMode {
    /// File quick-open (default, Cmd+P).
    File,
    /// Symbol search mode (triggered by leading `@`).
    ///
    /// Real symbol data is a Non-Goal for MS-4; mock list is used.
    Symbol,
    /// Issue search mode (triggered by leading `#`).
    ///
    /// Real issue data is a Non-Goal for MS-4; mock list is used.
    Issue,
}

impl PaletteMode {
    /// Detect mode from raw query string.
    pub fn detect(query: &str) -> Self {
        if query.starts_with('@') {
            PaletteMode::Symbol
        } else if query.starts_with('#') {
            PaletteMode::Issue
        } else {
            PaletteMode::File
        }
    }

    /// Strip the mode prefix from query for downstream filtering.
    pub fn strip_prefix<'a>(&self, query: &'a str) -> &'a str {
        match self {
            PaletteMode::Symbol | PaletteMode::Issue => query.get(1..).unwrap_or(""),
            PaletteMode::File => query,
        }
    }
}

// ============================================================
// Mock symbol / issue data (MS-4; real source is N-Goal)
// ============================================================

/// Mock symbol list for @mention mode.
///
/// Real symbol index integration is a Non-Goal for MS-4.
const MOCK_SYMBOLS: &[(&str, &str)] = &[
    ("symbol.RootView", "RootView (struct)"),
    ("symbol.PaletteView", "PaletteView (struct)"),
    ("symbol.CommandRegistry", "CommandRegistry (struct)"),
    ("symbol.TabContainer", "TabContainer (struct)"),
    ("symbol.TerminalSurface", "TerminalSurface (struct)"),
];

/// Mock issue list for # issue mode.
///
/// Real issue integration is a Non-Goal for MS-4.
const MOCK_ISSUES: &[(&str, &str)] = &[
    ("issue.1", "#1 — Initial scaffold"),
    ("issue.2", "#2 — Palette Surface"),
    ("issue.3", "#3 — Terminal integration"),
];

// ============================================================
// CmdPalette 이벤트
// ============================================================

/// CmdPalette 에서 발생하는 이벤트.
#[derive(Debug, Clone, PartialEq)]
pub enum CmdPaletteEvent {
    /// 파일 선택 확정 — Enter 시 선택된 파일 경로.
    FileOpened(String),
    /// Palette 닫기 요청.
    DismissRequested,
}

// ============================================================
// CmdPalette — 파일 quick open variant
// ============================================================

/// CmdPalette — Cmd+P 파일 quick open variant with @mention / # issue mode (MS-4).
///
/// - file_index for File mode (default); mock symbols for @; mock issues for #.
/// - Mode is detected automatically from query prefix.
/// - On Enter: emits FileOpened(path) in File mode; Symbol/Issue modes also emit FileOpened(id).
/// - Activation keybinding: Cmd+P.
pub struct CmdPalette {
    /// 공유 PaletteView core.
    pub view: PaletteView,
    /// 전체 파일 인덱스 (필터링 원본).
    file_index: Vec<String>,
    /// 마지막 이벤트.
    pub last_event: Option<CmdPaletteEvent>,
    /// 활성화 keybinding 정보 (MS-3 에서 실제 설치).
    pub activation_key: &'static str,
    /// Current active mode (File / Symbol / Issue).
    pub mode: PaletteMode,
}

impl CmdPalette {
    /// Create CmdPalette with the default mock file index.
    pub fn new() -> Self {
        let file_index: Vec<String> = MOCK_FILE_INDEX.iter().map(|s| s.to_string()).collect();
        let items = files_to_items(&file_index);
        Self {
            view: PaletteView::with_items(items),
            file_index,
            last_event: None,
            activation_key: "cmd+p",
            mode: PaletteMode::File,
        }
    }

    /// Build a CmdPalette populated from a real workspace directory (F-1).
    ///
    /// Walks `workspace_dir` recursively and collects relative file paths
    /// (excluding hidden files and common build artefacts). If the directory
    /// does not exist or the walk produces no results, falls back to
    /// `MOCK_FILE_INDEX` so the palette is never empty.
    ///
    /// File paths are returned as forward-slash-separated strings relative to
    /// `workspace_dir`, suitable for display and fuzzy matching.
    pub fn from_workspace_dir(workspace_dir: &Path) -> Self {
        let file_index = scan_workspace_files(workspace_dir);
        let file_index = if file_index.is_empty() {
            // Fallback to mock when dir is missing or empty.
            MOCK_FILE_INDEX.iter().map(|s| s.to_string()).collect()
        } else {
            file_index
        };
        let items = files_to_items(&file_index);
        Self {
            view: PaletteView::with_items(items),
            file_index,
            last_event: None,
            activation_key: "cmd+p",
            mode: PaletteMode::File,
        }
    }

    /// Create CmdPalette with custom file index (for tests).
    pub fn with_file_index(files: Vec<String>) -> Self {
        let items = files_to_items(&files);
        Self {
            view: PaletteView::with_items(items),
            file_index: files,
            last_event: None,
            activation_key: "cmd+p",
            mode: PaletteMode::File,
        }
    }

    /// Update query, detect mode from prefix, and re-filter the appropriate list.
    ///
    /// Mode switching (MS-4 AC-PL-20):
    /// - `@...` → Symbol mode with mock symbol list
    /// - `#...` → Issue mode with mock issue list
    /// - Otherwise → File mode (default)
    ///
    /// On mode switch, the visible list is cleared and replaced with the
    /// mode-appropriate data source.
    pub fn set_query(&mut self, query: String) {
        let new_mode = PaletteMode::detect(&query);
        if new_mode != self.mode {
            self.mode = new_mode;
        }
        self.view.set_query(query.clone());
        let sub_query = self.mode.strip_prefix(&query);
        let filtered = match self.mode {
            PaletteMode::File => filter_files(&self.file_index, &query),
            PaletteMode::Symbol => filter_static_pairs(MOCK_SYMBOLS, sub_query),
            PaletteMode::Issue => filter_static_pairs(MOCK_ISSUES, sub_query),
        };
        self.view.set_items(filtered);
    }

    /// 현재 query 로 필터링된 항목 수를 반환한다.
    pub fn filtered_count(&self) -> usize {
        self.view.items.len()
    }

    /// Enter 키 처리 — FileOpened 이벤트 발생 (AC-PL-6).
    pub fn on_enter(&mut self) -> Option<CmdPaletteEvent> {
        let event = match self.view.on_enter() {
            Some(PaletteEvent::ItemSelected(item)) => {
                Some(CmdPaletteEvent::FileOpened(item.id.clone()))
            }
            _ => None,
        };
        self.last_event = event.clone();
        event
    }

    /// Esc 키 처리 — DismissRequested 이벤트 발생.
    pub fn on_escape(&mut self) -> CmdPaletteEvent {
        self.view.on_escape();
        let event = CmdPaletteEvent::DismissRequested;
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

impl Default for CmdPalette {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 헬퍼
// ============================================================

/// Recursively scan `root` and return relative paths for non-hidden, non-artefact files.
///
/// Hidden entries (names starting with '.'), build artefacts (target/, node_modules/,
/// .git/), and directories themselves are excluded. Paths are separated with '/' on
/// all platforms for consistent display and fuzzy matching.
///
/// Depth is capped at 8 levels to avoid extremely large repositories causing
/// visible latency. For production use, real file-index integration (SPEC-V3-012 N1)
/// should replace this synchronous walk.
fn scan_workspace_files(root: &Path) -> Vec<String> {
    const MAX_DEPTH: usize = 8;
    const MAX_FILES: usize = 2000;

    // Directories to skip during traversal.
    let skip_dirs: &[&str] = &[
        "target",
        "node_modules",
        ".git",
        ".cargo",
        "dist",
        "build",
        ".next",
        "__pycache__",
    ];

    let mut files: Vec<String> = Vec::new();
    walk_dir(root, root, 0, MAX_DEPTH, skip_dirs, &mut files, MAX_FILES);
    files.sort();
    files
}

/// Recursive directory walker used by `scan_workspace_files`.
fn walk_dir(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    skip_dirs: &[&str],
    out: &mut Vec<String>,
    max_files: usize,
) {
    if depth > max_depth || out.len() >= max_files {
        return;
    }
    let read_dir = match std::fs::read_dir(current) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        if out.len() >= max_files {
            break;
        }
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        // Skip hidden entries.
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            // Skip known artefact directories.
            if skip_dirs.contains(&name) {
                continue;
            }
            walk_dir(root, &path, depth + 1, max_depth, skip_dirs, out, max_files);
        } else {
            // Build a relative path string with forward slashes.
            if let Ok(rel) = path.strip_prefix(root) {
                let rel_str = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push(rel_str);
            }
        }
    }
}

/// Filter a static `(id, label)` pair slice for Symbol or Issue mode.
fn filter_static_pairs(pairs: &[(&str, &str)], query: &str) -> Vec<PaletteItem> {
    if query.is_empty() {
        return pairs
            .iter()
            .map(|(id, label)| PaletteItem::new(id.to_string(), label.to_string()))
            .collect();
    }
    let mut results: Vec<(i32, PaletteItem)> = pairs
        .iter()
        .filter_map(|(id, label)| {
            fuzzy_match(query, label)
                .map(|(score, _)| (score, PaletteItem::new(id.to_string(), label.to_string())))
        })
        .collect();
    results.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    results.into_iter().map(|(_, item)| item).collect()
}

fn files_to_items(files: &[String]) -> Vec<PaletteItem> {
    files
        .iter()
        .map(|f| PaletteItem::new(f.clone(), f.clone()))
        .collect()
}

fn filter_files(files: &[String], query: &str) -> Vec<PaletteItem> {
    if query.is_empty() {
        return files_to_items(files);
    }
    let mut results: Vec<(i32, PaletteItem)> = files
        .iter()
        .filter_map(|f| {
            fuzzy_match(query, f).map(|(score, _)| (score, PaletteItem::new(f.clone(), f.clone())))
        })
        .collect();
    // Sort by score descending.
    results.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    results.into_iter().map(|(_, item)| item).collect()
}

// ============================================================
// 단위 테스트 — AC-PL-6
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_palette_with_files(files: &[&str]) -> CmdPalette {
        CmdPalette::with_file_index(files.iter().map(|s| s.to_string()).collect())
    }

    /// AC-PL-6: 초기 상태에서 전체 목록이 표시된다.
    #[test]
    fn initial_shows_all_files() {
        let palette = make_palette_with_files(&["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"]);
        assert_eq!(palette.filtered_count(), 5, "초기 상태 전체 파일 표시");
    }

    /// AC-PL-6: query 로 필터링 시 매칭 subset 만 표시되고 index 가 0 이다.
    #[test]
    fn filters_by_query() {
        let mut palette = make_palette_with_files(&[
            "src/main.rs",
            "src/lib.rs",
            "src/palette/mod.rs",
            "src/palette/fuzzy.rs",
            "Cargo.toml",
            "README.md",
        ]);
        palette.set_query("palette".to_string());
        let count = palette.filtered_count();
        assert!(
            count > 0 && count < 6,
            "palette 쿼리는 일부 항목만 필터링해야 함, got {count}"
        );
        // 선택 index 는 0 이어야 함.
        assert_eq!(
            palette.view.nav.selected_index,
            Some(0),
            "필터 후 선택 index 는 0"
        );
        // 모든 결과에 "palette" 가 포함되어 있어야 함.
        for item in &palette.view.items {
            assert!(
                item.label.contains("palette"),
                "필터 결과에 'palette' 가 없는 항목: {}",
                item.label
            );
        }
    }

    /// 매칭 없는 쿼리 → 빈 목록.
    #[test]
    fn no_match_query_returns_empty() {
        let mut palette = make_palette_with_files(&["alpha.rs", "beta.rs"]);
        palette.set_query("xyz_no_match".to_string());
        assert_eq!(palette.filtered_count(), 0, "매칭 없으면 빈 목록");
    }

    /// Enter 시 FileOpened 이벤트에 올바른 경로가 담긴다.
    #[test]
    fn enter_emits_file_opened() {
        let mut palette = make_palette_with_files(&["src/main.rs", "src/lib.rs"]);
        // 첫 번째 항목이 선택된 상태에서 Enter.
        let event = palette.on_enter();
        assert!(event.is_some(), "Enter 시 이벤트 발생해야 함");
        if let Some(CmdPaletteEvent::FileOpened(path)) = event {
            assert!(!path.is_empty(), "파일 경로가 비어있으면 안 됨");
        } else {
            panic!("FileOpened 이벤트가 아님");
        }
    }

    /// Esc 시 DismissRequested 이벤트 발생.
    #[test]
    fn escape_emits_dismiss() {
        let mut palette = make_palette_with_files(&["a.rs"]);
        let event = palette.on_escape();
        assert_eq!(event, CmdPaletteEvent::DismissRequested);
    }

    /// 빈 쿼리로 초기화하면 전체 목록 복원.
    #[test]
    fn empty_query_restores_all() {
        let mut palette = make_palette_with_files(&["a.rs", "b.rs", "c.rs"]);
        palette.set_query("a".to_string());
        palette.set_query("".to_string());
        assert_eq!(palette.filtered_count(), 3, "빈 쿼리 → 전체 목록 복원");
    }

    /// activation_key 가 "cmd+p" 이다.
    #[test]
    fn activation_key_is_cmd_p() {
        let palette = CmdPalette::new();
        assert_eq!(palette.activation_key, "cmd+p");
    }

    /// ArrowDown/Up 이 view 로 위임된다.
    #[test]
    fn arrow_keys_delegate_to_view() {
        let mut palette = make_palette_with_files(&["a.rs", "b.rs", "c.rs"]);
        assert_eq!(palette.view.nav.selected_index, Some(0));
        palette.on_arrow_down();
        assert_eq!(palette.view.nav.selected_index, Some(1));
        palette.on_arrow_up();
        assert_eq!(palette.view.nav.selected_index, Some(0));
    }

    // ── F-1: from_workspace_dir tests ──

    /// F-1: from_workspace_dir with existing dir returns a non-empty file list.
    #[test]
    fn from_workspace_dir_scans_files() {
        use std::fs;
        let tmp = std::env::temp_dir().join("moai_palette_test_scan");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("create tmp dir");
        fs::write(tmp.join("main.rs"), "fn main() {}").expect("write file");
        fs::write(tmp.join("lib.rs"), "").expect("write file");
        fs::create_dir_all(tmp.join("src")).expect("create src dir");
        fs::write(tmp.join("src/mod.rs"), "").expect("write nested file");

        let palette = CmdPalette::from_workspace_dir(&tmp);
        let count = palette.filtered_count();
        assert!(count >= 3, "should scan at least 3 files, got {count}");

        // All listed paths should be relative strings (no absolute prefix).
        for item in &palette.view.items {
            assert!(
                !item.label.starts_with('/'),
                "paths should be relative, got: {}",
                item.label
            );
        }
        let _ = fs::remove_dir_all(&tmp);
    }

    /// F-1: from_workspace_dir on non-existent path falls back to mock index.
    #[test]
    fn from_workspace_dir_missing_dir_falls_back_to_mock() {
        let nonexistent = std::path::PathBuf::from("/tmp/moai_does_not_exist_xyz_12345");
        let palette = CmdPalette::from_workspace_dir(&nonexistent);
        // Falls back to MOCK_FILE_INDEX — must have items.
        assert!(
            palette.filtered_count() > 0,
            "fallback should produce non-empty list"
        );
    }

    /// F-1: from_workspace_dir result is fuzzy-searchable.
    #[test]
    fn from_workspace_dir_fuzzy_search_works() {
        use std::fs;
        let tmp = std::env::temp_dir().join("moai_palette_test_fuzzy");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).expect("create tmp dir");
        fs::write(tmp.join("main.rs"), "").expect("write main.rs");
        fs::write(tmp.join("palette.rs"), "").expect("write palette.rs");

        let mut palette = CmdPalette::from_workspace_dir(&tmp);
        palette.set_query("palette".to_string());
        assert!(
            palette.filtered_count() > 0,
            "fuzzy search 'palette' must match palette.rs"
        );
        let has_palette_rs = palette
            .view
            .items
            .iter()
            .any(|i| i.label.contains("palette"));
        assert!(
            has_palette_rs,
            "palette.rs should appear in filtered results"
        );
        let _ = fs::remove_dir_all(&tmp);
    }

    // ── MS-4: @mention / # mode switching tests (AC-PL-20) ──

    /// AC-PL-20: initial mode is File.
    #[test]
    fn initial_mode_is_file() {
        let palette = CmdPalette::new();
        assert_eq!(palette.mode, PaletteMode::File);
    }

    /// AC-PL-20: query starting with '@' switches to Symbol mode.
    #[test]
    fn at_prefix_switches_to_symbol_mode() {
        let mut palette = make_palette_with_files(&["a.rs", "b.rs"]);
        palette.set_query("@".to_string());
        assert_eq!(
            palette.mode,
            PaletteMode::Symbol,
            "@ prefix must switch to Symbol mode"
        );
    }

    /// AC-PL-20: query starting with '#' switches to Issue mode.
    #[test]
    fn hash_prefix_switches_to_issue_mode() {
        let mut palette = make_palette_with_files(&["a.rs"]);
        palette.set_query("#".to_string());
        assert_eq!(
            palette.mode,
            PaletteMode::Issue,
            "# prefix must switch to Issue mode"
        );
    }

    /// Symbol mode shows mock symbols, not file list.
    #[test]
    fn symbol_mode_shows_mock_symbols() {
        let mut palette = make_palette_with_files(&["a.rs", "b.rs"]);
        palette.set_query("@".to_string());
        assert_eq!(palette.mode, PaletteMode::Symbol);
        // Should show mock symbols (not file list)
        assert!(
            palette.filtered_count() > 0,
            "Symbol mode must show mock symbols"
        );
        // Symbols contain 'struct' or known symbol names — not file names
        let has_struct = palette
            .view
            .items
            .iter()
            .any(|i| i.label.contains("struct") || i.label.contains("RootView"));
        assert!(has_struct, "Symbol mode must show struct symbols");
    }

    /// Issue mode shows mock issues, not file list.
    #[test]
    fn issue_mode_shows_mock_issues() {
        let mut palette = make_palette_with_files(&["a.rs"]);
        palette.set_query("#".to_string());
        assert_eq!(palette.mode, PaletteMode::Issue);
        assert!(
            palette.filtered_count() > 0,
            "Issue mode must show mock issues"
        );
        let has_issue = palette.view.items.iter().any(|i| i.label.starts_with('#'));
        assert!(
            has_issue,
            "Issue mode must show issue labels starting with #"
        );
    }

    /// Returning to empty query (no prefix) reverts to File mode.
    #[test]
    fn empty_query_reverts_to_file_mode() {
        let mut palette = make_palette_with_files(&["a.rs", "b.rs"]);
        palette.set_query("@symbol".to_string());
        assert_eq!(palette.mode, PaletteMode::Symbol);
        palette.set_query("".to_string());
        assert_eq!(
            palette.mode,
            PaletteMode::File,
            "empty query must revert to File mode"
        );
        // File list is restored
        assert_eq!(palette.filtered_count(), 2, "file list must be restored");
    }

    /// Symbol mode filters by sub-query after '@'.
    #[test]
    fn symbol_mode_filters_by_sub_query() {
        let mut palette = make_palette_with_files(&["a.rs"]);
        // "@Root" should filter symbols containing "Root"
        palette.set_query("@Root".to_string());
        assert_eq!(palette.mode, PaletteMode::Symbol);
        let has_root = palette.view.items.iter().any(|i| i.label.contains("Root"));
        assert!(has_root, "@Root must match RootView symbol");
    }

    /// PaletteMode::detect works correctly.
    #[test]
    fn palette_mode_detect() {
        assert_eq!(PaletteMode::detect(""), PaletteMode::File);
        assert_eq!(PaletteMode::detect("main.rs"), PaletteMode::File);
        assert_eq!(PaletteMode::detect("@Root"), PaletteMode::Symbol);
        assert_eq!(PaletteMode::detect("#1"), PaletteMode::Issue);
        assert_eq!(PaletteMode::detect("@"), PaletteMode::Symbol);
        assert_eq!(PaletteMode::detect("#"), PaletteMode::Issue);
    }

    /// PaletteMode::strip_prefix strips leading @ or # correctly.
    #[test]
    fn palette_mode_strip_prefix() {
        assert_eq!(PaletteMode::Symbol.strip_prefix("@Root"), "Root");
        assert_eq!(PaletteMode::Issue.strip_prefix("#1"), "1");
        assert_eq!(PaletteMode::File.strip_prefix("main.rs"), "main.rs");
        assert_eq!(PaletteMode::Symbol.strip_prefix("@"), "");
    }
}
