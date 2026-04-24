//! `TabContainer` 구현 + 탭 생성/전환/닫기 로직 + last_focused_pane 복원.
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-5 (REQ-P-040 ~ REQ-P-045)
//! - spec.md §5 RG-P-3 REQ-P-023 (탭 전환 시 last_focused_pane 복원)
//! - spec.md §5 RG-P-4 REQ-P-034 (tmux 중첩 시 OS/GPUI 레벨 우선 — AC-P-26)

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::panes::{GpuiNativeSplitter, PaneId, PaneSplitter};

/// 탭 ID 충돌 방지용 원자적 시퀀스 카운터.
///
/// nanos 가 동일한 경우 (고속 연속 생성) 카운터로 구분한다.
static TAB_SEQ: AtomicU64 = AtomicU64::new(0);

// ============================================================
// TabId
// ============================================================

/// 탭을 고유하게 식별하는 ID.
///
/// Spike 3 결정: `format!("tab-{:x}", nanos)` — workspace / pane ID 패턴 차용.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TabId(pub String);

impl TabId {
    /// 나노초 + 원자적 시퀀스 기반 고유 TabId 생성 (Spike 3 패턴 재사용 + 충돌 방지).
    ///
    /// 고속 연속 생성 시 nanos 가 같을 수 있어 seq 를 접미사로 추가한다.
    pub fn new_unique() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = TAB_SEQ.fetch_add(1, Ordering::Relaxed);
        Self(format!("tab-{:x}-{:x}", nanos, seq))
    }
}

impl std::fmt::Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================
// TabError
// ============================================================

/// TabContainer 조작 시 발생할 수 있는 에러.
#[derive(Debug, PartialEq, Eq)]
pub enum TabError {
    /// 탭이 1개만 남아있을 때 close_tab 호출 → no-op (AC-P-10).
    LastTabCloseNoop,
    /// switch_tab / close_tab 에서 idx 가 범위를 벗어남 (AC-P-25 negative).
    IndexOutOfBounds,
    /// split_horizontal / split_vertical 에서 target pane 이 트리에 없음.
    SplitTargetNotFound,
}

impl From<crate::panes::SplitError> for TabError {
    fn from(e: crate::panes::SplitError) -> Self {
        match e {
            crate::panes::SplitError::TargetNotFound => TabError::SplitTargetNotFound,
            crate::panes::SplitError::MinSizeViolated => TabError::SplitTargetNotFound,
        }
    }
}

// ============================================================
// Tab
// ============================================================

/// 단일 탭: TabId + 타이틀 + GpuiNativeSplitter (pane tree + factory) + last_focused_pane 복원.
///
/// ## REQ-P-040
///
/// 탭 타이틀 초기값은 `cwd.file_name()` 또는 "untitled".
///
/// ## REQ-P-023
///
/// `last_focused_pane` 은 탭 전환 시 저장되고, 복귀 시 해당 pane 으로 focus 를 복원한다 (AC-P-11).
pub struct Tab<L: Clone + 'static> {
    /// 탭 고유 식별자.
    pub id: TabId,
    /// 탭 타이틀 (초기값: cwd.file_name() 또는 "untitled").
    pub title: String,
    /// 탭의 독립 pane tree + factory 래핑.
    pub splitter: GpuiNativeSplitter<L>,
    // @MX:NOTE: [AUTO] last-focused-pane-restoration
    // REQ-P-023: 탭 전환 시 이전 탭의 포커스 상태를 저장하고,
    // 복귀 시 last_focused_pane 으로 splitter.focus_pane() 을 호출해 복원한다.
    // switch_tab 에서 (1) 이전 탭 last_focused 저장, (2) 새 탭 last_focused 복원 두 단계 실행.
    /// 탭을 마지막으로 떠날 때의 focused PaneId. 탭 복귀 시 복원 (AC-P-11).
    pub last_focused_pane: Option<PaneId>,
}

