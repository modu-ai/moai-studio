//! `PaneTree` 이진 트리 자료구조 + in-order iterator + split/close 알고리즘.
//!
//! 스펙 참조:
//! - spec.md §5 RG-P-1 (REQ-P-001 ~ REQ-P-005)
//! - spec.md §7.1 용어 정의:
//!   `Horizontal` = 좌/우 배치 (수직 divider, first=left / second=right)
//!   `Vertical`   = 상/하 배치 (수평 divider, first=top  / second=bottom)
//!
//! ## 제네릭 설계 rationale
//!
//! `PaneTree<L>` 은 leaf 타입에 대해 제네릭이다.
//! - prod:  `PaneTree<Entity<TerminalSurface>>` — T4 에서 GPUI Entity 통합 (AC-P-1 통합)
//! - test:  `PaneTree<String>`                  — GPUI context 없이 단위 테스트 가능
//!
//! GPUI `Entity<TerminalSurface>` 를 직접 new 하려면 `gpui::TestAppContext` 가 필요하며,
//! T4 에서 PtyWorker spawn 과 함께 통합한다. T1 에서는 String payload 로 동일한 알고리즘을
//! 완전히 검증한다.
//!
//! @MX:TODO: [AUTO] T4 에서 PaneLeafHandle 이 gpui::Entity<TerminalSurface> 를 owning
//!   하도록 통합. 현재 prod 타입 파라미터는 플레이스홀더.

use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// PaneId / SplitNodeId
// ============================================================

/// pane 을 고유하게 식별하는 ID.
///
/// Spike 3 결정: `format!("pane-{:x}", nanos)` — workspace generate_id 패턴 차용.
/// uuid crate 추가는 YAGNI (workspace/src/lib.rs:60-67 일관성 유지).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaneId(pub String);

impl PaneId {
    /// 나노초 + 프로세스-모노톤 카운터 기반 고유 PaneId 생성.
    ///
    /// 병렬 테스트에서 동일 틱 충돌 방지를 위해 `AtomicU64` suffix 로 보강.
    /// Spike 3 `pane-{:x}` 패턴은 prefix 부분에서 유지.
    pub fn new_unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("pane-{:x}-{:x}", nanos, seq))
    }

    /// 지정 문자열로 PaneId 생성 (테스트 전용 편의 메서드).
    pub fn new_from_literal(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for PaneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Split 노드를 고유하게 식별하는 ID.
///
/// `format!("split-{:x}", nanos)` 패턴.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SplitNodeId(pub String);

impl SplitNodeId {
    /// 나노초 + 프로세스-모노톤 카운터 기반 고유 SplitNodeId 생성.
    ///
    /// 병렬 테스트 충돌 방지, Spike 3 `split-{:x}` prefix 유지.
    pub fn new_unique() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("split-{:x}-{:x}", nanos, seq))
    }
}

// ============================================================
// SplitDirection
// ============================================================

/// @MX:NOTE: [AUTO] horizontal-is-left-right-not-top-bottom
///   spec.md §7.1 의 C-3 해소 용어 계약.
///   Horizontal = 좌/우 배치 (수직 divider), Vertical = 상/하 배치 (수평 divider).
///   first/second 는 방향 중립 명명: Horizontal → first=left / second=right,
///   Vertical → first=top / second=bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    /// 좌/우 배치 (수직 divider). first = left, second = right.
    Horizontal,
    /// 상/하 배치 (수평 divider). first = top,  second = bottom.
    Vertical,
}

// ============================================================
// Leaf 래퍼 — PaneId + payload
// ============================================================

/// Leaf 노드: PaneId + 실제 터미널 payload (제네릭).
///
/// prod: `L = Entity<TerminalSurface>`  (T4 통합 시 교체)
/// test: `L = String`                   (단위 테스트용 stub)
#[derive(Debug, Clone)]
pub struct Leaf<L> {
    /// 이 leaf 의 고유 식별자.
    pub id: PaneId,
    /// 실제 터미널 또는 테스트 stub 페이로드.
    pub payload: L,
}

// ============================================================
// PaneTree
// ============================================================

