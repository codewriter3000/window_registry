mod weston_sys;
pub use weston_sys::*;

// Mostly private modules
mod ids;
mod model;
mod registry;
mod events;
mod error;
mod shared;

// Public re-exports
pub use ids::{WindowId, DesktopKey, SurfaceKey};
pub use model::{WindowRecord, WindowInfo, LifecycleState};
pub use registry::{Slot, Registry};
pub use events::RegistryEvent;
pub use error::RegistryError;
pub use shared::SharedRegistry;
