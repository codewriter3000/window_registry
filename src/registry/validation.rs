use crate::{RegistryError, WindowGeometry, WindowId, WindowState};

use super::Registry;

impl Registry {
    pub(crate) fn find_focused(&self, exclude: Option<WindowId>) -> Option<WindowId> {
        for slot in &self.slots {
            let Some(rec) = slot.value.as_ref() else { continue };
            if Some(rec.id) == exclude {
                continue;
            }
            if rec.is_focused {
                return Some(rec.id);
            }
        }
        None
    }

    pub(crate) fn is_ancestor(&self, start: WindowId, ancestor: WindowId) -> bool {
        let mut current = Some(start);
        while let Some(id) = current {
            let Some(rec) = self.get(id) else { break };
            if rec.parent_id == Some(ancestor) {
                return true;
            }
            current = rec.parent_id;
        }
        false
    }

    pub(crate) fn validate_geometry(
        &self,
        id: WindowId,
        geometry: WindowGeometry,
    ) -> Result<(), RegistryError> {
        if geometry.width < 0 || geometry.height < 0 {
            return Err(RegistryError::InvalidGeometry { id, geometry });
        }
        if geometry.x.checked_add(geometry.width).is_none()
            || geometry.y.checked_add(geometry.height).is_none() {
            return Err(RegistryError::GeometryOverflow { id, geometry });
        }
        Ok(())
    }

    pub(crate) fn is_valid_state(state: WindowState) -> bool {
        if state.minimized && (state.maximized || state.fullscreen) {
            return false;
        }
        if state.maximized && state.fullscreen {
            return false;
        }
        true
    }
}
