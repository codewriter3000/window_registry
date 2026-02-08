use crate::{
    DesktopKey,
    Registry,
    RegistryError,
    RegistryEventQueue,
    SharedRegistry,
    SurfaceKey,
    WindowId,
    WindowGeometry,
    WindowUpdate,
    WorkspaceId,
    OutputId,
    weston_desktop_surface,
    weston_surface,
};

// exact function depends on libweston bindings
extern "C" {
    fn weston_desktop_surface_get_surface(ds: *mut weston_desktop_surface) -> *mut weston_surface;
}

pub fn on_new_desktop_surface_with_keys(
    dk: DesktopKey,
    sk: SurfaceKey,
    reg: &mut Registry,
) -> Result<WindowId, RegistryError> {
    let (id, _events) = reg.insert_window(dk, sk)?;
    Ok(id)
}

pub unsafe fn on_new_desktop_surface(
    ds: *mut weston_desktop_surface,
    reg: &mut Registry,
) -> Result<WindowId, RegistryError> {
    let dk = DesktopKey::from_ptr(ds);

    // you likely extract weston_surface* from the desktop surface via libweston-desktop API:
    let s: *mut weston_surface = weston_desktop_surface_get_surface(ds);
    let sk = SurfaceKey::from_ptr(s);

    let id = on_new_desktop_surface_with_keys(dk, sk, reg)?;
    eprintln!("New window: {:?}", id);
    Ok(id)
}

#[derive(Clone, Debug)]
pub struct WestonGlueContext {
    pub reg: SharedRegistry,
    pub queue: RegistryEventQueue,
}

impl WestonGlueContext {
    pub fn new(reg: SharedRegistry, queue: RegistryEventQueue) -> Self {
        Self { reg, queue }
    }
}

fn lookup_id_from_desktop(reg: &SharedRegistry, ds: *mut weston_desktop_surface) -> Result<WindowId, RegistryError> {
    let dk = unsafe { DesktopKey::from_ptr(ds) };
    reg.from_desktop(dk)
        .ok_or_else(|| RegistryError::InvalidWindowId(WindowId { index: u32::MAX, gen: std::num::NonZeroU32::new(1).unwrap() }))
}

pub unsafe fn weston_handle_new_desktop_surface(
    ds: *mut weston_desktop_surface,
    ctx: &WestonGlueContext,
) -> Result<WindowId, RegistryError> {
    let dk = DesktopKey::from_ptr(ds);
    let s: *mut weston_surface = weston_desktop_surface_get_surface(ds);
    let sk = SurfaceKey::from_ptr(s);
    ctx.reg.insert_window_queued(dk, sk, &ctx.queue)
}

pub unsafe fn weston_handle_map(
    ds: *mut weston_desktop_surface,
    ctx: &WestonGlueContext,
) -> Result<(), RegistryError> {
    let id = lookup_id_from_desktop(&ctx.reg, ds)?;
    ctx.reg.on_map_queued(id, &ctx.queue)
}

pub unsafe fn weston_handle_unmap(
    ds: *mut weston_desktop_surface,
    ctx: &WestonGlueContext,
) -> Result<(), RegistryError> {
    let id = lookup_id_from_desktop(&ctx.reg, ds)?;
    ctx.reg.on_unmap_queued(id, &ctx.queue)
}

pub unsafe fn weston_handle_destroy(
    ds: *mut weston_desktop_surface,
    ctx: &WestonGlueContext,
) -> Result<(), RegistryError> {
    let id = lookup_id_from_desktop(&ctx.reg, ds)?;
    ctx.reg.remove_window_queued(id, &ctx.queue)
}

pub unsafe fn weston_handle_configure(
    ds: *mut weston_desktop_surface,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    ctx: &WestonGlueContext,
) -> Result<(), RegistryError> {
    let id = lookup_id_from_desktop(&ctx.reg, ds)?;
    let mut update = WindowUpdate::default();
    update.geometry = Some(Some(WindowGeometry { x, y, width, height }));
    ctx.reg.update_window_queued(id, update, &ctx.queue)
}

pub unsafe fn weston_handle_commit(
    _ds: *mut weston_desktop_surface,
    _ctx: &WestonGlueContext,
) -> Result<(), RegistryError> {
    Ok(())
}

pub unsafe fn weston_handle_focus(
    ds: *mut weston_desktop_surface,
    focused: bool,
    ctx: &WestonGlueContext,
) -> Result<(), RegistryError> {
    let id = lookup_id_from_desktop(&ctx.reg, ds)?;
    let mut update = WindowUpdate::default();
    update.is_focused = Some(focused);
    ctx.reg.update_window_queued(id, update, &ctx.queue)
}

pub unsafe fn weston_handle_output(
    ds: *mut weston_desktop_surface,
    output_id: u32,
    workspace_id: u32,
    ctx: &WestonGlueContext,
) -> Result<(), RegistryError> {
    let id = lookup_id_from_desktop(&ctx.reg, ds)?;
    let mut update = WindowUpdate::default();
    update.output = Some(Some(OutputId(output_id)));
    update.workspace = Some(Some(WorkspaceId(workspace_id)));
    ctx.reg.update_window_queued(id, update, &ctx.queue)
}

pub unsafe fn weston_handle_parent(
    ds: *mut weston_desktop_surface,
    parent: *mut weston_desktop_surface,
    ctx: &WestonGlueContext,
) -> Result<(), RegistryError> {
    let id = lookup_id_from_desktop(&ctx.reg, ds)?;
    let parent_id = if parent.is_null() {
        None
    } else {
        Some(lookup_id_from_desktop(&ctx.reg, parent)?)
    };
    let mut update = WindowUpdate::default();
    update.parent_id = Some(parent_id);
    ctx.reg.update_window_queued(id, update, &ctx.queue)
}

