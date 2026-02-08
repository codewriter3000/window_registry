use crate::{
    DesktopKey,
    LifecycleState,
    OutputId,
    SurfaceKey,
    WindowGeometry,
    WindowId,
    WindowState,
    WorkspaceId,
};

#[derive(Debug, Clone, PartialEq)]
pub struct WindowChange<T> {
    pub old: T,
    pub new: T,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WindowChanges {
    pub lifecycle: Option<WindowChange<LifecycleState>>,
    pub geometry: Option<WindowChange<Option<WindowGeometry>>>,
    pub state: Option<WindowChange<WindowState>>,
    pub is_focused: Option<WindowChange<bool>>,
    pub workspace: Option<WindowChange<Option<WorkspaceId>>>,
    pub output: Option<WindowChange<Option<OutputId>>>,
    pub stack_index: Option<WindowChange<i32>>,
    pub parent_id: Option<WindowChange<Option<WindowId>>>,
    pub children: Option<WindowChange<Vec<WindowId>>>,
    pub title: Option<WindowChange<Option<String>>>,
    pub app_id: Option<WindowChange<Option<String>>>,
}

impl WindowChanges {
    pub fn is_empty(&self) -> bool {
        self.lifecycle.is_none()
            && self.geometry.is_none()
            && self.state.is_none()
            && self.is_focused.is_none()
            && self.workspace.is_none()
            && self.output.is_none()
            && self.stack_index.is_none()
            && self.parent_id.is_none()
            && self.children.is_none()
            && self.title.is_none()
            && self.app_id.is_none()
    }
}

#[derive(Debug, Clone)]
pub enum RegistryEvent {
    WindowCreated {
        id: WindowId,
        dk: DesktopKey,
        sk: SurfaceKey,
    },
    WindowChanged {
        id: WindowId,
        changes: WindowChanges,
    },
    WindowDestroyed {
        id: WindowId,
    },
}

