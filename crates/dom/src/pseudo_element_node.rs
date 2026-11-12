//! Retained PseudoElement node identity, separate from the author child list.
//! Source: core/dom/pseudo_element.cc:196-214,744-787;
//! core/dom/events/event_path.cc:42-57,107-128.
use super::*;
use layoutng_style::style::computed_style_constants::PseudoId;

#[derive(Clone, Copy, Debug)]
pub struct DOMPseudoElementFacts {
    pub pseudo_id: PseudoId,
    pub originating_element: usize,
}
impl DOMPseudoElementFacts {
    /// Source subclass virtuals: scroll_button_pseudo_element.h:26,
    /// scroll_marker_pseudo_element.h:24, interest_button_pseudo_element.h:20.
    pub fn HasActivationBehavior(self) -> bool {
        matches!(
            self.pseudo_id,
            PseudoId::kPseudoIdScrollMarker
                | PseudoId::kPseudoIdInterestButton
                | PseudoId::kPseudoIdScrollButtonBlockStart
                | PseudoId::kPseudoIdScrollButtonInlineStart
                | PseudoId::kPseudoIdScrollButtonInlineEnd
                | PseudoId::kPseudoIdScrollButtonBlockEnd
        )
    }
    pub fn SupportsHitTesting(self) -> bool {
        // The three CSSPseudoElement feature inputs in source744-787 are all
        // stable in runtime_enabled_features.json5:1830-1840,4920-4921.
        matches!(
            self.pseudo_id,
            PseudoId::kPseudoIdBackdrop
                | PseudoId::kPseudoIdBefore
                | PseudoId::kPseudoIdAfter
                | PseudoId::kPseudoIdMarker
                | PseudoId::kPseudoIdInterestButton
                | PseudoId::kPseudoIdOverscrollBackdrop
                | PseudoId::kPseudoIdScrollMarker
                | PseudoId::kPseudoIdScrollMarkerGroupBefore
                | PseudoId::kPseudoIdScrollMarkerGroupAfter
                | PseudoId::kPseudoIdScrollButtonBlockStart
                | PseudoId::kPseudoIdScrollButtonInlineStart
                | PseudoId::kPseudoIdScrollButtonInlineEnd
                | PseudoId::kPseudoIdScrollButtonBlockEnd
                | PseudoId::kPseudoIdViewTransition
                | PseudoId::kPseudoIdViewTransitionGroup
                | PseudoId::kPseudoIdViewTransitionImagePair
                | PseudoId::kPseudoIdViewTransitionOld
                | PseudoId::kPseudoIdViewTransitionNew
        )
    }
}
impl PersistentDocument {
    // Source: Node::kElementChildPseudoOrder and
    // LayoutTreeBuilderTraversal::FirstChild/NextSibling,
    // node.h:321-343; layout_tree_builder_traversal.cc:103-178,355-383.
    // kPseudoIdColumn is an indexed owner which is not yet admitted by this
    // DOM allocator; ordinary children occupy the explicit None position.
    const LAYOUT_TREE_BUILDER_PSEUDO_ORDER: [Option<PseudoId>; 18] = [
        Some(PseudoId::kPseudoIdScrollMarkerGroupBefore),
        Some(PseudoId::kPseudoIdMarker),
        Some(PseudoId::kPseudoIdScrollMarker),
        Some(PseudoId::kPseudoIdScrollButtonBlockStart),
        Some(PseudoId::kPseudoIdScrollButtonInlineStart),
        Some(PseudoId::kPseudoIdScrollButtonInlineEnd),
        Some(PseudoId::kPseudoIdScrollButtonBlockEnd),
        Some(PseudoId::kPseudoIdOverscrollAreaParent),
        Some(PseudoId::kPseudoIdCheckMark),
        Some(PseudoId::kPseudoIdBefore),
        None,
        Some(PseudoId::kPseudoIdAfter),
        Some(PseudoId::kPseudoIdExpandIcon),
        Some(PseudoId::kPseudoIdPickerIcon),
        Some(PseudoId::kPseudoIdInterestButton),
        Some(PseudoId::kPseudoIdScrollMarkerGroupAfter),
        Some(PseudoId::kPseudoIdSkeleton),
        Some(PseudoId::kPseudoIdViewTransition),
    ];

