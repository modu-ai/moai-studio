//! SPEC-V3-006 MS-3a: LSP Hover Tooltip (mock diagnostic provider).
//!
//! AC-MV-4 의 일부 — 실제 rust-analyzer spawn 은 MS-3b 로 deferral.
//!
//! MS-3a 에서는 **mock diagnostic provider** 만 구현한다:
//! - `LspDiagnosticProvider` trait: CodeViewer 가 진단 데이터를 얻는 추상 인터페이스
//! - `MockLspProvider`: 테스트용 seed 데이터 제공자
//! - `Diagnostic` struct: 진단 메시지 + severity + position
//! - Tooltip 데이터 구조
//! - AC-MV-5 graceful degradation 로직: provider unavailable → banner 표시
//!
//! ## MS-3b 마이그레이션 훅
//!
//! `LspDiagnosticProvider` trait 을 구현하는 `RealLspProvider` 를 `lsp/` 모듈로 추가하면
//! `CodeViewer::lsp_provider` 필드만 교체하면 된다. Trait 시그니처는 변경하지 않는다.

// @MX:ANCHOR: [AUTO] lsp-diagnostic-provider-trait
// @MX:REASON: [AUTO] SPEC-V3-006 MS-3a AC-MV-4/5. LspDiagnosticProvider 는
//   mock (MS-3a) 과 real (MS-3b) 구현의 공통 인터페이스다.
//   fan_in >= 3: CodeViewer::load, MockLspProvider (테스트), RealLspProvider (MS-3b).

// ============================================================
// DiagnosticSeverity
// ============================================================

/// LSP 진단 심각도 (REQ-MV-045).
///
/// LSP spec 의 `DiagnosticSeverity` 와 동형.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Error — 빨간 squiggly
    Error,
    /// Warning — 노란/주황 squiggly
    Warning,
    /// Information — 파란 squiggly
    Information,
    /// Hint — 회색 squiggly
    Hint,
}

impl DiagnosticSeverity {
    /// severity 에 따른 RGBA 색상 u32 를 반환한다 (design tokens 기반).
    ///
    /// | severity    | color     | token                   |
    /// |-------------|-----------|-------------------------|
    /// | Error       | #c44a3a   | semantic::DANGER        |
    /// | Warning     | #c47b2a   | semantic::WARNING       |
    /// | Information | #4080d0   | (blue, info variant)    |
    /// | Hint        | #888888   | (gray, muted)           |
    pub fn color_u32(self) -> u32 {
        match self {
            DiagnosticSeverity::Error => 0xc4_4a_3a,
            DiagnosticSeverity::Warning => 0xc4_7b_2a,
            DiagnosticSeverity::Information => 0x40_80_d0,
            DiagnosticSeverity::Hint => 0x88_88_88,
        }
    }
}

// ============================================================
// DiagnosticPosition
// ============================================================

/// 진단 위치 (줄 + 컬럼, 0-indexed).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticPosition {
    pub line: usize,
    pub column: usize,
}

// ============================================================
// DiagnosticRange
// ============================================================

/// 진단이 적용되는 범위 (start..end).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticRange {
    pub start: DiagnosticPosition,
    pub end: DiagnosticPosition,
}

// ============================================================
// Diagnostic
// ============================================================

/// LSP 진단 항목 (REQ-MV-040 ~ MV-046).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// 진단 메시지
    pub message: String,
    /// 심각도
    pub severity: DiagnosticSeverity,
    /// 진단 위치 범위 (줄/컬럼, 0-indexed)
    pub range: DiagnosticRange,
    /// 진단 출처 (예: "rust-analyzer", "gopls")
    pub source: Option<String>,
}

// ============================================================
// TooltipData
// ============================================================

/// Hover tooltip 표시에 필요한 데이터.
///
/// CodeViewer 의 Render 레이어가 이 구조체를 받아 tooltip 을 그린다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooltipData {
    /// 진단 메시지
    pub message: String,
    /// 심각도 아이콘/색상 결정용
    pub severity: DiagnosticSeverity,
    /// 출처 레이블 (없으면 공백)
    pub source: String,
}

impl TooltipData {
    /// `Diagnostic` 에서 변환한다.
    pub fn from_diagnostic(d: &Diagnostic) -> Self {
        Self {
            message: d.message.clone(),
            severity: d.severity,
            source: d.source.clone().unwrap_or_default(),
        }
    }

    /// 심각도 아이콘 문자열 반환 (접근성: 색상 외 형태 식별 — NFR-MV-10).
    pub fn severity_icon(&self) -> &'static str {
        match self.severity {
            DiagnosticSeverity::Error => "✖",
            DiagnosticSeverity::Warning => "⚠",
            DiagnosticSeverity::Information => "ℹ",
            DiagnosticSeverity::Hint => "💡",
        }
    }
}

// ============================================================
// LspDiagnosticProvider trait
// ============================================================

