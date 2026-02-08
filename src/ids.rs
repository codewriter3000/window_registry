use std::{
    num::NonZeroU32, 
    hash::{Hash, Hasher},
    fmt::{Debug, Formatter, Result},
};

use crate::{
    weston_desktop_surface,
    weston_surface,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct WindowId {
    pub(crate) index: u32,
    pub(crate) gen: NonZeroU32,
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct DesktopKey(usize);

impl DesktopKey {
    pub unsafe fn from_ptr(ptr: *mut weston_desktop_surface) -> Self {
        Self(ptr as usize)
    }

    pub fn as_ptr(self) -> *mut weston_desktop_surface {
        self.0 as *mut weston_desktop_surface
    }
}

impl Hash for DesktopKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Debug for DesktopKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "DesktopKey({:#x})", self.0)
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SurfaceKey(usize);

impl SurfaceKey {
    pub unsafe fn from_ptr(ptr: *mut weston_surface) -> Self {
        Self(ptr as usize)
    }

    pub fn as_ptr(self) -> *mut weston_surface {
        self.0 as *mut weston_surface
    }
}

impl Hash for SurfaceKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Debug for SurfaceKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "SurfaceKey({:#x})", self.0)
    }
}

