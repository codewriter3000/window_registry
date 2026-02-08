use std::{
    ptr::NonNull,
    fmt::Debug,
};

use crate::{
    WindowId,
    DesktopKey,
    SurfaceKey,
    weston_view,
};

#[derive(Debug)]
pub struct WindowRecord {
    pub id: WindowId,
    pub dk: DesktopKey,
    pub sk: SurfaceKey,
    pub view: Option<NonNull<weston_view>>,
    // desktop ptr, surface ptr, view ptr, title/app_id later...
}
