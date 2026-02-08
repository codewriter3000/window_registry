use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use window_registry::{
    Registry,
    RegistryEvent,
    RegistryEventQueue,
    SharedRegistry,
    WindowUpdate,
};

mod common;
use common::TestPtrs;

#[test]
fn event_queue_orders_events_from_registry_calls() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();

    let p1 = TestPtrs::new();
    let (dk1, sk1) = unsafe { p1.keys() };
    let id1 = reg
        .insert_window_queued(dk1, sk1, &queue)
        .expect("insert_window_queued should succeed");

    let p2 = TestPtrs::new();
    let (dk2, sk2) = unsafe { p2.keys() };
    let id2 = reg
        .insert_window_queued(dk2, sk2, &queue)
        .expect("insert_window_queued should succeed");

    reg.remove_window_queued(id2, &queue)
        .expect("remove_window_queued should succeed");

    let ev1 = receiver.recv().expect("recv event 1");
    let ev2 = receiver.recv().expect("recv event 2");
    let ev3 = receiver.recv().expect("recv event 3");

    assert!(matches!(ev1, RegistryEvent::WindowCreated { id, .. } if id == id1));
    assert!(matches!(ev2, RegistryEvent::WindowCreated { id, .. } if id == id2));
    assert!(matches!(ev3, RegistryEvent::WindowDestroyed { id } if id == id2));
}

#[test]
fn event_queue_broadcasts_to_all_subscribers() {
    let queue = RegistryEventQueue::unbounded();
    let rx_a = queue.subscribe();
    let rx_b = queue.subscribe();

    let mut reg = Registry::new();
    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    queue
        .send(vec![RegistryEvent::WindowDestroyed { id }])
        .expect("send should succeed");

    let ev_a = rx_a.recv().expect("subscriber A recv");
    let ev_b = rx_b.recv().expect("subscriber B recv");

    assert!(matches!(ev_a, RegistryEvent::WindowDestroyed { .. }));
    assert!(matches!(ev_b, RegistryEvent::WindowDestroyed { .. }));
}

#[test]
fn event_queue_bounded_blocks_until_receive() {
    let queue = RegistryEventQueue::bounded(1);
    let rx = queue.subscribe();

    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();
    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let id1 = reg.insert_window(dk1, sk1).expect("insert_window A").0;
    let id2 = reg.insert_window(dk2, sk2).expect("insert_window B").0;

    let (signal_tx, signal_rx) = std::sync::mpsc::channel();

    std::thread::spawn({
        let queue = queue.clone();
        move || {
            queue
                .send(vec![RegistryEvent::WindowDestroyed { id: id1 }])
                .expect("send 1 should succeed");
            queue
                .send(vec![RegistryEvent::WindowDestroyed { id: id2 }])
                .expect("send 2 should succeed");
            signal_tx.send(()).ok();
        }
    });

    let result = signal_rx.recv_timeout(Duration::from_millis(50));
    assert!(matches!(result, Err(RecvTimeoutError::Timeout)));

    let _ev1 = rx.recv().expect("recv first event");
    let result = signal_rx.recv_timeout(Duration::from_millis(200));
    assert!(matches!(result, Ok(())));
}

#[test]
fn event_queue_update_window_emits_change_events() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();

    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };
    let id = reg
        .insert_window_queued(dk, sk, &queue)
        .expect("insert_window_queued should succeed");

    let mut update = WindowUpdate::default();
    update.is_focused = Some(true);
    reg.update_window_queued(id, update, &queue)
        .expect("update_window_queued should succeed");

    let _created = receiver.recv().expect("created event");
    let changed = receiver.recv().expect("changed event");
    assert!(matches!(changed, RegistryEvent::WindowChanged { id: ev_id, .. } if ev_id == id));
}

#[test]
fn event_queue_preserves_per_thread_ordering() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();

    let reg_a = reg.clone();
    let queue_a = queue.clone();
    let handle_a = std::thread::spawn(move || {
        let p = TestPtrs::new();
        let (dk, sk) = unsafe { p.keys() };
        let id = reg_a
            .insert_window_queued(dk, sk, &queue_a)
            .expect("insert A");
        reg_a.remove_window_queued(id, &queue_a)
            .expect("remove A");
        id
    });

    let reg_b = reg.clone();
    let queue_b = queue.clone();
    let handle_b = std::thread::spawn(move || {
        let p = TestPtrs::new();
        let (dk, sk) = unsafe { p.keys() };
        let id = reg_b
            .insert_window_queued(dk, sk, &queue_b)
            .expect("insert B");
        reg_b.remove_window_queued(id, &queue_b)
            .expect("remove B");
        id
    });

    let id_a = handle_a.join().expect("thread A join");
    let id_b = handle_b.join().expect("thread B join");

    let mut created_a = None;
    let mut destroyed_a = None;
    let mut created_b = None;
    let mut destroyed_b = None;

    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_millis(200)
        && (created_a.is_none() || destroyed_a.is_none() || created_b.is_none() || destroyed_b.is_none())
    {
        let event = receiver
            .recv_timeout(Duration::from_millis(20))
            .expect("recv event");
        if let RegistryEvent::WindowCreated { id, .. } = event {
            if id == id_a && created_a.is_none() {
                created_a = Some(start.elapsed());
            } else if id == id_b && created_b.is_none() {
                created_b = Some(start.elapsed());
            }
        }
        if let RegistryEvent::WindowDestroyed { id } = event {
            if id == id_a && destroyed_a.is_none() {
                destroyed_a = Some(start.elapsed());
            } else if id == id_b && destroyed_b.is_none() {
                destroyed_b = Some(start.elapsed());
            }
        }
    }

    assert!(created_a.is_some() && destroyed_a.is_some());
    assert!(created_b.is_some() && destroyed_b.is_some());
    assert!(created_a.unwrap() <= destroyed_a.unwrap());
    assert!(created_b.unwrap() <= destroyed_b.unwrap());
}

