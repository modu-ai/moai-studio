//! D-5: Workspace color tag CRUD tests.

use moai_store::{ColorTag, NewWorkspace, Store, WorkspaceStoreExt};

fn new_ws(name: &str) -> NewWorkspace {
    NewWorkspace {
        name: name.to_string(),
        project_path: "/tmp".to_string(),
        spec_id: None,
        color_tag: None,
    }
}

#[test]
fn insert_and_read_color_tag() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let row = dao
        .insert(&NewWorkspace {
            name: "red-ws".into(),
            project_path: "/tmp/r".into(),
            spec_id: None,
            color_tag: Some(ColorTag::Red),
        })
        .unwrap();
    assert_eq!(row.color_tag, Some(ColorTag::Red));

    let reloaded = dao.get(row.id).unwrap().unwrap();
    assert_eq!(reloaded.color_tag, Some(ColorTag::Red));
}

#[test]
fn insert_without_color_tag() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let row = dao.insert(&new_ws("plain")).unwrap();
    assert_eq!(row.color_tag, None);
}

#[test]
fn set_color_tag() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let row = dao.insert(&new_ws("ws")).unwrap();
    assert_eq!(row.color_tag, None);

    dao.set_color_tag(row.id, Some(ColorTag::Blue)).unwrap();
    let reloaded = dao.get(row.id).unwrap().unwrap();
    assert_eq!(reloaded.color_tag, Some(ColorTag::Blue));
}

#[test]
fn clear_color_tag() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let row = dao
        .insert(&NewWorkspace {
            name: "ws".into(),
            project_path: "/tmp".into(),
            spec_id: None,
            color_tag: Some(ColorTag::Green),
        })
        .unwrap();

    dao.set_color_tag(row.id, None).unwrap();
    let reloaded = dao.get(row.id).unwrap().unwrap();
    assert_eq!(reloaded.color_tag, None);
}

#[test]
fn all_color_tags_roundtrip() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();

    for (i, ct) in ColorTag::ALL.iter().enumerate() {
        let row = dao
            .insert(&NewWorkspace {
                name: format!("ws-{i}"),
                project_path: format!("/tmp/{i}"),
                spec_id: None,
                color_tag: Some(*ct),
            })
            .unwrap();
        assert_eq!(
            row.color_tag,
            Some(*ct),
            "roundtrip failed for {:?}",
            ct
        );
    }
    assert_eq!(dao.list().unwrap().len(), 8);
}

#[test]
fn color_tag_str_roundtrip() {
    for ct in ColorTag::ALL {
        let s: &str = ct.as_str();
        let parsed: ColorTag = s.parse().unwrap();
        assert_eq!(ct, parsed);
    }
}

#[test]
fn color_tag_invalid_string() {
    let result = "magenta".parse::<ColorTag>();
    assert!(result.is_err());
}

#[test]
fn set_color_tag_not_found() {
    let store = Store::open_in_memory().unwrap();
    let dao = store.workspaces();
    let result = dao.set_color_tag(9999, Some(ColorTag::Red));
    assert!(result.is_err());
}
