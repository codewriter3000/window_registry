use window_registry::{
    FakeWeston,
    Registry,
    RegistryAdapter,
    RegistryEventQueue,
    RegistryError,
    SharedRegistry,
    WestonAdapter,
    WestonEvent,
    WindowChange,
    WindowGeometry,
    WindowState,
    WorkspaceId,
    OutputId,
};

mod common;
use common::TestPtrs;

#[test]
fn weston_adapter_map_unmap_destroy_flow() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::unbounded();
    let adapter = RegistryAdapter::new(reg.clone(), queue.clone());
    let receiver = queue.subscribe();

    let mut fake = FakeWeston::new(adapter);

    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };

    fake.push(WestonEvent::NewSurface { dk, sk });
    fake.run().expect("run should succeed");

    let id = reg.from_desktop(dk).expect("id should be assigned");
    fake.push(WestonEvent::Map { id });
    fake.push(WestonEvent::Unmap { id });
    fake.push(WestonEvent::Destroy { id });
    fake.run().expect("run should succeed");

    let created = receiver.recv().expect("created event");
    let mapped = receiver.recv().expect("mapped event");
    let unmapped = receiver.recv().expect("unmapped event");
    let destroyed = receiver.recv().expect("destroyed event");

    assert!(matches!(created, window_registry::RegistryEvent::WindowCreated { id: ev_id, .. } if ev_id == id));
    assert!(matches!(mapped, window_registry::RegistryEvent::WindowChanged { id: ev_id, .. } if ev_id == id));
    assert!(matches!(unmapped, window_registry::RegistryEvent::WindowChanged { id: ev_id, .. } if ev_id == id));
    assert!(matches!(destroyed, window_registry::RegistryEvent::WindowDestroyed { id: ev_id } if ev_id == id));
}

#[test]
fn weston_adapter_updates_fields_from_events() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::unbounded();
    let adapter = RegistryAdapter::new(reg.clone(), queue.clone());
    let receiver = queue.subscribe();

    let mut fake = FakeWeston::new(adapter);

    let p_parent = TestPtrs::new();
    let (dk_parent, sk_parent) = unsafe { p_parent.keys() };
    let p_child = TestPtrs::new();
    let (dk_child, sk_child) = unsafe { p_child.keys() };

    fake.push(WestonEvent::NewSurface { dk: dk_parent, sk: sk_parent });
    fake.push(WestonEvent::NewSurface { dk: dk_child, sk: sk_child });
    fake.run().expect("run should succeed");

    let parent_id = reg.from_desktop(dk_parent).expect("parent id");
    let child_id = reg.from_desktop(dk_child).expect("child id");

    let geom = WindowGeometry { x: 10, y: 20, width: 300, height: 400 };
    fake.push(WestonEvent::Configure { id: child_id, geom });
    fake.push(WestonEvent::Commit { id: child_id });
    fake.push(WestonEvent::Focus { id: child_id, focused: true });
    fake.push(WestonEvent::Output { id: child_id, output: OutputId(2), workspace: WorkspaceId(1) });
    fake.push(WestonEvent::Parent { id: child_id, parent: Some(parent_id) });
    fake.push(WestonEvent::Title { id: child_id, title: Some("Child".to_string()) });
    fake.push(WestonEvent::AppId { id: child_id, app_id: Some("com.example.child".to_string()) });
    fake.run().expect("run should succeed");

    let mut events = Vec::new();
    while events.len() < 9 {
        let event = receiver.recv().expect("recv event");
        events.push(event);
    }

    assert!(events.iter().any(|e| matches!(
        e,
        window_registry::RegistryEvent::WindowChanged { id, changes }
            if *id == child_id
                && changes.geometry == Some(WindowChange { old: None, new: Some(geom) })
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        window_registry::RegistryEvent::WindowChanged { id, changes }
            if *id == child_id
                && changes.is_focused == Some(WindowChange { old: false, new: true })
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        window_registry::RegistryEvent::WindowChanged { id, changes }
            if *id == child_id
                && changes.workspace == Some(WindowChange { old: None, new: Some(WorkspaceId(1)) })
                && changes.output == Some(WindowChange { old: None, new: Some(OutputId(2)) })
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        window_registry::RegistryEvent::WindowChanged { id, changes }
            if *id == parent_id
                && changes.children.as_ref().map(|c| c.new.as_slice()) == Some(&[child_id])
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        window_registry::RegistryEvent::WindowChanged { id, changes }
            if *id == child_id
                && changes.parent_id == Some(WindowChange { old: None, new: Some(parent_id) })
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        window_registry::RegistryEvent::WindowChanged { id, changes }
            if *id == child_id
                && changes.title == Some(WindowChange { old: None, new: Some("Child".to_string()) })
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        window_registry::RegistryEvent::WindowChanged { id, changes }
            if *id == child_id
                && changes.app_id == Some(WindowChange { old: None, new: Some("com.example.child".to_string()) })
    )));

    let child_snapshot = reg.snapshot(child_id).expect("child snapshot");
    assert_eq!(child_snapshot.geometry, Some(geom));
    assert_eq!(child_snapshot.is_focused, true);
    assert_eq!(child_snapshot.workspace, Some(WorkspaceId(1)));
    assert_eq!(child_snapshot.output, Some(OutputId(2)));
    assert_eq!(child_snapshot.parent_id, Some(parent_id));
    assert_eq!(child_snapshot.title.as_deref(), Some("Child"));
    assert_eq!(child_snapshot.app_id.as_deref(), Some("com.example.child"));
    assert_eq!(child_snapshot.state, WindowState::default());
}

#[test]
fn weston_adapter_commit_is_noop() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::unbounded();
    let mut adapter = RegistryAdapter::new(reg.clone(), queue.clone());
    let receiver = queue.subscribe();

    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };

    adapter
        .handle_event(WestonEvent::NewSurface { dk, sk })
        .expect("new surface should succeed");

    let id = reg.from_desktop(dk).expect("id should be assigned");
    let _created = receiver.recv().expect("created event");

    adapter
        .handle_event(WestonEvent::Commit { id })
        .expect("commit should succeed");

    let err = receiver
        .recv_timeout(std::time::Duration::from_millis(10))
        .expect_err("commit should not emit events");
    assert!(matches!(err, RegistryError::EventQueueTimeout));
}