// ============================================================
// TabContainer
// ============================================================

// @MX:ANCHOR: [AUTO] tab-create-api
// @MX:REASON: [AUTO] 탭 생성 API 의 단일 진입점.
//   fan_in >= 3: T9 Cmd/Ctrl+T 핸들러, T13 persistence 복원, RootView::new 초기화.
//   최소 1 탭 불변 조건 (MS-2 invariant): TabContainer 는 항상 최소 1 개의 탭을 보유한다.
/// N 개의 Tab 을 소유하고 active_tab_idx 를 관리하는 컨테이너.
///
/// ## MS-2 Invariant
///
/// `tabs.len() >= 1` 은 항상 성립한다. `close_tab` 은 마지막 탭이면 `LastTabCloseNoop` 를 반환한다.
///
/// ## 제네릭 파라미터
///
/// - `L: Clone + 'static`: leaf payload 타입.
///   - prod: `Entity<TerminalSurface>` (lib.rs wire-up)
///   - test: `String` 또는 `Arc<Mutex<i32>>`
pub struct TabContainer<L: Clone + 'static> {
    /// 탭 목록. `tabs.len() >= 1` 불변 조건.
    pub tabs: Vec<Tab<L>>,
    /// 현재 활성 탭 인덱스. 항상 `< tabs.len()`.
    pub active_tab_idx: usize,
}

