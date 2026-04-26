//! UpdateBanner — 업데이트 가용 알림 (Info, auto-dismiss 8s).
//!
//! REQ-V14-022: severity=Info, default strong="Update v{x.y.z} available",
//!   meta="<size> · changelog →", actions=[Update(primary), Later].
//!   auto_dismiss_after=Some(8s).

use crate::banners::{ActionButton, BannerData, BannerId, Severity};

/// UpdateBanner 빌더 — 새 버전 가용 알림 BannerData 생성.
///
/// @MX:ANCHOR: [AUTO] update-banner-factory
/// @MX:REASON: [AUTO] REQ-V14-022. Info severity 업데이트 배너 생성의 유일한 진입점.
///   fan_in >= 3: BannerStack::push_update (MS-3), 테스트, 통합 시나리오.
pub struct UpdateBanner;

impl UpdateBanner {
    /// 업데이트 가용 배너 생성.
    ///
    /// - `version`: 새 버전 문자열 (예: "0.2.0")
    /// - `size`: 다운로드 크기 (예: "12.3 MB")
    pub fn build(version: impl Into<String>, size: impl Into<String>) -> BannerData {
        let version = version.into();
        let size = size.into();
        let id = BannerId::new(format!("update:{version}"));
        let message = format!("Update v{version} available");
        let meta = format!("{size} · changelog →");
        let actions = vec![
            ActionButton::new("Update", "update:install", true),
            ActionButton::new("Later", "update:dismiss", false),
        ];
        // Info → auto_dismiss_after = Some(8s) (severity default 활용)
        BannerData::new(id, Severity::Info, message, Some(meta), actions)
    }
}

// ============================================================
// 단위 테스트 — UpdateBanner (AC-V14-9, REQ-V14-022)
// ============================================================

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// REQ-V14-022: severity=Info 검증.
    #[test]
    fn update_banner_severity_is_info() {
        let data = UpdateBanner::build("0.2.0", "12.3 MB");
        assert_eq!(data.severity, Severity::Info);
    }

    /// REQ-V14-022: auto_dismiss_after=Some(8s) 검증.
    #[test]
    fn update_banner_auto_dismiss_8s() {
        let data = UpdateBanner::build("0.2.0", "12.3 MB");
        assert_eq!(data.auto_dismiss_after, Some(Duration::from_secs(8)));
    }

    /// REQ-V14-022: default strong text = "Update v{x.y.z} available".
    #[test]
    fn update_banner_default_message() {
        let data = UpdateBanner::build("0.2.0", "12.3 MB");
        assert_eq!(data.message, "Update v0.2.0 available");
    }

    /// REQ-V14-022: action count = 2.
    #[test]
    fn update_banner_action_count() {
        let data = UpdateBanner::build("0.2.0", "12.3 MB");
        assert_eq!(data.actions.len(), 2, "UpdateBanner 액션 수는 2");
    }

    /// REQ-V14-022: primary action label = "Update".
    #[test]
    fn update_banner_primary_action_is_update() {
        let data = UpdateBanner::build("0.2.0", "12.3 MB");
        let primary = data.actions.iter().find(|a| a.primary);
        assert!(primary.is_some(), "primary 액션이 존재해야 함");
        assert_eq!(primary.unwrap().label, "Update");
    }

    /// secondary action = "Later".
    #[test]
    fn update_banner_secondary_action_is_later() {
        let data = UpdateBanner::build("0.2.0", "12.3 MB");
        let secondary = data.actions.iter().find(|a| !a.primary);
        assert!(secondary.is_some(), "secondary 액션이 존재해야 함");
        assert_eq!(secondary.unwrap().label, "Later");
    }

    /// meta 에 size + changelog → 포함 검증.
    #[test]
    fn update_banner_meta_format() {
        let data = UpdateBanner::build("0.2.0", "15 MB");
        let meta = data.meta.as_deref().unwrap_or("");
        assert!(meta.contains("15 MB"), "meta 에 size 포함되어야 함: {meta}");
        assert!(
            meta.contains("changelog →"),
            "meta 에 'changelog →' 포함되어야 함: {meta}"
        );
    }

    /// BannerId 에 버전 포함 검증.
    #[test]
    fn update_banner_id_contains_version() {
        let data = UpdateBanner::build("0.3.1", "8 MB");
        assert!(
            data.id.as_str().contains("0.3.1"),
            "BannerId 에 버전이 포함되어야 함: {}",
            data.id.as_str()
        );
    }

    /// AC-V14-9: update_banner_default_spec 종합 검증.
    #[test]
    fn update_banner_default_spec() {
        let data = UpdateBanner::build("0.2.0", "12.3 MB");
        assert_eq!(data.severity, Severity::Info);
        assert_eq!(data.actions.len(), 2);
        let primary_label = &data.actions.iter().find(|a| a.primary).unwrap().label;
        assert_eq!(primary_label, "Update");
        assert_eq!(data.auto_dismiss_after, Some(Duration::from_secs(8)));
    }
}
