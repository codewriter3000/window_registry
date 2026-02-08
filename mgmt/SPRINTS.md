# Sprint Plan and Deliverables

This document captures sprint scope, deliverables, and exit criteria for the window_registry project.

## Sprint 0: Baseline and Spec

Scope:
- Document current public API and behavior.
- Write design notes for the expanded registry model and event taxonomy.

Deliverables:
- README updates that describe current registry capabilities and invariants.
- API overview in library docs with cross-links to core types.
- High-level design notes for new model fields and event semantics.

Exit criteria:
- Docs are clear enough for a reviewer to implement without additional guidance.
- All existing tests still pass.

## Sprint 1: Model and Event Taxonomy (Tests First)

Scope:
- Define expanded model fields (geometry, state, focus, workspace/output, stacking, parent/child).
- Define expanded event types.
- Write tests that specify model and event behavior.

Deliverables:
- New tests that fail until model and events are implemented.
- Updated model structs and enums with documentation.
- Updated event enum and event docs.

Exit criteria:
- Tests for model and events pass.
- Existing APIs continue to compile with minimal breakage.

## Sprint 2: Registry Core Behavior (Tests First)

Scope:
- Implement registry methods for new model fields.
- Validate transitions and emit events in correct order.

Deliverables:
- Registry tests for new operations, validation, and event ordering.
- New registry methods to update geometry, state, focus, output, workspace, stacking, and parents.

Exit criteria:
- Registry tests pass and existing tests remain green.
- Documented invariants match behavior.

## Sprint 3: Event Queue and Threading Semantics (Tests First)

Scope:
- Replace or extend current event return style with a channel-based event queue.
- Define threading guarantees and backpressure behavior.

Deliverables:
- Tests for channel delivery order, blocking, and drop behavior.
- Event queue API and receiver subscription docs.
- Updated SharedRegistry behavior and documentation.

Exit criteria:
- New event queue tests pass.
- Event delivery works safely across threads and preserves ordering.

## Sprint 4: libweston Glue Expansion (Tests First)

Scope:
- Extend libweston integration points and event mapping.
- Formalize FFI expectations.

Deliverables:
- Tests for weston hook expectations and lifecycle mapping.
- Expanded weston glue to handle map/unmap/destroy/configure/commit/focus/output changes.
- Updated FFI stubs and documentation of required callbacks.

Exit criteria:
- Tests covering weston glue pass.
- No regression in registry core behavior.

## Sprint 5: Hardening and Documentation

Scope:
- Add edge-case tests and finalize documentation.
- Validate API stability and integration guidance.

Deliverables:
- Tests for edge cases (orphaned transients, output removal, focus fallback, destroyed windows).
- Finalized README and library docs.
- Optional integration example or usage guide.

Exit criteria:
- Full test suite passes.
- Documentation is complete and accurate.
