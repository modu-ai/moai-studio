//! PtyBanner — 터미널 spawn 실패 알림 (Error, manual dismiss).
//!
//! REQ-V14-024: severity=Error, default strong="Terminal failed to spawn",
//!   meta="<error code> · cwd <path>", actions=[Restart Terminal(primary), Dismiss].
//!   auto_dismiss_after=None.

use std::path::PathBuf;

use crate::banners::{ActionButton, BannerData, BannerId, Severity};

/// PtyBanner 빌더 — 터미널 spawn 실패 알림 BannerData 생성.
///
/// @MX:ANCHOR: [AUTO] pty-banner-factory
/// @MX:REASON: [AUTO] REQ-V14-024. Error severity PTY 배너 생성의 유일한 진입점.
///   fan_in >= 3: BannerStack::push_pty (MS-3), 테스트, 통합 시나리오.
pub struct PtyBanner;

impl PtyBanner {
    /// 터미널 spawn 실패 배너 생성.
    ///
    /// - `error_code`: 오류 코드 (예: 1, 126, 127)
    /// - `cwd`: 현재 작업 디렉터리 경로 (meta 에 표시)
    pub fn build(error_code: i32, cwd: PathBuf) -> BannerData {
        let id = BannerId::new("pty:spawn");
        let message = "Terminal failed to spawn".to_string();
        let meta = format!("exit {} · cwd {}", error_code, cwd.display());
        let actions = vec![
            ActionButton::new("Restart Terminal", "pty:restart", true),
            ActionButton::new("Dismiss", "pty:dismiss", false),
        ];
        // Error → auto_dismiss_after=None (severity default 활용)
        BannerData::new(id, Severity::Error, message, Some(meta), actions)
    }
}

// ============================================================
// 단위 테스트 — PtyBanner (AC-V14-9, REQ-V14-024)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-V14-024: severity=Error 검증.
    #[test]
    fn pty_banner_severity_is_error() {
        let data = PtyBanner::build(1, "/home/user/project".into());
        assert_eq!(data.severity, Severity::Error);
    }

    /// REQ-V14-024: auto_dismiss_after=None 검증 (Error = manual only).
    #[test]
    fn pty_banner_no_auto_dismiss() {
        let data = PtyBanner::build(1, "/home/user/project".into());
        assert_eq!(data.auto_dismiss_after, None);
    }

    /// REQ-V14-024: default strong text = "Terminal failed to spawn".
    #[test]
    fn pty_banner_default_message() {
        let data = PtyBanner::build(1, "/tmp".into());
        assert_eq!(data.message, "Terminal failed to spawn");
    }

    /// REQ-V14-024: action count = 2.
    #[test]
    fn pty_banner_action_count() {
        let data = PtyBanner::build(127, "/tmp".into());
        assert_eq!(data.actions.len(), 2, "PtyBanner 액션 수는 2");
    }

    /// REQ-V14-024: primary action label = "Restart Terminal".
    #[test]
    fn pty_banner_primary_action_is_restart_terminal() {
        let data = PtyBanner::build(1, "/tmp".into());
        let primary = data.actions.iter().find(|a| a.primary);
        assert!(primary.is_some(), "primary 액션이 존재해야 함");
        assert_eq!(primary.unwrap().label, "Restart Terminal");
    }

    /// secondary action = "Dismiss".
    #[test]
    fn pty_banner_secondary_action_is_dismiss() {
        let data = PtyBanner::build(1, "/tmp".into());
        let secondary = data.actions.iter().find(|a| !a.primary);
        assert!(secondary.is_some(), "secondary 액션이 존재해야 함");
        assert_eq!(secondary.unwrap().label, "Dismiss");
    }

    /// meta 에 error code 포함 검증.
    #[test]
    fn pty_banner_meta_contains_error_code() {
        let data = PtyBanner::build(127, "/home/user".into());
        let meta = data.meta.as_deref().unwrap_or("");
        assert!(
            meta.contains("127"),
            "meta 에 error code 포함되어야 함: {meta}"
        );
    }

    /// meta 에 cwd 포함 검증.
    #[test]
    fn pty_banner_meta_contains_cwd() {
        let data = PtyBanner::build(1, "/workspace/moai".into());
        let meta = data.meta.as_deref().unwrap_or("");
        assert!(
            meta.contains("/workspace/moai"),
            "meta 에 cwd 포함되어야 함: {meta}"
        );
        assert!(meta.contains("cwd"), "meta 에 'cwd' 포함되어야 함: {meta}");
    }

    /// BannerId = "pty:spawn" 고정.
    #[test]
    fn pty_banner_id_is_fixed() {
        let data = PtyBanner::build(1, "/tmp".into());
        assert_eq!(data.id.as_str(), "pty:spawn");
    }

    /// AC-V14-9: pty_banner_default_spec 종합 검증.
    #[test]
    fn pty_banner_default_spec() {
        let data = PtyBanner::build(1, "/tmp".into());
        assert_eq!(data.severity, Severity::Error);
        assert_eq!(data.actions.len(), 2);
        let primary_label = &data.actions.iter().find(|a| a.primary).unwrap().label;
        assert_eq!(primary_label, "Restart Terminal");
        assert_eq!(data.auto_dismiss_after, None);
    }
}
