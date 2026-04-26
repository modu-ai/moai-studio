//! Banners Surface 모듈 — SPEC-V3-014 MS-1.
//!
//! @MX:ANCHOR: [AUTO] banners-module-public-api
//! @MX:REASON: [AUTO] SPEC-V3-014 RG-V14-1. banners 모듈의 공개 API 진입점.
//!   fan_in >= 3: banner_stack.rs (push/dismiss), banner_view.rs (render), lib.rs (RootView 통합).
//!
//! 공개 API:
//! - [`Severity`] — 5단계 우선순위 enum (Critical > Error > Warning > Info > Success)
//! - [`BannerId`] — 불투명 중복 방지 식별자 (Eq + Hash)
//! - [`ActionButton`] — 배너 액션 버튼 데이터 (label + action_id)
//! - [`BannerData`] — 개별 배너 데이터 컨테이너 (BannerStack 이 관리)
//! - [`BannerView`] — 개별 배너 UI Entity (icon + text + actions + dismiss)
//! - [`BannerStack`] — 최대 3개 배너 Entity, severity priority + FIFO

pub mod banner_stack;
pub mod banner_view;
pub mod variants;

pub use banner_stack::BannerStack;
pub use banner_view::BannerView;
pub use variants::{CrashBanner, LspBanner, PtyBanner, UpdateBanner, WorkspaceBanner};

// ============================================================
// banner 레이아웃 치수 상수 (design::tokens 에 이전 전까지 임시 보유)
// ============================================================

/// 배너 높이 (px) — REQ-V14-006
pub const BANNER_HEIGHT_PX: f32 = 36.0;
/// 배너 수평 패딩 (px) = design::layout::spacing::S_3
pub const BANNER_PADDING_X_PX: f32 = 12.0;
/// 배너 아이콘 크기 (px) — REQ-V14-007
pub const BANNER_ICON_SIZE_PX: f32 = 16.0;
/// 배너 요소 간격 (px) = design::layout::spacing::S_2
pub const BANNER_ELEMENT_GAP_PX: f32 = 8.0;

// ============================================================
// Severity — 5단계 우선순위 (REQ-V14-002)
// ============================================================

/// 배너 심각도 — 5단계 우선순위 정렬.
///
/// @MX:ANCHOR: [AUTO] severity-ordering-invariant
/// @MX:REASON: [AUTO] BannerStack push/evict 정책의 기반 ordering.
///   Critical > Error > Warning > Info > Success (높을수록 우선).
///   fan_in >= 3: BannerStack::push, BannerStack::evict_lowest, banner_view severity_bg_color.
///   Ord 구현 변경 시 stack policy 전체 영향.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// 최저 우선순위 — 성공/완료 알림 (auto-dismiss 5초)
    Success = 0,
    /// 정보 알림 (auto-dismiss 8초)
    Info = 1,
    /// 경고 — 사용자 확인 필요 (manual dismiss)
    Warning = 2,
    /// 오류 — 처리 실패 (manual dismiss)
    Error = 3,
    /// 최고 우선순위 — 즉각 대응 필요 (manual dismiss)
    Critical = 4,
}

impl Severity {
    /// severity 에 대한 auto-dismiss duration (초). None = manual dismiss only.
    ///
    /// - Success: 5초
    /// - Info: 8초
    /// - Warning/Error/Critical: None
    pub fn auto_dismiss_secs(&self) -> Option<u64> {
        match self {
            Severity::Success => Some(5),
            Severity::Info => Some(8),
            Severity::Warning | Severity::Error | Severity::Critical => None,
        }
    }

    /// Severity 숫자값 반환 (높을수록 우선순위 높음).
    fn priority(&self) -> u8 {
        *self as u8
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Severity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority().cmp(&other.priority())
    }
}

// ============================================================
// BannerId — 불투명 식별자 (REQ-V14-004)
// ============================================================

