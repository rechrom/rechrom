#![allow(non_snake_case)]

use std::cell::RefCell;

use layoutng_assembly::internal::layout_input::FloatSide;

use crate::box_fragment_painter::BoxFragmentPainter;
use crate::geometry_mapper::{MapRectFromRoot, MapRectToRoot};
use crate::paint_context::PaintContext;
use crate::paint_engine::PaintPhase;
use crate::paint_info::PaintInfo;
use crate::pre_paint_tree_walk::PaintTreeNode;
use crate::PaintRect;

// cpp: paint/paint_layer_painter.cc:12-14
fn ZIndex(node: &PaintTreeNode<'_>) -> i32 {
    node.stacking_level
}

// cpp: paint/paint_layer_painter.cc:16-19
fn IsFloat(node: &PaintTreeNode<'_>) -> bool {
    let fragment = node
        .fragment
        .as_deref()
        .expect("PrePaint node must own a fragment");
    fragment.paint.has_source && fragment.paint.floating != FloatSide::kNone
}

struct CollectedLayer<'n, 'f> {
    layer: &'n PaintTreeNode<'f>,
    property_ancestors: Vec<&'n PaintTreeNode<'f>>,
}

// cpp: paint/paint_layer_painter.cc:21-34
fn HasEscapingChildLayer(parent: &PaintTreeNode<'_>) -> bool {
    for child_owner in &parent.children {
        let child = child_owner.as_ref();
        if child.is_paint_layer {
            if child.escapes_ancestor_geometry {
                return true;
            }
            if child.is_stacking_context {
                continue;
            }
        }
        if HasEscapingChildLayer(child) {
            return true;
        }
    }
    false
}

// cpp: paint/paint_layer_painter.h:12-38
pub struct PaintLayerPainter<'n, 'f, 'c, 'o> {
    layer: &'n PaintTreeNode<'f>,
    context: &'c RefCell<PaintContext<'o>>,
    inherited_geometry: Vec<&'n PaintTreeNode<'f>>,
}

impl<'n, 'f, 'c, 'o> PaintLayerPainter<'n, 'f, 'c, 'o> {
    // cpp: paint/paint_layer_painter.h:14-22
    pub fn new(
        layer: &'n PaintTreeNode<'f>,
        context: &'c RefCell<PaintContext<'o>>,
        inherited_geometry: Vec<&'n PaintTreeNode<'f>>,
    ) -> Self {
        Self {
            layer,
            context,
            inherited_geometry,
        }
    }

    // cpp: paint/paint_layer_painter.h:25
    // cpp: paint/paint_layer_painter.cc:38-56
    fn UsesInheritedGeometry(&self, ancestor: &PaintTreeNode<'f>) -> bool {
        if !self.layer.escapes_ancestor_geometry {
            return true;
        }
        let logical_position = self
            .inherited_geometry
            .iter()
            .position(|candidate| std::ptr::eq(*candidate, ancestor));
        let Some(logical_position) = logical_position else {
            return false;
        };
        self.inherited_geometry[logical_position..]
            .iter()
            .any(|candidate| {
                let pointer = *candidate as *const PaintTreeNode<'f>;
                self.layer.physical_ancestors.contains(&pointer)
            })
    }

    // cpp: paint/paint_layer_painter.h:26-27
    // cpp: paint/paint_layer_painter.cc:58-67
    fn InheritedClipInLayerSpace(&self, ancestor: &PaintTreeNode<'f>) -> Option<PaintRect> {
        let local_clip = ancestor.local_clip?;
        let Some(root_clip) = MapRectToRoot(local_clip, &ancestor.transforms) else {
            return Some(PaintRect::default());
        };
        Some(MapRectFromRoot(root_clip, &self.layer.transforms).unwrap_or_default())
    }

    // cpp: paint/paint_layer_painter.h:22
    // cpp: paint/paint_layer_painter.cc:69-71
    pub fn Paint(&self) {
        self.PaintInternal(true);
    }

