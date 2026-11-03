//! Translated from Blink platform/graphics/paint/display_item.h: enum Type.
//! This is display-item identity, distinct from low-level drawing opcodes.
#![allow(non_upper_case_globals, non_snake_case)]

use crate::PaintPhase;

// Source: display_item.cc:70-102.
fn PaintPhaseAsDebugString(paint_phase: u8) -> &'static str {
    match paint_phase {
        0 => "PaintPhaseBlockBackground",
        1 => "PaintPhaseSelfBlockBackgroundOnly",
        2 => "PaintPhaseDescendantBlockBackgroundsOnly",
        3 => "PaintPhaseForcedColorsModeBackplate",
        4 => "PaintPhaseFloat",
        5 => "PaintPhaseForeground",
        6 => "PaintPhaseOutline",
        7 => "PaintPhaseSelfOutlineOnly",
        8 => "PaintPhaseDescendantOutlinesOnly",
        9 => "PaintPhaseOverlayOverflowControls",
        10 => "PaintPhaseSelection",
        11 => "PaintPhaseTextClip",
        12 => "PaintPhaseMask",
        _ => "PaintPhaseInvalid",
    }
}

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

    pub fn PaintPhaseToScrollType(phase: PaintPhase) -> Self {
        Self(Self::kScrollPaintPhaseFirst.0 + phase as u8)
    }

    pub fn PaintPhaseToSVGTransformType(phase: PaintPhase) -> Self {
        Self(Self::kSVGTransformPaintPhaseFirst.0 + phase as u8)
    }

    pub fn PaintPhaseToSVGEffectType(phase: PaintPhase) -> Self {
        Self(Self::kSVGEffectPaintPhaseFirst.0 + phase as u8)
    }

    pub fn IsDrawingType(self) -> bool {
        self.0 >= Self::kDrawingFirst.0 && self.0 <= Self::kDrawingLast.0
    }

    pub fn IsForeignLayerType(self) -> bool {
        self.0 >= Self::kForeignLayerFirst.0 && self.0 <= Self::kForeignLayerLast.0
    }

    // Source: display_item.cc:104-201.
    pub fn TypeAsDebugString(self) -> String {
        let phase_category = [
            (Self::kDrawingPaintPhaseFirst.0, "Drawing"),
            (Self::kClipPaintPhaseFirst.0, "Clip"),
            (Self::kScrollPaintPhaseFirst.0, "Scroll"),
            (Self::kSVGTransformPaintPhaseFirst.0, "SVGTransform"),
            (Self::kSVGEffectPaintPhaseFirst.0, "SVGEffect"),
        ]
        .into_iter()
        .find(|(first, _)| self.0 >= *first && self.0 <= *first + PaintPhase::kMax as u8);
        if let Some((first, category)) = phase_category {
            return format!("{category}{}", PaintPhaseAsDebugString(self.0 - first));
        }

        let name = match self.0 {
            0 => "UninitializedType",
            14 => "BoxDecorationBackground",
            15 => "FixedAttachmentBackground",
            16 => "CapsLockIndicator",
            17 => "Caret",
            18 => "ColumnRules",
            19 => "CustomHighlightTint",
            20 => "DocumentRootBackdrop",
            21 => "DocumentBackground",
            22 => "DragCaret",
            23 => "ForcedColorsModeBackplate",
            24 => "SVGImage",
            25 => "ImageAreaFocusRing",
            26 => "OverflowControls",
            27 => "FrameOverlay",
            28 => "PrintedContentDestinationLocations",
            29 => "PrintedContentPDFURLRect",
            30 => "ReflectionMask",
            31 => "Resizer",
            32 => "SVGClip",
            33 => "SVGMask",
            34 => "ScrollCorner",
            35 => "ScrollbarTrackAndButtons",
            36 => "ScrollbarThumb",
            37 => "ScrollbarTickmarks",
            38 => "SelectionTint",
            39 => "TableCollapsedBorders",
            40 => "WebPlugin",
            41 => "ForeignLayerCanvas",
            42 => "ForeignLayerDevToolsOverlay",
            43 => "ForeignLayerPlugin",
            44 => "ForeignLayerVideo",
            45 => "ForeignLayerRemoteFrame",
            46 => "ForeignLayerLinkHighlight",
            47 => "ForeignLayerViewportScroll",
            48 => "ForeignLayerViewportScrollbar",
            49 => "ForeignLayerViewTransitionContent",
            102 => "HitTest",
            103 => "WebPluginHitTest",
            104 => "RegionCapture",
            105 => "TrackedElement",
            106 => "ScrollHitTest",
            107 => "ResizerScrollHitTest",
            108 => "ScrollbarHitTest",
            109 => "LayerChunk",
            110 => "LayerChunkForeground",
            111 => "ScrollbarHorizontal",
            112 => "ScrollbarVertical",
            value => return format!("DisplayItemType({value})"),
        };
        name.into()
    }
}

impl std::fmt::Debug for DisplayItemIdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.TypeAsDebugString())
    }
}

// Source: display_item.cc:259-265.
impl std::fmt::Display for DisplayItemIdType {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output.write_str(&self.TypeAsDebugString())
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

impl DisplayItemId {
    // Source: display_item.cc:245-253.
    pub fn ToString(self) -> String {
        format!("{:#x}:{}:{}", self.client_id, self.r#type, self.fragment)
    }
}

// Source: display_item.cc:267-269.
impl std::fmt::Display for DisplayItemId {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        output.write_str(&self.ToString())
    }
}

#[cfg(test)]
mod display_item_debug_tests {
    use super::*;

    #[test]
    fn phase_and_special_type_names_match_blink_debug_strings() {
        assert_eq!(
            DisplayItemIdType::PaintPhaseToDrawingType(PaintPhase::kForeground).TypeAsDebugString(),
            "DrawingPaintPhaseForeground"
        );
        assert_eq!(
            DisplayItemIdType::PaintPhaseToSVGEffectType(PaintPhase::kMask).TypeAsDebugString(),
            "SVGEffectPaintPhaseMask"
        );
        assert_eq!(
            DisplayItemIdType::kForeignLayerVideo.TypeAsDebugString(),
            "ForeignLayerVideo"
        );
        assert_eq!(DisplayItemIdType::kLayerChunk.to_string(), "LayerChunk");
    }

    #[test]
    fn id_string_uses_source_type_name_and_fragment() {
        let id = DisplayItemId {
            client_id: 0x2a,
            r#type: DisplayItemIdType::kCaret,
            fragment: 7,
        };
        assert_eq!(id.ToString(), "0x2a:Caret:7");
        assert_eq!(id.to_string(), id.ToString());
    }
}