/// @MX:ANCHOR: [AUTO] pane-tree-invariant
/// @MX:REASON: [AUTO] PaneTree 자료구조는 split/close/render/persistence 에서 fan_in >= 4.
///   enum 불변 조건 (Split 의 ratio 경계 제외, leaf/split 교차 불변) 을 이 정의가 계약.
///   Leaf 는 항상 terminal-or-stub 이며 Split 은 항상 2 자녀를 가진다.
#[derive(Debug)]
pub enum PaneTree<L> {
    /// 단일 터미널 pane.
    Leaf(Leaf<L>),
    /// 두 서브트리를 direction 방향으로 나눈 내부 노드.
    Split {
        /// 분할 방향 (Horizontal = 좌우, Vertical = 상하).
        direction: SplitDirection,
        /// first 의 비율 (0.0 < ratio < 1.0). second 비율 = 1.0 - ratio.
        ratio: f32,
        /// 첫 번째 자녀 (Horizontal: left, Vertical: top).
        first: Box<PaneTree<L>>,
        /// 두 번째 자녀 (Horizontal: right, Vertical: bottom).
        second: Box<PaneTree<L>>,
        /// 이 Split 노드의 고유 식별자.
        id: SplitNodeId,
    },
}

// ============================================================
// 오류 타입
// ============================================================

/// split 연산 실패 원인.
#[derive(Debug, PartialEq, Eq)]
pub enum SplitError {
    /// target PaneId 가 트리에 존재하지 않는다 (Leaf 가 아닌 Split 노드 ID 는 해당 없음).
    TargetNotFound,
    /// 최소 크기 위반으로 분할 불가 (T5 이후 사용, T1 에서는 미검사).
    MinSizeViolated,
}

/// ratio 설정 실패 원인.
#[derive(Debug, PartialEq, Eq)]
pub enum RatioError {
    /// ratio ≤ 0.0 또는 ratio ≥ 1.0 또는 NaN/Inf (AC-P-20).
    OutOfBounds,
    /// target Split 노드 ID 가 트리에 존재하지 않는다.
    SplitNodeNotFound,
}

// ============================================================
// PaneTree impl
// ============================================================

impl<L: Clone> PaneTree<L> {
    // ----------------------------------------------------------
    // 생성자
    // ----------------------------------------------------------

    /// 단일 leaf 로 구성된 PaneTree 를 생성한다.
    pub fn new_leaf(id: PaneId, payload: L) -> Self {
        Self::Leaf(Leaf { id, payload })
    }

    // ----------------------------------------------------------
    // @MX:ANCHOR: [AUTO] pane-split-api
    // @MX:REASON: [AUTO] 외부 UI 레이어의 split 진입점. REQ-P-002/003 의 semantics 고정.
    //   fan_in >= 3: T4 구체 구현체 (GpuiNativeSplitter), T7 RootView, T9 키 바인딩 dispatcher.
    // ----------------------------------------------------------

    /// target_id leaf 를 Horizontal (좌/우) 로 분할한다.
    ///
    /// 분할 후 기존 leaf 는 first (left) 위치, 새 leaf 는 second (right) 위치.
    /// ratio 기본값 = 0.5 (균등 분할).
    ///
    /// # Errors
    ///
    /// - [`SplitError::TargetNotFound`]: target_id 가 Leaf 노드로 존재하지 않을 때.
    pub fn split_horizontal(
        &mut self,
        target_id: &PaneId,
        new_id: PaneId,
        new_payload: L,
    ) -> Result<(), SplitError> {
        self.split_inner(target_id, new_id, new_payload, SplitDirection::Horizontal)
    }

    /// target_id leaf 를 Vertical (상/하) 로 분할한다.
    ///
    /// 분할 후 기존 leaf 는 first (top) 위치, 새 leaf 는 second (bottom) 위치.
    /// ratio 기본값 = 0.5 (균등 분할).
    ///
    /// # Errors
    ///
    /// - [`SplitError::TargetNotFound`]: target_id 가 Leaf 노드로 존재하지 않을 때.
    pub fn split_vertical(
        &mut self,
        target_id: &PaneId,
        new_id: PaneId,
        new_payload: L,
    ) -> Result<(), SplitError> {
        self.split_inner(target_id, new_id, new_payload, SplitDirection::Vertical)
    }