    // cpp: paint/paint_layer_painter.h:28
    // cpp: paint/paint_layer_painter.cc:73-129
    fn PaintInternal(&self, include_descendant_layers: bool) {
        if self
            .layer
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment")
            .paint
            .hidden
        {
            return;
        }
        let _layer_chunk_properties = crate::paint_context::ScopedPaintChunkProperties::layer(
            self.context,
            self.layer,
            crate::display_item_id::DisplayItemIdType::kLayerChunk,
        );
        let effect_contains_escaping_child =
            !self.layer.escapes_ancestor_geometry && HasEscapingChildLayer(self.layer);
        for ancestor in &self.inherited_geometry {
            self.context.borrow_mut().BeginGeometry(ancestor);
            self.context.borrow_mut().BeginEffects(ancestor);
            if !effect_contains_escaping_child && self.UsesInheritedGeometry(ancestor) {
                self.context
                    .borrow_mut()
                    .BeginEscapableGeometry(ancestor, None);
            }
        }
        self.context.borrow_mut().BeginGeometry(self.layer);
        self.context.borrow_mut().BeginEffects(self.layer);
        if effect_contains_escaping_child {
            for ancestor in &self.inherited_geometry {
                if self.UsesInheritedGeometry(ancestor) {
                    self.context
                        .borrow_mut()
                        .BeginEscapableGeometry(ancestor, self.InheritedClipInLayerSpace(ancestor));
                }
            }
        }

        BoxFragmentPainter::PaintFragment(
            self.layer,
            &PaintInfo::new(self.context, PaintPhase::kSelfBlockBackgroundOnly),
        );
        self.context
            .borrow_mut()
            .BeginEscapableGeometry(self.layer, None);
        // The immutable paint tree has the same descendants in both z-order
        // phases. Collect/sort once, then consume each group at its original point.
        let children = if include_descendant_layers {
            self.CollectLayerChildren()
        } else {
            Vec::new()
        };
        let negative_count = children.partition_point(|child| ZIndex(child.layer) < 0);
        let mut children = children.into_iter();
        if include_descendant_layers {
            self.PaintLayerChildren(children.by_ref().take(negative_count));
        }
        {
            let _foreground_properties = crate::paint_context::ScopedPaintChunkProperties::layer(
                self.context,
                self.layer,
                crate::display_item_id::DisplayItemIdType::kLayerChunkForeground,
            );
            self.PaintForegroundPhases();
        }
        BoxFragmentPainter::PaintFragment(
            self.layer,
            &PaintInfo::new(self.context, PaintPhase::kSelfOutlineOnly),
        );
        if include_descendant_layers {
            self.PaintLayerChildren(children);
        }

        self.context.borrow_mut().EndEscapableGeometry(self.layer);
        if effect_contains_escaping_child {
            for ancestor in self.inherited_geometry.iter().rev() {
                if self.UsesInheritedGeometry(ancestor) {
                    self.context.borrow_mut().EndEscapableGeometry(ancestor);
                }
            }
        }
        // Mask's parent is the content Effect, outside a separate Filter node.
        // Finish the flat filter before recording the source Mask drawing, as
        // PaintFragmentWithPhase(kMask) uses the Mask child's property state.
        self.context.borrow_mut().EndFilter(self.layer);
        self.context.borrow_mut().EndCssClip(self.layer);
        self.context.borrow_mut().EndMaskClip(self.layer);
        if self.layer.applies_mask {
            BoxFragmentPainter::PaintFragment(
                self.layer,
                &PaintInfo::new(self.context, PaintPhase::kMask),
            );
        }
        self.context.borrow_mut().EndEffectsAfterFilter(self.layer);
        self.context.borrow_mut().EndGeometry(self.layer);
        for ancestor in self.inherited_geometry.iter().rev() {
            if !effect_contains_escaping_child && self.UsesInheritedGeometry(ancestor) {
                self.context.borrow_mut().EndEscapableGeometry(ancestor);
            }
            self.context.borrow_mut().EndEffects(ancestor);
            self.context.borrow_mut().EndGeometry(ancestor);
        }
    }

    // cpp: paint/paint_layer_painter.h:33
    // cpp: paint/paint_layer_painter.cc:131-138
    fn SuspendGeometryForEscapingChild(&self) {
        if !self.layer.is_root {
            self.context.borrow_mut().EndEscapableGeometry(self.layer);
        }
        for ancestor in self.inherited_geometry.iter().rev() {
            self.context.borrow_mut().EndEscapableGeometry(ancestor);
        }
    }

    // cpp: paint/paint_layer_painter.h:34
    // cpp: paint/paint_layer_painter.cc:140-146
    fn ResumeGeometryAfterEscapingChild(&self) {
        for ancestor in &self.inherited_geometry {
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(ancestor, self.InheritedClipInLayerSpace(ancestor));
        }
        if !self.layer.is_root {
            self.context
                .borrow_mut()
                .BeginEscapableGeometry(self.layer, None);
        }
    }

