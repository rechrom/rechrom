#![allow(non_camel_case_types, non_upper_case_globals)]

// cpp: foundation/graphics_types/graphics/paint/display_item_client_types.h:12-13
pub type DisplayItemClientId = usize;
pub const kInvalidDisplayItemClientId: DisplayItemClientId = 0;

// cpp: foundation/graphics_types/graphics/paint/display_item_client_types.h:15-19
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RasterEffectOutset {
    kNone,
    kHalfPixel,
    kWholePixel,
}