    fn split_inner(
        &mut self,
        target_id: &PaneId,
        new_id: PaneId,
        new_payload: L,
        direction: SplitDirection,
    ) -> Result<(), SplitError> {
        match self {
            PaneTree::Leaf(leaf) if &leaf.id == target_id => {
                // 현재 leaf 를 Split 노드로 교체 (in-place mutation).
                let old_leaf = Leaf {
                    id: leaf.id.clone(),
                    payload: leaf.payload.clone(),
                };
                *self = PaneTree::Split {
                    direction,
                    ratio: 0.5,
                    first: Box::new(PaneTree::Leaf(old_leaf)),
                    second: Box::new(PaneTree::Leaf(Leaf {
                        id: new_id,
                        payload: new_payload,
                    })),
                    id: SplitNodeId::new_unique(),
                };
                Ok(())
            }
            PaneTree::Leaf(_) => Err(SplitError::TargetNotFound),
            PaneTree::Split { first, second, .. } => {
                match first.split_inner(target_id, new_id.clone(), new_payload.clone(), direction) {
                    Ok(()) => Ok(()),
                    Err(SplitError::TargetNotFound) => {
                        second.split_inner(target_id, new_id, new_payload, direction)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    /// target_id leaf 를 트리에서 제거한다.
    ///
    /// 제거 후 형제 서브트리가 부모 위치를 승계한다 (AC-P-2).
    /// tree 가 단일 leaf 인 경우 no-op (AC-P-3).
    ///
    /// # Errors
    ///
    /// - [`SplitError::TargetNotFound`]: target_id 가 트리에 없을 때.
    pub fn close_pane(&mut self, target_id: &PaneId) -> Result<(), SplitError> {
        // 단일 leaf — no-op (AC-P-3).
        if let PaneTree::Leaf(leaf) = self {
            if &leaf.id == target_id {
                return Ok(()); // no-op: 마지막 leaf 유지
            } else {
                return Err(SplitError::TargetNotFound);
            }
        }

        // owned 재귀: tree 를 소비하고 새 tree 를 돌려받는다.
        // 임시 placeholder 로 self 를 꺼내 close_owned 에 넘긴다.
        // placeholder 는 close_owned 결과로 즉시 교체된다.
        //
        // 주의: placeholder 의 payload 는 즉시 버려지므로 의미 없음.
        // Clone 이 필요하므로 트리의 첫 번째 leaf payload 를 빌려 placeholder 를 만든다.
        let placeholder = self.borrow_any_leaf_payload().clone();
        let old_tree = std::mem::replace(
            self,
            PaneTree::Leaf(Leaf {
                id: PaneId::new_from_literal("__close_placeholder__"),
                payload: placeholder,
            }),
        );

        match close_owned(old_tree, target_id) {
            Ok(new_tree) => {
                *self = new_tree;
                Ok(())
            }
            Err((returned_tree, e)) => {
                *self = returned_tree;
                Err(e)
            }
        }
    }

    /// 트리에서 임의의 첫 번째 leaf payload 를 참조한다 (placeholder 생성용).
    fn borrow_any_leaf_payload(&self) -> &L {
        match self {
            PaneTree::Leaf(l) => &l.payload,
            PaneTree::Split { first, .. } => first.borrow_any_leaf_payload(),
        }
    }

    // ----------------------------------------------------------
    // 조회 API
    // ----------------------------------------------------------

    /// 트리의 leaf 수를 반환한다.
    ///
    /// 불변 조건: Split 노드는 항상 ≥ 2 leaf 를 가진다.
    pub fn leaf_count(&self) -> usize {
        match self {
            PaneTree::Leaf(_) => 1,
            PaneTree::Split { first, second, .. } => first.leaf_count() + second.leaf_count(),
        }
    }

    /// in-order (first → second) 순서로 모든 leaf 를 순회한다.
    pub fn leaves(&self) -> Vec<&Leaf<L>> {
        let mut result = Vec::new();
        self.collect_leaves(&mut result);
        result
    }

    fn collect_leaves<'a>(&'a self, acc: &mut Vec<&'a Leaf<L>>) {
        match self {
            PaneTree::Leaf(leaf) => acc.push(leaf),
            PaneTree::Split { first, second, .. } => {
                first.collect_leaves(acc);
                second.collect_leaves(acc);
            }
        }
    }

    /// 트리가 단일 leaf 일 때 해당 leaf 의 PaneId 를 반환한다.
    pub fn root_pane_id(&self) -> Option<&PaneId> {
        match self {
            PaneTree::Leaf(leaf) => Some(&leaf.id),
            PaneTree::Split { .. } => None,
        }
    }

    /// Split 노드 id 로 현재 ratio 를 조회한다.
    pub fn get_ratio(&self, node_id: &SplitNodeId) -> Result<f32, RatioError> {
        match self {
            PaneTree::Leaf(_) => Err(RatioError::SplitNodeNotFound),
            PaneTree::Split {
                id,
                ratio,
                first,
                second,
                ..
            } => {
                if id == node_id {
                    Ok(*ratio)
                } else {
                    first
                        .get_ratio(node_id)
                        .or_else(|_| second.get_ratio(node_id))
                }
            }
        }
    }

    /// Split 노드 id 의 ratio 를 설정한다.
    ///
    /// `ratio <= 0.0 || ratio >= 1.0` 또는 NaN/Inf → `Err(RatioError::OutOfBounds)` (AC-P-20).
    /// Min-size clamp 는 T5 (divider drag) 에서 추가.
    pub fn set_ratio(&mut self, node_id: &SplitNodeId, new_ratio: f32) -> Result<(), RatioError> {
        // AC-P-20: boundary 검증.
        if !new_ratio.is_finite() || new_ratio <= 0.0 || new_ratio >= 1.0 {
            return Err(RatioError::OutOfBounds);
        }
        self.set_ratio_inner(node_id, new_ratio)
    }

    fn set_ratio_inner(&mut self, node_id: &SplitNodeId, new_ratio: f32) -> Result<(), RatioError> {
        match self {
            PaneTree::Leaf(_) => Err(RatioError::SplitNodeNotFound),
            PaneTree::Split {
                id,
                ratio,
                first,
                second,
                ..
            } => {
                if id == node_id {
                    *ratio = new_ratio;
                    Ok(())
                } else {
                    first
                        .set_ratio_inner(node_id, new_ratio)
                        .or_else(|_| second.set_ratio_inner(node_id, new_ratio))
                }
            }
        }
    }

    /// 트리의 첫 번째 Split 노드 ID 를 반환한다 (get_ratio / set_ratio 테스트 목적).
    pub fn find_split_node_id(&self) -> Option<&SplitNodeId> {
        match self {
            PaneTree::Leaf(_) => None,
            PaneTree::Split { id, .. } => Some(id),
        }
    }

    /// target_id leaf 의 payload 를 new_payload 로 교체한다 (REQ-MV-080).
    ///
    /// target 이 존재하면 `true`, 존재하지 않으면 `false` 를 반환한다.
    pub fn set_leaf_payload(&mut self, target_id: &PaneId, new_payload: L) -> bool {
        match self {
            PaneTree::Leaf(leaf) => {
                if &leaf.id == target_id {
                    leaf.payload = new_payload;
                    true
                } else {
                    false
                }
            }
            PaneTree::Split { first, second, .. } => {
                if first.set_leaf_payload(target_id, new_payload.clone()) {
                    true
                } else {
                    second.set_leaf_payload(target_id, new_payload)
                }
            }
        }
    }
}

// ============================================================
// close 알고리즘 — owned 소비 재귀
// ============================================================

/// tree 를 소비하여 target_id leaf 를 제거한 새 tree 를 반환한다.
///
/// `Ok(new_tree)`:        제거 성공, new_tree 를 사용하라.
/// `Err((tree, NotFound)`: target 없음, 원래 tree 를 반환.
///
/// 알고리즘:
/// - Leaf 가 target → 부모가 형제를 대신 반환해야 하므로 `Err` 에 special sentinel 를 쓰지 않고
///   `CloseLeafResult` 패턴으로 처리한다.
/// - Split 의 경우 자녀를 재귀적으로 처리하며, 자녀가 "제거됨"을 알리면 형제를 그 자리에 넣는다.
fn close_owned<L: Clone>(
    tree: PaneTree<L>,
    target_id: &PaneId,
) -> Result<PaneTree<L>, (PaneTree<L>, SplitError)> {
    match tree {
        PaneTree::Leaf(ref l) if &l.id == target_id => {
            // 이 leaf 자신이 target — 부모가 처리해야 하므로 여기서는 Err 로 알린다.
            // (close_pane 은 단일 leaf no-op 을 이미 처리했으므로 이 경로는 부모 Split 내부에서만 호출됨.)
            Err((tree, SplitError::TargetNotFound)) // sentinel: 실제로 부모가 형제를 반환
        }
        PaneTree::Leaf(_) => {
            // 다른 leaf — target 없음.
            Err((tree, SplitError::TargetNotFound))
        }
        PaneTree::Split {
            direction,
            ratio,
            first,
            second,
            id,
        } => {
            // first 자녀가 target 인지 확인.
            let first_is_target = matches!(first.as_ref(), PaneTree::Leaf(l) if &l.id == target_id);
            if first_is_target {
                // first 제거 → second 가 이 Split 위치를 승계.
                return Ok(*second);
            }

            // second 자녀가 target 인지 확인.
            let second_is_target =
                matches!(second.as_ref(), PaneTree::Leaf(l) if &l.id == target_id);
            if second_is_target {
                // second 제거 → first 가 이 Split 위치를 승계.
                return Ok(*first);
            }

            // 깊은 재귀: first 서브트리 탐색.
            match close_owned(*first, target_id) {
                Ok(new_first) => Ok(PaneTree::Split {
                    direction,
                    ratio,
                    first: Box::new(new_first),
                    second,
                    id,
                }),
                Err((returned_first, _)) => {
                    // first 에서 찾지 못함 → second 탐색.
                    match close_owned(*second, target_id) {
                        Ok(new_second) => Ok(PaneTree::Split {
                            direction,
                            ratio,
                            first: Box::new(returned_first),
                            second: Box::new(new_second),
                            id,
                        }),
                        Err((returned_second, e)) => Err((
                            PaneTree::Split {
                                direction,
                                ratio,
                                first: Box::new(returned_first),
                                second: Box::new(returned_second),
                                id,
                            },
                            e,
                        )),
                    }
                }
            }
        }
    }
}

// ============================================================
// #[cfg(test)] 단위 테스트
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------
    // 테스트 헬퍼
    // -------------------------------------------------------

    /// 테스트용 leaf PaneTree 생성 (payload = id 문자열).
    fn leaf(id: &str) -> PaneTree<String> {
        PaneTree::new_leaf(PaneId::new_from_literal(id), id.to_string())
    }

    fn pid(s: &str) -> PaneId {
        PaneId::new_from_literal(s)
    }

    // -------------------------------------------------------
    // AC-P-1: split_horizontal / split_vertical
    // -------------------------------------------------------

    /// Horizontal split 을 leaf 에서 실행하면 leaf_count == 2 이고
    /// first=기존, second=신규 순서임을 확인한다.
    #[test]
    fn split_horizontal_from_leaf() {
        let mut tree = leaf("a");
        tree.split_horizontal(&pid("a"), pid("b"), "b".to_string())
            .expect("split 성공");

        assert_eq!(tree.leaf_count(), 2, "split 후 leaf 수 == 2");

        let leaves = tree.leaves();
        assert_eq!(leaves[0].id, pid("a"), "first = 기존 leaf");
        assert_eq!(leaves[1].id, pid("b"), "second = 신규 leaf");
    }

    /// Vertical split 은 Horizontal 과 대칭적으로 동작한다.
    #[test]
    fn split_vertical_from_leaf() {
        let mut tree = leaf("a");
        tree.split_vertical(&pid("a"), pid("b"), "b".to_string())
            .expect("split 성공");

        assert_eq!(tree.leaf_count(), 2);

        let leaves = tree.leaves();
        assert_eq!(leaves[0].id, pid("a"), "first = top");
        assert_eq!(leaves[1].id, pid("b"), "second = bottom");
    }

    // -------------------------------------------------------
    // spec.md §7.1 용어 고정 검증
    // -------------------------------------------------------

    /// Horizontal → first=left / second=right, Vertical → first=top / second=bottom.
    /// SplitDirection 이 방향 계약대로 저장됨을 확인한다.
    #[test]
    fn split_direction_first_second_semantics() {
        let mut h_tree = leaf("a");
        h_tree
            .split_horizontal(&pid("a"), pid("b"), "b".to_string())
            .unwrap();

        if let PaneTree::Split {
            direction,
            first,
            second,
            ..
        } = &h_tree
        {
            assert_eq!(
                *direction,
                SplitDirection::Horizontal,
                "horizontal split 방향"
            );
            assert!(matches!(first.as_ref(), PaneTree::Leaf(l) if l.id == pid("a")));
            assert!(matches!(second.as_ref(), PaneTree::Leaf(l) if l.id == pid("b")));
        } else {
            panic!("Split 노드여야 한다");
        }

        let mut v_tree = leaf("x");
        v_tree
            .split_vertical(&pid("x"), pid("y"), "y".to_string())
            .unwrap();

        if let PaneTree::Split {
            direction,
            first,
            second,
            ..
        } = &v_tree
        {
            assert_eq!(*direction, SplitDirection::Vertical, "vertical split 방향");
            assert!(matches!(first.as_ref(), PaneTree::Leaf(l) if l.id == pid("x")));
            assert!(matches!(second.as_ref(), PaneTree::Leaf(l) if l.id == pid("y")));
        } else {
            panic!("Split 노드여야 한다");
        }
    }

    // -------------------------------------------------------
    // AC-P-2: close_pane — 형제 승계
    // -------------------------------------------------------

    /// close_pane 이 형제 서브트리를 부모 위치로 승계시킨다.
    #[test]
    fn close_promotes_sibling() {
        let mut tree = leaf("a");
        tree.split_horizontal(&pid("a"), pid("b"), "b".to_string())
            .unwrap();

        tree.close_pane(&pid("a")).expect("a 닫기 성공");

        assert_eq!(tree.leaf_count(), 1);
        assert_eq!(tree.root_pane_id(), Some(&pid("b")), "b 가 루트로 승계");
    }

    // -------------------------------------------------------
    // AC-P-3: close_pane on last leaf — no-op
    // -------------------------------------------------------

    /// 마지막 단일 leaf 에 close_pane 을 호출하면 no-op 이고 leaf 가 유지된다.
    #[test]
    fn close_last_leaf_is_noop() {
        let mut tree = leaf("only");
        tree.close_pane(&pid("only")).expect("no-op 이어야 함");
        assert_eq!(tree.leaf_count(), 1);
        assert_eq!(tree.root_pane_id(), Some(&pid("only")));
    }

    // -------------------------------------------------------
    // AC-P-20: ratio boundary
    // -------------------------------------------------------

    /// set_ratio(0.0) 과 set_ratio(1.0) 은 모두 Err(OutOfBounds) 를 반환해야 한다.
    #[test]
    fn ratio_boundary_rejected() {
        let mut tree = leaf("a");
        tree.split_horizontal(&pid("a"), pid("b"), "b".to_string())
            .unwrap();
        let node_id = tree.find_split_node_id().unwrap().clone();

        assert_eq!(tree.set_ratio(&node_id, 0.0), Err(RatioError::OutOfBounds));
        assert_eq!(tree.set_ratio(&node_id, 1.0), Err(RatioError::OutOfBounds));
        assert_eq!(
            tree.set_ratio(&node_id, f32::NAN),
            Err(RatioError::OutOfBounds)
        );
        assert_eq!(
            tree.set_ratio(&node_id, f32::INFINITY),
            Err(RatioError::OutOfBounds)
        );
    }

    // -------------------------------------------------------
    // in-order 순회 순서 보증
    // -------------------------------------------------------

    /// leaves() 가 in-order (first → second) 순서를 보증한다. 3-level 트리 검증.
    #[test]
    fn leaves_in_order_iteration() {
        // 트리: Split{ first=Leaf(a), second=Split{ first=Leaf(b), second=Leaf(c) } }
        let mut tree = leaf("a");
        tree.split_horizontal(&pid("a"), pid("b"), "b".to_string())
            .unwrap();
        tree.split_horizontal(&pid("b"), pid("c"), "c".to_string())
            .unwrap();

        let leaves = tree.leaves();
        assert_eq!(leaves.len(), 3);
        assert_eq!(leaves[0].id, pid("a"));
        assert_eq!(leaves[1].id, pid("b"));
        assert_eq!(leaves[2].id, pid("c"));
    }

    // -------------------------------------------------------
    // leaf_count 정확성
    // -------------------------------------------------------

    /// 연속 split 후 leaf_count 가 정확히 누적된다.
    #[test]
    fn leaf_count_after_splits() {
        let mut tree = leaf("root");
        assert_eq!(tree.leaf_count(), 1);

        tree.split_horizontal(&pid("root"), pid("p2"), "p2".to_string())
            .unwrap();
        assert_eq!(tree.leaf_count(), 2);

        tree.split_vertical(&pid("p2"), pid("p3"), "p3".to_string())
            .unwrap();
        assert_eq!(tree.leaf_count(), 3);
    }

    // -------------------------------------------------------
    // root_pane_id — 단일 leaf
    // -------------------------------------------------------

    /// 단일 leaf 트리에서 root_pane_id 는 해당 leaf 의 PaneId 를 반환한다.
    #[test]
    fn root_pane_id_returns_leaf_id() {
        let tree = leaf("solo");
        assert_eq!(tree.root_pane_id(), Some(&pid("solo")));
    }

    // -------------------------------------------------------
    // get_ratio / set_ratio round-trip
    // -------------------------------------------------------

    /// set_ratio 로 설정한 값을 get_ratio 로 다시 읽을 수 있다.
    #[test]
    fn get_set_ratio_round_trip() {
        let mut tree = leaf("a");
        tree.split_horizontal(&pid("a"), pid("b"), "b".to_string())
            .unwrap();
        let node_id = tree.find_split_node_id().unwrap().clone();

        // 기본값 0.5 확인
        assert!((tree.get_ratio(&node_id).unwrap() - 0.5).abs() < f32::EPSILON);

        // 0.3 으로 설정 후 읽기
        tree.set_ratio(&node_id, 0.3).unwrap();
        assert!((tree.get_ratio(&node_id).unwrap() - 0.3).abs() < f32::EPSILON);
    }

    // -------------------------------------------------------
    // 추가 edge case
    // -------------------------------------------------------

    /// 존재하지 않는 PaneId 에 split 하면 Err(TargetNotFound).
    #[test]
    fn split_nonexistent_target_returns_error() {
        let mut tree = leaf("a");
        let result = tree.split_horizontal(&pid("nonexistent"), pid("b"), "b".to_string());
        assert_eq!(result, Err(SplitError::TargetNotFound));
    }

    /// Split 노드 ID (PaneId 가 아님) 로 split 을 호출하면 Err(TargetNotFound).
    #[test]
    fn split_on_split_node_id_is_not_found() {
        let mut tree = leaf("a");
        tree.split_horizontal(&pid("a"), pid("b"), "b".to_string())
            .unwrap();

        let result = tree.split_horizontal(&pid("some-split-id"), pid("c"), "c".to_string());
        assert_eq!(result, Err(SplitError::TargetNotFound));
    }

    /// close_pane — 3-level 중첩 split 에서 내부 leaf 제거 시 sibling 트리가 올바르게 mount.
    ///
    /// hand-traced 테스트:
    /// 초기:   Split{ first=Leaf(a), second=Split{ first=Leaf(b), second=Leaf(c) } }
    /// b 제거: Split{ first=Leaf(a), second=Leaf(c) }
    #[test]
    fn close_nested_promotes_subtree() {
        let mut tree = leaf("a");
        tree.split_horizontal(&pid("a"), pid("b"), "b".to_string())
            .unwrap();
        tree.split_horizontal(&pid("b"), pid("c"), "c".to_string())
            .unwrap();

        assert_eq!(tree.leaf_count(), 3);

        tree.close_pane(&pid("b")).expect("b 닫기 성공");

        let leaves = tree.leaves();
        assert_eq!(leaves.len(), 2, "a 와 c 만 남아야 함");
        assert_eq!(leaves[0].id, pid("a"));
        assert_eq!(leaves[1].id, pid("c"));
    }
}
