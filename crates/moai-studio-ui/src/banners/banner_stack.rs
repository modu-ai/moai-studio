//! BannerStack — 최대 3개 배너 관리 (SPEC-V3-014 MS-1).
//!
//! @MX:ANCHOR: [AUTO] banner-stack-push-policy
//! @MX:REASON: [AUTO] SPEC-V3-014 RG-V14-3. BannerStack 은 severity priority + FIFO 로 배너를 관리한다.
//!   push/dismiss/tick 이 모든 배너 상태 변이의 진입점.
//!   fan_in >= 3: MS-3 helper API (push_crash/update/lsp/pty/workspace), RootView::render, tick loop.

use std::collections::HashSet;
use std::time::Instant;

use super::{BannerData, BannerId, Severity, should_dismiss};

/// 최대 동시 배너 수 (REQ-V14-011).
pub const MAX_BANNERS: usize = 3;

// ============================================================
// BannerEntry — 스택 내부 배너 보관 단위
// ============================================================

/// BannerStack 내부에서 단일 배너를 보관하는 구조체.
pub struct BannerEntry {
    /// 배너 데이터
    pub data: BannerData,
    /// 삽입 순서 (FIFO 동순위 정렬 기준)
    pub insert_seq: u64,
}

// ============================================================
// BannerStack
// ============================================================

/// 최대 3개 배너를 severity priority + FIFO 로 관리하는 스택 (REQ-V14-011 ~ REQ-V14-016).
///
/// ## Push 정책
/// - capacity < MAX: severity 정렬 후 삽입
/// - capacity == MAX && new_severity > lowest: 최저 priority 중 가장 오래된 것 evict
/// - capacity == MAX && new_severity <= lowest: drop (무시)
/// - 동일 BannerId 가 이미 있으면 dedup (무시)
///
/// ## 정렬 순서
/// - 높은 severity 먼저 (Critical > Error > Warning > Info > Success)
/// - 동일 severity 내에서는 insert_seq 오름차순 (FIFO)
pub struct BannerStack {
    /// 배너 엔트리 목록 (정렬 유지)
    entries: Vec<BannerEntry>,
    /// 현재 스택에 있는 id 집합 (dedup)
    ids: HashSet<BannerId>,
    /// 삽입 순서 카운터 (FIFO 정렬 기준)
    next_seq: u64,
}

impl BannerStack {
    /// 빈 BannerStack 생성.
    pub fn new() -> Self {
        Self {
            entries: Vec::with_capacity(MAX_BANNERS),
            ids: HashSet::new(),
            next_seq: 0,
        }
    }

    /// 현재 스택에 있는 배너 데이터 슬라이스 (읽기 전용).
    pub fn entries(&self) -> &[BannerEntry] {
        &self.entries
    }

    /// 현재 배너 수.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 스택이 비어있는지 확인.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 배너 id 가 스택에 있는지 확인 (dedup 용).
    pub fn contains(&self, id: &BannerId) -> bool {
        self.ids.contains(id)
    }

    // ── Push 정책 (REQ-V14-012 ~ REQ-V14-015) ──

    /// 새 배너를 스택에 추가한다.
    ///
    /// 반환값: 실제로 삽입되었으면 true, drop/dedup 이면 false.
    pub fn push(&mut self, data: BannerData) -> bool {
        // REQ-V14-015: dedup — 동일 id 가 이미 있으면 무시.
        if self.ids.contains(&data.id) {
            return false;
        }

        if self.entries.len() < MAX_BANNERS {
            // REQ-V14-012: capacity 여유 있으면 삽입 + 정렬.
            self.insert_sorted(data);
            true
        } else {
            // Full 상태 — evict 또는 drop 결정.
            let lowest_severity = self.lowest_severity();
            if data.severity > lowest_severity {
                // REQ-V14-013: 새 배너가 더 높으면 최저 priority 중 가장 오래된 것 evict.
                self.evict_lowest_oldest();
                self.insert_sorted(data);
                true
            } else {
                // REQ-V14-014: 동일하거나 낮으면 drop.
                false
            }
        }
    }

