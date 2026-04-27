//! T13 panes_convert round-trip tests — comprehensive coverage for A-1/A-2 persistence.
//!
//! These tests verify the complete DTO conversion layer:
//! - Empty PaneTree snapshot round-trip
//! - Single leaf with tabs
//! - Binary split (Horizontal/Vertical) preserving ratio
//! - Nested splits preserving tree shape
//! - last_focused_pane id round-trip
//! - Missing/corrupt JSON graceful degradation

use moai_studio_workspace::panes_convert::{
    PaneTreeInput, SplitDirectionInput, TabSnapshotInput, layout_v1_to_tab_inputs, snapshot_path,
    tab_container_to_layout_v1,
};
use moai_studio_workspace::persistence::{
    PaneLayoutV1, PaneTreeSnapshotV1, SCHEMA_VERSION, TabSnapshotV1, load_panes, save_panes,
};
use std::path::PathBuf;

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("moai-panes-convert-{}.json", name))
}

// ============================================================
// Test 1: Empty tab list round-trip
// ============================================================

/// Empty PaneLayoutV1 (no tabs) serializes and deserializes cleanly.
#[test]
fn empty_layout_roundtrip() {
    let layout = PaneLayoutV1 {
        schema_version: SCHEMA_VERSION.to_string(),
        tabs: vec![],
    };

    let path = temp_path("empty-layout");
    let _ = std::fs::remove_file(&path);

    save_panes(&path, &layout).expect("save empty layout");
    let restored = load_panes(&path).expect("load empty layout");

    assert_eq!(restored.schema_version, SCHEMA_VERSION);
    assert!(restored.tabs.is_empty(), "empty tabs preserved");

    // Convert to DTO and back
    let inputs = layout_v1_to_tab_inputs(&restored);
    assert!(
        inputs.is_empty(),
        "DTO conversion of empty layout yields empty vec"
    );

    let re_layout = tab_container_to_layout_v1(&inputs);
    assert_eq!(re_layout.tabs.len(), 0);

    let _ = std::fs::remove_file(&path);
}

// ============================================================
// Test 2: Single pane with tabs (Leaf only)
// ============================================================

/// Single tab with a single leaf pane round-trips through DTO and JSON.
#[test]
fn single_leaf_tab_roundtrip() {
    let input = TabSnapshotInput {
        id: "tab-single".to_string(),
        title: "Home".to_string(),
        last_focused_pane: Some("pane-home".to_string()),
        pane_tree: PaneTreeInput::Leaf {
            id: "pane-home".to_string(),
            cwd: Some(PathBuf::from("/home/user")),
        },
    };
    let inputs = vec![input.clone()];

    // DTO → layout
    let layout = tab_container_to_layout_v1(&inputs);
    assert_eq!(layout.schema_version, SCHEMA_VERSION);
    assert_eq!(layout.tabs.len(), 1);
    assert_eq!(layout.tabs[0].id, "tab-single");
    assert_eq!(
        layout.tabs[0].last_focused_pane,
        Some("pane-home".to_string())
    );

    // Layout → DTO
    let restored_inputs = layout_v1_to_tab_inputs(&layout);
    assert_eq!(restored_inputs, inputs, "single leaf tab DTO round-trip");

    // Full JSON round-trip
    let path = temp_path("single-leaf");
    let _ = std::fs::remove_file(&path);
    save_panes(&path, &layout).expect("save single leaf");
    let loaded = load_panes(&path).expect("load single leaf");
    let final_inputs = layout_v1_to_tab_inputs(&loaded);
    assert_eq!(final_inputs, inputs, "single leaf JSON round-trip");

    let _ = std::fs::remove_file(&path);
}

// ============================================================
// Test 3: Horizontal split — ratio preserved
// ============================================================

