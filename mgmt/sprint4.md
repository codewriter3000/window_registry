# Sprint 4 Design Notes: libweston Glue Expansion

## Goals

Sprint 4 extends libweston integration to map compositor callbacks into registry updates,
expands the FFI surface, and validates lifecycle mapping without regressing core behavior.

## Scope

- Hook additional libweston desktop surface events (map/unmap/destroy/configure/commit/focus/output).
- Translate weston callbacks into `Registry` updates and grouped `WindowChanged` events.
- Formalize FFI expectations, required callbacks, and ownership/lifetime rules.

## Expected API Flow

1. Create or fetch `WindowId` on new desktop surface.
2. On map/unmap, call `SharedRegistry::on_map_queued` / `on_unmap_queued`.
3. On destroy, call `SharedRegistry::remove_window_queued`.
4. On configure/commit/focus/output/parent/geometry updates, call `SharedRegistry::update_window_queued`.
5. Route all emitted events into the queue for external consumers.

## FFI Surface Expansion

Add stubs (and document expectations) for weston types and callbacks (libweston 14.0.2):
- `weston_desktop_surface_listener` callbacks for map, unmap, destroy, configure, commit.
- Compositor focus hook callback (separate from desktop surface listener).
- Compositor output hook callback (separate from desktop surface listener).
- Surface metadata accessors (title, app_id, parent relationships) remain optional.

Document:
- Which weston callbacks are required vs optional.
- Threading expectations (callbacks likely on compositor thread).
- Ownership/lifetime rules for pointers and when keys are considered valid.

## Event Mapping Rules

- `map` -> `Registry::on_map` (emit lifecycle change in a single `WindowChanged`).
- `unmap` -> `Registry::on_unmap`.
- `destroy` -> `Registry::remove_window`.
- `configure` / `commit` -> geometry/state updates (batched when possible).
- Focus changes -> `is_focused` updates via compositor hook.
- Output/workspace changes -> update both fields together via compositor hook.

## Tests to Add

- Weston glue tests:
  - New surface registers and emits `WindowCreated`.
  - Map/unmap/destroy callbacks trigger correct registry updates.
  - Configure updates geometry via callback glue; commit is a no-op.
  - Focus changes via compositor hook clear previous focus and emit events for both windows.
  - Output/workspace changes via compositor hook update together and validate pairing.
  - Parent changes update both parent/child snapshots.
- FFI expectations:
  - Missing callbacks produce clear errors or no-ops (documented behavior).
  - Pointer uniqueness and reverse lookup invariants are preserved.

## Open Questions

- Exact weston callback names and required headers for libweston 14.0.2.
- Whether geometry should be sourced from configure or commit (or both).
- How to represent stacking order from weston (if available) for `stack_index` updates.

## Threading and Lifetime Notes

- Callbacks are expected on the compositor thread; glue must not block.
- Pointers passed from libweston must remain valid for the lifetime of registration.
- Desktop/surface pointer uniqueness is assumed while registered.
- Queue send happens after registry write lock is released.

