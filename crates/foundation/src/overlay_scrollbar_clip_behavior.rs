#![allow(non_camel_case_types)]

// cpp: foundation/graphics_types/graphics/overlay_scrollbar_clip_behavior.h:11-14
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayScrollbarClipBehavior {
    kIgnoreOverlayScrollbarSize,
    kExcludeOverlayScrollbarSizeForHitTesting,
}
