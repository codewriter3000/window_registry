mod weston_sys;
pub use weston_sys::*;

// Mostly private modules
mod ids;
mod model;
mod registry;

// Public re-exports
pub use ids::{WindowId, Slot, DesktopKey, SurfaceKey};
pub use model::{WindowRecord};
pub use registry::{Registry, on_new_desktop_surface};
