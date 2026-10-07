//! Translated from Blink platform/graphics/paint/display_item.h: enum Type.
//! This is display-item identity, distinct from low-level drawing opcodes.
#![allow(non_upper_case_globals, non_snake_case)]

use crate::paint_engine::PaintPhase;

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct DisplayItemIdType(pub u8);

impl DisplayItemIdType {
    pub const kUninitializedType: Self = Self(0);
    pub const kDrawingFirst: Self = Self(1);
    pub const kDrawingPaintPhaseFirst: Self = Self(1);
    pub const kDrawingPaintPhaseLast: Self = Self(13);
    pub const kBoxDecorationBackground: Self = Self(14);
    pub const kFixedAttachmentBackground: Self = Self(15);
    pub const kCapsLockIndicator: Self = Self(16);
    pub const kCaret: Self = Self(17);
    pub const kColumnRules: Self = Self(18);
    pub const kCustomHighlightTint: Self = Self(19);
    pub const kDocumentRootBackdrop: Self = Self(20);
    pub const kDocumentBackground: Self = Self(21);
    pub const kDragCaret: Self = Self(22);
    pub const kForcedColorsModeBackplate: Self = Self(23);
    pub const kSVGImage: Self = Self(24);
    pub const kImageAreaFocusRing: Self = Self(25);
    pub const kOverflowControls: Self = Self(26);
    pub const kFrameOverlay: Self = Self(27);
    pub const kPrintedContentDestinationLocations: Self = Self(28);
    pub const kPrintedContentPDFURLRect: Self = Self(29);
    pub const kReflectionMask: Self = Self(30);
    pub const kResizer: Self = Self(31);
    pub const kSVGClip: Self = Self(32);
    pub const kSVGMask: Self = Self(33);
    pub const kScrollCorner: Self = Self(34);
    pub const kScrollbarTrackAndButtons: Self = Self(35);
    pub const kScrollbarThumb: Self = Self(36);
    pub const kScrollbarTickmarks: Self = Self(37);
    pub const kSelectionTint: Self = Self(38);
    pub const kTableCollapsedBorders: Self = Self(39);
    pub const kWebPlugin: Self = Self(40);
    pub const kDrawingLast: Self = Self(40);
    pub const kForeignLayerFirst: Self = Self(41);
    pub const kForeignLayerCanvas: Self = Self(41);
    pub const kForeignLayerDevToolsOverlay: Self = Self(42);
    pub const kForeignLayerPlugin: Self = Self(43);
    pub const kForeignLayerVideo: Self = Self(44);
    pub const kForeignLayerRemoteFrame: Self = Self(45);
    pub const kForeignLayerLinkHighlight: Self = Self(46);
    pub const kForeignLayerViewportScroll: Self = Self(47);
    pub const kForeignLayerViewportScrollbar: Self = Self(48);
    pub const kForeignLayerViewTransitionContent: Self = Self(49);
    pub const kForeignLayerLast: Self = Self(49);
    pub const kClipPaintPhaseFirst: Self = Self(50);
    pub const kClipPaintPhaseLast: Self = Self(62);
    pub const kScrollPaintPhaseFirst: Self = Self(63);
    pub const kScrollPaintPhaseLast: Self = Self(75);
    pub const kSVGTransformPaintPhaseFirst: Self = Self(76);
    pub const kSVGTransformPaintPhaseLast: Self = Self(88);
    pub const kSVGEffectPaintPhaseFirst: Self = Self(89);
    pub const kSVGEffectPaintPhaseLast: Self = Self(101);
    pub const kHitTest: Self = Self(102);
    pub const kWebPluginHitTest: Self = Self(103);
    pub const kRegionCapture: Self = Self(104);
    pub const kTrackedElement: Self = Self(105);
    pub const kScrollHitTest: Self = Self(106);
    pub const kResizerScrollHitTest: Self = Self(107);
    pub const kScrollbarHitTest: Self = Self(108);
    pub const kLayerChunk: Self = Self(109);
    pub const kLayerChunkForeground: Self = Self(110);
    pub const kScrollbarHorizontal: Self = Self(111);
    pub const kScrollbarVertical: Self = Self(112);
    pub const kTypeLast: Self = Self(112);

