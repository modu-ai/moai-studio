//! PaneTree → GPUI element tree 재귀 변환 (SPEC-V3-004 RG-R-2).
//!
//! 스펙 참조:
//! - spec.md §5 RG-R-2 (REQ-R-010 ~ REQ-R-014)
//! - spec.md §7.1 아키텍처 그림: PaneTree::Leaf → leaf.into_element()
//!   PaneTree::Split{Horizontal} → flex_row + divider
//!   PaneTree::Split{Vertical}   → flex_col + divider
//!
//! ## 설계 노트
//!
//! GPUI TestAppContext 없이 테스트하기 위해 `count_dividers` / `count_leaves` 구조 헬퍼를
//! 별도로 제공한다 (USER-DECISION-REQUIRED: gpui-test-support-adoption-v3-004 → option-b 채택).
//! 실제 GPUI element 생성은 `render_pane_tree` 가 담당하고,
//! AC-R-2/R-4 는 `count_*` 헬퍼를 통한 logic-level 검증으로 커버한다.

use crate::design::tokens as tok;
use crate::panes::{PaneTree, SplitDirection};
use gpui::{IntoElement, ParentElement, Styled, div};

// @MX:ANCHOR: [AUTO] render-pane-tree-recursion
// @MX:REASON: [AUTO] SPEC-V3-004 REQ-R-010 ~ REQ-R-014. PaneTree → GPUI element tree 변환 진입점.
//   fan_in >= 3: TabContainer.render (MS-2), integration_render 테스트 (AC-R-2), 향후 PTY-per-pane SPEC.
//   Horizontal split → flex_row + 수직 divider.
//   Vertical split   → flex_col + 수평 divider.
/// PaneTree<L> 를 GPUI element tree 로 재귀 변환한다.
///
/// AC-R-2 검증: 1 회 horizontal split 시 flex_row 1 개 + leaf 2 개 + divider 1 개.
/// AC-R-7 검증: N split 노드 → divider 정확히 N 개.
///
/// `L: IntoElement + Clone` 제약:
///   - `IntoElement`: leaf payload 를 div 자식으로 마운트 가능.
///   - `Clone`: 재귀 구조에서 소유권 전달 필요.
pub fn render_pane_tree<L>(tree: &PaneTree<L>) -> impl IntoElement
where
    L: IntoElement + Clone + 'static,
{
    match tree {
        PaneTree::Leaf(leaf) => {
            // REQ-R-011: leaf payload 를 GPUI element 로 마운트.
            div()
                .flex()
                .flex_col()
                .flex_grow()
                .child(leaf.payload.clone())
        }
        PaneTree::Split {
            direction,
            first,
            second,
            ..
        } => {
            match direction {
                SplitDirection::Horizontal => {
                    // REQ-R-012: flex_row(first / 수직 divider / second)
                    div()
                        .flex()
                        .flex_row()
                        .flex_grow()
                        .child(render_pane_tree(first.as_ref()))
                        .child(divider_vertical())
                        .child(render_pane_tree(second.as_ref()))
                }
                SplitDirection::Vertical => {
                    // REQ-R-013: flex_col(first / 수평 divider / second)
                    div()
                        .flex()
                        .flex_col()
                        .flex_grow()
                        .child(render_pane_tree(first.as_ref()))
                        .child(divider_horizontal())
                        .child(render_pane_tree(second.as_ref()))
                }
            }
        }
    }
}

/// 수직 divider element (Horizontal split 의 left/right 사이, REQ-R-014).
///
/// REQ-R-014: Split 노드 당 정확히 1 개의 divider element 생성.
pub fn divider_vertical() -> impl IntoElement {
    use gpui::rgb;
    div()
        .w(gpui::px(4.0))
        .h_full()
        .bg(rgb(tok::BORDER_SUBTLE))
        .flex_shrink_0()
}

/// 수평 divider element (Vertical split 의 top/bottom 사이, REQ-R-014).
///
/// REQ-R-014: Split 노드 당 정확히 1 개의 divider element 생성.
pub fn divider_horizontal() -> impl IntoElement {
    use gpui::rgb;
    div()
        .h(gpui::px(4.0))
        .w_full()
        .bg(rgb(tok::BORDER_SUBTLE))
        .flex_shrink_0()
}

// ============================================================
// 구조적 카운팅 헬퍼 (AC-R-2/7 logic-level 검증, GPUI context 불필요)
// ============================================================

/// PaneTree 에서 Split 노드 수를 재귀 카운팅한다.
///
/// AC-R-2: split 1 회 → `count_splits == 1`.
/// AC-R-7: N split 노드 → divider N 개 (split 수 == divider 수 불변).
pub fn count_splits<L>(tree: &PaneTree<L>) -> usize {
    match tree {
        PaneTree::Leaf(_) => 0,
        PaneTree::Split { first, second, .. } => {
            1 + count_splits(first.as_ref()) + count_splits(second.as_ref())
        }
    }
}