    /// id 로 배너를 제거한다 (REQ-V14-016).
    ///
    /// 반환값: 실제로 제거되었으면 true.
    pub fn dismiss(&mut self, id: &BannerId) -> bool {
        if let Some(pos) = self.entries.iter().position(|e| &e.data.id == id) {
            let removed = self.entries.remove(pos);
            self.ids.remove(&removed.data.id);
            true
        } else {
            false
        }
    }

    /// 스택의 모든 배너를 제거한다.
    pub fn dismiss_all(&mut self) {
        self.entries.clear();
        self.ids.clear();
    }

    /// auto-dismiss 만료된 배너를 제거한다 (REQ-V14-019).
    ///
    /// 반환값: 제거된 배너 id 목록.
    pub fn tick(&mut self, now: Instant) -> Vec<BannerId> {
        let to_remove: Vec<BannerId> = self
            .entries
            .iter()
            .filter(|e| should_dismiss(e.data.mounted_at, e.data.auto_dismiss_after, now))
            .map(|e| e.data.id.clone())
            .collect();

        for id in &to_remove {
            self.dismiss(id);
        }
        to_remove
    }

    // ── 내부 헬퍼 ──

    /// 정렬 순서를 유지하면서 배너를 삽입한다.
    /// 정렬 기준: severity 내림차순 → insert_seq 오름차순 (FIFO).
    fn insert_sorted(&mut self, data: BannerData) {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.ids.insert(data.id.clone());
        let entry = BannerEntry {
            data,
            insert_seq: seq,
        };
        // 삽입 위치: severity 내림차순 유지 (같은 severity 내에서는 뒤에 추가 = FIFO)
        let pos = self.entries.iter().position(|e| {
            e.data.severity < entry.data.severity
                || (e.data.severity == entry.data.severity && e.insert_seq > entry.insert_seq)
        });
        match pos {
            Some(i) => self.entries.insert(i, entry),
            None => self.entries.push(entry),
        }
    }

    /// 현재 스택에서 가장 낮은 severity 를 반환한다.
    fn lowest_severity(&self) -> Severity {
        self.entries
            .iter()
            .map(|e| e.data.severity)
            .min()
            .unwrap_or(Severity::Critical)
    }

    /// 가장 낮은 priority 중 가장 오래된 배너 (insert_seq 가장 작은 것) 를 제거한다.
    fn evict_lowest_oldest(&mut self) {
        let lowest = self.lowest_severity();
        // 가장 낮은 severity 중 insert_seq 가 가장 작은 것 (= 가장 오래된 것)
        let pos = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.data.severity == lowest)
            .min_by_key(|(_, e)| e.insert_seq)
            .map(|(i, _)| i);

        if let Some(i) = pos {
            let removed = self.entries.remove(i);
            self.ids.remove(&removed.data.id);
        }
    }
}

impl Default for BannerStack {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// 단위 테스트 — BannerStack push/dismiss/tick/dedup (AC-V14-3 ~ AC-V14-8)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::banners::{ActionButton, BannerData, BannerId, Severity};
    use std::time::Duration;

    // ── 헬퍼 팩토리 ──

    fn make_banner(id: &str, severity: Severity) -> BannerData {
        BannerData::new(
            BannerId::new(id),
            severity,
            format!("Message for {id}"),
            None,
            vec![],
        )
    }

    fn make_banner_with_actions(
        id: &str,
        severity: Severity,
        actions: Vec<ActionButton>,
    ) -> BannerData {
        BannerData::new(
            BannerId::new(id),
            severity,
            format!("Message for {id}"),
            None,
            actions,
        )
    }

    // ── 기본 push (AC-V14-3 일부) ──

    /// 빈 스택에 배너 1개 push → len == 1.
    #[test]
    fn stack_push_single_to_empty() {
        let mut stack = BannerStack::new();
        let inserted = stack.push(make_banner("crash:1", Severity::Critical));
        assert!(inserted, "빈 스택에 push 는 성공해야 함");
        assert_eq!(stack.len(), 1);
    }

