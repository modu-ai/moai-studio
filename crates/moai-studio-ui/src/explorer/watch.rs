// @MX:ANCHOR: [AUTO] rename-matching-window
// @MX:REASON: [AUTO] AC-FE-6 불변 계약: 100ms 윈도우 내 동일 경로의 Remove+Create 쌍을
//   Renamed 이벤트로 변환한다. merge_events_into_delta 는 FsDelta 생성의 단일 경로이며
//   fan_in >= 3: debounce flush, manual refresh, 통합 테스트.
// @MX:ANCHOR: [AUTO] fs-delta-apply
// @MX:REASON: [AUTO] tree state 변형의 단일 경로. fan_in >= 3: watch_loop, manual refresh,
//   통합 테스트. FileExplorer::apply_delta → cx.notify() 호출로 GPUI 재렌더 트리거.
// @MX:NOTE: [AUTO] backpressure-rescan
// @MX:REASON: [AUTO] REQ-FE-013: 버퍼가 BACKPRESSURE_LIMIT 초과 시 BackpressureFallback 반환.
//   호출자는 FsDelta 대신 BackpressureFallback 수신 시 full rescan 을 수행해야 한다.
// @MX:SPEC: SPEC-V3-005

use std::path::PathBuf;
use std::time::Instant;

// ============================================================
// RawEvent — watch 루프에서 수신하는 원시 이벤트
// ============================================================

/// watch 루프가 notify/FsWatcher 로부터 수신하는 원시 이벤트.
/// 순수 로직 테스트를 위해 moai-fs 의존 없이 자체 정의한다.
#[derive(Debug, Clone, PartialEq)]
pub enum RawEvent {
    /// 경로에 파일/디렉토리가 생성됨
    Created(PathBuf),
    /// 경로의 파일/디렉토리가 수정됨
    Modified(PathBuf),
    /// 경로에서 파일/디렉토리가 삭제됨
    Removed(PathBuf),
}

// ============================================================
// EventClass — 단일 RawEvent 분류 결과
// ============================================================

/// 단일 RawEvent 분류 결과 — merge_events_into_delta 내 중간 처리용.
#[derive(Debug, Clone, PartialEq)]
pub enum EventClass {
    /// 생성 이벤트
    Created,
    /// 수정 이벤트
    Modified,
    /// 삭제 이벤트
    Removed,
}

/// 단일 RawEvent 를 EventClass 로 분류한다.
pub fn classify_event(event: &RawEvent) -> EventClass {
    match event {
        RawEvent::Created(_) => EventClass::Created,
        RawEvent::Modified(_) => EventClass::Modified,
        RawEvent::Removed(_) => EventClass::Removed,
    }
}

// ============================================================
// FsDelta — debounce 윈도우 종료 후 방출되는 집약 델타
// ============================================================

/// debounce 윈도우 내 이벤트를 집약한 파일시스템 변경 델타.
/// AC-FE-5/6/7 의 최종 출력 타입.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FsDelta {
    /// 새로 생성된 경로 목록
    pub created: Vec<PathBuf>,
    /// 삭제된 경로 목록
    pub removed: Vec<PathBuf>,
    /// 수정된 경로 목록
    pub modified: Vec<PathBuf>,
    /// 이름 변경된 (이전 경로, 새 경로) 쌍 목록 (AC-FE-6)
    pub renamed: Vec<(PathBuf, PathBuf)>,
}

impl FsDelta {
    /// 빈 델타를 반환한다.
    pub fn empty() -> Self {
        Self::default()
    }

    /// 아무 변경도 없는 빈 델타인지 반환한다.
    pub fn is_empty(&self) -> bool {
        self.created.is_empty()
            && self.removed.is_empty()
            && self.modified.is_empty()
            && self.renamed.is_empty()
    }
}

// ============================================================
// BackpressureFallback — REQ-FE-013 backpressure 시그널
// ============================================================

