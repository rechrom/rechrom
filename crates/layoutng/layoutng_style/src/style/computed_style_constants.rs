/*
 * Copyright (C) 2000 Lars Knoll (knoll@kde.org)
 *           (C) 2000 Antti Koivisto (koivisto@kde.org)
 *           (C) 2000 Dirk Mueller (mueller@kde.org)
 * Copyright (C) 2003, 2005, 2006, 2007, 2008, 2009, 2010 Apple Inc. All rights
 * reserved.
 * Copyright (C) 2006 Graham Dennis (graham.dennis@gmail.com)
 * Copyright (C) 2009 Torch Mobile Inc. All rights reserved.
 * (http://www.torchmobile.com/)
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 */

// cpp: layoutng_style/style/computed_style_constants.h:33-35
// Pending connection to foundation:style_values_api for generated constants.

// cpp: layoutng_style/style/computed_style_constants.h:39-42
/// Extracts the C++ unsigned representation of a flag-bearing enum.
pub trait EnumFlagBits {
    fn bits(self) -> u32;
}
#[allow(non_snake_case)]
pub fn EnumHasFlags<E: EnumFlagBits>(value: E, mask: E) -> bool {
    (value.bits() & mask.bits()) != 0
}

// cpp: layoutng_style/style/computed_style_constants.h:51-53
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum BoxSide {
    kTop,
    kRight,
    kBottom,
    kLeft,
}

// cpp: layoutng_style/style/computed_style_constants.h:55-132
// Alias values at the end of the C++ enum require a transparent newtype.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PseudoId(u8);

#[allow(non_upper_case_globals)]
impl PseudoId {
    pub const kPseudoIdNone: Self = Self(0);
    pub const kPseudoIdFirstLine: Self = Self(1);
    pub const kPseudoIdFirstLetter: Self = Self(2);
    pub const kPseudoIdCheckMark: Self = Self(3);
    pub const kPseudoIdBefore: Self = Self(4);
    pub const kPseudoIdAfter: Self = Self(5);
    pub const kPseudoIdExpandIcon: Self = Self(6);
    pub const kPseudoIdPickerIcon: Self = Self(7);
    pub const kPseudoIdInterestButton: Self = Self(8);
    pub const kPseudoIdMarker: Self = Self(9);
    pub const kPseudoIdBackdrop: Self = Self(10);
    pub const kPseudoIdOverscrollBackdrop: Self = Self(11);
    pub const kPseudoIdSelection: Self = Self(12);
    pub const kPseudoIdScrollbar: Self = Self(13);
    pub const kPseudoIdScrollMarker: Self = Self(14);
    pub const kPseudoIdScrollMarkerGroup: Self = Self(15);
    pub const kPseudoIdScrollButton: Self = Self(16);
    pub const kPseudoIdScrollButtonBlockStart: Self = Self(17);
    pub const kPseudoIdScrollButtonInlineStart: Self = Self(18);
    pub const kPseudoIdScrollButtonInlineEnd: Self = Self(19);
    pub const kPseudoIdScrollButtonBlockEnd: Self = Self(20);
    pub const kPseudoIdColumn: Self = Self(21);
    pub const kPseudoIdSearchText: Self = Self(22);
    pub const kPseudoIdTargetText: Self = Self(23);
    pub const kPseudoIdHighlight: Self = Self(24);
    pub const kPseudoIdSpellingError: Self = Self(25);
    pub const kPseudoIdGrammarError: Self = Self(26);
    pub const kPseudoIdViewTransition: Self = Self(27);
    pub const kPseudoIdViewTransitionGroup: Self = Self(28);
    pub const kPseudoIdViewTransitionGroupChildren: Self = Self(29);
    pub const kPseudoIdViewTransitionImagePair: Self = Self(30);
    pub const kPseudoIdViewTransitionOld: Self = Self(31);
    pub const kPseudoIdViewTransitionNew: Self = Self(32);
    pub const kPseudoIdSkeleton: Self = Self(33);
    pub const kPseudoIdOverscrollAreaParent: Self = Self(34);
    pub const kPseudoIdFirstLineInherited: Self = Self(35);
    pub const kPseudoIdScrollbarThumb: Self = Self(36);
    pub const kPseudoIdScrollbarButton: Self = Self(37);
    pub const kPseudoIdScrollbarTrack: Self = Self(38);
    pub const kPseudoIdScrollbarTrackPiece: Self = Self(39);
    pub const kPseudoIdScrollbarCorner: Self = Self(40);
    pub const kPseudoIdScrollMarkerGroupAfter: Self = Self(41);
    pub const kPseudoIdScrollMarkerGroupBefore: Self = Self(42);
    pub const kPseudoIdResizer: Self = Self(43);
    pub const kPseudoIdInputListButton: Self = Self(44);
    pub const kPseudoIdPlaceholder: Self = Self(45);
    pub const kPseudoIdFileSelectorButton: Self = Self(46);
    pub const kPseudoIdDetailsContent: Self = Self(47);
    pub const kPseudoIdPickerSelect: Self = Self(48);
    pub const kPseudoIdSelectListbox: Self = Self(49);
    pub const kPseudoIdPermissionIcon: Self = Self(50);
    pub const kAfterLastInternalPseudoId: Self = Self(51);
    pub const kPseudoIdInvalid: Self = Self(52);
    pub const kFirstPublicPseudoId: Self = Self::kPseudoIdFirstLine;
    pub const kLastTrackedPublicPseudoId: Self = Self::kPseudoIdGrammarError;
    pub const kLastPublicPseudoId: Self = Self::kPseudoIdOverscrollAreaParent;
    pub const kFirstInternalPseudoId: Self = Self::kPseudoIdFirstLineInherited;
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
    pub const fn value(self) -> u8 {
        self.0
    }
}

