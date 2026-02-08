# Sprint 1 Design Notes

## Model Fields

Sprint 1 expands `WindowRecord` and `WindowInfo` with fields that describe geometry, state,
focus, workspace/output placement, stacking, and parent/child relationships. `WindowInfo`
is the public-facing interface and should mirror `WindowRecord`.

Default values on insertion:
- `geometry`: `None` (unknown until configured)
- `state`: all flags false
- `is_focused`: `false`
- `workspace`: `None`
- `output`: `None`
- `stack_index`: `0`
- `parent_id`: `None`
- `children`: empty
- `title`/`app_id`: `None`

Destroy policy remains remove-on-destroy (no `Destroyed` in snapshots).

## Event Taxonomy (Grouped Changes)

Sprint 1 replaces per-field events with grouped change events:
- `WindowCreated { id, dk, sk }`
- `WindowChanged { id, changes }`
- `WindowDestroyed { id }`

`WindowChanged` carries a `WindowChanges` payload; each changed field is represented by a
`WindowChange<T> { old, new }`. This allows a single event to describe multiple changes
when they occur in the same operation.

## Ordering Rules

Within a single registry call, events are emitted in deterministic order and reflect
post-update state. For lifecycle transitions (`on_map`, `on_unmap`), emit a single
`WindowChanged` event whose `lifecycle` field captures old/new values.
