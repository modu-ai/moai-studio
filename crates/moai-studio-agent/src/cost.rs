//! CostTracker — API self-report USD 비용 집계 (RG-AD-3, AC-AD-5/6)
//!
//! SPEC-V3-010 REQ-AD-014: usage 필드 → CostSnapshot 추출.
//! SPEC-V3-010 REQ-AD-017: 로컬 token×price 계산 절대 금지. API 제공 usd 만 사용.
//!
//! @MX:ANCHOR: [AUTO] cost-tracker-domain
//! @MX:REASON: [AUTO] 비용 집계 단일 진실 원천. fan_in >= 3:
//!   CostPanelView, DashboardView, 테스트 코드.
//!   SPEC: SPEC-V3-010 RG-AD-3, REQ-AD-014, REQ-AD-017

// @MX:NOTE: [AUTO] cost-extraction-api-only
// @MX:SPEC: SPEC-V3-010 REQ-AD-014/017
// API self-report cost_usd 만 기록. token×price 로컬 계산 금지 (REQ-AD-017 HARD).
// usage 필드 없는 이벤트는 None 반환 — 기록하지 않는다.

use std::time::{Duration, SystemTime};

use crate::events::{AgentRunId, StreamJsonEvent};

/// API self-report 비용 스냅샷 (REQ-AD-014).
///
/// REQ-AD-017: `usd` 는 반드시 API 가 제공한 값이어야 한다.
/// token×price 로컬 계산으로 채워서는 안 된다.
#[derive(Debug, Clone)]
pub struct CostSnapshot {
    /// 스냅샷 시각 (UTC)
    pub timestamp: SystemTime,
    /// API self-report USD 비용 (REQ-AD-014)
    pub usd: f64,
    /// 비용이 귀속되는 run
    pub run_id: AgentRunId,
}

/// 비용 스냅샷 목록 집계기 (AC-AD-5/6).
///
/// @MX:ANCHOR: [AUTO] cost-tracker-impl
/// @MX:REASON: [AUTO] session/daily/weekly 집계 공개 API. fan_in >= 3:
///   CostPanelView, DashboardView, 통합 테스트.
pub struct CostTracker {
    snapshots: Vec<CostSnapshot>,
}

impl CostTracker {
    /// 새 CostTracker 를 생성한다.
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    /// 스냅샷을 기록한다.
    pub fn record(&mut self, snap: CostSnapshot) {
        self.snapshots.push(snap);
    }

    /// 특정 run 의 총 USD 비용을 반환한다 (AC-AD-5).
    pub fn session_total(&self, run_id: &AgentRunId) -> f64 {
        self.snapshots
            .iter()
            .filter(|s| &s.run_id == run_id)
            .map(|s| s.usd)
            .sum()
    }

    /// 지정한 시각이 속한 날(UTC)의 총 USD 비용을 반환한다 (AC-AD-6).
    ///
    /// `day` 와 동일한 UTC 날짜에 해당하는 스냅샷만 합산한다.
    pub fn daily_total(&self, day: SystemTime) -> f64 {
        let day_secs = system_time_to_unix_secs(day);
        let day_start = day_secs - (day_secs % 86400);
        let day_end = day_start + 86400;

        self.snapshots
            .iter()
            .filter(|s| {
                let t = system_time_to_unix_secs(s.timestamp);
                t >= day_start && t < day_end
            })
            .map(|s| s.usd)
            .sum()
    }

    /// `week_start` 로부터 7일(604800초) 이내의 총 USD 비용을 반환한다 (AC-AD-6).
    pub fn weekly_total(&self, week_start: SystemTime) -> f64 {
        let start_secs = system_time_to_unix_secs(week_start);
        let end_secs = start_secs + 7 * 86400;

        self.snapshots
            .iter()
            .filter(|s| {
                let t = system_time_to_unix_secs(s.timestamp);
                t >= start_secs && t < end_secs
            })
            .map(|s| s.usd)
            .sum()
    }
}

impl Default for CostTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// StreamJsonEvent 에서 CostSnapshot 을 추출한다 (REQ-AD-014).
///
/// usage 필드가 없거나 cost_usd 가 None 이면 None 을 반환한다.
///
/// REQ-AD-017: cost_usd 가 0 이어도 기록한다 — 0 도 API 가 제공한 값이다.
pub fn extract_from_stream_json(
    event: &StreamJsonEvent,
    run_id: AgentRunId,
) -> Option<CostSnapshot> {
    let usd = event.usage.as_ref()?.cost_usd?;
    Some(CostSnapshot {
        timestamp: SystemTime::now(),
        usd,
        run_id,
    })
}

// ----------------------------------------------------------------
// 내부 헬퍼
// ----------------------------------------------------------------