// cpp: layoutng_style/style/computed_style_constants.h:134-143
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PseudoIdFlags {
    bits_: u32,
}

#[allow(non_snake_case, non_upper_case_globals)]
impl PseudoIdFlags {
    pub const kFirstValid: PseudoId = PseudoId::kFirstPublicPseudoId;
    pub const kLastValid: PseudoId = PseudoId::kLastTrackedPublicPseudoId;

    pub const fn FromBits(bits: u32) -> Self {
        Self { bits_: bits }
    }

    // cpp: layoutng_style/style/computed_style_constants.h:145-151
    pub const fn from_list<const N: usize>(list: &[PseudoId; N]) -> Self {
        let mut result = Self { bits_: 0 };
        let mut index = 0;
        while index < N {
            result.bits_ |= 1u32 << Self::Bit(list[index]);
            index += 1;
        }
        result
    }

    // cpp: layoutng_style/style/computed_style_constants.h:155-158
    pub fn or_assign(&mut self, other: &Self) -> &mut Self {
        self.bits_ |= other.bits_;
        self
    }

    // cpp: layoutng_style/style/computed_style_constants.h:160-163
    pub fn Set(&mut self, pseudo_id: PseudoId) {
        let bit = Self::Bit(pseudo_id);
        debug_assert!(bit < 32);
        self.bits_ |= 1u32 << bit;
    }

    // cpp: layoutng_style/style/computed_style_constants.h:165-169
    pub fn MaybeSet(&mut self, pseudo_id: PseudoId) {
        if pseudo_id >= Self::kFirstValid && pseudo_id <= Self::kLastValid {
            self.Set(pseudo_id);
        }
    }

    // cpp: layoutng_style/style/computed_style_constants.h:171-174
    pub fn Has(&self, pseudo_id: PseudoId) -> bool {
        let bit = Self::Bit(pseudo_id);
        debug_assert!(bit < 32);
        self.bits_ & (1u32 << bit) != 0
    }

    // cpp: layoutng_style/style/computed_style_constants.h:176-178
    pub fn HasAny(&self) -> bool {
        self.bits_ != 0
    }
    pub fn Bits(&self) -> u32 {
        self.bits_
    }

    // cpp: layoutng_style/style/computed_style_constants.h:180-188
    const fn Bit(pseudo_id: PseudoId) -> u32 {
        (pseudo_id.value() as u32).wrapping_sub(Self::kFirstValid.value() as u32)
    }
}

