use std::ptr;

use window_registry::{
    on_new_desktop_surface,
    DesktopKey,
    Registry,
    RegistryEventQueue,
    RegistryEvent,
    RegistryError,
    SharedRegistry,
    SurfaceKey,
    WestonGlueContext,
    weston_handle_commit,
    weston_handle_configure,
    weston_handle_destroy,
    weston_handle_focus,
    weston_handle_map,
    weston_handle_new_desktop_surface,
    weston_handle_output,
    weston_handle_parent,
    weston_handle_unmap,
    WindowChange,
    WindowGeometry,
    WorkspaceId,
    OutputId,
    weston_desktop_surface,
    weston_surface,
};

static mut NEXT_SURFACE: *mut weston_surface = ptr::null_mut();

#[no_mangle]
pub extern "C" fn weston_desktop_surface_get_surface(
    _ds: *mut weston_desktop_surface,
) -> *mut weston_surface {
    unsafe { NEXT_SURFACE }
}

#[test]
fn on_new_desktop_surface_inserts_window() {
    let mut reg = Registry::new();

    let ds = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let s = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    unsafe {
        NEXT_SURFACE = s;
        let _ = on_new_desktop_surface(ds, &mut reg);
    }

    let dk = unsafe { DesktopKey::from_ptr(ds) };
    let sk = unsafe { SurfaceKey::from_ptr(s) };

    let id = reg.from_desktop(dk).expect("desktop key should be registered");
    assert_eq!(reg.from_surface(sk), Some(id));

    let snap = reg.snapshot(id).expect("snapshot should exist");
    assert_eq!(snap.dk, dk);
    assert_eq!(snap.sk, sk);

    unsafe {
        drop(Box::from_raw(ds));
        drop(Box::from_raw(s));
    }
}

#[test]
fn weston_glue_map_unmap_destroy_flow() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();
    let ctx = WestonGlueContext::new(reg.clone(), queue.clone());

    let ds = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let s = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    unsafe {
        NEXT_SURFACE = s;
        weston_handle_new_desktop_surface(ds, &ctx).expect("new surface should succeed");
        let id = reg.from_desktop(DesktopKey::from_ptr(ds)).expect("id should be assigned");

        weston_handle_map(ds, &ctx).expect("map should succeed");
        weston_handle_unmap(ds, &ctx).expect("unmap should succeed");
        weston_handle_destroy(ds, &ctx).expect("destroy should succeed");

        let created = receiver.recv().expect("created event");
        let mapped = receiver.recv().expect("mapped event");
        let unmapped = receiver.recv().expect("unmapped event");
        let destroyed = receiver.recv().expect("destroyed event");

        assert!(matches!(created, RegistryEvent::WindowCreated { id: ev_id, .. } if ev_id == id));
        assert!(matches!(mapped, RegistryEvent::WindowChanged { id: ev_id, .. } if ev_id == id));
        assert!(matches!(unmapped, RegistryEvent::WindowChanged { id: ev_id, .. } if ev_id == id));
        assert!(matches!(destroyed, RegistryEvent::WindowDestroyed { id: ev_id } if ev_id == id));

        drop(Box::from_raw(ds));
        drop(Box::from_raw(s));
    }
}

#[test]
fn weston_glue_configure_commit_focus_output_parent_mapping() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();
    let ctx = WestonGlueContext::new(reg.clone(), queue.clone());

    let parent_ds = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let parent_s = Box::into_raw(Box::new(0u8)) as *mut weston_surface;
    let child_ds = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let child_s = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    unsafe {
        NEXT_SURFACE = parent_s;
        weston_handle_new_desktop_surface(parent_ds, &ctx).expect("parent insert");
        NEXT_SURFACE = child_s;
        weston_handle_new_desktop_surface(child_ds, &ctx).expect("child insert");

        let parent_id = reg.from_desktop(DesktopKey::from_ptr(parent_ds)).expect("parent id");
        let child_id = reg.from_desktop(DesktopKey::from_ptr(child_ds)).expect("child id");

        let geom = WindowGeometry { x: 5, y: 6, width: 100, height: 200 };
        weston_handle_configure(child_ds, geom.x, geom.y, geom.width, geom.height, &ctx)
            .expect("configure");
        weston_handle_commit(child_ds, &ctx).expect("commit");
        weston_handle_focus(child_ds, true, &ctx).expect("focus");
        weston_handle_output(child_ds, 2, 1, &ctx).expect("output");
        weston_handle_parent(child_ds, parent_ds, &ctx).expect("parent");

        let mut events = Vec::new();
        let start = std::time::Instant::now();
        while events.len() < 5 && start.elapsed() < std::time::Duration::from_millis(200) {
            let event = receiver
                .recv_timeout(std::time::Duration::from_millis(20))
                .expect("recv event");
            if matches!(event, RegistryEvent::WindowCreated { .. }) {
                continue;
            }
            events.push(event);
        }

        assert_eq!(events.len(), 5, "expected 5 non-created events");

        assert!(events.iter().any(|e| matches!(
            e,
            RegistryEvent::WindowChanged { id, changes }
                if *id == child_id
                    && changes.geometry == Some(WindowChange { old: None, new: Some(geom) })
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            RegistryEvent::WindowChanged { id, changes }
                if *id == child_id
                    && changes.is_focused == Some(WindowChange { old: false, new: true })
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            RegistryEvent::WindowChanged { id, changes }
                if *id == child_id
                    && changes.workspace == Some(WindowChange { old: None, new: Some(WorkspaceId(1)) })
                    && changes.output == Some(WindowChange { old: None, new: Some(OutputId(2)) })
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            RegistryEvent::WindowChanged { id, changes }
                if *id == child_id
                    && changes.parent_id == Some(WindowChange { old: None, new: Some(parent_id) })
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            RegistryEvent::WindowChanged { id, changes }
                if *id == parent_id
                    && changes.children.as_ref().map(|c| c.new.as_slice()) == Some(&[child_id])
        )));

        let child_snap = reg.snapshot(child_id).expect("child snapshot");
        assert_eq!(child_snap.geometry, Some(geom));
        assert_eq!(child_snap.is_focused, true);
        assert_eq!(child_snap.workspace, Some(WorkspaceId(1)));
        assert_eq!(child_snap.output, Some(OutputId(2)));
        assert_eq!(child_snap.parent_id, Some(parent_id));

        drop(Box::from_raw(parent_ds));
        drop(Box::from_raw(parent_s));
        drop(Box::from_raw(child_ds));
        drop(Box::from_raw(child_s));
    }
}

