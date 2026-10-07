use super::block_node::BlockNode;
use super::break_appeal::BreakAppeal;
use super::layout_box::LayoutBox;
use foundation::{Member, Visitor};

// cpp: layoutng/internal/early_break.h:20-23
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum BreakType {
    kLine = 0,
    kBlock = 1,
}

// The source union has an active arm fixed by const_type_. An enum keeps the
// same arm invariant without a second mutable discriminator.
// cpp: layoutng/internal/early_break.h:60-69
enum EarlyBreakTarget {
    Block(Member<LayoutBox>),
    Line(i32),
}

// cpp: layoutng/internal/early_break.h:18-18
pub struct EarlyBreak {
    target_: EarlyBreakTarget,
    break_inside_child_: Member<EarlyBreak>,
    break_appeal_: BreakAppeal,
}

#[allow(non_snake_case)]
impl EarlyBreak {
    // cpp: layoutng/internal/early_break.h:25-31
    pub fn from_block(block: BlockNode, break_appeal: BreakAppeal) -> Self {
        Self::from_block_with_child(block, break_appeal, std::ptr::null())
    }

    pub fn from_block_with_child(
        block: BlockNode,
        break_appeal: BreakAppeal,
        break_inside_child: *const EarlyBreak,
    ) -> Self {
        Self {
            target_: EarlyBreakTarget::Block(Member::from_ptr(block.GetLayoutBox())),
            break_inside_child_: Member::from_ptr(break_inside_child as *mut EarlyBreak),
            break_appeal_: break_appeal,
        }
    }

    // cpp: layoutng/internal/early_break.h:32-35
    pub fn from_line(line_number: i32, break_appeal: BreakAppeal) -> Self {
        Self {
            target_: EarlyBreakTarget::Line(line_number),
            break_inside_child_: Member::default(),
            break_appeal_: break_appeal,
        }
    }

    // cpp: layoutng/internal/early_break.h:37-38
    pub fn Type(&self) -> BreakType {
        match &self.target_ {
            EarlyBreakTarget::Line(_) => BreakType::kLine,
            EarlyBreakTarget::Block(_) => BreakType::kBlock,
        }
    }

    pub fn IsBreakBefore(&self) -> bool {
        self.break_inside_child_.Get().is_null()
    }

    // cpp: layoutng/internal/early_break.h:39-42
    pub fn GetBlockNode(&self) -> BlockNode {
        match &self.target_ {
            EarlyBreakTarget::Block(box_) => BlockNode::new(box_.Get()),
            EarlyBreakTarget::Line(_) => panic!("block break required"),
        }
    }

    // cpp: layoutng/internal/early_break.h:43-51
    pub fn LineNumber(&self) -> i32 {
        match &self.target_ {
            EarlyBreakTarget::Line(line_number) => *line_number,
            EarlyBreakTarget::Block(_) => panic!("line break required"),
        }
    }

    pub fn BreakInside(&self) -> *const EarlyBreak {
        self.break_inside_child_.Get()
    }

    pub fn GetBreakAppeal(&self) -> BreakAppeal {
        self.break_appeal_
    }

    // cpp: layoutng/internal/early_break.h:53-58
    pub fn Trace(&self, visitor: &mut Visitor) {
        if let EarlyBreakTarget::Block(box_) = &self.target_ {
            visitor.Trace(box_);
        }
        visitor.Trace(&self.break_inside_child_);
    }
}
