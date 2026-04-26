//! SPEC-V3-009 RG-SU-1 — 파일 변경 감시 + debounce (AC-SU-4, AC-SU-5).
//!
//! REQ-SU-003: spec.md / progress.md 가 외부에서 변경되면 100ms debounce 후
//!             해당 SpecRecord 를 재파싱하고 `cx.notify()` 를 호출한다.
//!
//! MS-1 구현: polling 방식 (1s 간격) — notify crate 도입은 MS-2/MS-3.
//! notify crate 의존을 최소화하기 위해 파일 mtime polling 으로 단순화한다.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::info;

use crate::state::SpecId;

/// 파일 변경 이벤트.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecChangeEvent {
    /// 변경된 SPEC ID
    pub spec_id: SpecId,
    /// 변경된 파일 경로
    pub path: PathBuf,
    /// 변경 감지 시각
    pub detected_at: SystemTime,
}

/// SPEC 디렉터리 변경 감시기 (polling 기반, MS-1).
///
/// `start()` 를 호출하면 tokio task 가 spawn 되어 백그라운드에서
/// 지정된 파일 목록의 mtime 변화를 감지한다.
pub struct SpecWatcher {
    /// 감시 파일: (SPEC ID, 파일 경로)
    watched: Vec<(SpecId, PathBuf)>,
    /// 폴링 간격 (default: 1s)
    poll_interval: Duration,
}

impl SpecWatcher {
    /// 새 SpecWatcher 를 생성한다.
    pub fn new() -> Self {
        Self {
            watched: Vec::new(),
            poll_interval: Duration::from_secs(1),
        }
    }

    /// 폴링 간격을 설정한다 (테스트에서 단축 가능).
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// 감시 파일을 추가한다.
    pub fn watch(&mut self, spec_id: SpecId, path: PathBuf) {
        self.watched.push((spec_id, path));
    }

    /// `.moai/specs/` 디렉터리 내 모든 SPEC 의 spec.md / progress.md 를
    /// 자동 등록한다 (REQ-SU-003 대상 파일).
    pub fn watch_specs_dir(&mut self, specs_dir: &Path, spec_ids: &[SpecId]) {
        for id in spec_ids {
            let spec_dir = specs_dir.join(id.as_str());
            let spec_path = spec_dir.join("spec.md");
            let progress_path = spec_dir.join("progress.md");

            if spec_path.exists() {
                self.watch(id.clone(), spec_path);
            }
            if progress_path.exists() {
                self.watch(id.clone(), progress_path);
            }
        }
    }

    /// 백그라운드 polling task 를 시작한다.
    ///
    /// 변경이 감지되면 `sender` 로 `SpecChangeEvent` 를 전송한다.
    /// task 는 `drop(sender)` 되거나 채널이 닫힐 때 자동 종료된다.
    ///
    /// 반환값은 task join handle (abort 가능).
    pub fn start(self, sender: mpsc::Sender<SpecChangeEvent>) -> tokio::task::JoinHandle<()> {
        let watched = self.watched;
        let poll_interval = self.poll_interval;

        tokio::task::spawn(async move {
            // 초기 mtime 스냅샷
            let mut mtimes: HashMap<PathBuf, SystemTime> = HashMap::new();
            for (_, path) in &watched {
                if let Ok(meta) = std::fs::metadata(path)
                    && let Ok(mtime) = meta.modified()
                {
                    mtimes.insert(path.clone(), mtime);
                }
            }

            loop {
                sleep(poll_interval).await;

                // 채널 닫힌 경우 task 종료 (변경이 없어도 매 poll 마다 확인)
                if sender.is_closed() {
                    info!("SpecWatcher: 채널 닫힘, task 종료");
                    return;
                }

                for (spec_id, path) in &watched {
                    let current_mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();

                    let last = mtimes.get(path).copied();

                    let changed = match (current_mtime, last) {
                        (Some(curr), Some(prev)) => curr != prev,
                        (Some(_), None) => true, // 새 파일
                        _ => false,
                    };

                    if changed {
                        // mtime 갱신
                        if let Some(t) = current_mtime {
                            mtimes.insert(path.clone(), t);
                        }

                        let event = SpecChangeEvent {
                            spec_id: spec_id.clone(),
                            path: path.clone(),
                            detected_at: SystemTime::now(),
                        };

                        info!("SPEC 파일 변경 감지: {:?}", event.path);

                        // 채널 닫힌 경우 task 종료
                        if sender.send(event).await.is_err() {
                            info!("SpecWatcher: 수신자 없음, task 종료");
                            return;
                        }
                    }
                }
            }
        })
    }
}