/// 배너 고유 식별자 — 중복 push 방지 및 dismiss targeting 에 사용.
///
/// `BannerId::new("lsp:rust-analyzer")` 등으로 생성.
/// Eq + Hash 로 HashSet dedup 지원.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BannerId(String);

impl BannerId {
    /// 새 BannerId 생성.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// 내부 id 문자열 참조.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BannerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================
// ActionButton — 배너 액션 버튼 (REQ-V14-005)
// ============================================================

/// 배너 액션 버튼 데이터.
///
/// `label` 은 버튼 텍스트, `action_id` 는 핸들러 dispatch 키.
/// MS-1/MS-2 에서는 `action_id` 를 log::info! 로만 사용 (mock action).
#[derive(Debug, Clone, PartialEq)]
pub struct ActionButton {
    /// 버튼 라벨 텍스트
    pub label: String,
    /// 핸들러 식별자 (dispatch key)
    pub action_id: String,
    /// 주요 액션 여부 — true 이면 brand.primary 색상 적용
    pub primary: bool,
}

impl ActionButton {
    /// 새 ActionButton 생성.
    pub fn new(label: impl Into<String>, action_id: impl Into<String>, primary: bool) -> Self {
        Self {
            label: label.into(),
            action_id: action_id.into(),
            primary,
        }
    }
}

// ============================================================
// BannerData — 배너 데이터 컨테이너
// ============================================================

/// 개별 배너 데이터 — BannerStack 이 Vec 에 보관하는 단위.
///
/// severity, id, message, optional meta, actions, auto_dismiss_after 를 캡슐화.
pub struct BannerData {
    /// 배너 식별자 (dedup 용)
    pub id: BannerId,
    /// 심각도 (정렬 기준)
    pub severity: Severity,
    /// 주 메시지 텍스트 (strong)
    pub message: String,
    /// 보조 메시지 (muted, 옵션)
    pub meta: Option<String>,
    /// 액션 버튼 목록 (0~2개)
    pub actions: Vec<ActionButton>,
    /// auto-dismiss duration (None = manual only)
    pub auto_dismiss_after: Option<std::time::Duration>,
    /// 배너가 스택에 추가된 시각
    pub mounted_at: std::time::Instant,
}

impl BannerData {
    /// 새 BannerData 생성 (mounted_at = Instant::now()).
    pub fn new(
        id: BannerId,
        severity: Severity,
        message: impl Into<String>,
        meta: Option<String>,
        actions: Vec<ActionButton>,
    ) -> Self {
        let auto_dismiss_after = severity
            .auto_dismiss_secs()
            .map(std::time::Duration::from_secs);
        Self {
            id,
            severity,
            message: message.into(),
            meta,
            actions,
            auto_dismiss_after,
            mounted_at: std::time::Instant::now(),
        }
    }

    /// 커스텀 auto_dismiss_after 로 BannerData 생성.
    pub fn with_dismiss(
        id: BannerId,
        severity: Severity,
        message: impl Into<String>,
        meta: Option<String>,
        actions: Vec<ActionButton>,
        auto_dismiss_after: Option<std::time::Duration>,
    ) -> Self {
        Self {
            id,
            severity,
            message: message.into(),
            meta,
            actions,
            auto_dismiss_after,
            mounted_at: std::time::Instant::now(),
        }
    }
}

// ============================================================
// should_dismiss — 순수 함수 auto-dismiss 상태 (REQ-V14-018)
// ============================================================

/// auto-dismiss 만료 여부를 판단하는 순수 함수 (REQ-V14-018).
///
/// `auto_dismiss_after` 가 Some(d) 이고 `now - mounted_at >= d` 이면 true.
/// `auto_dismiss_after` 가 None 이면 항상 false.
///
/// @MX:ANCHOR: [AUTO] should-dismiss-pure-fn
/// @MX:REASON: [AUTO] SPEC-V3-014 REQ-V14-018. auto-dismiss 정책의 유일한 구현.
///   fan_in >= 3: BannerStack::tick, tests::should_dismiss_truth_table, MS-3 통합 테스트.
pub fn should_dismiss(
    mounted_at: std::time::Instant,
    auto_dismiss_after: Option<std::time::Duration>,
    now: std::time::Instant,
) -> bool {
    match auto_dismiss_after {
        None => false,
        Some(d) => now.duration_since(mounted_at) >= d,
    }
}

// ============================================================
// 단위 테스트 — Severity / BannerId / ActionButton / should_dismiss
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    // ── Severity ordering (AC-V14-2) ──

