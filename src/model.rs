use std::{
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

    pub title: Option<String>,
    pub app_id: Option<String>,

    // desktop ptr, surface ptr, view ptr later...
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: WindowId,
    pub dk: DesktopKey,
    pub sk: SurfaceKey,
    pub title: Option<String>,
    pub app_id: Option<String>,
    // later: lifecycle, rect, workspace, output, state, etc.
}

impl From<&WindowRecord> for WindowInfo {
    fn from(r: &WindowRecord) -> Self {
        Self {
            id: r.id,
            dk: r.dk,
            sk: r.sk,
            title: r.title.clone(),
            app_id: r.app_id.clone(),
        }
    }
}