// cpp: layoutng_style/style/computed_style_constants.h:153
// Equality is derived on the bits_ member above.
// cpp: layoutng_style/style/computed_style_constants.h:155-158
impl std::ops::BitOrAssign for PseudoIdFlags {
    fn bitor_assign(&mut self, other: Self) {
        self.bits_ |= other.bits_;
    }
}

// cpp: layoutng_style/style/computed_style_constants.h:187
const _: () =
    assert!((PseudoIdFlags::kLastValid.value() - PseudoIdFlags::kFirstValid.value()) < 32);

// cpp: layoutng_style/style/computed_style_constants.h:191-203
#[allow(non_snake_case)]
pub fn IsHighlightPseudoElement(pseudo_id: PseudoId) -> bool {
    pseudo_id == PseudoId::kPseudoIdSelection
        || pseudo_id == PseudoId::kPseudoIdSearchText
        || pseudo_id == PseudoId::kPseudoIdTargetText
        || pseudo_id == PseudoId::kPseudoIdHighlight
        || pseudo_id == PseudoId::kPseudoIdSpellingError
        || pseudo_id == PseudoId::kPseudoIdGrammarError
}

// cpp: layoutng_style/style/computed_style_constants.h:205-217
#[allow(non_snake_case)]
pub fn IsTransitionPseudoElement(pseudo_id: PseudoId) -> bool {
    pseudo_id == PseudoId::kPseudoIdViewTransition
        || pseudo_id == PseudoId::kPseudoIdViewTransitionGroup
        || pseudo_id == PseudoId::kPseudoIdViewTransitionGroupChildren
        || pseudo_id == PseudoId::kPseudoIdViewTransitionImagePair
        || pseudo_id == PseudoId::kPseudoIdViewTransitionOld
        || pseudo_id == PseudoId::kPseudoIdViewTransitionNew
}

// cpp: layoutng_style/style/computed_style_constants.h:219-231
#[allow(non_snake_case)]
pub fn PseudoElementHasArguments(pseudo_id: PseudoId) -> bool {
    pseudo_id == PseudoId::kPseudoIdHighlight
        || pseudo_id == PseudoId::kPseudoIdViewTransitionGroup
        || pseudo_id == PseudoId::kPseudoIdViewTransitionGroupChildren
        || pseudo_id == PseudoId::kPseudoIdViewTransitionImagePair
        || pseudo_id == PseudoId::kPseudoIdViewTransitionNew
        || pseudo_id == PseudoId::kPseudoIdViewTransitionOld
}

// cpp: layoutng_style/style/computed_style_constants.h:233
// C++ has a bool underlying type; u8 preserves its one-byte layout.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum OutlineIsAuto {
    kOff = 0,
    kOn = 1,
}

// cpp: layoutng_style/style/computed_style_constants.h:237-248
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EVerticalAlign {
    kBaseline,
    kMiddle,
    kSub,
    kSuper,
    kTextTop,
    kTextBottom,
    kTop,
    kBottom,
    kBaselineMiddle,
    kLength,
}

// cpp: layoutng_style/style/computed_style_constants.h:250
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EFillAttachment {
    kScroll,
    kLocal,
    kFixed,
}

// cpp: layoutng_style/style/computed_style_constants.h:252-277
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EFillBox {
    kBorder,
    kPadding,
    kContent,
    kText,
    kBorderArea,
    kBorderAreaText,
    kFillBox,
    kStrokeBox,
    kViewBox,
    kNoClip,
}

// cpp: layoutng_style/style/computed_style_constants.h:279-284
#[allow(non_snake_case)]
pub fn IsSpecialClipFillBox(fill_box: EFillBox) -> bool {
    fill_box == EFillBox::kText
        || fill_box == EFillBox::kBorderArea
        || fill_box == EFillBox::kBorderAreaText
}