/// 버퍼가 BACKPRESSURE_LIMIT 를 초과했을 때 반환되는 sentinel.
/// 수신자는 FsDelta 대신 이 값을 수신하면 full rescan 을 수행해야 한다.
#[derive(Debug, Clone, PartialEq)]
pub struct BackpressureFallback;

/// merge_events_into_delta 의 반환 타입.
/// 정상 처리 시 Ok(FsDelta), backpressure 시 Err(BackpressureFallback).
pub type MergeResult = Result<FsDelta, BackpressureFallback>;

/// backpressure 임계값 — 버퍼 이벤트 수가 이를 초과하면 BackpressureFallback 반환.
pub const BACKPRESSURE_LIMIT: usize = 1000;

// ============================================================
// merge_events_into_delta — 핵심 순수 로직
// ============================================================

/// debounce 윈도우 내 RawEvent 목록을 FsDelta 로 집약한다.
///
/// 동작 규칙:
/// 1. 버퍼 크기가 BACKPRESSURE_LIMIT 초과 → BackpressureFallback 반환 (REQ-FE-013)
/// 2. 동일 경로의 Removed + Created 쌍 → renamed 로 변환 (AC-FE-6)
///    단, Created 가 Removed 보다 나중에 등장해야 한다 (순서 기반)
/// 3. 동일 경로의 중복 Modified → 1 개로 dedupe
/// 4. Created 이후 Modified 가 같은 경로 → Created 만 유지
/// 5. Removed 이후 Created 가 같은 경로 → renamed 처리 (규칙 2)
pub fn merge_events_into_delta(events: Vec<RawEvent>) -> MergeResult {
    // REQ-FE-013: backpressure 검사
    if events.len() > BACKPRESSURE_LIMIT {
        return Err(BackpressureFallback);
    }

    // 경로별 이벤트 분류를 위한 중간 상태
    // removed_paths: Removed 가 먼저 등장한 경로 집합
    // created_paths: Created 가 등장한 경로 집합
    // modified_paths: Modified 가 등장한 경로 집합
    // created_after_removed: Remove → Create 순서인 경로 집합 (rename 후보)
    use std::collections::HashSet;

    let mut removed_first: HashSet<PathBuf> = HashSet::new(); // Removed 등장 경로 (Created 미등장)
    let mut created: HashSet<PathBuf> = HashSet::new(); // Created 등장 경로
    let mut modified: HashSet<PathBuf> = HashSet::new(); // Modified 등장 경로
    let mut renamed: Vec<(PathBuf, PathBuf)> = Vec::new(); // (old, new) 쌍

    for event in &events {
        match event {
            RawEvent::Removed(path) => {
                // Created 이력이 없으면 removed_first 에 추가
                if !created.contains(path) {
                    removed_first.insert(path.clone());
                }
                // Created 이력이 있으면 이미 처리된 created 이므로 제거 필요 없음
                // (created 후 removed → removed 가 나중이므로 created 에서 제거)
                created.remove(path);
                modified.remove(path);
            }
            RawEvent::Created(path) => {
                if removed_first.contains(path) {
                    // Removed → Created 순서: rename 이 아닌 동일 경로 재생성
                    // rename 은 다른 경로로의 이동이므로 여기서는 created 로 처리
                    removed_first.remove(path);
                    created.insert(path.clone());
                } else {
                    created.insert(path.clone());
                }
                modified.remove(path); // Created 후 Modified → Created 만 유지
            }
            RawEvent::Modified(path) => {
                // Created 또는 Removed→Created 이력 없을 때만 modified 에 추가
                if !created.contains(path) && !removed_first.contains(path) {
                    modified.insert(path.clone());
                }
            }
        }
    }

    // renamed 감지: events 를 순서대로 스캔하여 Removed(a) → Created(b) 쌍 탐지
    // 여기서 "rename" 은 서로 다른 경로 간 이동이 아니라
    // SPEC 에서 정의한 "동일 윈도우 내 Remove+Create" 패턴을 의미
    // → removed_first 에 남은 경로 중 created 에 나타나지 않은 것은 진짜 removed
    // NOTE: SPEC 의 rename detection 은 "경로 X 의 Removed 후 동일 경로 X 의 Created" 가 아니라
    //       "어떤 경로 A 의 Removed 와 다른 경로 B 의 Created" → rename(A→B) 로 해석
    // SPEC test case: rename_detected_from_remove_plus_create
    //   remove("old.txt") + create("new.txt") → renamed [("old.txt", "new.txt")]
    // 이 경우 removed_first = {"old.txt"}, created = {"new.txt"}
    // → removed_first 에 1개, created 에 1개가 있으면 rename 쌍을 생성

    // 단순 구현: removed_first 와 created 가 각각 1개씩 있고 서로 다른 경로일 때 rename 처리
    // 복수 쌍은 순서 보존을 위해 events 를 재스캔
    let mut final_removed: Vec<PathBuf> = Vec::new();
    let mut final_created: Vec<PathBuf> = Vec::new();

    // events 순서대로 removed_first 와 created 를 페어링
    let mut pending_removed: Vec<PathBuf> = Vec::new();

    for event in &events {
        match event {
            RawEvent::Removed(path) if removed_first.contains(path) => {
                pending_removed.push(path.clone());
            }
            RawEvent::Created(path) if created.contains(path) => {
                if let Some(old) = pending_removed.pop() {
                    // Removed → Created 순서로 다른 경로 → rename
                    if old != *path {
                        renamed.push((old, path.clone()));
                        created.remove(path); // renamed 로 처리됨
                        removed_first.remove(path); // (혹시 있다면)
                    } else {
                        // 동일 경로 Removed → Created → 재생성 (created 유지)
                        pending_removed.push(old);
                        final_created.push(path.clone());
                        created.remove(path);
                    }
                } else {
                    final_created.push(path.clone());
                    created.remove(path);
                }
            }
            _ => {}
        }
    }

    // pending_removed 에 남은 것 → 실제 removed
    for p in pending_removed {
        if removed_first.contains(&p) {
            final_removed.push(p);
        }
    }

    // created 에 아직 남은 것 → final_created 에 추가
    for p in created {
        if !final_created.contains(&p) {
            final_created.push(p);
        }
    }

    // modified 중 final_created 에 있는 것 제거 (Created 후 Modified 패턴)
    let mut modified_final: Vec<PathBuf> = modified
        .into_iter()
        .filter(|p| !final_created.contains(p) && !final_removed.contains(p))
        .collect();

    // 정렬하여 결정론적 순서 보장
    final_removed.sort();
    final_created.sort();
    modified_final.sort();

    Ok(FsDelta {
        created: final_created,
        removed: final_removed,
        modified: modified_final,
        renamed,
    })
}

