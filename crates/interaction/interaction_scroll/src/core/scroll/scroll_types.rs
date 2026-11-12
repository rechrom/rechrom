/*
 * Copyright (C) 2006 Apple Computer, Inc. All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 * 1. Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * THIS SOFTWARE IS PROVIDED BY APPLE COMPUTER, INC. ``AS IS'' AND ANY EXPRESS
 * OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
 * OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
 * IN NO EVENT SHALL APPLE COMPUTER, INC. OR CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
 * THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/scroll/scroll_types.h:37-136,144,169-212

pub use crate::public::mojom::input::scroll_direction::ScrollDirection;
use crate::public::mojom::scroll::scroll_enums::ScrollType;
use foundation::gfx::Vector2dF;

// Source: scroll_types.h:37-49
pub type ScrollOffset = Vector2dF;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollDirectionPhysical {
    kScrollUp,
    kScrollDown,
    kScrollLeft,
    kScrollRight,
}

// Source: scroll_types.h:51-57.
// An explicit scroll is requested by the user or webpage; an implicit scroll
// is a side effect of a layout change.
pub fn IsExplicitScrollType(scroll_type: ScrollType) -> bool {
    scroll_type == ScrollType::kUser
        || scroll_type == ScrollType::kProgrammatic
        || scroll_type == ScrollType::kCompositor
}

// Source: scroll_types.h:59-119. The source's is_vertical argument receives
// LayoutBox::IsHorizontalWritingMode (vertical block progression).
pub fn ToPhysicalDirection(
    direction: ScrollDirection,
    is_vertical: bool,
    is_flipped: bool,
) -> ScrollDirectionPhysical {
    use ScrollDirection::*;
    use ScrollDirectionPhysical::*;
    match direction {
        kScrollBlockDirectionBackward => {
            if is_vertical {
                if !is_flipped {
                    kScrollUp
                } else {
                    kScrollDown
                }
            } else if !is_flipped {
                kScrollLeft
            } else {
                kScrollRight
            }
        }
        kScrollBlockDirectionForward => {
            if is_vertical {
                if !is_flipped {
                    kScrollDown
                } else {
                    kScrollUp
                }
            } else if !is_flipped {
                kScrollRight
            } else {
                kScrollLeft
            }
        }
        kScrollInlineDirectionBackward => {
            if is_vertical {
                if !is_flipped {
                    kScrollLeft
                } else {
                    kScrollRight
                }
            } else if !is_flipped {
                kScrollUp
            } else {
                kScrollDown
            }
        }
        kScrollInlineDirectionForward => {
            if is_vertical {
                if !is_flipped {
                    kScrollRight
                } else {
                    kScrollLeft
                }
            } else if !is_flipped {
                kScrollDown
            } else {
                kScrollUp
            }
        }
        kScrollUpIgnoringWritingMode => kScrollUp,
        kScrollDownIgnoringWritingMode => kScrollDown,
        kScrollLeftIgnoringWritingMode => kScrollLeft,
        kScrollRightIgnoringWritingMode => kScrollRight,
    }
}

// Source: scroll_types.h:121-136
pub fn ToScrollDirection(direction: ScrollDirectionPhysical) -> ScrollDirection {
    match direction {
        ScrollDirectionPhysical::kScrollUp => ScrollDirection::kScrollUpIgnoringWritingMode,
        ScrollDirectionPhysical::kScrollDown => ScrollDirection::kScrollDownIgnoringWritingMode,
        ScrollDirectionPhysical::kScrollLeft => ScrollDirection::kScrollLeftIgnoringWritingMode,
        ScrollDirectionPhysical::kScrollRight => ScrollDirection::kScrollRightIgnoringWritingMode,
    }
}

// Source: scroll_types.h:144
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarOrientation {
    kHorizontalScrollbar,
    kVerticalScrollbar,
}

// Source: scroll_types.h:169-198
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollConsumption {
    pub did_scroll_x: bool,
    pub did_scroll_y: bool,
    pub unused_scroll_delta_x: f32,
    pub unused_scroll_delta_y: f32,
}
impl ScrollConsumption {
    pub fn new(
        did_scroll_x: bool,
        did_scroll_y: bool,
        unused_scroll_delta_x: f32,
        unused_scroll_delta_y: f32,
    ) -> Self {
        Self {
            did_scroll_x,
            did_scroll_y,
            unused_scroll_delta_x,
            unused_scroll_delta_y,
        }
    }
    pub fn DidScroll(&self) -> bool {
        self.did_scroll_x || self.did_scroll_y
    }
}

// Source: scroll_types.h:200-204
pub fn ToScrollDeltaForOrientation(orientation: ScrollbarOrientation, delta: f32) -> ScrollOffset {
    if orientation == ScrollbarOrientation::kHorizontalScrollbar {
        ScrollOffset::new(delta, 0.0)
    } else {
        ScrollOffset::new(0.0, delta)
    }
}

// Source: scroll_types.h:206-212
pub fn ToScrollDelta(direction: ScrollDirectionPhysical, mut delta: f32) -> ScrollOffset {
    if matches!(
        direction,
        ScrollDirectionPhysical::kScrollUp | ScrollDirectionPhysical::kScrollLeft
    ) {
        delta = -delta;
    }
    if matches!(
        direction,
        ScrollDirectionPhysical::kScrollLeft | ScrollDirectionPhysical::kScrollRight
    ) {
        ScrollOffset::new(delta, 0.0)
    } else {
        ScrollOffset::new(0.0, delta)
    }
}