/// LSP 진단 제공자 추상 인터페이스.
///
/// MS-3a: `MockLspProvider` 가 이 trait 을 구현한다.
/// MS-3b: `RealLspProvider` (async-lsp 기반) 가 이 trait 을 구현한다.
pub trait LspDiagnosticProvider {
    /// 주어진 (line, column) 위치에서 진단 목록을 반환한다.
    ///
    /// 반환값이 비어있으면 tooltip 없음.
    fn diagnostics_at(&self, line: usize, column: usize) -> Vec<Diagnostic>;

    /// 파일 전체의 진단 목록을 반환한다.
    fn all_diagnostics(&self) -> Vec<Diagnostic>;

    /// LSP provider 가 현재 사용 가능한지 여부.
    ///
    /// false 이면 CodeViewer 는 "LSP unavailable" 배너를 표시하고
    /// syntax highlight 만으로 동작한다 (AC-MV-5, REQ-MV-043).
    fn is_available(&self) -> bool;
}

// ============================================================
// MockLspProvider
// ============================================================

/// 테스트 및 MS-3a 개발 중 사용하는 mock 진단 제공자.
///
/// 생성 시 진단 목록을 seed 하고, `diagnostics_at` 은 위치가 겹치는 진단을 반환한다.
pub struct MockLspProvider {
    diagnostics: Vec<Diagnostic>,
    available: bool,
}

impl MockLspProvider {
    /// 빈 진단 목록으로 생성 (가용 상태).
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            available: true,
        }
    }

    /// seed 진단 목록으로 생성한다.
    pub fn with_diagnostics(diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            diagnostics,
            available: true,
        }
    }

    /// unavailable 상태로 생성 (AC-MV-5 graceful degradation 테스트).
    pub fn unavailable() -> Self {
        Self {
            diagnostics: Vec::new(),
            available: false,
        }
    }
}

impl Default for MockLspProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LspDiagnosticProvider for MockLspProvider {
    fn diagnostics_at(&self, line: usize, column: usize) -> Vec<Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| {
                let s = &d.range.start;
                let e = &d.range.end;
                // 범위 포함 여부: line 이 범위 안에 있고 column 이 start..end 안에 있어야 함
                if s.line == e.line {
                    line == s.line && column >= s.column && column < e.column
                } else {
                    (line > s.line && line < e.line)
                        || (line == s.line && column >= s.column)
                        || (line == e.line && column < e.column)
                }
            })
            .cloned()
            .collect()
    }

    fn all_diagnostics(&self) -> Vec<Diagnostic> {
        self.diagnostics.clone()
    }

    fn is_available(&self) -> bool {
        self.available
    }
}

// ============================================================
// LspBannerState
// ============================================================

/// LSP unavailable 배너 표시 상태 (AC-MV-5, REQ-MV-043).
///
/// 최초 unavailable 감지 시 배너를 1회만 표시한다.
#[derive(Debug, Clone, Default)]
pub struct LspBannerState {
    /// "LSP unavailable: {server}" 배너 문자열. None = 표시 불필요.
    pub message: Option<String>,
    /// 이미 배너를 표시했는지 여부 (1회 제한)
    pub shown: bool,
}

impl LspBannerState {
    /// provider 가 unavailable 이면 배너를 설정한다 (1회만).
    pub fn update<P: LspDiagnosticProvider>(&mut self, provider: &P, server_name: &str) {
        if !provider.is_available() && !self.shown {
            self.message = Some(format!("LSP 미설치: {}", server_name));
            self.shown = true;
        }
    }

    /// 배너 메시지가 표시 대기 중인지 확인한다.
    pub fn has_banner(&self) -> bool {
        self.message.is_some()
    }

    /// 배너를 닫는다 (dismiss).
    pub fn dismiss(&mut self) {
        self.message = None;
    }
}

