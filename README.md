# Window Registry

A small, libweston-friendly window registry with stable IDs, reverse lookups, and event hooks.
This crate tracks windows created from libweston desktop surfaces and provides a clean Rust API
for querying, snapshotting, and lifecycle transitions.

## Highlights

- Stable `WindowId` values with generation counters to prevent stale reuse.
- Reverse lookups from `DesktopKey` / `SurfaceKey` to `WindowId`.
- Event emission for create/map/unmap/destroy + lifecycle changes.
- `SharedRegistry` wrapper for thread-safe use with event dispatch.

## Module Map

- `lib.rs`: public re-exports and module wiring.
- `ids.rs`: `WindowId`, `DesktopKey`, `SurfaceKey`.
- `model.rs`: `WindowRecord`, `WindowInfo`, `LifecycleState`.
- `registry.rs`: core `Registry` and `Slot` storage.
- `events.rs`: `RegistryEvent` definitions.
- `error.rs`: `RegistryError` types.
- `shared.rs`: `SharedRegistry` for `Arc<RwLock<Registry>>` access.
- `weston.rs`: helper glue for libweston desktop surfaces.
- `weston_sys.rs`: minimal FFI stubs for `weston_surface`, `weston_view`, `weston_desktop_surface`.

## Core Types

### IDs and Keys

- `WindowId`: `{ index: u32, gen: NonZeroU32 }` stable identifier.
- `DesktopKey`: `usize` wrapper for `*mut weston_desktop_surface`.
- `SurfaceKey`: `usize` wrapper for `*mut weston_surface`.

`DesktopKey` / `SurfaceKey` are created with `unsafe fn from_ptr(...)` and used for reverse lookup.
They are `Copy`, `Eq`, `Hash`, and include pointer values in `Debug` output.

### Window Data

- `WindowRecord`: full mutable record stored in the registry.
- `WindowInfo`: immutable snapshot type, cloned from a record.
- `LifecycleState`: `Created | Mapped | Unmapped | Destroyed`.

### Events and Errors

`RegistryEvent` variants:

- `WindowCreated { id, dk, sk }`
- `WindowMapped { id }`
- `WindowUnmapped { id }`
- `WindowDestroyed { id }`
- `LifecycleChanged { id, old, new }`

`RegistryError` variants:

- `DesktopKeyAlreadyRegistered { dk, existing }`
- `SurfaceKeyAlreadyRegistered { sk, existing }`
- `InvalidWindowId(WindowId)`

## Registry API Reference

### Construction

```rust
use window_registry::Registry;

let mut reg = Registry::new();
```

### Insertion

`insert_window(dk, sk)` creates a `WindowRecord`, registers reverse lookup maps, and emits
`WindowCreated`.

```rust
use window_registry::{Registry, DesktopKey, SurfaceKey};

let mut reg = Registry::new();
let (id, events) = reg.insert_window(dk, sk)?;
```

### Lookup and Snapshot

```rust
let record = reg.get(id);
let record_mut = reg.get_mut(id);

let snapshot = reg.snapshot(id);
let all = reg.snapshot_all();

let id_from_desktop = reg.from_desktop(dk);
let id_from_surface = reg.from_surface(sk);
```

### Updates

```rust
let ok = reg.set_title(id, "My App".to_string());
```

### Removal

`remove_window(id)` removes the record, frees the slot, and cleans reverse lookup maps.
It emits `WindowDestroyed`.

```rust
let (record, events) = reg.remove_window(id)?;
```

### Lifecycle Transitions

```rust
let events = reg.on_map(id)?;
let events = reg.on_unmap(id)?;
```

## Shared Registry

`SharedRegistry` wraps `Registry` in `Arc<RwLock<_>>` and provides helpers that dispatch events
after the lock is released.

```rust
use window_registry::{SharedRegistry, Registry, DesktopKey, SurfaceKey};

let shared = SharedRegistry::new(Registry::new());

let id = shared.insert_window_with(dk, sk, |events| {
		// dispatch events to subscribers
		for ev in events {
				eprintln!("event: {ev:?}");
		}
})?;

shared.remove_window_with(id, |events| {
		for ev in events {
				eprintln!("event: {ev:?}");
		}
})?;
```

## Libweston Glue

`weston.rs` shows a sketch for integrating with libweston desktop surfaces. It derives
`DesktopKey` / `SurfaceKey`, inserts the window, and logs the new `WindowId`.

Note: the FFI symbols in `weston_sys.rs` are minimal stubs. The exact function names and
integration flow depend on the libweston-desktop API you target.

## Safety Notes

- `DesktopKey::from_ptr` / `SurfaceKey::from_ptr` are `unsafe` and assume the pointer remains
	valid for the lifetime of its usage in the registry.
- `WindowId` generation counters prevent stale reuse, but callers must still treat raw pointers
	with care.

## Quick Index

- IDs and keys: [src/ids.rs](src/ids.rs)
- Registry core: [src/registry.rs](src/registry.rs)
- Events: [src/events.rs](src/events.rs)
- Errors: [src/error.rs](src/error.rs)
- Shared registry: [src/shared.rs](src/shared.rs)
- Weston glue: [src/weston.rs](src/weston.rs)
- Weston FFI stubs: [src/weston_sys.rs](src/weston_sys.rs)

