//! CmdPalette variant — Cmd+P 파일 quick open.
//!
//! @MX:NOTE: [AUTO] CmdPalette variant — mock 파일 인덱스 데이터 소스. 실제 소스는 후속 SPEC.
//! @MX:SPEC: SPEC-V3-012 MS-2 AC-PL-6

use crate::palette::fuzzy::fuzzy_match;
use crate::palette::palette_view::{PaletteEvent, PaletteItem, PaletteView};

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

/// CmdPalette — Cmd+P 파일 quick open variant.
///
/// - mock_file_index 에서 데이터 소스를 가져온다 (AC-PL-6).
/// - query 변경 시 fuzzy match 로 필터링하고 PaletteView 를 갱신한다.
/// - Enter 시 FileOpened(path) 이벤트를 발생시킨다.
/// - 활성화 keybinding: Cmd+P (데이터만 선언, 실제 설치는 MS-3).
pub struct CmdPalette {
    /// 공유 PaletteView core.
    pub view: PaletteView,
    /// 전체 파일 인덱스 (필터링 원본).
    file_index: Vec<String>,
    /// 마지막 이벤트.
    pub last_event: Option<CmdPaletteEvent>,
    /// 활성화 keybinding 정보 (MS-3 에서 실제 설치).
    pub activation_key: &'static str,
}

impl CmdPalette {
    /// 기본 mock 파일 인덱스로 새 CmdPalette 를 생성한다.
    pub fn new() -> Self {
        let file_index: Vec<String> = MOCK_FILE_INDEX.iter().map(|s| s.to_string()).collect();
        let items = files_to_items(&file_index);
        Self {
            view: PaletteView::with_items(items),
            file_index,
            last_event: None,
            activation_key: "cmd+p",
        }
    }

    /// 커스텀 파일 인덱스로 CmdPalette 를 생성한다 (테스트용).
    pub fn with_file_index(files: Vec<String>) -> Self {
        let items = files_to_items(&files);
        Self {
            view: PaletteView::with_items(items),
            file_index: files,
            last_event: None,
            activation_key: "cmd+p",
        }
    }

    /// query 를 갱신하고 파일 인덱스를 fuzzy 필터링한다 (AC-PL-6).
    ///
    /// 빈 쿼리면 전체 목록 표시. 필터링 후 선택 index 는 0 으로 초기화.
    pub fn set_query(&mut self, query: String) {
        self.view.set_query(query.clone());
        let filtered = filter_files(&self.file_index, &query);
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
    // 점수 내림차순 정렬.
    results.sort_by(|a, b| b.0.cmp(&a.0));
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
}
