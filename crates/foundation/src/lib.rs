#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// The source checkout already owns these translated foundation primitives.
// This crate assembles the remaining interfaces required by LayoutNG while
// keeping the source checkout read-only.
pub use foundation_base::blink_geometry::geometry::color_channel_keyword::ColorChannelKeyword;
pub use foundation_base::blink_geometry::geometry::path_types::{LineCap, LineJoin, WindRule};
pub use foundation_base::blink_geometry::geometry::physical_size::AspectRatioFit;
pub use foundation_base::blink_geometry::geometry::{
    layout_unit::{kIndefiniteSize, FloorToInt},
    LayoutRatioFromSizeF, LayoutUnit, LogicalDirection, MarginStrut, PhysicalDirection,
    PhysicalOffset, PhysicalSize,
};
pub use foundation_base::gfx_geometry::Point;
pub use foundation_base::layout_features::runtime_enabled_features::RuntimeEnabledFeatures;
pub use foundation_base::text::native::east_asian_spacing_type::EastAsianSpacingType;
pub use foundation_base::text::native::han_kerning_char_type::HanKerningCharType;
pub use foundation_base::text::native::text_justify::TextJustify;
pub use foundation_base::text::native::unicode_bidi::{IsIsolated, IsOverride, UnicodeBidi};
pub use foundation_base::text::native::{
    text_direction::{DirectionFromLevel, IsLtr, IsRtl},
    writing_mode::{
        IsFlippedBlocksWritingMode, IsFlippedLinesWritingMode, IsHorizontalTypographicMode,
        IsHorizontalWritingMode, IsParallelWritingMode, IsVerticalWritingMode, ToLineWritingMode,
    },
    writing_mode_utils::{LogicalToLogical, LogicalToPhysical, PhysicalToLogical},
    TabSize, TabSizeValueType, TextDirection, WritingDirectionMode, WritingMode,
};
pub use foundation_base::unsupported_layout::UnsupportedLayout;
pub use gfx_ext::RectF;
pub use path_forward::Path;
pub use physical_conversions::{ToFlooredPoint, ToFlooredSize, ToRoundedSize};
pub use physical_rect::PhysicalRect;
pub use physical_rect::{SnapSizeToPixel, ToPixelSnappedRect};
pub const RULE_NONZERO: WindRule = WindRule::RULE_NONZERO;

pub mod gfx {
    pub use crate::gfx_ext::{OutsetsF, Rect, RectF, ToRoundedVector2d, TransposeSize};
    pub use crate::gfx_quad_f::QuadF;
    pub use crate::gfx_transform::Transform;
    pub use foundation_base::gfx_geometry::*;
}

