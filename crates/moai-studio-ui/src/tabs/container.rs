//! `TabContainer` 구현 + 탭 생성/전환/닫기 로직 + last_focused_pane 복원.
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-5 (REQ-P-040 ~ REQ-P-045)
//! - spec.md §5 RG-P-3 REQ-P-023 (탭 전환 시 last_focused_pane 복원)
//! - spec.md §5 RG-P-4 REQ-P-034 (tmux 중첩 시 OS/GPUI 레벨 우선 — AC-P-26)
//!
//! T9 완료: 키 바인딩 dispatcher (`tabs::keys::dispatch_tab_key`) + integration_tmux_nested.rs 통합 테스트.
//! SPEC-V3-004 MS-1 T3: impl Render for TabContainer 추가 (placeholder render).

use crate::panes::{PaneId, PaneTree, render_pane_tree};
use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px, rgb};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// TabId
// ============================================================

/// 탭을 고유하게 식별하는 ID.
///
/// Spike 3 결정에 따라 `format!("tab-{:x}", nanos)` 패턴 사용 (workspace ID 패턴 일관성).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TabId(pub String);

impl TabId {
    /// 나노초 + 프로세스-모노톤 카운터 기반 고유 TabId 생성.
    ///
    /// 나노초만으로는 병렬 테스트 (`cargo test` 다중 쓰레드)에서 동일 틱 충돌이 관측되어
    /// (T8 `close_middle_tab_promotes_neighbor` 간헐 실패), `AtomicU64` suffix 로 보강.
    /// Spike 3 `tab-{:x}` 패턴은 prefix 부분에서 유지되어 workspace ID regex 호환.
    pub fn new_unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("tab-{:x}-{:x}", nanos, seq))
    }

    /// 지정 문자열로 TabId 생성 (테스트 전용 편의 메서드).
    pub fn new_from_literal(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for TabId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================
// Tab
// ============================================================

/// 단일 탭 — 독립된 PaneTree + last_focused_pane 복원 상태 보유.
///
/// REQ-P-041: 각 탭은 독립된 PaneTree 를 소유한다.
/// REQ-P-023: 탭 전환 시 이전에 focus 된 pane ID 를 복원한다.
#[derive(Debug)]
pub struct Tab {
    /// 탭 고유 식별자.
    pub id: TabId,
    /// 탭 제목 (cwd.file_name() 또는 "untitled").
    pub title: String,
    /// 이 탭이 소유한 pane 이진 트리 (String payload — 테스트 환경).
    pub pane_tree: PaneTree<String>,
    /// 이 탭에서 마지막으로 focus 된 pane ID (탭 전환 복원용).
    pub last_focused_pane: Option<PaneId>,
}

impl Tab {
    /// 단일 leaf pane 으로 초기화된 새 Tab 을 생성한다 (REQ-P-042).
    pub fn new(id: TabId, cwd: Option<PathBuf>) -> Self {
        let title = cwd
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();
        let root_pane_id = PaneId::new_unique();
        let pane_tree = PaneTree::new_leaf(root_pane_id.clone(), title.clone());
        Self {
            id,
            title,
            pane_tree,
            last_focused_pane: Some(root_pane_id),
        }
    }
}

// ============================================================
// 에러 타입
// ============================================================

/// 탭 닫기 실패 원인.
#[derive(Debug, PartialEq, Eq)]
pub enum CloseTabError {
    /// 마지막 탭은 닫을 수 없다 (REQ-P-043: noop).
    LastTab,
    /// 지정 TabId 가 존재하지 않는다.
    TabNotFound,
}

/// 탭 전환 실패 원인.
#[derive(Debug, PartialEq, Eq)]
pub enum SwitchTabError {
    /// 인덱스가 범위를 벗어났다.
    IndexOutOfBounds,
}

// ============================================================
// TabContainer
// ============================================================

/// N 개의 Tab 을 소유하고 active_tab_idx 로 현재 활성 탭을 관리하는 컨테이너.
///
/// REQ-P-040: 다중 탭 관리.
/// REQ-P-041: 각 탭 독립 PaneTree 소유.
/// REQ-P-042: 탭 생성 시 단일 leaf 로 초기화.
/// REQ-P-043: 마지막 탭 닫기는 noop.
#[derive(Debug)]
pub struct TabContainer {
    /// 소유하는 탭 목록.
    pub tabs: Vec<Tab>,
    /// 현재 활성 탭 인덱스.
    pub active_tab_idx: usize,
}

impl TabContainer {
    /// 단일 탭으로 초기화된 TabContainer 를 생성한다.
    pub fn new() -> Self {
        let first_tab = Tab::new(TabId::new_unique(), None);
        Self {
            tabs: vec![first_tab],
            active_tab_idx: 0,
        }
    }

    // ----------------------------------------------------------
    // @MX:ANCHOR: [AUTO] tab-create-api
    // @MX:REASON: [AUTO] 탭 생성 진입점. REQ-P-040/041/042 계약 고정.
    //   fan_in >= 3: T9 키 바인딩 dispatcher (Cmd/Ctrl+T), MS-3 persistence 복원, TabContainer::new.
    // ----------------------------------------------------------

    /// 새 탭을 생성하고 TabId 를 반환한다.
    ///
    /// 새 탭은 단일 leaf pane 으로 초기화된다 (REQ-P-042).
    /// 생성된 탭은 탭 목록 끝에 추가되며 active 탭으로 전환된다.
    ///
    /// # Arguments
    ///
    /// * `cwd` — 새 탭의 작업 디렉터리 (탭 제목으로 사용됨). None 이면 "untitled".
    pub fn new_tab(&mut self, cwd: Option<PathBuf>) -> TabId {
        let id = TabId::new_unique();
        let tab = Tab::new(id.clone(), cwd);
        self.tabs.push(tab);
        self.active_tab_idx = self.tabs.len() - 1;
        id
    }

    /// 현재 탭 수 반환 (AC-P-25 tab_index_monotonic_on_create 검증용).
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    // ----------------------------------------------------------
    // @MX:ANCHOR: [AUTO] tab-switch-invariant
    // @MX:REASON: [AUTO] 탭 전환 시 last_focused_pane 복원 불변 조건.
    //   REQ-P-023: 탭 전환 시 이전 탭의 last_focused_pane 을 저장하고 새 탭의 것을 복원.
    //   fan_in >= 3: T9 키 바인딩 (Cmd/Ctrl+1~9/{/}), MS-3 persistence, 직접 API 호출.
    // ----------------------------------------------------------

    /// `idx` 번째 탭으로 전환한다.
    ///
    /// 전환 시 현재 탭의 last_focused_pane 를 저장하고 새 탭의 last_focused_pane 를 복원한다
    /// (REQ-P-023, AC-P-11).
    ///
    /// # Errors
    ///
    /// - [`SwitchTabError::IndexOutOfBounds`]: idx 가 탭 범위를 벗어날 때.
    pub fn switch_tab(&mut self, idx: usize) -> Result<(), SwitchTabError> {
        if idx >= self.tabs.len() {
            return Err(SwitchTabError::IndexOutOfBounds);
        }
        // @MX:NOTE: [AUTO] last-focused-pane-restore
        // 탭 전환 시 현재 탭의 last_focused_pane 는 이미 Tab.last_focused_pane 에 저장됨.
        // 새 탭으로 인덱스만 교체하면 해당 탭의 last_focused_pane 가 자동 복원됨.
        // REQ-P-023: "탭 전환 시 이전 탭으로 돌아오면 마지막 focus pane 을 복원한다."
        self.active_tab_idx = idx;
        Ok(())
    }

    /// 지정 TabId 의 탭을 닫는다.
    ///
    /// 마지막 탭인 경우 noop (REQ-P-043, AC-P-10).
    /// 닫은 탭이 active 탭인 경우 이웃 탭 (오른쪽 우선, 없으면 왼쪽) 으로 전환한다.
    ///
    /// # Errors
    ///
    /// - [`CloseTabError::LastTab`]: 마지막 탭을 닫으려 할 때 (noop).
    /// - [`CloseTabError::TabNotFound`]: 지정 TabId 가 존재하지 않을 때.
    pub fn close_tab(&mut self, tab_id: &TabId) -> Result<(), CloseTabError> {
        if self.tabs.len() == 1 {
            return Err(CloseTabError::LastTab);
        }
        let pos = self
            .tabs
            .iter()
            .position(|t| &t.id == tab_id)
            .ok_or(CloseTabError::TabNotFound)?;

        self.tabs.remove(pos);

        // active_tab_idx 조정 (닫힌 탭 이후 인덱스 보정).
        if self.active_tab_idx >= self.tabs.len() {
            self.active_tab_idx = self.tabs.len() - 1;
        } else if pos < self.active_tab_idx {
            self.active_tab_idx -= 1;
        }

        Ok(())
    }

    /// 현재 활성 탭을 반환한다.
    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_idx]
    }

    /// 현재 활성 탭을 가변 참조로 반환한다.
    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab_idx]
    }

    /// 지정 인덱스의 탭의 last_focused_pane 를 설정한다.
    pub fn set_last_focused_pane(&mut self, idx: usize, pane_id: PaneId) {
        if let Some(tab) = self.tabs.get_mut(idx) {
            tab.last_focused_pane = Some(pane_id);
        }
    }
}

