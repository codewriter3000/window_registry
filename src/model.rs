use std::{
    fmt::Debug,
};

use crate::{
    WindowId,
    DesktopKey,
    SurfaceKey,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LifecycleState {
    Created,   // seen but not mapped
    Mapped,    // visible / participating in layout
    Unmapped,  // previously mapped, now hidden
    Destroyed, // terminal (you may not need to store this if you remove records)
}

#[derive(Debug)]
pub struct WindowRecord {
    pub id: WindowId,
    pub dk: DesktopKey,
    pub sk: SurfaceKey,

    pub lifecycle: LifecycleState,

    // later:
    pub title: Option<String>,
    pub app_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: WindowId,
    pub dk: DesktopKey,
    pub sk: SurfaceKey,

    pub lifecycle: LifecycleState,

    pub title: Option<String>,
    pub app_id: Option<String>,
}

impl From<&WindowRecord> for WindowInfo {
    fn from(r: &WindowRecord) -> Self {
        Self {
            id: r.id,
            dk: r.dk,
            sk: r.sk,
            lifecycle: r.lifecycle,
            title: r.title.clone(),
            app_id: r.app_id.clone(),
        }
    }
}

