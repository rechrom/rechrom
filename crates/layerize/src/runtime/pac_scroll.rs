//! Page-local main-thread scroll callbacks installed by PaintArtifactCompositor.
//!
//! Chromium correspondence:
//! - cc/trees/property_tree.h:583-598
//! - cc/trees/property_tree.cc:2405-2429
//! - third_party/blink/renderer/platform/graphics/compositing/
//!   paint_artifact_compositor.cc:1173-1174
//!
//! The callback retains only a typed FIFO (Arc/Mutex/atomic so the main tree
//! that names it can be lent to the impl thread, S1B_PROXY_DESIGN.md B6). It does not retain PAC, Host,
//! PropertyTrees, a Page, or any layer/tile owner.

use crate::commit::{
    geometry::PointF,
    input_types::ElementId,
    main_effects::PacScrollEffect,
    property_tree::ScrollCallbacks,
    scroll_snap_data::TargetSnapAreaElementIds,
    scroll_source_type::ScrollSourceType,
};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    collections::VecDeque,
    sync::{Arc, Mutex, Weak},
};

#[derive(Clone, Default)]
pub struct PacScrollEffects(Arc<Mutex<VecDeque<PacScrollEffect>>>);

impl PacScrollEffects {
    pub fn Pop(&self) -> Option<PacScrollEffect> {
        self.0.lock().unwrap().pop_front()
    }

    pub fn IsEmpty(&self) -> bool {
        self.0.lock().unwrap().is_empty()
    }

    pub fn Push(&self, effect: PacScrollEffect) {
        self.0.lock().unwrap().push_back(effect)
    }
}

pub struct PacScrollCallbacks {
    effects: PacScrollEffects,
    sequence: AtomicU64,
}

impl PacScrollCallbacks {
    pub fn new(effects: PacScrollEffects) -> Arc<Self> {
        Arc::new(Self {
            effects,
            sequence: AtomicU64::new(0),
        })
    }

    pub fn Downgrade(callbacks: &Arc<Self>) -> Weak<dyn ScrollCallbacks> {
        let callbacks: Arc<dyn ScrollCallbacks> = callbacks.clone();
        Arc::downgrade(&callbacks)
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
        let sequence = self.sequence.load(Ordering::Relaxed).wrapping_add(1).max(1);
        self.sequence.store(sequence, Ordering::Relaxed);
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