/// PaneTree 에서 Leaf 노드 수를 재귀 카운팅한다.
///
/// AC-R-2: split 1 회 → `count_leaves == 2` (original + new).
pub fn count_leaves<L>(tree: &PaneTree<L>) -> usize {
    match tree {
        PaneTree::Leaf(_) => 1,
        PaneTree::Split { first, second, .. } => {
            count_leaves(first.as_ref()) + count_leaves(second.as_ref())
        }
    }
}

// ============================================================
// 단위 테스트 (RED-GREEN-REFACTOR)
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panes::{PaneId, PaneTree};

    // -------------------------------------------------------
    // T4 AC-R-2: 1 split → leaf 2, divider 1
    // -------------------------------------------------------

    /// 단일 leaf 트리는 split 0 개, leaf 1 개 — divider 없음.
    #[test]
    fn single_leaf_has_no_splits() {
        let tree: PaneTree<String> = PaneTree::new_leaf(PaneId::new_unique(), "pane-0".to_string());
        assert_eq!(count_splits(&tree), 0, "단일 leaf: split 0");
        assert_eq!(count_leaves(&tree), 1, "단일 leaf: leaf 1");
    }

    /// 1 회 horizontal split → split 1 개, leaf 2 개 (AC-R-2 핵심 검증).
    #[test]
    fn single_horizontal_split_emits_one_divider() {
        let mut tree: PaneTree<String> =
            PaneTree::new_leaf(PaneId::new_unique(), "pane-0".to_string());
        let root_id = match &tree {
            PaneTree::Leaf(l) => l.id.clone(),
            _ => unreachable!(),
        };
        tree.split_horizontal(&root_id, PaneId::new_unique(), "pane-1".to_string())
            .expect("split_horizontal 은 성공해야 한다");

        // AC-R-2: split 1 회 → split 노드 1, leaf 2, divider 1 (split 수 == divider 수)
        assert_eq!(
            count_splits(&tree),
            1,
            "horizontal split 1 회 → split 노드 1"
        );
        assert_eq!(count_leaves(&tree), 2, "horizontal split 1 회 → leaf 2");
    }

    /// 1 회 vertical split → split 1 개, leaf 2 개.
    #[test]
    fn single_vertical_split_emits_one_divider() {
        let mut tree: PaneTree<String> =
            PaneTree::new_leaf(PaneId::new_unique(), "pane-0".to_string());
        let root_id = match &tree {
            PaneTree::Leaf(l) => l.id.clone(),
            _ => unreachable!(),
        };
        tree.split_vertical(&root_id, PaneId::new_unique(), "pane-1".to_string())
            .expect("split_vertical 은 성공해야 한다");

        assert_eq!(count_splits(&tree), 1, "vertical split 1 회 → split 노드 1");
        assert_eq!(count_leaves(&tree), 2, "vertical split 1 회 → leaf 2");
    }

    // -------------------------------------------------------
    // T4 AC-R-7: 3-level split → split 3, leaf 4, divider 3
    // -------------------------------------------------------

    /// 3 level split (1 horizontal + 2 vertical) → split 3 개, leaf 4 개.
    ///
    /// 구조:
    ///   Split{H}
    ///     ├── Split{V}
    ///     │     ├── Leaf("pane-0")
    ///     │     └── Leaf("pane-2")
    ///     └── Split{V}
    ///           ├── Leaf("pane-1")
    ///           └── Leaf("pane-3")
    #[test]
    fn three_level_split_emits_three_dividers() {
        // 1. 단일 leaf 트리
        let mut tree: PaneTree<String> =
            PaneTree::new_leaf(PaneId::new_unique(), "pane-0".to_string());
        let id0 = match &tree {
            PaneTree::Leaf(l) => l.id.clone(),
            _ => unreachable!(),
        };

        // 2. horizontal split: pane-0 | pane-1
        let id1 = PaneId::new_unique();
        tree.split_horizontal(&id0, id1.clone(), "pane-1".to_string())
            .unwrap();
        assert_eq!(count_splits(&tree), 1);
        assert_eq!(count_leaves(&tree), 2);

        // 3. vertical split on pane-0 (left): pane-0 / pane-2
        let id2 = PaneId::new_unique();
        tree.split_vertical(&id0, id2, "pane-2".to_string())
            .unwrap();
        assert_eq!(count_splits(&tree), 2);
        assert_eq!(count_leaves(&tree), 3);

        // 4. vertical split on pane-1 (right): pane-1 / pane-3
        let id3 = PaneId::new_unique();
        tree.split_vertical(&id1, id3, "pane-3".to_string())
            .unwrap();

        // AC-R-7: split 3 개, leaf 4 개, divider 3 개
        assert_eq!(count_splits(&tree), 3, "3-level split → split 노드 3");
        assert_eq!(count_leaves(&tree), 4, "3-level split → leaf 4");
    }
}
