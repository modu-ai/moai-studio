//! WorkspaceBanner — Workspace 상태 손상 알림 (Warning, manual dismiss).
//!
//! REQ-V14-025: severity=Warning, default strong="Workspace state corrupted",
//!   meta="<bak path>", actions=[Reset Workspace(primary), Continue].
//!   auto_dismiss_after=None.

use std::path::PathBuf;

use crate::banners::{ActionButton, BannerData, BannerId, Severity};

/// WorkspaceBanner 빌더 — Workspace 상태 손상 알림 BannerData 생성.
///
/// @MX:ANCHOR: [AUTO] workspace-banner-factory
/// @MX:REASON: [AUTO] REQ-V14-025. Warning severity Workspace 배너 생성의 유일한 진입점.
///   fan_in >= 3: BannerStack::push_workspace (MS-3), 테스트, 통합 시나리오.
pub struct WorkspaceBanner;

impl WorkspaceBanner {
    /// Workspace 상태 손상 배너 생성.
    ///
    /// - `bak_path`: 백업 파일 경로 (None 이면 meta 없음)
    pub fn build(bak_path: Option<PathBuf>) -> BannerData {
        let id = BannerId::new("workspace:corrupted");
        let message = "Workspace state corrupted".to_string();
        let meta = bak_path.map(|p| p.display().to_string());
        let actions = vec![
            ActionButton::new("Reset Workspace", "workspace:reset", true),
            ActionButton::new("Continue", "workspace:continue", false),
        ];
        // Warning → auto_dismiss_after=None (severity default 활용)
        BannerData::new(id, Severity::Warning, message, meta, actions)
    }
}

// ============================================================
// 단위 테스트 — WorkspaceBanner (AC-V14-9, REQ-V14-025)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-V14-025: severity=Warning 검증.
    #[test]
    fn workspace_banner_severity_is_warning() {
        let data = WorkspaceBanner::build(None);
        assert_eq!(data.severity, Severity::Warning);
    }

    /// REQ-V14-025: auto_dismiss_after=None 검증 (Warning = manual only).
    #[test]
    fn workspace_banner_no_auto_dismiss() {
        let data = WorkspaceBanner::build(None);
        assert_eq!(data.auto_dismiss_after, None);
    }

    /// REQ-V14-025: default strong text = "Workspace state corrupted".
    #[test]
    fn workspace_banner_default_message() {
        let data = WorkspaceBanner::build(None);
        assert_eq!(data.message, "Workspace state corrupted");
    }

    /// REQ-V14-025: action count = 2.
    #[test]
    fn workspace_banner_action_count() {
        let data = WorkspaceBanner::build(None);
        assert_eq!(data.actions.len(), 2, "WorkspaceBanner 액션 수는 2");
    }

    /// REQ-V14-025: primary action label = "Reset Workspace".
    #[test]
    fn workspace_banner_primary_action_is_reset() {
        let data = WorkspaceBanner::build(None);
        let primary = data.actions.iter().find(|a| a.primary);
        assert!(primary.is_some(), "primary 액션이 존재해야 함");
        assert_eq!(primary.unwrap().label, "Reset Workspace");
    }

    /// secondary action = "Continue".
    #[test]
    fn workspace_banner_secondary_action_is_continue() {
        let data = WorkspaceBanner::build(None);
        let secondary = data.actions.iter().find(|a| !a.primary);
        assert!(secondary.is_some(), "secondary 액션이 존재해야 함");
        assert_eq!(secondary.unwrap().label, "Continue");
    }

    /// bak_path Some → meta 에 경로 포함.
    #[test]
    fn workspace_banner_meta_with_bak_path() {
        let data = WorkspaceBanner::build(Some("/tmp/workspace.bak".into()));
        let meta = data.meta.as_deref().unwrap_or("");
        assert!(
            meta.contains("/tmp/workspace.bak"),
            "meta 에 bak_path 포함되어야 함: {meta}"
        );
    }

    /// bak_path None → meta = None.
    #[test]
    fn workspace_banner_no_meta_when_no_bak_path() {
        let data = WorkspaceBanner::build(None);
        assert!(data.meta.is_none(), "bak_path=None 이면 meta=None");
    }

    /// BannerId = "workspace:corrupted" 고정.
    #[test]
    fn workspace_banner_id_is_fixed() {
        let data = WorkspaceBanner::build(None);
        assert_eq!(data.id.as_str(), "workspace:corrupted");
    }

    /// AC-V14-9: workspace_banner_default_spec 종합 검증.
    #[test]
    fn workspace_banner_default_spec() {
        let data = WorkspaceBanner::build(None);
        assert_eq!(data.severity, Severity::Warning);
        assert_eq!(data.actions.len(), 2);
        let primary_label = &data.actions.iter().find(|a| a.primary).unwrap().label;
        assert_eq!(primary_label, "Reset Workspace");
        assert_eq!(data.auto_dismiss_after, None);
    }
}
