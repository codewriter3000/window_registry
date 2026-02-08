mod weston_sys;
pub use weston_sys::*;

// Mostly private modules
mod ids;
mod model;
mod registry;

// Public re-exports
pub use ids::{WindowId, DesktopKey, SurfaceKey};
pub use model::{WindowRecord, WindowInfo};
pub use registry::{Slot, Registry};
