use std::sync::{Arc, RwLock};

use crate::{
    Registry, RegistryEvent, RegistryError,
    WindowId, WindowInfo, DesktopKey, SurfaceKey,
};

#[derive(Clone, Debug)]
pub struct SharedRegistry {
    inner: Arc<RwLock<Registry>>,
}

impl SharedRegistry {
    pub fn new(reg: Registry) -> Self {
        Self { inner: Arc::new(RwLock::new(reg)) }
    }

    // READ snapshots
    pub fn snapshot(&self, id: WindowId) -> Option<WindowInfo> {
        let r = self.inner.read().expect("registry lock poisoned");
        r.snapshot(id)
    }

    pub fn snapshot_all(&self) -> Vec<WindowInfo> {
        let r = self.inner.read().expect("registry lock poisoned");
        r.snapshot_all()
    }

    // WRITE + dispatch after unlock
    pub fn insert_window_with<F>(
        &self,
        dk: DesktopKey,
        sk: SurfaceKey,
        mut dispatch: F,
    ) -> Result<WindowId, RegistryError>
    where
        F: FnMut(Vec<RegistryEvent>),
    {
        let (id, events) = {
            let mut r = self.inner.write().expect("registry lock poisoned");
            r.insert_window(dk, sk)?
        }; // unlock here

        dispatch(events);
        Ok(id)
    }

    pub fn remove_window_with<F>(
        &self,
        id: WindowId,
        mut dispatch: F,
    ) -> Result<(), RegistryError>
    where
        F: FnMut(Vec<RegistryEvent>),
    {
        let events = {
            let mut r = self.inner.write().expect("registry lock poisoned");
            let (_record, events) = r.remove_window(id)?;
            events
        }; // unlock

        dispatch(events);
        Ok(())
    }
}

