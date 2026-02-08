# Sprint 2 Summary

This sprint focused on implementing registry core behavior for the expanded window model,
including validation, event emission, and ordering guarantees for updates.

## Implementation Overview

- Added a unified update API (`WindowUpdate` + `Registry::update_window`) that updates geometry,
  state, focus, workspace/output, stacking, and parent/child relationships, emitting a single
  grouped `WindowChanged` event per target window.
- Enforced validation rules for geometry bounds and overflow, state flag compatibility,
  workspace/output pairing, stack index bounds, parent/child relationships, and cycle detection.
- Implemented focus switching semantics to keep a single focused window and emit events for
  both the previously focused window and the newly focused window.
- Implemented stack reordering adjustments when changing `stack_index`, including emitting
  events for affected windows and keeping indices contiguous.
- Added parent/child maintenance for both parent updates and child add/remove flows, with
  bidirectional updates and grouped change events.
- Refactored registry code into modules for clarity: core storage/lifecycle, update logic,
  and validation helpers.

## Tests Added

- Comprehensive tests for validation errors (geometry, state, workspace/output, parent/child,
  stack bounds), grouped event emission, and ordering behaviors.
- Coverage for parent switching, child add/remove, focus handoff, and forward/backward
  stack index reordering.
- Added SharedRegistry tests ensuring update errors propagate and dispatch is skipped on error.

## Files Touched

- Registry implementation split and expanded under `src/registry/` (core, updates, validation).
- New update API and error variants wired into public surface.
- New tests added in `tests/registry_updates.rs` and updates to shared registry error tests.

