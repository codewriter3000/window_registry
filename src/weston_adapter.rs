use crate::{
    DesktopKey,
    OutputId,
    RegistryError,
    RegistryEventQueue,
    SharedRegistry,
    SurfaceKey,
    WindowGeometry,
    WindowId,
    WindowUpdate,
    WorkspaceId,
};

#[derive(Debug, Clone)]
pub enum WestonEvent {
    NewSurface { dk: DesktopKey, sk: SurfaceKey },
    Map { id: WindowId },
    Unmap { id: WindowId },
    Destroy { id: WindowId },
    Configure { id: WindowId, geom: WindowGeometry },
    Commit { id: WindowId },
    Focus { id: WindowId, focused: bool },
    Output { id: WindowId, output: OutputId, workspace: WorkspaceId },
    Parent { id: WindowId, parent: Option<WindowId> },
    Title { id: WindowId, title: Option<String> },
    AppId { id: WindowId, app_id: Option<String> },
}

pub trait WestonAdapter {
    fn handle_event(&mut self, event: WestonEvent) -> Result<(), RegistryError>;
}

#[derive(Debug, Clone)]
pub struct RegistryAdapter {
    pub reg: SharedRegistry,
    pub queue: RegistryEventQueue,
}

impl RegistryAdapter {
    pub fn new(reg: SharedRegistry, queue: RegistryEventQueue) -> Self {
        Self { reg, queue }
    }
}

impl WestonAdapter for RegistryAdapter {
    fn handle_event(&mut self, event: WestonEvent) -> Result<(), RegistryError> {
        match event {
            WestonEvent::NewSurface { dk, sk } => {
                self.reg.insert_window_queued(dk, sk, &self.queue)?;
                Ok(())
            }
            WestonEvent::Map { id } => self.reg.on_map_queued(id, &self.queue),
            WestonEvent::Unmap { id } => self.reg.on_unmap_queued(id, &self.queue),
            WestonEvent::Destroy { id } => self.reg.remove_window_queued(id, &self.queue),
            WestonEvent::Configure { id, geom } => {
                let mut update = WindowUpdate::default();
                update.geometry = Some(Some(geom));
                self.reg.update_window_queued(id, update, &self.queue)
            }
            WestonEvent::Commit { id } => {
                let _ = id;
                Ok(())
            }
            WestonEvent::Focus { id, focused } => {
                let mut update = WindowUpdate::default();
                update.is_focused = Some(focused);
                self.reg.update_window_queued(id, update, &self.queue)
            }
            WestonEvent::Output { id, output, workspace } => {
                let mut update = WindowUpdate::default();
                update.output = Some(Some(output));
                update.workspace = Some(Some(workspace));
                self.reg.update_window_queued(id, update, &self.queue)
            }
            WestonEvent::Parent { id, parent } => {
                let mut update = WindowUpdate::default();
                update.parent_id = Some(parent);
                self.reg.update_window_queued(id, update, &self.queue)
            }
            WestonEvent::Title { id, title } => {
                let mut update = WindowUpdate::default();
                update.title = Some(title);
                self.reg.update_window_queued(id, update, &self.queue)
            }
            WestonEvent::AppId { id, app_id } => {
                let mut update = WindowUpdate::default();
                update.app_id = Some(app_id);
                self.reg.update_window_queued(id, update, &self.queue)
            }
        }
    }
}

pub struct FakeWeston<A: WestonAdapter> {
    adapter: A,
    events: Vec<WestonEvent>,
}

impl<A: WestonAdapter> FakeWeston<A> {
    pub fn new(adapter: A) -> Self {
        Self { adapter, events: Vec::new() }
    }

    pub fn push(&mut self, event: WestonEvent) {
        self.events.push(event);
    }

    pub fn run(&mut self) -> Result<(), RegistryError> {
        for event in self.events.drain(..) {
            self.adapter.handle_event(event)?;
        }
        Ok(())
    }
}
