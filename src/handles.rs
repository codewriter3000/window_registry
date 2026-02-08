use std::{collections::HashMap, ptr::NonNull};
use crate::{WindowId, weston_view};

pub struct CompositorHandles {
    views: HashMap<WindowId, NonNull<weston_view>>,
}

impl CompositorHandles {
    pub fn set_view(&mut self, id: WindowId, view: *mut weston_view) {
        let nn = NonNull::new(view).expect("weston_view ptr was null");
        self.views.insert(id, nn);
    }

    pub fn remove_view(&mut self, id: WindowId) {
        self.views.remove(&id);
    }

    pub fn get_view(&self, id: WindowId) -> Option<NonNull<weston_view>> {
        self.views.get(&id).copied()
    }
}

