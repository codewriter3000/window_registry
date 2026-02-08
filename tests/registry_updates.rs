use window_registry::{
    OutputId,
    Registry,
    RegistryError,
    RegistryEvent,
    WindowChange,
    WindowGeometry,
    WindowState,
    WindowUpdate,
    WorkspaceId,
};

mod common;
use common::TestPtrs;

#[test]
fn update_window_rejects_negative_geometry() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let mut update = WindowUpdate::default();
    update.geometry = Some(Some(WindowGeometry { x: 0, y: 0, width: -1, height: 10 }));

    let err = reg.update_window(id, update).expect_err("negative width should fail");
    assert!(matches!(err, RegistryError::InvalidGeometry { id: err_id, .. } if err_id == id));
}

#[test]
fn update_window_rejects_geometry_overflow() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let mut update = WindowUpdate::default();
    update.geometry = Some(Some(WindowGeometry { x: i32::MAX, y: 0, width: 1, height: 1 }));

    let err = reg.update_window(id, update).expect_err("overflow should fail");
    assert!(matches!(err, RegistryError::GeometryOverflow { id: err_id, .. } if err_id == id));
}

#[test]
fn update_window_invalid_id_errors() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;
    reg.remove_window(id).expect("remove_window should succeed");

    let err = reg.update_window(id, WindowUpdate::default()).expect_err("stale id should fail");
    assert!(matches!(err, RegistryError::InvalidWindowId(stale) if stale == id));
}

#[test]
fn update_window_rejects_invalid_state_combinations() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let mut update = WindowUpdate::default();
    update.state = Some(WindowState { minimized: true, maximized: true, fullscreen: false });

    let err = reg.update_window(id, update).expect_err("invalid state should fail");
    assert!(matches!(err, RegistryError::InvalidState { id: err_id, .. } if err_id == id));
}

#[test]
fn update_window_requires_workspace_output_pairing() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let mut update = WindowUpdate::default();
    update.workspace = Some(Some(WorkspaceId(1)));

    let err = reg.update_window(id, update).expect_err("workspace without output should fail");
    assert!(matches!(err, RegistryError::WorkspaceOutputMismatch { id: err_id, .. } if err_id == id));
}

#[test]
fn update_window_focus_clears_previous_focus_first() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let id1 = reg.insert_window(dk1, sk1).expect("insert_window A should succeed").0;
    let id2 = reg.insert_window(dk2, sk2).expect("insert_window B should succeed").0;

    let mut update = WindowUpdate::default();
    update.is_focused = Some(true);
    reg.update_window(id1, update).expect("focus A should succeed");

    let mut update = WindowUpdate::default();
    update.is_focused = Some(true);
    let events = reg.update_window(id2, update).expect("focus B should succeed");

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == id1
                && changes.is_focused == Some(WindowChange { old: true, new: false })
    ));
    assert!(matches!(
        events[1],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == id2
                && changes.is_focused == Some(WindowChange { old: false, new: true })
    ));
}

#[test]
fn update_window_rejects_self_parent() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let mut update = WindowUpdate::default();
    update.parent_id = Some(Some(id));

    let err = reg.update_window(id, update).expect_err("self parent should fail");
    assert!(matches!(err, RegistryError::ParentIsSelf { id: err_id } if err_id == id));
}

#[test]
fn update_window_rejects_parent_cycles() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let id1 = reg.insert_window(dk1, sk1).expect("insert_window A should succeed").0;
    let id2 = reg.insert_window(dk2, sk2).expect("insert_window B should succeed").0;

    let mut update = WindowUpdate::default();
    update.parent_id = Some(Some(id1));
    reg.update_window(id2, update).expect("set parent should succeed");

    let mut update = WindowUpdate::default();
    update.parent_id = Some(Some(id2));
    let err = reg.update_window(id1, update).expect_err("cycle should fail");

    assert!(matches!(err, RegistryError::ParentCycle { id: err_id, parent } if err_id == id1 && parent == id2));
}

#[test]
fn update_window_parent_not_found_error() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let id = reg.insert_window(dk1, sk1).expect("insert_window should succeed").0;
    let missing_parent = reg.insert_window(dk2, sk2).expect("insert_window should succeed").0;
    reg.remove_window(missing_parent).expect("remove_window should succeed");

    let mut update = WindowUpdate::default();
    update.parent_id = Some(Some(missing_parent));

    let err = reg.update_window(id, update).expect_err("missing parent should fail");
    assert!(matches!(
        err,
        RegistryError::ParentNotFound { id: err_id, parent }
            if err_id == id && parent == missing_parent
    ));
}

