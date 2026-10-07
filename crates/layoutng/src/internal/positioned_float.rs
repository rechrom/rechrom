use foundation::{LayoutUnit, Member, Visitor};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;

// cpp: layoutng/internal/positioned_float.h:16-46
pub struct PositionedFloat {
    pub layout_result: Member<LayoutResult>,
    pub break_before_token: Member<BlockBreakToken>,
    pub bfc_offset: BfcOffset,
    pub tallest_unbreakable_block_size: LayoutUnit,
    pub minimum_space_shortage: LayoutUnit,
}

impl Default for PositionedFloat {
    // cpp: layoutng/internal/positioned_float.h:21-21
    fn default() -> Self {
        Self {
            layout_result: Member::default(),
            break_before_token: Member::default(),
            bfc_offset: BfcOffset::default(),
            tallest_unbreakable_block_size: LayoutUnit::default(),
            minimum_space_shortage: LayoutUnit::default(),
        }
    }
}

impl Clone for PositionedFloat {
    // cpp: layoutng/internal/positioned_float.h:32-35
    fn clone(&self) -> Self {
        Self {
            layout_result: Member::from_ptr(self.layout_result.Get()),
            break_before_token: Member::from_ptr(self.break_before_token.Get()),
            bfc_offset: self.bfc_offset,
            tallest_unbreakable_block_size: self.tallest_unbreakable_block_size,
            minimum_space_shortage: self.minimum_space_shortage,
        }
    }
}

#[allow(non_snake_case)]
impl PositionedFloat {
    // cpp: layoutng/internal/positioned_float.h:22-31
    pub fn new(
        layout_result: *const LayoutResult,
        break_before_token: *const BlockBreakToken,
        bfc_offset: &BfcOffset,
        tallest_unbreakable_block_size: LayoutUnit,
        minimum_space_shortage: LayoutUnit,
    ) -> Self {
        Self {
            layout_result: Member::from_ptr(layout_result as *mut LayoutResult),
            break_before_token: Member::from_ptr(break_before_token as *mut BlockBreakToken),
            bfc_offset: *bfc_offset,
            tallest_unbreakable_block_size,
            minimum_space_shortage,
        }
    }

    // cpp: layoutng/internal/positioned_float.cc:11-14
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layout_result);
        visitor.Trace(&self.break_before_token);
    }

    // cpp: layoutng/internal/positioned_float.h:39-39
    // cpp: layoutng/internal/positioned_float.cc:16-22
    pub fn BreakToken(&self) -> *const BlockBreakToken {
        let before = self.break_before_token.Get();
        if !before.is_null() {
            return before;
        }
        let result = self.layout_result.Get();
        let token = unsafe { &*result }.GetPhysicalFragment().GetBreakToken();
        if !token.is_null() {
            debug_assert!(unsafe { &*token }.IsBlockType());
        }
        token.cast::<BlockBreakToken>()
    }
}