    // cpp: paint/paint_layer_painter.h:30
    // cpp: paint/paint_layer_painter.cc:148-155
    fn PaintForegroundPhases(&self) {
        self.PaintWithPhase(
            self.layer,
            &PaintInfo::new(self.context, PaintPhase::kDescendantBlockBackgroundsOnly),
        );
        self.PaintWithPhase(
            self.layer,
            &PaintInfo::new(self.context, PaintPhase::kFloat),
        );
        self.PaintWithPhase(
            self.layer,
            &PaintInfo::new(self.context, PaintPhase::kForeground),
        );
        self.PaintWithPhase(
            self.layer,
            &PaintInfo::new(self.context, PaintPhase::kDescendantOutlinesOnly),
        );
        self.PaintWithPhase(
            self.layer,
            &PaintInfo::new(self.context, PaintPhase::kOverlayOverflowControls),
        );
    }

    // cpp: paint/paint_layer_painter.h:29
    // cpp: paint/paint_layer_painter.cc:157-230
    fn PaintWithPhase(&self, node: &PaintTreeNode<'f>, paint_info: &PaintInfo<'c, 'o>) {
        if node
            .fragment
            .as_deref()
            .expect("PrePaint node must own a fragment")
            .paint
            .hidden
        {
            return;
        }
        let phase = paint_info.phase;
        if matches!(phase, PaintPhase::kFloat) && !node.needs_float_phase
            || matches!(
                phase,
                PaintPhase::kOutline | PaintPhase::kDescendantOutlinesOnly
            ) && !node.needs_outline_phase
            || matches!(phase, PaintPhase::kOverlayOverflowControls)
                && !node.needs_overflow_controls_phase
        {
            return;
        }
        let enter_node = !std::ptr::eq(node, self.layer);
        if !enter_node && phase == PaintPhase::kForeground {
            BoxFragmentPainter::PaintFragment(node, paint_info);
        }
        let paint_self_outside_clip = enter_node
            && node.applies_overflow_clip
            && !node.is_paint_layer
            && matches!(
                phase,
                PaintPhase::kBlockBackground
                    | PaintPhase::kDescendantBlockBackgroundsOnly
                    | PaintPhase::kOutline
                    | PaintPhase::kDescendantOutlinesOnly
            );
        if enter_node {
            if paint_self_outside_clip {
                // The box's own background is outside its overflow clip, but
                // it is still inside the node's transform, clip-path and
                // effect. Keep the same ordering as Blink's property-tree
                // conversion: outer geometry/effect, self paint, then the
                // escapable overflow clip used by descendants.
                let mut context = self.context.borrow_mut();
                context.BeginGeometry(node);
                context.BeginEffects(node);
                drop(context);
                BoxFragmentPainter::PaintFragment(node, paint_info);
                self.context.borrow_mut().BeginEscapableGeometry(node, None);
            } else {
                self.context.borrow_mut().BeginNode(node);
                BoxFragmentPainter::PaintFragment(node, paint_info);
            }
        }

        let _contents_properties =
            crate::paint_context::ScopedPaintChunkProperties::contents(self.context, node, phase);

        for child_owner in &node.children {
            let child = child_owner.as_ref();
            if child.is_paint_layer {
                continue;
            }
            let is_atomic = child
                .fragment
                .as_deref()
                .expect("PrePaint node must own a fragment")
                .paint
                .painted_atomically;
            match phase {
                PaintPhase::kDescendantBlockBackgroundsOnly | PaintPhase::kBlockBackground => {
                    if !IsFloat(child) && !is_atomic {
                        let mut child_info = paint_info.ForDescendants();
                        child_info.phase = PaintPhase::kBlockBackground;
                        self.PaintWithPhase(child, &child_info);
                    }
                }
                PaintPhase::kFloat => {
                    if IsFloat(child) {
                        self.PaintAllPhasesAtomically(child);
                    } else if !is_atomic {
                        self.PaintWithPhase(child, &paint_info.ForDescendants());
                    }
                }
                PaintPhase::kForeground => {
                    if !IsFloat(child) && is_atomic {
                        self.PaintAllPhasesAtomically(child);
                    } else if !IsFloat(child) {
                        self.PaintWithPhase(child, &paint_info.ForDescendants());
                    }
                }
                PaintPhase::kDescendantOutlinesOnly | PaintPhase::kOutline => {
                    if !is_atomic {
                        let mut child_info = paint_info.ForDescendants();
                        child_info.phase = PaintPhase::kOutline;
                        self.PaintWithPhase(child, &child_info);
                    }
                }
                PaintPhase::kOverlayOverflowControls => {
                    self.PaintWithPhase(child, &paint_info.ForDescendants());
                }
                _ => {}
            }
        }
        drop(_contents_properties);
        // Blink's overflow controls use border-box properties, outside their
        // owner's OverflowClip/InnerBorderRadiusClip (paint property builder,
        // UpdateOverflowControlEffects). Keep the flat recording in that same
        // space; an extra fractional AA overflow clip is not redundant.
        let paint_controls = phase == PaintPhase::kOverlayOverflowControls
            && node
                .fragment
                .as_deref()
                .is_some_and(|fragment| fragment.paint.scrollbars.is_some());
        if paint_controls {
            self.context.borrow_mut().EndEscapableGeometry(node);
        }
        BoxFragmentPainter::PaintFragmentAfterChildren(node, paint_info);
        if paint_controls {
            self.context.borrow_mut().BeginEscapableGeometry(node, None);
        }
        if enter_node {
            self.context.borrow_mut().EndNode(node);
        }
    }