pub mod atomic_string;
pub mod begin_frame;
pub mod blink_geometry;
pub mod blink_string;
pub mod calculation_expression_node;
pub mod calculation_operator;
pub mod calculation_value;
pub mod casting;
pub mod clear_collection_scope;
pub mod color;
pub mod compositor_element_id;
pub mod cpp_ostream;
pub mod css_bitset;
pub mod css_property_id;
pub mod css_value_id;
pub mod disallow_new_wrapper;
pub mod display_adjustment_context;
pub mod doubly_linked_list;
pub mod dynamic_range_limit;
pub mod evaluation_input;
pub mod gc_heap;
pub mod gc_member;
mod gfx_ext;
pub mod gfx_quad_f;
mod gfx_transform;
pub mod graphics_enums;
pub mod graphics_types;
pub mod hanging_punctuation;
pub mod hash_containers;
pub mod hash_functions;
pub mod heap_hash_containers;
pub mod heap_vector;
pub mod image_animation_enum;
pub mod infinite_int_rect;
pub mod keywords;
pub mod length;
pub mod length_box;
pub mod length_functions;
pub mod length_point;
pub mod length_size;
pub mod math_functions;
pub mod math_transform;
pub mod numeric_conversion;
pub mod overlay_scrollbar_clip_behavior;
pub mod paint_invalidation_reason;
pub mod paint_property_update_reason;
mod path_forward;
pub mod physical_conversions;
mod physical_rect;
pub mod quotes_data;
pub mod rapidhash;
pub mod scoped_css_name;
pub mod scoped_refptr;
pub mod string_statics;
pub mod string_view;
pub mod text_position;
pub mod transform_operation_type;
pub mod transform_operations;
pub mod unique_object_id;
pub use atomic_string::{g_empty_atom, g_null_atom, AtomicString, AtomicStringHashTraits};
pub use blink_geometry::geometry::transform_state::{TransformAccumulation, TransformState};
pub use blink_string::BlinkString;
pub use color::{Color, ColorSpace, HueInterpolationMethod, RGBA32};
pub use compositor_element_id::{
    CompositorElementId, CompositorElementIdFromUniqueObjectId, CompositorElementIdNamespace,
};
pub use css_bitset::{CSSBitset, CSSBitsetBase, CSSBitsetIterator};
pub use gc_heap::{
    BasicPersistent, CollectLayoutHeapForTesting, FreeLayoutBacking,
    IsLayoutHeapSweepingOnOwningThread, IsManagedLayoutAddress,
    LayoutHeapAllocationCountForTesting, LayoutHeapScope, LayoutObjectSize, LivenessBroker,
    MakeGarbageCollected, MakeGarbageCollectedWithAdditionalBytes, Persistent, TraceIfNeeded,
    Traceable, Visitor, WeakPersistent,
};
pub use gc_member::{kMemberDeletedValue, Member, ReferenceKind, UntracedMember, WeakMember};
pub use graphics_enums::InterpolationQuality::kInterpolationNone;
pub use graphics_enums::{
    BlendMode, CompositeOperator, InterpolationQuality, RespectImageOrientationEnum,
};
pub use graphics_types::graphics::compositing_reasons::{CompositingReason, CompositingReasons};
pub use hash_containers::{HashMap, HashSet, IntWithZeroKeyHashTraits};
pub use heap_hash_containers::{
    BasicHeapHashMap, BasicHeapHashSet, GCedHeapHashMap, GCedHeapHashSet, HeapHashMap, HeapHashSet,
    WeakHeapHashMap,
};
pub use heap_vector::{BasicHeapVector, GCedHeapVector, HeapVector};
pub use image_animation_enum::ImageAnimationEnum;
pub use infinite_int_rect::InfiniteIntRect;
pub use length::{Length, LengthType, LengthValueRange, PixelsAndPercent};
pub use length_box::LengthBox;
pub use length_functions::{
    FloatValueForLength, FloatValueForLengthWithInput, MinimumValueForLength,
    MinimumValueForLengthWithInput, PointForLengthPoint, SizeForLengthSize, ValueForLength,
    ValueForLengthWithInput,
};
pub use length_point::LengthPoint;
pub use length_size::LengthSize;
pub use math_transform::ItalicMathVariant;
pub use overlay_scrollbar_clip_behavior::OverlayScrollbarClipBehavior;
pub use paint_invalidation_reason::{
    IsFullPaintInvalidationReason, IsLayoutFullPaintInvalidationReason,
    IsLayoutPaintInvalidationReason, IsNonLayoutFullPaintInvalidationReason,
    PaintInvalidationReason,
};
pub use paint_property_update_reason::SubtreePaintPropertyUpdateReason;
pub use string_statics::InitStringStatics;
pub use string_view::StringView;
pub use text_position::OrdinalNumber;
pub use transform_operation_type::TransformOperationType;
pub use transform_operations::{
    EmptyTransformOperations, Matrix3DTransformOperation, RotateTransformOperation,
    ScaleTransformOperation, TransformOperation, TransformOperations, TranslateTransformOperation,
};
pub use unique_object_id::{NewUniqueObjectId, UniqueObjectId};
pub mod style_aspect_ratio;
pub mod style_constants;
pub mod style_flags;
pub mod style_initial_letter;
pub mod style_name_scope;
pub mod style_values;
pub mod text_decoration_thickness;
pub mod text_offset_map;
pub mod threading_traits;
pub mod touch_action;
pub mod unicode;
pub mod unicode_data;
pub mod values_equivalent;
pub mod vector_ext;
pub mod wtf_size_t;
pub mod wtf_uchar;
pub use calculation_expression_node::{CalculationExpressionNode, CalculationSizingKeyword};
pub use calculation_operator::CalculationOperator;
pub use calculation_value::CalculationValue;
pub use casting::{CastSource, DowncastFrom, DynamicTo, IsA, To};
pub use clear_collection_scope::ClearCollectionScope;
pub use cpp_ostream::CppOstream;
pub use css_property_id::*;
pub use css_value_id::*;
pub use disallow_new_wrapper::DisallowNewWrapper;
pub use display_adjustment_context::DisplayAdjustmentContext;
pub use dynamic_range_limit::{DynamicRangeLimit, DynamicRangeLimitKind, DynamicRangeLimitMixture};
pub use evaluation_input::{CalcSizeKeywordBehavior, EvaluationInput};
pub use hanging_punctuation::HangingPunctuation;
pub use hash_functions::{
    AddFloatToHash, AddIntToHash, FloatEqualForHash, GetHash, HashFloat, HashInt, HashInt64,
    HashInts, HashPointer, NormalizeSign,
};
pub use quotes_data::QuotesData;
pub use scoped_css_name::{
    ScopedCSSName, ScopedCSSNameList, ScopedCSSNameWrapperPtrHashTraits, TreeScope,
};
pub use scoped_refptr::ScopedRefPtr;
pub use style_aspect_ratio::{EAspectRatioType, StyleAspectRatio};
pub use style_constants::*;
pub use style_flags::*;
pub use style_initial_letter::StyleInitialLetter;
pub use style_name_scope::{StyleNameScope, StyleNameScopeType};
pub use text_decoration_thickness::TextDecorationThickness;
pub use text_offset_map::{TextOffsetMap, TextOffsetMapEntry};
pub use threading_traits::{ThreadAffinity, ThreadingTrait};
pub use touch_action::{kTouchActionBits, TouchAction, TouchActionToString};
pub use values_equivalent::{ValuePointer, ValuesEquivalent};
pub use vector_ext::VectorExt;
pub use wtf_size_t::{kNotFound, CheckedDistance, WtfSizeT};
pub use wtf_uchar::{UChar, UChar32};

