// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Platform-neutral inputs to Blink's uncached custom scrollbar style request.
//!
//! The embedding converts its native ScrollbarPart at the package boundary and
//! supplies the live scrollbar facts for this synchronous request. No Layout
//! owner or callback crosses into Style.
#![allow(non_snake_case, non_upper_case_globals)]

use layoutng_style::style::computed_style_constants::PseudoId;

/// Checked scalar projection of core/scroll/scroll_types.h ScrollbarPart.
/// Combined masks are not individual style targets.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ScrollbarStylePart(u32);
impl ScrollbarStylePart {
    pub const kNoPart: Self = Self(0);
    pub const kBackButtonStartPart: Self = Self(1);
    pub const kForwardButtonStartPart: Self = Self(2);
    pub const kBackTrackPart: Self = Self(4);
    pub const kThumbPart: Self = Self(8);
    pub const kForwardTrackPart: Self = Self(16);
    pub const kBackButtonEndPart: Self = Self(32);
    pub const kForwardButtonEndPart: Self = Self(64);
    pub const kScrollbarBGPart: Self = Self(128);
    pub const kTrackBGPart: Self = Self(256);

    pub const fn Bits(self) -> u32 {
        self.0
    }

    /// cpp: PseudoForScrollbarPart, custom_scrollbar.cc:217-238.
    pub(crate) fn PseudoId(self) -> Option<PseudoId> {
        Some(match self {
            Self::kScrollbarBGPart => PseudoId::kPseudoIdScrollbar,
            Self::kBackButtonStartPart
            | Self::kForwardButtonStartPart
            | Self::kBackButtonEndPart
            | Self::kForwardButtonEndPart => PseudoId::kPseudoIdScrollbarButton,
            Self::kBackTrackPart | Self::kForwardTrackPart => {
                PseudoId::kPseudoIdScrollbarTrackPiece
            }
            Self::kThumbPart => PseudoId::kPseudoIdScrollbarThumb,
            Self::kTrackBGPart => PseudoId::kPseudoIdScrollbarTrack,
            _ => return None,
        })
    }
}
impl TryFrom<u32> for ScrollbarStylePart {
    type Error = InvalidScrollbarStylePart;
    fn try_from(bits: u32) -> Result<Self, Self::Error> {
        match bits {
            0 | 1 | 2 | 4 | 8 | 16 | 32 | 64 | 128 | 256 => Ok(Self(bits)),
            _ => Err(InvalidScrollbarStylePart),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidScrollbarStylePart;
impl std::fmt::Display for InvalidScrollbarStylePart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("expected an individual Chromium scrollbar part")
    }
}
impl std::error::Error for InvalidScrollbarStylePart {}

/// Values read by SelectorChecker::CheckScrollbarPseudoClass. Hover and press
/// name the scrollbar's actual selected part, independently of the part whose
/// style is being resolved (the scrollbar/track backgrounds match descendants).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollbarStyleState {
    /// FocusController::IsActive, independent of focused-frame ownership.
    pub page_active: bool,
    pub enabled: bool,
    pub horizontal: bool,
    pub hovered_part: ScrollbarStylePart,
    pub pressed_part: ScrollbarStylePart,
    pub native_theme_has_buttons: bool,
    pub scroll_corner_visible: bool,
}