#[test]
fn event_queue_send_drops_without_subscribers() {
    let queue = RegistryEventQueue::unbounded();

    let mut reg = Registry::new();
    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    queue
        .send(vec![RegistryEvent::WindowDestroyed { id }])
        .expect("send should not error without subscribers");
}

#[test]
fn event_queue_receiver_closes_when_queue_dropped() {
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();

    drop(queue);

    let err = receiver.recv().expect_err("recv should fail after queue drop");
    assert!(matches!(err, window_registry::RegistryError::EventQueueClosed));
}

#[test]
fn event_queue_send_happens_after_unlock() {
    let reg = SharedRegistry::new(Registry::new());
    let queue = RegistryEventQueue::bounded(0);
    let receiver = queue.subscribe();

    let reg_clone = reg.clone();
    let queue_clone = queue.clone();
    let handle = std::thread::spawn(move || {
        let p = TestPtrs::new();
        let (dk, sk) = unsafe { p.keys() };
        reg_clone
            .insert_window_queued(dk, sk, &queue_clone)
            .expect("insert_window_queued should succeed");
    });

    let start = std::time::Instant::now();
    let mut seen = false;
    while start.elapsed() < Duration::from_millis(200) {
        if reg.snapshot_all().len() == 1 {
            seen = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(seen, "write lock should be released before send");

    let _event = receiver.recv().expect("recv event");
    handle.join().expect("thread join");
}

#[test]
fn event_queue_sender_delivers_events() {
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();

    let mut reg = Registry::new();
    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    let sender = queue.sender();
    sender
        .send(RegistryEvent::WindowDestroyed { id })
        .expect("sender should deliver");

    let ev = receiver.recv().expect("recv event");
    assert!(matches!(ev, RegistryEvent::WindowDestroyed { id: ev_id } if ev_id == id));
}

#[test]
fn event_queue_try_recv_reports_empty_and_value() {
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();

    let empty = receiver.try_recv().expect("try_recv should succeed");
    assert!(empty.is_none());

    let mut reg = Registry::new();
    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    queue
        .send(vec![RegistryEvent::WindowDestroyed { id }])
        .expect("send should succeed");

    let received = receiver.try_recv().expect("try_recv should succeed");
    assert!(matches!(received, Some(RegistryEvent::WindowDestroyed { id: ev_id }) if ev_id == id));
}

#[test]
fn event_queue_recv_timeout_reports_timeout_and_value() {
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();

    let err = receiver
        .recv_timeout(Duration::from_millis(10))
        .expect_err("timeout should error");
    assert!(matches!(err, window_registry::RegistryError::EventQueueTimeout));

    let mut reg = Registry::new();
    let p = TestPtrs::new();
    let (dk, sk) = unsafe { p.keys() };
    let id = reg.insert_window(dk, sk).expect("insert_window should succeed").0;

    queue
        .send(vec![RegistryEvent::WindowDestroyed { id }])
        .expect("send should succeed");

    let ev = receiver
        .recv_timeout(Duration::from_millis(50))
        .expect("recv_timeout should return event");
    assert!(matches!(ev, RegistryEvent::WindowDestroyed { id: ev_id } if ev_id == id));
}

#[test]
fn event_queue_iter_yields_events_in_order() {
    let queue = RegistryEventQueue::unbounded();
    let receiver = queue.subscribe();

    let mut reg = Registry::new();
    let p1 = TestPtrs::new();
    let p2 = TestPtrs::new();
    let (dk1, sk1) = unsafe { p1.keys() };
    let (dk2, sk2) = unsafe { p2.keys() };
    let id1 = reg.insert_window(dk1, sk1).expect("insert_window A").0;
    let id2 = reg.insert_window(dk2, sk2).expect("insert_window B").0;

    queue
        .send(vec![
            RegistryEvent::WindowDestroyed { id: id1 },
            RegistryEvent::WindowDestroyed { id: id2 },
        ])
        .expect("send should succeed");

    let events: Vec<_> = receiver.iter().take(2).collect();
    assert!(matches!(events.get(0), Some(RegistryEvent::WindowDestroyed { id }) if *id == id1));
    assert!(matches!(events.get(1), Some(RegistryEvent::WindowDestroyed { id }) if *id == id2));
}