    /// Critical > Error > Warning > Info > Success — 내림차순 정렬 검증.
    #[test]
    fn severity_ordering_descending() {
        let mut sevs = vec![
            Severity::Info,
            Severity::Critical,
            Severity::Warning,
            Severity::Success,
            Severity::Error,
        ];
        // 내림차순 (높은 priority 먼저)
        sevs.sort_by(|a, b| b.cmp(a));
        assert_eq!(
            sevs,
            vec![
                Severity::Critical,
                Severity::Error,
                Severity::Warning,
                Severity::Info,
                Severity::Success,
            ],
            "Severity 내림차순 정렬: Critical > Error > Warning > Info > Success"
        );
    }

    /// PartialOrd 검증 — Critical 이 Success 보다 크다.
    #[test]
    fn severity_partial_ord_critical_gt_success() {
        assert!(Severity::Critical > Severity::Success);
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
        assert!(Severity::Info > Severity::Success);
        assert!(Severity::Success <= Severity::Info);
    }

    /// 동일 severity 는 Equal.
    #[test]
    fn severity_equal_same_variant() {
        assert_eq!(
            Severity::Warning.cmp(&Severity::Warning),
            std::cmp::Ordering::Equal
        );
        assert_eq!(Severity::Critical, Severity::Critical);
    }

    /// Severity 오름차순 정렬 — Success...Critical 순.
    #[test]
    fn severity_ordering_ascending() {
        let mut sevs = vec![Severity::Critical, Severity::Info, Severity::Success];
        sevs.sort();
        assert_eq!(
            sevs,
            vec![Severity::Success, Severity::Info, Severity::Critical]
        );
    }

    // ── auto_dismiss_secs (REQ-V14-017) ──

    #[test]
    fn auto_dismiss_secs_success_is_5() {
        assert_eq!(Severity::Success.auto_dismiss_secs(), Some(5));
    }

    #[test]
    fn auto_dismiss_secs_info_is_8() {
        assert_eq!(Severity::Info.auto_dismiss_secs(), Some(8));
    }

    #[test]
    fn auto_dismiss_secs_warning_is_none() {
        assert_eq!(Severity::Warning.auto_dismiss_secs(), None);
    }

    #[test]
    fn auto_dismiss_secs_error_is_none() {
        assert_eq!(Severity::Error.auto_dismiss_secs(), None);
    }

    #[test]
    fn auto_dismiss_secs_critical_is_none() {
        assert_eq!(Severity::Critical.auto_dismiss_secs(), None);
    }

    // ── BannerId (REQ-V14-004) ──

    #[test]
    fn banner_id_eq_same_string() {
        let a = BannerId::new("lsp:rust-analyzer");
        let b = BannerId::new("lsp:rust-analyzer");
        assert_eq!(a, b);
    }

    #[test]
    fn banner_id_ne_different_string() {
        let a = BannerId::new("crash:1");
        let b = BannerId::new("crash:2");
        assert_ne!(a, b);
    }