// ============================================================
// 단위 테스트 (MS-3a TDD — RED → GREEN)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ── T1: DiagnosticSeverity 색상 ──

    #[test]
    fn diagnostic_severity_error_is_red() {
        assert_eq!(DiagnosticSeverity::Error.color_u32(), 0xc44a3a);
    }

    #[test]
    fn diagnostic_severity_warning_is_orange() {
        assert_eq!(DiagnosticSeverity::Warning.color_u32(), 0xc47b2a);
    }

    #[test]
    fn diagnostic_severity_info_is_blue() {
        assert_eq!(DiagnosticSeverity::Information.color_u32(), 0x4080d0);
    }

    #[test]
    fn diagnostic_severity_hint_is_gray() {
        assert_eq!(DiagnosticSeverity::Hint.color_u32(), 0x888888);
    }

    // ── T2: TooltipData 변환 ──

    #[test]
    fn tooltip_data_from_diagnostic_copies_fields() {
        let diag = Diagnostic {
            message: "undefined variable `x`".to_string(),
            severity: DiagnosticSeverity::Error,
            range: DiagnosticRange {
                start: DiagnosticPosition { line: 5, column: 4 },
                end: DiagnosticPosition { line: 5, column: 5 },
            },
            source: Some("rust-analyzer".to_string()),
        };
        let tip = TooltipData::from_diagnostic(&diag);
        assert_eq!(tip.message, "undefined variable `x`");
        assert_eq!(tip.severity, DiagnosticSeverity::Error);
        assert_eq!(tip.source, "rust-analyzer");
    }

    #[test]
    fn tooltip_data_from_diagnostic_no_source_is_empty() {
        let diag = Diagnostic {
            message: "msg".to_string(),
            severity: DiagnosticSeverity::Warning,
            range: DiagnosticRange {
                start: DiagnosticPosition { line: 0, column: 0 },
                end: DiagnosticPosition { line: 0, column: 1 },
            },
            source: None,
        };
        let tip = TooltipData::from_diagnostic(&diag);
        assert_eq!(tip.source, "");
    }

    #[test]
    fn tooltip_data_severity_icon_matches_severity() {
        let tip_error = TooltipData {
            message: String::new(),
            severity: DiagnosticSeverity::Error,
            source: String::new(),
        };
        assert_eq!(tip_error.severity_icon(), "✖");

        let tip_warn = TooltipData {
            severity: DiagnosticSeverity::Warning,
            ..tip_error.clone()
        };
        assert_eq!(tip_warn.severity_icon(), "⚠");

        let tip_info = TooltipData {
            severity: DiagnosticSeverity::Information,
            ..tip_error.clone()
        };
        assert_eq!(tip_info.severity_icon(), "ℹ");
    }

    // ── T3: MockLspProvider ──

    fn make_diag(line: usize, col_start: usize, col_end: usize, msg: &str) -> Diagnostic {
        Diagnostic {
            message: msg.to_string(),
            severity: DiagnosticSeverity::Error,
            range: DiagnosticRange {
                start: DiagnosticPosition {
                    line,
                    column: col_start,
                },
                end: DiagnosticPosition {
                    line,
                    column: col_end,
                },
            },
            source: None,
        }
    }

    #[test]
    fn mock_provider_empty_has_no_diagnostics() {
        let p = MockLspProvider::new();
        assert!(p.all_diagnostics().is_empty());
        assert!(p.is_available());
    }

    #[test]
    fn mock_provider_diagnostics_at_returns_matching() {
        let diag = make_diag(3, 5, 10, "missing semicolon");
        let p = MockLspProvider::with_diagnostics(vec![diag.clone()]);
        // 범위 안 위치
        let results = p.diagnostics_at(3, 7);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].message, "missing semicolon");
    }

    #[test]
    fn mock_provider_diagnostics_at_outside_range_returns_empty() {
        let diag = make_diag(3, 5, 10, "error");
        let p = MockLspProvider::with_diagnostics(vec![diag]);
        // 다른 줄
        assert!(p.diagnostics_at(0, 5).is_empty());
        // 같은 줄 범위 밖
        assert!(p.diagnostics_at(3, 0).is_empty());
        assert!(p.diagnostics_at(3, 10).is_empty());
    }

    #[test]
    fn mock_provider_multiple_diagnostics_same_line() {
        let d1 = make_diag(2, 0, 3, "first error");
        let d2 = make_diag(2, 5, 8, "second error");
        let p = MockLspProvider::with_diagnostics(vec![d1, d2]);
        // 두 진단이 모두 반환되어야 함
        let all = p.all_diagnostics();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn mock_provider_unavailable_is_not_available() {
        let p = MockLspProvider::unavailable();
        assert!(!p.is_available());
    }

    // ── T4: LspBannerState ──

    #[test]
    fn lsp_banner_state_available_provider_no_banner() {
        let p = MockLspProvider::new();
        let mut banner = LspBannerState::default();
        banner.update(&p, "rust-analyzer");
        assert!(!banner.has_banner(), "사용 가능하면 배너 없음");
    }

    #[test]
    fn lsp_banner_state_unavailable_sets_message() {
        let p = MockLspProvider::unavailable();
        let mut banner = LspBannerState::default();
        banner.update(&p, "rust-analyzer");
        assert!(banner.has_banner());
        let msg = banner.message.clone().unwrap();
        assert!(msg.contains("rust-analyzer"), "배너에 서버 이름 포함");
    }

    #[test]
    fn lsp_banner_state_shown_only_once() {
        let p = MockLspProvider::unavailable();
        let mut banner = LspBannerState::default();
        banner.update(&p, "rust-analyzer");
        banner.dismiss(); // 배너 닫기
        banner.update(&p, "rust-analyzer"); // 재호출
        assert!(
            !banner.has_banner(),
            "2번째 update 는 배너 다시 표시하지 않음"
        );
    }

    #[test]
    fn lsp_banner_state_dismiss_clears_message() {
        let p = MockLspProvider::unavailable();
        let mut banner = LspBannerState::default();
        banner.update(&p, "gopls");
        assert!(banner.has_banner());
        banner.dismiss();
        assert!(!banner.has_banner());
    }

    // ── T5: all_diagnostics 위치 기반 필터링 ──

    #[test]
    fn mock_provider_diagnostics_at_column_boundary() {
        // column = end 는 범위 밖 (exclusive)
        let diag = make_diag(0, 0, 5, "test");
        let p = MockLspProvider::with_diagnostics(vec![diag]);
        assert_eq!(p.diagnostics_at(0, 4).len(), 1); // 포함
        assert_eq!(p.diagnostics_at(0, 5).len(), 0); // end 제외
    }
}
