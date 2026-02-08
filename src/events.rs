use crate::{WindowId, DesktopKey, SurfaceKey, LifecycleState};

#[derive(Debug, Clone)]
pub enum RegistryEvent {
    WindowCreated {
        id: WindowId,
        dk: DesktopKey,
        sk: SurfaceKey,
    },
    WindowMapped {
        id: WindowId,
    },
    WindowUnmapped {
        id: WindowId,
    },
    WindowDestroyed {
        id: WindowId,
    },
    LifecycleChanged {
        id: WindowId,
        old: LifecycleState,
        new: LifecycleState,
    },
}

