use crate::{Registry, DesktopKey, SurfaceKey, weston_desktop_surface, weston_surface};

// exact function depends on libweston bindings
extern "C" {
    fn weston_desktop_surface_get_surface(ds: *mut weston_desktop_surface) -> *mut weston_surface;
}

pub unsafe fn on_new_desktop_surface(ds: *mut weston_desktop_surface, reg: &mut Registry) {
    let dk = DesktopKey::from_ptr(ds);

    // you likely extract weston_surface* from the desktop surface via libweston-desktop API:
    let s: *mut weston_surface = weston_desktop_surface_get_surface(ds);
    let sk = SurfaceKey::from_ptr(s);

    let id = reg.insert_window(dk, sk, None);
    
    // log
    eprintln!("New window: {:?}", id);
}