    fn LayoutTreeBuilderChildAtSlot(&self, parent: usize, slot: Option<PseudoId>) -> Option<usize> {
        match slot {
            None => self.FlatTreeChildren(parent).first().copied(),
            Some(PseudoId::kPseudoIdViewTransition) if self.DocumentElement() == Some(parent) => {
                None
            }
            Some(pseudo) => self.GeneratedPseudoElement(parent, pseudo),
        }
    }

    /// Native node order used by focus reading-flow traversal. This returns
    /// one live identity and never materializes a parallel child list.
    pub fn LayoutTreeBuilderFirstChild(&self, parent: usize) -> Option<usize> {
        Self::LAYOUT_TREE_BUILDER_PSEUDO_ORDER
            .into_iter()
            .find_map(|slot| self.LayoutTreeBuilderChildAtSlot(parent, slot))
    }

    pub fn LayoutTreeBuilderNextSibling(&self, node: usize) -> Option<usize> {
        if self.PseudoElementFacts(node).is_none() {
            if let Some(next) = self.FlatTreeNextSibling(node) {
                return Some(next);
            }
        }
        let parent = if self.PseudoElementFacts(node).is_some() {
            self.Node(node).Parent()
        } else {
            self.FlatTreeParent(node)
        }?;
        let current = self.PseudoElementFacts(node).map(|facts| facts.pseudo_id);
        let start = Self::LAYOUT_TREE_BUILDER_PSEUDO_ORDER
            .iter()
            .position(|slot| *slot == current)?;
        Self::LAYOUT_TREE_BUILDER_PSEUDO_ORDER[start + 1..]
            .iter()
            .copied()
            .find_map(|slot| self.LayoutTreeBuilderChildAtSlot(parent, slot))
    }

