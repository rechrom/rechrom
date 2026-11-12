//! FindBuffer native OffsetMapping/IFC reads. Source find_buffer.cc:45-62,
//! 662-704; offset_mapping.cc:301-355,492-513,560-578.
//! These views borrow the resident LayoutOutput; no mapping/tree copy escapes.
use super::LayoutNode;
use crate::{
    internal::{
        inline_node::InlineNode, layout_block_flow::LayoutBlockFlow, layout_node_metadata::Node,
    },
    offset_mapping::OffsetMapping,
};
use foundation::DynamicTo;
#[derive(Clone, Copy)]
pub struct EditingFindMapping<'a> {
    mapping: &'a OffsetMapping,
    node: &'a Node,
}
impl<'a> EditingFindMapping<'a> {
    pub fn ForTextNode(&self, node: &LayoutNode<'a>) -> Option<Self> {
        let node: &'a Node = unsafe { node.object.NonPseudoNode().as_ref() }?;
        node.IsTextNode().then_some(Self {
            mapping: self.mapping,
            node,
        })
    }

    pub fn Text(&self) -> &'a foundation::String {
        self.mapping.GetText()
    }
    pub fn FirstPosition(&self, offset: u32) -> crate::EditingPosition {
        self.mapping.GetFirstPosition(offset)
    }
    pub fn LastPosition(&self, offset: u32) -> crate::EditingPosition {
        self.mapping.GetLastPosition(offset)
    }
    pub fn Units(&self, start: u32, end: u32) -> impl Iterator<Item = (u32, u32)> + 'a {
        let mapping: &'a OffsetMapping = self.mapping;
        let units = mapping.GetMappingUnitsForNode(self.node);
        let first = units.partition_point(|unit| unit.DOMEnd() < start);
        let last = units.partition_point(|unit| unit.DOMStart() <= end);
        units[first.min(last)..last].iter().map(move |unit| {
            (
                unit.ConvertDOMOffsetToTextContent(unit.DOMStart().max(start)),
                unit.ConvertDOMOffsetToTextContent(unit.DOMEnd().min(end)),
            )
        })
    }
}
impl PartialEq for EditingFindMapping<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.mapping, other.mapping)
    }
}
impl Eq for EditingFindMapping<'_> {}
#[derive(Clone, Copy)]
pub struct EditingFindBlockIdentity<'a>(&'a LayoutBlockFlow);
impl PartialEq for EditingFindBlockIdentity<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}
impl Eq for EditingFindBlockIdentity<'_> {}
impl<'a> LayoutNode<'a> {
    pub fn EditingFindBlockIdentity(
        &self,
        in_user_agent_shadow_root: bool,
    ) -> Option<EditingFindBlockIdentity<'a>> {
        let block =
            unsafe { OffsetMapping::GetInlineFormattingContextOfObject(self.object).as_ref() }?;
        if block.IsAnonymous() && in_user_agent_shadow_root {
            let mut parent = block.ContainingBlock();
            while let Some(current) = unsafe { parent.as_ref() } {
                if !current.IsAnonymous() && current.IsLayoutBlockFlow() {
                    return Some(EditingFindBlockIdentity(unsafe {
                        &*DynamicTo::<LayoutBlockFlow>(current)
                    }));
                }
                parent = current.ContainingBlock();
            }
        }
        Some(EditingFindBlockIdentity(block))
    }
}
impl<'a> LayoutNode<'a> {
    pub fn EditingFindMapping(&self) -> Option<EditingFindMapping<'a>> {
        let node: &'a Node = unsafe { self.object.NonPseudoNode().as_ref() }?;
        if !node.IsTextNode() {
            return None;
        }
        let block = OffsetMapping::GetInlineFormattingContextOfObject(self.object);
        let mapping: &'a OffsetMapping = unsafe { InlineNode::GetOffsetMapping(block).as_ref() }?;
        Some(EditingFindMapping { mapping, node })
    }
}

pub use foundation::find_buffer::FindStyleFacts as EditingFindStyleFacts;
impl LayoutNode<'_> {
    pub fn EditingFindStyleFacts(&self) -> EditingFindStyleFacts {
        let style = self.object.StyleRef();
        EditingFindStyleFacts {
            display_none: style.Display() == foundation::EDisplay::kNone,
            visible: style.Visibility() == foundation::EVisibility::kVisible,
            inert: style.IsInert(),
            collapse_spaces: style.ShouldCollapseWhiteSpaces(),
            ensured: style.IsEnsuredInDisplayNone(),
        }
    }
    pub fn EditingFindIsAnonymous(&self) -> bool {
        self.object.IsAnonymous()
    }
}

impl LayoutNode<'_> {
    pub fn EditingFindIsLayoutText(&self) -> bool {
        self.object.IsText()
    }
}