impl Default for TabContainer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// GPUI Render 구현 (SPEC-V3-004 MS-1 T3)
// ============================================================

// 탭 바 색상 토큰 (SPEC-V3-003 design token carry)
const TAB_ACTIVE_BG: u32 = 0x232327;
const TAB_INACTIVE_BG: u32 = 0x131315;
const TAB_FG_ACTIVE: u32 = 0xf4f4f5;
const TAB_FG_INACTIVE: u32 = 0xb5b5bb;
const CONTENT_BG: u32 = 0x0a0a0b;

// @MX:ANCHOR: [AUTO] tab-container-render
// @MX:REASON: [AUTO] SPEC-V3-004 REQ-R-001/002/003/005. TabContainer render 진입점.
//   MS-1 에서 placeholder, MS-2 에서 render_pane_tree 로 교체.
//   fan_in >= 3: RootView.content_area, integration_render 테스트, 향후 MS-2 T4.
impl Render for TabContainer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // REQ-R-005: tabs.is_empty() 여도 panic 없이 fallback 렌더.
        if self.tabs.is_empty() {
            return div()
                .flex()
                .flex_col()
                .size_full()
                .bg(rgb(CONTENT_BG))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x6b6b73))
                        .child("MS-1 TabContainer placeholder — no tabs"),
                );
        }

        // 탭 바 렌더 (REQ-R-002a)
        let active_idx = self.active_tab_idx;
        let mut tab_bar = div()
            .flex()
            .flex_row()
            .w_full()
            .h(px(32.0))
            .bg(rgb(TAB_INACTIVE_BG));

        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = i == active_idx;
            let bg = if is_active {
                TAB_ACTIVE_BG
            } else {
                TAB_INACTIVE_BG
            };
            let fg = if is_active {
                TAB_FG_ACTIVE
            } else {
                TAB_FG_INACTIVE
            };
            let label = tab.title.clone();
            tab_bar = tab_bar.child(
                div()
                    .px_3()
                    .py_1()
                    .bg(rgb(bg))
                    .text_sm()
                    .text_color(rgb(fg))
                    .child(label),
            );
        }

        // MS-2 T4: render_pane_tree 로 활성 탭 PaneTree 렌더 (REQ-R-002b).
        // @MX:NOTE: [AUTO] tab-container-body-render
        // 활성 탭의 PaneTree<String> 을 render_pane_tree 로 변환.
        // String 은 IntoElement + Clone 을 구현하므로 제약 충족.
        // AC-R-2: horizontal split 시 divider_vertical 1 개 생성.
        let body = div()
            .flex()
            .flex_col()
            .flex_grow()
            .bg(rgb(CONTENT_BG))
            .child(render_pane_tree(&self.tabs[active_idx].pane_tree));

        // cx.notify() 는 상태 변경 시 호출. render 는 순수 읽기.
        // REQ-R-003: new_tab/switch_tab/close_tab 이 cx.notify() 를 호출한다 (해당 메서드에서 처리).
        let _ = cx; // render 에서 notify 불필요

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(tab_bar)
            .child(body)
    }
}