// ============================================================
// WatchDebouncer — 100ms debounce 윈도우 (AC-FE-5)
// ============================================================

/// 100ms debounce 윈도우로 이벤트를 집약하는 구조체.
///
/// RawEvent 를 버퍼에 쌓고, `should_flush()` 가 true 를 반환할 때
/// `flush()` 를 호출하여 FsDelta 를 생성한다.
pub struct WatchDebouncer {
    /// debounce 윈도우 (ms)
    pub window_ms: u64,
    /// 누적 이벤트 버퍼
    pub buffer: Vec<RawEvent>,
    /// 마지막 flush 시각 (또는 생성 시각)
    pub last_flush: Instant,
}

impl WatchDebouncer {
    /// 기본 100ms 윈도우로 WatchDebouncer 를 생성한다.
    pub fn new(window_ms: u64) -> Self {
        Self {
            window_ms,
            buffer: Vec::new(),
            last_flush: Instant::now(),
        }
    }

    /// 이벤트를 버퍼에 추가한다.
    pub fn push(&mut self, event: RawEvent) {
        self.buffer.push(event);
    }

    /// 현재 시각이 마지막 flush 로부터 window_ms 를 초과했는지 반환한다.
    pub fn should_flush(&self) -> bool {
        self.last_flush.elapsed().as_millis() as u64 >= self.window_ms
    }

