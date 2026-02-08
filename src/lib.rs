mod weston_sys;
pub use weston_sys::*;

// Mostly private modules
mod ids;
mod model;
mod registry;
mod events;
mod error;
mod shared;
mod event_queue;
mod handles;
mod weston;
mod weston_adapter;

// Public re-exports
pub use ids::{WindowId, DesktopKey, SurfaceKey};
pub use model::{
	LifecycleState,
	OutputId,
	WindowGeometry,
	WindowInfo,
	WindowRecord,
	WindowState,
	WindowUpdate,
	WorkspaceId,
};
pub use registry::{Slot, Registry};
pub use events::{RegistryEvent, WindowChange, WindowChanges};
pub use error::RegistryError;
pub use shared::SharedRegistry;
pub use event_queue::{RegistryEventQueue, RegistryEventReceiver};
pub use handles::CompositorHandles;
pub use weston::on_new_desktop_surface;
pub use weston::on_new_desktop_surface_with_keys;
pub use weston::{
	WestonGlueContext,
	weston_handle_commit,
	weston_handle_configure,
	weston_handle_destroy,
	weston_handle_focus,
	weston_handle_map,
	weston_handle_new_desktop_surface,
	weston_handle_output,
	weston_handle_parent,
	weston_handle_unmap,
};
pub use weston_adapter::{FakeWeston, RegistryAdapter, WestonAdapter, WestonEvent};