// cpp: layoutng_style/style/computed_style_constants.h:286-313
#[allow(non_snake_case)]
pub fn EnclosingFillBox(box_a: EFillBox, box_b: EFillBox) -> EFillBox {
    if box_a == EFillBox::kNoClip || box_b == EFillBox::kNoClip {
        return EFillBox::kNoClip;
    }
    if box_a == EFillBox::kViewBox || box_b == EFillBox::kViewBox {
        return EFillBox::kViewBox;
    }
    if box_a == EFillBox::kStrokeBox || box_b == EFillBox::kStrokeBox {
        return EFillBox::kStrokeBox;
    }
    if box_a == EFillBox::kBorder
        || box_a == EFillBox::kText
        || box_a == EFillBox::kBorderArea
        || box_a == EFillBox::kBorderAreaText
        || box_b == EFillBox::kBorder
        || box_b == EFillBox::kText
        || box_b == EFillBox::kBorderArea
        || box_b == EFillBox::kBorderAreaText
    {
        return EFillBox::kBorder;
    }
    if box_a == EFillBox::kPadding || box_b == EFillBox::kPadding {
        return EFillBox::kPadding;
    }
    if box_a == EFillBox::kFillBox || box_b == EFillBox::kFillBox {
        return EFillBox::kFillBox;
    }
    debug_assert_eq!(box_a, EFillBox::kContent);
    debug_assert_eq!(box_b, EFillBox::kContent);
    EFillBox::kContent
}

// cpp: layoutng_style/style/computed_style_constants.h:315-320
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EFillRepeat {
    kRepeatFill,
    kNoRepeatFill,
    kRoundFill,
    kSpaceFill,
}

// cpp: layoutng_style/style/computed_style_constants.h:322
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EFillMaskMode {
    kAlpha,
    kLuminance,
    kMatchSource,
}

// cpp: layoutng_style/style/computed_style_constants.h:324
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EFillLayerType {
    kBackground,
    kMask,
}

// cpp: layoutng_style/style/computed_style_constants.h:326-332
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EFillSizeType {
    kContain,
    kCover,
    kSizeLength,
    kSizeNone,
}

// cpp: layoutng_style/style/computed_style_constants.h:334-335
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum BackgroundEdgeOrigin {
    kTop,
    kRight,
    kBottom,
    kLeft,
}

// cpp: layoutng_style/style/computed_style_constants.h:337-338
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum QuoteType {
    kOpen,
    kClose,
    kNoOpen,
    kNoClose,
}

// cpp: layoutng_style/style/computed_style_constants.h:340
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EAnimPlayState {
    kPlaying,
    kPaused,
}

// cpp: layoutng_style/style/computed_style_constants.h:342
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum OffsetRotationType {
    kAuto,
    kFixed,
}

