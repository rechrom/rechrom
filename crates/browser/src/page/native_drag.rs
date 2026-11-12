//! RenderWidget drag-source completion assembly. The source EventHandler owns
//! dragend, touch cleanup and DataTransfer policy; Page lends its live runtime.
use super::*;
use interaction::{
    dispatch::core::clipboard::data_transfer::DragOperation,
    input::public::common::input::{
        web_input_event::Type,
        web_mouse_event::WebMouseEvent,
        web_pointer_properties::{Button, PointF},
    },
};
impl Page {
    /// WebFrameWidgetImpl::DragSourceEndedAt (cc:752-780): completion belongs to
    /// this Page and pointer, even if another tab becomes active meanwhile.
    pub fn EndSystemDrag(
        &mut self,
        pointer_id: i32,
        position: Option<(f32, f32)>,
        screen_position: Option<(f32, f32)>,
        completed_at: std::time::Instant,
        operation: DragOperation,
    ) -> io::Result<()> {
        let mut heap_scope = foundation::LayoutHeapScope::new();
        heap_scope.DeferCollection();
        self.FlushFrameInputs()?;
        self.state.EnsureLayoutOutput();
        // wl_data_source does not disclose final local/global drop coordinates.
        // Preserve the live MouseEventManager's last observed positions when
        // the native host explicitly reports those values as unavailable.
        let mouse = &self.interaction.SourceInputOwners().mouse;
        let position = position
            .map(|(x, y)| PointF::new(x, y))
            .unwrap_or_else(|| mouse.LastKnownMousePositionInViewport());
        let screen_position = screen_position
            .map(|(x, y)| PointF::new(x, y))
            .unwrap_or_else(|| mouse.LastKnownMouseScreenPosition());
        let mut event = WebMouseEvent::new(
            Type::kMouseMove,
            position,
            screen_position,
            Button::kLeft,
            0,
            0,
            self.state
                .rendering
                .borrow()
                .InputTimeForInstant(completed_at),
            interaction::input::ui::base::mojom::menu_source_type::MenuSourceType::kNone,
            pointer_id,
        );
        event.SetFrameScale(1.);
        if let Some(scripts) = &mut self.scripts {
            scripts.RunInteractionTask(popup_tasks::PageInteractionTask::SystemDragEnd {
                event: Arc::new(event),
                operation,
            })?;
        } else {
            self.interaction
                .DragSourceEndedAt(&self.state.document.Handle(), &event, operation);
        }
        self.UpdateFrameAndStartVisibleDeferredImages()?;
        self.UpdateCursor();
        Ok(())
    }
}

impl Page {
    /// Renderer target entry for native or DevTools hosts. The action travels
    /// through this Page's ordinary script task scope, so drag listeners and
    /// synchronous default actions borrow its current runtime and live owners.
    pub fn HandleTargetDrag(
        &mut self,
        event: WebMouseEvent,
        action: interaction::default::core::page::drag_controller::DragTargetAction,
    ) -> io::Result<interaction::default::core::page::drag_controller::DragTargetOperation> {
        let mut heap_scope = foundation::LayoutHeapScope::new();
        heap_scope.DeferCollection();
        self.FlushFrameInputs()?;
        self.state.EnsureLayoutOutput();
        self.EnsureInteractionScrollOwners();
        let result = if let Some(scripts) = &mut self.scripts {
            let result = Rc::new(Cell::new(None));
            scripts.RunInteractionTask(popup_tasks::PageInteractionTask::SystemDragTarget {
                event: Arc::new(event),
                action,
                result: result.clone(),
            })?;
            result
                .take()
                .ok_or_else(|| io::Error::other("drag target task did not complete"))?
        } else {
            self.interaction
                .HandleTargetDrag(&self.state.document.Handle(), &event, &action)
                .map_err(io::Error::other)?
        };
        self.UpdateFrameAndStartVisibleDeferredImages()?;
        self.UpdateCursor();
        Ok(result)
    }
}

impl Page {
    pub fn TargetSystemDrag(
        &mut self,
        action: interaction::default::core::page::drag_controller::DragTargetAction,
        position: (f32, f32),
        screen: Option<(f32, f32)>,
        modifiers: i32,
        observed_at: std::time::Instant,
    ) -> io::Result<interaction::default::core::page::drag_controller::DragTargetOperation> {
        let point = PointF::new(position.0, position.1);
        let screen = screen.map(|p| PointF::new(p.0, p.1)).unwrap_or_else(|| {
            self.interaction
                .SourceInputOwners()
                .mouse
                .LastKnownMouseScreenPosition()
        });
        let mut event = WebMouseEvent::new(
            Type::kMouseMove,
            point,
            screen,
            Button::kLeft,
            0,
            modifiers,
            self.state
                .rendering
                .borrow()
                .InputTimeForInstant(observed_at),
            interaction::input::ui::base::mojom::menu_source_type::MenuSourceType::kNone,
            0,
        );
        event.SetFrameScale(1.);
        self.HandleTargetDrag(event, action)
    }
}

impl Page {
    pub fn DirectoryFilesChosen(&mut self, result: DirectoryEnumerationResult) -> io::Result<()> {
        let request = result.request;
        let current = self
            .state
            .directory_drop_requests
            .borrow()
            .get(&request.target)
            .copied();
        if current != Some(request.request_id) {
            return Ok(());
        }
        self.state
            .directory_drop_requests
            .borrow_mut()
            .remove(&request.target);
        if self
            .state
            .document
            .borrow()
            .GetDocument()
            .RootHandle()
            .Identity()
            != request.document_identity
        {
            return Ok(());
        }
        if let Some(scripts) = &mut self.scripts {
            scripts.RunInteractionTask(popup_tasks::PageInteractionTask::DirectoryFilesChosen {
                target: request.target,
                files: result.files,
            })?;
        } else {
            self.interaction.DirectoryFilesChosen(
                &self.state.document.Handle(),
                request.target,
                result.files,
            );
        }
        self.UpdateFrameAndStartVisibleDeferredImages()
    }
}
