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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Default for WindowGeometry {
    fn default() -> Self {
        Self { x: 0, y: 0, width: 0, height: 0 }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct WindowState {
    pub minimized: bool,
    pub maximized: bool,
    pub fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self { minimized: false, maximized: false, fullscreen: false }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct WorkspaceId(pub u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct OutputId(pub u32);

#[derive(Debug)]
pub struct WindowRecord {
    pub id: WindowId,
    pub dk: DesktopKey,
    pub sk: SurfaceKey,

    pub lifecycle: LifecycleState,

    pub geometry: Option<WindowGeometry>,
    pub state: WindowState,
    pub is_focused: bool,
    pub workspace: Option<WorkspaceId>,
    pub output: Option<OutputId>,
    pub stack_index: i32,
    pub parent_id: Option<WindowId>,
    pub children: Vec<WindowId>,

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

    pub geometry: Option<WindowGeometry>,
    pub state: WindowState,
    pub is_focused: bool,
    pub workspace: Option<WorkspaceId>,
    pub output: Option<OutputId>,
    pub stack_index: i32,
    pub parent_id: Option<WindowId>,
    pub children: Vec<WindowId>,

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
            geometry: r.geometry,
            state: r.state,
            is_focused: r.is_focused,
            workspace: r.workspace,
            output: r.output,
            stack_index: r.stack_index,
            parent_id: r.parent_id,
            children: r.children.clone(),
            title: r.title.clone(),
            app_id: r.app_id.clone(),
        }
    }
}

