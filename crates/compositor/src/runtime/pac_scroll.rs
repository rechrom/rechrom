//! Page-local main-thread scroll callbacks installed by PaintArtifactCompositor.
//!
//! Chromium correspondence:
//! - cc/trees/property_tree.h:583-598
//! - cc/trees/property_tree.cc:2405-2429
//! - third_party/blink/renderer/platform/graphics/compositing/
//!   paint_artifact_compositor.cc:1173-1174
//!
//! The callback retains only a typed FIFO. It does not retain PAC, Host,
//! PropertyTrees, a Page, or any layer/tile owner.

use crate::{
    adapter::{geometry::PointF, input_types::ElementId},
    cc::{
        input::scroll_snap_data::TargetSnapAreaElementIds,
        trees::{property_tree::ScrollCallbacks, scroll_source_type::ScrollSourceType},
    },
};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    rc::{Rc, Weak},
};

#[derive(Clone, Debug)]
pub(crate) enum PacScrollEffect {
    DidCompositorScroll {
        sequence: u64,
        element: ElementId,
        offset: PointF,
        source: ScrollSourceType,
        snap: Option<TargetSnapAreaElementIds>,
    },
    DidChangeScrollbarsHidden {
        element: ElementId,
        hidden: bool,
    },
}

#[derive(Clone, Default)]
pub(crate) struct PacScrollEffects(Rc<RefCell<VecDeque<PacScrollEffect>>>);

impl PacScrollEffects {
    pub(crate) fn Pop(&self) -> Option<PacScrollEffect> {
        self.0.borrow_mut().pop_front()
    }

    pub(crate) fn IsEmpty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    fn Push(&self, effect: PacScrollEffect) {
        self.0.borrow_mut().push_back(effect)
    }
}

pub(crate) struct PacScrollCallbacks {
    effects: PacScrollEffects,
    sequence: Cell<u64>,
}

impl PacScrollCallbacks {
    pub(crate) fn new(effects: PacScrollEffects) -> Rc<Self> {
        Rc::new(Self {
            effects,
            sequence: Cell::new(0),
        })
    }

    pub(crate) fn Downgrade(callbacks: &Rc<Self>) -> Weak<dyn ScrollCallbacks> {
        let callbacks: Rc<dyn ScrollCallbacks> = callbacks.clone();
        Rc::downgrade(&callbacks)
    }
}

impl ScrollCallbacks for PacScrollCallbacks {
    fn DidCompositorScroll(
        &self,
        element: ElementId,
        offset: &PointF,
        source: ScrollSourceType,
        snap: &Option<TargetSnapAreaElementIds>,
    ) {
        let sequence = self.sequence.get().wrapping_add(1).max(1);
        self.sequence.set(sequence);
        self.effects.Push(PacScrollEffect::DidCompositorScroll {
            sequence,
            element,
            offset: *offset,
            source,
            snap: snap.clone(),
        })
    }

    fn DidChangeScrollbarsHidden(&self, element: ElementId, hidden: bool) {
        self.effects
            .Push(PacScrollEffect::DidChangeScrollbarsHidden { element, hidden })
    }
}