    pub fn PaintPhaseToClipType(phase: PaintPhase) -> Self {
        Self(Self::kClipPaintPhaseFirst.0 + phase as u8)
    }

    pub fn PaintPhaseToDrawingType(phase: PaintPhase) -> Self {
        Self(Self::kDrawingPaintPhaseFirst.0 + phase as u8)
    }
}

impl std::fmt::Debug for DisplayItemIdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            0 => f.write_str("kUninitializedType"),
            14 => f.write_str("kBoxDecorationBackground"),
            15 => f.write_str("kFixedAttachmentBackground"),
            16 => f.write_str("kCapsLockIndicator"),
            17 => f.write_str("kCaret"),
            18 => f.write_str("kColumnRules"),
            19 => f.write_str("kCustomHighlightTint"),
            20 => f.write_str("kDocumentRootBackdrop"),
            21 => f.write_str("kDocumentBackground"),
            22 => f.write_str("kDragCaret"),
            23 => f.write_str("kForcedColorsModeBackplate"),
            24 => f.write_str("kSVGImage"),
            25 => f.write_str("kImageAreaFocusRing"),
            26 => f.write_str("kOverflowControls"),
            27 => f.write_str("kFrameOverlay"),
            28 => f.write_str("kPrintedContentDestinationLocations"),
            29 => f.write_str("kPrintedContentPDFURLRect"),
            30 => f.write_str("kReflectionMask"),
            31 => f.write_str("kResizer"),
            32 => f.write_str("kSVGClip"),
            33 => f.write_str("kSVGMask"),
            34 => f.write_str("kScrollCorner"),
            35 => f.write_str("kScrollbarTrackAndButtons"),
            36 => f.write_str("kScrollbarThumb"),
            37 => f.write_str("kScrollbarTickmarks"),
            38 => f.write_str("kSelectionTint"),
            39 => f.write_str("kTableCollapsedBorders"),
            40 => f.write_str("kWebPlugin"),
            41 => f.write_str("kForeignLayerCanvas"),
            42 => f.write_str("kForeignLayerDevToolsOverlay"),
            43 => f.write_str("kForeignLayerPlugin"),
            44 => f.write_str("kForeignLayerVideo"),
            45 => f.write_str("kForeignLayerRemoteFrame"),
            46 => f.write_str("kForeignLayerLinkHighlight"),
            47 => f.write_str("kForeignLayerViewportScroll"),
            48 => f.write_str("kForeignLayerViewportScrollbar"),
            49 => f.write_str("kForeignLayerViewTransitionContent"),
            102 => f.write_str("kHitTest"),
            103 => f.write_str("kWebPluginHitTest"),
            104 => f.write_str("kRegionCapture"),
            105 => f.write_str("kTrackedElement"),
            106 => f.write_str("kScrollHitTest"),
            107 => f.write_str("kResizerScrollHitTest"),
            108 => f.write_str("kScrollbarHitTest"),
            109 => f.write_str("kLayerChunk"),
            110 => f.write_str("kLayerChunkForeground"),
            111 => f.write_str("kScrollbarHorizontal"),
            112 => f.write_str("kScrollbarVertical"),
            1..=13 => write!(f, "DrawingPaintPhase({})", self.0 - 1),
            value => write!(f, "DisplayItemIdType({value})"),
        }
    }
}

/// Blink DisplayItem::Id: client identity, semantic type, fragment scope.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DisplayItemId {
    pub client_id: u64,
    pub r#type: DisplayItemIdType,
    // Blink wtf_size_t is uint32_t, including on 64-bit platforms.
    pub fragment: u32,
}