/// Binary horizontal split with custom ratio round-trips preserving exact ratio.
#[test]
fn horizontal_split_preserves_ratio() {
    let input = TabSnapshotInput {
        id: "tab-h".to_string(),
        title: "Split".to_string(),
        last_focused_pane: Some("pane-left".to_string()),
        pane_tree: PaneTreeInput::Split {
            id: "split-h".to_string(),
            direction: SplitDirectionInput::Horizontal,
            ratio: 0.35,
            first: Box::new(PaneTreeInput::Leaf {
                id: "pane-left".to_string(),
                cwd: Some(PathBuf::from("/tmp/left")),
            }),
            second: Box::new(PaneTreeInput::Leaf {
                id: "pane-right".to_string(),
                cwd: None,
            }),
        },
    };
    let inputs = vec![input.clone()];

    let layout = tab_container_to_layout_v1(&inputs);
    let restored = layout_v1_to_tab_inputs(&layout);

    assert_eq!(restored, inputs, "horizontal split DTO round-trip");

    // Verify ratio is preserved exactly
    match &restored[0].pane_tree {
        PaneTreeInput::Split {
            ratio, direction, ..
        } => {
            assert!(
                (ratio - 0.35_f32).abs() < f32::EPSILON,
                "ratio preserved: {}",
                ratio
            );
            assert_eq!(*direction, SplitDirectionInput::Horizontal);
        }
        _ => panic!("root should be Split"),
    }
}

// ============================================================
// Test 4: Vertical split — ratio preserved
// ============================================================

/// Binary vertical split with custom ratio round-trips preserving exact ratio.
#[test]
fn vertical_split_preserves_ratio() {
    let input = TabSnapshotInput {
        id: "tab-v".to_string(),
        title: "Vertical".to_string(),
        last_focused_pane: None,
        pane_tree: PaneTreeInput::Split {
            id: "split-v".to_string(),
            direction: SplitDirectionInput::Vertical,
            ratio: 0.7,
            first: Box::new(PaneTreeInput::Leaf {
                id: "pane-top".to_string(),
                cwd: None,
            }),
            second: Box::new(PaneTreeInput::Leaf {
                id: "pane-bottom".to_string(),
                cwd: Some(PathBuf::from("/var/log")),
            }),
        },
    };
    let inputs = vec![input.clone()];

    let layout = tab_container_to_layout_v1(&inputs);
    let restored = layout_v1_to_tab_inputs(&layout);

    assert_eq!(restored, inputs, "vertical split DTO round-trip");

    match &restored[0].pane_tree {
        PaneTreeInput::Split {
            ratio, direction, ..
        } => {
            assert!((ratio - 0.7_f32).abs() < f32::EPSILON);
            assert_eq!(*direction, SplitDirectionInput::Vertical);
        }
        _ => panic!("root should be Split"),
    }
}

// ============================================================
// Test 5: Nested splits — tree shape preserved
// ============================================================

/// Three-pane nested split (H-Split → V-Split on right) preserves tree shape.
#[test]
fn nested_splits_preserve_tree_shape() {
    //     H-split (0.4)
    //    /              \
    // pane-a       V-split (0.6)
    //              /          \
    //           pane-b       pane-c
    let input = TabSnapshotInput {
        id: "tab-nested".to_string(),
        title: "Nested".to_string(),
        last_focused_pane: Some("pane-b".to_string()),
        pane_tree: PaneTreeInput::Split {
            id: "h-root".to_string(),
            direction: SplitDirectionInput::Horizontal,
            ratio: 0.4,
            first: Box::new(PaneTreeInput::Leaf {
                id: "pane-a".to_string(),
                cwd: Some(PathBuf::from("/tmp/a")),
            }),
            second: Box::new(PaneTreeInput::Split {
                id: "v-inner".to_string(),
                direction: SplitDirectionInput::Vertical,
                ratio: 0.6,
                first: Box::new(PaneTreeInput::Leaf {
                    id: "pane-b".to_string(),
                    cwd: None,
                }),
                second: Box::new(PaneTreeInput::Leaf {
                    id: "pane-c".to_string(),
                    cwd: Some(PathBuf::from("/home/c")),
                }),
            }),
        },
    };
    let inputs = vec![input.clone()];

    let layout = tab_container_to_layout_v1(&inputs);
    let restored = layout_v1_to_tab_inputs(&layout);

    assert_eq!(restored, inputs, "nested split DTO round-trip");

    // Verify tree shape
    match &restored[0].pane_tree {
        PaneTreeInput::Split {
            direction: SplitDirectionInput::Horizontal,
            ratio,
            second,
            ..
        } => {
            assert!((ratio - 0.4_f32).abs() < f32::EPSILON, "outer ratio");
            match second.as_ref() {
                PaneTreeInput::Split {
                    direction: SplitDirectionInput::Vertical,
                    ratio: inner_ratio,
                    ..
                } => {
                    assert!((inner_ratio - 0.6_f32).abs() < f32::EPSILON, "inner ratio");
                }
                _ => panic!("inner node should be V-Split"),
            }
        }
        _ => panic!("root should be H-Split"),
    }
}

