//! Retained ScrollMarkerGroupPseudoElement::scroll_marker_group_data_.
//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15:
//! core/dom/scroll_marker_group_data.h:87-208; .cc:424-448,616-621;
//! core/dom/scroll_marker_group_pseudo_element.cc:47-52,146-166,208-219.
use super::*;
use layoutng_style::style::computed_style_constants::PseudoId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum DOMScrollMarkerInvalidation {
    #[default]
    Clean,
    NeedsActiveMarkerUpdate,
    NeedsFullUpdate,
}

/// Small read-only projection of the one retained DOM owner. In particular,
/// pending is not selected until the source post-layout snapshot applies it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DOMScrollMarkerGroupState {
    pub selected: Option<u64>,
    pub pending_selected: Option<u64>,
    pub selected_is_pinned: bool,
    pub apply_snap_alignment: bool,
    pub invalidation: DOMScrollMarkerInvalidation,
}

#[derive(Clone, Debug, Default)]
pub(super) struct ScrollMarkerGroupData {
    selected: Option<usize>,
    pending_selected: Option<usize>,
    selected_is_pinned: bool,
    apply_snap_alignment: bool,
    invalidation: DOMScrollMarkerInvalidation,
}
impl ScrollMarkerGroupData {
    // source scroll_marker_group_data.cc:437-444.
    fn SetPendingSelectedMarker(&mut self, marker: Option<usize>, apply_snap_alignment: bool) {
        self.invalidation = self
            .invalidation
            .max(DOMScrollMarkerInvalidation::NeedsActiveMarkerUpdate);
        self.pending_selected = marker;
        self.apply_snap_alignment = apply_snap_alignment;
    }
}
impl PersistentDocument {
    /// PaintLayerScrollableArea::GetScrollMarkerGroup, source.cc:3630-3650.
    /// GeneratedPseudoElement is the canonical identity lookup; CSS style bits
    /// cannot establish the existence of a ScrollMarkerGroupPseudoElement.
    pub fn GetScrollMarkerGroupForScroller(&self, node_id: u64) -> Option<u64> {
        let node = self.FindNodeById(node_id)?;
        let element = match self.arena[node].node_type {
            DOMNodeType::kElement => node,
            DOMNodeType::kDocument => self.arena[node]
                .children
                .iter()
                .copied()
                .find(|&child| self.arena[child].node_type == DOMNodeType::kElement)?,
            _ => return None,
        };
        // An existing before wins even if its subtype check were to fail.
        let group = self
            .GeneratedPseudoElement(element, PseudoId::kPseudoIdScrollMarkerGroupBefore)
            .or_else(|| {
                self.GeneratedPseudoElement(element, PseudoId::kPseudoIdScrollMarkerGroupAfter)
            })?;
        self.arena[group].scroll_marker_group_data.as_ref()?;
        Some(self.arena[group].id)
    }

    pub fn ScrollMarkerGroupState(&self, group_id: u64) -> Option<DOMScrollMarkerGroupState> {
        let group = self.FindNodeById(group_id)?;
        let data = self.arena[group].scroll_marker_group_data.as_ref()?;
        Some(DOMScrollMarkerGroupState {
            selected: data.selected.map(|marker| self.arena[marker].id),
            pending_selected: data.pending_selected.map(|marker| self.arena[marker].id),
            selected_is_pinned: data.selected_is_pinned,
            apply_snap_alignment: data.apply_snap_alignment,
            invalidation: data.invalidation,
        })
    }

    // ScrollMarkerGroupPseudoElement::SetSelected, cc:146-150, delegates to
    // data::MaybeSetPendingSelectedMarker, cc:424-435; no snapshot is applied.
    pub fn SetPendingSelectedScrollMarker(
        &mut self,
        group_id: u64,
        marker_id: Option<u64>,
        apply_snap_alignment: bool,
    ) {
        let group = self
            .FindNodeById(group_id)
            .expect("native scroll-marker group must exist");
        let marker = marker_id.map(|id| {
            self.FindNodeById(id)
                .expect("native scroll marker must exist")
        });
        if let Some(marker) = marker {
            assert!(self
                .PseudoElementFacts(marker)
                .is_some_and(|facts| facts.pseudo_id == PseudoId::kPseudoIdScrollMarker));
        }
        let data = self.arena[group]
            .scroll_marker_group_data
            .as_mut()
            .expect("native ScrollMarkerGroupPseudoElement required");
        if !data.selected_is_pinned {
            data.SetPendingSelectedMarker(marker, apply_snap_alignment);
        }
    }

    // source group.cc:156-159, data.h:127-130. Pin always writes pending first.
    pub fn PinSelectedMarker(&mut self, group_id: u64, marker_id: Option<u64>) {
        let group = self
            .FindNodeById(group_id)
            .expect("native scroll-marker group must exist");
        let marker = marker_id.map(|id| {
            self.FindNodeById(id)
                .expect("native scroll marker must exist")
        });
        if let Some(marker) = marker {
            assert!(self
                .PseudoElementFacts(marker)
                .is_some_and(|facts| facts.pseudo_id == PseudoId::kPseudoIdScrollMarker));
        }
        let data = self.arena[group]
            .scroll_marker_group_data
            .as_mut()
            .expect("native ScrollMarkerGroupPseudoElement required");
        data.SetPendingSelectedMarker(marker, true);
        data.selected_is_pinned = true;
    }

    // source group.cc:161-163; data.h:132.
    pub fn UnPinSelectedMarker(&mut self, group_id: u64) {
        let group = self
            .FindNodeById(group_id)
            .expect("native scroll-marker group must exist");
        self.arena[group]
            .scroll_marker_group_data
            .as_mut()
            .expect("native ScrollMarkerGroupPseudoElement required")
            .selected_is_pinned = false;
    }

    /// source group.cc:208-219 and data.cc:616-621. The caller supplies facts
    /// from the originating element's real native LayoutObject. This only
    /// invalidates; ChooseMarkerRecursively/ApplyPendingScrollMarker run later
    /// through PostLayoutSnapshotClient (not yet translated here).
    pub fn UpdateScrollMarkers(
        &mut self,
        group_id: u64,
        scroller_has_layout_box: bool,
        scroller_is_scroll_container: bool,
    ) -> bool {
        let group = self
            .FindNodeById(group_id)
            .expect("native scroll-marker group must exist");
        let origin = self
            .PseudoElementFacts(group)
            .expect("native pseudo required")
            .originating_element;
        let is_document_element = self.arena[origin].parent.is_some_and(|parent| {
            self.arena[parent].node_type == DOMNodeType::kDocument
                && self.arena[parent]
                    .children
                    .iter()
                    .copied()
                    .find(|&child| self.arena[child].node_type == DOMNodeType::kElement)
                    == Some(origin)
        });
        if !scroller_has_layout_box || (!scroller_is_scroll_container && !is_document_element) {
            return false;
        }
        let data = self.arena[group]
            .scroll_marker_group_data
            .as_mut()
            .expect("native ScrollMarkerGroupPseudoElement required");
        if !data.selected_is_pinned {
            data.invalidation = DOMScrollMarkerInvalidation::NeedsFullUpdate;
        }
        true
    }
}
