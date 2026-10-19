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
    cell::RefCell,
    collections::VecDeque,
    rc::{Rc, Weak},
};

#[derive(Clone, Debug)]
pub(crate) enum NativePacScrollEffect {
    DidCompositorScroll {
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
pub(crate) struct NativePacScrollEffects(Rc<RefCell<VecDeque<NativePacScrollEffect>>>);

impl NativePacScrollEffects {
    pub(crate) fn Pop(&self) -> Option<NativePacScrollEffect> {
        self.0.borrow_mut().pop_front()
    }

    pub(crate) fn IsEmpty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    fn Push(&self, effect: NativePacScrollEffect) {
        self.0.borrow_mut().push_back(effect)
    }
}

pub(crate) struct NativePacScrollCallbacks {
    effects: NativePacScrollEffects,
}

impl NativePacScrollCallbacks {
    pub(crate) fn new(effects: NativePacScrollEffects) -> Rc<Self> {
        Rc::new(Self { effects })
    }

    pub(crate) fn Downgrade(callbacks: &Rc<Self>) -> Weak<dyn ScrollCallbacks> {
        let callbacks: Rc<dyn ScrollCallbacks> = callbacks.clone();
        Rc::downgrade(&callbacks)
    }
}

impl ScrollCallbacks for NativePacScrollCallbacks {
    fn DidCompositorScroll(
        &self,
        element: ElementId,
        offset: &PointF,
        source: ScrollSourceType,
        snap: &Option<TargetSnapAreaElementIds>,
    ) {
        self.effects
            .Push(NativePacScrollEffect::DidCompositorScroll {
                element,
                offset: *offset,
                source,
                snap: snap.clone(),
            })
    }

    fn DidChangeScrollbarsHidden(&self, element: ElementId, hidden: bool) {
        self.effects
            .Push(NativePacScrollEffect::DidChangeScrollbarsHidden { element, hidden })
    }
}
