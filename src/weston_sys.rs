#[repr(C)]
pub struct weston_surface {
    _private: [u8; 0],
}

#[repr(C)]
pub struct weston_view {
    _private: [u8; 0],
}

#[repr(C)]
pub struct weston_desktop_surface {
    _private: [u8; 0],
}

#[repr(C)]
pub struct weston_output {
    _private: [u8; 0],
}

#[repr(C)]
pub struct weston_desktop_surface_listener {
    pub map: Option<extern "C" fn(*mut weston_desktop_surface, *mut std::ffi::c_void)>,
    pub unmap: Option<extern "C" fn(*mut weston_desktop_surface, *mut std::ffi::c_void)>,
    pub destroy: Option<extern "C" fn(*mut weston_desktop_surface, *mut std::ffi::c_void)>,
    pub configure: Option<extern "C" fn(*mut weston_desktop_surface, i32, i32, i32, i32, *mut std::ffi::c_void)>,
    pub commit: Option<extern "C" fn(*mut weston_desktop_surface, *mut std::ffi::c_void)>,
}

#[repr(C)]
pub struct weston_compositor_focus_listener {
    pub focus: Option<extern "C" fn(*mut weston_desktop_surface, bool, *mut std::ffi::c_void)>,
}

#[repr(C)]
pub struct weston_compositor_output_listener {
    pub output: Option<extern "C" fn(*mut weston_desktop_surface, u32, u32, *mut std::ffi::c_void)>,
}

extern "C" {
    // example: actual name depends on libweston-desktop
    pub fn weston_desktop_surface_get_surface(
        surface: *mut weston_desktop_surface,
    ) -> *mut weston_surface;

    pub fn weston_desktop_surface_add_listener(
        surface: *mut weston_desktop_surface,
        listener: *mut weston_desktop_surface_listener,
        data: *mut std::ffi::c_void,
    );

    pub fn weston_compositor_add_focus_listener(
        listener: *mut weston_compositor_focus_listener,
        data: *mut std::ffi::c_void,
    );

    pub fn weston_compositor_add_output_listener(
        listener: *mut weston_compositor_output_listener,
        data: *mut std::ffi::c_void,
    );
}

