//! Shared layout position for insertion-caret painting.
use crate::internal::layout_input::{Offset, Size};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalCaretRect {
    pub offset: Offset,
    pub size: Size,
}