// cpp: layoutng_style/style/computed_style_constants.h:344-364
pub const K_GRID_AUTO_FLOW_BITS: usize = 4;
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum InternalGridAutoFlowAlgorithm {
    kInternalAutoFlowAlgorithmSparse = 0x1,
    kInternalAutoFlowAlgorithmDense = 0x2,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum InternalGridAutoFlowDirection {
    kInternalAutoFlowDirectionRow = 0x4,
    kInternalAutoFlowDirectionColumn = 0x8,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum GridAutoFlow {
    kAutoFlowRow = 0x1 | 0x4,
    kAutoFlowColumn = 0x1 | 0x8,
    kAutoFlowRowDense = 0x2 | 0x4,
    kAutoFlowColumnDense = 0x2 | 0x8,
}

// cpp: layoutng_style/style/computed_style_constants.h:366-384
pub const K_CONTAINMENT_BITS: usize = 5;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Containment(i32);
#[allow(non_upper_case_globals)]
impl Containment {
    pub const kContainsNone: Self = Self(0);
    pub const kContainsLayout: Self = Self(0x1);
    pub const kContainsStyle: Self = Self(0x2);
    pub const kContainsPaint: Self = Self(0x4);
    pub const kContainsBlockSize: Self = Self(0x8);
    pub const kContainsInlineSize: Self = Self(0x10);
    pub const kContainsSize: Self = Self(0x8 | 0x10);
    pub const kContainsStrict: Self = Self(0x2 | 0x1 | 0x4 | 0x8 | 0x10);
    pub const kContainsContent: Self = Self(0x2 | 0x1 | 0x4);
    pub const fn value(self) -> i32 {
        self.0
    }
    pub const fn from_bits(bits: i32) -> Self {
        Self(bits)
    }
}
impl std::ops::BitOr for Containment {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl std::ops::BitOrAssign for Containment {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

// cpp: layoutng_style/style/computed_style_constants.h:386-400
pub const K_CONTAINER_TYPE_BITS: usize = 4;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EContainerType(i32);
#[allow(non_upper_case_globals)]
impl EContainerType {
    pub const kContainerTypeNormal: Self = Self(0);
    pub const kContainerTypeInlineSize: Self = Self(0x1);
    pub const kContainerTypeBlockSize: Self = Self(0x2);
    pub const kContainerTypeScrollState: Self = Self(0x4);
    pub const kContainerTypeAnchored: Self = Self(0x8);
    pub const kContainerTypeSize: Self = Self(0x1 | 0x2);
    pub const fn value(self) -> i32 {
        self.0
    }
    pub const fn from_bits(bits: i32) -> Self {
        Self(bits)
    }
}
impl std::ops::BitOr for EContainerType {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl std::ops::BitOrAssign for EContainerType {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

// cpp: layoutng_style/style/computed_style_constants.h:402-414
pub type MarginTrimMask = u32;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EMarginTrim(i32);
#[allow(non_upper_case_globals)]
impl EMarginTrim {
    pub const kMarginTrimNone: Self = Self(0);
    pub const kMarginTrimBlockStart: Self = Self(0x1);
    pub const kMarginTrimBlockEnd: Self = Self(0x2);
    pub const kMarginTrimBlock: Self = Self(0x1 | 0x2);
    pub const fn value(self) -> i32 {
        self.0
    }
    pub const fn from_bits(bits: i32) -> Self {
        Self(bits)
    }
}
impl std::ops::BitOr for EMarginTrim {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl std::ops::BitOrAssign for EMarginTrim {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

// cpp: layoutng_style/style/computed_style_constants.h:416-431
pub const K_TEXT_UNDERLINE_POSITION_BITS: usize = 4;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TextUnderlinePosition(u32);
#[allow(non_upper_case_globals)]
impl TextUnderlinePosition {
    pub const kAuto: Self = Self(0);
    pub const kFromFont: Self = Self(0x1);
    pub const kUnder: Self = Self(0x2);
    pub const kLeft: Self = Self(0x4);
    pub const kRight: Self = Self(0x8);
    pub const fn value(self) -> u32 {
        self.0
    }
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
}
impl std::ops::BitOr for TextUnderlinePosition {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl std::ops::BitOrAssign for TextUnderlinePosition {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

// cpp: layoutng_style/style/computed_style_constants.h:433-450
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum ItemPosition {
    kLegacy,
    kAuto,
    kNormal,
    kStretch,
    kBaseline,
    kLastBaseline,
    kAnchorCenter,
    kCenter,
    kStart,
    kEnd,
    kSelfStart,
    kSelfEnd,
    kFlexStart,
    kFlexEnd,
    kLeft,
    kRight,
}

// cpp: layoutng_style/style/computed_style_constants.h:452
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum OverflowAlignment {
    kDefault,
    kUnsafe,
    kSafe,
}

// cpp: layoutng_style/style/computed_style_constants.h:454
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum ItemPositionType {
    kNonLegacy,
    kLegacy,
}

// cpp: layoutng_style/style/computed_style_constants.h:456-467
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum ContentPosition {
    kNormal,
    kBaseline,
    kLastBaseline,
    kCenter,
    kStart,
    kEnd,
    kFlexStart,
    kFlexEnd,
    kLeft,
    kRight,
}

// cpp: layoutng_style/style/computed_style_constants.h:469-475
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum ContentDistributionType {
    kDefault,
    kSpaceBetween,
    kSpaceAround,
    kSpaceEvenly,
    kStretch,
}

// cpp: layoutng_style/style/computed_style_constants.h:477-481
#[allow(non_upper_case_globals)]
pub const kMaximumAllowedFontSize: f32 = 10000.0;

// cpp: layoutng_style/style/computed_style_constants.h:483-489
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum TextEmphasisPosition {
    kOverRight,
    kOverLeft,
    kUnderRight,
    kUnderLeft,
    kAuto,
}

// cpp: layoutng_style/style/computed_style_constants.h:491-503
#[allow(non_snake_case)]
pub fn IsOver(position: TextEmphasisPosition) -> bool {
    position == TextEmphasisPosition::kOverRight || position == TextEmphasisPosition::kOverLeft
}
#[allow(non_snake_case)]
pub fn IsRight(position: TextEmphasisPosition) -> bool {
    position == TextEmphasisPosition::kOverRight || position == TextEmphasisPosition::kUnderRight
}
#[allow(non_snake_case)]
pub fn IsLeft(position: TextEmphasisPosition) -> bool {
    !IsRight(position)
}

// cpp: layoutng_style/style/computed_style_constants.h:505-515
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum LineLogicalSide {
    kOver,
    kUnder,
}
impl PartialEq<foundation::RubyPosition> for LineLogicalSide {
    fn eq(&self, ruby_position: &foundation::RubyPosition) -> bool {
        (*self == Self::kOver && *ruby_position == foundation::RubyPosition::kOver)
            || (*self == Self::kUnder && *ruby_position == foundation::RubyPosition::kUnder)
    }
}

// cpp: layoutng_style/style/computed_style_constants.h:517-528
pub const K_SCROLLBAR_GUTTER_BITS: usize = 2;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ScrollbarGutter(i32);
#[allow(non_upper_case_globals)]
impl ScrollbarGutter {
    pub const kScrollbarGutterAuto: Self = Self(0);
    pub const kScrollbarGutterStable: Self = Self(1);
    pub const kScrollbarGutterBothEdges: Self = Self(2);
    pub const fn value(self) -> i32 {
        self.0
    }
    pub const fn from_bits(bits: i32) -> Self {
        Self(bits)
    }
}
impl std::ops::BitOr for ScrollbarGutter {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl std::ops::BitOrAssign for ScrollbarGutter {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

// cpp: layoutng_style/style/computed_style_constants.h:530
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EBaselineShiftType {
    kLength,
    kSub,
    kSuper,
}

// cpp: layoutng_style/style/computed_style_constants.h:532-537
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EPaintOrderType {
    PT_NONE = 0,
    PT_FILL = 1,
    PT_STROKE = 2,
    PT_MARKERS = 3,
}

// cpp: layoutng_style/style/computed_style_constants.h:539-547
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EPaintOrder {
    kPaintOrderNormal,
    kPaintOrderFillStrokeMarkers,
    kPaintOrderFillMarkersStroke,
    kPaintOrderStrokeFillMarkers,
    kPaintOrderStrokeMarkersFill,
    kPaintOrderMarkersFillStroke,
    kPaintOrderMarkersStrokeFill,
}

// cpp: layoutng_style/style/computed_style_constants.h:549-555
pub const K_VIEWPORT_UNIT_FLAG_BITS: usize = 2;
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum ViewportUnitFlag {
    kStatic = 0x1,
    kDynamic = 0x2,
}

// cpp: layoutng_style/style/computed_style_constants.h:557-558
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum TimelineAxis {
    kBlock,
    kInline,
    kX,
    kY,
}
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum TimelineScroller {
    kNearest,
    kRoot,
    kSelf,
}

// cpp: layoutng_style/style/computed_style_constants.h:560-568
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum ShapeBox {
    kMarginBox,
    kBorderBox,
    kPaddingBox,
    kContentBox,
}

// cpp: layoutng_style/style/computed_style_constants.h:570-580
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum CoordBox {
    kContentBox,
    kPaddingBox,
    kBorderBox,
    kFillBox,
    kStrokeBox,
    kViewBox,
}

// cpp: layoutng_style/style/computed_style_constants.h:582-594
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum GeometryBox {
    kBorderBox,
    kPaddingBox,
    kContentBox,
    kMarginBox,
    kFillBox,
    kStrokeBox,
    kViewBox,
    kHalfBorderBox,
}

// cpp: layoutng_style/style/computed_style_constants.h:596-617
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum CompositingOperator {
    kAdd,
    kSubtract,
    kIntersect,
    kExclude,
    kClear,
    kCopy,
    kSourceOver,
    kSourceIn,
    kSourceOut,
    kSourceAtop,
    kDestinationOver,
    kDestinationIn,
    kDestinationOut,
    kDestinationAtop,
    kXOR,
    kPlusLighter,
}

// cpp: layoutng_style/style/computed_style_constants.h:619-627
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum TryTactic {
    kNone,
    kFlipBlock,
    kFlipInline,
    kFlipStart,
    kFlipX,
    kFlipY,
}

// cpp: layoutng_style/style/computed_style_constants.h:629-633
#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum EAnimationTriggerBehavior {
    kPlay,
    kPause,
    kReset,
    kPlayOnce,
    kPlayForwards,
    kPlayBackwards,
    kReplay,
    kNone,
}

// cpp: layoutng_style/style/computed_style_constants.h:635-649
pub const K_POSITION_VISIBILITY_BITS: usize = 2;
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PositionVisibility(u8);
#[allow(non_upper_case_globals)]
impl PositionVisibility {
    pub const kAlways: Self = Self(0);
    pub const kAnchorsVisible: Self = Self(1);
    pub const kNoOverflow: Self = Self(2);
    pub const fn value(self) -> u8 {
        self.0
    }
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }
}
impl std::ops::BitOr for PositionVisibility {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}
impl std::ops::BitOrAssign for PositionVisibility {
    fn bitor_assign(&mut self, other: Self) {
        *self = *self | other;
    }
}

// cpp: layoutng_style/style/computed_style_constants.h:652
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum FlexWrapMode {
    kNowrap,
    kWrap,
    kWrapReverse,
}

// cpp: layoutng_style/style/computed_style_constants.h:39-42
// The C++ template accepts every enum. Each local enum opts in with its exact
// unsigned cast; external foundation enums need local impls when connected.
macro_rules! impl_enum_flag_bits {
    ($($enum_type:ty),+ $(,)?) => {
        $(impl EnumFlagBits for $enum_type {
            fn bits(self) -> u32 { self as u32 }
        })+
    };
}
impl_enum_flag_bits!(
    BoxSide,
    OutlineIsAuto,
    EVerticalAlign,
    EFillAttachment,
    EFillBox,
    EFillRepeat,
    EFillMaskMode,
    EFillLayerType,
    EFillSizeType,
    BackgroundEdgeOrigin,
    QuoteType,
    EAnimPlayState,
    OffsetRotationType,
    InternalGridAutoFlowAlgorithm,
    InternalGridAutoFlowDirection,
    GridAutoFlow,
    ItemPosition,
    OverflowAlignment,
    ItemPositionType,
    ContentPosition,
    ContentDistributionType,
    TextEmphasisPosition,
    LineLogicalSide,
    EBaselineShiftType,
    EPaintOrderType,
    EPaintOrder,
    ViewportUnitFlag,
    TimelineAxis,
    TimelineScroller,
    ShapeBox,
    CoordBox,
    GeometryBox,
    CompositingOperator,
    TryTactic,
    EAnimationTriggerBehavior,
    FlexWrapMode,
);
impl EnumFlagBits for PseudoId {
    fn bits(self) -> u32 {
        self.0 as u32
    }
}
impl EnumFlagBits for Containment {
    fn bits(self) -> u32 {
        self.0 as u32
    }
}
impl EnumFlagBits for EContainerType {
    fn bits(self) -> u32 {
        self.0 as u32
    }
}
impl EnumFlagBits for EMarginTrim {
    fn bits(self) -> u32 {
        self.0 as u32
    }
}
impl EnumFlagBits for TextUnderlinePosition {
    fn bits(self) -> u32 {
        self.0
    }
}
impl EnumFlagBits for ScrollbarGutter {
    fn bits(self) -> u32 {
        self.0 as u32
    }
}
impl EnumFlagBits for PositionVisibility {
    fn bits(self) -> u32 {
        self.0 as u32
    }
}
