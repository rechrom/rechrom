#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// cpp: foundation/graphics_types/graphics/subtree_paint_property_update_reason.h:15-23
#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SubtreePaintPropertyUpdateReason {
    #[default]
    kNone = 0,
    kContainerChainMayChange = 1 << 0,
    kPreviouslySkipped = 1 << 1,
    kPrinting = 1 << 2,
    kTransformStyleChanged = 1 << 3,
}

// cpp: foundation/graphics_types/graphics/subtree_paint_property_update_reason.h:24-24
pub const kSubtreePaintPropertyUpdateReasonsBitfieldWidth: u32 = 4;
