//! MS-4 Tab CRUD completion tests — AC-P-36 / AC-P-37.
//!
//! RED phase: move_tab and duplicate_tab do not yet exist on TabContainer.
//! These tests will fail until GREEN implementation in container.rs.
//!
//! AC-P-36: move_tab boundary cases (same position, out-of-range)
//! AC-P-37: duplicate_tab clones pane tree independently (no shared state)

use moai_studio_ui::panes::PaneId;
use moai_studio_ui::tabs::TabContainer;

// ============================================================
// AC-P-36: move_tab — basic movement
// ============================================================

/// move_tab from index 0 to index 2 in a 3-tab container reorders correctly.
#[test]
fn ac_p36_move_tab_forward() {
    let mut c = TabContainer::new(); // tab-0
    let id1 = c.new_tab(None); // tab-1
    let id2 = c.new_tab(None); // tab-2

    // Remember original tab at idx 0
    let id0 = c.tabs[0].id.clone();

    // Move tab-0 to index 2
    c.move_tab(0, 2).expect("move_tab(0, 2) should succeed");

    // Now: tab-1, tab-2, tab-0
    assert_eq!(
        c.tabs[0].id, id1,
        "AC-P-36: after move(0→2), idx-0 = former idx-1"
    );
    assert_eq!(
        c.tabs[1].id, id2,
        "AC-P-36: after move(0→2), idx-1 = former idx-2"
    );
    assert_eq!(
        c.tabs[2].id, id0,
        "AC-P-36: after move(0→2), idx-2 = former idx-0"
    );
}

/// move_tab from index 2 to index 0 in a 3-tab container reorders correctly.
#[test]
fn ac_p36_move_tab_backward() {
    let mut c = TabContainer::new(); // tab-0
    let id1 = c.new_tab(None); // tab-1
    c.new_tab(None); // tab-2
    let id0 = c.tabs[0].id.clone();
    let id2 = c.tabs[2].id.clone();

    // Move tab-2 to index 0
    c.move_tab(2, 0).expect("move_tab(2, 0) should succeed");

    // Now: tab-2, tab-0, tab-1
    assert_eq!(
        c.tabs[0].id, id2,
        "AC-P-36: after move(2→0), idx-0 = former idx-2"
    );
    assert_eq!(
        c.tabs[1].id, id0,
        "AC-P-36: after move(2→0), idx-1 = former idx-0"
    );
    assert_eq!(
        c.tabs[2].id, id1,
        "AC-P-36: after move(2→0), idx-2 = former idx-1"
    );
}

/// move_tab to the same position is a no-op (no error, no change).
#[test]
fn ac_p36_move_tab_same_position_noop() {
    let mut c = TabContainer::new(); // tab-0
    c.new_tab(None); // tab-1
    c.new_tab(None); // tab-2

    let before: Vec<_> = c.tabs.iter().map(|t| t.id.clone()).collect();

    // Move tab-1 to same index 1 — should succeed with no change
    c.move_tab(1, 1)
        .expect("move_tab(same) is a no-op, must succeed");

    let after: Vec<_> = c.tabs.iter().map(|t| t.id.clone()).collect();
    assert_eq!(
        before, after,
        "AC-P-36: move to same index must not reorder tabs"
    );
}

/// move_tab with from index out of range returns an error.
#[test]
fn ac_p36_move_tab_from_out_of_range_errors() {
    let mut c = TabContainer::new(); // 1 tab only
    let result = c.move_tab(5, 0);
    assert!(
        result.is_err(),
        "AC-P-36: from index out of range must return Err"
    );
}

/// move_tab with to index out of range returns an error.
#[test]
fn ac_p36_move_tab_to_out_of_range_errors() {
    let mut c = TabContainer::new(); // 1 tab only
    c.new_tab(None); // 2 tabs
    let result = c.move_tab(0, 99);
    assert!(
        result.is_err(),
        "AC-P-36: to index out of range must return Err"
    );
}

/// move_tab updates active_tab_idx to follow the moved tab when it was active.
#[test]
fn ac_p36_move_active_tab_updates_active_idx() {
    let mut c = TabContainer::new(); // idx 0 (active after new)
    c.new_tab(None); // idx 1 (now active)
    c.new_tab(None); // idx 2 (now active)

    // Set active to idx 0
    c.switch_tab(0).unwrap();
    assert_eq!(c.active_tab_idx, 0);

    // Move active tab (idx 0) to idx 2
    c.move_tab(0, 2).expect("move active tab");

    // active_tab_idx must follow the moved tab
    assert_eq!(
        c.active_tab_idx, 2,
        "AC-P-36: active_tab_idx must follow the moved tab to its new position"
    );
}

// ============================================================
// AC-P-37: duplicate_tab — independent clone
// ============================================================