#[test]
fn update_window_rejects_child_already_parented() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();
    let p3 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let (dk3, sk3) = unsafe { p3.keys() };
    let parent_a = reg.insert_window(dk1, sk1).expect("insert_window A should succeed").0;
    let parent_b = reg.insert_window(dk2, sk2).expect("insert_window B should succeed").0;
    let child = reg.insert_window(dk3, sk3).expect("insert_window child should succeed").0;

    let mut update = WindowUpdate::default();
    update.add_children.push(child);
    reg.update_window(parent_a, update).expect("add child should succeed");

    let mut update = WindowUpdate::default();
    update.add_children.push(child);
    let err = reg.update_window(parent_b, update).expect_err("child already parented should fail");
    assert!(matches!(
        err,
        RegistryError::ChildAlreadyHasParent { id, child: err_child, existing_parent }
            if id == parent_b && err_child == child && existing_parent == parent_a
    ));
}

#[test]
fn update_window_rejects_child_same_as_parent() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let mut update = WindowUpdate::default();
    update.add_children.push(id);

    let err = reg.update_window(id, update).expect_err("child id == parent id should fail");
    assert!(matches!(err, RegistryError::ParentIsSelf { id: err_id } if err_id == id));
}

#[test]
fn update_window_rejects_child_cycle_via_ancestor() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let parent = reg.insert_window(dk1, sk1).expect("insert parent").0;
    let child = reg.insert_window(dk2, sk2).expect("insert child").0;

    let mut update = WindowUpdate::default();
    update.parent_id = Some(Some(parent));
    reg.update_window(child, update).expect("set parent should succeed");

    let mut update = WindowUpdate::default();
    update.add_children.push(parent);
    let err = reg.update_window(child, update).expect_err("ancestor cycle should fail");

    assert!(matches!(
        err,
        RegistryError::ParentCycle { id: err_id, parent: err_parent }
            if err_id == parent && err_parent == child
    ));
}

#[test]
fn update_window_rejects_child_already_present() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let parent = reg.insert_window(dk1, sk1).expect("insert parent").0;
    let child = reg.insert_window(dk2, sk2).expect("insert child").0;

    let mut update = WindowUpdate::default();
    update.add_children.push(child);
    reg.update_window(parent, update).expect("add child should succeed");

    let mut update = WindowUpdate::default();
    update.add_children.push(child);
    let err = reg.update_window(parent, update).expect_err("child already present should fail");

    assert!(matches!(
        err,
        RegistryError::ChildAlreadyPresent { id, child: err_child }
            if id == parent && err_child == child
    ));
}

#[test]
fn update_window_rejects_missing_child_on_add() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let parent = reg.insert_window(dk1, sk1).expect("insert parent").0;
    let child = reg.insert_window(dk2, sk2).expect("insert child").0;
    reg.remove_window(child).expect("remove child should succeed");

    let mut update = WindowUpdate::default();
    update.add_children.push(child);
    let err = reg.update_window(parent, update).expect_err("missing child should fail");

    assert!(matches!(
        err,
        RegistryError::ChildNotFound { id, child: err_child }
            if id == parent && err_child == child
    ));
}

#[test]
fn update_window_rejects_duplicate_children() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let parent = reg.insert_window(dk1, sk1).expect("insert_window parent should succeed").0;
    let child = reg.insert_window(dk2, sk2).expect("insert_window child should succeed").0;

    let mut update = WindowUpdate::default();
    update.add_children.push(child);
    update.add_children.push(child);

    let err = reg.update_window(parent, update).expect_err("duplicate child should fail");
    assert!(matches!(
        err,
        RegistryError::ChildAlreadyPresent { id, child: err_child }
            if id == parent && err_child == child
    ));
}

#[test]
fn update_window_reorders_stack_indices() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();
    let p3 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let (dk3, sk3) = unsafe { p3.keys() };
    let id1 = reg.insert_window(dk1, sk1).expect("insert A").0;
    let id2 = reg.insert_window(dk2, sk2).expect("insert B").0;
    let id3 = reg.insert_window(dk3, sk3).expect("insert C").0;

    let mut update = WindowUpdate::default();
    update.stack_index = Some(0);
    let events = reg.update_window(id3, update).expect("stack reorder should succeed");

    assert_eq!(events.len(), 3);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == id1
                && changes.stack_index == Some(WindowChange { old: 0, new: 1 })
    ));
    assert!(matches!(
        events[1],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == id2
                && changes.stack_index == Some(WindowChange { old: 1, new: 2 })
    ));
    assert!(matches!(
        events[2],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == id3
                && changes.stack_index == Some(WindowChange { old: 2, new: 0 })
    ));

    let snap1 = reg.snapshot(id1).expect("snapshot A");
    let snap2 = reg.snapshot(id2).expect("snapshot B");
    let snap3 = reg.snapshot(id3).expect("snapshot C");
    assert_eq!(snap1.stack_index, 1);
    assert_eq!(snap2.stack_index, 2);
    assert_eq!(snap3.stack_index, 0);
}

