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

extern "C" {
    // example: actual name depends on libweston-desktop
    pub fn weston_desktop_surface_get_surface(
        surface: *mut weston_desktop_surface,
    ) -> *mut weston_surface;
}

