//! Caret shape value shared by the fragment caret contract and paint style.

#![allow(non_camel_case_types)]

// Declared with the fragment caret contract (layout_fragment/src/caret_rect.rs),
// which re-exports it.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaretShape {
    kBar,
    kBlock,
    kUnderscore,
}