/// duplicate_tab clones a single-leaf tab; both tabs have identical structure
/// but modifications to one do not affect the other.
#[test]
fn ac_p37_duplicate_tab_single_leaf_independent() {
    let mut c = TabContainer::new(); // tab-0, single leaf
    assert_eq!(c.tab_count(), 1);

    // Duplicate tab-0
    let new_idx = c
        .duplicate_tab(0)
        .expect("AC-P-37: duplicate_tab(0) must succeed");
    assert_eq!(c.tab_count(), 2, "AC-P-37: after duplicate, 2 tabs exist");

    // Both tabs have single leaf
    assert_eq!(
        c.tabs[0].pane_tree.leaf_count(),
        1,
        "AC-P-37: original still single leaf"
    );
    assert_eq!(
        c.tabs[new_idx].pane_tree.leaf_count(),
        1,
        "AC-P-37: duplicate is single leaf"
    );

    // Duplicate has a DIFFERENT pane id (independent clone, not shared)
    let orig_pane_id = match &c.tabs[0].pane_tree {
        moai_studio_ui::panes::PaneTree::Leaf(l) => l.id.clone(),
        _ => panic!("expected Leaf"),
    };
    let dup_pane_id = match &c.tabs[new_idx].pane_tree {
        moai_studio_ui::panes::PaneTree::Leaf(l) => l.id.clone(),
        _ => panic!("expected Leaf"),
    };
    assert_ne!(
        orig_pane_id, dup_pane_id,
        "AC-P-37: duplicate must have NEW unique pane IDs (independent clone)"
    );
}

/// duplicate_tab on a split tab clones the entire tree independently.
/// Splitting the duplicate does not affect the original.
#[test]
fn ac_p37_duplicate_split_tab_is_independent() {
    let mut c = TabContainer::new();

    // Split tab-0: create 2-pane layout
    let focused = c.active_tab().last_focused_pane.clone().unwrap();
    c.active_tab_mut()
        .pane_tree
        .split_horizontal(&focused, PaneId::new_unique(), "right".to_string())
        .expect("split succeeds");

    assert_eq!(c.tabs[0].pane_tree.leaf_count(), 2);

    // Duplicate tab-0
    let dup_idx = c.duplicate_tab(0).expect("AC-P-37: duplicate split tab");
    assert_eq!(
        c.tabs[dup_idx].pane_tree.leaf_count(),
        2,
        "AC-P-37: duplicate has 2 leaves"
    );

    // Further split the duplicate — must NOT affect the original
    let dup_focused = c.tabs[dup_idx].last_focused_pane.clone();
    if let Some(pane_id) = dup_focused {
        c.tabs[dup_idx]
            .pane_tree
            .split_vertical(&pane_id, PaneId::new_unique(), "bottom".to_string())
            .unwrap_or(()); // may fail if that pane no longer exists; ignore for this test
    }

    // Original must still have exactly 2 leaves
    assert_eq!(
        c.tabs[0].pane_tree.leaf_count(),
        2,
        "AC-P-37: original tab must still have 2 leaves after modifying duplicate"
    );
}

/// duplicate_tab returns the index of the newly created tab (appended after original).
#[test]
fn ac_p37_duplicate_tab_appended_at_end() {
    let mut c = TabContainer::new(); // tab-0
    c.new_tab(None); // tab-1
    // Duplicate tab-0 → should appear at index 2 (end)
    let new_idx = c.duplicate_tab(0).expect("AC-P-37: duplicate tab-0");
    assert_eq!(new_idx, 2, "AC-P-37: duplicate appended at end, idx=2");
    assert_eq!(c.tab_count(), 3, "AC-P-37: 3 tabs total");
}

/// duplicate_tab with out-of-range index returns an error.
#[test]
fn ac_p37_duplicate_tab_out_of_range_errors() {
    let mut c = TabContainer::new();
    let result = c.duplicate_tab(99);
    assert!(
        result.is_err(),
        "AC-P-37: out-of-range duplicate returns Err"
    );
}

/// duplicate_tab preserves the title of the original tab.
#[test]
fn ac_p37_duplicate_tab_preserves_title() {
    use std::path::PathBuf;
    let mut c = TabContainer::new();
    // Create tab with a known title via cwd
    c.new_tab(Some(PathBuf::from("/home/user/myproject")));
    // tab index 1 has title "myproject"
    let orig_title = c.tabs[1].title.clone();

    let dup_idx = c
        .duplicate_tab(1)
        .expect("AC-P-37: duplicate tab with title");
    // Duplicate should preserve the title
    assert_eq!(
        c.tabs[dup_idx].title, orig_title,
        "AC-P-37: duplicate tab preserves original title"
    );
    // But must have a different TabId
    assert_ne!(
        c.tabs[dup_idx].id, c.tabs[1].id,
        "AC-P-37: duplicate tab has new unique TabId"
    );
}