/// SystemTime → Unix 초 변환. UNIX_EPOCH 이전이면 0 반환.
fn system_time_to_unix_secs(t: SystemTime) -> u64 {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Unix 초 → SystemTime 변환 (테스트 헬퍼로도 사용).
pub fn unix_secs_to_system_time(secs: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
}

// ================================================================
// 테스트 (RED-GREEN 사이클)
// ================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{StreamJsonEvent, TokenUsage};

    fn run_id(s: &str) -> AgentRunId {
        AgentRunId(s.to_string())
    }

    /// 스냅샷을 기록하고 동일 run_id 합산이 맞는지 확인 (AC-AD-5)
    #[test]
    fn record_and_session_total() {
        let mut tracker = CostTracker::new();
        let id = run_id("r1");

        tracker.record(CostSnapshot {
            timestamp: SystemTime::UNIX_EPOCH,
            usd: 0.01,
            run_id: id.clone(),
        });
        tracker.record(CostSnapshot {
            timestamp: SystemTime::UNIX_EPOCH,
            usd: 0.04,
            run_id: id.clone(),
        });

        let total = tracker.session_total(&id);
        assert!((total - 0.05).abs() < 1e-9, "expected 0.05 got {total}");
    }

    /// 다른 run_id 는 session_total 에 포함되지 않아야 한다
    #[test]
    fn session_total_filters_by_run_id() {
        let mut tracker = CostTracker::new();
        let r1 = run_id("r1");
        let r2 = run_id("r2");

        tracker.record(CostSnapshot {
            timestamp: SystemTime::UNIX_EPOCH,
            usd: 0.10,
            run_id: r1.clone(),
        });
        tracker.record(CostSnapshot {
            timestamp: SystemTime::UNIX_EPOCH,
            usd: 0.99,
            run_id: r2.clone(),
        });

        let t1 = tracker.session_total(&r1);
        let t2 = tracker.session_total(&r2);

        assert!((t1 - 0.10).abs() < 1e-9, "r1 expected 0.10 got {t1}");
        assert!((t2 - 0.99).abs() < 1e-9, "r2 expected 0.99 got {t2}");
    }

    /// 같은 날의 스냅샷 3개 합산 (AC-AD-6)
    #[test]
    fn daily_total_aggregates_same_day() {
        let mut tracker = CostTracker::new();
        let id = run_id("r1");
        // 2024-01-01 00:00:00 UTC = 1704067200
        let day_start = unix_secs_to_system_time(1_704_067_200);
        // 당일 내 세 시각: +1h, +12h, +23h
        for offset in [3600u64, 43200, 82800] {
            tracker.record(CostSnapshot {
                timestamp: unix_secs_to_system_time(1_704_067_200 + offset),
                usd: 0.01,
                run_id: id.clone(),
            });
        }

        let total = tracker.daily_total(day_start);
        assert!((total - 0.03).abs() < 1e-9, "expected 0.03 got {total}");
    }

    /// 다른 날 스냅샷은 daily_total 에 포함되지 않아야 한다
    #[test]
    fn daily_total_excludes_other_days() {
        let mut tracker = CostTracker::new();
        let id = run_id("r1");
        // 2024-01-01 12:00 UTC
        let day = unix_secs_to_system_time(1_704_067_200 + 43200);
        // 다음날 스냅샷 (2024-01-02 01:00)
        tracker.record(CostSnapshot {
            timestamp: unix_secs_to_system_time(1_704_067_200 + 86400 + 3600),
            usd: 0.99,
            run_id: id.clone(),
        });
        // 당일 스냅샷
        tracker.record(CostSnapshot {
            timestamp: day,
            usd: 0.05,
            run_id: id.clone(),
        });

        let total = tracker.daily_total(day);
        assert!((total - 0.05).abs() < 1e-9, "expected 0.05 got {total}");
    }

    /// weekly_total: week_start 로부터 7일(포함) 이내 합산
    #[test]
    fn weekly_total_within_7_days_inclusive() {
        let mut tracker = CostTracker::new();
        let id = run_id("r1");
        // week_start = 2024-01-01 00:00 UTC
        let week_start = unix_secs_to_system_time(1_704_067_200);

        // 7일 내: day 1, day 3, day 6 (+0, +2d, +5d)
        for day_offset in [0u64, 2 * 86400, 5 * 86400] {
            tracker.record(CostSnapshot {
                timestamp: unix_secs_to_system_time(1_704_067_200 + day_offset + 3600),
                usd: 0.10,
                run_id: id.clone(),
            });
        }
        // 7일 경계 밖: day 7 (정확히 7*86400 = 604800 초 후)
        tracker.record(CostSnapshot {
            timestamp: unix_secs_to_system_time(1_704_067_200 + 7 * 86400 + 1),
            usd: 0.50,
            run_id: id.clone(),
        });

        let total = tracker.weekly_total(week_start);
        assert!((total - 0.30).abs() < 1e-9, "expected 0.30 got {total}");
    }

    /// usage{cost_usd: 0.05} 이벤트 → snapshot 추출 성공 (REQ-AD-014)
    #[test]
    fn extract_with_usage_returns_snapshot() {
        let event = StreamJsonEvent {
            type_: "result".to_string(),
            payload: serde_json::Value::Null,
            usage: Some(TokenUsage {
                input_tokens: 100,
                output_tokens: 50,
                cost_usd: Some(0.05),
            }),
        };

        let snap = extract_from_stream_json(&event, run_id("r1"));
        let snap = snap.expect("usage 있으면 Some 이어야 한다");
        assert!((snap.usd - 0.05).abs() < 1e-9, "usd mismatch");
        assert_eq!(snap.run_id, run_id("r1"));
    }

    /// usage 없는 이벤트 → None 반환
    #[test]
    fn extract_without_usage_returns_none() {
        let event = StreamJsonEvent {
            type_: "assistant".to_string(),
            payload: serde_json::Value::Null,
            usage: None,
        };

        let result = extract_from_stream_json(&event, run_id("r1"));
        assert!(result.is_none(), "usage 없으면 None 이어야 한다");
    }

    /// usd=0 이어도 기록해야 한다 (REQ-AD-017: 0 도 API 제공값)
    #[test]
    fn extract_with_zero_cost_returns_snapshot() {
        let event = StreamJsonEvent {
            type_: "result".to_string(),
            payload: serde_json::Value::Null,
            usage: Some(TokenUsage {
                input_tokens: 0,
                output_tokens: 0,
                cost_usd: Some(0.0),
            }),
        };

        let snap = extract_from_stream_json(&event, run_id("r1"));
        let snap = snap.expect("cost_usd=0 도 Some 이어야 한다");
        assert_eq!(snap.usd, 0.0);
    }
}