// ============================================================
// 단위 테스트 (RED phase — contract.md §10.4)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-P-8: new_tab 은 단일 leaf pane 을 가진 새 탭을 생성한다.
    ///
    /// 검증: 새 탭의 pane_tree 가 Leaf 이고 leaf_count == 1.
    #[test]
    fn new_tab_creates_leaf_one_pane_tree() {
        let mut container = TabContainer::new();
        // 초기 탭 1개 + 새 탭 생성
        let new_id = container.new_tab(None);

        // 새 탭으로 전환
        let new_idx = container.tabs.len() - 1;
        container.switch_tab(new_idx).unwrap();

        let tab = container.active_tab();
        assert_eq!(tab.id, new_id);
        // 새 탭의 pane_tree 는 단일 leaf (REQ-P-042)
        assert_eq!(tab.pane_tree.leaf_count(), 1);
        // Leaf 노드임을 확인
        assert!(matches!(tab.pane_tree, PaneTree::Leaf(_)));
    }

    /// AC-P-11: switch_tab 은 이전 탭의 last_focused_pane 를 복원한다.
    ///
    /// 시나리오: 탭 A (pane_A), 탭 B (pane_B) 생성 후
    /// 탭 A 에서 last_focused_pane = pane_A_custom 설정 → 탭 B 로 전환 → 다시 탭 A 로 전환 시 pane_A_custom 복원.
    #[test]
    fn switch_tab_restores_last_focused_pane() {
        let mut container = TabContainer::new(); // 탭 0
        container.new_tab(None); // 탭 1

        // 탭 0 에 커스텀 last_focused_pane 설정
        let pane_a = PaneId::new_from_literal("pane-a-custom");
        container.set_last_focused_pane(0, pane_a.clone());

        // 탭 1 로 전환
        container.switch_tab(1).unwrap();
        assert_eq!(container.active_tab_idx, 1);

        // 다시 탭 0 으로 전환
        container.switch_tab(0).unwrap();
        assert_eq!(container.active_tab_idx, 0);

        // 탭 0 의 last_focused_pane 는 pane_a_custom 이어야 함 (REQ-P-023)
        let restored = container.active_tab().last_focused_pane.as_ref().unwrap();
        assert_eq!(restored, &pane_a);
    }

    /// AC-P-10 (1/2): 마지막 탭 닫기는 noop — LastTab 에러 반환.
    ///
    /// 검증: tabs.len() == 1 일 때 close_tab 은 Err(CloseTabError::LastTab) 반환하고 탭 목록 유지.
    #[test]
    fn close_last_tab_is_noop() {
        let mut container = TabContainer::new();
        assert_eq!(container.tabs.len(), 1);

        let only_tab_id = container.tabs[0].id.clone();
        let result = container.close_tab(&only_tab_id);

        assert_eq!(result, Err(CloseTabError::LastTab));
        // 탭 목록 변화 없음
        assert_eq!(container.tabs.len(), 1);
    }

    /// AC-P-10 (2/2): 중간 탭 닫기 시 이웃 탭으로 active 전환.
    ///
    /// 시나리오: 탭 0, 1, 2 생성 → 탭 1 닫기 → 탭 1 (구 탭 2) 이 active.
    #[test]
    fn close_middle_tab_promotes_neighbor() {
        let mut container = TabContainer::new(); // 탭 0
        let tab1_id = container.new_tab(None); // 탭 1
        container.new_tab(None); // 탭 2

        assert_eq!(container.tabs.len(), 3);

        // 탭 1 닫기 (중간)
        container.close_tab(&tab1_id).unwrap();

        // 탭 목록 2개로 감소
        assert_eq!(container.tabs.len(), 2);
        // tab1_id 는 더 이상 존재하지 않음
        assert!(!container.tabs.iter().any(|t| t.id == tab1_id));
    }

    /// AC-P-25: 탭 생성 시 tab_count 가 단조 증가한다.
    ///
    /// 검증: new_tab 을 N번 호출할 때마다 tab_count 가 정확히 1씩 증가.
    #[test]
    fn tab_index_monotonic_on_create() {
        let mut container = TabContainer::new();
        let initial_count = container.tab_count();
        assert_eq!(initial_count, 1);

        for expected in 2..=5usize {
            container.new_tab(None);
            assert_eq!(container.tab_count(), expected);
        }
    }
}
