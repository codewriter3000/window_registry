use std::{
    num::NonZeroU32, 
    collections::HashMap,
    fmt::Debug,
};

use crate::{
    RegistryEvent,
    RegistryError,
    LifecycleState,
    WindowRecord,
    WindowId,
    WindowInfo,
    DesktopKey,
    SurfaceKey,
    WindowChange,
    WindowChanges,
    WindowState,
};

#[derive(Debug)]
pub struct Slot {
    pub gen: NonZeroU32,
    pub value: Option<WindowRecord>,
}

#[derive(Debug)]
pub struct Registry {
    slots: Vec<Slot>,
    free: Vec<u32>,

    pub surface_map: HashMap<SurfaceKey, WindowId>,
    pub desktop_map: HashMap<DesktopKey, WindowId>,
}

impl Registry {
    pub fn new() -> Self {
        Self { 
            slots: Vec::new(),
            free: Vec::new(),
            surface_map: HashMap::new(),
            desktop_map: HashMap::new(),
        }
    }

    /// Reserves a slot and returns a fresh (index, generation) id.
    fn alloc_id(&mut self) -> WindowId {
        if let Some(index) = self.free.pop() {
            // Slot exists but is currently empty
            let slot = &mut self.slots[index as usize];

            // Bump generation (wrapping is fine; extremely unlikely to collide in practice
            let next = slot.gen.get().wrapping_add(1).max(1);
            slot.gen = NonZeroU32::new(next).unwrap();

            WindowId { index, gen: slot.gen }
        } else {
            // create a new slot
            let gen = NonZeroU32::new(1).unwrap();
            let index = self.slots.len() as u32;
            self.slots.push(Slot { gen, value: None });
            WindowId { index, gen }
        }
    }

    /// Inserts value and returns its WindowId (fresh id each time).
    /// This is the low-level, libweston-agnostic insertion.
    pub fn insert(&mut self, value: WindowRecord) -> WindowId {
        let id = self.alloc_id();
        let slot = &mut self.slots[id.index as usize];
        debug_assert!(slot.value.is_none());
        slot.value = Some(value);
        id
    }

    /// Deliverable C: libweston-aware insertion helper with invariant checks.
    ///
    /// Returns:
    /// - Ok((id, events)) on success
    /// - Err(RegistryError::...) if dk/sk are already registered
    pub fn insert_window(
        &mut self,
        dk: DesktopKey,
        sk: SurfaceKey,
    ) -> Result<(WindowId, Vec<RegistryEvent>), RegistryError> {
        if let Some(existing) = self.desktop_map.get(&dk).copied() {
            return Err(RegistryError::DesktopKeyAlreadyRegistered { dk, existing });
        }
        if let Some(existing) = self.surface_map.get(&sk).copied() {
            return Err(RegistryError::SurfaceKeyAlreadyRegistered { sk, existing });
        }

        let id = self.alloc_id();

        let record = WindowRecord {
            id,
            dk,
            sk,
            lifecycle: LifecycleState::Created,
            geometry: None,
            state: WindowState::default(),
            is_focused: false,
            workspace: None,
            output: None,
            stack_index: 0,
            parent_id: None,
            children: Vec::new(),
            title: None,
            app_id: None,
        };

        let slot = &mut self.slots[id.index as usize];
        debug_assert!(slot.value.is_none());
        slot.value = Some(record);

        self.desktop_map.insert(dk, id);
        self.surface_map.insert(sk, id);

        let events = vec![
            RegistryEvent::WindowCreated { id, dk, sk },
        ];

        Ok((id, events))
    }
    /// Validates that an id is still live and returns a reference.
    pub fn get(&self, id: WindowId) -> Option<&WindowRecord> {
        let slot = self.slots.get(id.index as usize)?;
        if slot.gen == id.gen {
            slot.value.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: WindowId) -> Option<&mut WindowRecord> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.gen == id.gen {
            slot.value.as_mut()
        } else {
            None
        }
    }

    pub fn snapshot(&self, id: WindowId) -> Option<WindowInfo> {
        self.get(id).map(WindowInfo::from)
    }

    pub fn snapshot_all(&self) -> Vec<WindowInfo> {
        self.slots
            .iter()
            .filter_map(|s| s.value.as_ref())
            .map(WindowInfo::from)
            .collect()
    } 

    pub fn from_desktop(&self, dk: DesktopKey) -> Option<WindowId> {
        self.desktop_map.get(&dk).copied()
    }

    pub fn from_surface(&self, sk: SurfaceKey) -> Option<WindowId> {
        self.surface_map.get(&sk).copied()
    }

    pub fn set_title(&mut self, id: WindowId, title: String) -> bool {
        let rec = match self.get_mut(id) { Some(r) => r, None => return false };
        rec.title = Some(title);
        true
    } 

    /// Removes the value if the id is valid; invalidates the id thereafter.
    ///
    /// NOTE: this does NOT remove from desktop_map/surface_map because we don't know the dk/sk
    /// here (T is generic). In practice, your WindowRecord should store dk/sk so you can remove
    /// them too (see note below).
    pub fn remove(&mut self, id: WindowId) -> Option<WindowRecord> {
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.gen != id.gen {
            return None;
        }
        let out = slot.value.take();
        if out.is_some() {
            self.free.push(id.index);
        }
        out
    }

    pub fn remove_window(
        &mut self,
        id: WindowId,
    ) -> Result<(WindowRecord, Vec<RegistryEvent>), RegistryError> {
        let slot = self.slots.get_mut(id.index as usize)
            .ok_or(RegistryError::InvalidWindowId(id))?;

        if slot.gen != id.gen {
            return Err(RegistryError::InvalidWindowId(id));
        }

        let record = slot.value.take().ok_or(RegistryError::InvalidWindowId(id))?;

        // Remove reverse lookups
        self.desktop_map.remove(&record.dk);
        self.surface_map.remove(&record.sk);

        // Free slot for reuse
        self.free.push(id.index);

        let events = vec![
            RegistryEvent::WindowDestroyed { id },
        ];

        Ok((record, events))
    }

    // Optional: lifecycle transitions (C-level completeness)
    pub fn on_map(&mut self, id: WindowId) -> Result<Vec<RegistryEvent>, RegistryError> {
        let r = self.get_mut(id).ok_or(RegistryError::InvalidWindowId(id))?;
        let old = r.lifecycle;
        if old != LifecycleState::Mapped {
            r.lifecycle = LifecycleState::Mapped;
            Ok(vec![
                RegistryEvent::WindowChanged {
                    id,
                    changes: WindowChanges {
                        lifecycle: Some(WindowChange { old, new: LifecycleState::Mapped }),
                        ..WindowChanges::default()
                    },
                },
            ])
        } else {
            Ok(vec![])
        }
    }

    pub fn on_unmap(&mut self, id: WindowId) -> Result<Vec<RegistryEvent>, RegistryError> {
        let r = self.get_mut(id).ok_or(RegistryError::InvalidWindowId(id))?;
        let old = r.lifecycle;
        if old == LifecycleState::Mapped {
            r.lifecycle = LifecycleState::Unmapped;
            Ok(vec![
                RegistryEvent::WindowChanged {
                    id,
                    changes: WindowChanges {
                        lifecycle: Some(WindowChange { old, new: LifecycleState::Unmapped }),
                        ..WindowChanges::default()
                    },
                },
            ])
        } else {
            Ok(vec![])
        }
    }
}

