use std::collections::HashSet;

use crate::{
    RegistryError,
    RegistryEvent,
    WindowChange,
    WindowChanges,
    WindowUpdate,
};

use super::Registry;


impl Registry {
    pub fn update_window(
        &mut self,
        id: crate::WindowId,
        update: WindowUpdate,
    ) -> Result<Vec<RegistryEvent>, RegistryError> {
        let current = self.get(id).ok_or(RegistryError::InvalidWindowId(id))?;

        let current_geometry = current.geometry;
        let current_state = current.state;
        let current_focus = current.is_focused;
        let current_workspace = current.workspace;
        let current_output = current.output;
        let current_stack_index = current.stack_index;
        let current_parent_id = current.parent_id;
        let current_children = current.children.clone();
        let current_title = current.title.clone();
        let current_app_id = current.app_id.clone();

        #[allow(dropping_references)]
        drop(current);

        if let Some(Some(geom)) = update.geometry {
            self.validate_geometry(id, geom)?;
        }

        if let Some(state) = update.state {
            if !Self::is_valid_state(state) {
                return Err(RegistryError::InvalidState { id, state });
            }
        }

        let next_workspace = update.workspace.unwrap_or(current_workspace);
        let next_output = update.output.unwrap_or(current_output);
        if (next_workspace.is_some()) != (next_output.is_some()) {
            return Err(RegistryError::WorkspaceOutputMismatch {
                id,
                workspace: next_workspace,
                output: next_output,
            });
        }

        if let Some(index) = update.stack_index {
            let count = self.live_count();
            if index < 0 || index as usize >= count {
                return Err(RegistryError::StackIndexOutOfBounds { id, index, count });
            }
        }

        if let Some(next_parent) = update.parent_id {
            if let Some(parent_id) = next_parent {
                if parent_id == id {
                    return Err(RegistryError::ParentIsSelf { id });
                }
                if self.get(parent_id).is_none() {
                    return Err(RegistryError::ParentNotFound { id, parent: parent_id });
                }
                if self.is_ancestor(parent_id, id) {
                    return Err(RegistryError::ParentCycle { id, parent: parent_id });
                }
            }
        }

        let mut seen_children = HashSet::new();
        for child_id in &update.add_children {
            if *child_id == id {
                return Err(RegistryError::ParentIsSelf { id });
            }
            if !seen_children.insert(*child_id) {
                return Err(RegistryError::ChildAlreadyPresent { id, child: *child_id });
            }

            let child = self
                .get(*child_id)
                .ok_or(RegistryError::ChildNotFound { id, child: *child_id })?;

            if let Some(existing_parent) = child.parent_id {
                if existing_parent != id {
                    return Err(RegistryError::ChildAlreadyHasParent {
                        id,
                        child: *child_id,
                        existing_parent,
                    });
                }
            }

            if self.is_ancestor(id, *child_id) {
                return Err(RegistryError::ParentCycle { id: *child_id, parent: id });
            }

            if current_children.contains(child_id) {
                return Err(RegistryError::ChildAlreadyPresent { id, child: *child_id });
            }
        }

        for child_id in &update.remove_children {
            if self.get(*child_id).is_none() {
                return Err(RegistryError::ChildNotFound { id, child: *child_id });
            }
        }

        let mut events = Vec::new();
        let mut changes = WindowChanges::default();

        if update.is_focused == Some(true) && !current_focus {
            if let Some(other_id) = self.find_focused(Some(id)) {
                if let Some(other) = self.get_mut(other_id) {
                    let old = other.is_focused;
                    if old {
                        other.is_focused = false;
                        let mut other_changes = WindowChanges::default();
                        other_changes.is_focused = Some(WindowChange { old, new: false });
                        events.push(RegistryEvent::WindowChanged { id: other_id, changes: other_changes });
                    }
                }
            }
        }

        if let Some(new_index) = update.stack_index {
            if new_index != current_stack_index {
                let mut affected = Vec::new();
                for slot in &self.slots {
                    let Some(other) = slot.value.as_ref() else { continue };
                    if other.id == id {
                        continue;
                    }
                    let idx = other.stack_index;
                    if current_stack_index < new_index {
                        if idx > current_stack_index && idx <= new_index {
                            affected.push((other.id, idx, idx - 1));
                        }
                    } else if new_index < current_stack_index {
                        if idx >= new_index && idx < current_stack_index {
                            affected.push((other.id, idx, idx + 1));
                        }
                    }
                }

                affected.sort_by_key(|(win_id, idx, _)| (*idx, win_id.index, win_id.gen));
                for (other_id, old, new) in affected {
                    if let Some(other) = self.get_mut(other_id) {
                        other.stack_index = new;
                        let mut other_changes = WindowChanges::default();
                        other_changes.stack_index = Some(WindowChange { old, new });
                        events.push(RegistryEvent::WindowChanged { id: other_id, changes: other_changes });
                    }
                }

                if let Some(target) = self.get_mut(id) {
                    let old = target.stack_index;
                    target.stack_index = new_index;
                    changes.stack_index = Some(WindowChange { old, new: new_index });
                }
            }
        }

        if let Some(next_parent) = update.parent_id {
            if next_parent != current_parent_id {
                if let Some(old_parent) = current_parent_id {
                    if let Some(parent_rec) = self.get_mut(old_parent) {
                        let old_children = parent_rec.children.clone();
                        parent_rec.children.retain(|cid| *cid != id);
                        if parent_rec.children != old_children {
                            let mut parent_changes = WindowChanges::default();
                            parent_changes.children = Some(WindowChange {
                                old: old_children,
                                new: parent_rec.children.clone(),
                            });
                            events.push(RegistryEvent::WindowChanged { id: old_parent, changes: parent_changes });
                        }
                    }
                }

                if let Some(new_parent) = next_parent {
                    if let Some(parent_rec) = self.get_mut(new_parent) {
                        let old_children = parent_rec.children.clone();
                        if !parent_rec.children.contains(&id) {
                            parent_rec.children.push(id);
                        }
                        if parent_rec.children != old_children {
                            let mut parent_changes = WindowChanges::default();
                            parent_changes.children = Some(WindowChange {
                                old: old_children,
                                new: parent_rec.children.clone(),
                            });
                            events.push(RegistryEvent::WindowChanged { id: new_parent, changes: parent_changes });
                        }
                    }
                }

                if let Some(target) = self.get_mut(id) {
                    let old = target.parent_id;
                    target.parent_id = next_parent;
                    changes.parent_id = Some(WindowChange { old, new: next_parent });
                }
            }
        }

        if !update.add_children.is_empty() || !update.remove_children.is_empty() {
            let mut new_children = current_children.clone();

            for child_id in &update.remove_children {
                if let Some(pos) = new_children.iter().position(|cid| cid == child_id) {
                    new_children.remove(pos);
                    if let Some(child) = self.get_mut(*child_id) {
                        let old = child.parent_id;
                        if old == Some(id) {
                            child.parent_id = None;
                            let mut child_changes = WindowChanges::default();
                            child_changes.parent_id = Some(WindowChange { old, new: None });
                            events.push(RegistryEvent::WindowChanged { id: *child_id, changes: child_changes });
                        }
                    }
                }
            }

            for child_id in &update.add_children {
                if !new_children.contains(child_id) {
                    new_children.push(*child_id);
                }

                if let Some(child) = self.get_mut(*child_id) {
                    let old = child.parent_id;
                    if old != Some(id) {
                        child.parent_id = Some(id);
                        let mut child_changes = WindowChanges::default();
                        child_changes.parent_id = Some(WindowChange { old, new: Some(id) });
                        events.push(RegistryEvent::WindowChanged { id: *child_id, changes: child_changes });
                    }
                }
            }

            if new_children != current_children {
                if let Some(target) = self.get_mut(id) {
                    let old = target.children.clone();
                    target.children = new_children.clone();
                    changes.children = Some(WindowChange { old, new: new_children });
                }
            }
        }

        if let Some(new_geometry) = update.geometry {
            if new_geometry != current_geometry {
                if let Some(target) = self.get_mut(id) {
                    target.geometry = new_geometry;
                }
                changes.geometry = Some(WindowChange { old: current_geometry, new: new_geometry });
            }
        }

        if let Some(new_state) = update.state {
            if new_state != current_state {
                if let Some(target) = self.get_mut(id) {
                    target.state = new_state;
                }
                changes.state = Some(WindowChange { old: current_state, new: new_state });
            }
        }

        if let Some(new_focus) = update.is_focused {
            if new_focus != current_focus {
                if let Some(target) = self.get_mut(id) {
                    target.is_focused = new_focus;
                }
                changes.is_focused = Some(WindowChange { old: current_focus, new: new_focus });
            }
        }

        if update.workspace.is_some() && next_workspace != current_workspace {
            if let Some(target) = self.get_mut(id) {
                target.workspace = next_workspace;
            }
            changes.workspace = Some(WindowChange { old: current_workspace, new: next_workspace });
        }

        if update.output.is_some() && next_output != current_output {
            if let Some(target) = self.get_mut(id) {
                target.output = next_output;
            }
            changes.output = Some(WindowChange { old: current_output, new: next_output });
        }

        if let Some(new_title) = update.title {
            if new_title != current_title {
                if let Some(target) = self.get_mut(id) {
                    target.title = new_title.clone();
                }
                changes.title = Some(WindowChange { old: current_title, new: new_title });
            }
        }

        if let Some(new_app_id) = update.app_id {
            if new_app_id != current_app_id {
                if let Some(target) = self.get_mut(id) {
                    target.app_id = new_app_id.clone();
                }
                changes.app_id = Some(WindowChange { old: current_app_id, new: new_app_id });
            }
        }

        if !changes.is_empty() {
            events.push(RegistryEvent::WindowChanged { id, changes });
        }

        Ok(events)
    }
}