    // cpp: paint/paint_layer_painter.h:32
    // cpp: paint/paint_layer_painter.cc:232-240
    fn PaintAllPhasesAtomically(&self, node: &PaintTreeNode<'f>) {
        self.PaintWithPhase(
            node,
            &PaintInfo::new(self.context, PaintPhase::kBlockBackground),
        );
        self.PaintWithPhase(node, &PaintInfo::new(self.context, PaintPhase::kFloat));
        self.PaintWithPhase(node, &PaintInfo::new(self.context, PaintPhase::kForeground));
        self.PaintWithPhase(node, &PaintInfo::new(self.context, PaintPhase::kOutline));
    }

    // cpp: paint/paint_layer_painter.h:31
    // cpp: paint/paint_layer_painter.cc:242-302
    fn CollectLayerChildren(&self) -> Vec<CollectedLayer<'n, 'f>> {
        fn collect<'n, 'f>(
            parent: &'n PaintTreeNode<'f>,
            path: &mut Vec<&'n PaintTreeNode<'f>>,
            children: &mut Vec<CollectedLayer<'n, 'f>>,
        ) {
            for child_owner in &parent.children {
                let child = child_owner.as_ref();
                if child
                    .fragment
                    .as_deref()
                    .expect("PrePaint node must own a fragment")
                    .paint
                    .hidden
                {
                    continue;
                }
                if child.is_paint_layer {
                    children.push(CollectedLayer {
                        layer: child,
                        property_ancestors: path.clone(),
                    });
                    if !child.is_stacking_context {
                        path.push(child);
                        collect(child, path, children);
                        path.pop();
                    }
                } else {
                    path.push(child);
                    collect(child, path, children);
                    path.pop();
                }
            }
        }
        let mut children = Vec::new();
        let mut path = Vec::new();
        collect(self.layer, &mut path, &mut children);
        children.sort_by_key(|child| ZIndex(child.layer));
        children
    }

    fn PaintLayerChildren(&self, children: impl IntoIterator<Item = CollectedLayer<'n, 'f>>) {
        for child in children {
            let escapes_geometry = child.layer.escapes_ancestor_geometry;
            let has_physical_ancestor_below_current =
                child.property_ancestors.iter().any(|candidate| {
                    let pointer = *candidate as *const PaintTreeNode<'f>;
                    child.layer.physical_ancestors.contains(&pointer)
                });
            let current_pointer = self.layer as *const PaintTreeNode<'f>;
            let current_is_physical_ancestor =
                child.layer.physical_ancestors.contains(&current_pointer);
            let escapes_current_geometry = escapes_geometry
                && !current_is_physical_ancestor
                && !has_physical_ancestor_below_current;
            if escapes_current_geometry {
                self.SuspendGeometryForEscapingChild();
            }
            let child_painter =
                PaintLayerPainter::new(child.layer, self.context, child.property_ancestors);
            if child.layer.is_stacking_context {
                child_painter.Paint();
            } else {
                child_painter.PaintInternal(false);
            }
            if escapes_current_geometry {
                self.ResumeGeometryAfterEscapingChild();
            }
        }
    }
}