    pub fn PseudoElementFacts(&self, node: usize) -> Option<DOMPseudoElementFacts> {
        self.generated_pseudo_facts.get(&node).copied()
    }
    pub fn GeneratedPseudoElement(&self, parent: usize, kind: PseudoId) -> Option<usize> {
        self.generated_pseudo_elements
            .get(&parent)?
            .get(&kind)
            .copied()
    }
    /// The pseudo node is a DOM owner, never an author-tree child or a Layout
    /// alias. Its style is the same immutable owner subsequently sent to Layout.
    pub fn EnsureGeneratedPseudoElement(
        &mut self,
        parent: usize,
        kind: PseudoId,
        style: Persistent<StyleComputedStyle>,
    ) -> usize {
        let node = if let Some(node) = self.GeneratedPseudoElement(parent, kind) {
            node
        } else {
            let node = self.Allocate(
                DOMNodeType::kElement,
                self.Node(parent).Namespace(),
                String::from("::pseudo"),
                String::new(),
                Vec::new(),
            );
            self.arena[node].parent = Some(parent);
            self.generated_pseudo_elements
                .entry(parent)
                .or_default()
                .insert(kind, node);
            self.arena[node].is_in_canvas_subtree = self.IsInCanvasSubtree(parent);
            let origin = self
                .PseudoElementFacts(parent)
                .map_or(parent, |p| p.originating_element);
            self.generated_pseudo_facts.insert(
                node,
                DOMPseudoElementFacts {
                    pseudo_id: kind,
                    originating_element: origin,
                },
            );
            self.ultimate_originating_elements.insert(node, origin);
            if matches!(
                kind,
                PseudoId::kPseudoIdScrollMarkerGroupBefore
                    | PseudoId::kPseudoIdScrollMarkerGroupAfter
            ) {
                // ScrollMarkerGroupPseudoElement constructor, source.cc:47-52.
                self.arena[node].scroll_marker_group_data = Some(Box::default());
            }
            node
        };
        let mut resolved = ResolvedNodeStyle::default();
        resolved.computed_style_owner = style;
        self.arena[node].resolved_style.style = Some(std::sync::Arc::new(resolved));
        node
    }
    pub(crate) fn DetachGeneratedPseudoElement(&mut self, parent: usize, kind: PseudoId) {
        if let Some(node) = self
            .generated_pseudo_elements
            .get_mut(&parent)
            .and_then(|pseudos| pseudos.remove(&kind))
        {
            self.DetachGeneratedPseudoElement(node, PseudoId::kPseudoIdMarker);
            self.arena[node].parent = None;
            self.SetIsInCanvasSubtree(node, false);
        }
    }
    fn BindDefaultMarker(&mut self, parent: usize, style: &StyleComputedStyle) {
        if matches!(
            style.Display(),
            foundation::EDisplay::kListItem
                | foundation::EDisplay::kInlineListItem
                | foundation::EDisplay::kFlowRootListItem
                | foundation::EDisplay::kInlineFlowRootListItem
        ) {
            // Move the existing default marker style creation to its DOM
            // PseudoElement owner so Layout receives this same style pointer.
            let mut builder =
                layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(style);
            builder.SetDisplay(foundation::EDisplay::kInline);
            builder.SetStyleType(PseudoId::kPseudoIdMarker);
            let owner = Persistent::from_ptr(builder.TakeStyle().cast_mut());
            self.EnsureGeneratedPseudoElement(parent, PseudoId::kPseudoIdMarker, owner);
        } else {
            self.DetachGeneratedPseudoElement(parent, PseudoId::kPseudoIdMarker)
        }
    }
    fn BindGeneratedPseudoStyle(
        &mut self,
        parent: usize,
        kind: PseudoId,
        pseudo: Option<&PseudoElement>,
    ) {
        let Some(pseudo) = pseudo else {
            self.DetachGeneratedPseudoElement(parent, kind);
            return;
        };
        if pseudo.computed_style_owner.Get().is_null() {
            self.DetachGeneratedPseudoElement(parent, kind);
            return;
        }
        let node =
            self.EnsureGeneratedPseudoElement(parent, kind, pseudo.computed_style_owner.clone());
        if let Some(marker) = pseudo.marker.as_deref() {
            self.BindGeneratedPseudoStyle(node, PseudoId::kPseudoIdMarker, Some(marker));
        } else if let Some(style) = unsafe { pseudo.computed_style_owner.Get().as_ref() } {
            self.BindDefaultMarker(node, style);
        }
    }
    // Element::UpdatePseudoElement(kOverscrollAreaParent) ignores parent
    // ComputedStyle pseudo bits; source tracker membership decides existence.
    pub fn SetOverscrollAreaParentStyle(
        &mut self,
        area: usize,
        style: Option<Persistent<StyleComputedStyle>>,
    ) {
        let kind = PseudoId::kPseudoIdOverscrollAreaParent;
        let old = self.GeneratedPseudoElement(area, kind);
        let same = match (old, style.as_ref()) {
            (Some(node), Some(style)) => self
                .ResolvedStyleFor(node)
                .and_then(|old| old.ComputedStyle())
                .zip(unsafe { style.Get().as_ref() })
                .is_some_and(|(old, new)| old == new),
            (None, None) => true,
            _ => false,
        };
        if same {
            return;
        }
        if let Some(style) = style {
            self.EnsureGeneratedPseudoElement(area, kind, style);
        } else {
            self.DetachGeneratedPseudoElement(area, kind);
        }
        let container = self
            .GetOverscrollContainer(area)
            .or_else(|| self.Node(area).Parent());
        if let Some(container) = container {
            let id = self.Node(container).Id();
            self.StyleStateMut().layout_reattach_nodes.insert(id);
            self.RecordLayoutInputChange(container);
        }
    }
    pub(super) fn BindGeneratedPseudoElements(&mut self, parent: usize, style: &ResolvedNodeStyle) {
        self.BindGeneratedPseudoStyle(parent, PseudoId::kPseudoIdBefore, style.before.as_ref());
        self.BindGeneratedPseudoStyle(parent, PseudoId::kPseudoIdAfter, style.after.as_ref());
        self.BindGeneratedPseudoStyle(
            parent,
            PseudoId::kPseudoIdFirstLetter,
            style.first_letter.as_ref(),
        );
        if let Some(marker) = style.marker.as_ref() {
            self.BindGeneratedPseudoStyle(parent, PseudoId::kPseudoIdMarker, Some(marker));
        } else if let Some(style) = style.ComputedStyle() {
            self.BindDefaultMarker(parent, style);
        }
    }
}