#[test]
fn update_window_rejects_stack_index_out_of_bounds() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let id1 = reg.insert_window(dk1, sk1).expect("insert A").0;
    reg.insert_window(dk2, sk2).expect("insert B");

    let mut update = WindowUpdate::default();
    update.stack_index = Some(2);
    let err = reg.update_window(id1, update).expect_err("out of bounds should fail");

    assert!(matches!(
        err,
        RegistryError::StackIndexOutOfBounds { id, index, count } if id == id1 && index == 2 && count == 2
    ));
}

#[test]
fn update_window_parent_change_updates_children() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let parent = reg.insert_window(dk1, sk1).expect("insert parent").0;
    let child = reg.insert_window(dk2, sk2).expect("insert child").0;

    let mut update = WindowUpdate::default();
    update.parent_id = Some(Some(parent));
    let events = reg.update_window(child, update).expect("set parent should succeed");

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == parent
                && changes.children.as_ref().map(|c| c.new.as_slice()) == Some(&[child])
    ));
    assert!(matches!(
        events[1],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == child
                && changes.parent_id == Some(WindowChange { old: None, new: Some(parent) })
    ));

    let parent_snap = reg.snapshot(parent).expect("parent snapshot");
    let child_snap = reg.snapshot(child).expect("child snapshot");
    assert_eq!(parent_snap.children, vec![child]);
    assert_eq!(child_snap.parent_id, Some(parent));
}

#[test]
fn update_window_accepts_existing_parent_without_changes() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let parent = reg.insert_window(dk1, sk1).expect("insert parent").0;
    let child = reg.insert_window(dk2, sk2).expect("insert child").0;

    let mut update = WindowUpdate::default();
    update.parent_id = Some(Some(parent));
    reg.update_window(child, update).expect("set parent should succeed");

    let mut update = WindowUpdate::default();
    update.parent_id = Some(Some(parent));
    let events = reg.update_window(child, update).expect("reassert parent should succeed");
    assert!(events.is_empty(), "no changes expected when parent is unchanged");
}

#[test]
fn update_window_sets_workspace_and_output_together() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let mut update = WindowUpdate::default();
    update.workspace = Some(Some(WorkspaceId(1)));
    update.output = Some(Some(OutputId(2)));
    let events = reg.update_window(id, update).expect("workspace/output set should succeed");

    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id: ev_id, ref changes }
            if ev_id == id
                && changes.workspace == Some(WindowChange { old: None, new: Some(WorkspaceId(1)) })
                && changes.output == Some(WindowChange { old: None, new: Some(OutputId(2)) })
    ));
}

#[test]
fn update_window_rejects_missing_child_on_remove() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let parent = reg.insert_window(dk1, sk1).expect("insert parent").0;
    let child = reg.insert_window(dk2, sk2).expect("insert child").0;
    reg.remove_window(child).expect("remove child");

    let mut update = WindowUpdate::default();
    update.remove_children.push(child);
    let err = reg.update_window(parent, update).expect_err("missing child should fail");

    assert!(matches!(
        err,
        RegistryError::ChildNotFound { id, child: err_child }
            if id == parent && err_child == child
    ));
}

#[test]
fn update_window_reorders_stack_indices_forward() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();
    let p3 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let (dk3, sk3) = unsafe { p3.keys() };
    let id1 = reg.insert_window(dk1, sk1).expect("insert A").0;
    let id2 = reg.insert_window(dk2, sk2).expect("insert B").0;
    let id3 = reg.insert_window(dk3, sk3).expect("insert C").0;

    let mut update = WindowUpdate::default();
    update.stack_index = Some(2);
    let events = reg.update_window(id1, update).expect("stack reorder should succeed");

    assert_eq!(events.len(), 3);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == id2
                && changes.stack_index == Some(WindowChange { old: 1, new: 0 })
    ));
    assert!(matches!(
        events[1],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == id3
                && changes.stack_index == Some(WindowChange { old: 2, new: 1 })
    ));
    assert!(matches!(
        events[2],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == id1
                && changes.stack_index == Some(WindowChange { old: 0, new: 2 })
    ));

    let snap1 = reg.snapshot(id1).expect("snapshot A");
    let snap2 = reg.snapshot(id2).expect("snapshot B");
    let snap3 = reg.snapshot(id3).expect("snapshot C");
    assert_eq!(snap1.stack_index, 2);
    assert_eq!(snap2.stack_index, 0);
    assert_eq!(snap3.stack_index, 1);
}