impl<L: Clone + 'static> TabContainer<L> {
    /// 초기 탭 1 개로 TabContainer 를 생성한다 (MS-2 최소 1 탭 불변 조건).
    ///
    /// # Arguments
    ///
    /// - `initial_tab`: 첫 번째 탭. 반드시 1 개 이상이어야 한다.
    pub fn new(initial_tab: Tab<L>) -> Self {
        Self {
            tabs: vec![initial_tab],
            active_tab_idx: 0,
        }
    }

    // @MX:ANCHOR: [AUTO] tab-create-api — see struct-level ANCHOR above
    /// 새 탭을 추가하고 active_tab_idx 를 새 탭으로 이동한다 (AC-P-8).
    ///
    /// 기존 active 탭의 `last_focused_pane` 을 저장 후 새 탭으로 전환한다.
    ///
    /// # Arguments
    ///
    /// - `title`: 새 탭의 타이틀.
    /// - `splitter`: 새 탭의 GpuiNativeSplitter.
    pub fn new_tab(&mut self, title: String, splitter: GpuiNativeSplitter<L>) {
        // 기존 active 탭의 last_focused_pane 저장
        let current_focused = self.tabs[self.active_tab_idx].splitter.focused().cloned();
        self.tabs[self.active_tab_idx].last_focused_pane = current_focused;

        let tab = Tab {
            id: TabId::new_unique(),
            title,
            splitter,
            last_focused_pane: None,
        };
        self.tabs.push(tab);
        self.active_tab_idx = self.tabs.len() - 1;
    }

    // @MX:ANCHOR: [AUTO] tab-switch-invariant
    // @MX:REASON: [AUTO] 탭 전환의 단일 진입점이며 last_focused_pane 복원 로직을 보유한다.
    //   fan_in >= 3: T9 Cmd/Ctrl+1~9 dispatcher, T13 persistence restore, RootView render.
    //   AC-P-11: 이전 탭 포커스 저장 + 새 탭 포커스 복원이 동시에 실행되어야 한다.
    /// active 탭을 `idx` 로 전환하고 last_focused_pane 복원을 수행한다 (AC-P-11).
    ///
    /// ## 동작
    ///
    /// 1. 현재 active 탭의 focused pane 을 `last_focused_pane` 에 저장.
    /// 2. `active_tab_idx` 를 `idx` 로 변경.
    /// 3. 새 탭의 `last_focused_pane` 이 `Some(id)` 이면 `splitter.focus_pane(id)` 호출.
    ///
    /// # Errors
    ///
    /// - `TabError::IndexOutOfBounds`: `idx >= tabs.len()`.
    pub fn switch_tab(&mut self, idx: usize) -> Result<(), TabError> {
        if idx >= self.tabs.len() {
            return Err(TabError::IndexOutOfBounds);
        }
        if idx == self.active_tab_idx {
            return Ok(());
        }

        // 1. 현재 탭 포커스 저장
        let current_focused = self.tabs[self.active_tab_idx].splitter.focused().cloned();
        self.tabs[self.active_tab_idx].last_focused_pane = current_focused;

        // 2. active 전환
        self.active_tab_idx = idx;

        // 3. 새 탭 last_focused 복원
        if let Some(restore_id) = self.tabs[self.active_tab_idx].last_focused_pane.clone() {
            self.tabs[self.active_tab_idx]
                .splitter
                .focus_pane(restore_id);
        }

        Ok(())
    }

    /// 지정된 `idx` 의 탭을 닫는다.
    ///
    /// ## 동작
    ///
    /// - 탭이 1 개 뿐이면 `LastTabCloseNoop` Err 반환 (AC-P-10).
    /// - active 탭이 닫힐 경우 right neighbor 로 승격 (없으면 left neighbor) (AC-P-10).
    /// - `active_tab_idx` 를 조정하여 항상 유효한 인덱스를 가리킨다 (AC-P-25).
    ///
    /// # Errors
    ///
    /// - `TabError::LastTabCloseNoop`: 탭이 1 개뿐이어서 닫을 수 없음.
    /// - `TabError::IndexOutOfBounds`: `idx >= tabs.len()`.
    pub fn close_tab(&mut self, idx: usize) -> Result<(), TabError> {
        if self.tabs.len() == 1 {
            return Err(TabError::LastTabCloseNoop);
        }
        if idx >= self.tabs.len() {
            return Err(TabError::IndexOutOfBounds);
        }

        self.tabs.remove(idx);

        // active_tab_idx 조정
        let new_len = self.tabs.len();
        if idx < self.active_tab_idx {
            // 닫힌 탭이 active 보다 앞에 있었으면 active 인덱스를 하나 감소
            self.active_tab_idx -= 1;
        } else if idx == self.active_tab_idx {
            // 닫힌 탭이 active 였으면 right neighbor (idx) 또는 마지막(idx-1)
            if self.active_tab_idx >= new_len {
                self.active_tab_idx = new_len - 1;
            }
            // idx < new_len 이면 그대로 유지 (right neighbor 가 승격)
        }
        // idx > active_tab_idx: active 보다 뒤를 닫음 → 인덱스 변화 없음

        Ok(())
    }

    /// 현재 active 탭의 GpuiNativeSplitter 를 반환한다 (lib.rs render 용).
    pub fn get_active_splitter(&self) -> &GpuiNativeSplitter<L> {
        &self.tabs[self.active_tab_idx].splitter
    }

    /// 현재 active 탭의 GpuiNativeSplitter 를 mutable 로 반환한다.
    pub fn get_active_splitter_mut(&mut self) -> &mut GpuiNativeSplitter<L> {
        &mut self.tabs[self.active_tab_idx].splitter
    }

    /// 현재 active 탭의 참조를 반환한다.
    pub fn active_tab(&self) -> &Tab<L> {
        &self.tabs[self.active_tab_idx]
    }

    /// 현재 active 탭의 mutable 참조를 반환한다.
    pub fn active_tab_mut(&mut self) -> &mut Tab<L> {
        &mut self.tabs[self.active_tab_idx]
    }

    // @MX:ANCHOR: [AUTO] tab-dispatch-api
    // @MX:REASON: [AUTO] FocusCommand MS-2 변형을 TabContainer 연산으로 매핑하는 단일 진입점.
    //   fan_in >= 2: T10 키 바인딩 핸들러 wire, T11 bench trigger.
    //   PrevTab/NextTab 은 saturating 처리 (첫/마지막 탭에서 no-op).
    /// MS-2 [`FocusCommand`] 를 TabContainer 상태 변경으로 디스패치한다.
    ///
    /// ## 매핑
    ///
    /// | 명령 | 동작 |
    /// |------|------|
    /// | `NewTab` | `new_tab(title, splitter)` — splitter_factory 호출 |
    /// | `CloseTab` | `close_tab(active_tab_idx)` |
    /// | `SwitchTabIdx(n)` | `switch_tab(n)` |
    /// | `SplitHorizontal` | `get_active_splitter_mut().split_horizontal(focused)` |
    /// | `SplitVertical` | `get_active_splitter_mut().split_vertical(focused)` |
    /// | `PrevTab` | `switch_tab(active_tab_idx - 1)` saturating (0 에서 no-op) |
    /// | `NextTab` | `switch_tab(active_tab_idx + 1)` saturating (마지막에서 no-op) |
    /// | MS-1 명령 (`Prev`/`Next`/`Click`) | `Ok(())` no-op (탭 레벨 불필요) |
    ///
    /// # Errors
    ///
    /// - [`TabError::IndexOutOfBounds`]: `SwitchTabIdx(n)` 에서 n >= tabs.len()
    /// - [`TabError::LastTabCloseNoop`]: 탭 1개 남았을 때 `CloseTab`
    pub fn dispatch_tab_command(
        &mut self,
        cmd: crate::panes::FocusCommand,
        new_tab_factory: Option<(&str, GpuiNativeSplitter<L>)>,
    ) -> Result<(), TabError> {
        use crate::panes::FocusCommand;
        match cmd {
            FocusCommand::NewTab => {
                if let Some((title, splitter)) = new_tab_factory {
                    self.new_tab(title.to_string(), splitter);
                }
                Ok(())
            }
            FocusCommand::CloseTab => self.close_tab(self.active_tab_idx),
            FocusCommand::SwitchTabIdx(n) => self.switch_tab(n),
            FocusCommand::SplitHorizontal => {
                let splitter = self.get_active_splitter_mut();
                if let Some(focused_id) = splitter.focused().cloned() {
                    splitter.split_horizontal(focused_id).map(|_| ())?;
                }
                Ok(())
            }
            FocusCommand::SplitVertical => {
                let splitter = self.get_active_splitter_mut();
                if let Some(focused_id) = splitter.focused().cloned() {
                    splitter.split_vertical(focused_id).map(|_| ())?;
                }
                Ok(())
            }
            FocusCommand::PrevTab => {
                // saturating: 첫 번째 탭에서는 no-op
                if self.active_tab_idx > 0 {
                    self.switch_tab(self.active_tab_idx - 1)?;
                }
                Ok(())
            }
            FocusCommand::NextTab => {
                // saturating: 마지막 탭에서는 no-op
                let next = self.active_tab_idx + 1;
                if next < self.tabs.len() {
                    self.switch_tab(next)?;
                }
                Ok(())
            }
            // MS-1 명령: 탭 레벨에서는 no-op (pane 레벨 FocusRouter 가 처리)
            FocusCommand::Prev | FocusCommand::Next | FocusCommand::Click(_) => Ok(()),
        }
    }
}

