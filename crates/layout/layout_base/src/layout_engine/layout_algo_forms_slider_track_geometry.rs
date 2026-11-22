// Copyright The Chromium Authors. BSD license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: slider_thumb_element.cc:61-73,105-120,188-192.
use super::*;
use crate::internal::{
    layout_box_model_object::LayoutBoxModelObject, map_coordinates_flags::MapCoordinatesFlags,
};
use foundation::{slider_geometry::SliderTrackGeometry, PhysicalOffset};
impl LayoutEngine {
    /// A synchronous prepared-geometry query. Each id resolves to its resident
    /// native LayoutObject; coordinate conversion uses the same transform path
    /// as hit testing and never a copied rectangle or a document scan.
    pub fn SliderTrackGeometry(
        &self,
        host_id: u64,
        thumb_id: u64,
        track_id: u64,
        point: PhysicalOffset,
    ) -> Option<SliderTrackGeometry> {
        let _scope = self.NativeScrollPassScope();
        let tree = self.tree_.as_ref()?;
        let input = tree.OutputNode(host_id)?;
        let thumb = tree.OutputNode(thumb_id)?;
        let track = tree.OutputNode(track_id)?;
        let input_box = DynamicTo::<LayoutBox>(input as *const LayoutObject as *mut LayoutObject);
        let thumb_box = DynamicTo::<LayoutBox>(thumb as *const LayoutObject as *mut LayoutObject);
        let track_box = DynamicTo::<LayoutBox>(track as *const LayoutObject as *mut LayoutObject);
        if input_box.is_null() || thumb_box.is_null() || track_box.is_null() {
            return None;
        }
        let thumb_box = unsafe { &*thumb_box };
        let track_box = unsafe { &*track_box };
        let flags = MapCoordinatesFlags::default();
        let ancestor = input_box.cast::<LayoutBoxModelObject>();
        let thumb_offset =
            thumb_box.LocalToAncestorPointPhysical(&PhysicalOffset::default(), ancestor, flags)
                - track_box.LocalToAncestorPointPhysical(
                    &PhysicalOffset::default(),
                    ancestor,
                    flags,
                );
        let margins = thumb_box.MarginOutsets();
        let direction = thumb_box.StyleRef().GetWritingDirection();
        Some(SliderTrackGeometry {
            point_in_track: track_box.AbsoluteToLocalPointPhysical(&point, flags),
            thumb_offset,
            thumb_size: thumb_box.StitchedSize(),
            thumb_margin_top: margins.top,
            thumb_margin_right: margins.right,
            thumb_margin_bottom: margins.bottom,
            thumb_margin_left: margins.left,
            track_content_size: track_box.PhysicalContentBoxRect().size,
            horizontal: direction.IsHorizontal(),
            flipped_inlines: direction.IsFlippedInlines(),
        })
    }
    pub fn InvalidateSliderThumbPosition(&mut self, thumb_id: u64, host_id: Option<u64>) -> bool {
        let _scope = self.NativeScrollPassScope();
        let Some(tree) = self.tree_.as_mut() else {
            return false;
        };
        let Some(thumb) = tree.OutputNodeMut(thumb_id) else {
            return false;
        };
        thumb.SetNeedsLayoutAndFullPaintInvalidation(
            &raw const crate::internal::layout_invalidation_reason::kSliderValueChanged,
        );
        if let Some(host_id) = host_id {
            if let Some(input) = tree.OutputNodeMut(host_id) {
                input.SetShouldDoFullPaintInvalidation();
            }
        }
        if let Some(fragments) = self.fragments.take() {
            self.invalidated_fragments_ = Some(fragments);
        }
        true
    }
}
impl LayoutEngine {
    // Source: slider_thumb_element.cc:280-309,418-461. Return only the
    // current scalar facts; source default owner retains the control decisions.
    pub fn SliderStyleFacts(
        &self,
        host_id: u64,
        container_id: u64,
    ) -> foundation::slider_geometry::SliderStyleFacts {
        let _scope = self.NativeScrollPassScope();
        let host = self
            .tree_
            .as_ref()
            .and_then(|tree| tree.OutputNode(host_id));
        let container = self
            .tree_
            .as_ref()
            .and_then(|tree| tree.OutputNode(container_id));
        let host_appearance = host.map(|host| host.StyleRef().EffectiveAppearance());
        let host_has_rotate_or_rotate_z = host.is_some_and(|host| {
            let transforms = host.StyleRef().Transform();
            (0..transforms.size()).any(|index| {
                transforms.at(index).is_some_and(|op| {
                    matches!(
                        op.GetType(),
                        foundation::TransformOperationType::kRotate
                            | foundation::TransformOperationType::kRotateZ
                    )
                })
            })
        });
        let container_horizontal = container
            .and_then(|container| unsafe { container.Style().as_ref() })
            .map(|style| style.IsHorizontalWritingMode());
        foundation::slider_geometry::SliderStyleFacts {
            host_appearance,
            host_has_rotate_or_rotate_z,
            container_horizontal,
        }
    }
}