    /// 3개 순차 push — 모두 보유 (AC-V14-3 전반부).
    #[test]
    fn stack_push_under_capacity_keeps_all() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("update:1", Severity::Info));
        stack.push(make_banner("lsp:1", Severity::Warning));
        stack.push(make_banner("crash:1", Severity::Critical));
        assert_eq!(stack.len(), 3, "3개 push 후 len == 3");
    }

    /// Crash(Critical), Lsp(Warning), Update(Info) 순 push → priority 정렬 (AC-V14-3).
    #[test]
    fn stack_push_three_priority_ordered() {
        let mut stack = BannerStack::new();
        // push 순서: crash → lsp → update
        stack.push(make_banner("crash:1", Severity::Critical));
        stack.push(make_banner("lsp:1", Severity::Warning));
        stack.push(make_banner("update:1", Severity::Info));

        let entries = stack.entries();
        assert_eq!(entries.len(), 3);
        assert_eq!(
            entries[0].data.severity,
            Severity::Critical,
            "0번째 = Critical"
        );
        assert_eq!(
            entries[1].data.severity,
            Severity::Warning,
            "1번째 = Warning"
        );
        assert_eq!(entries[2].data.severity, Severity::Info, "2번째 = Info");
    }

    /// 역순 push (Info → Warning → Critical) 해도 priority 정렬 유지.
    #[test]
    fn stack_push_reverse_order_still_sorted() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("update:1", Severity::Info));
        stack.push(make_banner("lsp:1", Severity::Warning));
        stack.push(make_banner("crash:1", Severity::Critical));

        let entries = stack.entries();
        assert_eq!(entries[0].data.severity, Severity::Critical);
        assert_eq!(entries[1].data.severity, Severity::Warning);
        assert_eq!(entries[2].data.severity, Severity::Info);
    }

    // ── Evict 정책 (AC-V14-4) ──

    /// Full stack + Critical push → 최저 priority 의 가장 오래된 것 evict.
    ///
    /// 초기: [Update(Info), Update2(Info), Lsp(Warning)]
    /// push Crash(Critical) → Update(Info, oldest) evict
    /// 결과: [Crash(Critical), Lsp(Warning), Update2(Info)]
    #[test]
    fn stack_evict_lowest_priority_oldest() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("update:1", Severity::Info)); // seq=0 (oldest Info)
        stack.push(make_banner("update:2", Severity::Info)); // seq=1
        stack.push(make_banner("lsp:1", Severity::Warning));

        // Full 상태에서 Critical push → Info 중 oldest(update:1) evict
        let inserted = stack.push(make_banner("crash:1", Severity::Critical));
        assert!(inserted, "Critical > Info 이므로 evict 후 삽입되어야 함");
        assert_eq!(stack.len(), 3);

        let entries = stack.entries();
        assert_eq!(entries[0].data.severity, Severity::Critical);
        // crash:1 이 스택에 있어야 함
        assert!(stack.contains(&BannerId::new("crash:1")));
        // update:1 (oldest Info) 이 evict 되어야 함
        assert!(
            !stack.contains(&BannerId::new("update:1")),
            "update:1 은 evict 되어야 함"
        );
        // update:2 는 남아있어야 함
        assert!(
            stack.contains(&BannerId::new("update:2")),
            "update:2 는 남아있어야 함"
        );
    }

    /// Error push 로 Lsp(Warning) evict 검증 — severity 다단계.
    #[test]
    fn stack_evict_on_priority_increase() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("crash:1", Severity::Critical));
        stack.push(make_banner("lsp:1", Severity::Warning)); // lowest
        stack.push(make_banner("update:1", Severity::Info)); // lowest

        // Full 에서 Error push → Info(lowest) 의 oldest evict
        stack.push(make_banner("pty:1", Severity::Error));
        assert_eq!(stack.len(), 3);
        // Info 배너(update:1)가 evict
        assert!(!stack.contains(&BannerId::new("update:1")));
        assert!(stack.contains(&BannerId::new("pty:1")));
    }

    // ── Drop 정책 (AC-V14-5) ──

    /// Full stack + 동일 priority push → drop (무시).
    ///
    /// 초기: [Crash(Critical), Pty(Error), Lsp(Warning)]
    /// push Workspace(Warning) → Warning <= lowest(Warning) → drop
    #[test]
    fn stack_drop_on_equal_priority_when_full() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("crash:1", Severity::Critical));
        stack.push(make_banner("pty:1", Severity::Error));
        stack.push(make_banner("lsp:1", Severity::Warning));

        let inserted = stack.push(make_banner("workspace:1", Severity::Warning));
        assert!(!inserted, "동일 priority (Warning) 는 drop 되어야 함");
        assert_eq!(stack.len(), 3, "스택은 3개 유지");
        assert!(
            !stack.contains(&BannerId::new("workspace:1")),
            "workspace:1 은 없어야 함"
        );
    }

    /// Full stack + 낮은 priority push → drop.
    #[test]
    fn stack_drop_on_lower_priority_when_full() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("crash:1", Severity::Critical));
        stack.push(make_banner("pty:1", Severity::Error));
        stack.push(make_banner("lsp:1", Severity::Warning));

        let inserted = stack.push(make_banner("update:1", Severity::Info));
        assert!(!inserted, "낮은 priority (Info) 는 drop 되어야 함");
        assert_eq!(stack.len(), 3);
    }

    // ── Dedup (AC-V14-6) ──

    /// 동일 BannerId 의 두 번째 push → dedup (무시), len 유지.
    #[test]
    fn stack_dedup_same_id() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("lsp:rust-analyzer", Severity::Warning));
        let second = stack.push(make_banner("lsp:rust-analyzer", Severity::Warning));
        assert!(!second, "중복 id push 는 false 반환");
        assert_eq!(stack.len(), 1, "dedup 후 len == 1");
    }

    /// 서로 다른 id 는 dedup 적용 안 됨.
    #[test]
    fn stack_no_dedup_different_ids() {
        let mut stack = BannerStack::new();
        let a = stack.push(make_banner("lsp:rust-analyzer", Severity::Warning));
        let b = stack.push(make_banner("lsp:gopls", Severity::Warning));
        assert!(a && b, "서로 다른 id 는 모두 삽입");
        assert_eq!(stack.len(), 2);
    }

    // ── Dismiss (AC-V14-7) ──

    /// dismiss(id) → 해당 배너 제거.
    #[test]
    fn dismiss_removes_target() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("crash:1", Severity::Critical));
        stack.push(make_banner("lsp:1", Severity::Warning));
        stack.push(make_banner("update:1", Severity::Info));

        let removed = stack.dismiss(&BannerId::new("lsp:1"));
        assert!(removed, "존재하는 id dismiss 는 true");
        assert_eq!(stack.len(), 2);
        assert!(
            !stack.contains(&BannerId::new("lsp:1")),
            "lsp:1 은 없어야 함"
        );
        assert!(stack.contains(&BannerId::new("crash:1")));
        assert!(stack.contains(&BannerId::new("update:1")));
    }

    /// dismiss(없는 id) → false, 스택 변경 없음.
    #[test]
    fn dismiss_unknown_id_returns_false() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("crash:1", Severity::Critical));
        let result = stack.dismiss(&BannerId::new("no-such-id"));
        assert!(!result, "없는 id dismiss 는 false");
        assert_eq!(stack.len(), 1, "스택 변경 없음");
    }

    /// dismiss_all() → 스택 비어짐.
    #[test]
    fn dismiss_all_clears_stack() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("crash:1", Severity::Critical));
        stack.push(make_banner("lsp:1", Severity::Warning));
        stack.dismiss_all();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    // ── FIFO 동순위 정렬 ──

    /// 같은 severity 내에서는 FIFO (먼저 push 된 것이 앞에).
    #[test]
    fn same_severity_fifo_ordering() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("warn:1", Severity::Warning)); // seq=0 (first)
        stack.push(make_banner("warn:2", Severity::Warning)); // seq=1
        stack.push(make_banner("warn:3", Severity::Warning)); // seq=2

        let entries = stack.entries();
        assert_eq!(
            entries[0].data.id.as_str(),
            "warn:1",
            "첫 번째 push 가 0번째"
        );
        assert_eq!(entries[1].data.id.as_str(), "warn:2");
        assert_eq!(entries[2].data.id.as_str(), "warn:3");
    }

    // ── Tick / auto-dismiss ──

    /// tick() — 만료된 배너 제거.
    #[test]
    fn tick_removes_expired_banners() {
        let mut stack = BannerStack::new();

        // Success 배너 (5초 auto-dismiss) 를 수동으로 과거 시각으로 설정
        let mut data = make_banner("success:1", Severity::Success);
        // auto_dismiss_after 을 1ms 로 짧게
        data.auto_dismiss_after = Some(Duration::from_millis(1));
        // mounted_at 을 과거로 설정 (이미 만료)
        data.mounted_at = Instant::now() - Duration::from_secs(1);
        stack.push(data);

        // Critical 배너 (manual dismiss)
        stack.push(make_banner("crash:1", Severity::Critical));

        let now = Instant::now();
        let removed = stack.tick(now);

        assert_eq!(removed.len(), 1, "만료된 배너 1개 제거");
        assert_eq!(removed[0].as_str(), "success:1");
        assert_eq!(stack.len(), 1, "Critical 은 남아있어야 함");
    }

    /// tick() — 만료되지 않은 배너는 유지.
    #[test]
    fn tick_keeps_non_expired_banners() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("crash:1", Severity::Critical));
        stack.push(make_banner("lsp:1", Severity::Warning));

        let removed = stack.tick(Instant::now());
        assert!(removed.is_empty(), "만료된 배너 없음");
        assert_eq!(stack.len(), 2);
    }

    // ── contains / len / is_empty ──

    #[test]
    fn stack_contains_inserted_id() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("crash:1", Severity::Critical));
        assert!(stack.contains(&BannerId::new("crash:1")));
        assert!(!stack.contains(&BannerId::new("crash:99")));
    }

    #[test]
    fn stack_is_empty_initially() {
        let stack = BannerStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn stack_not_empty_after_push() {
        let mut stack = BannerStack::new();
        stack.push(make_banner("crash:1", Severity::Critical));
        assert!(!stack.is_empty());
    }

    // ── ActionButton 을 가진 배너 push ──

    #[test]
    fn push_banner_with_actions_preserves_actions() {
        let mut stack = BannerStack::new();
        let actions = vec![
            ActionButton::new("Reopen", "crash:reopen", true),
            ActionButton::new("Dismiss", "crash:dismiss", false),
        ];
        stack.push(make_banner_with_actions(
            "crash:1",
            Severity::Critical,
            actions.clone(),
        ));
        let entries = stack.entries();
        assert_eq!(entries[0].data.actions.len(), 2);
        assert_eq!(entries[0].data.actions[0].label, "Reopen");
        assert_eq!(entries[0].data.actions[1].label, "Dismiss");
    }

    // ── Max capacity 경계 ──

    /// MAX_BANNERS 가 3 이다.
    #[test]
    fn max_banners_constant_is_3() {
        assert_eq!(MAX_BANNERS, 3);
    }

    /// 4번째 push 시 evict 또는 drop — len 은 항상 MAX_BANNERS 이하.
    #[test]
    fn stack_never_exceeds_max_banners() {
        let mut stack = BannerStack::new();
        for i in 0..10 {
            stack.push(make_banner(&format!("warn:{i}"), Severity::Warning));
        }
        assert!(
            stack.len() <= MAX_BANNERS,
            "스택은 MAX_BANNERS({MAX_BANNERS}) 초과 불가"
        );
    }
}