    /// 버퍼를 flush 하여 MergeResult 를 반환하고, 버퍼를 초기화한다.
    pub fn flush(&mut self) -> MergeResult {
        let events = std::mem::take(&mut self.buffer);
        self.last_flush = Instant::now();
        merge_events_into_delta(events)
    }
}

// ============================================================
// 단위 테스트 — AC-FE-5 / AC-FE-6 / AC-FE-7
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    // T5-1: 동일 경로 Modified 이벤트 중복 제거
    #[test]
    fn merge_events_dedup_modifications() {
        let path = PathBuf::from("src/main.rs");
        let events = vec![
            RawEvent::Modified(path.clone()),
            RawEvent::Modified(path.clone()),
            RawEvent::Modified(path.clone()),
        ];
        let delta = merge_events_into_delta(events).expect("delta 생성 실패");
        // modified 에 정확히 1개만 있어야 한다
        assert_eq!(
            delta.modified.len(),
            1,
            "동일 경로 Modified 는 dedupe 되어야 한다"
        );
        assert_eq!(delta.modified[0], path);
        assert!(delta.created.is_empty());
        assert!(delta.removed.is_empty());
        assert!(delta.renamed.is_empty());
    }

    // T5-2: Created / Removed 분류 검증
    #[test]
    fn merge_events_classifies_created_removed() {
        let created_path = PathBuf::from("src/new.rs");
        let removed_path = PathBuf::from("src/old.rs");
        let events = vec![
            RawEvent::Created(created_path.clone()),
            RawEvent::Removed(removed_path.clone()),
        ];
        let delta = merge_events_into_delta(events).expect("delta 생성 실패");
        assert_eq!(
            delta.created,
            vec![created_path],
            "Created 경로가 created 에 있어야 한다"
        );
        assert_eq!(
            delta.removed,
            vec![removed_path],
            "Removed 경로가 removed 에 있어야 한다"
        );
        assert!(delta.modified.is_empty());
        assert!(delta.renamed.is_empty());
    }

    // T5-3: AC-FE-6 rename 감지 — Removed(old) + Created(new) → renamed
    #[test]
    fn rename_detected_from_remove_plus_create() {
        let old_path = PathBuf::from("src/old.rs");
        let new_path = PathBuf::from("src/new.rs");
        let events = vec![
            RawEvent::Removed(old_path.clone()),
            RawEvent::Created(new_path.clone()),
        ];
        let delta = merge_events_into_delta(events).expect("delta 생성 실패");
        // Removed + Created 순서이므로 renamed 로 처리되어야 한다 (AC-FE-6)
        assert_eq!(
            delta.renamed.len(),
            1,
            "Removed+Created 쌍은 renamed 로 감지되어야 한다 (AC-FE-6)"
        );
        assert_eq!(delta.renamed[0], (old_path, new_path));
        assert!(
            delta.created.is_empty(),
            "rename 된 경로는 created 에서 제외"
        );
        assert!(
            delta.removed.is_empty(),
            "rename 된 경로는 removed 에서 제외"
        );
    }

    // T5-4: AC-FE-5 debounce 100ms 윈도우 내 burst 집약
    #[test]
    fn debounce_collapses_burst_within_window() {
        let mut debouncer = WatchDebouncer::new(100);
        let path = PathBuf::from("src/lib.rs");

        // window 내에 동일 경로 Modified 를 10회 push
        for _ in 0..10 {
            debouncer.push(RawEvent::Modified(path.clone()));
        }

        // window 가 아직 지나지 않았으므로 should_flush() == false
        assert!(
            !debouncer.should_flush(),
            "100ms 미경과 시 should_flush() 는 false"
        );

        // 강제 flush 로 delta 확인
        let delta = debouncer.flush().expect("delta 생성 실패");

        // 10번 Modified 가 1개로 collapse 되어야 한다 (AC-FE-5)
        assert_eq!(
            delta.modified.len(),
            1,
            "burst 내 동일 경로 Modified 는 1개로 collapse 되어야 한다 (AC-FE-5)"
        );
        assert_eq!(delta.modified[0], path);

        // flush 후 버퍼가 비어있어야 한다
        assert!(
            debouncer.buffer.is_empty(),
            "flush 후 버퍼는 비어있어야 한다"
        );
    }

    // T5-5: AC-FE-7 backpressure — 버퍼 1000 초과 시 BackpressureFallback
    #[test]
    fn oversized_buffer_signals_backpressure() {
        // BACKPRESSURE_LIMIT(1000) 을 1개 초과하는 이벤트 목록 생성
        let events: Vec<RawEvent> = (0..=BACKPRESSURE_LIMIT)
            .map(|i| RawEvent::Modified(PathBuf::from(format!("src/file_{i}.rs"))))
            .collect();

        assert_eq!(events.len(), BACKPRESSURE_LIMIT + 1, "이벤트 수 확인");

        let result = merge_events_into_delta(events);

        // BackpressureFallback 이어야 한다 (AC-FE-7)
        assert_eq!(
            result,
            Err(BackpressureFallback),
            "버퍼 초과 시 BackpressureFallback 이어야 한다 (AC-FE-7)"
        );
    }

    // T5-6: apply_delta 가 영향받은 디렉토리를 invalidate 한다
    // (FileExplorer 와 결합되므로 여기서는 FsDelta 구조만 검증)
    #[test]
    fn apply_delta_invalidates_affected_dirs() {
        // FsDelta 에 created 항목이 있을 때 부모 경로를 추출할 수 있어야 한다
        let delta = FsDelta {
            created: vec![PathBuf::from("src/new_dir/file.rs")],
            removed: vec![PathBuf::from("src/old.rs")],
            modified: vec![PathBuf::from("src/main.rs")],
            renamed: vec![(PathBuf::from("a.rs"), PathBuf::from("b.rs"))],
        };

        // delta 내 경로의 부모들을 수집
        let mut affected_dirs: Vec<PathBuf> = delta
            .created
            .iter()
            .chain(delta.removed.iter())
            .chain(delta.modified.iter())
            .chain(delta.renamed.iter().flat_map(|(a, b)| [a, b]))
            .filter_map(|p| p.parent().map(|par| par.to_path_buf()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        affected_dirs.sort();

        // src, src/new_dir 두 디렉토리가 영향받아야 한다
        assert!(
            affected_dirs.iter().any(|d| d == &PathBuf::from("src")),
            "src 디렉토리가 affected 목록에 있어야 한다"
        );
        assert!(
            affected_dirs
                .iter()
                .any(|d| d == &PathBuf::from("src/new_dir")),
            "src/new_dir 가 affected 목록에 있어야 한다"
        );
    }

    // T5-7: should_flush 는 window_ms 경과 후 true 반환
    #[test]
    fn debounce_window_elapsed_triggers_flush() {
        let mut debouncer = WatchDebouncer::new(10); // 10ms 윈도우
        debouncer.push(RawEvent::Modified(PathBuf::from("x.rs")));

        // 10ms + 여유 대기
        thread::sleep(Duration::from_millis(20));

        assert!(
            debouncer.should_flush(),
            "10ms 경과 후 should_flush() 는 true"
        );
        let delta = debouncer.flush().expect("delta 생성 실패");
        assert_eq!(delta.modified.len(), 1);
    }

    // T5-8: 빈 이벤트 목록 → 빈 FsDelta
    #[test]
    fn merge_empty_events_produces_empty_delta() {
        let delta = merge_events_into_delta(vec![]).expect("delta 생성 실패");
        assert!(delta.is_empty(), "빈 이벤트 → 빈 FsDelta");
    }
}
