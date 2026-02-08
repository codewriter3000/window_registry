use std::sync::{Arc, Mutex};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TryRecvError};

use crate::{RegistryError, RegistryEvent};

#[derive(Debug)]
struct QueueInner {
    subscribers: Mutex<Vec<Sender<RegistryEvent>>>,
    capacity: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RegistryEventQueue {
    inner: Arc<QueueInner>,
}

#[derive(Debug, Clone)]
pub struct RegistryEventReceiver {
    receiver: Receiver<RegistryEvent>,
}

impl RegistryEventQueue {
    pub fn bounded(capacity: usize) -> Self {
        Self::new(Some(capacity))
    }

    pub fn unbounded() -> Self {
        Self::new(None)
    }

    fn new(capacity: Option<usize>) -> Self {
        let inner = Arc::new(QueueInner {
            subscribers: Mutex::new(Vec::new()),
            capacity,
        });
        Self { inner }
    }

    pub fn subscribe(&self) -> RegistryEventReceiver {
        let (tx, rx) = match self.inner.capacity {
            Some(capacity) => crossbeam_channel::bounded(capacity),
            None => crossbeam_channel::unbounded(),
        };

        self.inner.subscribers.lock().expect("event queue lock poisoned").push(tx);

        RegistryEventReceiver { receiver: rx }
    }

    pub fn send(&self, events: Vec<RegistryEvent>) -> Result<(), RegistryError> {
        let mut subscribers = self
            .inner
            .subscribers
            .lock()
            .expect("event queue lock poisoned");

        for event in events {
            subscribers.retain(|sender| sender.send(event.clone()).is_ok());
        }
        Ok(())
    }

    pub fn sender(&self) -> Sender<RegistryEvent> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let queue = self.clone();
        std::thread::spawn(move || {
            while let Ok(event) = rx.recv() {
                if queue.send(vec![event]).is_err() {
                    break;
                }
            }
        });
        tx
    }
}

impl RegistryEventReceiver {
    pub fn recv(&self) -> Result<RegistryEvent, RegistryError> {
        self.receiver.recv().map_err(|_| RegistryError::EventQueueClosed)
    }

    pub fn try_recv(&self) -> Result<Option<RegistryEvent>, RegistryError> {
        match self.receiver.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(TryRecvError::Disconnected) => Err(RegistryError::EventQueueClosed),
        }
    }

    pub fn recv_timeout(&self, timeout: std::time::Duration) -> Result<RegistryEvent, RegistryError> {
        self.receiver.recv_timeout(timeout).map_err(Self::map_recv_timeout)
    }

    pub fn iter(&self) -> impl Iterator<Item = RegistryEvent> + '_ {
        self.receiver.iter()
    }

    pub(crate) fn map_recv_timeout(err: RecvTimeoutError) -> RegistryError {
        match err {
            RecvTimeoutError::Timeout => RegistryError::EventQueueTimeout,
            RecvTimeoutError::Disconnected => RegistryError::EventQueueClosed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RegistryEventReceiver;
    use crate::RegistryError;

    #[test]
    fn map_recv_timeout_maps_errors() {
        let timeout = RegistryEventReceiver::map_recv_timeout(
            crossbeam_channel::RecvTimeoutError::Timeout,
        );
        assert!(matches!(timeout, RegistryError::EventQueueTimeout));

        let closed = RegistryEventReceiver::map_recv_timeout(
            crossbeam_channel::RecvTimeoutError::Disconnected,
        );
        assert!(matches!(closed, RegistryError::EventQueueClosed));
    }
}

