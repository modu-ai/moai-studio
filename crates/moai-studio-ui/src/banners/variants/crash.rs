//! CrashBanner — Agent crash 알림 (Critical, manual dismiss).
//!
//! REQ-V14-021: severity=Critical, default strong="Agent crashed",
//!   meta="<log_path> · last alive <duration>", actions=[Reopen(primary), Dismiss].
//!   auto_dismiss_after=None.

use std::path::PathBuf;
use std::time::Duration;

use crate::banners::{ActionButton, BannerData, BannerId, Severity};

/// CrashBanner 빌더 — Agent crash 알림 BannerData 생성.
///
/// @MX:ANCHOR: [AUTO] crash-banner-factory
/// @MX:REASON: [AUTO] REQ-V14-021. Critical severity crash 배너 생성의 유일한 진입점.
///   fan_in >= 3: BannerStack::push_crash (MS-3), 테스트, 통합 시나리오.
pub struct CrashBanner;

impl CrashBanner {
    /// 기본 "Agent crashed" 배너 생성.
    ///
    /// - `log_path`: crash log 파일 경로 (meta 에 표시)
    /// - `last_alive`: crash 전 마지막 정상 동작 시점까지의 경과 시간
    pub fn build(log_path: PathBuf, last_alive: Duration) -> BannerData {
        let id = BannerId::new("crash:agent");
        let message = "Agent crashed".to_string();
        let meta = format!(
            "{} · last alive {}",
            log_path.display(),
            format_duration(last_alive),
        );
        let actions = vec![
            ActionButton::new("Reopen", "crash:reopen", true),
            ActionButton::new("Dismiss", "crash:dismiss", false),
        ];
        // Critical → auto_dismiss_after=None (severity default 활용)
        BannerData::new(id, Severity::Critical, message, Some(meta), actions)
    }
}

/// Duration 을 사람이 읽기 쉬운 형식으로 변환.
///
/// 예) 125s → "2m 5s", 45s → "45s", 0s → "0s"
pub(crate) fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs >= 60 {
        let m = secs / 60;
        let s = secs % 60;
        if s == 0 {
            format!("{m}m")
        } else {
            format!("{m}m {s}s")
        }
    } else {
        format!("{secs}s")
    }
}

// ============================================================
// 단위 테스트 — CrashBanner (AC-V14-9, REQ-V14-021)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// REQ-V14-021: severity=Critical 검증.
    #[test]
    fn crash_banner_severity_is_critical() {
        let data = CrashBanner::build("/tmp/crash.log".into(), Duration::from_secs(12));
        assert_eq!(data.severity, Severity::Critical);
    }

    /// REQ-V14-021: auto_dismiss_after=None 검증 (Critical = manual only).
    #[test]
    fn crash_banner_no_auto_dismiss() {
        let data = CrashBanner::build("/tmp/crash.log".into(), Duration::from_secs(12));
        assert_eq!(data.auto_dismiss_after, None);
    }

    /// REQ-V14-021: default strong text = "Agent crashed".
    #[test]
    fn crash_banner_default_message() {
        let data = CrashBanner::build("/tmp/crash.log".into(), Duration::from_secs(12));
        assert_eq!(data.message, "Agent crashed");
    }

    /// REQ-V14-021: action count = 2.
    #[test]
    fn crash_banner_action_count() {
        let data = CrashBanner::build("/tmp/crash.log".into(), Duration::from_secs(12));
        assert_eq!(data.actions.len(), 2, "CrashBanner 액션 수는 2");
    }

    /// REQ-V14-021: primary action label = "Reopen".
    #[test]
    fn crash_banner_primary_action_is_reopen() {
        let data = CrashBanner::build("/tmp/crash.log".into(), Duration::from_secs(12));
        let primary = data.actions.iter().find(|a| a.primary);
        assert!(primary.is_some(), "primary 액션이 존재해야 함");
        assert_eq!(primary.unwrap().label, "Reopen");
    }

    /// secondary action = "Dismiss".
    #[test]
    fn crash_banner_secondary_action_is_dismiss() {
        let data = CrashBanner::build("/tmp/crash.log".into(), Duration::from_secs(12));
        let secondary = data.actions.iter().find(|a| !a.primary);
        assert!(secondary.is_some(), "secondary 액션이 존재해야 함");
        assert_eq!(secondary.unwrap().label, "Dismiss");
    }

    /// meta 에 log_path 포함 검증.
    #[test]
    fn crash_banner_meta_contains_log_path() {
        let data = CrashBanner::build("/var/log/moai/crash.log".into(), Duration::from_secs(5));
        let meta = data.meta.as_deref().unwrap_or("");
        assert!(
            meta.contains("/var/log/moai/crash.log"),
            "meta 에 log_path 가 포함되어야 함: {meta}"
        );
    }

    /// meta 에 last_alive duration 포함 검증.
    #[test]
    fn crash_banner_meta_contains_last_alive() {
        let data = CrashBanner::build("/tmp/crash.log".into(), Duration::from_secs(125));
        let meta = data.meta.as_deref().unwrap_or("");
        assert!(
            meta.contains("last alive"),
            "meta 에 'last alive' 포함되어야 함: {meta}"
        );
        // 125초 = 2m 5s
        assert!(
            meta.contains("2m 5s"),
            "125초 → '2m 5s' 포맷 포함되어야 함: {meta}"
        );
    }

    /// BannerId = "crash:agent" 고정.
    #[test]
    fn crash_banner_id_is_fixed() {
        let data = CrashBanner::build("/tmp/crash.log".into(), Duration::from_secs(1));
        assert_eq!(data.id.as_str(), "crash:agent");
    }

    // ── format_duration 보조 함수 테스트 ──

    #[test]
    fn format_duration_less_than_minute() {
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
    }

    #[test]
    fn format_duration_exact_minute() {
        assert_eq!(format_duration(Duration::from_secs(60)), "1m");
    }

    #[test]
    fn format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
    }

    #[test]
    fn format_duration_zero() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
    }

    /// AC-V14-9: crash_banner_default_spec 종합 검증.
    #[test]
    fn crash_banner_default_spec() {
        let data = CrashBanner::build("/tmp/crash.log".into(), Duration::from_secs(12));
        assert_eq!(data.severity, Severity::Critical);
        assert_eq!(data.actions.len(), 2);
        let primary_label = &data.actions.iter().find(|a| a.primary).unwrap().label;
        assert_eq!(primary_label, "Reopen");
        assert_eq!(data.auto_dismiss_after, None);
    }
}