// ============================================================
// 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panes::{PaneId, PaneSplitter};

    // -------------------------------------------------------
    // 테스트 헬퍼
    // -------------------------------------------------------

    /// 테스트용 GpuiNativeSplitter<String> 을 생성한다.
    fn make_splitter(root_label: &str) -> GpuiNativeSplitter<String> {
        let root_id = PaneId::new_unique();
        GpuiNativeSplitter::new_with_factory(
            root_id,
            root_label.to_string(),
            Box::new(|id| format!("pane-{}", id.0)),
        )
    }

    /// 테스트용 Tab<String> 을 생성한다.
    fn make_tab(title: &str) -> Tab<String> {
        Tab {
            id: TabId::new_unique(),
            title: title.to_string(),
            splitter: make_splitter(title),
            last_focused_pane: None,
        }
    }

    /// 테스트용 TabContainer<String> (탭 1 개) 을 생성한다.
    fn make_container() -> TabContainer<String> {
        TabContainer::new(make_tab("tab-a"))
    }

    // -------------------------------------------------------
    // Test 1: new_tab_creates_leaf_one_pane_tree (AC-P-8)
    // -------------------------------------------------------

    /// TabContainer 초기 상태에 Tab 1 개 + splitter.leaf_count == 1 (AC-P-8).
    #[test]
    fn new_tab_creates_leaf_one_pane_tree() {
        let container = make_container();
        assert_eq!(container.tabs.len(), 1, "초기 탭 수 == 1");
        assert_eq!(container.active_tab_idx, 0, "active_tab_idx == 0");
        assert_eq!(
            container.get_active_splitter().tree().leaf_count(),
            1,
            "초기 splitter.leaf_count == 1"
        );
    }

    // -------------------------------------------------------
    // Test 2: new_tab_increments_active_idx (AC-P-8)
    // -------------------------------------------------------

    /// new_tab 호출 후 active_tab_idx 가 last 로 이동한다 (AC-P-8).
    #[test]
    fn new_tab_increments_active_idx() {
        let mut container = make_container();
        assert_eq!(container.active_tab_idx, 0);

        container.new_tab("tab-b".to_string(), make_splitter("b"));
        assert_eq!(container.tabs.len(), 2, "탭 수 == 2");
        assert_eq!(container.active_tab_idx, 1, "active_tab_idx == 1 (마지막)");

        container.new_tab("tab-c".to_string(), make_splitter("c"));
        assert_eq!(container.tabs.len(), 3, "탭 수 == 3");
        assert_eq!(container.active_tab_idx, 2, "active_tab_idx == 2 (마지막)");
    }

    // -------------------------------------------------------
    // Test 3: switch_tab_preserves_last_focused_pane (AC-P-11)
    // -------------------------------------------------------

    /// Tab A 에서 split 후 pane X focus → Tab B 전환 → Tab A 로 돌아옴 → focused == X (AC-P-11).
    #[test]
    fn switch_tab_preserves_last_focused_pane() {
        let mut container = make_container(); // tab idx 0 = tab-a

        // Tab A: split → pane X
        let root_id = container.tabs[0].splitter.focused().cloned().unwrap();
        let pane_x = container.tabs[0]
            .splitter
            .split_horizontal(root_id)
            .expect("split 성공");
        // pane_x 에 포커스
        container.tabs[0].splitter.focus_pane(pane_x.clone());
        assert_eq!(container.tabs[0].splitter.focused(), Some(&pane_x));

        // Tab B 추가 (idx 1)
        container.new_tab("tab-b".to_string(), make_splitter("b"));
        assert_eq!(container.active_tab_idx, 1);

        // Tab A 로 복귀 (idx 0)
        container.switch_tab(0).expect("switch_tab 성공");
        assert_eq!(container.active_tab_idx, 0);

        // AC-P-11: 복원된 focus 가 pane_x 이어야 한다.
        assert_eq!(
            container.tabs[0].splitter.focused(),
            Some(&pane_x),
            "switch_tab 복귀 후 focused == pane_x (AC-P-11)"
        );
    }

    // -------------------------------------------------------
    // Test 4: switch_tab_out_of_bounds_returns_error (AC-P-25 negative)
    // -------------------------------------------------------

    /// idx >= tabs.len() → Err(IndexOutOfBounds) (AC-P-25 negative).
    #[test]
    fn switch_tab_out_of_bounds_returns_error() {
        let container = make_container();
        let result = container
            .tabs
            .len()
            .checked_add(99)
            .map(|oor| {
                let mut c = TabContainer::new(make_tab("x"));
                c.switch_tab(oor)
            })
            .unwrap();
        assert_eq!(result, Err(TabError::IndexOutOfBounds));
    }

    // -------------------------------------------------------
    // Test 5: close_last_tab_is_noop (AC-P-10)
    // -------------------------------------------------------

    /// 탭 1 개만 있을 때 close → Err(LastTabCloseNoop) + tabs.len() 유지 (AC-P-10).
    #[test]
    fn close_last_tab_is_noop() {
        let mut container = make_container();
        assert_eq!(container.tabs.len(), 1);
        let result = container.close_tab(0);
        assert_eq!(result, Err(TabError::LastTabCloseNoop));
        assert_eq!(container.tabs.len(), 1, "tabs.len() 유지");
    }

    // -------------------------------------------------------
    // Test 6: close_middle_tab_promotes_right_neighbor (AC-P-10)
    // -------------------------------------------------------

    /// 3 tabs, 중간 탭 close → tabs.len() == 2 + active_tab_idx 유지 (AC-P-10).
    #[test]
    fn close_middle_tab_promotes_right_neighbor() {
        let mut container = make_container(); // idx 0
        container.new_tab("tab-b".to_string(), make_splitter("b")); // idx 1
        container.new_tab("tab-c".to_string(), make_splitter("c")); // idx 2

        // active 를 idx 0 으로 돌려놓기
        container.switch_tab(0).expect("switch_tab 0");
        assert_eq!(container.active_tab_idx, 0);

        // 중간 탭 (idx 1) close
        container.close_tab(1).expect("close_tab 1 성공");
        assert_eq!(container.tabs.len(), 2, "tabs.len() == 2");
        // active_tab_idx 는 0 유지 (닫힌 탭이 active 보다 뒤)
        assert_eq!(container.active_tab_idx, 0, "active_tab_idx 유지 == 0");
    }

    // -------------------------------------------------------
    // Test 7: close_active_tab_shifts_active_to_right (AC-P-10)
    // -------------------------------------------------------

    /// 3 tabs, active=1, close active → tabs.len() == 2 + active_tab_idx = 1 (right neighbor 승격).
    #[test]
    fn close_active_tab_shifts_active_to_right() {
        let mut container = make_container(); // idx 0
        container.new_tab("tab-b".to_string(), make_splitter("b")); // idx 1
        container.new_tab("tab-c".to_string(), make_splitter("c")); // idx 2

        container.switch_tab(1).expect("switch_tab 1");
        assert_eq!(container.active_tab_idx, 1);

        container.close_tab(1).expect("close_tab 1 (active) 성공");
        assert_eq!(container.tabs.len(), 2, "tabs.len() == 2");
        // active 였던 idx 1 을 닫았으므로 right neighbor (현재 idx 1) 가 active
        assert_eq!(
            container.active_tab_idx, 1,
            "active_tab_idx == 1 (right 승격)"
        );
    }

    // -------------------------------------------------------
    // Test 8: close_last_active_tab_shifts_to_left (AC-P-10)
    // -------------------------------------------------------

    /// 3 tabs, active=2 (last), close active → tabs.len() == 2 + active_tab_idx = 1 (left).
    #[test]
    fn close_last_active_tab_shifts_to_left() {
        let mut container = make_container(); // idx 0
        container.new_tab("tab-b".to_string(), make_splitter("b")); // idx 1
        container.new_tab("tab-c".to_string(), make_splitter("c")); // idx 2

        container.switch_tab(2).expect("switch_tab 2");
        assert_eq!(container.active_tab_idx, 2);

        container
            .close_tab(2)
            .expect("close_tab 2 (last active) 성공");
        assert_eq!(container.tabs.len(), 2, "tabs.len() == 2");
        assert_eq!(
            container.active_tab_idx, 1,
            "active_tab_idx == 1 (left 승격)"
        );
    }

    // -------------------------------------------------------
    // Test 9: tab_id_is_unique_across_creation (Spike 3 패턴 준수)
    // -------------------------------------------------------

    /// new_tab 여러 번 호출 시 TabId 중복 없음 (Spike 3 패턴 준수).
    #[test]
    fn tab_id_is_unique_across_creation() {
        let mut container = make_container();
        for i in 0..9 {
            container.new_tab(format!("tab-{i}"), make_splitter(&format!("s{i}")));
            // 잠깐 대기 없이 연속 생성 — nanos 기반이므로 중복 없어야 함
        }
        let ids: Vec<&TabId> = container.tabs.iter().map(|t| &t.id).collect();
        let mut unique_ids: Vec<&TabId> = ids.clone();
        unique_ids.dedup();
        // 중복이 없으면 dedup 전후 길이 동일
        // 단, nanos 가 동일할 수 있으므로 HashSet 으로 검증
        let set: std::collections::HashSet<&str> =
            container.tabs.iter().map(|t| t.id.0.as_str()).collect();
        assert_eq!(set.len(), container.tabs.len(), "모든 TabId 가 고유해야 함");
    }

    // -------------------------------------------------------
    // Test 10: active_tab_returns_reference_to_current_tab
    // -------------------------------------------------------

    /// active_tab() 반환값이 current active 와 일치한다.
    #[test]
    fn active_tab_returns_reference_to_current_tab() {
        let mut container = make_container();
        container.new_tab("tab-b".to_string(), make_splitter("b"));

        container.switch_tab(0).expect("switch 0");
        let active_id_0 = container.active_tab().id.clone();
        assert_eq!(active_id_0, container.tabs[0].id);

        container.switch_tab(1).expect("switch 1");
        let active_id_1 = container.active_tab().id.clone();
        assert_eq!(active_id_1, container.tabs[1].id);
    }

    // -------------------------------------------------------
    // Test 11: tab_index_monotonic_on_create_sequence (AC-P-25)
    // -------------------------------------------------------

    /// new_tab × 5 후 active_tab_idx == 4 (AC-P-25 monotonic).
    #[test]
    fn tab_index_monotonic_on_create_sequence() {
        let mut container = make_container(); // idx 0
        for i in 1..=4 {
            container.new_tab(format!("tab-{i}"), make_splitter(&format!("s{i}")));
            assert_eq!(
                container.active_tab_idx, i,
                "new_tab #{i} 후 active_tab_idx == {i}"
            );
        }
        assert_eq!(container.active_tab_idx, 4, "최종 active_tab_idx == 4");
    }

    // -------------------------------------------------------
    // Test T9-1: dispatch_new_tab_command_creates_tab (MS-2 dispatch)
    // -------------------------------------------------------

    /// dispatch_tab_command(NewTab, factory) → tabs.len() 증가 + active_tab_idx 이동.
    #[test]
    fn dispatch_new_tab_command_creates_tab() {
        use crate::panes::FocusCommand;
        let mut container = make_container();
        assert_eq!(container.tabs.len(), 1);

        let new_splitter = make_splitter("new-dispatch");
        container
            .dispatch_tab_command(FocusCommand::NewTab, Some(("dispatch-tab", new_splitter)))
            .expect("NewTab dispatch 성공");

        assert_eq!(
            container.tabs.len(),
            2,
            "dispatch NewTab 후 tabs.len() == 2"
        );
        assert_eq!(
            container.active_tab_idx, 1,
            "active_tab_idx == 1 (새 탭 활성)"
        );
    }

    // -------------------------------------------------------
    // Test T9-2: dispatch_split_horizontal_command_updates_active_pane_tree (MS-2 dispatch)
    // -------------------------------------------------------

    /// dispatch_tab_command(SplitHorizontal, None) → active splitter leaf_count 증가.
    #[test]
    fn dispatch_split_horizontal_command_updates_active_pane_tree() {
        use crate::panes::FocusCommand;
        let mut container = make_container();
        assert_eq!(container.get_active_splitter().tree().leaf_count(), 1);

        container
            .dispatch_tab_command(FocusCommand::SplitHorizontal, None)
            .expect("SplitHorizontal dispatch 성공");

        assert_eq!(
            container.get_active_splitter().tree().leaf_count(),
            2,
            "SplitHorizontal 후 leaf_count == 2"
        );
    }

    // -------------------------------------------------------
    // Test T9-3: dispatch_prev_next_tab_saturating_at_boundary (MS-2 dispatch)
    // -------------------------------------------------------

    /// PrevTab 은 첫 번째 탭에서 no-op, NextTab 은 마지막 탭에서 no-op (saturating).
    #[test]
    fn dispatch_prev_next_tab_saturating_at_boundary() {
        use crate::panes::FocusCommand;
        let mut container = make_container(); // idx 0

        // 첫 번째 탭에서 PrevTab → no-op
        container
            .dispatch_tab_command(FocusCommand::PrevTab, None)
            .expect("PrevTab at first tab Ok");
        assert_eq!(
            container.active_tab_idx, 0,
            "PrevTab saturating: 여전히 idx 0"
        );

        // 탭 추가
        container.new_tab("tab-b".to_string(), make_splitter("b"));
        assert_eq!(container.active_tab_idx, 1);

        // 마지막 탭에서 NextTab → no-op
        container
            .dispatch_tab_command(FocusCommand::NextTab, None)
            .expect("NextTab at last tab Ok");
        assert_eq!(
            container.active_tab_idx, 1,
            "NextTab saturating: 여전히 idx 1"
        );

        // PrevTab → idx 0 으로 이동 (정상 동작 확인)
        container
            .dispatch_tab_command(FocusCommand::PrevTab, None)
            .expect("PrevTab 정상 이동");
        assert_eq!(container.active_tab_idx, 0, "PrevTab 이동 후 idx 0");

        // NextTab → idx 1 (정상 동작 확인)
        container
            .dispatch_tab_command(FocusCommand::NextTab, None)
            .expect("NextTab 정상 이동");
        assert_eq!(container.active_tab_idx, 1, "NextTab 이동 후 idx 1");
    }

    // -------------------------------------------------------
    // Test 12: get_active_splitter_mut_allows_split_on_current_tab
    // -------------------------------------------------------

    /// splitter 참조로 split_horizontal 호출 → 해당 탭만 leaf_count 증가 (cross-tab isolation).
    #[test]
    fn get_active_splitter_mut_allows_split_on_current_tab() {
        let mut container = make_container(); // idx 0: 1 leaf
        container.new_tab("tab-b".to_string(), make_splitter("b")); // idx 1: 1 leaf

        // Tab A (idx 0) 에서 split × 3
        container.switch_tab(0).expect("switch 0");
        let root_id_a = container.tabs[0].splitter.focused().cloned().unwrap();
        let _ = container
            .get_active_splitter_mut()
            .split_horizontal(root_id_a.clone())
            .unwrap();
        let _ = container
            .get_active_splitter_mut()
            .split_horizontal(root_id_a.clone())
            .unwrap();
        let _ = container
            .get_active_splitter_mut()
            .split_horizontal(root_id_a)
            .unwrap();
        let tab_a_leaves = container.tabs[0].splitter.tree().leaf_count();

        // Tab B (idx 1) leaf_count 는 1 유지 (cross-tab isolation)
        assert_eq!(
            container.tabs[1].splitter.tree().leaf_count(),
            1,
            "Tab B leaf_count == 1 (불변)"
        );
        // Tab A leaf_count > 1
        assert!(tab_a_leaves > 1, "Tab A leaf_count > 1 (split 반영)");
    }
}
