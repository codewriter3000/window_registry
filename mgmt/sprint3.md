# Sprint 3 Design Notes: Event Queue and Threading Semantics

## Goals

Sprint 3 replaces or extends the current event return style with a channel-based event queue
that is safe across threads, preserves ordering, and documents backpressure behavior.

## Proposed API Surface

- `RegistryEventQueue` (new type) that manages broadcast delivery to subscribers.
- `RegistryEventReceiver` (new type) for consuming events.
- `SharedRegistry` gains helpers to publish to a queue after write locks are released.

Implemented methods:
- `RegistryEventQueue::bounded(capacity)` / `RegistryEventQueue::unbounded()`
- `RegistryEventQueue::subscribe()`
- `RegistryEventQueue::send(Vec<RegistryEvent>)`
- `RegistryEventReceiver::recv()` / `try_recv()` / `recv_timeout()` / `iter()`
- `SharedRegistry::{insert,remove,on_map,on_unmap,update}_queued(...)`

## Ordering and Delivery Rules

- Events are delivered in the same order they are emitted by a single registry call.
- Across multiple threads, ordering is per-producer call order; interleaving reflects actual
  execution order of write operations.
- Dispatch occurs after releasing the registry write lock.

## Backpressure and Drop Behavior

- Queue supports bounded and unbounded modes.
- Bounded queues apply backpressure by blocking senders when subscribers are full.
- Unbounded queues do not block but can grow without bound.
- If all subscribers are dropped, sends are silently discarded.
- If the queue is dropped, receivers observe `EventQueueClosed`.

## Threading Guarantees

- `SharedRegistry` remains `Send + Sync`.
- Receivers should be `Send` but not necessarily `Sync`.
- Document whether a single receiver can be shared between threads safely.

## Tests to Add

- Ordering tests:
  - Single-threaded ordering within one call and across sequential calls.
  - Multi-threaded per-thread ordering preserved.
- Backpressure tests:
  - Bounded queue blocks senders until receivers drain.
  - Receiver drop behavior and queue drop behavior.
- Locking tests:
  - Ensure event dispatch happens after unlock (no deadlocks or lock contention).

## Open Questions

- Queue type: `crossbeam-channel`.
- Queue ownership: external queue passed to `SharedRegistry` queued methods.
- Multiple subscribers are supported (broadcast semantics).
- Callback-based dispatch remains available as an optional path.

