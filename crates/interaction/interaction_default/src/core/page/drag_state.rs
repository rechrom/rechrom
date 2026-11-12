//! Chromium core/page/drag_state.h:37-53.
//! The drag source and DataTransfer are shared with DragEvent dispatch. Neither
//! is projected into a node id or recreated for the next mouse event.
use interaction_dispatch::core::{
    clipboard::data_transfer::DataTransfer, dom::events::event::EventTargetRef,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub use super::drag_actions::DragSourceAction;

#[derive(Default)]
pub struct DragState {
    pub drag_src: RefCell<Option<EventTargetRef>>,
    pub drag_type: Cell<DragSourceAction>,
    pub drag_data_transfer: RefCell<Option<Rc<DataTransfer>>>,
}
