//! LspBanner — LSP 서버 시작 실패 알림 (Warning, manual dismiss).
//!
//! REQ-V14-023: severity=Warning, default strong="<server> failed to start",
//!   meta="<error reason>", actions=[Configure(primary), Dismiss].
//!   auto_dismiss_after=None.

use crate::banners::{ActionButton, BannerData, BannerId, Severity};

/// LspBanner 빌더 — LSP 서버 시작 실패 알림 BannerData 생성.
///
/// @MX:ANCHOR: [AUTO] lsp-banner-factory
/// @MX:REASON: [AUTO] REQ-V14-023. Warning severity LSP 배너 생성의 유일한 진입점.
///   fan_in >= 3: BannerStack::push_lsp (MS-3), 테스트, 통합 시나리오.
pub struct LspBanner;

impl LspBanner {
    /// LSP 서버 시작 실패 배너 생성.
    ///
    /// - `server`: LSP 서버 이름 (예: "rust-analyzer")
    /// - `error`: 오류 원인 문자열 (meta 에 표시)
    pub fn build(server: impl Into<String>, error: impl Into<String>) -> BannerData {
        let server = server.into();
        let error = error.into();
        let id = BannerId::new(format!("lsp:{server}"));
        let message = format!("{server} failed to start");
        let actions = vec![
            ActionButton::new("Configure", "lsp:configure", true),
            ActionButton::new("Dismiss", "lsp:dismiss", false),
        ];
        // Warning → auto_dismiss_after=None (severity default 활용)
        BannerData::new(id, Severity::Warning, message, Some(error), actions)
    }
}

// ============================================================
// 단위 테스트 — LspBanner (AC-V14-9, REQ-V14-023)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-V14-023: severity=Warning 검증.
    #[test]
    fn lsp_banner_severity_is_warning() {
        let data = LspBanner::build("rust-analyzer", "binary not found");
        assert_eq!(data.severity, Severity::Warning);
    }

    /// REQ-V14-023: auto_dismiss_after=None 검증 (Warning = manual only).
    #[test]
    fn lsp_banner_no_auto_dismiss() {
        let data = LspBanner::build("rust-analyzer", "binary not found");
        assert_eq!(data.auto_dismiss_after, None);
    }

    /// REQ-V14-023: default strong text = "<server> failed to start".
    #[test]
    fn lsp_banner_default_message() {
        let data = LspBanner::build("rust-analyzer", "binary not found");
        assert_eq!(data.message, "rust-analyzer failed to start");
    }

    /// REQ-V14-023: action count = 2.
    #[test]
    fn lsp_banner_action_count() {
        let data = LspBanner::build("rust-analyzer", "binary not found");
        assert_eq!(data.actions.len(), 2, "LspBanner 액션 수는 2");
    }

    /// REQ-V14-023: primary action label = "Configure".
    #[test]
    fn lsp_banner_primary_action_is_configure() {
        let data = LspBanner::build("rust-analyzer", "binary not found");
        let primary = data.actions.iter().find(|a| a.primary);
        assert!(primary.is_some(), "primary 액션이 존재해야 함");
        assert_eq!(primary.unwrap().label, "Configure");
    }

    /// secondary action = "Dismiss".
    #[test]
    fn lsp_banner_secondary_action_is_dismiss() {
        let data = LspBanner::build("rust-analyzer", "binary not found");
        let secondary = data.actions.iter().find(|a| !a.primary);
        assert!(secondary.is_some(), "secondary 액션이 존재해야 함");
        assert_eq!(secondary.unwrap().label, "Dismiss");
    }

    /// meta 에 error reason 포함 검증.
    #[test]
    fn lsp_banner_meta_contains_error() {
        let data = LspBanner::build("pyright", "timeout on spawn");
        let meta = data.meta.as_deref().unwrap_or("");
        assert_eq!(meta, "timeout on spawn", "meta 가 error reason 이어야 함");
    }

    /// BannerId 에 server 이름 포함.
    #[test]
    fn lsp_banner_id_contains_server() {
        let data = LspBanner::build("tsserver", "missing node");
        assert!(
            data.id.as_str().contains("tsserver"),
            "BannerId 에 server 이름이 포함되어야 함: {}",
            data.id.as_str()
        );
    }

    /// AC-V14-9: lsp_banner_default_spec 종합 검증.
    #[test]
    fn lsp_banner_default_spec() {
        let data = LspBanner::build("rust-analyzer", "binary not found");
        assert_eq!(data.severity, Severity::Warning);
        assert_eq!(data.actions.len(), 2);
        let primary_label = &data.actions.iter().find(|a| a.primary).unwrap().label;
        assert_eq!(primary_label, "Configure");
        assert_eq!(data.auto_dismiss_after, None);
    }
}
