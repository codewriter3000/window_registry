use std::{
    num::NonZeroU32, 
    hash::{Hash, Hasher},
    ptr::NonNull,
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
#[derive(Copy, Clone)]
pub struct DesktopKey(NonNull<weston_desktop_surface>);

impl DesktopKey {
    pub unsafe fn from_ptr(ptr: *mut weston_desktop_surface) -> Self {
        Self(NonNull::new(ptr).expect("desktop_surface ptr was null"))
    }

    pub fn as_ptr(self) -> *mut weston_desktop_surface {
        self.0.as_ptr()
    }
}

impl PartialEq for DesktopKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for DesktopKey {}

impl Hash for DesktopKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0.as_ptr() as usize).hash(state);
    }
}

impl Debug for DesktopKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "DesktopKey({:p})", self.0.as_ptr())
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct SurfaceKey(NonNull<weston_surface>);

impl SurfaceKey {
    pub unsafe fn from_ptr(ptr: *mut weston_surface) -> Self {
        Self(NonNull::new(ptr).expect("weston_surface ptr was null"))
    }

    pub fn as_ptr(self) -> *mut weston_surface {
        self.0.as_ptr()
    }
}

impl PartialEq for SurfaceKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for SurfaceKey {}

impl Hash for SurfaceKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0.as_ptr() as usize).hash(state);
    }
}

impl Debug for SurfaceKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "SurfaceKey({:p})", self.0.as_ptr())
    }
}
