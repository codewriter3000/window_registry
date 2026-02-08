use crate::{DesktopKey, SurfaceKey, WindowId};

#[derive(Debug, Clone)]
pub enum RegistryError {
    DesktopKeyAlreadyRegistered { dk: DesktopKey, existing: WindowId },
    SurfaceKeyAlreadyRegistered { sk: SurfaceKey, existing: WindowId },
    InvalidWindowId(WindowId),
}

