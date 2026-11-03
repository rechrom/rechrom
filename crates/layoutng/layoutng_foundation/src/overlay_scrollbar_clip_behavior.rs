#![allow(non_camel_case_types)]

// Source: third_party/blink/renderer/platform/graphics/overlay_scrollbar_clip_behavior.h:10-13.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayScrollbarClipBehavior {
    kIgnoreOverlayScrollbarSize,
    kExcludeOverlayScrollbarSizeForHitTesting,
}
