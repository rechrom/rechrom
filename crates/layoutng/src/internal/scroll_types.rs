use foundation::gfx;

// cpp: layoutng/internal/scroll_types.h:35-35
pub type ScrollOffset = gfx::Vector2dF;

// cpp: layoutng/internal/scroll_types.h:38-41
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum IncludeScrollbarsInRect {
    kExcludeScrollbars = 0,
    kIncludeScrollbars = 1,
}

// cpp: layoutng/internal/scroll_types.h:45-48
#[allow(non_snake_case)]
pub fn SnapScrollOffsetToPhysicalPixels(offset: &gfx::Vector2dF) -> gfx::Vector2d {
    gfx::ToRoundedVector2d(offset)
}