// ============================================================
// Test 6: last_focused_pane id round-trip
// ============================================================

/// last_focused_pane id is preserved through JSON serialization.
#[test]
fn last_focused_pane_id_roundtrip() {
    let focused_id = "pane-focused-XYZ";
    let inputs = vec![TabSnapshotInput {
        id: "tab-focus".to_string(),
        title: "Focus Test".to_string(),
        last_focused_pane: Some(focused_id.to_string()),
        pane_tree: PaneTreeInput::Leaf {
            id: focused_id.to_string(),
            cwd: None,
        },
    }];

    let layout = tab_container_to_layout_v1(&inputs);
    assert_eq!(
        layout.tabs[0].last_focused_pane.as_deref(),
        Some(focused_id),
        "last_focused_pane preserved in layout"
    );

    let path = temp_path("last-focused");
    let _ = std::fs::remove_file(&path);
    save_panes(&path, &layout).expect("save");
    let loaded = load_panes(&path).expect("load");
    let final_inputs = layout_v1_to_tab_inputs(&loaded);

    assert_eq!(
        final_inputs[0].last_focused_pane.as_deref(),
        Some(focused_id),
        "last_focused_pane preserved through JSON round-trip"
    );

    let _ = std::fs::remove_file(&path);
}

/// last_focused_pane = None is also preserved.
#[test]
fn last_focused_pane_none_roundtrip() {
    let inputs = vec![TabSnapshotInput {
        id: "tab-no-focus".to_string(),
        title: "No Focus".to_string(),
        last_focused_pane: None,
        pane_tree: PaneTreeInput::Leaf {
            id: "pane-x".to_string(),
            cwd: None,
        },
    }];

    let layout = tab_container_to_layout_v1(&inputs);
    let restored = layout_v1_to_tab_inputs(&layout);
    assert!(
        restored[0].last_focused_pane.is_none(),
        "None last_focused_pane preserved"
    );
}

// ============================================================
// Test 7: Corrupt JSON degrades gracefully
// ============================================================

/// Corrupt JSON returns default empty PaneLayoutV1 without panicking.
#[test]
fn corrupt_json_degrades_gracefully() {
    let path = temp_path("corrupt-graceful");
    std::fs::write(&path, b"{{{{not valid json at all}}}").unwrap();

    let result = load_panes(&path).expect("corrupted JSON must not return Err");
    assert_eq!(
        result,
        PaneLayoutV1::default(),
        "corrupt JSON → default empty layout"
    );
    assert!(result.tabs.is_empty(), "empty tabs on corruption");
    assert_eq!(result.schema_version, SCHEMA_VERSION);

    // DTO conversion of default layout yields empty vec
    let inputs = layout_v1_to_tab_inputs(&result);
    assert!(inputs.is_empty());

    let _ = std::fs::remove_file(&path);
}

/// Missing JSON file (file not found) returns Io error, not panic.
#[test]
fn missing_json_file_returns_io_error() {
    let path = temp_path("does-not-exist-abcdef");
    let _ = std::fs::remove_file(&path);

    let result = load_panes(&path);
    assert!(
        result.is_err(),
        "missing file must return Err (IoError), not Ok"
    );
}