impl Default for SpecWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::timeout;

    fn make_tmp_spec_file() -> (TempDir, PathBuf) {
        let tmp = tempfile::tempdir().expect("tempdir 생성 실패");
        let path = tmp.path().join("spec.md");
        fs::write(&path, "initial content").unwrap();
        (tmp, path)
    }

    #[tokio::test]
    async fn watcher_detects_file_change() {
        // 파일 변경 감지 smoke 테스트 (AC-SU-4 관련)
        let (_tmp, path) = make_tmp_spec_file();

        let (tx, mut rx) = mpsc::channel::<SpecChangeEvent>(8);
        let spec_id = SpecId::new("SPEC-TEST-001");

        let mut watcher = SpecWatcher::new().with_poll_interval(Duration::from_millis(50));
        watcher.watch(spec_id.clone(), path.clone());

        let handle = watcher.start(tx);

        // 약간 대기 후 파일 변경
        sleep(Duration::from_millis(80)).await;
        fs::write(&path, "modified content").unwrap();

        // 변경 이벤트가 200ms 이내에 도달해야 한다
        let result = timeout(Duration::from_millis(300), rx.recv()).await;
        assert!(result.is_ok(), "timeout 내에 이벤트를 수신해야 함");
        let event = result.unwrap().expect("채널에서 이벤트를 받아야 함");
        assert_eq!(event.spec_id, spec_id);
        assert_eq!(event.path, path);

        handle.abort();
    }

    #[tokio::test]
    async fn watcher_no_event_on_no_change() {
        let (_tmp, path) = make_tmp_spec_file();

        let (tx, mut rx) = mpsc::channel::<SpecChangeEvent>(8);
        let mut watcher = SpecWatcher::new().with_poll_interval(Duration::from_millis(50));
        watcher.watch(SpecId::new("SPEC-NO-CHANGE"), path.clone());

        let handle = watcher.start(tx);

        // 파일 변경 없이 대기
        let result = timeout(Duration::from_millis(150), rx.recv()).await;
        // timeout 이어야 함 (변경 없음)
        assert!(result.is_err(), "변경 없으면 timeout 이어야 함");

        handle.abort();
    }

    #[tokio::test]
    async fn watcher_terminates_when_channel_closed() {
        let (_tmp, path) = make_tmp_spec_file();

        let (tx, rx) = mpsc::channel::<SpecChangeEvent>(1);
        let mut watcher = SpecWatcher::new().with_poll_interval(Duration::from_millis(20));
        watcher.watch(SpecId::new("SPEC-CLOSE"), path.clone());

        let handle = watcher.start(tx);

        // 수신자 drop → 채널 닫힘 → task 종료
        drop(rx);

        // task 가 종료되어야 한다 (500ms 내)
        let join_result = timeout(Duration::from_millis(500), handle).await;
        assert!(join_result.is_ok(), "채널 닫히면 task 가 종료되어야 함");
    }

    #[test]
    fn watcher_watch_specs_dir_registers_existing_files() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_dir = tmp.path().join("SPEC-V3-TEST");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("spec.md"), "content").unwrap();
        // progress.md 는 없음

        let mut watcher = SpecWatcher::new();
        let ids = vec![SpecId::new("SPEC-V3-TEST")];
        watcher.watch_specs_dir(tmp.path(), &ids);

        // spec.md 만 등록되어야 함
        assert_eq!(watcher.watched.len(), 1);
        assert!(watcher.watched[0].1.ends_with("spec.md"));
    }
}