#[test]
fn update_window_parent_change_updates_old_and_new_parent() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();
    let p3 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let (dk3, sk3) = unsafe { p3.keys() };
    let parent_a = reg.insert_window(dk1, sk1).expect("insert parent A").0;
    let parent_b = reg.insert_window(dk2, sk2).expect("insert parent B").0;
    let child = reg.insert_window(dk3, sk3).expect("insert child").0;

    let mut update = WindowUpdate::default();
    update.parent_id = Some(Some(parent_a));
    reg.update_window(child, update).expect("set parent A should succeed");

    let mut update = WindowUpdate::default();
    update.parent_id = Some(Some(parent_b));
    let events = reg.update_window(child, update).expect("switch parent should succeed");

    assert_eq!(events.len(), 3);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == parent_a
                && changes.children.as_ref().map(|c| c.new.as_slice()) == Some(&[])
    ));
    assert!(matches!(
        events[1],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == parent_b
                && changes.children.as_ref().map(|c| c.new.as_slice()) == Some(&[child])
    ));
    assert!(matches!(
        events[2],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == child
                && changes.parent_id == Some(WindowChange { old: Some(parent_a), new: Some(parent_b) })
    ));

    let snap_a = reg.snapshot(parent_a).expect("parent A snapshot");
    let snap_b = reg.snapshot(parent_b).expect("parent B snapshot");
    let snap_child = reg.snapshot(child).expect("child snapshot");
    assert!(snap_a.children.is_empty());
    assert_eq!(snap_b.children, vec![child]);
    assert_eq!(snap_child.parent_id, Some(parent_b));
}

#[test]
fn update_window_remove_child_clears_parent() {
    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();

    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let parent = reg.insert_window(dk1, sk1).expect("insert parent").0;
    let child = reg.insert_window(dk2, sk2).expect("insert child").0;

    let mut update = WindowUpdate::default();
    update.add_children.push(child);
    reg.update_window(parent, update).expect("add child should succeed");

    let mut update = WindowUpdate::default();
    update.remove_children.push(child);
    let events = reg.update_window(parent, update).expect("remove child should succeed");

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == child
                && changes.parent_id == Some(WindowChange { old: Some(parent), new: None })
    ));
    assert!(matches!(
        events[1],
        RegistryEvent::WindowChanged { id, ref changes }
            if id == parent
                && changes.children.as_ref().map(|c| c.new.as_slice()) == Some(&[])
    ));

    let parent_snap = reg.snapshot(parent).expect("parent snapshot");
    let child_snap = reg.snapshot(child).expect("child snapshot");
    assert!(parent_snap.children.is_empty());
    assert_eq!(child_snap.parent_id, None);
}

#[test]
fn update_window_geometry_change_emits_event() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let new_geom = WindowGeometry { x: 10, y: 20, width: 300, height: 400 };
    let mut update = WindowUpdate::default();
    update.geometry = Some(Some(new_geom));
    let events = reg.update_window(id, update).expect("geometry update should succeed");

    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id: ev_id, ref changes }
            if ev_id == id
                && changes.geometry == Some(WindowChange { old: None, new: Some(new_geom) })
    ));
}

#[test]
fn update_window_state_change_emits_event() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let new_state = WindowState { minimized: true, maximized: false, fullscreen: false };
    let mut update = WindowUpdate::default();
    update.state = Some(new_state);
    let events = reg.update_window(id, update).expect("state update should succeed");

    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id: ev_id, ref changes }
            if ev_id == id
                && changes.state == Some(WindowChange { old: WindowState::default(), new: new_state })
    ));
}

#[test]
fn update_window_title_change_emits_event() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let mut update = WindowUpdate::default();
    update.title = Some(Some("New Title".to_string()));
    let events = reg.update_window(id, update).expect("title update should succeed");

    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id: ev_id, ref changes }
            if ev_id == id
                && changes.title == Some(WindowChange { old: None, new: Some("New Title".to_string()) })
    ));
}

#[test]
fn update_window_app_id_change_emits_event() {
    let mut reg = Registry::new();
    let p = TestPtrs::new();

    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let mut update = WindowUpdate::default();
    update.app_id = Some(Some("com.example.app".to_string()));
    let events = reg.update_window(id, update).expect("app_id update should succeed");

    assert_eq!(events.len(), 1);
    assert!(matches!(
        events[0],
        RegistryEvent::WindowChanged { id: ev_id, ref changes }
            if ev_id == id
                && changes.app_id == Some(WindowChange { old: None, new: Some("com.example.app".to_string()) })
    ));
}
