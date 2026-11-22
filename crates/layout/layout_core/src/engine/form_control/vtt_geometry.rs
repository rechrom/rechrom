//! VttCueLayoutAlgorithm.cc24-66,90-129,178-194 resident geometry queries.
use crate::{
    internal::{
        layout_block_flow::LayoutBlockFlow, layout_box::LayoutBox,
        layout_box_model_object::LayoutBoxModelObject, layout_node_metadata::Element,
        layout_object::LayoutObject, map_coordinates_flags::MapCoordinatesFlags,
    },
    layout_engine::LayoutEngine,
};
use foundation::{
    gfx,
    vtt_geometry::{VTTCueLayoutFacts, VTTLayoutObjectId},
    DynamicTo, PhysicalSize,
};
use layout_fragment::inline_cursor::InlineCursor;
impl LayoutEngine {
    pub fn VTTCueLayoutObject(&self, node: u64) -> Option<VTTLayoutObjectId> {
        let object = self.GetLayoutTree()?.GetLayoutObjectForNode(node)?;
        (!DynamicTo::<LayoutBox>(object as *const _).is_null())
            .then_some(VTTLayoutObjectId(object as *const _ as usize))
    }
    fn VTTObject(&self, object: VTTLayoutObjectId) -> &LayoutObject {
        assert_ne!(object.0, 0);
        unsafe { &*(object.0 as *const LayoutObject) }
    }
    pub fn VTTCueLayoutFacts(&self, object: VTTLayoutObjectId) -> VTTCueLayoutFacts {
        let _scope = self.NativeScrollPassScope();
        let object = self.VTTObject(object);
        let cue = unsafe { DynamicTo::<LayoutBox>(object as *const _).as_ref() }
            .expect("same native cue LayoutBox");
        let container =
            unsafe { cue.ContainingBlock().as_ref() }.expect("source VTTCueBox containing block");
        let parent = unsafe { cue.Parent().as_ref() };
        let parent_node = parent.and_then(|p| unsafe { p.GetNode().as_ref() });
        VTTCueLayoutFacts {
            horizontal: cue.IsHorizontalWritingMode(),
            flipped_blocks: cue.HasFlippedBlocksWritingMode(),
            stitched_width: cue.StitchedSize().width,
            logical_height: cue.LogicalHeight(),
            has_parent_and_grandparent: parent.is_some_and(|p| !p.Parent().is_null()),
            container_size: container.StitchedSize(),
            container_border_box: container.PhysicalBorderBoxRect(),
            border_box_in_containing_block: cue.LocalToAncestorRect(
                &cue.PhysicalBorderBoxRect(),
                container as *const _ as *const LayoutBoxModelObject,
                MapCoordinatesFlags::default(),
            ),
            parent_node_id: parent_node.map(|n| n.InputId()),
        }
    }
    pub fn VTTFirstInlineBoxSize(&self, object: VTTLayoutObjectId) -> PhysicalSize {
        let _scope = self.NativeScrollPassScope();
        let flow =
            unsafe { DynamicTo::<LayoutBlockFlow>(self.VTTObject(object) as *const _).as_ref() }
                .expect("non-region VTTCueBox standard block flow");
        let mut cursor = InlineCursor::from_container(flow);
        if !cursor.HasRoot() {
            return PhysicalSize::default();
        }
        cursor.MoveToFirstLine();
        if cursor.IsNull() {
            return PhysicalSize::default();
        }
        unsafe { &*cursor.CurrentItem() }.Size()
    }
    pub fn VTTPreviousLayoutSibling(&self, object: VTTLayoutObjectId) -> Option<VTTLayoutObjectId> {
        let _scope = self.NativeScrollPassScope();
        let previous = self.VTTObject(object).PreviousSibling();
        (!previous.is_null()).then_some(VTTLayoutObjectId(previous as usize))
    }
    pub fn VTTLayoutNodeId(&self, object: VTTLayoutObjectId) -> Option<u64> {
        let _scope = self.NativeScrollPassScope();
        unsafe { self.VTTObject(object).GetNode().as_ref() }.map(|n| n.InputId())
    }
    pub fn VTTControlsRects(
        &self,
        container: u64,
        button: u64,
        timeline: u64,
    ) -> Option<(gfx::Rect, gfx::Rect)> {
        let _scope = self.NativeScrollPassScope();
        let tree = self.GetLayoutTree()?;
        let container = unsafe {
            DynamicTo::<LayoutBox>(tree.GetLayoutObjectForNode(container)? as *const _).as_ref()
        }?;
        let button = unsafe {
            DynamicTo::<LayoutBox>(tree.GetLayoutObjectForNode(button)? as *const _).as_ref()
        }?;
        let timeline = unsafe {
            DynamicTo::<LayoutBox>(tree.GetLayoutObjectForNode(timeline)? as *const _).as_ref()
        }?;
        let ancestor = container as *const _ as *const LayoutBoxModelObject;
        let border = |box_: &LayoutBox| {
            gfx::ToEnclosingRect(gfx::RectF::from(box_.LocalToAncestorRect(
                &box_.PhysicalBorderBoxRect(),
                ancestor,
                MapCoordinatesFlags::default(),
            )))
        };
        Some((border(button), border(timeline)))
    }
}
