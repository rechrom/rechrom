use super::block_node::BlockNode;
use super::layout_box::LayoutBox;
use foundation::{Member, Visitor};

// cpp: layoutng/internal/column_spanner_path.h:18-35
pub struct ColumnSpannerPath {
    box_: Member<LayoutBox>,
    child_: Member<ColumnSpannerPath>,
}

#[allow(non_snake_case)]
impl ColumnSpannerPath {
    // cpp: layoutng/internal/column_spanner_path.h:20-22
    pub fn new(block: BlockNode) -> Self {
        Self::with_child(block, std::ptr::null())
    }

    pub fn with_child(block: BlockNode, child: *const ColumnSpannerPath) -> Self {
        Self {
            box_: Member::from_ptr(block.GetLayoutBox()),
            child_: Member::from_ptr(child as *mut ColumnSpannerPath),
        }
    }

    // cpp: layoutng/internal/column_spanner_path.h:24-25
    pub fn GetBlockNode(&self) -> BlockNode {
        BlockNode::new(self.box_.Get())
    }

    pub fn Child(&self) -> *const ColumnSpannerPath {
        self.child_.Get()
    }

    // cpp: layoutng/internal/column_spanner_path.h:27-30
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.box_);
        visitor.Trace(&self.child_);
    }
}