#[test]
fn weston_glue_handlers_error_on_unknown_desktop() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::unbounded();
    let ctx = WestonGlueContext::new(reg, queue);

    let ds = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;

    unsafe {
        assert!(matches!(
            weston_handle_map(ds, &ctx),
            Err(RegistryError::InvalidWindowId(_))
        ));
        assert!(matches!(
            weston_handle_unmap(ds, &ctx),
            Err(RegistryError::InvalidWindowId(_))
        ));
        assert!(matches!(
            weston_handle_destroy(ds, &ctx),
            Err(RegistryError::InvalidWindowId(_))
        ));
        assert!(matches!(
            weston_handle_configure(ds, 0, 0, 10, 10, &ctx),
            Err(RegistryError::InvalidWindowId(_))
        ));
        assert!(matches!(
            weston_handle_focus(ds, true, &ctx),
            Err(RegistryError::InvalidWindowId(_))
        ));
        assert!(matches!(
            weston_handle_output(ds, 1, 1, &ctx),
            Err(RegistryError::InvalidWindowId(_))
        ));
        assert!(matches!(
            weston_handle_parent(ds, std::ptr::null_mut(), &ctx),
            Err(RegistryError::InvalidWindowId(_))
        ));

        drop(Box::from_raw(ds));
    }
}

#[test]
fn weston_glue_parent_null_and_missing_parent() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();
    let ctx = WestonGlueContext::new(reg.clone(), queue.clone());

    let parent_ds = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let parent_s = Box::into_raw(Box::new(0u8)) as *mut weston_surface;
    let child_ds = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
    let child_s = Box::into_raw(Box::new(0u8)) as *mut weston_surface;

    unsafe {
        NEXT_SURFACE = parent_s;
        weston_handle_new_desktop_surface(parent_ds, &ctx).expect("parent insert");
        NEXT_SURFACE = child_s;
        weston_handle_new_desktop_surface(child_ds, &ctx).expect("child insert");

        let parent_id = reg.from_desktop(DesktopKey::from_ptr(parent_ds)).expect("parent id");
        let child_id = reg.from_desktop(DesktopKey::from_ptr(child_ds)).expect("child id");

        weston_handle_parent(child_ds, parent_ds, &ctx).expect("set parent");
        let _ = receiver.recv().expect("created event");
        let _ = receiver.recv().expect("created event");
        let _ = receiver.recv().expect("parent update");
        let _ = receiver.recv().expect("child update");

        weston_handle_parent(child_ds, std::ptr::null_mut(), &ctx).expect("clear parent");
        let parent_event = receiver.recv().expect("parent clear event");
        let child_event = receiver.recv().expect("child clear event");
        assert!(matches!(
            parent_event,
            RegistryEvent::WindowChanged { id, changes }
                if id == parent_id
                    && changes.children.as_ref().map(|c| c.new.as_slice()) == Some(&[])
        ));
        assert!(matches!(
            child_event,
            RegistryEvent::WindowChanged { id, changes }
                if id == child_id
                    && changes.parent_id == Some(WindowChange { old: Some(parent_id), new: None })
        ));

        let missing_parent = Box::into_raw(Box::new(0u8)) as *mut weston_desktop_surface;
        assert!(matches!(
            weston_handle_parent(child_ds, missing_parent, &ctx),
            Err(RegistryError::InvalidWindowId(_))
        ));

        drop(Box::from_raw(missing_parent));
        drop(Box::from_raw(parent_ds));
        drop(Box::from_raw(parent_s));
        drop(Box::from_raw(child_ds));
        drop(Box::from_raw(child_s));
    }
}
