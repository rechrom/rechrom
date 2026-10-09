/*
 * Copyright (C) 1999-2003 Lars Knoll (knoll@kde.org)
 *               1999 Waldo Bastian (bastian@kde.org)
 * Copyright (C) 2004, 2006, 2007, 2008, 2009, 2010, 2013 Apple Inc. All rights
 * reserved.
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

//! Blink CSSSelector data, classification, specificity, serialization and nesting.
//! Complex-selector traversal uses checked cursors into flat selector storage.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use foundation::{AtomicString, String};
use layoutng_style::style::computed_style_constants::PseudoId;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    LazyLock,
};

// cpp: css_selector.cc:377-601
pub fn GetPseudoId(pseudo_type: PseudoType) -> PseudoId {
    use PseudoType::*;
    match pseudo_type {
        kPseudoFirstLine => PseudoId::kPseudoIdFirstLine,
        kPseudoFirstLetter => PseudoId::kPseudoIdFirstLetter,
        kPseudoSelection => PseudoId::kPseudoIdSelection,
        kPseudoCheckMark => PseudoId::kPseudoIdCheckMark,
        kPseudoBefore => PseudoId::kPseudoIdBefore,
        kPseudoAfter => PseudoId::kPseudoIdAfter,
        kPseudoExpandIcon => PseudoId::kPseudoIdExpandIcon,
        kPseudoPickerIcon => PseudoId::kPseudoIdPickerIcon,
        kPseudoInterestButton => PseudoId::kPseudoIdInterestButton,
        kPseudoMarker => PseudoId::kPseudoIdMarker,
        kPseudoBackdrop => PseudoId::kPseudoIdBackdrop,
        kPseudoScrollbar => PseudoId::kPseudoIdScrollbar,
        kPseudoScrollMarker => PseudoId::kPseudoIdScrollMarker,
        kPseudoScrollMarkerGroup => PseudoId::kPseudoIdScrollMarkerGroup,
        kPseudoScrollButton => PseudoId::kPseudoIdScrollButton,
        kPseudoColumn => PseudoId::kPseudoIdColumn,
        kPseudoScrollbarButton => PseudoId::kPseudoIdScrollbarButton,
        kPseudoScrollbarCorner => PseudoId::kPseudoIdScrollbarCorner,
        kPseudoScrollbarThumb => PseudoId::kPseudoIdScrollbarThumb,
        kPseudoScrollbarTrack => PseudoId::kPseudoIdScrollbarTrack,
        kPseudoScrollbarTrackPiece => PseudoId::kPseudoIdScrollbarTrackPiece,
        kPseudoResizer => PseudoId::kPseudoIdResizer,
        kPseudoSearchText => PseudoId::kPseudoIdSearchText,
        kPseudoTargetText => PseudoId::kPseudoIdTargetText,
        kPseudoHighlight => PseudoId::kPseudoIdHighlight,
        kPseudoSpellingError => PseudoId::kPseudoIdSpellingError,
        kPseudoGrammarError => PseudoId::kPseudoIdGrammarError,
        kPseudoPlaceholder => PseudoId::kPseudoIdPlaceholder,
        kPseudoFileSelectorButton => PseudoId::kPseudoIdFileSelectorButton,
        kPseudoDetailsContent => PseudoId::kPseudoIdDetailsContent,
        kPseudoPermissionIcon => PseudoId::kPseudoIdPermissionIcon,
        kPseudoPicker => PseudoId::kPseudoIdPickerSelect,
        kPseudoSelectListbox => PseudoId::kPseudoIdSelectListbox,
        kPseudoViewTransition => PseudoId::kPseudoIdViewTransition,
        kPseudoViewTransitionGroup => PseudoId::kPseudoIdViewTransitionGroup,
        kPseudoViewTransitionGroupChildren => PseudoId::kPseudoIdViewTransitionGroupChildren,
        kPseudoViewTransitionImagePair => PseudoId::kPseudoIdViewTransitionImagePair,
        kPseudoViewTransitionOld => PseudoId::kPseudoIdViewTransitionOld,
        kPseudoViewTransitionNew => PseudoId::kPseudoIdViewTransitionNew,
        kPseudoOverscrollAreaParent => PseudoId::kPseudoIdOverscrollAreaParent,
        kPseudoOverscrollBackdrop => PseudoId::kPseudoIdOverscrollBackdrop,
        kPseudoSkeleton => PseudoId::kPseudoIdSkeleton,
        _ => PseudoId::kPseudoIdNone,
    }
}

// cpp: third_party/blink/renderer/core/css/css_selector.h:115-133
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MatchType {
    kUnknown = 0,
    kInvalidList = 1,
    kTag = 2,
    kUniversalTag = 3,
    kId = 4,
    kClass = 5,
    kPseudoClass = 6,
    kPseudoElement = 7,
    kPagePseudoClass = 8,
    kAttributeExact = 9,
    kAttributeSet = 10,
    kAttributeHyphen = 11,
    kAttributeList = 12,
    kAttributeContain = 13,
    kAttributeBegin = 14,
    kAttributeEnd = 15,
}
impl MatchType {
    fn from_bits(value: u32) -> Self {
        match value {
            0 => Self::kUnknown,
            1 => Self::kInvalidList,
            2 => Self::kTag,
            3 => Self::kUniversalTag,
            4 => Self::kId,
            5 => Self::kClass,
            6 => Self::kPseudoClass,
            7 => Self::kPseudoElement,
            8 => Self::kPagePseudoClass,
            9 => Self::kAttributeExact,
            10 => Self::kAttributeSet,
            11 => Self::kAttributeHyphen,
            12 => Self::kAttributeList,
            13 => Self::kAttributeContain,
            14 => Self::kAttributeBegin,
            15 => Self::kAttributeEnd,
            _ => unreachable!("invalid selector bit-field enum"),
        }
    }
    pub const kFirstAttributeSelectorMatch: Self = Self::kAttributeExact;
}

// cpp: third_party/blink/renderer/core/css/css_selector.h:189-237
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationType {
    kSubSelector = 0,
    kDescendant = 1,
    kChild = 2,
    kDirectAdjacent = 3,
    kIndirectAdjacent = 4,
    kPseudoChild = 5,
    kUAShadow = 6,
    kShadowSlot = 7,
    kShadowPart = 8,
    kRelativeDescendant = 9,
    kRelativeChild = 10,
    kRelativeDirectAdjacent = 11,
    kRelativeIndirectAdjacent = 12,
}
impl RelationType {
    fn from_bits(value: u32) -> Self {
        match value {
            0 => Self::kSubSelector,
            1 => Self::kDescendant,
            2 => Self::kChild,
            3 => Self::kDirectAdjacent,
            4 => Self::kIndirectAdjacent,
            5 => Self::kPseudoChild,
            6 => Self::kUAShadow,
            7 => Self::kShadowSlot,
            8 => Self::kShadowPart,
            9 => Self::kRelativeDescendant,
            10 => Self::kRelativeChild,
            11 => Self::kRelativeDirectAdjacent,
            12 => Self::kRelativeIndirectAdjacent,
            _ => unreachable!("invalid selector bit-field enum"),
        }
    }
}

// cpp: third_party/blink/renderer/core/css/css_selector.h:239-443
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PseudoType {
    kPseudoActive = 0,
    kPseudoActiveOption = 1,
    kPseudoActiveViewTransition = 2,
    kPseudoActiveViewTransitionType = 3,
    kPseudoAfter = 4,
    kPseudoAnimatedImage = 5,
    kPseudoAny = 6,
    kPseudoAnyLink = 7,
    kPseudoAutofill = 8,
    kPseudoAutofillPreviewed = 9,
    kPseudoAutofillSelected = 10,
    kPseudoBackdrop = 11,
    kPseudoBefore = 12,
    kPseudoCheckMark = 13,
    kPseudoChecked = 14,
    kPseudoCornerPresent = 15,
    kPseudoCurrent = 16,
    kPseudoDecrement = 17,
    kPseudoDefault = 18,
    kPseudoDetailsContent = 19,
    kPseudoDialogInTopLayer = 20,
    kPseudoDisabled = 21,
    kPseudoDoubleButton = 22,
    kPseudoDrag = 23,
    kPseudoEmpty = 24,
    kPseudoEnabled = 25,
    kPseudoEnd = 26,
    kPseudoExpandIcon = 27,
    kPseudoFileSelectorButton = 28,
    kPseudoFiltered = 29,
    kPseudoFirstChild = 30,
    kPseudoFirstLetter = 31,
    kPseudoFirstLine = 32,
    kPseudoFirstOfType = 33,
    kPseudoFirstPage = 34,
    kPseudoFocus = 35,
    kPseudoFocusVisible = 36,
    kPseudoFocusWithin = 37,
    kPseudoFullPageMedia = 38,
    kPseudoHasSlotted = 39,
    kPseudoHorizontal = 40,
    kPseudoHover = 41,
    kPseudoIncrement = 42,
    kPseudoIndeterminate = 43,
    kPseudoInterestButton = 44,
    kPseudoInterestSource = 45,
    kPseudoInterestTarget = 46,
    kPseudoInvalid = 47,
    kPseudoIs = 48,
    kPseudoLang = 49,
    kPseudoLastChild = 50,
    kPseudoLastOfType = 51,
    kPseudoLeftPage = 52,
    kPseudoLink = 53,
    kPseudoMarker = 54,
    kPseudoModal = 55,
    kPseudoNavigationSource = 56,
    kPseudoNoButton = 57,
    kPseudoNot = 58,
    kPseudoNthChild = 59,
    kPseudoNthLastChild = 60,
    kPseudoNthLastOfType = 61,
    kPseudoNthOfType = 62,
    kPseudoOnlyChild = 63,
    kPseudoOnlyOfType = 64,
    kPseudoOptional = 65,
    kPseudoParent = 66,
    kPseudoPart = 67,
    kPseudoPermissionGranted = 68,
    kPseudoPermissionIcon = 69,
    kPseudoPlaceholder = 70,
    kPseudoPlaceholderShown = 71,
    kPseudoReadOnly = 72,
    kPseudoReadWrite = 73,
    kPseudoRequired = 74,
    kPseudoResizer = 75,
    kPseudoRightPage = 76,
    kPseudoRoot = 77,
    kPseudoScope = 78,
    kPseudoScrollbar = 79,
    kPseudoScrollbarButton = 80,
    kPseudoScrollbarCorner = 81,
    kPseudoScrollbarThumb = 82,
    kPseudoScrollbarTrack = 83,
    kPseudoScrollbarTrackPiece = 84,
    kPseudoSearchText = 85,
    kPseudoPickerIcon = 86,
    kPseudoPicker = 87,
    kPseudoSelectListbox = 88,
    kPseudoSelectContainsInput = 89,
    kPseudoSelectHasSlottedButton = 90,
    kPseudoSelection = 91,
    kPseudoSingleButton = 92,
    kPseudoStart = 93,
    kPseudoState = 94,
    kPseudoTarget = 95,
    kPseudoTextField = 96,
    kPseudoToolFormActive = 97,
    kPseudoToolSubmitActive = 98,
    kPseudoUnknown = 99,
    kPseudoUnparsed = 100,
    kPseudoUserInvalid = 101,
    kPseudoUserValid = 102,
    kPseudoValid = 103,
    kPseudoVertical = 104,
    kPseudoVisited = 105,
    kPseudoWebKitAutofill = 106,
    kPseudoWebkitAnyLink = 107,
    kPseudoWhere = 108,
    kPseudoWindowInactive = 109,
    kPseudoFullScreen = 110,
    kPseudoFullScreenAncestor = 111,
    kPseudoFullscreen = 112,
    kPseudoInRange = 113,
    kPseudoOutOfRange = 114,
    kPseudoPictureInPicture = 115,
    kPseudoXrOverlay = 116,
    kPseudoWebKitCustomElement = 117,
    kPseudoBlinkInternalElement = 118,
    kPseudoColumn = 119,
    kPseudoCue = 120,
    kPseudoDefined = 121,
    kPseudoDir = 122,
    kPseudoFutureCue = 123,
    kPseudoGrammarError = 124,
    kPseudoHas = 125,
    kPseudoHasDatalist = 126,
    kPseudoHasOpenMenuitem = 127,
    kPseudoHighlight = 128,
    kPseudoHost = 129,
    kPseudoHostContext = 130,
    kPseudoHostHasNonAutoAppearance = 131,
    kPseudoIsHtml = 132,
    kPseudoListBox = 133,
    kPseudoMenulistPopoverWithMenubarAnchor = 134,
    kPseudoMenulistPopoverWithMenulistAnchor = 135,
    kPseudoMultiSelectFocus = 136,
    kPseudoOpen = 137,
    kPseudoPastCue = 138,
    kPseudoPopoverInTopLayer = 139,
    kPseudoPopoverOpen = 140,
    kPseudoRelativeAnchor = 141,
    kPseudoSlotted = 142,
    kPseudoSpatialNavigationFocus = 143,
    kPseudoSpellingError = 144,
    kPseudoTargetText = 145,
    kPseudoUnbounded = 146,
    kPseudoVideoPersistent = 147,
    kPseudoVideoPersistentAncestor = 148,
    kPseudoTargetAfter = 149,
    kPseudoTargetBefore = 150,
    kPseudoTargetCurrent = 151,
    kPseudoViewTransition = 152,
    kPseudoViewTransitionGroup = 153,
    kPseudoViewTransitionGroupChildren = 154,
    kPseudoViewTransitionImagePair = 155,
    kPseudoViewTransitionNew = 156,
    kPseudoViewTransitionOld = 157,
    kPseudoScrollMarker = 158,
    kPseudoScrollMarkerGroup = 159,
    kPseudoScrollButton = 160,
    kPseudoOverscrollAreaParent = 161,
    kPseudoOverscrollBackdrop = 162,
    kPseudoOverscrollOpen = 163,
    kPseudoLinkTo = 164,
    kPseudoPlaying = 165,
    kPseudoPaused = 166,
    kPseudoSeeking = 167,
    kPseudoBuffering = 168,
    kPseudoStalled = 169,
    kPseudoMuted = 170,
    kPseudoVolumeLocked = 171,
    kPseudoSkeleton = 172,
}
impl PseudoType {
    fn from_bits(value: u32) -> Self {
        match value {
            0 => Self::kPseudoActive,
            1 => Self::kPseudoActiveOption,
            2 => Self::kPseudoActiveViewTransition,
            3 => Self::kPseudoActiveViewTransitionType,
            4 => Self::kPseudoAfter,
            5 => Self::kPseudoAnimatedImage,
            6 => Self::kPseudoAny,
            7 => Self::kPseudoAnyLink,
            8 => Self::kPseudoAutofill,
            9 => Self::kPseudoAutofillPreviewed,
            10 => Self::kPseudoAutofillSelected,
            11 => Self::kPseudoBackdrop,
            12 => Self::kPseudoBefore,
            13 => Self::kPseudoCheckMark,
            14 => Self::kPseudoChecked,
            15 => Self::kPseudoCornerPresent,
            16 => Self::kPseudoCurrent,
            17 => Self::kPseudoDecrement,
            18 => Self::kPseudoDefault,
            19 => Self::kPseudoDetailsContent,
            20 => Self::kPseudoDialogInTopLayer,
            21 => Self::kPseudoDisabled,
            22 => Self::kPseudoDoubleButton,
            23 => Self::kPseudoDrag,
            24 => Self::kPseudoEmpty,
            25 => Self::kPseudoEnabled,
            26 => Self::kPseudoEnd,
            27 => Self::kPseudoExpandIcon,
            28 => Self::kPseudoFileSelectorButton,
            29 => Self::kPseudoFiltered,
            30 => Self::kPseudoFirstChild,
            31 => Self::kPseudoFirstLetter,
            32 => Self::kPseudoFirstLine,
            33 => Self::kPseudoFirstOfType,
            34 => Self::kPseudoFirstPage,
            35 => Self::kPseudoFocus,
            36 => Self::kPseudoFocusVisible,
            37 => Self::kPseudoFocusWithin,
            38 => Self::kPseudoFullPageMedia,
            39 => Self::kPseudoHasSlotted,
            40 => Self::kPseudoHorizontal,
            41 => Self::kPseudoHover,
            42 => Self::kPseudoIncrement,
            43 => Self::kPseudoIndeterminate,
            44 => Self::kPseudoInterestButton,
            45 => Self::kPseudoInterestSource,
            46 => Self::kPseudoInterestTarget,
            47 => Self::kPseudoInvalid,
            48 => Self::kPseudoIs,
            49 => Self::kPseudoLang,
            50 => Self::kPseudoLastChild,
            51 => Self::kPseudoLastOfType,
            52 => Self::kPseudoLeftPage,
            53 => Self::kPseudoLink,
            54 => Self::kPseudoMarker,
            55 => Self::kPseudoModal,
            56 => Self::kPseudoNavigationSource,
            57 => Self::kPseudoNoButton,
            58 => Self::kPseudoNot,
            59 => Self::kPseudoNthChild,
            60 => Self::kPseudoNthLastChild,
            61 => Self::kPseudoNthLastOfType,
            62 => Self::kPseudoNthOfType,
            63 => Self::kPseudoOnlyChild,
            64 => Self::kPseudoOnlyOfType,
            65 => Self::kPseudoOptional,
            66 => Self::kPseudoParent,
            67 => Self::kPseudoPart,
            68 => Self::kPseudoPermissionGranted,
            69 => Self::kPseudoPermissionIcon,
            70 => Self::kPseudoPlaceholder,
            71 => Self::kPseudoPlaceholderShown,
            72 => Self::kPseudoReadOnly,
            73 => Self::kPseudoReadWrite,
            74 => Self::kPseudoRequired,
            75 => Self::kPseudoResizer,
            76 => Self::kPseudoRightPage,
            77 => Self::kPseudoRoot,
            78 => Self::kPseudoScope,
            79 => Self::kPseudoScrollbar,
            80 => Self::kPseudoScrollbarButton,
            81 => Self::kPseudoScrollbarCorner,
            82 => Self::kPseudoScrollbarThumb,
            83 => Self::kPseudoScrollbarTrack,
            84 => Self::kPseudoScrollbarTrackPiece,
            85 => Self::kPseudoSearchText,
            86 => Self::kPseudoPickerIcon,
            87 => Self::kPseudoPicker,
            88 => Self::kPseudoSelectListbox,
            89 => Self::kPseudoSelectContainsInput,
            90 => Self::kPseudoSelectHasSlottedButton,
            91 => Self::kPseudoSelection,
            92 => Self::kPseudoSingleButton,
            93 => Self::kPseudoStart,
            94 => Self::kPseudoState,
            95 => Self::kPseudoTarget,
            96 => Self::kPseudoTextField,
            97 => Self::kPseudoToolFormActive,
            98 => Self::kPseudoToolSubmitActive,
            99 => Self::kPseudoUnknown,
            100 => Self::kPseudoUnparsed,
            101 => Self::kPseudoUserInvalid,
            102 => Self::kPseudoUserValid,
            103 => Self::kPseudoValid,
            104 => Self::kPseudoVertical,
            105 => Self::kPseudoVisited,
            106 => Self::kPseudoWebKitAutofill,
            107 => Self::kPseudoWebkitAnyLink,
            108 => Self::kPseudoWhere,
            109 => Self::kPseudoWindowInactive,
            110 => Self::kPseudoFullScreen,
            111 => Self::kPseudoFullScreenAncestor,
            112 => Self::kPseudoFullscreen,
            113 => Self::kPseudoInRange,
            114 => Self::kPseudoOutOfRange,
            115 => Self::kPseudoPictureInPicture,
            116 => Self::kPseudoXrOverlay,
            117 => Self::kPseudoWebKitCustomElement,
            118 => Self::kPseudoBlinkInternalElement,
            119 => Self::kPseudoColumn,
            120 => Self::kPseudoCue,
            121 => Self::kPseudoDefined,
            122 => Self::kPseudoDir,
            123 => Self::kPseudoFutureCue,
            124 => Self::kPseudoGrammarError,
            125 => Self::kPseudoHas,
            126 => Self::kPseudoHasDatalist,
            127 => Self::kPseudoHasOpenMenuitem,
            128 => Self::kPseudoHighlight,
            129 => Self::kPseudoHost,
            130 => Self::kPseudoHostContext,
            131 => Self::kPseudoHostHasNonAutoAppearance,
            132 => Self::kPseudoIsHtml,
            133 => Self::kPseudoListBox,
            134 => Self::kPseudoMenulistPopoverWithMenubarAnchor,
            135 => Self::kPseudoMenulistPopoverWithMenulistAnchor,
            136 => Self::kPseudoMultiSelectFocus,
            137 => Self::kPseudoOpen,
            138 => Self::kPseudoPastCue,
            139 => Self::kPseudoPopoverInTopLayer,
            140 => Self::kPseudoPopoverOpen,
            141 => Self::kPseudoRelativeAnchor,
            142 => Self::kPseudoSlotted,
            143 => Self::kPseudoSpatialNavigationFocus,
            144 => Self::kPseudoSpellingError,
            145 => Self::kPseudoTargetText,
            146 => Self::kPseudoUnbounded,
            147 => Self::kPseudoVideoPersistent,
            148 => Self::kPseudoVideoPersistentAncestor,
            149 => Self::kPseudoTargetAfter,
            150 => Self::kPseudoTargetBefore,
            151 => Self::kPseudoTargetCurrent,
            152 => Self::kPseudoViewTransition,
            153 => Self::kPseudoViewTransitionGroup,
            154 => Self::kPseudoViewTransitionGroupChildren,
            155 => Self::kPseudoViewTransitionImagePair,
            156 => Self::kPseudoViewTransitionNew,
            157 => Self::kPseudoViewTransitionOld,
            158 => Self::kPseudoScrollMarker,
            159 => Self::kPseudoScrollMarkerGroup,
            160 => Self::kPseudoScrollButton,
            161 => Self::kPseudoOverscrollAreaParent,
            162 => Self::kPseudoOverscrollBackdrop,
            163 => Self::kPseudoOverscrollOpen,
            164 => Self::kPseudoLinkTo,
            165 => Self::kPseudoPlaying,
            166 => Self::kPseudoPaused,
            167 => Self::kPseudoSeeking,
            168 => Self::kPseudoBuffering,
            169 => Self::kPseudoStalled,
            170 => Self::kPseudoMuted,
            171 => Self::kPseudoVolumeLocked,
            172 => Self::kPseudoSkeleton,
            _ => unreachable!("invalid selector bit-field enum"),
        }
    }
}

// cpp: third_party/blink/renderer/core/css/css_selector.h:445-449
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AttributeMatchType {
    kCaseSensitive = 0,
    kCaseInsensitive = 1,
    kCaseSensitiveAlways = 2,
}
impl AttributeMatchType {
    fn from_bits(value: u32) -> Self {
        match value {
            0 => Self::kCaseSensitive,
            1 => Self::kCaseInsensitive,
            2 => Self::kCaseSensitiveAlways,
            _ => unreachable!("invalid selector bit-field enum"),
        }
    }
}

// cpp: third_party/blink/renderer/core/css/css_selector.h:705-709
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LinkMatchMask {
    kMatchLink = 1,
    kMatchVisited = 2,
    kMatchAll = 3,
}

// cpp: third_party/blink/renderer/core/css/css_selector.h:172-179
pub const kIdSpecificity: u32 = 0x010000;
pub const kClassLikeSpecificity: u32 = 0x000100;
pub const kTagSpecificity: u32 = 0x000001;
pub const kMaxValueMask: u32 = 0xffffff;
pub const kIdMask: u32 = 0xff0000;
pub const kClassMask: u32 = 0x00ff00;
pub const kElementMask: u32 = 0x0000ff;

// cpp: third_party/blink/renderer/core/css/css_selector.cc:208-220
// Component saturation shared by CSSSelectorComplex::Specificity.
pub fn AccumulateSpecificity(values: impl IntoIterator<Item = u32>) -> u32 {
    let mut total: u32 = 0;
    for value in values {
        let temp = total.wrapping_add(value);
        if temp & kIdMask < total & kIdMask {
            total |= kIdMask;
        } else if temp & kClassMask < total & kClassMask {
            total |= kClassMask;
        } else if temp & kElementMask < total & kElementMask {
            total |= kElementMask;
        } else {
            total = temp;
        }
    }
    total
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:227-232,340-345
// Field extraction shared by the two tuple methods; does not calculate specificity.
pub fn SpecificityComponents(specificity: u32) -> [u8; 3] {
    [
        (specificity >> 16 & 0xff) as u8,
        (specificity >> 8 & 0xff) as u8,
        (specificity & 0xff) as u8,
    ]
}
// cpp: third_party/blink/renderer/core/css/css_selector.h:613-638
pub fn MatchNth(nth_a: i32, nth_b: i32, count: u32) -> bool {
    const MAX_VALUE: i32 = i32::MAX / 2;
    const MIN_VALUE: i32 = i32::MIN / 2;
    if count > MAX_VALUE as u32
        || nth_a > MAX_VALUE
        || nth_a < MIN_VALUE
        || nth_b > MAX_VALUE
        || nth_b < MIN_VALUE
    {
        return false;
    }
    let current_count = count as i32;
    if nth_a == 0 {
        return current_count == nth_b;
    }
    if nth_a > 0 {
        if current_count < nth_b {
            return false;
        }
        return (current_count - nth_b) % nth_a == 0;
    }
    if current_count > nth_b {
        return false;
    }
    (nth_b - current_count) % (-nth_a) == 0
}
// cpp: third_party/blink/renderer/core/css/css_selector.h:640-642
pub fn IsAdjacentRelation(relation: RelationType) -> bool {
    matches!(
        relation,
        RelationType::kDirectAdjacent | RelationType::kIndirectAdjacent
    )
}
// cpp: third_party/blink/renderer/core/css/css_selector.h:731-734
pub fn CanStoreSelectorListInline(pseudo_type: PseudoType) -> bool {
    matches!(
        pseudo_type,
        PseudoType::kPseudoIs
            | PseudoType::kPseudoWhere
            | PseudoType::kPseudoNot
            | PseudoType::kPseudoHas
    )
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:146-164
pub fn NameForInlineSelectorListPseudo(pseudo_type: PseudoType) -> &'static AtomicString {
    static IS_ATOM: LazyLock<AtomicString> = LazyLock::new(|| AtomicString::from_str("is"));
    static WHERE_ATOM: LazyLock<AtomicString> = LazyLock::new(|| AtomicString::from_str("where"));
    static NOT_ATOM: LazyLock<AtomicString> = LazyLock::new(|| AtomicString::from_str("not"));
    static HAS_ATOM: LazyLock<AtomicString> = LazyLock::new(|| AtomicString::from_str("has"));
    match pseudo_type {
        PseudoType::kPseudoIs => &IS_ATOM,
        PseudoType::kPseudoWhere => &WHERE_ATOM,
        PseudoType::kPseudoNot => &NOT_ATOM,
        PseudoType::kPseudoHas => &HAS_ATOM,
        _ => unreachable!("NOTREACHED: pseudo cannot store an inline selector list"),
    }
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:1884-2112
pub fn IsAllowedAfterPart(match_type: MatchType, pseudo_type: PseudoType) -> bool {
    if match_type != MatchType::kPseudoElement && match_type != MatchType::kPseudoClass {
        return false;
    }
    match pseudo_type {
        PseudoType::kPseudoCheckMark
        | PseudoType::kPseudoBefore
        | PseudoType::kPseudoAfter
        | PseudoType::kPseudoExpandIcon
        | PseudoType::kPseudoPickerIcon
        | PseudoType::kPseudoInterestButton
        | PseudoType::kPseudoPlaceholder
        | PseudoType::kPseudoFileSelectorButton
        | PseudoType::kPseudoFirstLine
        | PseudoType::kPseudoFirstLetter
        | PseudoType::kPseudoPicker
        | PseudoType::kPseudoSelectListbox
        | PseudoType::kPseudoSelection
        | PseudoType::kPseudoSearchText
        | PseudoType::kPseudoTargetText
        | PseudoType::kPseudoHighlight
        | PseudoType::kPseudoSpellingError
        | PseudoType::kPseudoGrammarError
        | PseudoType::kPseudoBackdrop
        | PseudoType::kPseudoOverscrollBackdrop
        | PseudoType::kPseudoCue
        | PseudoType::kPseudoMarker
        | PseudoType::kPseudoResizer
        | PseudoType::kPseudoScrollbar
        | PseudoType::kPseudoScrollbarButton
        | PseudoType::kPseudoScrollbarCorner
        | PseudoType::kPseudoScrollbarThumb
        | PseudoType::kPseudoScrollbarTrack
        | PseudoType::kPseudoScrollbarTrackPiece
        | PseudoType::kPseudoScrollMarker
        | PseudoType::kPseudoScrollMarkerGroup
        | PseudoType::kPseudoScrollButton
        | PseudoType::kPseudoColumn
        | PseudoType::kPseudoWebKitCustomElement
        | PseudoType::kPseudoBlinkInternalElement
        | PseudoType::kPseudoDetailsContent
        | PseudoType::kPseudoPermissionIcon
        | PseudoType::kPseudoViewTransition
        | PseudoType::kPseudoViewTransitionGroup
        | PseudoType::kPseudoViewTransitionGroupChildren
        | PseudoType::kPseudoViewTransitionImagePair
        | PseudoType::kPseudoViewTransitionNew
        | PseudoType::kPseudoViewTransitionOld
        | PseudoType::kPseudoOverscrollAreaParent
        | PseudoType::kPseudoSkeleton => true,
        PseudoType::kPseudoSlotted => false,
        PseudoType::kPseudoPart => false,
        PseudoType::kPseudoAnimatedImage
        | PseudoType::kPseudoAutofill
        | PseudoType::kPseudoAutofillPreviewed
        | PseudoType::kPseudoAutofillSelected
        | PseudoType::kPseudoWebKitAutofill
        | PseudoType::kPseudoActive
        | PseudoType::kPseudoActiveOption
        | PseudoType::kPseudoActiveViewTransition
        | PseudoType::kPseudoActiveViewTransitionType
        | PseudoType::kPseudoAnyLink
        | PseudoType::kPseudoBuffering
        | PseudoType::kPseudoChecked
        | PseudoType::kPseudoDefault
        | PseudoType::kPseudoDialogInTopLayer
        | PseudoType::kPseudoDisabled
        | PseudoType::kPseudoDrag
        | PseudoType::kPseudoEnabled
        | PseudoType::kPseudoFiltered
        | PseudoType::kPseudoFocus
        | PseudoType::kPseudoFocusVisible
        | PseudoType::kPseudoFocusWithin
        | PseudoType::kPseudoFullPageMedia
        | PseudoType::kPseudoHasSlotted
        | PseudoType::kPseudoHover
        | PseudoType::kPseudoIndeterminate
        | PseudoType::kPseudoInterestSource
        | PseudoType::kPseudoInterestTarget
        | PseudoType::kPseudoInvalid
        | PseudoType::kPseudoLang
        | PseudoType::kPseudoLink
        | PseudoType::kPseudoLinkTo
        | PseudoType::kPseudoMenulistPopoverWithMenubarAnchor
        | PseudoType::kPseudoMenulistPopoverWithMenulistAnchor
        | PseudoType::kPseudoModal
        | PseudoType::kPseudoMuted
        | PseudoType::kPseudoOptional
        | PseudoType::kPseudoOverscrollOpen
        | PseudoType::kPseudoPermissionGranted
        | PseudoType::kPseudoPlaceholderShown
        | PseudoType::kPseudoReadOnly
        | PseudoType::kPseudoReadWrite
        | PseudoType::kPseudoRequired
        | PseudoType::kPseudoSeeking
        | PseudoType::kPseudoSelectContainsInput
        | PseudoType::kPseudoSelectHasSlottedButton
        | PseudoType::kPseudoStalled
        | PseudoType::kPseudoState
        | PseudoType::kPseudoTarget
        | PseudoType::kPseudoUserInvalid
        | PseudoType::kPseudoUserValid
        | PseudoType::kPseudoValid
        | PseudoType::kPseudoVisited
        | PseudoType::kPseudoVolumeLocked
        | PseudoType::kPseudoWebkitAnyLink
        | PseudoType::kPseudoWindowInactive
        | PseudoType::kPseudoFullScreen
        | PseudoType::kPseudoFullScreenAncestor
        | PseudoType::kPseudoFullscreen
        | PseudoType::kPseudoInRange
        | PseudoType::kPseudoOutOfRange
        | PseudoType::kPseudoPaused
        | PseudoType::kPseudoPictureInPicture
        | PseudoType::kPseudoPlaying
        | PseudoType::kPseudoXrOverlay
        | PseudoType::kPseudoDefined
        | PseudoType::kPseudoDir
        | PseudoType::kPseudoFutureCue
        | PseudoType::kPseudoIsHtml
        | PseudoType::kPseudoListBox
        | PseudoType::kPseudoMultiSelectFocus
        | PseudoType::kPseudoNavigationSource
        | PseudoType::kPseudoOpen
        | PseudoType::kPseudoPastCue
        | PseudoType::kPseudoPopoverInTopLayer
        | PseudoType::kPseudoPopoverOpen
        | PseudoType::kPseudoRelativeAnchor
        | PseudoType::kPseudoSpatialNavigationFocus
        | PseudoType::kPseudoTargetCurrent
        | PseudoType::kPseudoTargetBefore
        | PseudoType::kPseudoTargetAfter
        | PseudoType::kPseudoTextField
        | PseudoType::kPseudoToolFormActive
        | PseudoType::kPseudoToolSubmitActive
        | PseudoType::kPseudoUnbounded
        | PseudoType::kPseudoVideoPersistent
        | PseudoType::kPseudoVideoPersistentAncestor => true,
        PseudoType::kPseudoIs | PseudoType::kPseudoNot | PseudoType::kPseudoWhere => true,
        PseudoType::kPseudoAny => false,
        PseudoType::kPseudoParent => false,
        PseudoType::kPseudoHorizontal
        | PseudoType::kPseudoVertical
        | PseudoType::kPseudoDecrement
        | PseudoType::kPseudoIncrement
        | PseudoType::kPseudoStart
        | PseudoType::kPseudoEnd
        | PseudoType::kPseudoDoubleButton
        | PseudoType::kPseudoSingleButton
        | PseudoType::kPseudoNoButton
        | PseudoType::kPseudoCornerPresent
        | PseudoType::kPseudoCurrent => false,
        PseudoType::kPseudoFirstPage
        | PseudoType::kPseudoLeftPage
        | PseudoType::kPseudoRightPage => false,
        PseudoType::kPseudoEmpty
        | PseudoType::kPseudoFirstChild
        | PseudoType::kPseudoFirstOfType
        | PseudoType::kPseudoLastChild
        | PseudoType::kPseudoLastOfType
        | PseudoType::kPseudoNthChild
        | PseudoType::kPseudoNthLastChild
        | PseudoType::kPseudoNthLastOfType
        | PseudoType::kPseudoNthOfType
        | PseudoType::kPseudoOnlyChild
        | PseudoType::kPseudoOnlyOfType
        | PseudoType::kPseudoRoot => false,
        PseudoType::kPseudoHas
        | PseudoType::kPseudoHasDatalist
        | PseudoType::kPseudoHasOpenMenuitem
        | PseudoType::kPseudoHost
        | PseudoType::kPseudoHostContext
        | PseudoType::kPseudoHostHasNonAutoAppearance
        | PseudoType::kPseudoScope => false,
        PseudoType::kPseudoUnparsed | PseudoType::kPseudoUnknown => false,
    }
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:2261-2277
pub fn IsChildIndexedSelector(pseudo_type: PseudoType) -> bool {
    matches!(
        pseudo_type,
        PseudoType::kPseudoFirstChild
            | PseudoType::kPseudoFirstOfType
            | PseudoType::kPseudoLastChild
            | PseudoType::kPseudoLastOfType
            | PseudoType::kPseudoNthChild
            | PseudoType::kPseudoNthLastChild
            | PseudoType::kPseudoNthLastOfType
            | PseudoType::kPseudoNthOfType
            | PseudoType::kPseudoOnlyChild
            | PseudoType::kPseudoOnlyOfType
    )
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:2297-2393
pub fn SupportsPseudoStateChange(pseudo_type: PseudoType) -> bool {
    matches!(
        pseudo_type,
        PseudoType::kPseudoAnimatedImage
            | PseudoType::kPseudoActive
            | PseudoType::kPseudoActiveOption
            | PseudoType::kPseudoActiveViewTransition
            | PseudoType::kPseudoActiveViewTransitionType
            | PseudoType::kPseudoAnyLink
            | PseudoType::kPseudoAutofill
            | PseudoType::kPseudoAutofillPreviewed
            | PseudoType::kPseudoAutofillSelected
            | PseudoType::kPseudoBuffering
            | PseudoType::kPseudoChecked
            | PseudoType::kPseudoDefault
            | PseudoType::kPseudoDefined
            | PseudoType::kPseudoDialogInTopLayer
            | PseudoType::kPseudoDir
            | PseudoType::kPseudoDisabled
            | PseudoType::kPseudoDrag
            | PseudoType::kPseudoEmpty
            | PseudoType::kPseudoEnabled
            | PseudoType::kPseudoFiltered
            | PseudoType::kPseudoFirstChild
            | PseudoType::kPseudoFirstOfType
            | PseudoType::kPseudoFocus
            | PseudoType::kPseudoFocusVisible
            | PseudoType::kPseudoFocusWithin
            | PseudoType::kPseudoFullScreen
            | PseudoType::kPseudoFullScreenAncestor
            | PseudoType::kPseudoFullscreen
            | PseudoType::kPseudoHas
            | PseudoType::kPseudoHasDatalist
            | PseudoType::kPseudoHasOpenMenuitem
            | PseudoType::kPseudoHasSlotted
            | PseudoType::kPseudoHover
            | PseudoType::kPseudoInRange
            | PseudoType::kPseudoIndeterminate
            | PseudoType::kPseudoInterestSource
            | PseudoType::kPseudoInterestTarget
            | PseudoType::kPseudoInvalid
            | PseudoType::kPseudoLang
            | PseudoType::kPseudoLastChild
            | PseudoType::kPseudoLastOfType
            | PseudoType::kPseudoLink
            | PseudoType::kPseudoLinkTo
            | PseudoType::kPseudoListBox
            | PseudoType::kPseudoModal
            | PseudoType::kPseudoMultiSelectFocus
            | PseudoType::kPseudoMuted
            | PseudoType::kPseudoNavigationSource
            | PseudoType::kPseudoNthChild
            | PseudoType::kPseudoNthLastChild
            | PseudoType::kPseudoNthLastOfType
            | PseudoType::kPseudoNthOfType
            | PseudoType::kPseudoOnlyChild
            | PseudoType::kPseudoOnlyOfType
            | PseudoType::kPseudoOpen
            | PseudoType::kPseudoOptional
            | PseudoType::kPseudoOutOfRange
            | PseudoType::kPseudoPaused
            | PseudoType::kPseudoPermissionGranted
            | PseudoType::kPseudoPictureInPicture
            | PseudoType::kPseudoPlaceholderShown
            | PseudoType::kPseudoPlaying
            | PseudoType::kPseudoPopoverInTopLayer
            | PseudoType::kPseudoPopoverOpen
            | PseudoType::kPseudoReadOnly
            | PseudoType::kPseudoReadWrite
            | PseudoType::kPseudoRequired
            | PseudoType::kPseudoSeeking
            | PseudoType::kPseudoSelectContainsInput
            | PseudoType::kPseudoSelectHasSlottedButton
            | PseudoType::kPseudoSelection
            | PseudoType::kPseudoStalled
            | PseudoType::kPseudoState
            | PseudoType::kPseudoTarget
            | PseudoType::kPseudoTargetAfter
            | PseudoType::kPseudoTargetBefore
            | PseudoType::kPseudoTargetCurrent
            | PseudoType::kPseudoTextField
            | PseudoType::kPseudoToolFormActive
            | PseudoType::kPseudoToolSubmitActive
            | PseudoType::kPseudoUnbounded
            | PseudoType::kPseudoUserInvalid
            | PseudoType::kPseudoUserValid
            | PseudoType::kPseudoValid
            | PseudoType::kPseudoVideoPersistent
            | PseudoType::kPseudoVideoPersistentAncestor
            | PseudoType::kPseudoVisited
            | PseudoType::kPseudoWebKitAutofill
            | PseudoType::kPseudoWebkitAnyLink
            | PseudoType::kPseudoXrOverlay
    )
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:1873-1877
pub fn IsElementBackedPseudoElement(pseudo: PseudoType) -> bool {
    matches!(
        pseudo,
        PseudoType::kPseudoDetailsContent
            | PseudoType::kPseudoPicker
            | PseudoType::kPseudoPermissionIcon
            | PseudoType::kPseudoSelectListbox
    )
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:1850-1871
pub fn IsTreeAbidingPseudoElement(match_type: MatchType, pseudo: PseudoType) -> bool {
    match_type == MatchType::kPseudoElement
        && (IsElementBackedPseudoElement(pseudo)
            || matches!(
                pseudo,
                PseudoType::kPseudoCheckMark
                    | PseudoType::kPseudoBefore
                    | PseudoType::kPseudoAfter
                    | PseudoType::kPseudoExpandIcon
                    | PseudoType::kPseudoPickerIcon
                    | PseudoType::kPseudoInterestButton
                    | PseudoType::kPseudoMarker
                    | PseudoType::kPseudoPlaceholder
                    | PseudoType::kPseudoFileSelectorButton
                    | PseudoType::kPseudoBackdrop
                    | PseudoType::kPseudoOverscrollBackdrop
                    | PseudoType::kPseudoViewTransition
                    | PseudoType::kPseudoViewTransitionGroup
                    | PseudoType::kPseudoViewTransitionGroupChildren
                    | PseudoType::kPseudoViewTransitionImagePair
                    | PseudoType::kPseudoViewTransitionOld
                    | PseudoType::kPseudoViewTransitionNew
                    | PseudoType::kPseudoOverscrollAreaParent
                    | PseudoType::kPseudoSkeleton
            ))
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:2279-2294
pub fn ConvertRelationToRelative(relation: RelationType) -> RelationType {
    use RelationType::*;
    match relation {
        kSubSelector | kDescendant => kRelativeDescendant,
        kChild => kRelativeChild,
        kDirectAdjacent => kRelativeDirectAdjacent,
        kIndirectAdjacent => kRelativeIndirectAdjacent,
        _ => unreachable!("NOTREACHED: relation has no relative combinator"),
    }
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:645-797
pub(crate) const kPseudoTypeWithoutArgumentsMap: &[(&str, PseudoType)] = &[
    (
        "-internal-autofill-previewed",
        PseudoType::kPseudoAutofillPreviewed,
    ),
    (
        "-internal-autofill-selected",
        PseudoType::kPseudoAutofillSelected,
    ),
    (
        "-internal-dialog-in-top-layer",
        PseudoType::kPseudoDialogInTopLayer,
    ),
    ("-internal-has-datalist", PseudoType::kPseudoHasDatalist),
    (
        "-internal-has-open-menuitem",
        PseudoType::kPseudoHasOpenMenuitem,
    ),
    ("-internal-is-html", PseudoType::kPseudoIsHtml),
    ("-internal-list-box", PseudoType::kPseudoListBox),
    (
        "-internal-media-controls-overlay-cast-button",
        PseudoType::kPseudoWebKitCustomElement,
    ),
    (
        "-internal-menulist-popover-with-menubar-anchor",
        PseudoType::kPseudoMenulistPopoverWithMenubarAnchor,
    ),
    (
        "-internal-menulist-popover-with-menulist-anchor",
        PseudoType::kPseudoMenulistPopoverWithMenulistAnchor,
    ),
    (
        "-internal-multi-select-focus",
        PseudoType::kPseudoMultiSelectFocus,
    ),
    (
        "-internal-popover-in-top-layer",
        PseudoType::kPseudoPopoverInTopLayer,
    ),
    (
        "-internal-relative-anchor",
        PseudoType::kPseudoRelativeAnchor,
    ),
    (
        "-internal-select-contains-input",
        PseudoType::kPseudoSelectContainsInput,
    ),
    (
        "-internal-select-has-slotted-button",
        PseudoType::kPseudoSelectHasSlottedButton,
    ),
    (
        "-internal-shadow-host-has-non-auto-appearance",
        PseudoType::kPseudoHostHasNonAutoAppearance,
    ),
    (
        "-internal-spatial-navigation-focus",
        PseudoType::kPseudoSpatialNavigationFocus,
    ),
    ("-internal-text-field", PseudoType::kPseudoTextField),
    (
        "-internal-video-persistent",
        PseudoType::kPseudoVideoPersistent,
    ),
    (
        "-internal-video-persistent-ancestor",
        PseudoType::kPseudoVideoPersistentAncestor,
    ),
    ("-webkit-any-link", PseudoType::kPseudoWebkitAnyLink),
    ("-webkit-autofill", PseudoType::kPseudoWebKitAutofill),
    ("-webkit-drag", PseudoType::kPseudoDrag),
    ("-webkit-full-page-media", PseudoType::kPseudoFullPageMedia),
    ("-webkit-full-screen", PseudoType::kPseudoFullScreen),
    (
        "-webkit-full-screen-ancestor",
        PseudoType::kPseudoFullScreenAncestor,
    ),
    ("-webkit-resizer", PseudoType::kPseudoResizer),
    ("-webkit-scrollbar", PseudoType::kPseudoScrollbar),
    (
        "-webkit-scrollbar-button",
        PseudoType::kPseudoScrollbarButton,
    ),
    (
        "-webkit-scrollbar-corner",
        PseudoType::kPseudoScrollbarCorner,
    ),
    ("-webkit-scrollbar-thumb", PseudoType::kPseudoScrollbarThumb),
    ("-webkit-scrollbar-track", PseudoType::kPseudoScrollbarTrack),
    (
        "-webkit-scrollbar-track-piece",
        PseudoType::kPseudoScrollbarTrackPiece,
    ),
    ("active", PseudoType::kPseudoActive),
    ("active-option", PseudoType::kPseudoActiveOption),
    (
        "active-view-transition",
        PseudoType::kPseudoActiveViewTransition,
    ),
    ("after", PseudoType::kPseudoAfter),
    ("animated-image", PseudoType::kPseudoAnimatedImage),
    ("any-link", PseudoType::kPseudoAnyLink),
    ("autofill", PseudoType::kPseudoAutofill),
    ("backdrop", PseudoType::kPseudoBackdrop),
    ("before", PseudoType::kPseudoBefore),
    ("buffering", PseudoType::kPseudoBuffering),
    ("checked", PseudoType::kPseudoChecked),
    ("checkmark", PseudoType::kPseudoCheckMark),
    ("column", PseudoType::kPseudoColumn),
    ("corner-present", PseudoType::kPseudoCornerPresent),
    ("cue", PseudoType::kPseudoWebKitCustomElement),
    ("current", PseudoType::kPseudoCurrent),
    ("decrement", PseudoType::kPseudoDecrement),
    ("default", PseudoType::kPseudoDefault),
    ("defined", PseudoType::kPseudoDefined),
    ("details-content", PseudoType::kPseudoDetailsContent),
    ("disabled", PseudoType::kPseudoDisabled),
    ("double-button", PseudoType::kPseudoDoubleButton),
    ("empty", PseudoType::kPseudoEmpty),
    ("enabled", PseudoType::kPseudoEnabled),
    ("end", PseudoType::kPseudoEnd),
    ("expand-icon", PseudoType::kPseudoExpandIcon),
    (
        "file-selector-button",
        PseudoType::kPseudoFileSelectorButton,
    ),
    ("filtered", PseudoType::kPseudoFiltered),
    ("first", PseudoType::kPseudoFirstPage),
    ("first-child", PseudoType::kPseudoFirstChild),
    ("first-letter", PseudoType::kPseudoFirstLetter),
    ("first-line", PseudoType::kPseudoFirstLine),
    ("first-of-type", PseudoType::kPseudoFirstOfType),
    ("focus", PseudoType::kPseudoFocus),
    ("focus-visible", PseudoType::kPseudoFocusVisible),
    ("focus-within", PseudoType::kPseudoFocusWithin),
    ("fullscreen", PseudoType::kPseudoFullscreen),
    ("future", PseudoType::kPseudoFutureCue),
    ("grammar-error", PseudoType::kPseudoGrammarError),
    ("granted", PseudoType::kPseudoPermissionGranted),
    ("has-slotted", PseudoType::kPseudoHasSlotted),
    ("horizontal", PseudoType::kPseudoHorizontal),
    ("host", PseudoType::kPseudoHost),
    ("hover", PseudoType::kPseudoHover),
    ("in-range", PseudoType::kPseudoInRange),
    ("increment", PseudoType::kPseudoIncrement),
    ("indeterminate", PseudoType::kPseudoIndeterminate),
    ("interest-button", PseudoType::kPseudoInterestButton),
    ("interest-source", PseudoType::kPseudoInterestSource),
    ("interest-target", PseudoType::kPseudoInterestTarget),
    ("invalid", PseudoType::kPseudoInvalid),
    ("last-child", PseudoType::kPseudoLastChild),
    ("last-of-type", PseudoType::kPseudoLastOfType),
    ("left", PseudoType::kPseudoLeftPage),
    ("link", PseudoType::kPseudoLink),
    ("marker", PseudoType::kPseudoMarker),
    ("modal", PseudoType::kPseudoModal),
    ("muted", PseudoType::kPseudoMuted),
    ("navigation-source", PseudoType::kPseudoNavigationSource),
    ("no-button", PseudoType::kPseudoNoButton),
    ("only-child", PseudoType::kPseudoOnlyChild),
    ("only-of-type", PseudoType::kPseudoOnlyOfType),
    ("open", PseudoType::kPseudoOpen),
    ("optional", PseudoType::kPseudoOptional),
    ("out-of-range", PseudoType::kPseudoOutOfRange),
    ("overscroll-backdrop", PseudoType::kPseudoOverscrollBackdrop),
    ("overscroll-open", PseudoType::kPseudoOverscrollOpen),
    ("past", PseudoType::kPseudoPastCue),
    ("paused", PseudoType::kPseudoPaused),
    ("permission-icon", PseudoType::kPseudoPermissionIcon),
    ("picker-icon", PseudoType::kPseudoPickerIcon),
    ("picture-in-picture", PseudoType::kPseudoPictureInPicture),
    ("placeholder", PseudoType::kPseudoPlaceholder),
    ("placeholder-shown", PseudoType::kPseudoPlaceholderShown),
    ("playing", PseudoType::kPseudoPlaying),
    ("popover-open", PseudoType::kPseudoPopoverOpen),
    ("read-only", PseudoType::kPseudoReadOnly),
    ("read-write", PseudoType::kPseudoReadWrite),
    ("required", PseudoType::kPseudoRequired),
    ("right", PseudoType::kPseudoRightPage),
    ("root", PseudoType::kPseudoRoot),
    ("scope", PseudoType::kPseudoScope),
    ("scroll-marker", PseudoType::kPseudoScrollMarker),
    ("scroll-marker-group", PseudoType::kPseudoScrollMarkerGroup),
    ("search-text", PseudoType::kPseudoSearchText),
    ("seeking", PseudoType::kPseudoSeeking),
    ("select-listbox", PseudoType::kPseudoSelectListbox),
    ("selection", PseudoType::kPseudoSelection),
    ("single-button", PseudoType::kPseudoSingleButton),
    ("skeleton", PseudoType::kPseudoSkeleton),
    ("spelling-error", PseudoType::kPseudoSpellingError),
    ("stalled", PseudoType::kPseudoStalled),
    ("start", PseudoType::kPseudoStart),
    ("target", PseudoType::kPseudoTarget),
    ("target-after", PseudoType::kPseudoTargetAfter),
    ("target-before", PseudoType::kPseudoTargetBefore),
    ("target-current", PseudoType::kPseudoTargetCurrent),
    ("target-text", PseudoType::kPseudoTargetText),
    ("tool-form-active", PseudoType::kPseudoToolFormActive),
    ("tool-submit-active", PseudoType::kPseudoToolSubmitActive),
    ("unbounded", PseudoType::kPseudoUnbounded),
    ("user-invalid", PseudoType::kPseudoUserInvalid),
    ("user-valid", PseudoType::kPseudoUserValid),
    ("valid", PseudoType::kPseudoValid),
    ("vertical", PseudoType::kPseudoVertical),
    ("view-transition", PseudoType::kPseudoViewTransition),
    ("visited", PseudoType::kPseudoVisited),
    ("volume-locked", PseudoType::kPseudoVolumeLocked),
    ("window-inactive", PseudoType::kPseudoWindowInactive),
    ("xr-overlay", PseudoType::kPseudoXrOverlay),
];
// cpp: third_party/blink/renderer/core/css/css_selector.cc:799-831
pub(crate) const kPseudoTypeWithArgumentsMap: &[(&str, PseudoType)] = &[
    (
        "-internal-overscroll-area-parent",
        PseudoType::kPseudoOverscrollAreaParent,
    ),
    ("-webkit-any", PseudoType::kPseudoAny),
    (
        "active-view-transition-type",
        PseudoType::kPseudoActiveViewTransitionType,
    ),
    ("cue", PseudoType::kPseudoCue),
    ("dir", PseudoType::kPseudoDir),
    ("has", PseudoType::kPseudoHas),
    ("highlight", PseudoType::kPseudoHighlight),
    ("host", PseudoType::kPseudoHost),
    ("host-context", PseudoType::kPseudoHostContext),
    ("is", PseudoType::kPseudoIs),
    ("lang", PseudoType::kPseudoLang),
    ("link-to", PseudoType::kPseudoLinkTo),
    ("not", PseudoType::kPseudoNot),
    ("nth-child", PseudoType::kPseudoNthChild),
    ("nth-last-child", PseudoType::kPseudoNthLastChild),
    ("nth-last-of-type", PseudoType::kPseudoNthLastOfType),
    ("nth-of-type", PseudoType::kPseudoNthOfType),
    ("part", PseudoType::kPseudoPart),
    ("picker", PseudoType::kPseudoPicker),
    ("scroll-button", PseudoType::kPseudoScrollButton),
    ("slotted", PseudoType::kPseudoSlotted),
    ("state", PseudoType::kPseudoState),
    (
        "view-transition-group",
        PseudoType::kPseudoViewTransitionGroup,
    ),
    (
        "view-transition-group-children",
        PseudoType::kPseudoViewTransitionGroupChildren,
    ),
    (
        "view-transition-image-pair",
        PseudoType::kPseudoViewTransitionImagePair,
    ),
    ("view-transition-new", PseudoType::kPseudoViewTransitionNew),
    ("view-transition-old", PseudoType::kPseudoViewTransitionOld),
    ("where", PseudoType::kPseudoWhere),
];

// cpp: third_party/blink/renderer/core/css/css_selector.cc:2173-2188
pub fn FormatPseudoTypeForDebugging(pseudo_type: PseudoType) -> String {
    for &(name, value) in kPseudoTypeWithoutArgumentsMap {
        if value == pseudo_type {
            return String::FromUtf8(name.as_bytes());
        }
    }
    for &(name, value) in kPseudoTypeWithArgumentsMap {
        if value == pseudo_type {
            return String::FromUtf8(name.as_bytes());
        }
    }
    String::FromUtf8(format!("pseudo-{}", pseudo_type as u32).as_bytes())
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:2394-2406
const fn NameMapIsSorted(map: &[(&str, PseudoType)]) -> bool {
    let mut entry = 1;
    while entry < map.len() {
        let a = map[entry - 1].0.as_bytes();
        let b = map[entry].0.as_bytes();
        let mut i = 0;
        while i < a.len() && i < b.len() && a[i] == b[i] {
            i += 1;
        }
        if (i < a.len() && i < b.len() && a[i] > b[i]) || (i == b.len() && a.len() > b.len()) {
            return false;
        }
        entry += 1;
    }
    true
}
const _: () = assert!(NameMapIsSorted(kPseudoTypeWithoutArgumentsMap));
const _: () = assert!(NameMapIsSorted(kPseudoTypeWithArgumentsMap));

// cpp: third_party/blink/renderer/core/css/css_selector.cc:1286-1294
// Vec<u16> is the StringBuilder output buffer; no lossy Unicode projection.
pub fn SerializeIdentifierOrAny(
    identifier: &AtomicString,
    any: &AtomicString,
    builder: &mut Vec<u16>,
) {
    if identifier != any {
        crate::css_markup::SerializeIdentifierTo(
            &String::from_utf16(identifier.utf16_units().unwrap_or_default()),
            builder,
            false,
        );
    } else {
        builder.push(b'*' as u16);
    }
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:1296-1305
pub fn SerializeNamespacePrefixIfNeeded(
    prefix: &AtomicString,
    any: &AtomicString,
    builder: &mut Vec<u16>,
    is_attribute_selector: bool,
) {
    if prefix.IsNull() || (prefix.empty() && is_attribute_selector) {
        return;
    }
    SerializeIdentifierOrAny(prefix, any, builder);
    builder.push(b'|' as u16);
}
// cpp: third_party/blink/renderer/core/css/css_selector.cc:1307-1318
pub fn SerializeIdentifierList<'a>(
    builder: &mut Vec<u16>,
    list: impl IntoIterator<Item = &'a AtomicString>,
) {
    let mut is_first = true;
    for item in list {
        if !is_first {
            builder.extend(", ".encode_utf16());
        }
        crate::css_markup::SerializeIdentifierTo(
            &String::from_utf16(item.utf16_units().unwrap_or_default()),
            builder,
            false,
        );
        is_first = false;
    }
}
// cpp: third_party/blink/renderer/core/css/css_selector.h:1008-1010
pub fn IsASCIILower(value: &AtomicString) -> bool {
    value.ContainsNoAsciiUpper()
}
// cpp: third_party/blink/renderer/core/css/css_selector.h:519
// Returning an interned null atom preserves the distinction from an empty name.
pub fn UniversalSelectorAtom() -> &'static AtomicString {
    static NULL_ATOM: LazyLock<AtomicString> = LazyLock::new(AtomicString::default);
    &NULL_ATOM
}

// cpp: third_party/blink/renderer/core/css/css_selector.h:792-844
// cpp dependency: third_party/blink/renderer/platform/wtf/bit_field.h:61-99,148-160
// Exactly the source field offsets: relation[0..4], match[4..8], pseudo[8..16],
// then selector-list end, complex-selector end, rare data, page, implicit,
// bucketing, attribute match[22..24], legacy case, scope, inline list[26].
// CSSSelector owns these flags alongside its active DataUnion member.
#[repr(transparent)]
pub struct CSSSelectorBits {
    bits_: AtomicU32,
}
impl Clone for CSSSelectorBits {
    fn clone(&self) -> Self {
        Self {
            bits_: AtomicU32::new(self.bits_.load(Ordering::Relaxed)),
        }
    }
}
impl Default for CSSSelectorBits {
    // cpp: third_party/blink/renderer/core/css/css_selector.h:1033-1042
    // The complete selector initializer is CSSSelector::default below.
    fn default() -> Self {
        Self {
            bits_: AtomicU32::new((PseudoType::kPseudoUnknown as u32) << 8),
        }
    }
}
impl CSSSelectorBits {
    fn get(&self, offset: u32, width: u32) -> u32 {
        (self.bits_.load(Ordering::Relaxed) >> offset) & ((1 << width) - 1)
    }
    fn set(&mut self, offset: u32, width: u32, value: u32) {
        let mask = (1 << width) - 1;
        debug_assert_eq!(value & !mask, 0);
        let previous = *self.bits_.get_mut();
        self.bits_.store(
            (previous & !(mask << offset)) | (value << offset),
            Ordering::Relaxed,
        );
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.h:451-456
    pub fn GetPseudoType(&self) -> PseudoType {
        PseudoType::from_bits(self.get(8, 8))
    }
    pub fn GetPseudoTypeForOilpan(&self) -> PseudoType {
        self.GetPseudoType()
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.h:847-850
    pub fn SetPseudoType(&mut self, pseudo_type: PseudoType) {
        self.set(8, 8, pseudo_type as u32);
        debug_assert_eq!(self.GetPseudoType(), pseudo_type);
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.h:660-678
    pub fn Relation(&self) -> RelationType {
        RelationType::from_bits(self.get(0, 4))
    }
    pub fn SetRelation(&mut self, relation: RelationType) {
        self.set(0, 4, relation as u32);
        debug_assert_eq!(self.Relation(), relation);
    }
    pub fn Match(&self) -> MatchType {
        MatchType::from_bits(self.get(4, 4))
    }
    pub fn MatchForOilpan(&self) -> MatchType {
        self.Match()
    }
    pub fn SetMatch(&mut self, match_type: MatchType) {
        self.set(4, 4, match_type as u32);
        debug_assert_eq!(self.Match(), match_type);
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.h:680-695
    pub fn IsLastInSelectorList(&self) -> bool {
        self.get(16, 1) != 0
    }
    pub fn IsLastInSelectorListForOilpan(&self) -> bool {
        self.IsLastInSelectorList()
    }
    pub fn SetLastInSelectorList(&mut self, is_last: bool) {
        self.set(16, 1, is_last as u32);
    }
    pub fn IsLastInComplexSelector(&self) -> bool {
        self.get(17, 1) != 0
    }
    pub fn SetLastInComplexSelector(&mut self, is_last: bool) {
        self.set(17, 1, is_last as u32);
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.h:717-726
    pub fn HasRareData(&self) -> bool {
        self.get(18, 1) != 0
    }
    pub fn HasRareDataForOilpan(&self) -> bool {
        self.HasRareData()
    }
    pub fn HasInlineSelectorList(&self) -> bool {
        self.get(26, 1) != 0
    }
    pub fn HasInlineSelectorListForOilpan(&self) -> bool {
        self.HasInlineSelectorList()
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.h:740-753,768
    pub fn IsForPage(&self) -> bool {
        self.get(19, 1) != 0
    }
    pub fn SetForPage(&mut self) {
        self.set(19, 1, 1);
    }
    pub fn IsCoveredByBucketing(&self) -> bool {
        self.get(21, 1) != 0
    }
    pub fn SetCoveredByBucketing(&self, value: bool) {
        // RuleSet construction marks selectors through a shared StyleRule
        // handle. Chromium mutates this single bookkeeping bit through its GC
        // pointer; keep the same shared mutation without making selector data
        // or the remaining flags interior-mutable.
        const MASK: u32 = 1 << 21;
        if value {
            self.bits_.fetch_or(MASK, Ordering::Relaxed);
        } else {
            self.bits_.fetch_and(!MASK, Ordering::Relaxed);
        }
    }
    pub fn IsScopeContaining(&self) -> bool {
        self.get(25, 1) != 0
    }
    pub fn SetScopeContaining(&mut self, value: bool) {
        self.set(25, 1, value as u32);
    }
    pub fn IsImplicit(&self) -> bool {
        self.get(20, 1) != 0
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.h:643-649,774-776
    pub fn IsAttributeSelector(&self) -> bool {
        self.Match() >= MatchType::kFirstAttributeSelectorMatch
    }
    pub fn IsHostPseudoClass(&self) -> bool {
        matches!(
            self.GetPseudoType(),
            PseudoType::kPseudoHost | PseudoType::kPseudoHostContext
        )
    }
    pub fn IsPseudoParent(&self) -> bool {
        self.Match() == MatchType::kPseudoClass && self.GetPseudoType() == PseudoType::kPseudoParent
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.h:998-1006
    pub fn AttributeMatch(&self) -> AttributeMatchType {
        debug_assert!(self.IsAttributeSelector());
        AttributeMatchType::from_bits(self.get(22, 2))
    }
    pub fn LegacyCaseInsensitiveMatch(&self) -> bool {
        debug_assert!(self.IsAttributeSelector());
        self.get(24, 1) != 0
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.h:1179-1184
    pub fn IsUserActionPseudoClass(&self) -> bool {
        matches!(
            self.GetPseudoType(),
            PseudoType::kPseudoHover
                | PseudoType::kPseudoActive
                | PseudoType::kPseudoFocus
                | PseudoType::kPseudoDrag
                | PseudoType::kPseudoFocusWithin
                | PseudoType::kPseudoFocusVisible
        )
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.h:1186-1189
    pub fn IsIdClassOrAttributeSelector(&self) -> bool {
        self.IsAttributeSelector()
            || self.Match() == MatchType::kId
            || self.Match() == MatchType::kClass
    }
    // cpp: third_party/blink/renderer/core/css/css_selector.cc:1879-1882
    pub fn IsElementBackedPseudoElement(&self) -> bool {
        self.Match() == MatchType::kPseudoElement
            && IsElementBackedPseudoElement(self.GetPseudoType())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn field_offsets_and_selector_kind_guards_match_source() {
        let mut bits = CSSSelectorBits::default();
        assert_eq!(bits.Relation(), RelationType::kSubSelector);
        assert_eq!(bits.Match(), MatchType::kUnknown);
        assert_eq!(bits.GetPseudoType(), PseudoType::kPseudoUnknown);
        assert!(!bits.HasRareData() && !bits.HasInlineSelectorList());
        bits.SetMatch(MatchType::kAttributeEnd);
        bits.SetRelation(RelationType::kRelativeIndirectAdjacent);
        bits.SetPseudoType(PseudoType::kPseudoSkeleton);
        bits.SetLastInComplexSelector(true);
        bits.SetLastInSelectorList(true);
        bits.SetForPage();
        bits.SetCoveredByBucketing(true);
        bits.SetScopeContaining(true);
        let expected = RelationType::kRelativeIndirectAdjacent as u32
            | (MatchType::kAttributeEnd as u32) << 4
            | (PseudoType::kPseudoSkeleton as u32) << 8
            | 1 << 16
            | 1 << 17
            | 1 << 19
            | 1 << 21
            | 1 << 25;
        assert_eq!(bits.bits_.load(Ordering::Relaxed), expected);
        assert!(bits.IsAttributeSelector() && bits.IsIdClassOrAttributeSelector());
        bits.SetMatch(MatchType::kPseudoClass);
        bits.SetPseudoType(PseudoType::kPseudoParent);
        assert!(bits.IsPseudoParent());
        bits.SetPseudoType(PseudoType::kPseudoHover);
        assert!(bits.IsUserActionPseudoClass());
        bits.SetPseudoType(PseudoType::kPseudoHas);
        assert!(!bits.IsUserActionPseudoClass());
        assert!(bits.IsLastInSelectorListForOilpan());
        assert_eq!(bits.GetPseudoTypeForOilpan(), bits.GetPseudoType());
        assert_eq!(bits.MatchForOilpan(), bits.Match());
    }
    #[test]
    fn nth_limits_sign_and_specificity_overflow_preserve_original_rules() {
        assert!(MatchNth(0, 4, 4));
        assert!(!MatchNth(0, 4, 5));
        assert!(MatchNth(2, -1, 3));
        assert!(!MatchNth(2, 2, 1));
        assert!(MatchNth(-2, 5, 3));
        assert!(!MatchNth(-2, 5, 6));
        assert!(MatchNth(-1, -1, 0) == false);
        assert!(!MatchNth(i32::MIN, 0, 0));
        assert!(!MatchNth(1, i32::MAX, 1));
        assert!(!MatchNth(1, 0, (i32::MAX / 2 + 1) as u32));
        assert!(MatchNth(i32::MIN / 2, 0, 0));
        assert_eq!(AccumulateSpecificity([0x00ffff, 1]), 0x00ffff);
        assert_eq!(AccumulateSpecificity([0x00ff00, 0x100]), 0x00ff00);
        assert_eq!(AccumulateSpecificity([0xff0000, 0x10000]), 0xff0000);
        assert_eq!(SpecificityComponents(0x070b13), [7, 11, 19]);
    }
    #[test]
    fn classification_and_debugging_preserve_context_and_alias_order() {
        use PseudoType::*;
        assert!(IsAllowedAfterPart(MatchType::kPseudoClass, kPseudoHover));
        assert!(!IsAllowedAfterPart(
            MatchType::kPseudoClass,
            kPseudoNthChild
        ));
        assert!(!IsAllowedAfterPart(MatchType::kClass, kPseudoHover));
        assert!(!IsAllowedAfterPart(MatchType::kPseudoElement, kPseudoPart));
        assert!(IsAllowedAfterPart(
            MatchType::kPseudoElement,
            kPseudoSkeleton
        ));
        assert!(IsTreeAbidingPseudoElement(
            MatchType::kPseudoElement,
            kPseudoBefore
        ));
        assert!(!IsTreeAbidingPseudoElement(
            MatchType::kPseudoClass,
            kPseudoBefore
        ));
        assert!(IsElementBackedPseudoElement(kPseudoDetailsContent));
        assert!(SupportsPseudoStateChange(kPseudoHover));
        assert!(!SupportsPseudoStateChange(kPseudoUnknown));
        assert_eq!(GetPseudoId(kPseudoBefore), PseudoId::kPseudoIdBefore);
        assert_eq!(GetPseudoId(kPseudoHover), PseudoId::kPseudoIdNone);
        assert!(IsChildIndexedSelector(kPseudoNthLastOfType));
        assert!(!IsChildIndexedSelector(kPseudoEmpty));
        assert_eq!(
            FormatPseudoTypeForDebugging(kPseudoWebKitCustomElement).as_str(),
            "-internal-media-controls-overlay-cast-button"
        );
        assert_eq!(FormatPseudoTypeForDebugging(kPseudoHost).as_str(), "host");
        assert_eq!(
            FormatPseudoTypeForDebugging(kPseudoParent).as_str(),
            format!("pseudo-{}", kPseudoParent as u32)
        );
        assert_eq!(
            NameForInlineSelectorListPseudo(kPseudoIs),
            &AtomicString::from_str("is")
        );
        assert!(UniversalSelectorAtom().IsNull());
        assert_eq!(
            ConvertRelationToRelative(RelationType::kSubSelector),
            RelationType::kRelativeDescendant
        );
        assert!(!IsAdjacentRelation(RelationType::kRelativeDirectAdjacent));
    }
    #[test]
    fn namespace_and_identifier_serialization_preserve_null_empty_and_utf16() {
        let null = AtomicString::default();
        let empty = AtomicString::from_str("");
        let any = AtomicString::from_str("*");
        let mut output = Vec::new();
        SerializeNamespacePrefixIfNeeded(&null, &any, &mut output, false);
        SerializeNamespacePrefixIfNeeded(&empty, &any, &mut output, true);
        assert!(output.is_empty());
        SerializeNamespacePrefixIfNeeded(&empty, &any, &mut output, false);
        SerializeNamespacePrefixIfNeeded(&any, &any, &mut output, true);
        assert_eq!(String::from_utf16(&output).as_str(), "|*|");
        output.clear();
        let names = [
            AtomicString::from_str("1x"),
            AtomicString::from_utf16(&[0xd800]),
        ];
        SerializeIdentifierList(&mut output, &names);
        let mut expected: Vec<u16> = "\\31 x, ".encode_utf16().collect();
        expected.push(0xd800);
        assert_eq!(output, expected);
        assert!(IsASCIILower(&names[1]));
        assert!(!IsASCIILower(&AtomicString::from_str("A")));
    }
}
// cpp: css_selector.h:461-479, css_selector.cc:832-982
// Feature and document queries are required; there are deliberately no defaults.
pub trait CSSSelectorRuntime {
    fn CSSMediaElementPseudosEnabled(&self) -> bool;
    fn GeolocationElementEnabled(&self) -> bool;
    fn UserMediaElementEnabled(&self) -> bool;
    fn InstallElementEnabled(&self) -> bool;
    fn CSSPseudoScrollMarkersEnabled(&self) -> bool;
    fn CSSScrollMarkerTargetBeforeAfterEnabled(&self) -> bool;
    fn CSSPseudoScrollButtonsEnabled(&self) -> bool;
    fn CSSPseudoColumnEnabled(&self) -> bool;
    fn SearchTextHighlightPseudoEnabled(&self) -> bool;
    fn HasDocument(&self) -> bool;
    fn WebMCPEnabled(&self) -> bool;
    fn UnboundedElementEnabled(&self) -> bool;
    fn CSSPseudoHasSlottedEnabled(&self) -> bool;
    fn OverscrollGesturesEnabled(&self) -> bool;
    fn CustomizableComboboxEnabled(&self) -> bool;
    fn FilterableSelectEnabled(&self) -> bool;
    fn CSSImageAnimationEnabled(&self) -> bool;
    fn MenuElementsEnabled(&self) -> bool;
    fn NavigationSourcePseudoClassEnabled(&self) -> bool;
    fn DeclarativeSkeletonsEnabled(&self) -> bool;
}
pub trait CSSSelectorParserContext: CSSSelectorRuntime {
    // CSSSelectorParser::ParsePseudoType (includes -webkit-/internal handling).
    fn ParsePseudoType(&self, name: &AtomicString, has_arguments: bool) -> PseudoType;
}
pub trait CSSSelectorAttributeContext {
    fn IsCaseSensitiveAttribute(&self, attribute: &QualifiedName) -> bool;
}
pub trait CSSSelectorParentRule {
    // Returns the actual flattened selector storage of the StyleRule.
    fn Selectors(&self) -> &CSSSelectorList;
}
pub trait CSSSelectorNavigationLocation {
    fn SerializeTo(&self, output: &mut Vec<u16>);
}

use crate::css_selector_list::CSSSelectorList;
use crate::parser::css_nesting_type::CSSNestingType;
use std::cell::{Ref, RefCell};
use std::ops::Deref;
use std::rc::Rc;

// QualifiedName's three actual atom components. Selector storage never reduces
// a name to a local-name string: prefix/null/namespace are retained separately.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QualifiedName {
    prefix_: AtomicString,
    local_name_: AtomicString,
    namespace_uri_: AtomicString,
}
impl QualifiedName {
    pub fn new(
        prefix: AtomicString,
        local_name: AtomicString,
        namespace_uri: AtomicString,
    ) -> Self {
        Self {
            prefix_: prefix,
            local_name_: local_name,
            namespace_uri_: namespace_uri,
        }
    }
    pub fn Prefix(&self) -> &AtomicString {
        &self.prefix_
    }
    pub fn LocalName(&self) -> &AtomicString {
        &self.local_name_
    }
    pub fn NamespaceURI(&self) -> &AtomicString {
        &self.namespace_uri_
    }
    pub fn AnyQName() -> Self {
        Self::new(
            AtomicString::default(),
            AtomicString::default(),
            AtomicString::from_str("*"),
        )
    }
}

// cpp: css_selector.h:879-925; byte storage preserves the overlapping nth/has/
// nesting fields of RareData::bits_ rather than assigning independent flags.
#[derive(Clone, Default)]
struct RareDataBits([u8; 8]);
impl RareDataBits {
    fn NthAValue(&self) -> i32 {
        i32::from_ne_bytes(self.0[..4].try_into().unwrap())
    }
    fn NthBValue(&self) -> i32 {
        i32::from_ne_bytes(self.0[4..].try_into().unwrap())
    }
    fn SetNth(&mut self, a: i32, b: i32) {
        self.0[..4].copy_from_slice(&a.to_ne_bytes());
        self.0[4..].copy_from_slice(&b.to_ne_bytes());
    }
    fn nesting(&self) -> CSSNestingType {
        match self.NthAValue() {
            0 => CSSNestingType::kNone,
            1 => CSSNestingType::kScope,
            2 => CSSNestingType::kNesting,
            3 => CSSNestingType::kFunction,
            4 => CSSNestingType::kMixin,
            _ => panic!("invalid RareData nesting tag"),
        }
    }
}
struct RareData {
    matching_value_: AtomicString,
    serializing_value_: AtomicString,
    bits_: RareDataBits,
    attribute_: QualifiedName,
    argument_: AtomicString,
    argument_list_: Option<Vec<AtomicString>>,
    selector_list_: Option<Rc<CSSSelectorList>>,
    navigation_location_: Option<Rc<dyn CSSSelectorNavigationLocation>>,
    ident_list_: Option<Vec<AtomicString>>,
}
impl RareData {
    // cpp: css_selector.cc:2190-2195
    fn new(value: AtomicString) -> Self {
        Self {
            matching_value_: value.clone(),
            serializing_value_: value,
            bits_: RareDataBits::default(),
            attribute_: QualifiedName::AnyQName(),
            argument_: AtomicString::default(),
            argument_list_: None,
            selector_list_: None,
            navigation_location_: None,
            ident_list_: None,
        }
    }
    // cpp: css_selector.cc:2197-2206. The source copy constructor intentionally
    // omits argument_list_ and navigation_location_; preserve that behavior.
    fn CopyForRenest(&self) -> Self {
        Self {
            matching_value_: self.matching_value_.clone(),
            serializing_value_: self.serializing_value_.clone(),
            bits_: self.bits_.clone(),
            attribute_: self.attribute_.clone(),
            argument_: self.argument_.clone(),
            argument_list_: None,
            selector_list_: self.selector_list_.clone(),
            navigation_location_: None,
            ident_list_: self.ident_list_.clone(),
        }
    }
}
// cpp: css_selector.h:938-987. Rust owns the active union member explicitly;
// the source flags remain the same and rare/list/rule copies preserve identity.
#[derive(Clone)]
enum DataUnion {
    Value(AtomicString),
    QualifiedName(QualifiedName),
    RareData(Rc<RefCell<RareData>>),
    ParentRule(Option<Rc<dyn CSSSelectorParentRule>>),
    SelectorList(Option<Rc<CSSSelectorList>>),
}
#[derive(Clone)]
pub struct CSSSelector {
    bits_: CSSSelectorBits,
    data_: DataUnion,
}
impl Deref for CSSSelector {
    type Target = CSSSelectorBits;
    fn deref(&self) -> &Self::Target {
        &self.bits_
    }
}
impl Default for CSSSelector {
    // cpp: css_selector.h:1033-1048
    fn default() -> Self {
        Self {
            bits_: CSSSelectorBits::default(),
            data_: DataUnion::Value(AtomicString::default()),
        }
    }
}

// Atom access is a borrowed reference into either the union or shared RareData.
// RefCell makes the source's shared mutable RareData observable after copying.
pub enum SelectorValue<'a> {
    Direct(&'a AtomicString),
    Rare(Ref<'a, AtomicString>),
}
impl Deref for SelectorValue<'_> {
    type Target = AtomicString;
    fn deref(&self) -> &AtomicString {
        match self {
            Self::Direct(v) => v,
            Self::Rare(v) => v,
        }
    }
}
pub enum SelectorQName<'a> {
    Direct(&'a QualifiedName),
    Rare(Ref<'a, QualifiedName>),
}
impl Deref for SelectorQName<'_> {
    type Target = QualifiedName;
    fn deref(&self) -> &QualifiedName {
        match self {
            Self::Direct(v) => v,
            Self::Rare(v) => v,
        }
    }
}
impl CSSSelector {
    // cpp: css_selector.h:1050-1070
    pub fn FromTag(tag: QualifiedName, implicit: bool) -> Self {
        let mut s = Self::default();
        s.SetMatch(
            if tag == QualifiedName::AnyQName() || tag.LocalName().IsNull() {
                MatchType::kUniversalTag
            } else {
                MatchType::kTag
            },
        );
        s.bits_.set(20, 1, implicit as u32);
        s.data_ = DataUnion::QualifiedName(tag);
        s
    }
    // cpp: css_selector.h:1072-1089
    pub fn FromParent(parent: Option<Rc<dyn CSSSelectorParentRule>>, implicit: bool) -> Self {
        let mut s = Self::default();
        s.SetMatch(MatchType::kPseudoClass);
        s.SetPseudoType(PseudoType::kPseudoParent);
        s.bits_.set(20, 1, implicit as u32);
        s.data_ = DataUnion::ParentRule(parent);
        s
    }
    // cpp: css_selector.h:1091-1112
    pub fn FromPseudo(
        name: AtomicString,
        implicit: bool,
        runtime: &impl CSSSelectorRuntime,
    ) -> Self {
        let mut s = Self::default();
        s.SetMatch(MatchType::kPseudoClass);
        s.SetPseudoType(Self::NameToPseudoType(&name, false, runtime));
        s.bits_.set(20, 1, implicit as u32);
        s.data_ = DataUnion::Value(name);
        s
    }
    // cpp: css_selector.cc:102-144
    pub fn FromAttribute(
        match_type: MatchType,
        attribute: QualifiedName,
        sensitivity: AttributeMatchType,
        value: Option<AtomicString>,
        context: &impl CSSSelectorAttributeContext,
    ) -> Self {
        let mut s = Self::default();
        s.SetMatch(match_type);
        assert!(s.IsAttributeSelector());
        s.bits_.set(22, 2, sensitivity as u32);
        s.bits_.set(
            24,
            1,
            (!context.IsCaseSensitiveAttribute(&attribute)
                && sensitivity != AttributeMatchType::kCaseSensitiveAlways) as u32,
        );
        if let Some(value) = value {
            let mut rare = RareData::new(value);
            rare.attribute_ = attribute;
            s.data_ = DataUnion::RareData(Rc::new(RefCell::new(rare)));
            s.bits_.set(18, 1, 1);
        } else {
            assert_eq!(match_type, MatchType::kAttributeSet);
            s.data_ = DataUnion::QualifiedName(attribute);
        }
        s
    }
    pub fn SetMatch(&mut self, value: MatchType) {
        self.bits_.SetMatch(value);
    }
    pub fn SetPseudoType(&mut self, value: PseudoType) {
        self.bits_.SetPseudoType(value);
    }
    pub fn SetRelation(&mut self, value: RelationType) {
        self.bits_.SetRelation(value);
    }
    pub fn SetLastInSelectorList(&mut self, value: bool) {
        self.bits_.SetLastInSelectorList(value);
    }
    pub fn SetLastInComplexSelector(&mut self, value: bool) {
        self.bits_.SetLastInComplexSelector(value);
    }
    pub fn SetForPage(&mut self) {
        self.bits_.SetForPage();
    }
    pub fn SetCoveredByBucketing(&self, value: bool) {
        self.bits_.SetCoveredByBucketing(value);
    }
    pub fn SetScopeContaining(&mut self, value: bool) {
        self.bits_.SetScopeContaining(value);
    }
    // cpp: css_selector.h:1139-1149
    pub fn TagQName(&self) -> &QualifiedName {
        match &self.data_ {
            DataUnion::QualifiedName(q)
                if matches!(self.Match(), MatchType::kTag | MatchType::kUniversalTag) =>
            {
                q
            }
            _ => panic!("TagQName on a non-tag selector"),
        }
    }
    pub fn ParentRule(&self) -> Option<&Rc<dyn CSSSelectorParentRule>> {
        match &self.data_ {
            DataUnion::ParentRule(p) => p.as_ref(),
            _ => panic!("ParentRule on a non-parent selector"),
        }
    }
    // cpp: css_selector.h:1151-1177
    pub fn Value(&self) -> SelectorValue<'_> {
        self.value(false)
    }
    pub fn SerializingValue(&self) -> SelectorValue<'_> {
        self.value(true)
    }
    fn value(&self, serializing: bool) -> SelectorValue<'_> {
        match &self.data_ {
            DataUnion::Value(v) => SelectorValue::Direct(v),
            DataUnion::RareData(r) => SelectorValue::Rare(Ref::map(r.borrow(), |r| {
                if serializing {
                    &r.serializing_value_
                } else {
                    &r.matching_value_
                }
            })),
            DataUnion::SelectorList(_) => {
                SelectorValue::Direct(NameForInlineSelectorListPseudo(self.GetPseudoType()))
            }
            _ => panic!("Value on a tag/parent/valueless attribute selector"),
        }
    }
    // cpp: css_selector.h:989-997, 546-567
    pub fn Attribute(&self) -> SelectorQName<'_> {
        assert!(self.IsAttributeSelector());
        match &self.data_ {
            DataUnion::QualifiedName(q) => SelectorQName::Direct(q),
            DataUnion::RareData(r) => SelectorQName::Rare(Ref::map(r.borrow(), |r| &r.attribute_)),
            _ => unreachable!(),
        }
    }
    fn rare(&self) -> &RefCell<RareData> {
        match &self.data_ {
            DataUnion::RareData(r) => r,
            _ => panic!("selector has no RareData"),
        }
    }
    pub fn Argument(&self) -> SelectorValue<'_> {
        if self.HasRareData() {
            SelectorValue::Rare(Ref::map(self.rare().borrow(), |r| &r.argument_))
        } else {
            SelectorValue::Direct(UniversalSelectorAtom())
        }
    }
    pub fn ArgumentList(&self) -> Option<Ref<'_, Vec<AtomicString>>> {
        if !self.HasRareData() {
            return None;
        }
        Ref::filter_map(self.rare().borrow(), |r| r.argument_list_.as_ref()).ok()
    }
    pub fn IdentList(&self) -> Ref<'_, Vec<AtomicString>> {
        Ref::map(self.rare().borrow(), |r| {
            r.ident_list_.as_ref().expect("missing ident list")
        })
    }
    pub fn SelectorList(&self) -> Option<Rc<CSSSelectorList>> {
        match &self.data_ {
            DataUnion::RareData(r) => r.borrow().selector_list_.clone(),
            DataUnion::SelectorList(l) => l.clone(),
            _ => None,
        }
    }
    pub fn GetNavigationLocation(&self) -> Option<Rc<dyn CSSSelectorNavigationLocation>> {
        if self.HasRareData() {
            self.rare().borrow().navigation_location_.clone()
        } else {
            None
        }
    }
    pub fn NthAValue(&self) -> u32 {
        assert_eq!(self.GetPseudoType(), PseudoType::kPseudoNthChild);
        self.rare().borrow().bits_.NthAValue() as u32
    }
    pub fn NthBValue(&self) -> u32 {
        assert_eq!(self.GetPseudoType(), PseudoType::kPseudoNthChild);
        self.rare().borrow().bits_.NthBValue() as u32
    }
    pub fn ContainsPseudoInsideHasPseudoClass(&self) -> bool {
        self.HasRareData() && self.rare().borrow().bits_.0[0] != 0
    }
    pub fn ContainsComplexLogicalCombinationsInsideHasPseudoClass(&self) -> bool {
        self.HasRareData() && self.rare().borrow().bits_.0[1] != 0
    }
    pub fn HasArgumentMatchInShadowTree(&self) -> bool {
        self.HasRareData() && self.rare().borrow().bits_.0[2] != 0
    }
    // cpp: css_selector.cc:165-199
    fn CreateRareData(&mut self) {
        if self.HasRareData() {
            return;
        }
        let mut rare = match &self.data_ {
            DataUnion::Value(v) => RareData::new(v.clone()),
            DataUnion::SelectorList(_) => {
                RareData::new(NameForInlineSelectorListPseudo(self.GetPseudoType()).clone())
            }
            _ => panic!("cannot create RareData for this union member"),
        };
        if let DataUnion::SelectorList(list) = &self.data_ {
            rare.selector_list_ = list.clone();
        }
        self.bits_.set(26, 1, 0);
        self.data_ = DataUnion::RareData(Rc::new(RefCell::new(rare)));
        self.bits_.set(18, 1, 1);
    }
    // cpp: css_selector.h:1012-1031
    pub fn SetValue(&mut self, value: AtomicString, match_lower_case: bool) {
        assert!(
            !matches!(self.Match(), MatchType::kTag | MatchType::kUniversalTag)
                && !self.IsPseudoParent()
        );
        if self.HasInlineSelectorList()
            || (match_lower_case && !self.HasRareData() && !IsASCIILower(&value))
        {
            self.CreateRareData();
        }
        if !self.HasRareData() {
            self.data_ = DataUnion::Value(value);
            return;
        }
        let mut r = self.rare().borrow_mut();
        r.matching_value_ = if match_lower_case {
            value.ToAsciiLower()
        } else {
            value.clone()
        };
        r.serializing_value_ = value;
    }
    // cpp: css_selector.cc:1671-1720,2226-2230
    pub fn SetArgument(&mut self, value: AtomicString) {
        self.CreateRareData();
        self.rare().borrow_mut().argument_ = value;
    }
    pub fn SetArgumentList(&mut self, list: Option<Vec<AtomicString>>) {
        self.CreateRareData();
        self.rare().borrow_mut().argument_list_ = list;
    }
    pub fn SetIdentList(&mut self, list: Option<Vec<AtomicString>>) {
        self.CreateRareData();
        self.rare().borrow_mut().ident_list_ = list;
    }
    pub fn SetSelectorList(&mut self, list: Option<Rc<CSSSelectorList>>) {
        if self.HasInlineSelectorList() {
            self.data_ = DataUnion::SelectorList(list);
            return;
        }
        if !self.HasRareData()
            && self.Match() == MatchType::kPseudoClass
            && CanStoreSelectorListInline(self.GetPseudoType())
            && *self.Value() == *NameForInlineSelectorListPseudo(self.GetPseudoType())
        {
            self.data_ = DataUnion::SelectorList(list);
            self.bits_.set(26, 1, 1);
            return;
        }
        self.CreateRareData();
        self.rare().borrow_mut().selector_list_ = list;
    }
    pub fn SetNavigationLocation(
        &mut self,
        location: Option<Rc<dyn CSSSelectorNavigationLocation>>,
    ) {
        self.CreateRareData();
        self.rare().borrow_mut().navigation_location_ = location;
    }
    pub fn SetContainsPseudoInsideHasPseudoClass(&mut self) {
        self.CreateRareData();
        self.rare().borrow_mut().bits_.0[0] = 1;
    }
    pub fn SetContainsComplexLogicalCombinationsInsideHasPseudoClass(&mut self) {
        self.CreateRareData();
        self.rare().borrow_mut().bits_.0[1] = 1;
    }
    pub fn SetHasArgumentMatchInShadowTree(&mut self) {
        self.CreateRareData();
        self.rare().borrow_mut().bits_.0[2] = 1;
    }
    // cpp: css_selector.cc:1815-1825
    pub fn SetNth(&mut self, a: i32, b: i32, list: Option<Rc<CSSSelectorList>>) {
        self.CreateRareData();
        let mut r = self.rare().borrow_mut();
        r.bits_.SetNth(a, b);
        r.selector_list_ = list;
    }
    pub fn MatchNth(&self, count: u32) -> bool {
        let r = self.rare().borrow();
        MatchNth(r.bits_.NthAValue(), r.bits_.NthBValue(), count)
    }
    // cpp: css_selector.cc:1257-1285; css_selector.h:463-466
    pub fn SetUnparsedPlaceholder(&mut self, nesting: CSSNestingType, value: AtomicString) {
        self.SetPseudoType(PseudoType::kPseudoUnparsed);
        self.CreateRareData();
        self.SetValue(value, false);
        self.rare().borrow_mut().bits_.0[..4].copy_from_slice(&(nesting as i32).to_ne_bytes());
    }
    pub fn GetNestingType(&self) -> CSSNestingType {
        match self.GetPseudoType() {
            PseudoType::kPseudoParent => CSSNestingType::kNesting,
            PseudoType::kPseudoUnparsed => self.rare().borrow().bits_.nesting(),
            PseudoType::kPseudoScope => CSSNestingType::kScope,
            _ => CSSNestingType::kNone,
        }
    }
    pub fn IsUnparsedInvalid(&self) -> bool {
        self.GetPseudoType() == PseudoType::kPseudoUnparsed
            && self.GetNestingType() == CSSNestingType::kNone
    }
    pub fn SetWhere(&mut self, list: Rc<CSSSelectorList>) {
        self.SetMatch(MatchType::kPseudoClass);
        self.SetPseudoType(PseudoType::kPseudoWhere);
        self.SetSelectorList(Some(list));
    }
    // cpp: css_selector.cc:2247-2259
    pub fn SelectorListOrParent(&self) -> Option<SelectorListRef<'_>> {
        if self.IsPseudoParent() {
            self.ParentRule()
                .map(|p| SelectorListRef::Parent(p.Selectors()))
        } else {
            self.SelectorList().map(SelectorListRef::Owned)
        }
    }
    // cpp: css_selector.cc:603-635,2215-2224. Unchanged lists/rules retain Rc identity.
    pub fn Renest(&self, parent: Option<Rc<dyn CSSSelectorParentRule>>) -> Option<Self> {
        if self.GetPseudoType() == PseudoType::kPseudoParent {
            let old = self.ParentRule();
            let same = match (old, parent.as_ref()) {
                (None, None) => true,
                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                _ => false,
            };
            if !same {
                let mut s = self.clone();
                s.data_ = DataUnion::ParentRule(parent);
                return Some(s);
            }
        } else if self.HasRareData() {
            let old = self.rare().borrow();
            if let Some(list) = &old.selector_list_ {
                if let Some(new_list) = list.RenestChanged(parent) {
                    let mut r = old.CopyForRenest();
                    r.selector_list_ = Some(Rc::new(new_list));
                    let mut s = self.clone();
                    s.data_ = DataUnion::RareData(Rc::new(RefCell::new(r)));
                    return Some(s);
                }
            }
        } else if let DataUnion::SelectorList(Some(list)) = &self.data_ {
            if let Some(new_list) = list.RenestChanged(parent) {
                let mut s = self.clone();
                s.data_ = DataUnion::SelectorList(Some(Rc::new(new_list)));
                return Some(s);
            }
        }
        None
    }
    // cpp: css_selector.cc:257-344
    pub fn SpecificityForOneSelector(&self) -> u32 {
        use MatchType::*;
        use PseudoType::*;
        let maximum = || self.SelectorList().map_or(0, |l| l.MaximumSpecificity());
        match self.Match() {
            kId => kIdSpecificity,
            kPseudoClass => match self.GetPseudoType() {
                kPseudoWhere | kPseudoRelativeAnchor => 0,
                kPseudoHost if self.SelectorList().is_none() => kClassLikeSpecificity,
                kPseudoHost | kPseudoHostContext => {
                    kClassLikeSpecificity
                        + self
                            .SelectorList()
                            .expect("host selector list")
                            .First()
                            .expect("host selector")
                            .Specificity()
                }
                kPseudoNot | kPseudoIs | kPseudoHas => maximum(),
                kPseudoParent => self.ParentRule().map_or(0, |p| {
                    p.Selectors()
                        .ComplexSelectors()
                        .filter(|s| s.IsAllowedInParentPseudo())
                        .map(|s| s.Specificity())
                        .max()
                        .unwrap_or(0)
                }),
                kPseudoNthChild | kPseudoNthLastChild => kClassLikeSpecificity + maximum(),
                kPseudoScope if self.IsImplicit() => 0,
                _ => kClassLikeSpecificity,
            },
            kPseudoElement => match self.GetPseudoType() {
                kPseudoSlotted => {
                    kTagSpecificity
                        + self
                            .SelectorList()
                            .expect("slotted selector list")
                            .First()
                            .expect("slotted selector")
                            .Specificity()
                }
                kPseudoViewTransitionGroup
                | kPseudoViewTransitionGroupChildren
                | kPseudoViewTransitionImagePair
                | kPseudoViewTransitionOld
                | kPseudoViewTransitionNew => {
                    let ids = self.IdentList();
                    assert!(!ids.is_empty());
                    if ids.len() == 1 && ids[0].IsNull() {
                        0
                    } else {
                        kTagSpecificity
                    }
                }
                _ => kTagSpecificity,
            },
            kClass | kAttributeExact | kAttributeSet | kAttributeList | kAttributeHyphen
            | kAttributeContain | kAttributeBegin | kAttributeEnd => kClassLikeSpecificity,
            kTag => kTagSpecificity,
            kUniversalTag | kUnknown => 0,
            kInvalidList | kPagePseudoClass => panic!("invalid specificity match kind"),
        }
    }
    pub fn SimpleSelectorSpecificityTuple(&self) -> [u8; 3] {
        SpecificityComponents(self.SpecificityForOneSelector())
    }
    pub fn IsTreeAbidingPseudoElement(&self) -> bool {
        IsTreeAbidingPseudoElement(self.Match(), self.GetPseudoType())
    }
    pub fn IsAllowedAfterPart(&self) -> bool {
        IsAllowedAfterPart(self.Match(), self.GetPseudoType())
    }
    pub fn IsChildIndexedSelector(&self) -> bool {
        IsChildIndexedSelector(self.GetPseudoType())
    }
    // cpp: css_selector.cc:2114-2138
    pub fn IsOrContainsHostPseudoClass(&self) -> bool {
        self.IsHostPseudoClass()
            || self.SelectorListOrParent().is_some_and(|l| {
                l.ComplexSelectors()
                    .any(|s| s.IsOrContainsHostPseudoClass())
            })
    }
    pub fn IsDeeplyHostPseudoClass(&self) -> bool {
        if matches!(
            self.GetPseudoType(),
            PseudoType::kPseudoIs | PseudoType::kPseudoWhere | PseudoType::kPseudoParent
        ) {
            if let Some(l) = self.SelectorListOrParent() {
                if l.IsSingleComplexSelector() {
                    return l.First().unwrap().IsDeeplyHostPseudoClass();
                }
            }
        }
        self.IsHostPseudoClass()
    }
}
// A slice cursor is Rust's checked equivalent of the C++ CSSSelector pointer
// into a flat array. It does not allocate, clone selectors, or add links.
#[derive(Clone, Copy)]
pub struct CSSSelectorComplex<'a> {
    selectors_: &'a [CSSSelector],
}
impl<'a> Deref for CSSSelectorComplex<'a> {
    type Target = CSSSelector;
    fn deref(&self) -> &CSSSelector {
        self.First()
    }
}
impl<'a> CSSSelectorComplex<'a> {
    pub fn new(selectors: &'a [CSSSelector]) -> Self {
        assert!(!selectors.is_empty());
        Self {
            selectors_: selectors,
        }
    }
    pub(crate) fn ArrayTail(self) -> &'a [CSSSelector] {
        self.selectors_
    }
    // cpp: css_selector_list.h:195-218. These static-list operations also serve
    // StyleRule's identical flat storage without constructing a list object.
    pub fn NextComplexSelectorIncludingUnparsedInvalid(self) -> Option<Self> {
        let mut last = 0;
        while !self.selectors_[last].IsLastInComplexSelector() {
            last += 1;
        }
        if self.selectors_[last].IsLastInSelectorList() {
            None
        } else {
            Some(Self::new(&self.selectors_[last + 1..]))
        }
    }
    pub fn NextComplexSelector(self) -> Option<Self> {
        let mut next = self.NextComplexSelectorIncludingUnparsedInvalid();
        while let Some(current) = next {
            if !current.IsUnparsedInvalid() {
                return Some(current);
            }
            next = current.NextComplexSelectorIncludingUnparsedInvalid();
        }
        None
    }
    pub fn First(self) -> &'a CSSSelector {
        &self.selectors_[0]
    }
    // cpp: css_selector.h:512-517
    pub fn NextSimpleSelector(self) -> Option<Self> {
        if self.IsLastInComplexSelector() {
            None
        } else {
            Some(Self::new(&self.selectors_[1..]))
        }
    }
    pub fn SimpleSelectors(self) -> SimpleSelectors<'a> {
        SimpleSelectors {
            current: Some(self),
        }
    }
    // cpp: css_selector.cc:201-242,354-374
    pub fn Specificity(self) -> u32 {
        if self.IsForPage() {
            return self.SpecificityForPage() & kMaxValueMask;
        }
        AccumulateSpecificity(
            self.SimpleSelectors()
                .map(|s| s.SpecificityForOneSelector()),
        )
    }
    pub fn SpecificityTuple(self) -> [u8; 3] {
        SpecificityComponents(self.Specificity())
    }
    fn SpecificityForPage(self) -> u32 {
        self.SimpleSelectors()
            .map(|s| match s.Match() {
                MatchType::kTag => 4,
                MatchType::kPagePseudoClass => match s.GetPseudoType() {
                    PseudoType::kPseudoFirstPage => 2,
                    PseudoType::kPseudoLeftPage | PseudoType::kPseudoRightPage => 1,
                    _ => panic!("invalid page pseudo"),
                },
                _ => 0,
            })
            .sum()
    }
    // cpp: css_selector.cc:1722-1778
    pub fn IsFullyCompound(self) -> bool {
        let mut previous: Option<&CSSSelector> = None;
        for s in self.SimpleSelectors() {
            if previous.is_some_and(|p| p.Relation() != RelationType::kSubSelector) {
                return false;
            }
            if !s.IsSubSelectorCompound() {
                return false;
            }
            previous = Some(s);
        }
        true
    }
    // cpp: css_selector.cc:1780-1813
    pub fn HasLinkOrVisited(self) -> bool {
        self.HasLinkPseudo(true)
    }
    pub fn HasVisited(self) -> bool {
        self.HasLinkPseudo(false)
    }
    fn HasLinkPseudo(self, include_link: bool) -> bool {
        self.SimpleSelectors().any(|s| {
            s.GetPseudoType() == PseudoType::kPseudoVisited
                || (include_link && s.GetPseudoType() == PseudoType::kPseudoLink)
                || s.SelectorListOrParent()
                    .is_some_and(|l| l.ComplexSelectors().any(|s| s.HasLinkPseudo(include_link)))
        })
    }
    // cpp: css_selector.cc:1827-1848
    pub fn MatchesPseudoElement(self) -> bool {
        for s in self.SimpleSelectors() {
            if s.Match() == MatchType::kPseudoElement {
                return true;
            }
            if s.Relation() != RelationType::kSubSelector {
                return false;
            }
        }
        false
    }
    pub fn IsAllowedInParentPseudo(self) -> bool {
        !self.MatchesPseudoElement()
    }
    // cpp: css_selector.cc:2141-2157
    pub fn ForAnyInComplexSelector(self, functor: &impl Fn(&CSSSelector) -> bool) -> bool {
        self.SimpleSelectors().any(|s| {
            functor(s)
                || s.SelectorList().is_some_and(|l| {
                    l.ComplexSelectors()
                        .any(|s| s.ForAnyInComplexSelector(functor))
                })
        })
    }
    // cpp: css_selector.cc:2159-2171
    pub fn CrossesTreeScopes(self) -> bool {
        self.SimpleSelectors().any(|s| {
            matches!(
                s.Relation(),
                RelationType::kShadowPart | RelationType::kUAShadow | RelationType::kShadowSlot
            )
        })
    }
    pub fn SelectorText(self) -> String {
        self.SelectorTextInternal(false, 0)
    }
    pub fn SelectorTextExpandingPseudoReferences(self, scope_id: usize) -> String {
        self.SelectorTextInternal(true, scope_id)
    }
    // cpp: css_selector.cc:1588-1662
    pub fn SelectorTextInternal(self, expand: bool, scope_id: usize) -> String {
        let mut result: Vec<u16> = Vec::new();
        let mut compound = Some(self);
        while let Some(mut current) = compound {
            let mut builder = Vec::new();
            loop {
                current.SerializeSimpleSelector(&mut builder, expand, scope_id);
                if current.Relation() != RelationType::kSubSelector {
                    break;
                }
                match current.NextSimpleSelector() {
                    Some(next) => current = next,
                    None => {
                        builder.extend(result);
                        return String::from_utf16(&builder);
                    }
                }
            }
            let next = current.NextSimpleSelector();
            let mut relation = current.Relation();
            if next.is_none()
                || next.is_some_and(|n| {
                    n.Match() == MatchType::kPseudoClass
                        && n.GetPseudoType() == PseudoType::kPseudoScope
                        && n.IsImplicit()
                        && !expand
                })
            {
                relation = ConvertRelationToRelative(relation);
            }
            use RelationType::*;
            let (prefix, terminal) = match relation {
                kDescendant => (" ", false),
                kChild => (" > ", false),
                kDirectAdjacent => (" + ", false),
                kIndirectAdjacent => (" ~ ", false),
                kSubSelector | kPseudoChild | kShadowPart | kUAShadow | kShadowSlot => ("", false),
                kRelativeDescendant => ("", true),
                kRelativeChild => ("> ", true),
                kRelativeDirectAdjacent => ("+ ", true),
                kRelativeIndirectAdjacent => ("~ ", true),
            };
            let mut joined: Vec<u16> = prefix.encode_utf16().collect();
            joined.extend(builder);
            joined.extend(result);
            if terminal {
                return String::from_utf16(&joined);
            }
            result = joined;
            compound = next;
        }
        panic!("selector chain has no terminal compound")
    }
}
pub struct SimpleSelectors<'a> {
    current: Option<CSSSelectorComplex<'a>>,
}
impl<'a> Iterator for SimpleSelectors<'a> {
    type Item = &'a CSSSelector;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current?;
        self.current = current.NextSimpleSelector();
        Some(current.First())
    }
}
impl CSSSelector {
    fn IsSubSelectorCompound(&self) -> bool {
        match self.Match() {
            MatchType::kTag
            | MatchType::kUniversalTag
            | MatchType::kId
            | MatchType::kClass
            | MatchType::kAttributeExact
            | MatchType::kAttributeSet
            | MatchType::kAttributeList
            | MatchType::kAttributeHyphen
            | MatchType::kAttributeContain
            | MatchType::kAttributeBegin
            | MatchType::kAttributeEnd => true,
            MatchType::kPseudoElement | MatchType::kUnknown => false,
            MatchType::kPagePseudoClass | MatchType::kPseudoClass => self
                .SelectorList()
                .is_none_or(|l| l.ComplexSelectors().all(|s| s.IsFullyCompound())),
            MatchType::kInvalidList => panic!("invalid selector list"),
        }
    }
    // cpp: css_selector.cc:1350-1586
    pub fn SerializeSimpleSelector(&self, builder: &mut Vec<u16>, expand: bool, scope_id: usize) {
        use MatchType::*;
        use PseudoType::*;
        let mut suppress_selector_list = false;
        if matches!(self.Match(), kTag | kUniversalTag) && !self.IsImplicit() {
            SerializeNamespacePrefixIfNeeded(
                self.TagQName().Prefix(),
                &AtomicString::from_str("*"),
                builder,
                self.IsAttributeSelector(),
            );
            SerializeIdentifierOrAny(
                self.TagQName().LocalName(),
                UniversalSelectorAtom(),
                builder,
            );
        } else if self.Match() == kId || self.Match() == kClass {
            builder.push(if self.Match() == kId { b'#' } else { b'.' } as u16);
            serialize_atom(&self.SerializingValue(), builder);
        } else if self.Match() == kInvalidList && self.IsUnparsedInvalid() {
            append_atom(&self.Value(), builder);
        } else if self.Match() == kPseudoClass || self.Match() == kPagePseudoClass {
            if self.GetPseudoType() == kPseudoUnparsed {
                append_atom(&self.Value(), builder);
            } else if !matches!(self.GetPseudoType(), kPseudoParent | kPseudoScope) {
                builder.push(b':' as u16);
                append_atom(&self.SerializingValue(), builder);
            }
            match self.GetPseudoType() {
                kPseudoNthChild | kPseudoNthLastChild | kPseudoNthOfType | kPseudoNthLastOfType => {
                    builder.push(b'(' as u16);
                    let r = self.rare().borrow();
                    let a = r.bits_.NthAValue();
                    let b = r.bits_.NthBValue();
                    if a == 0 {
                        builder.extend(b.to_string().encode_utf16());
                    } else {
                        match a {
                            1 => builder.push(b'n' as u16),
                            -1 => builder.extend("-n".encode_utf16()),
                            _ => {
                                builder.extend(a.to_string().encode_utf16());
                                builder.push(b'n' as u16);
                            }
                        }
                        if b < 0 {
                            builder.extend(b.to_string().encode_utf16());
                        } else if b > 0 {
                            builder.push(b'+' as u16);
                            builder.extend(b.to_string().encode_utf16());
                        }
                    }
                    if let Some(list) = &r.selector_list_ {
                        builder.extend(" of ".encode_utf16());
                        list.SerializeTo(builder, expand, scope_id);
                        suppress_selector_list = true;
                    }
                    builder.push(b')' as u16);
                }
                kPseudoDir | kPseudoState => {
                    builder.push(b'(' as u16);
                    serialize_atom(&self.Argument(), builder);
                    builder.push(b')' as u16);
                }
                kPseudoLang => {
                    builder.push(b'(' as u16);
                    SerializeIdentifierList(
                        builder,
                        self.ArgumentList().expect("lang argument list").iter(),
                    );
                    builder.push(b')' as u16);
                }
                kPseudoParent => {
                    if expand {
                        builder.extend(":is".encode_utf16());
                        if let Some(parent) = self.SelectorListOrParent() {
                            if let Some(first) = parent.First() {
                                builder.push(b'(' as u16);
                                builder.extend(
                                    first
                                        .SelectorTextExpandingPseudoReferences(scope_id)
                                        .EncodeForSelector(),
                                );
                                builder.push(b')' as u16);
                            }
                        }
                    } else {
                        builder.push(b'&' as u16);
                    }
                }
                kPseudoScope => {
                    if expand {
                        builder.extend(":-internal-scope-".encode_utf16());
                        builder.extend(scope_id.to_string().encode_utf16());
                    } else {
                        builder.push(b':' as u16);
                        append_atom(&self.SerializingValue(), builder);
                    }
                }
                kPseudoRelativeAnchor => panic!("cannot serialize relative anchor"),
                kPseudoActiveViewTransitionType => {
                    let ids = self.IdentList();
                    assert!(!ids.is_empty());
                    builder.push(b'(' as u16);
                    SerializeIdentifierList(builder, ids.iter());
                    builder.push(b')' as u16);
                }
                kPseudoLinkTo => {
                    builder.push(b'(' as u16);
                    self.GetNavigationLocation()
                        .expect("navigation location")
                        .SerializeTo(builder);
                    builder.push(b')' as u16);
                }
                _ => {}
            }
        } else if self.Match() == kPseudoElement {
            builder.extend("::".encode_utf16());
            serialize_atom(&self.SerializingValue(), builder);
            match self.GetPseudoType() {
                kPseudoPart => {
                    let mut separator = b'(';
                    for part in self.IdentList().iter() {
                        builder.push(separator as u16);
                        separator = b' ';
                        serialize_atom(part, builder);
                    }
                    builder.push(b')' as u16);
                }
                kPseudoPicker | kPseudoHighlight => {
                    builder.push(b'(' as u16);
                    serialize_atom(&self.Argument(), builder);
                    builder.push(b')' as u16);
                }
                kPseudoOverscrollAreaParent | kPseudoScrollButton => {
                    builder.push(b'(' as u16);
                    append_atom(&self.Argument(), builder);
                    builder.push(b')' as u16);
                }
                kPseudoViewTransitionGroup
                | kPseudoViewTransitionGroupChildren
                | kPseudoViewTransitionImagePair
                | kPseudoViewTransitionNew
                | kPseudoViewTransitionOld => {
                    builder.push(b'(' as u16);
                    for (i, name) in self.IdentList().iter().enumerate() {
                        if i != 0 {
                            builder.push(b'.' as u16);
                        }
                        if name.IsNull() {
                            builder.push(b'*' as u16);
                        } else {
                            serialize_atom(name, builder);
                        }
                    }
                    builder.push(b')' as u16);
                }
                _ => {}
            }
        } else if self.IsAttributeSelector() {
            builder.push(b'[' as u16);
            let attribute = self.Attribute();
            SerializeNamespacePrefixIfNeeded(
                attribute.Prefix(),
                &AtomicString::from_str("*"),
                builder,
                true,
            );
            serialize_atom(attribute.LocalName(), builder);
            let op = match self.Match() {
                kAttributeExact => "=",
                kAttributeSet => "]",
                kAttributeList => "~=",
                kAttributeHyphen => "|=",
                kAttributeBegin => "^=",
                kAttributeEnd => "$=",
                kAttributeContain => "*=",
                _ => unreachable!(),
            };
            builder.extend(op.encode_utf16());
            if self.Match() != kAttributeSet {
                crate::css_markup::SerializeStringTo(
                    &String::from_utf16(self.SerializingValue().utf16_units().unwrap_or_default()),
                    builder,
                );
                match self.AttributeMatch() {
                    AttributeMatchType::kCaseInsensitive => builder.extend(" i".encode_utf16()),
                    AttributeMatchType::kCaseSensitiveAlways => builder.extend(" s".encode_utf16()),
                    _ => {}
                }
                builder.push(b']' as u16);
            }
        }
        if !suppress_selector_list {
            if let Some(list) = self.SelectorList() {
                builder.push(b'(' as u16);
                list.SerializeTo(builder, expand, scope_id);
                builder.push(b')' as u16);
            }
        }
    }
    // cpp: css_selector.cc:1664-1669
    pub fn SimpleSelectorTextForDebug(&self) -> String {
        let mut output = Vec::new();
        self.SerializeSimpleSelector(&mut output, false, 0);
        String::from_utf16(&output)
    }
}
fn append_atom(atom: &AtomicString, output: &mut Vec<u16>) {
    output.extend_from_slice(atom.utf16_units().unwrap_or_default());
}
fn serialize_atom(atom: &AtomicString, output: &mut Vec<u16>) {
    crate::css_markup::SerializeIdentifierTo(
        &String::from_utf16(atom.utf16_units().unwrap_or_default()),
        output,
        false,
    );
}
pub enum SelectorListRef<'a> {
    Parent(&'a CSSSelectorList),
    Owned(Rc<CSSSelectorList>),
}
impl Deref for SelectorListRef<'_> {
    type Target = CSSSelectorList;
    fn deref(&self) -> &CSSSelectorList {
        match self {
            Self::Parent(l) => l,
            Self::Owned(l) => l,
        }
    }
}
// StringBuilder output remains UTF-16, including unpaired surrogate units.
pub(crate) trait SelectorStringUnits {
    fn EncodeForSelector(&self) -> Vec<u16>;
}
impl SelectorStringUnits for String {
    fn EncodeForSelector(&self) -> Vec<u16> {
        if let Some(units) = self.Span16() {
            units.to_vec()
        } else {
            self.Span8()
                .unwrap_or_default()
                .iter()
                .map(|v| *v as u16)
                .collect()
        }
    }
}
impl CSSSelector {
    // cpp: css_selector.cc:832-982. Map search uses the same Latin-1 ordering.
    pub fn NameToPseudoType(
        name: &AtomicString,
        has_arguments: bool,
        runtime: &impl CSSSelectorRuntime,
    ) -> PseudoType {
        use PseudoType::*;
        let Some(units) = name.utf16_units() else {
            return kPseudoUnknown;
        };
        if units.iter().any(|v| *v > 255) {
            return kPseudoUnknown;
        }
        let bytes: Vec<u8> = units.iter().map(|v| *v as u8).collect();
        let map = if has_arguments {
            kPseudoTypeWithArgumentsMap
        } else {
            kPseudoTypeWithoutArgumentsMap
        };
        let Ok(index) = map.binary_search_by(|(key, _)| key.as_bytes().cmp(&bytes)) else {
            return kPseudoUnknown;
        };
        let pseudo = map[index].1;
        let enabled = match pseudo {
            kPseudoPlaying | kPseudoPaused | kPseudoSeeking | kPseudoBuffering | kPseudoStalled
            | kPseudoMuted | kPseudoVolumeLocked => runtime.CSSMediaElementPseudosEnabled(),
            kPseudoPermissionGranted => {
                runtime.GeolocationElementEnabled()
                    || runtime.UserMediaElementEnabled()
                    || runtime.InstallElementEnabled()
            }
            kPseudoTargetCurrent | kPseudoScrollMarker | kPseudoScrollMarkerGroup => {
                runtime.CSSPseudoScrollMarkersEnabled()
            }
            kPseudoTargetBefore | kPseudoTargetAfter => {
                runtime.CSSScrollMarkerTargetBeforeAfterEnabled()
            }
            kPseudoScrollButton => runtime.CSSPseudoScrollButtonsEnabled(),
            kPseudoColumn => runtime.CSSPseudoColumnEnabled(),
            kPseudoSearchText | kPseudoCurrent => runtime.SearchTextHighlightPseudoEnabled(),
            kPseudoToolFormActive | kPseudoToolSubmitActive => {
                !runtime.HasDocument() || runtime.WebMCPEnabled()
            }
            kPseudoUnbounded => runtime.UnboundedElementEnabled(),
            kPseudoHasSlotted => runtime.CSSPseudoHasSlottedEnabled(),
            kPseudoOverscrollAreaParent | kPseudoOverscrollBackdrop | kPseudoOverscrollOpen => {
                runtime.OverscrollGesturesEnabled()
            }
            kPseudoActiveOption | kPseudoFiltered => {
                runtime.CustomizableComboboxEnabled() || runtime.FilterableSelectEnabled()
            }
            kPseudoAnimatedImage => runtime.CSSImageAnimationEnabled(),
            kPseudoExpandIcon => runtime.MenuElementsEnabled(),
            kPseudoNavigationSource => runtime.NavigationSourcePseudoClassEnabled(),
            kPseudoSkeleton => runtime.DeclarativeSkeletonsEnabled(),
            _ => true,
        };
        if enabled {
            pseudo
        } else {
            kPseudoUnknown
        }
    }
    // cpp: css_selector.cc:1019-1029
    pub fn UpdatePseudoPage(
        &mut self,
        value: AtomicString,
        context: &impl CSSSelectorParserContext,
    ) {
        assert_eq!(self.Match(), MatchType::kPagePseudoClass);
        self.SetValue(value.clone(), false);
        let pseudo = context.ParsePseudoType(&value, false);
        self.SetPseudoType(
            if matches!(
                pseudo,
                PseudoType::kPseudoFirstPage
                    | PseudoType::kPseudoLeftPage
                    | PseudoType::kPseudoRightPage
            ) {
                pseudo
            } else {
                PseudoType::kPseudoUnknown
            },
        );
    }
    // cpp: css_selector.cc:1031-1255
    pub fn UpdatePseudoType(
        &mut self,
        value: AtomicString,
        context: &impl CSSSelectorParserContext,
        has_arguments: bool,
        ua_sheet_mode: bool,
    ) {
        use PseudoType::*;
        let value = value.ToAsciiLower();
        self.SetPseudoType(context.ParsePseudoType(&value, has_arguments));
        self.SetValue(value, false);
        if matches!(
            self.GetPseudoType(),
            kPseudoAfter | kPseudoBefore | kPseudoFirstLetter | kPseudoFirstLine
        ) && self.Match() == MatchType::kPseudoClass
        {
            self.SetMatch(MatchType::kPseudoElement);
        }
        let valid = match self.GetPseudoType() {
            kPseudoAfter
            | kPseudoBefore
            | kPseudoFirstLetter
            | kPseudoFirstLine
            | kPseudoExpandIcon
            | kPseudoPickerIcon
            | kPseudoInterestButton
            | kPseudoCheckMark
            | kPseudoBackdrop
            | kPseudoOverscrollBackdrop
            | kPseudoCue
            | kPseudoMarker
            | kPseudoPart
            | kPseudoPlaceholder
            | kPseudoFileSelectorButton
            | kPseudoResizer
            | kPseudoScrollbar
            | kPseudoScrollbarCorner
            | kPseudoScrollbarButton
            | kPseudoScrollbarThumb
            | kPseudoScrollbarTrack
            | kPseudoScrollbarTrackPiece
            | kPseudoScrollMarker
            | kPseudoScrollMarkerGroup
            | kPseudoScrollButton
            | kPseudoColumn
            | kPseudoPicker
            | kPseudoSelectListbox
            | kPseudoSelection
            | kPseudoWebKitCustomElement
            | kPseudoSlotted
            | kPseudoSearchText
            | kPseudoTargetText
            | kPseudoHighlight
            | kPseudoSpellingError
            | kPseudoGrammarError
            | kPseudoViewTransition
            | kPseudoViewTransitionGroup
            | kPseudoViewTransitionGroupChildren
            | kPseudoViewTransitionImagePair
            | kPseudoViewTransitionOld
            | kPseudoViewTransitionNew
            | kPseudoDetailsContent => self.Match() == MatchType::kPseudoElement,
            kPseudoPermissionIcon => self.Match() == MatchType::kPseudoElement,
            kPseudoOverscrollAreaParent | kPseudoBlinkInternalElement => {
                self.Match() == MatchType::kPseudoElement && ua_sheet_mode
            }
            kPseudoSkeleton => self.Match() == MatchType::kPseudoElement,
            kPseudoHasDatalist
            | kPseudoHasOpenMenuitem
            | kPseudoHostHasNonAutoAppearance
            | kPseudoIsHtml
            | kPseudoListBox
            | kPseudoMultiSelectFocus
            | kPseudoSelectContainsInput
            | kPseudoSpatialNavigationFocus
            | kPseudoVideoPersistent
            | kPseudoVideoPersistentAncestor => {
                ua_sheet_mode && self.Match() == MatchType::kPseudoClass
            }
            kPseudoActive
            | kPseudoActiveOption
            | kPseudoActiveViewTransition
            | kPseudoActiveViewTransitionType
            | kPseudoAnimatedImage
            | kPseudoAny
            | kPseudoAnyLink
            | kPseudoAutofill
            | kPseudoAutofillPreviewed
            | kPseudoAutofillSelected
            | kPseudoBuffering
            | kPseudoChecked
            | kPseudoCornerPresent
            | kPseudoCurrent
            | kPseudoDecrement
            | kPseudoDefault
            | kPseudoDefined
            | kPseudoDialogInTopLayer
            | kPseudoDir
            | kPseudoDisabled
            | kPseudoDoubleButton
            | kPseudoDrag
            | kPseudoEmpty
            | kPseudoEnabled
            | kPseudoEnd
            | kPseudoFiltered
            | kPseudoFirstChild
            | kPseudoFirstOfType
            | kPseudoFocus
            | kPseudoFocusVisible
            | kPseudoFocusWithin
            | kPseudoFullPageMedia
            | kPseudoFullScreen
            | kPseudoFullScreenAncestor
            | kPseudoFullscreen
            | kPseudoFutureCue
            | kPseudoHas
            | kPseudoHasSlotted
            | kPseudoHorizontal
            | kPseudoHost
            | kPseudoHostContext
            | kPseudoHover
            | kPseudoInRange
            | kPseudoIncrement
            | kPseudoIndeterminate
            | kPseudoInterestSource
            | kPseudoInterestTarget
            | kPseudoInvalid
            | kPseudoIs
            | kPseudoLang
            | kPseudoLastChild
            | kPseudoLastOfType
            | kPseudoLink
            | kPseudoLinkTo
            | kPseudoMenulistPopoverWithMenubarAnchor
            | kPseudoMenulistPopoverWithMenulistAnchor
            | kPseudoModal
            | kPseudoMuted
            | kPseudoNavigationSource
            | kPseudoNoButton
            | kPseudoNot
            | kPseudoNthChild
            | kPseudoNthLastChild
            | kPseudoNthLastOfType
            | kPseudoNthOfType
            | kPseudoOnlyChild
            | kPseudoOnlyOfType
            | kPseudoOpen
            | kPseudoOptional
            | kPseudoOutOfRange
            | kPseudoOverscrollOpen
            | kPseudoParent
            | kPseudoPastCue
            | kPseudoPaused
            | kPseudoPermissionGranted
            | kPseudoPictureInPicture
            | kPseudoPlaceholderShown
            | kPseudoPlaying
            | kPseudoPopoverInTopLayer
            | kPseudoPopoverOpen
            | kPseudoReadOnly
            | kPseudoReadWrite
            | kPseudoRelativeAnchor
            | kPseudoRequired
            | kPseudoRoot
            | kPseudoScope
            | kPseudoSeeking
            | kPseudoSelectHasSlottedButton
            | kPseudoSingleButton
            | kPseudoStalled
            | kPseudoStart
            | kPseudoState
            | kPseudoTarget
            | kPseudoTargetCurrent
            | kPseudoTargetBefore
            | kPseudoTargetAfter
            | kPseudoTextField
            | kPseudoUnknown
            | kPseudoUnbounded
            | kPseudoUnparsed
            | kPseudoUserInvalid
            | kPseudoUserValid
            | kPseudoValid
            | kPseudoVertical
            | kPseudoVisited
            | kPseudoVolumeLocked
            | kPseudoWebKitAutofill
            | kPseudoWebkitAnyLink
            | kPseudoWhere
            | kPseudoWindowInactive
            | kPseudoXrOverlay
            | kPseudoToolFormActive
            | kPseudoToolSubmitActive => self.Match() == MatchType::kPseudoClass,
            kPseudoFirstPage | kPseudoLeftPage | kPseudoRightPage => false,
        };
        if !valid {
            self.SetPseudoType(kPseudoUnknown);
        }
    }
}
#[cfg(test)]
mod full_selector_tests {
    use super::*;
    fn atom(s: &str) -> AtomicString {
        AtomicString::from_str(s)
    }
    fn simple(kind: MatchType, value: &str) -> CSSSelector {
        let mut s = CSSSelector::default();
        s.SetMatch(kind);
        s.SetValue(atom(value), false);
        s
    }
    fn pseudo(kind: PseudoType, name: &str) -> CSSSelector {
        let mut s = simple(MatchType::kPseudoClass, name);
        s.SetPseudoType(kind);
        s
    }
    fn list(mut selectors: Vec<CSSSelector>) -> Rc<CSSSelectorList> {
        selectors.last_mut().unwrap().SetLastInComplexSelector(true);
        CSSSelectorList::AdoptSelectorVector(selectors)
    }
    struct Runtime {
        enabled: bool,
        document: bool,
    }
    macro_rules! feature_methods { ($($method:ident),*) => { $(fn $method(&self) -> bool { self.enabled })* }; }
    impl CSSSelectorRuntime for Runtime {
        feature_methods!(
            CSSMediaElementPseudosEnabled,
            GeolocationElementEnabled,
            UserMediaElementEnabled,
            InstallElementEnabled,
            CSSPseudoScrollMarkersEnabled,
            CSSScrollMarkerTargetBeforeAfterEnabled,
            CSSPseudoScrollButtonsEnabled,
            CSSPseudoColumnEnabled,
            SearchTextHighlightPseudoEnabled,
            WebMCPEnabled,
            UnboundedElementEnabled,
            CSSPseudoHasSlottedEnabled,
            OverscrollGesturesEnabled,
            CustomizableComboboxEnabled,
            FilterableSelectEnabled,
            CSSImageAnimationEnabled,
            MenuElementsEnabled,
            NavigationSourcePseudoClassEnabled,
            DeclarativeSkeletonsEnabled
        );
        fn HasDocument(&self) -> bool {
            self.document
        }
    }
    impl CSSSelectorParserContext for Runtime {
        fn ParsePseudoType(&self, value: &AtomicString, has_arguments: bool) -> PseudoType {
            CSSSelector::NameToPseudoType(&value.ToAsciiLower(), has_arguments, self)
        }
    }
    impl CSSSelectorAttributeContext for Runtime {
        fn IsCaseSensitiveAttribute(&self, attribute: &QualifiedName) -> bool {
            attribute.LocalName() != &atom("type")
        }
    }
    struct Parent(Rc<CSSSelectorList>);
    impl CSSSelectorParentRule for Parent {
        fn Selectors(&self) -> &CSSSelectorList {
            &self.0
        }
    }
    #[test]
    fn flat_compound_order_specificity_and_lists_follow_blink() {
        // .c3#ident, span.c2, div.c1 are one complex selector in flat storage.
        let mut right_id = simple(MatchType::kId, "ident");
        right_id.SetRelation(RelationType::kDirectAdjacent);
        let mut c2 = simple(MatchType::kClass, "c2");
        c2.SetRelation(RelationType::kChild);
        let tag = |name| {
            CSSSelector::FromTag(
                QualifiedName::new(AtomicString::default(), atom(name), AtomicString::default()),
                false,
            )
        };
        let selectors = list(vec![
            simple(MatchType::kClass, "c3"),
            right_id,
            tag("span"),
            c2,
            tag("div"),
            simple(MatchType::kClass, "c1"),
        ]);
        assert_eq!(
            selectors.SelectorsText().as_str(),
            "div.c1 > span.c2 + .c3#ident"
        );
        assert_eq!(selectors.ComputeLength(), 6);
        assert!(selectors.IsSingleComplexSelector());
        assert_eq!(selectors.MaximumSpecificity(), 0x010302);
        assert_eq!(selectors.First().unwrap().SpecificityTuple(), [1, 3, 2]);
        assert!(!selectors.First().unwrap().IsFullyCompound());
        let mut first = simple(MatchType::kClass, "a");
        first.SetLastInComplexSelector(true);
        let multi = list(vec![first, simple(MatchType::kId, "b")]);
        assert_eq!(multi.SelectorsText().as_str(), ".a, #b");
        assert_eq!(multi.IndexOfNextSelectorAfter(0), Some(1));
        assert_eq!(multi.SelectorIndex(multi.SelectorAt(1)), 1);
        assert_eq!(multi.MaximumSpecificity(), kIdSpecificity);
        assert_eq!(multi.ComplexSelectors().count(), 2);
    }
    #[test]
    fn rare_union_transition_shared_copy_and_original_spelling() {
        let mut s = simple(MatchType::kClass, "UPPER");
        s.SetValue(atom("UPPER"), true);
        assert!(s.HasRareData());
        assert_eq!(*s.Value(), atom("upper"));
        assert_eq!(*s.SerializingValue(), atom("UPPER"));
        let mut copy = s.clone();
        copy.SetArgument(atom("changed"));
        assert_eq!(*s.Argument(), atom("changed"));
        assert_eq!(copy.SimpleSelectorTextForDebug().as_str(), ".UPPER");
        let nested = list(vec![simple(MatchType::kId, "high")]);
        let mut is = pseudo(PseudoType::kPseudoIs, "is");
        is.SetSelectorList(Some(nested.clone()));
        assert!(is.HasInlineSelectorList());
        assert!(!is.HasRareData());
        assert_eq!(is.SpecificityForOneSelector(), kIdSpecificity);
        is.SetContainsPseudoInsideHasPseudoClass();
        assert!(is.HasRareData());
        assert!(!is.HasInlineSelectorList());
        assert!(Rc::ptr_eq(&is.SelectorList().unwrap(), &nested));
        assert_eq!(is.SimpleSelectorTextForDebug().as_str(), ":is(#high)");
    }
    #[test]
    fn nth_lists_attribute_namespace_and_utf16_serialize_without_projection() {
        let ctx = Runtime {
            enabled: true,
            document: true,
        };
        let attribute = CSSSelector::FromAttribute(
            MatchType::kAttributeExact,
            QualifiedName::new(atom("*"), atom("type"), atom("*")),
            AttributeMatchType::kCaseInsensitive,
            Some(atom("a\"\nb")),
            &ctx,
        );
        assert_eq!(
            attribute.SimpleSelectorTextForDebug().as_str(),
            "[*|type=\"a\\\"\\a b\" i]"
        );
        assert!(attribute.LegacyCaseInsensitiveMatch());
        let exact = CSSSelector::FromAttribute(
            MatchType::kAttributeSet,
            QualifiedName::new(atom(""), atom("type"), atom("")),
            AttributeMatchType::kCaseSensitiveAlways,
            None,
            &ctx,
        );
        assert!(!exact.LegacyCaseInsensitiveMatch());
        assert_eq!(exact.SimpleSelectorTextForDebug().as_str(), "[type]");
        let mut nth = pseudo(PseudoType::kPseudoNthChild, "nth-child");
        nth.SetNth(-2, 5, Some(list(vec![simple(MatchType::kClass, "a")])));
        assert_eq!(
            nth.SimpleSelectorTextForDebug().as_str(),
            ":nth-child(-2n+5 of .a)"
        );
        assert!(nth.MatchNth(3));
        assert!(!nth.MatchNth(4));
        assert_eq!(nth.SpecificityForOneSelector(), 2 * kClassLikeSpecificity);
        let mut c = simple(MatchType::kClass, "");
        c.SetValue(AtomicString::from_utf16(&[0xd800]), false);
        assert_eq!(
            c.SimpleSelectorTextForDebug().Span16().unwrap(),
            &[b'.' as u16, 0xd800]
        );
    }
    #[test]
    fn invalid_unparsed_traversal_keeps_cssom_text_and_hides_matching() {
        let empty = CSSSelectorList::Empty();
        assert!(!empty.IsValid());
        assert!(empty.First().is_none());
        assert_eq!(empty.ComputeLength(), 0);
        assert!(Rc::ptr_eq(&empty.Copy(), &empty));
        let mut invalid = pseudo(PseudoType::kPseudoUnknown, "");
        invalid.SetUnparsedPlaceholder(CSSNestingType::kNone, atom(".broken >"));
        invalid.SetMatch(MatchType::kInvalidList);
        invalid.SetLastInComplexSelector(true);
        let mixed = list(vec![invalid, simple(MatchType::kClass, "valid")]);
        assert!(!mixed.IsValid());
        assert_eq!(mixed.First().unwrap().Value().Utf8(), "valid");
        assert_eq!(mixed.ComputeLength(), 2);
        assert_eq!(mixed.SelectorsText().as_str(), ".broken >, .valid");
        assert_eq!(mixed.Copy().SelectorsText(), mixed.SelectorsText());
        assert_eq!(mixed.MaximumSpecificity(), kClassLikeSpecificity);
    }
    #[test]
    fn parent_nesting_excludes_pseudo_specificity_and_renests_actual_lists() {
        let mut before = simple(MatchType::kPseudoElement, "before");
        before.SetPseudoType(PseudoType::kPseudoBefore);
        before.SetLastInComplexSelector(true);
        let parent: Rc<dyn CSSSelectorParentRule> = Rc::new(Parent(list(vec![
            before,
            simple(MatchType::kClass, "parent"),
        ])));
        let nested = list(vec![CSSSelector::FromParent(Some(parent.clone()), false)]);
        assert_eq!(nested.MaximumSpecificity(), kClassLikeSpecificity);
        assert!(nested.RenestChanged(Some(parent.clone())).is_none());
        let replacement: Rc<dyn CSSSelectorParentRule> =
            Rc::new(Parent(list(vec![simple(MatchType::kId, "replacement")])));
        let mut is = pseudo(PseudoType::kPseudoIs, "is");
        is.SetSelectorList(Some(nested.clone()));
        let old = list(vec![is]);
        let new = old.Renest(Some(replacement.clone()));
        assert!(!Rc::ptr_eq(&old, &new));
        assert_eq!(new.MaximumSpecificity(), kIdSpecificity);
        assert_eq!(old.MaximumSpecificity(), kClassLikeSpecificity);
        assert_eq!(old.SelectorsText().as_str(), ":is(&)");
        assert_eq!(
            new.First()
                .unwrap()
                .SelectorTextExpandingPseudoReferences(7)
                .as_str(),
            ":is(:is(#replacement))"
        );
        let renested_parent = new.First().unwrap().SelectorList().unwrap();
        assert!(Rc::ptr_eq(
            renested_parent.First().unwrap().ParentRule().unwrap(),
            &replacement
        ));
    }
    #[test]
    fn runtime_gates_single_colon_and_ua_only_classes_preserve_context() {
        let off = Runtime {
            enabled: false,
            document: true,
        };
        let on = Runtime {
            enabled: true,
            document: true,
        };
        assert_eq!(
            CSSSelector::NameToPseudoType(&atom("playing"), false, &off),
            PseudoType::kPseudoUnknown
        );
        assert_eq!(
            CSSSelector::NameToPseudoType(&atom("playing"), false, &on),
            PseudoType::kPseudoPlaying
        );
        assert_eq!(
            CSSSelector::NameToPseudoType(
                &atom("tool-form-active"),
                false,
                &Runtime {
                    enabled: false,
                    document: false
                }
            ),
            PseudoType::kPseudoToolFormActive
        );
        assert_eq!(
            CSSSelector::NameToPseudoType(&AtomicString::from_utf16(&[0x100]), false, &on),
            PseudoType::kPseudoUnknown
        );
        let mut s = simple(MatchType::kPseudoClass, "");
        s.UpdatePseudoType(atom("BeFoRe"), &on, false, false);
        assert_eq!(s.Match(), MatchType::kPseudoElement);
        assert_eq!(s.GetPseudoType(), PseudoType::kPseudoBefore);
        assert_eq!(s.SimpleSelectorTextForDebug().as_str(), "::before");
        let mut ua = simple(MatchType::kPseudoClass, "");
        ua.UpdatePseudoType(atom("-internal-has-datalist"), &on, false, false);
        assert_eq!(ua.GetPseudoType(), PseudoType::kPseudoUnknown);
        ua.UpdatePseudoType(atom("-internal-has-datalist"), &on, false, true);
        assert_eq!(ua.GetPseudoType(), PseudoType::kPseudoHasDatalist);
    }
    #[test]
    fn implicit_scope_relative_combinators_and_recursive_link_queries() {
        let runtime = Runtime {
            enabled: true,
            document: true,
        };
        let mut right = simple(MatchType::kClass, "a");
        right.SetRelation(RelationType::kChild);
        let scope = CSSSelector::FromPseudo(atom("scope"), true, &runtime);
        let relative = list(vec![right, scope]);
        assert_eq!(relative.SelectorsText().as_str(), "> .a");
        assert_eq!(relative.MaximumSpecificity(), kClassLikeSpecificity);
        assert_eq!(
            relative
                .First()
                .unwrap()
                .SelectorTextExpandingPseudoReferences(42)
                .as_str(),
            ":-internal-scope-42 > .a"
        );
        let visited = list(vec![pseudo(PseudoType::kPseudoVisited, "visited")]);
        let mut is = pseudo(PseudoType::kPseudoIs, "is");
        is.SetSelectorList(Some(visited));
        let deep = list(vec![is]);
        assert!(deep.First().unwrap().HasVisited());
        assert!(deep.First().unwrap().HasLinkOrVisited());
        assert!(deep.First().unwrap().IsFullyCompound());
    }
}
// cpp: css_selector.cc:75-94; css_selector.h:1210
pub fn MaximumSpecificity(first: Option<CSSSelectorComplex<'_>>) -> u32 {
    MaximumSpecificityWhere(first, |_| true)
}
pub fn MaximumSpecificityWhere(
    first: Option<CSSSelectorComplex<'_>>,
    predicate: impl Fn(CSSSelectorComplex<'_>) -> bool,
) -> u32 {
    let mut result = 0;
    let mut current = first;
    while let Some(selector) = current {
        if predicate(selector) {
            result = result.max(selector.Specificity());
        }
        current = selector.NextComplexSelector();
    }
    result
}
// cpp: css_selector.h:1191-1198; native Rust swap moves the active union too.
pub fn swap(a: &mut CSSSelector, b: &mut CSSSelector) {
    std::mem::swap(a, b);
}
