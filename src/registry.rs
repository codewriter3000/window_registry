use std::{
    num::NonZeroU32, 
    ptr::NonNull,
    collections::HashMap,
    fmt::Debug,
};

use crate::{
    Slot,
    WindowRecord,
    WindowId,
    DesktopKey,
    SurfaceKey,
    weston_desktop_surface,
    weston_desktop_surface_get_surface,
    weston_surface,
    weston_view,
};

#[derive(Debug)]
pub struct Registry {
    slots: Vec<Slot>,
    free: Vec<u32>,

    // Deliverable B
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

    /// Inserts a window record AND ALSO registers reverse lookup keys.
    /// This is the libweston-aware insertion helper.
    ///
    /// Invariant: a DesktopKey/SurfaceKey must not already be registered.
    pub fn insert_window(&mut self, dk: DesktopKey, sk: SurfaceKey, view: Option<NonNull<weston_view>>) -> WindowId {
        debug_assert!(
            !self.desktop_map.contains_key(&dk),
            "DesktopKey already registered"
        );
        debug_assert!(
            !self.surface_map.contains_key(&sk),
            "SurfaceKey already registered"
        );

        let id = self.alloc_id();
        let record = WindowRecord { id, dk, sk, view };

        let slot = &mut self.slots[id.index as usize];
        debug_assert!(slot.value.is_none());
        slot.value = Some(record);

        self.desktop_map.insert(dk, id);
        self.surface_map.insert(sk, id);

        id
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

    pub fn from_desktop(&self, dk: DesktopKey) -> Option<WindowId> {
        self.desktop_map.get(&dk).copied()
    }

    pub fn from_surface(&self, sk: SurfaceKey) -> Option<WindowId> {
        self.surface_map.get(&sk).copied()
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

    pub fn remove_window(&mut self, id: WindowId) -> Option<WindowRecord> {
        // validate id + take record
        let slot = self.slots.get_mut(id.index as usize)?;
        if slot.gen != id.gen {
            return None;
        }

        let record = slot.value.take()?;

        // remove reverse lookups
        self.desktop_map.remove(&record.dk);
        self.surface_map.remove(&record.sk);

        // free slot index for reuse
        self.free.push(id.index);

        Some(record)
    }
}

/// The exact function names depend on the Weston version
pub unsafe fn on_new_desktop_surface(ds: *mut weston_desktop_surface, reg: &mut Registry) {
    let dk = DesktopKey::from_ptr(ds);

    // you likely extract weston_surface* from the desktop surface via libweston-desktop API:
    let s: *mut weston_surface = weston_desktop_surface_get_surface(ds);
    let sk = SurfaceKey::from_ptr(s);

    let id = reg.insert_window(dk, sk, None);
    
    // log
    eprintln!("New window: {:?}", id);
}