pub use blink_string::BlinkString as String;
pub type Vector<T> = Vec<T>;

pub fn StrCat(parts: &[String]) -> String {
    let mut result = String::from("");
    for part in parts {
        result.push_string(part);
    }
    result
}

pub fn Format(pattern: &str, parts: &[String]) -> String {
    assert_eq!(pattern.matches("{}").count(), parts.len());
    let mut result = String::new();
    let mut pieces = pattern.split("{}");
    result.push_str(pieces.next().unwrap_or_default());
    for (piece, value) in pieces.zip(parts) {
        result.push_string(value);
        result.push_str(piece);
    }
    result
}

pub struct StringBuilder {
    units: Vec<UChar>,
    has_value: bool,
    is_8bit: bool,
}

impl Default for StringBuilder {
    fn default() -> Self {
        Self {
            units: Vec::new(),
            has_value: false,
            is_8bit: true,
        }
    }
}

// cpp: foundation/layout_features/runtime_enabled_features.h:19-21,40
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "Rust" fn CSSLineClampLineBreakingEllipsisEnabled() -> bool {
    false
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "Rust" fn CollapseZeroWidthSpaceWhenReuseItemEnabled() -> bool {
    true
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "Rust" fn OffsetMappingReuseFullWidthSpaceFixEnabled() -> bool {
    true
}

impl StringBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn Append(&mut self, value: impl std::fmt::Display) {
        for unit in value.to_string().encode_utf16() {
            self.AppendCodeUnit(unit);
        }
        self.has_value = true;
    }

    // cpp: foundation/blink_base/wtf/text/string_builder.h:107-144
    pub fn AppendString(&mut self, value: &String) {
        let units = value.Span16().unwrap_or_default();
        if units.is_empty() {
            return;
        }
        if self.units.is_empty() {
            self.is_8bit = value.Is8Bit();
        } else if !value.Is8Bit() && units.len() > 1 {
            self.is_8bit = false;
        }
        if units.len() == 1 && !self.units.is_empty() {
            self.AppendCodeUnit(units[0]);
            return;
        }
        self.units.extend_from_slice(units);
        self.has_value = true;
    }

    // cpp: foundation/blink_base/wtf/text/string_builder.h:147-159
    pub fn AppendCodeUnit(&mut self, value: UChar) {
        if value > 0xff {
            self.is_8bit = false;
        }
        self.units.push(value);
        self.has_value = true;
    }

    // cpp: foundation/blink_base/wtf/text/string_builder.h:286-286
    // cpp: foundation/blink_base/wtf/text/string_builder.cc:182-193
    pub fn Resize(&mut self, new_size: u32) {
        assert!(new_size as usize <= self.units.len());
        self.units.truncate(new_size as usize);
        self.has_value = true;
    }

    // cpp: foundation/blink_base/wtf/text/string_builder.cc:52-60
    pub fn ToString(&self) -> String {
        if self.has_value {
            if self.is_8bit {
                String::Make8BitFrom16BitSource(&self.units)
            } else {
                String::from_utf16(&self.units)
            }
        } else {
            String::from("")
        }
    }

    // cpp: foundation/blink_base/wtf/text/string_builder.h:184-188
    pub fn AppendNumber(&mut self, value: impl std::fmt::Display) {
        self.Append(value);
    }

    // cpp: foundation/blink_base/wtf/text/string_builder.h:250-253
    // cpp: foundation/blink_base/wtf/text/string_builder.cc:39-49
    pub fn ReleaseString(&mut self) -> String {
        let units = std::mem::take(&mut self.units);
        let had_value = std::mem::take(&mut self.has_value);
        let was_8bit = std::mem::replace(&mut self.is_8bit, true);
        if had_value {
            if was_8bit {
                String::Make8BitFrom16BitSource(&units)
            } else {
                String::from_utf16(&units)
            }
        } else {
            String::from("")
        }
    }
}

// cpp: foundation/blink_base/wtf/text/string_builder.h:94-111,159-169
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "Rust" fn StringBuilderCapacity(builder: &StringBuilder) -> u32 {
    builder.units.capacity() as u32
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "Rust" fn StringBuilderReserveCapacity(builder: &mut StringBuilder, capacity: u32) {
    builder
        .units
        .reserve((capacity as usize).saturating_sub(builder.units.len()));
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "Rust" fn StringBuilderReserve16BitCapacity(builder: &mut StringBuilder, capacity: u32) {
    builder.is_8bit = false;
    StringBuilderReserveCapacity(builder, capacity);
}
