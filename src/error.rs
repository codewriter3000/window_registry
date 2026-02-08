use crate::{
    DesktopKey,
    OutputId,
    SurfaceKey,
    WindowGeometry,
    WindowId,
    WindowState,
    WorkspaceId,
};

#[derive(Debug, Clone)]
pub enum RegistryError {
    DesktopKeyAlreadyRegistered { dk: DesktopKey, existing: WindowId },
    SurfaceKeyAlreadyRegistered { sk: SurfaceKey, existing: WindowId },
    InvalidWindowId(WindowId),
    InvalidGeometry { id: WindowId, geometry: WindowGeometry },
    GeometryOverflow { id: WindowId, geometry: WindowGeometry },
    InvalidState { id: WindowId, state: WindowState },
    WorkspaceOutputMismatch { id: WindowId, workspace: Option<WorkspaceId>, output: Option<OutputId> },
    ParentIsSelf { id: WindowId },
    ParentNotFound { id: WindowId, parent: WindowId },
    ParentCycle { id: WindowId, parent: WindowId },
    ChildNotFound { id: WindowId, child: WindowId },
    ChildAlreadyHasParent { id: WindowId, child: WindowId, existing_parent: WindowId },
    ChildAlreadyPresent { id: WindowId, child: WindowId },
    StackIndexOutOfBounds { id: WindowId, index: i32, count: usize },
    EventQueueClosed,
    EventQueueTimeout,
}