// ============================================================
// Test 8: Multiple tabs round-trip
// ============================================================

/// Multiple tabs with mixed leaf/split trees all round-trip correctly.
#[test]
fn multiple_tabs_roundtrip() {
    let inputs = vec![
        TabSnapshotInput {
            id: "tab-a".to_string(),
            title: "Alpha".to_string(),
            last_focused_pane: Some("pane-alpha".to_string()),
            pane_tree: PaneTreeInput::Leaf {
                id: "pane-alpha".to_string(),
                cwd: Some(PathBuf::from("/alpha")),
            },
        },
        TabSnapshotInput {
            id: "tab-b".to_string(),
            title: "Beta".to_string(),
            last_focused_pane: None,
            pane_tree: PaneTreeInput::Split {
                id: "split-beta".to_string(),
                direction: SplitDirectionInput::Horizontal,
                ratio: 0.5,
                first: Box::new(PaneTreeInput::Leaf {
                    id: "pane-b1".to_string(),
                    cwd: None,
                }),
                second: Box::new(PaneTreeInput::Leaf {
                    id: "pane-b2".to_string(),
                    cwd: Some(PathBuf::from("/beta/two")),
                }),
            },
        },
        TabSnapshotInput {
            id: "tab-c".to_string(),
            title: "Gamma".to_string(),
            last_focused_pane: Some("pane-c".to_string()),
            pane_tree: PaneTreeInput::Leaf {
                id: "pane-c".to_string(),
                cwd: None,
            },
        },
    ];

    let layout = tab_container_to_layout_v1(&inputs);
    assert_eq!(layout.tabs.len(), 3, "3 tabs serialized");

    let path = temp_path("multiple-tabs");
    let _ = std::fs::remove_file(&path);
    save_panes(&path, &layout).expect("save multiple tabs");
    let loaded = load_panes(&path).expect("load multiple tabs");
    let restored = layout_v1_to_tab_inputs(&loaded);

    assert_eq!(restored, inputs, "multiple tabs JSON round-trip");
    assert_eq!(restored[0].title, "Alpha");
    assert_eq!(restored[1].title, "Beta");
    assert_eq!(restored[2].title, "Gamma");

    let _ = std::fs::remove_file(&path);
}

// ============================================================
// Test 9: snapshot_path format
// ============================================================

/// snapshot_path returns the expected ~/.moai/studio/panes-{id}.json path.
#[test]
fn snapshot_path_uses_home_dir() {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"));

    let path = snapshot_path("ws-deadbeef");
    assert_eq!(
        path,
        home.join(".moai")
            .join("studio")
            .join("panes-ws-deadbeef.json"),
        "snapshot_path format correct"
    );
}

// ============================================================
// Test 10: TabSnapshotV1 ↔ TabSnapshotInput type codec
// ============================================================

/// TabSnapshotV1 contains all fields from TabSnapshotInput after round-trip.
#[test]
fn tab_snapshot_v1_fields_match_input() {
    let input = TabSnapshotInput {
        id: "tab-field-check".to_string(),
        title: "Field Check".to_string(),
        last_focused_pane: Some("pane-fc".to_string()),
        pane_tree: PaneTreeInput::Leaf {
            id: "pane-fc".to_string(),
            cwd: Some(PathBuf::from("/fc")),
        },
    };

    let layout = tab_container_to_layout_v1(&[input]);
    let snap: &TabSnapshotV1 = &layout.tabs[0];

    assert_eq!(snap.id, "tab-field-check");
    assert_eq!(snap.title, "Field Check");
    assert_eq!(snap.last_focused_pane.as_deref(), Some("pane-fc"));

    match &snap.pane_tree {
        PaneTreeSnapshotV1::Leaf { id, cwd } => {
            assert_eq!(id, "pane-fc");
            assert_eq!(cwd.as_deref(), Some("/fc"));
        }
        _ => panic!("should be Leaf"),
    }
}