    #[test]
    fn banner_id_hash_usable_in_hashset() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(BannerId::new("a"));
        set.insert(BannerId::new("a"));
        set.insert(BannerId::new("b"));
        assert_eq!(set.len(), 2, "중복 BannerId 는 HashSet 에서 1개로 처리");
    }

    #[test]
    fn banner_id_as_str_returns_inner() {
        let id = BannerId::new("update:v0.2.0");
        assert_eq!(id.as_str(), "update:v0.2.0");
    }

    #[test]
    fn banner_id_display() {
        let id = BannerId::new("pty:1");
        assert_eq!(format!("{id}"), "pty:1");
    }

    // ── ActionButton ──

    #[test]
    fn action_button_fields() {
        let btn = ActionButton::new("Reopen", "crash:reopen", true);
        assert_eq!(btn.label, "Reopen");
        assert_eq!(btn.action_id, "crash:reopen");
        assert!(btn.primary);
    }

    #[test]
    fn action_button_secondary() {
        let btn = ActionButton::new("Dismiss", "crash:dismiss", false);
        assert!(!btn.primary);
    }

    #[test]
    fn action_button_equality() {
        let a = ActionButton::new("Update", "update:install", true);
        let b = ActionButton::new("Update", "update:install", true);
        assert_eq!(a, b);
    }

    #[test]
    fn action_button_inequality_different_label() {
        let a = ActionButton::new("Update", "update:install", true);
        let b = ActionButton::new("Later", "update:dismiss", false);
        assert_ne!(a, b);
    }

    // ── should_dismiss (AC-V14-8) — 4 케이스 진리표 ──

    /// should_dismiss 진리표 (REQ-V14-018):
    /// 1. auto_dismiss_after=Some(8s), now < T0+8s → false
    /// 2. auto_dismiss_after=Some(8s), now >= T0+8s → true
    /// 3. auto_dismiss_after=None, now=T0+1h → false
    /// 4. auto_dismiss_after=Some(5s), now = T0+5s exactly → true (경계값)
    #[test]
    fn should_dismiss_truth_table() {
        let t0 = Instant::now();

        // Case 1: 7.999초 경과 — 아직 dismiss 안 됨
        let before = t0 + Duration::from_millis(7_999);
        assert!(
            !should_dismiss(t0, Some(Duration::from_secs(8)), before),
            "7.999초 경과 시 dismiss 되지 않아야 함"
        );

        // Case 2: 8.001초 경과 — dismiss
        let after = t0 + Duration::from_millis(8_001);
        assert!(
            should_dismiss(t0, Some(Duration::from_secs(8)), after),
            "8.001초 경과 시 dismiss 되어야 함"
        );

        // Case 3: auto_dismiss_after=None, 1시간 경과 — never dismiss
        let one_hour = t0 + Duration::from_secs(3600);
        assert!(
            !should_dismiss(t0, None, one_hour),
            "auto_dismiss_after=None 이면 절대 dismiss 안 됨"
        );

        // Case 4: 경계값 — exactly 8초 경과 → true
        let exact = t0 + Duration::from_secs(8);
        assert!(
            should_dismiss(t0, Some(Duration::from_secs(8)), exact),
            "정확히 8초 경과 시 dismiss 되어야 함 (경계값 포함)"
        );
    }

    // ── BannerData 생성 ──

    #[test]
    fn banner_data_new_sets_auto_dismiss_from_severity() {
        let data = BannerData::new(
            BannerId::new("info:test"),
            Severity::Info,
            "Test message",
            None,
            vec![],
        );
        assert_eq!(data.severity, Severity::Info);
        assert_eq!(data.auto_dismiss_after, Some(Duration::from_secs(8)));
    }

    #[test]
    fn banner_data_critical_has_no_auto_dismiss() {
        let data = BannerData::new(
            BannerId::new("crash:test"),
            Severity::Critical,
            "Agent crashed",
            None,
            vec![],
        );
        assert_eq!(data.auto_dismiss_after, None);
    }

    #[test]
    fn banner_data_with_dismiss_overrides_severity_default() {
        let data = BannerData::with_dismiss(
            BannerId::new("custom"),
            Severity::Critical,
            "Custom message",
            Some("meta info".to_string()),
            vec![],
            Some(Duration::from_secs(3)),
        );
        // Critical 이지만 커스텀 dismiss 3초 설정
        assert_eq!(data.auto_dismiss_after, Some(Duration::from_secs(3)));
    }
}
