#![allow(non_snake_case)]

use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::sync::Arc;

use font_engine::fonts::canvas_rotation_in_vertical::IsCanvasRotationInVerticalUpright;
use font_engine::fonts::font_backend::{FontBackend, FontBackendFactory};
use font_engine::fonts::opentype::open_type_math_support::{MathConstants, OpenTypeMathSupport};
use font_engine::text::native::hyphenation::HyphenationBackend;
use font_engine::text::native::hyphenation_services::{
    NativeHyphenationResolver, NativeHyphenationResolverScope,
};
use font_engine::text::native::opentype_font::{FontMetric, OpenTypeFont, OpenTypeVariation};
use font_engine::text::native::phrase_break_services::{
    NativePhraseBreakResolver, NativePhraseBreakResolverScope,
};
use font_engine::{
    CanvasRotationInVertical, Font, FontBaseline, FontDescription, FontFamilyType, FontOrientation,
    FontPlatformData, FontSelectionValue, FontSmoothingMode, Hyphenation, LayoutLocale,
    ResolvedFontFeatures, ShapeResultView, SharedFontFamily, SimpleFontData, TextRenderingMode,
};
use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::{
    gfx, AtomicString, DynamicTo, EBorderStyle, EBoxDecorationBreak, EDisplay, ETextTransform,
    LayoutUnit, Length, MakeGarbageCollected, Member, Persistent, PhysicalOffset, PhysicalRect,
    PhysicalSize, String as BlinkString, TextOffsetMap, To, UnsupportedLayout,
    WritingMode as NativeWritingMode,
};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::fragment_item::{FragmentItem, ItemType};
use layoutng_fragment_tree::fragment_tree::{
    CollapsedTableEdge, CollapsedTablePaintData, CollapsedTableSection, ColumnRule,
    ColumnRulePaintData, FieldsetPaintData, FormControlPaintData, FragmentBoxSides, FragmentKind,
    FragmentNode, FragmentPaintRect, FrameSetPaintData, MathMLPaintData, MathMLPaintKind,
    PaintGlyph, PaintGlyphRun, PaintProperties, PaintResources, ReplacedContentPaintData,
    ScrollContainerPaintData, ScrollbarPaintAxis, ScrollbarPaintData, StitchedDecorationData,
    SvgTextPaintData, TablePaintColumn, TablePaintData,
};
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_fragment_tree::logical_fragment::LogicalFragment;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_fragment_tree::physical_fragment::PhysicalFragment;
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesHorizontal, kPhysicalAxesVertical, PhysicalAxes, PhysicalAxis,
};
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::ToLogicalSize;
use layoutng_geometry::geometry::logical_size::ToPhysicalSize;
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::appearance::AppearanceValue;
use layoutng_style::style::computed_style::{
    ComputedStyle as NativeComputedStyle, ComputedStyleBuilder,
};
use layoutng_style::style::computed_style_constants::{EVerticalAlign, PseudoId};
use layoutng_style::style::grid_enums::GridTrackSizingDirection;
use layoutng_style::text_transform_services::{
    NativeTextTransformResolver, NativeTextTransformResolverScope,
};

use crate::internal::boundary::font_from_selector::FontNewFromResolvedData;
use crate::internal::boundary::native_input::{CurrentNativeFontResolver, NativeFontResolverScope};
use crate::internal::constraint_space::ConstraintSpace as NativeConstraintSpace;
use crate::internal::form_control_types::AutofillState;
use crate::internal::fragmentation_utils::{
    FindPreviousBreakToken, IsBreakInside, OffsetInStitchedFragments,
};
use crate::internal::gap::gap_geometry::ContainerType;
use crate::internal::layout_block_flow::LayoutBlockFlow;
use crate::internal::layout_box::LayoutBox;
use crate::internal::layout_box_model_object::LayoutBoxModelObject;
use crate::internal::layout_font_resolver::{NativeFontRequest, NativeFontResolver};
use crate::internal::layout_inline::LayoutInline;
use crate::internal::layout_input::{
    BorderLineStyle, BoxDecorationBreak, ComputedStyle, ConstraintSpace, Display, Edges,
    ExtendedStyle, FontFace, FontSmoothing, FontVariation, HyphenationProvider,
    NativeNodeConstructionData, NodeKind, Offset, PaintImage, PhraseBreakProvider, Position, Size,
    TextDirection, TextTransform, TextTransformProvider, TransformMatrix, WritingMode,
};
use crate::internal::layout_invalidation_reason::{kFontsChanged, kStyleChange};
use crate::internal::layout_node_metadata::{Element, NativeNodeMetadataRelations, Node};
use crate::internal::layout_object::ApplyStyleChanges;
use crate::internal::layout_object::LayoutObject;
use crate::internal::layout_object_factory_set::{LayoutObjectFactory, LayoutObjectFactorySet};
use crate::internal::layout_pass_scope::LayoutPassScope;
use crate::internal::layout_replaced::LayoutReplaced;
use crate::internal::layout_text::LayoutText;
use crate::internal::layout_view::LayoutView;
use crate::internal::paint_input::PaintBlendMode;
use crate::internal::relative_utils::ComputeRelativeOffset;
use crate::internal::scroll_types::IncludeScrollbarsInRect;
use crate::internal::scrollbar_theme_metrics::ScrollbarThemeThickness;
use crate::internal::shapes::shape_image_services::{
    NativeShapeImageResolver, NativeShapeImageResolverScope,
};
use crate::internal::shapes::shape_outside_info::ShapeOutsideInfo;
use crate::internal::table_borders::EdgeSide;
use crate::layout_boundary_support::LayoutBoundaryEnvironment;

// cpp: layoutng/internal/boundary/layout_boundary.cc:76-111
pub(crate) struct ReusableLayoutEnvironment {
    snapshot: ConstraintSpace,
    // The native cache is shared with its root but only one layout pass uses
    // its mutable resolver at a time, as in the source shared_ptr boundary.
    fonts: UnsafeCell<Box<dyn NativeFontResolver>>,
}

impl ReusableLayoutEnvironment {
    fn new(input: &ConstraintSpace, resolver: Box<dyn NativeFontResolver>) -> Self {
        Self {
            snapshot: ConstraintSpace {
                available_size: input.available_size,
                fragmentainer_block_size: input.fragmentainer_block_size,
                compatibility_mode: input.compatibility_mode,
                printing: input.printing,
                vertical_scroll_enforced: input.vertical_scroll_enforced,
                scrollbar_theme: input.scrollbar_theme,
                minimum_font_physical_size: input.minimum_font_physical_size,
                device_pixel_ratio: input.device_pixel_ratio,
                canvas_draw_element_enabled: input.canvas_draw_element_enabled,
                viewport: input.viewport,
                control_theme: input.control_theme,
                hyphenation: input.hyphenation.clone(),
                phrase_break: input.phrase_break.clone(),
                text_transform: input.text_transform.clone(),
                font_backend_factory: input.font_backend_factory.clone(),
                fonts: input.fonts.clone(),
                ..ConstraintSpace::default()
            },
            fonts: UnsafeCell::new(resolver),
        }
    }

    fn Eligible(_space: &ConstraintSpace) -> bool {
        // Images and host callbacks have their own per-pass scopes; only fonts are cached.
        true
    }

    fn Matches(&self, space: &ConstraintSpace) -> bool {
        fn same_provider<T: ?Sized>(a: &Option<Arc<T>>, b: &Option<Arc<T>>) -> bool {
            match (a, b) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
        }
        Self::Eligible(space)
            && same_provider(&self.snapshot.hyphenation, &space.hyphenation)
            && same_provider(&self.snapshot.phrase_break, &space.phrase_break)
            && same_provider(&self.snapshot.text_transform, &space.text_transform)
            // Width/height change constraints and line breaking, not the
            // OpenType faces or resolved font requests cached here. Styles
            // computed from viewport units/media queries retain their normal
            // per-node font binding invalidation in the resident tree.
            && self.snapshot.fragmentainer_block_size == space.fragmentainer_block_size
            && self.snapshot.compatibility_mode == space.compatibility_mode
            && self.snapshot.printing == space.printing
            && self.snapshot.vertical_scroll_enforced == space.vertical_scroll_enforced
            && self.snapshot.scrollbar_theme == space.scrollbar_theme
            && self.snapshot.minimum_font_physical_size == space.minimum_font_physical_size
            && self.snapshot.device_pixel_ratio == space.device_pixel_ratio
            && self.snapshot.canvas_draw_element_enabled == space.canvas_draw_element_enabled
            && self.snapshot.control_theme == space.control_theme
            && match (
                &self.snapshot.font_backend_factory,
                &space.font_backend_factory,
            ) {
                (Some(left), Some(right)) => Arc::ptr_eq(left, right),
                (None, None) => true,
                _ => false,
            }
            && self.snapshot.fonts == space.fonts
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:2463-2494
struct FontBinding {
    cache: Option<Arc<ReusableLayoutEnvironment>>,
    transient: Option<Box<dyn NativeFontResolver>>,
    scope: Option<NativeFontResolverScope<'static>>,
    reused: bool,
}

impl FontBinding {
    fn new(root: &mut LayoutObject, space: &ConstraintSpace) -> Self {
        let root_node = root.GetNode();
        if root_node.is_null() {
            panic!("Layout root requires standalone metadata");
        }
        let previous = unsafe { &mut *root_node }.TakeReusableLayoutEnvironment();
        let mut cache = None;
        let mut transient: Option<Box<dyn NativeFontResolver>> = None;
        let mut reused = false;
        if let Some(previous) = previous {
            if previous.Matches(space) {
                cache = Some(previous);
                // Providers may change through interior mutability. Keep their
                // font resolver but conservatively prepare text again each pass.
                reused = space.hyphenation.is_none()
                    && space.phrase_break.is_none()
                    && space.text_transform.is_none();
            }
        }
        if cache.is_none() {
            let eligible = ReusableLayoutEnvironment::Eligible(space);
            let resolver = Box::new(InputFontResolver::new(
                &space.fonts,
                space.font_backend_factory.as_deref(),
            ));
            if eligible {
                cache = Some(Arc::new(ReusableLayoutEnvironment::new(space, resolver)));
            } else {
                transient = Some(resolver);
            }
        }
        let resolver: *mut dyn NativeFontResolver = if let Some(cache) = cache.as_ref() {
            // SAFETY: the layout environment owns this active pass; its scope
            // is dropped before the cache or root may release the resolver.
            unsafe { (&mut *cache.fonts.get()).as_mut() }
        } else {
            transient
                .as_mut()
                .expect("transient font resolver")
                .as_mut()
        };
        let scope = Some(unsafe { NativeFontResolverScope::new_from_raw(resolver) });
        Self {
            cache,
            transient,
            scope,
            reused,
        }
    }

    fn Commit(&mut self, root: &mut LayoutObject) {
        if let Some(cache) = self.cache.take() {
            let root_node = root.GetNode();
            unsafe { &mut *root_node }.SetReusableLayoutEnvironment(Some(cache));
        }
    }
}

impl Drop for FontBinding {
    fn drop(&mut self) {
        // The TLS pointer must be restored before either resolver is released.
        self.scope.take();
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:113-134
struct InputShapeImageResolver {
    images_: Vec<PaintImage>,
}

impl InputShapeImageResolver {
    fn new(images: &[PaintImage]) -> Self {
        // The C++ environment borrows ConstraintSpace::images. Owning the
        // immutable snapshot gives the Rust TLS resolver a stable allocation.
        Self {
            images_: images.to_vec(),
        }
    }
}

impl NativeShapeImageResolver for InputShapeImageResolver {
    fn Resolve(&self, resource_id: u64) -> *const PaintImage {
        let Some(image) = self.images_.iter().find(|image| image.id == resource_id) else {
            return std::ptr::null();
        };
        let expected_pixels = (image.width as usize)
            .checked_mul(image.height as usize)
            .and_then(|pixels| pixels.checked_mul(4));
        if image.width == 0
            || image.height == 0
            || !image.resolution_scale.is_finite()
            || image.resolution_scale <= 0.0
            || image
                .BitmapPixels()
                .is_some_and(|pixels| expected_pixels != Some(pixels.len()))
        {
            panic!("shape image requires dimensions, scale and RGBA pixels");
        }
        image as *const PaintImage
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:136-144
fn EqualFamilyName(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.bytes()
        .zip(right.bytes())
        .all(|(left, right)| left.to_ascii_lowercase() == right.to_ascii_lowercase())
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:153-175
fn ValidateFontFace(face: &FontFace) {
    // The public Rust family strings are valid UTF-8 by construction.
    if !face.weight.is_finite() || face.weight < 1.0 || face.weight > 1000.0 {
        panic!("FontFace.weight must be within 1..1000");
    }
    for range in &face.unicode_ranges {
        if range.start > range.end || range.end > 0x10ffff {
            panic!("FontFace unicode range is invalid");
        }
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:376-405
struct InputFontFace {
    family: String,
    weight: f64,
    italic: bool,
    font: Arc<OpenTypeFont>,
    skia_font: Option<Arc<dyn FontBackend>>,
    unicode_ranges: Vec<(u32, u32)>,
    variations: Vec<FontVariation>,
    supports_optical_size: bool,
    supports_weight: bool,
}

struct InputFontEntry {
    size: f64,
    specified_size: f64,
    letter_spacing: f64,
    word_spacing: f64,
    orientation: FontOrientation,
    language: String,
    families: Vec<String>,
    weight: f64,
    italic: bool,
    smoothing: FontSmoothing,
    font: Persistent<Font>,
}

struct InputFontResolver {
    faces_: Vec<InputFontFace>,
    fonts_: Vec<InputFontEntry>,
    font_indices_: HashMap<*const SimpleFontData, usize>,
    font_variations_: HashMap<*const SimpleFontData, Vec<FontVariation>>,
}

impl InputFontResolver {
    // cpp: layoutng/internal/boundary/layout_boundary.cc:146-200
    fn new(faces: &[FontFace], backend_factory: Option<&dyn FontBackendFactory>) -> Self {
        if faces.is_empty() {
            panic!("ConstraintSpace.fonts must contain an OpenType face");
        }
        let mut prepared_faces = Vec::with_capacity(faces.len());
        for face in faces {
            ValidateFontFace(face);
            let native_variations: Vec<OpenTypeVariation> = face
                .variations
                .iter()
                .map(|variation| OpenTypeVariation {
                    tag: variation.tag,
                    value: variation.value,
                })
                .collect();
            let unicode_ranges: Vec<(u32, u32)> = face
                .unicode_ranges
                .iter()
                .map(|range| (range.start, range.end))
                .collect();
            let open_type_font = Arc::new(OpenTypeFont::new_shared(
                face.bytes.clone(),
                face.face_index,
                1.0,
                &native_variations,
            ));
            let supports_optical_size = open_type_font.HasVariationAxis(OPTICAL_SIZE_TAG);
            let supports_weight = open_type_font.HasVariationAxis(WEIGHT_TAG);
            prepared_faces.push(InputFontFace {
                family: face.family.clone(),
                weight: face.weight,
                italic: face.italic,
                font: open_type_font,
                skia_font: backend_factory.map(|factory| {
                    factory.Create(
                        &face.bytes,
                        face.face_index,
                        &face.variations,
                        &face.native_family,
                        &face.metrics_family,
                        face.weight,
                        face.italic,
                    )
                }),
                unicode_ranges,
                variations: face.variations.clone(),
                supports_optical_size,
                supports_weight,
            });
        }
        Self {
            faces_: prepared_faces,
            fonts_: Vec::new(),
            font_indices_: HashMap::new(),
            font_variations_: HashMap::new(),
        }
    }

    // cpp: layoutng/internal/boundary/layout_boundary.cc:202-217
    fn CachedFont(&self, request: &NativeFontRequest<'_>) -> Option<*mut Font> {
        if !request.weight.is_finite() || request.weight < 1.0 || request.weight > 1000.0 {
            panic!("font request weight must be within 1..1000");
        }
        let entry = self.fonts_.iter().find(|entry| {
            entry.size == request.size
                && entry.specified_size == request.specified_size
                && entry.letter_spacing == request.letter_spacing
                && entry.word_spacing == request.word_spacing
                && entry.orientation == request.orientation
                && entry.language == request.language
                && entry.weight == request.weight
                && entry.italic == request.italic
                && entry.smoothing == request.smoothing
                && entry.families.as_slice() == request.families
        })?;
        Some(entry.font.Get())
    }

    // cpp: layoutng/internal/boundary/layout_boundary.cc:363-367
    fn FaceIndexFor(&self, font: *const SimpleFontData) -> usize {
        *self.font_indices_.get(&font).expect("font face index")
    }

    // cpp: layoutng/internal/boundary/layout_boundary.cc:369-374
    fn VariationsFor(&self, font: *const SimpleFontData) -> &[FontVariation] {
        self.font_variations_.get(&font).expect("font variations")
    }
}

impl NativeFontResolver for InputFontResolver {
    // cpp: layoutng/internal/boundary/layout_boundary.cc:202-361
    fn Resolve(&mut self, request: &NativeFontRequest<'_>) -> &mut Font {
        if let Some(font) = self.CachedFont(request) {
            return unsafe { &mut *font };
        }
        let mut description = FontDescription::default();
        description.SetSpecifiedSize(request.specified_size as f32);
        description.SetComputedSize(request.size as f32);
        description.SetAdjustedSize(request.size as f32);
        description.SetLetterSpacing(&Length::Fixed(request.letter_spacing));
        description.SetWordSpacing(&Length::Fixed(request.word_spacing));
        description.SetOrientation(request.orientation);
        description.SetWeight(FontSelectionValue::from_double(request.weight));
        description.SetStyle(FontSelectionValue::from_int(if request.italic {
            14
        } else {
            0
        }));
        description.SetFontSmoothing(match request.smoothing {
            FontSmoothing::kNone => FontSmoothingMode::kNoSmoothing,
            FontSmoothing::kAntialiased => FontSmoothingMode::kAntialiased,
            FontSmoothing::kSubpixelAntialiased => FontSmoothingMode::kSubpixelAntialiased,
            FontSmoothing::kAuto => FontSmoothingMode::kAutoSmoothing,
        });
        if !request.language.is_empty() {
            let locale = AtomicString::FromUtf8(request.language.as_bytes());
            description.SetLocale(LayoutLocale::GetShared(&locale));
        }
        if !request.families.is_empty() {
            let mut family_chain: Option<Arc<SharedFontFamily>> = None;
            for family in request.families.iter().rev() {
                let name = AtomicString::FromUtf8(family.as_bytes());
                family_chain = Some(SharedFontFamily::Create(
                    name,
                    FontFamilyType::kFamilyName,
                    family_chain,
                ));
            }
            description.SetFamily(family_chain.as_ref().expect("nonempty font family"));
        }
        let effective_size = description.EffectiveFontSize();
        let order = OrderFontFaces(&self.faces_, request);
        let mut ordered_fonts = Vec::with_capacity(self.faces_.len());
        for &index in &order {
            let face = &self.faces_[index];
            let open_type_font;
            let mut skia_font = face.skia_font.clone();
            let resolved_variations = ResolveFontVariations(
                &face.variations,
                face.supports_optical_size,
                face.supports_weight,
                effective_size as f64,
                request.weight,
            );
            let native_variations: Vec<OpenTypeVariation> = resolved_variations
                .iter()
                .map(|variation| OpenTypeVariation {
                    tag: variation.tag,
                    value: variation.value,
                })
                .collect();
            open_type_font = face.font.WithSizeVariationsAndSpecifiedSize(
                effective_size as f64,
                &native_variations,
                request.specified_size,
            );
            if face.supports_optical_size || face.supports_weight {
                skia_font = face
                    .skia_font
                    .as_ref()
                    .map(|backend| backend.WithVariations(&resolved_variations));
            }
            let platform = MakeGarbageCollected(FontPlatformData::new(
                open_type_font,
                skia_font,
                effective_size,
                request.weight >= 600.0 && face.weight < 600.0 && !face.supports_weight,
                request.italic && !face.italic,
                TextRenderingMode::kAutoTextRendering,
                ResolvedFontFeatures::default(),
                request.orientation,
                face.unicode_ranges.clone(),
            ));
            let simple = SimpleFontData::Create(platform);
            ordered_fonts.push(simple as *const SimpleFontData);
            self.font_indices_.entry(simple).or_insert(index);
            self.font_variations_
                .entry(simple)
                .or_insert(resolved_variations);
        }
        let primary_position = PrimaryFontPosition(&self.faces_, &order);
        let primary_font = ordered_fonts[primary_position];
        let font = MakeGarbageCollected(unsafe {
            FontNewFromResolvedData(&description, &ordered_fonts, primary_font)
        });
        self.fonts_.push(InputFontEntry {
            size: request.size,
            specified_size: request.specified_size,
            letter_spacing: request.letter_spacing,
            word_spacing: request.word_spacing,
            orientation: request.orientation,
            language: request.language.to_owned(),
            families: request.families.to_vec(),
            weight: request.weight,
            italic: request.italic,
            smoothing: request.smoothing,
            font: Persistent::from_ptr(font),
        });
        unsafe { &mut *font }
    }

    fn FaceIndex(&self, font_data: *const SimpleFontData) -> usize {
        self.FaceIndexFor(font_data)
    }

    fn Variations(&self, font_data: *const SimpleFontData) -> &[FontVariation] {
        self.VariationsFor(font_data)
    }
}

// Blink FontSelectionAlgorithm / CSS Fonts weight matching is directional.
// Above 500, prefer a heavier face before searching lighter weights; between
// 400 and 500, first search upward through 500, then downward, then above 500.
fn FontWeightRank(candidate: f64, requested: f64) -> (u8, f64) {
    if requested < 400.0 {
        if candidate <= requested {
            (0, requested - candidate)
        } else {
            (1, candidate - requested)
        }
    } else if requested <= 500.0 {
        if candidate >= requested && candidate <= 500.0 {
            (0, candidate - requested)
        } else if candidate < requested {
            (1, requested - candidate)
        } else {
            (2, candidate - requested)
        }
    } else if candidate >= requested {
        (0, candidate - requested)
    } else {
        (1, requested - candidate)
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:259-292
fn OrderFontFaces(faces: &[InputFontFace], request: &NativeFontRequest<'_>) -> Vec<usize> {
    let mut order = Vec::with_capacity(faces.len());
    let mut selected = vec![false; faces.len()];
    let append_best_face = |family: &str, order: &mut Vec<usize>, selected: &mut [bool]| loop {
        let mut best: Option<usize> = None;
        for (index, face) in faces.iter().enumerate() {
            if !selected[index] && !face.family.is_empty() && EqualFamilyName(family, &face.family)
            {
                let rank = (
                    face.italic != request.italic,
                    FontWeightRank(face.weight, request.weight),
                    index,
                );
                if best.map_or(true, |previous| {
                    let previous_face = &faces[previous];
                    let previous_rank = (
                        previous_face.italic != request.italic,
                        FontWeightRank(previous_face.weight, request.weight),
                        previous,
                    );
                    rank < previous_rank
                }) {
                    best = Some(index);
                }
            }
        }
        let Some(best) = best else { break };
        selected[best] = true;
        order.push(best);
    };
    for family in request.families {
        append_best_face(family, &mut order, &mut selected);
    }
    if request.families.is_empty() && !faces[0].family.is_empty() {
        append_best_face(&faces[0].family, &mut order, &mut selected);
    }
    if !request.families.is_empty() {
        append_best_face("serif", &mut order, &mut selected);
    }
    for index in 0..faces.len() {
        if !selected[index] {
            order.push(index);
        }
    }
    order
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:296-315
const OPTICAL_SIZE_TAG: u32 = u32::from_be_bytes(*b"opsz");
const WEIGHT_TAG: u32 = u32::from_be_bytes(*b"wght");

fn ResolveFontVariations(
    face_variations: &[FontVariation],
    supports_optical_size: bool,
    supports_weight: bool,
    effective_size: f64,
    request_weight: f64,
) -> Vec<FontVariation> {
    let mut variations = face_variations.to_vec();
    let mut set_variation = |tag, value: f64| {
        if let Some(setting) = variations.iter_mut().find(|setting| setting.tag == tag) {
            setting.value = value as f32;
        } else {
            variations.push(FontVariation {
                tag,
                value: value as f32,
            });
        }
    };
    if supports_optical_size {
        set_variation(OPTICAL_SIZE_TAG, effective_size);
    }
    if supports_weight {
        set_variation(WEIGHT_TAG, request_weight);
    }
    variations
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:338-349
fn PrimaryFontPosition(faces: &[InputFontFace], order: &[usize]) -> usize {
    for (position, &index) in order.iter().enumerate() {
        let ranges = &faces[index].unicode_ranges;
        if ranges.is_empty()
            || ranges
                .iter()
                .any(|range| range.0 <= 0x20 && 0x20 <= range.1)
        {
            return position;
        }
    }
    0
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:413-426
fn ProviderLastHyphenLocation(
    provider: &dyn HyphenationProvider,
    locale: &str,
    word: &[u16],
    before_index: u32,
) -> u32 {
    let result = provider.LastHyphenLocation(locale, word, before_index as usize);
    if result > word.len() || (result != 0 && result >= before_index as usize) {
        panic!("HyphenationProvider returned an invalid UTF-16 boundary");
    }
    result as u32
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:407-431
struct ProviderHyphenation {
    provider_: Arc<dyn HyphenationProvider>,
    locale_: String,
}

impl ProviderHyphenation {
    fn new(provider: Arc<dyn HyphenationProvider>, locale: String) -> Self {
        Self {
            provider_: provider,
            locale_: locale,
        }
    }
}

impl HyphenationBackend for ProviderHyphenation {
    fn LastHyphenLocation(&self, word: &BlinkString, before_index: u32) -> u32 {
        let characters: Vec<u16> = word.encode_utf16().collect();
        ProviderLastHyphenLocation(
            self.provider_.as_ref(),
            &self.locale_,
            &characters,
            before_index,
        )
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:433-458
struct InputHyphenationResolver {
    provider_: Option<Arc<dyn HyphenationProvider>>,
    dictionaries_: HashMap<String, Arc<Hyphenation>>,
}

impl InputHyphenationResolver {
    fn new(provider: Option<Arc<dyn HyphenationProvider>>) -> Self {
        Self {
            provider_: provider,
            dictionaries_: HashMap::new(),
        }
    }
}

impl NativeHyphenationResolver for InputHyphenationResolver {
    fn Resolve(&mut self, locale: &AtomicString) -> *mut Hyphenation {
        let Some(provider) = self.provider_.as_ref() else {
            return std::ptr::null_mut();
        };
        let key = locale.Utf8();
        if let Some(dictionary) = self.dictionaries_.get(&key) {
            return Arc::as_ptr(dictionary) as *mut Hyphenation;
        }
        // The font_engine base retains this backend and owns the inherited
        // hyphenation limits and virtual-default algorithms.
        let backend = Arc::new(ProviderHyphenation::new(Arc::clone(provider), key.clone()));
        let dictionary = Arc::new(Hyphenation::new_with_backend(backend));
        let result = Arc::as_ptr(&dictionary) as *mut Hyphenation;
        self.dictionaries_.insert(key, dictionary);
        result
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:472-497
fn TransformWithProvider(
    provider: Option<&dyn TextTransformProvider>,
    transform: TextTransform,
    locale: &str,
    text: &[u16],
    previous_character: u16,
    mut append_offset: impl FnMut(u32, u32),
) -> Vec<u16> {
    let Some(provider) = provider else {
        panic!("text-transform requires ConstraintSpace.text_transform");
    };
    let output = provider.Transform(transform, locale, text, previous_character);
    let mut previous_source = 0;
    let mut previous_target = 0;
    for entry in &output.offset_map {
        if entry.source > text.len()
            || entry.target > output.text.len()
            || entry.source < previous_source
            || entry.target < previous_target
        {
            panic!("TextTransformProvider returned an invalid offset map");
        }
        append_offset(entry.source as u32, entry.target as u32);
        previous_source = entry.source;
        previous_target = entry.target;
    }
    if text.len() != output.text.len() && output.offset_map.is_empty() {
        panic!("TextTransformProvider must map length-changing edits");
    }
    output.text
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:460-471
// cpp: layoutng/internal/boundary/layout_boundary.cc:498-503
struct InputTextTransformResolver {
    provider_: Option<Arc<dyn TextTransformProvider>>,
}

impl InputTextTransformResolver {
    fn new(provider: Option<Arc<dyn TextTransformProvider>>) -> Self {
        Self {
            provider_: provider,
        }
    }
}

impl NativeTextTransformResolver for InputTextTransformResolver {
    fn Transform(
        &self,
        transform: ETextTransform,
        locale: &AtomicString,
        text: &BlinkString,
        previous_character: u16,
        mut offset_map: Option<&mut TextOffsetMap>,
    ) -> BlinkString {
        let input: Vec<u16> = text.Span16().unwrap_or_default().to_vec();
        let output = TransformWithProvider(
            self.provider_.as_deref(),
            TextTransform(transform.bits()),
            &locale.Utf8(),
            &input,
            previous_character,
            |source, target| {
                if let Some(offset_map) = offset_map.as_deref_mut() {
                    offset_map.Append(source, target);
                }
            },
        );
        BlinkString::from_utf16(&output)
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:516-534
fn NextPhraseBreak(
    provider: Option<&dyn PhraseBreakProvider>,
    locale: &str,
    text: &[u16],
    from_index: u32,
    range_end: u32,
) -> u32 {
    let Some(provider) = provider else {
        panic!("word-break:auto-phrase requires ConstraintSpace.phrase_break");
    };
    if from_index > range_end || range_end as usize > text.len() {
        panic!("Invalid native phrase-break range");
    }
    let result = provider.NextBreakLocation(locale, text, from_index as usize, range_end as usize);
    if result < from_index as usize
        || result > range_end as usize
        || (result > 0
            && result < text.len()
            && (0xdc00..=0xdfff).contains(&text[result])
            && (0xd800..=0xdbff).contains(&text[result - 1]))
    {
        panic!("PhraseBreakProvider returned an invalid UTF-16 boundary");
    }
    result as u32
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:505-515
// cpp: layoutng/internal/boundary/layout_boundary.cc:535-539
struct InputPhraseBreakResolver {
    provider_: Option<Arc<dyn PhraseBreakProvider>>,
}

impl InputPhraseBreakResolver {
    fn new(provider: Option<Arc<dyn PhraseBreakProvider>>) -> Self {
        Self {
            provider_: provider,
        }
    }
}

impl NativePhraseBreakResolver for InputPhraseBreakResolver {
    fn NextBreak(
        &self,
        locale: &AtomicString,
        text: &BlinkString,
        from_index: u32,
        range_end: u32,
    ) -> u32 {
        let input: Vec<u16> = text.Span16().unwrap_or_default().to_vec();
        NextPhraseBreak(
            self.provider_.as_deref(),
            &locale.Utf8(),
            &input,
            from_index,
            range_end,
        )
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:2502-2514
// This is the text-transform/image suffix of Storage's constructor. The
// hyphenation and phrase-break prefix still needs the font_engine scopes.
struct InputTextAndImageScopes {
    text_transform: Box<InputTextTransformResolver>,
    text_transform_scope: Option<NativeTextTransformResolverScope<'static>>,
    shape_images: Box<InputShapeImageResolver>,
    shape_image_scope: Option<NativeShapeImageResolverScope<'static>>,
}

impl InputTextAndImageScopes {
    fn new(space: &ConstraintSpace) -> Self {
        let mut text_transform = Box::new(InputTextTransformResolver::new(
            space.text_transform.clone(),
        ));
        let text_transform_ptr = text_transform.as_mut() as *mut dyn NativeTextTransformResolver;
        let text_transform_scope =
            Some(unsafe { NativeTextTransformResolverScope::new_from_raw(text_transform_ptr) });
        let shape_images = Box::new(InputShapeImageResolver::new(&space.images));
        let shape_image_ptr = shape_images.as_ref() as *const dyn NativeShapeImageResolver;
        let shape_image_scope =
            Some(unsafe { NativeShapeImageResolverScope::new_from_raw(shape_image_ptr) });
        Self {
            text_transform,
            text_transform_scope,
            shape_images,
            shape_image_scope,
        }
    }
}

impl Drop for InputTextAndImageScopes {
    fn drop(&mut self) {
        // C++ destroys fields in reverse construction order. Restore both
        // TLS slots before dropping their boxed resolvers.
        self.shape_image_scope.take();
        self.text_transform_scope.take();
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:541-559
struct DeferredFontResolver {
    font_: Persistent<Font>,
}

impl DeferredFontResolver {
    fn new() -> Self {
        Self {
            font_: Persistent::from_ptr(MakeGarbageCollected(Font::default())),
        }
    }
}

impl NativeFontResolver for DeferredFontResolver {
    fn Resolve(&mut self, _request: &NativeFontRequest<'_>) -> &mut Font {
        unsafe { &mut *self.font_.Get() }
    }

    fn FaceIndex(&self, _font_data: *const SimpleFontData) -> usize {
        unreachable!("DeferredFontResolver cannot select a face")
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:560-565
pub(crate) fn ConstructionFontResolver(space: &ConstraintSpace) -> Box<dyn NativeFontResolver> {
    if space.fonts.is_empty() {
        return Box::new(DeferredFontResolver::new());
    }
    Box::new(InputFontResolver::new(
        &space.fonts,
        space.font_backend_factory.as_deref(),
    ))
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:567-693
pub(crate) fn CreateLayoutObject(
    node: &mut Node,
    style: &NativeComputedStyle,
    factories: &LayoutObjectFactorySet,
) -> *mut LayoutObject {
    let require = |factory: Option<LayoutObjectFactory>, capability: &str, node: &mut Node| {
        let Some(factory) = factory else {
            panic!("{capability} layout module is not installed");
        };
        let object = factory(node, style);
        if object.is_null() {
            panic!("{capability} layout factory rejected its selected node");
        }
        object
    };

    if node.IsTextNode() {
        let mut ancestor = node.parentNode();
        while !ancestor.is_null() {
            if unsafe { &*ancestor }.node.InputKind() == NodeKind::kSvgText {
                return require(factories.svg, "svg", node);
            }
            ancestor = unsafe { &*ancestor }.node.parentNode();
        }
        return require(factories.inline_text, "inline", node);
    }

    assert!(
        node.IsElementNode(),
        "CreateLayoutObject requires an element"
    );
    let element = node as *mut Node as *mut Element;
    if node.parentNode().is_null() {
        if node.InputKind() != NodeKind::kBox || style.Display() != EDisplay::kBlock {
            panic!("Layout root must be a block box suitable for LayoutView");
        }
        let container = unsafe { &mut (*element).container };
        return MakeGarbageCollected(LayoutView::new(container)) as *mut LayoutObject;
    }
    if style.StyleType() == PseudoId::kPseudoIdFirstLetter && !style.InitialLetter().IsNormal() {
        let container = unsafe { &mut (*element).container };
        return MakeGarbageCollected(LayoutBlockFlow::new(container)) as *mut LayoutObject;
    }
    let element_data = unsafe { &*element }.InputElementData().as_ref();
    if element_data.is_some_and(|data| data.text_control_inner_editor) {
        return require(factories.forms, "forms", node);
    }
    match node.InputKind() {
        NodeKind::kLineBreak => return require(factories.inline_line_break, "inline", node),
        NodeKind::kWordBreak => return require(factories.inline_word_break, "inline", node),
        NodeKind::kListMarker => return require(factories.list, "list", node),
        NodeKind::kReplaced | NodeKind::kFrame => {
            return require(factories.replaced, "replaced", node)
        }
        NodeKind::kFormControl => {
            let Some(factory) = factories.forms else {
                panic!("forms layout module is not installed");
            };
            let control = factory(node, style);
            if !control.is_null() {
                return control;
            }
            if element_data
                .and_then(|data| data.select_uses_menu_list)
                .unwrap_or(false)
            {
                return require(factories.flex, "flex", node);
            }
        }
        NodeKind::kFieldset => return require(factories.forms, "forms", node),
        NodeKind::kFrameSet => return require(factories.frameset, "frameset", node),
        NodeKind::kSvgRoot
        | NodeKind::kSvgGroup
        | NodeKind::kSvgForeignObject
        | NodeKind::kSvgText
        | NodeKind::kSvgInline
        | NodeKind::kSvgTSpan
        | NodeKind::kSvgTextPath
        | NodeKind::kSvgShape => return require(factories.svg, "svg", node),
        _ => {}
    }

    match style.Display() {
        EDisplay::kInline | EDisplay::kRuby | EDisplay::kRubyText => {
            require(factories.inline_box, "inline", node)
        }
        EDisplay::kBlock | EDisplay::kFlowRoot | EDisplay::kInlineBlock => {
            let container = unsafe { &mut (*element).container };
            MakeGarbageCollected(LayoutBlockFlow::new(container)) as *mut LayoutObject
        }
        EDisplay::kListItem | EDisplay::kFlowRootListItem | EDisplay::kInlineFlowRootListItem => {
            require(factories.list, "list", node)
        }
        EDisplay::kTable
        | EDisplay::kInlineTable
        | EDisplay::kTableRowGroup
        | EDisplay::kTableHeaderGroup
        | EDisplay::kTableFooterGroup
        | EDisplay::kTableRow
        | EDisplay::kTableColumnGroup
        | EDisplay::kTableColumn
        | EDisplay::kTableCell
        | EDisplay::kTableCaption => require(factories.table, "table", node),
        EDisplay::kFlex | EDisplay::kInlineFlex => require(factories.flex, "flex", node),
        EDisplay::kGrid | EDisplay::kInlineGrid => require(factories.grid, "grid", node),
        EDisplay::kGridLanes => require(factories.grid_lanes, "grid-lanes", node),
        EDisplay::kMath | EDisplay::kBlockMath => require(factories.mathml, "mathml", node),
        EDisplay::kBlockRuby => require(factories.block_ruby, "inline", node),
        EDisplay::kLayoutCustom => require(factories.custom, "custom", node),
        _ => panic!("Display value has no standalone native object"),
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:695-695
fn Number(value: LayoutUnit) -> f64 {
    value.ToDouble()
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:697-703
fn InputId(object: *const LayoutObject) -> u64 {
    if !object.is_null() {
        let node = unsafe { &*object }.GetNode();
        if !node.is_null() {
            return unsafe { &*node }.InputId();
        }
    }
    0
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:2533-2543
fn PublicWritingMode(mode: NativeWritingMode) -> WritingMode {
    match mode {
        NativeWritingMode::kHorizontalTb => WritingMode::kHorizontalTb,
        NativeWritingMode::kVerticalRl | NativeWritingMode::kSidewaysRl => WritingMode::kVerticalRl,
        NativeWritingMode::kVerticalLr | NativeWritingMode::kSidewaysLr => WritingMode::kVerticalLr,
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:705-712
struct SourceEntry {
    node: *const Node,
    logical_tree_order: u64,
}
type SourceMap = HashMap<*const LayoutObject, SourceEntry>;
type StyleSourceMap = HashMap<*const NativeComputedStyle, *const ComputedStyle>;

// cpp: layoutng/internal/boundary/layout_boundary.cc:735-830
fn ExportPaintProperties(
    object: *const LayoutObject,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    effective_style: *const NativeComputedStyle,
    resources: &Arc<PaintResources>,
) -> PaintProperties {
    let mut output = PaintProperties::default();
    output.resources = Some(Arc::clone(resources));
    if object.is_null() {
        return output;
    }
    // LayoutObject's base-first representation preserves the native
    // DisplayItemClient subobject, including anonymous layout clients.
    // cpp: core/layout/layout_object.h:280
    // cpp: platform/graphics/paint/display_item_client.h:33-35,71-82
    let client = unsafe { &*(object as *const DisplayItemClient) };
    output.display_item_client_id = client.Id() as u64;
    output.display_item_client_is_cacheable = client.IsCacheable();
    output.display_item_client_is_just_created = client.IsJustCreated();
    output.display_item_raster_effect_outset = client.VisualRectOutsetForRasterEffects();
    let layout_box = DynamicTo::<LayoutBox>(object);
    if !layout_box.is_null() {
        // Preserve the LayoutBox virtual override rather than calling only
        // the DisplayItemClient base default (layout_box.cc:4099-4108).
        output.display_item_raster_effect_outset =
            unsafe { &*layout_box }.VisualRectOutsetForRasterEffects();
    } else if unsafe { &*object }.IsSVGShape() {
        // layout_svg_shape.cc:664-674: hairline raster expansion is metadata,
        // not an inflation of the stored VisualRect or chunk bounds.
        let style = unsafe { &*object }.StyleRef();
        if style.HasVisibleStroke() {
            output.display_item_raster_effect_outset =
                if style.CapStyle() == foundation::LineCap::kButtCap {
                    layoutng_fragment_tree::fragment_tree::RasterEffectOutset::kHalfPixel
                } else {
                    layoutng_fragment_tree::fragment_tree::RasterEffectOutset::kWholePixel
                };
        }
    }
    // PaintLayerPainter uses the layer's own client, distinct from this box.
    // cpp: core/paint/paint_layer_painter.cc:452-454,484-486,730-732
    let box_model = DynamicTo::<LayoutBoxModelObject>(object);
    if !box_model.is_null() {
        let layer = unsafe { &*box_model }.Layer();
        if !layer.is_null() {
            let layer = unsafe { &*layer };
            output.paint_layer_client_id = layer.Id();
            output.paint_layer_client_is_cacheable = layer.IsCacheable();
            output.paint_layer_client_is_just_created = layer.IsJustCreated();
            output.paint_layer_is_self_painting = layer.IsSelfPaintingLayer();
        }
    }
    let Some(source_entry) = sources.get(&object) else {
        return output;
    };
    let source = unsafe { &*source_entry.node };
    let mut paint_style = source.InputStyle();
    if !effective_style.is_null() {
        if let Some(&effective_source) = style_sources.get(&effective_style) {
            paint_style = unsafe { &*effective_source };
        }
    }
    let zoomed_paint_style = crate::internal::css_zoom::ZoomedStyle(paint_style);
    let paint_style = zoomed_paint_style.as_ref();
    output.has_source = true;
    output.logical_tree_order = source_entry.logical_tree_order;
    let mut parent = unsafe { &*object }.Parent();
    while !parent.is_null() {
        if let Some(parent_source) = sources.get(&(parent as *const LayoutObject)) {
            output.logical_parent_node_id = Some(unsafe { &*parent_source.node }.InputId());
            break;
        }
        parent = unsafe { &*parent }.Parent();
    }
    output.source_kind = source.InputKind();
    output.effective_zoom = paint_style
        .extended
        .as_ref()
        .map_or(1.0, |e| e.effective_zoom);
    output.style = paint_style.paint.clone();
    output.border = paint_style.border;
    output.border_styles = paint_style.border_styles;
    output.padding = paint_style.padding;
    output.display = paint_style.display;
    output.position = paint_style.position;
    output.fixed_to_view =
        !layout_box.is_null() && unsafe { &*layout_box }.IsFixedToView(std::ptr::null());
    output.floating = paint_style.floating;
    if paint_style.position == Position::kSticky {
        let box_model = DynamicTo::<LayoutBoxModelObject>(object);
        if !box_model.is_null() {
            let offset = unsafe { &*box_model }.StickyPositionOffset();
            output.sticky_offset = Offset {
                x: Number(offset.left),
                y: Number(offset.top),
            };
        }
    }
    output.writing_mode = paint_style.writing_mode;
    output.direction = paint_style.direction;
    if let Some(extended) = &paint_style.extended {
        output.overflow_x = extended.overflow_x;
        output.overflow_y = extended.overflow_y;
    }
    let element_data = if source.IsElementNode() {
        unsafe { &*(source as *const Node as *const Element) }
            .InputElementData()
            .as_ref()
    } else {
        None
    };
    if let Some(element_data) = element_data {
        output.scroll_offset = element_data.scroll_offset;
        if let Some(svg_shape) = &element_data.svg_shape {
            output.svg_shape = Some(Arc::new(svg_shape.clone()));
        }
        output.svg_view_box = element_data.svg_view_box;
    }
    // Preserve the real editor identity rather than reconstructing its synthetic id.
    let mut ancestor = source as *const Node;
    while !ancestor.is_null() {
        let node = unsafe { &*ancestor };
        if node.InputKind() == NodeKind::kFormControl {
            output.text_control_host = Some(node.InputId());
            break;
        }
        ancestor = node.parentNode().cast();
    }
    if element_data.is_some_and(|data| data.text_control_inner_editor) {
        output.text_control_inner_editor = true;
        let style = unsafe { &*object }.StyleRef();
        let font = style.GetFontHeightForDefaultBaseline();
        let font_height = Number(font.LineHeight());
        let line_height = style.ComputedLineHeight() as f64;
        let editor = unsafe { &*layout_box };
        let size = editor.StitchedSize();
        let border = output.border;
        let padding = output.padding;
        let left = border.left + padding.left;
        let right = (Number(size.width) - border.right - padding.right - 1.0).max(left);
        let align = paint_style
            .extended
            .as_ref()
            .map_or(crate::internal::layout_input::TextAlign::kStart, |e| {
                e.text_align
            });
        use crate::internal::layout_input::TextAlign;
        let rtl = output.direction == TextDirection::kRtl;
        let x = match align {
            TextAlign::kCenter | TextAlign::kWebkitCenter => (left + right) * 0.5,
            TextAlign::kRight | TextAlign::kWebkitRight => right,
            TextAlign::kEnd if !rtl => right,
            TextAlign::kStart | TextAlign::kJustify if rtl => right,
            _ => left,
        };
        output.text_control_empty_caret = Some(FragmentPaintRect {
            offset: Offset {
                x,
                y: border.top + padding.top + (line_height - font_height) * 0.5,
            },
            size: Size {
                width: 1.0,
                height: font_height,
            },
        });
    }
    let appearance = paint_style
        .extended
        .as_ref()
        .map_or(AppearanceValue::kNone, |extended| {
            extended.effective_appearance
        });
    if appearance != AppearanceValue::kNone
        || element_data.is_some_and(|element_data| element_data.control_focused)
    {
        let mut control = FormControlPaintData {
            appearance,
            ..FormControlPaintData::default()
        };
        if let Some(element_data) = element_data {
            control.r#type = element_data.form_control_type;
            control.checked = element_data.control_checked;
            control.indeterminate = element_data.control_indeterminate;
            control.disabled = element_data.control_disabled;
            control.read_only = element_data.control_read_only;
            control.hovered = element_data.control_hovered;
            control.active = element_data.control_active;
            control.focused = element_data.control_focused;
            control.auto_focus_ring = element_data.control_focused
                && appearance == AppearanceValue::kNone
                && !paint_style
                    .extended
                    .as_ref()
                    .is_some_and(|extended| extended.has_author_outline);
            control.autofilled = element_data.autofill_state != AutofillState::kNotFilled;
            control.value_ratio = element_data
                .control_value_ratio
                .or(element_data.range_value_ratio);
        }
        if let Some(accent_color) = paint_style.paint.accent_color {
            control.accent_color = accent_color;
        }
        if source.InputKind() == NodeKind::kFormControl {
            let style = unsafe { &*object }.StyleRef();
            let height = style.GetFontHeightForDefaultBaseline();
            output.text_control_caret_metrics =
                Some(crate::fragment_tree::TextControlCaretMetrics {
                    font_height: Number(height.LineHeight()),
                    ascent: Number(height.ascent),
                    line_height: style.ComputedLineHeight() as f64,
                });
        }
        output.form_control = Some(Arc::new(control));
    }
    output
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:832-844
struct GlyphExportContext {
    runs: *mut Vec<PaintGlyphRun>,
    writing_mode: WritingMode,
    current_font: *const SimpleFontData,
    current_rotation: CanvasRotationInVertical,
    font_size: f64,
    baseline: f64,
    last_run: usize,
    last_glyph: usize,
    last_total_advance: f32,
    has_last_glyph: bool,
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:846-929
fn ExportGlyph(
    opaque: *mut c_void,
    character_index: u32,
    glyph: u16,
    glyph_offset: gfx::Vector2dF,
    total_advance: f32,
    is_horizontal: bool,
    canvas_rotation: CanvasRotationInVertical,
    font: *const SimpleFontData,
) {
    let context = unsafe { &mut *(opaque as *mut GlyphExportContext) };
    let runs = unsafe { &mut *context.runs };
    if context.has_last_glyph {
        runs[context.last_run].glyphs[context.last_glyph].advance =
            (total_advance - context.last_total_advance) as f64;
    }
    if runs.is_empty()
        || context.current_font != font
        || runs
            .last()
            .is_some_and(|run| run.horizontal != is_horizontal)
        || context.current_rotation != canvas_rotation
    {
        context.current_font = font;
        context.current_rotation = canvas_rotation;
        let font = unsafe { &*font };
        let platform = font.PlatformData();
        let raw_font = platform.RawFont();
        // Shaping may select a platform size different from the CSS computed size.
        let run_font_size = platform.size() as f64;
        let default_thickness = 1.0_f64.max(run_font_size / 16.0);
        let underline_position = -raw_font
            .Metric(FontMetric::kUnderlineOffset)
            .unwrap_or(-run_font_size / 10.0);
        let underline_thickness = raw_font
            .Metric(FontMetric::kUnderlineSize)
            .unwrap_or(default_thickness)
            .abs();
        let strikeout_position = raw_font
            .Metric(FontMetric::kStrikeoutOffset)
            .unwrap_or(run_font_size * 0.3);
        let strikeout_thickness = raw_font
            .Metric(FontMetric::kStrikeoutSize)
            .unwrap_or(default_thickness)
            .abs();
        let resolver = unsafe { &*CurrentNativeFontResolver() };
        let variations = resolver.Variations(context.current_font);
        runs.push(PaintGlyphRun {
            font_face_index: resolver.FaceIndex(context.current_font) as u32,
            font_variations: variations.to_vec(),
            font_size: run_font_size,
            ascent: font.GetFontMetrics().FloatAscent() as f64,
            // TextFragmentPainter establishes the origin from the primary font.
            baseline: context.baseline,
            underline_position,
            underline_thickness,
            strikeout_position,
            strikeout_thickness,
            horizontal: is_horizontal,
            rtl: false,
            synthetic_bold: platform.SyntheticBold(),
            synthetic_italic: platform.SyntheticItalic(),
            writing_mode: context.writing_mode,
            ..PaintGlyphRun::default()
        });
    }
    let run = runs.last_mut().expect("glyph run");
    let mut vertical_baseline_x_offset = 0.0;
    if !is_horizontal && IsCanvasRotationInVerticalUpright(canvas_rotation) {
        let metrics = unsafe { &*font }.GetFontMetrics();
        vertical_baseline_x_offset =
            (metrics.FloatAscent() - metrics.FloatAscentFor(FontBaseline::kCentralBaseline)) as f64;
    }
    run.glyphs.push(PaintGlyph {
        id: glyph as u32,
        character_index,
        canvas_rotation: canvas_rotation as u8,
        offset: if is_horizontal {
            Offset {
                x: (total_advance + glyph_offset.x()) as f64,
                y: glyph_offset.y() as f64,
            }
        } else {
            Offset {
                x: glyph_offset.x() as f64 + vertical_baseline_x_offset,
                y: (total_advance + glyph_offset.y()) as f64,
            }
        },
        ..PaintGlyph::default()
    });
    context.last_glyph = run.glyphs.len() - 1;
    context.last_run = runs.len() - 1;
    context.last_total_advance = total_advance;
    context.has_last_glyph = true;
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:931-951
fn ExportShapeResult(
    shape: *const ShapeResultView,
    font_size: f64,
    baseline: f64,
    writing_mode: WritingMode,
    runs: &mut Vec<PaintGlyphRun>,
) {
    if shape.is_null() {
        return;
    }
    let shape = unsafe { &*shape };
    let mut context = GlyphExportContext {
        runs: runs as *mut Vec<PaintGlyphRun>,
        writing_mode,
        current_font: std::ptr::null(),
        current_rotation: CanvasRotationInVertical::kRegular,
        font_size,
        baseline,
        last_run: 0,
        last_glyph: 0,
        last_total_advance: 0.0,
        has_last_glyph: false,
    };
    shape.ForEachGlyph(0.0, ExportGlyph, &mut context as *mut _ as *mut c_void);
    if context.has_last_glyph {
        runs[context.last_run].glyphs[context.last_glyph].advance =
            (shape.Width() - context.last_total_advance) as f64;
    }
    for run in runs {
        run.rtl = shape.IsRtl();
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:953-981
fn ExportGlyphRuns(item: &FragmentItem, paint: &mut PaintProperties) {
    if !item.IsText() {
        return;
    }
    let font_size = if item.IsSvgText() {
        item.ScaledFont().GetFontDescription().ComputedSize() as f64
    } else {
        item.Style().ComputedFontSize() as f64
    };
    let primary_font = item.ScaledFont().PrimaryFont();
    let baseline = if primary_font.is_null() {
        0.0
    } else {
        unsafe { &*primary_font }.GetFontMetrics().Ascent() as f64
    };
    ExportShapeResult(
        item.TextShapeResult(),
        font_size,
        baseline,
        paint.writing_mode,
        &mut paint.glyph_runs,
    );
    let smoothing = match item.Style().GetFontDescription().FontSmoothing() {
        FontSmoothingMode::kNoSmoothing => FontSmoothing::kNone,
        FontSmoothingMode::kAntialiased => FontSmoothing::kAntialiased,
        FontSmoothingMode::kSubpixelAntialiased => FontSmoothing::kSubpixelAntialiased,
        FontSmoothingMode::kAutoSmoothing => FontSmoothing::kAuto,
    };
    for run in &mut paint.glyph_runs {
        run.font_smoothing = smoothing;
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:983-1001
fn ExportListMarkerSymbol(item: &FragmentItem, paint: &mut PaintProperties) {
    if !item.IsSymbolMarker() {
        return;
    }
    let mut object = item.GetLayoutObject();
    while !object.is_null() {
        let layout_object = unsafe { &*object };
        if layout_object.IsLayoutInsideListMarker() {
            paint.list_marker_inside = true;
        }
        let node = layout_object.GetNode();
        if !node.is_null() && unsafe { &*node }.InputKind() == NodeKind::kListItem {
            let defaults = ExtendedStyle::default();
            let style = unsafe { &*node }.InputStyle();
            paint.list_marker_symbol =
                Some(style.extended.as_ref().unwrap_or(&defaults).list_style_type);
            return;
        }
        object = layout_object.Parent();
    }
    panic!("symbol list marker has no list-item ancestor");
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1003-1019
fn FragmentLocalAffineTransform(
    transform: &AffineTransform,
    fragment_offset: PhysicalOffset,
) -> TransformMatrix {
    let x = Number(fragment_offset.left);
    let y = Number(fragment_offset.top);
    let mut output = TransformMatrix::default();
    output.values[0] = transform.A();
    output.values[1] = transform.B();
    output.values[4] = transform.C();
    output.values[5] = transform.D();
    // T(-fragment_offset) * transform * T(fragment_offset).
    output.values[12] = transform.A() * x + transform.C() * y + transform.E() - x;
    output.values[13] = transform.B() * x + transform.D() * y + transform.F() - y;
    output
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1021-1046
fn ExportSvgTextPaintData(
    item: &FragmentItem,
    fragment_rect: PhysicalRect,
    paint: &mut PaintProperties,
) {
    let svg = item.GetSvgFragmentData();
    if svg.is_null() {
        return;
    }
    let svg = unsafe { &*svg };
    let scaling_factor = item.SvgScalingFactor();
    assert!(scaling_factor.is_finite() && scaling_factor > 0.0);
    assert!(svg.length_adjust_scale.is_finite() && svg.length_adjust_scale > 0.0);
    let mut output = SvgTextPaintData {
        glyph_origin: Offset {
            x: svg.rect.x() as f64 - Number(fragment_rect.offset.left),
            y: svg.rect.y() as f64 - Number(fragment_rect.offset.top),
        },
        untransformed_size: Size {
            width: (svg.rect.width() / svg.length_adjust_scale) as f64,
            height: svg.rect.height() as f64,
        },
        scaling_factor: scaling_factor as f64,
        ..SvgTextPaintData::default()
    };
    let transform = item.BuildSvgTransformForPaint();
    output.has_transform = scaling_factor != 1.0 || !transform.IsIdentity();
    output.local_transform = FragmentLocalAffineTransform(&transform, fragment_rect.offset);
    output.hidden = item.IsHiddenForPaint();
    paint.svg_text = Some(Arc::new(output));
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1048-1110
fn ExportStitchedDecorationData(
    box_fragment: &PhysicalBoxFragment,
) -> Option<StitchedDecorationData> {
    let object = box_fragment.GetLayoutObject();
    if box_fragment.IsInlineBox()
        || box_fragment.IsOnlyForNode()
        || object.is_null()
        || !unsafe { &*object }.IsBox()
        || unsafe { &*object }.IsFieldset()
        || box_fragment.Style().BoxDecorationBreak() != EBoxDecorationBreak::kSlice
    {
        return None;
    }

    if box_fragment.IsTable() {
        let writing_direction = box_fragment.Style().GetWritingDirection();
        let mut stitched_grid = layoutng_geometry::geometry::logical_rect::LogicalRect::default();
        let mut current_grid = layoutng_geometry::geometry::logical_rect::LogicalRect::default();
        let mut stitched_block_size = LayoutUnit::default();
        let mut has_grid = false;
        let mut found_current = false;
        let layout_box = To::<LayoutBox>(object);
        for fragment in unsafe { &*layout_box }.PhysicalFragments() {
            let mut grid = fragment.TableGridRect();
            grid.offset.block_offset += stitched_block_size;
            if !has_grid {
                stitched_grid = grid;
                has_grid = true;
            } else {
                stitched_grid.Unite(&grid);
            }
            if std::ptr::eq(fragment, box_fragment) {
                current_grid = grid;
                found_current = true;
            }
            stitched_block_size += LogicalFragment::new(writing_direction, fragment).BlockSize();
        }
        if !has_grid || !found_current {
            return None;
        }
        current_grid.offset -= stitched_grid.offset;
        let stitched_converter =
            WritingModeConverter::from_logical_size(writing_direction, stitched_grid.size);
        let current_in_stitched = stitched_converter.ToPhysicalRect(current_grid);
        let fragment_converter = WritingModeConverter::new(writing_direction, box_fragment.Size());
        let local_grid = fragment_converter.ToPhysicalRect(box_fragment.TableGridRect());
        let stitched_size = stitched_converter.ToPhysicalSize(stitched_grid.size);
        return Some(StitchedDecorationData {
            fragment_origin: Offset {
                x: Number(local_grid.offset.left),
                y: Number(local_grid.offset.top),
            },
            fragment_offset: Offset {
                x: Number(current_in_stitched.offset.left),
                y: Number(current_in_stitched.offset.top),
            },
            size: Size {
                width: Number(stitched_size.width),
                height: Number(stitched_size.height),
            },
        });
    }

    let mut stitched_size = PhysicalSize::default();
    let stitched_offset = OffsetInStitchedFragments(box_fragment, &mut stitched_size);
    Some(StitchedDecorationData {
        fragment_offset: Offset {
            x: Number(stitched_offset.left),
            y: Number(stitched_offset.top),
        },
        size: Size {
            width: Number(stitched_size.width),
            height: Number(stitched_size.height),
        },
        ..StitchedDecorationData::default()
    })
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1112-1136
fn ExportBoxPaintGeometry(box_fragment: &PhysicalBoxFragment, paint: &mut PaintProperties) {
    let layout_box = DynamicTo::<LayoutBox>(box_fragment.GetLayoutObject());
    if !layout_box.is_null() {
        let layout_box = unsafe { &*layout_box };
        paint.scroll_size = Size {
            width: Number(layout_box.ScrollWidth()),
            height: Number(layout_box.ScrollHeight()),
        };
        // Export the native scroll owner, not the unpropagated input CSS.
        // Blink's UpdateScrollNode/UpdateScrollTranslation use this same area.
        let area = layout_box.GetScrollableArea();
        let needs_local_scroll = !area.is_null()
            && layout_box.IsScrollContainer()
            && (layout_box.ScrollsOverflow() || paint.scroll_offset != Offset::default());
        if layout_box.IsLayoutView() || needs_local_scroll {
            let clip = layout_box.OverflowClipRectForScrollNode();
            let (horizontal, vertical) = if layout_box.IsLayoutView() {
                use crate::internal::scrollbar_mode::mojom::blink::ScrollbarMode;
                let view = unsafe { &*To::<LayoutView>(box_fragment.GetLayoutObject()) };
                let mut horizontal = ScrollbarMode::kAuto;
                let mut vertical = ScrollbarMode::kAuto;
                // This standalone LayoutView uses the HTML owner's scroll
                // offset even when visible overflow has no local CSS area.
                view.CalculateScrollbarModes(&mut horizontal, &mut vertical, None, None);
                (
                    horizontal != ScrollbarMode::kAlwaysOff,
                    vertical != ScrollbarMode::kAlwaysOff,
                )
            } else {
                (layout_box.ScrollsOverflowX(), layout_box.ScrollsOverflowY())
            };
            paint.scroll_container = Some(ScrollContainerPaintData {
                container_rect: FragmentPaintRect {
                    offset: Offset {
                        x: Number(clip.offset.left),
                        y: Number(clip.offset.top),
                    },
                    size: Size {
                        width: Number(clip.size.width),
                        height: Number(clip.size.height),
                    },
                },
                user_scrollable_horizontal: horizontal,
                user_scrollable_vertical: vertical,
            });
        }
        let replaced = DynamicTo::<LayoutReplaced>(box_fragment.GetLayoutObject());
        if !replaced.is_null() {
            let rect = unsafe { &*replaced }.ReplacedContentRect();
            paint.replaced_content = Some(ReplacedContentPaintData {
                offset: Offset {
                    x: Number(rect.offset.left),
                    y: Number(rect.offset.top),
                },
                size: Size {
                    width: Number(rect.size.width),
                    height: Number(rect.size.height),
                },
            });
        }
    }
    let borders = box_fragment.Borders();
    paint.border = Edges {
        top: Number(borders.top),
        right: Number(borders.right),
        bottom: Number(borders.bottom),
        left: Number(borders.left),
    };
    let sides = box_fragment.SidesToInclude();
    paint.border_sides = FragmentBoxSides {
        top: sides.top,
        right: sides.right,
        bottom: sides.bottom,
        left: sides.left,
    };
    let padding = box_fragment.Padding();
    paint.padding = Edges {
        top: Number(padding.top),
        right: Number(padding.right),
        bottom: Number(padding.bottom),
        left: Number(padding.left),
    };
    paint.box_decoration_break =
        if box_fragment.Style().BoxDecorationBreak() == EBoxDecorationBreak::kClone {
            BoxDecorationBreak::kClone
        } else {
            BoxDecorationBreak::kSlice
        };
    paint.stitched_decoration = ExportStitchedDecorationData(box_fragment);
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1138-1278
fn ExportScrollbarPaintData(box_fragment: &PhysicalBoxFragment, paint: &mut PaintProperties) {
    let layout_box = DynamicTo::<LayoutBox>(box_fragment.GetLayoutObject());
    if layout_box.is_null() {
        return;
    }
    let layout_box = unsafe { &*layout_box };
    let area = layout_box.GetScrollableArea();
    if area.is_null()
        || (!unsafe { &*area }.HasHorizontalScrollbar()
            && !unsafe { &*area }.HasVerticalScrollbar())
    {
        return;
    }
    let area = unsafe { &*area };
    let theme = layout_box.ScrollbarThemeForLayout();
    let themed_thickness =
        ScrollbarThemeThickness(theme, box_fragment.Style().UsedScrollbarWidth()) as f64;
    let borders = box_fragment.Borders();
    let reserved = box_fragment.Scrollbar();
    let border_top = Number(borders.top);
    let border_right = Number(borders.right);
    let border_bottom = Number(borders.bottom);
    let border_left = Number(borders.left);
    let vertical_on_left =
        Number(reserved.left) > 0.0 || layout_box.ShouldPlaceVerticalScrollbarOnLeft();
    let vertical_thickness = if area.HasVerticalScrollbar() {
        if theme.uses_overlay_scrollbars {
            themed_thickness
        } else {
            Number(reserved.left).max(Number(reserved.right))
        }
    } else {
        0.0
    };
    let horizontal_thickness = if area.HasHorizontalScrollbar() {
        if theme.uses_overlay_scrollbars {
            themed_thickness
        } else {
            Number(reserved.bottom)
        }
    } else {
        0.0
    };
    if (area.HasVerticalScrollbar() && vertical_thickness <= 0.0)
        || (area.HasHorizontalScrollbar() && horizontal_thickness <= 0.0)
    {
        return;
    }

    let corner_client = area.ScrollCornerDisplayItemClient();
    let mut data = ScrollbarPaintData {
        corner_client_id: corner_client.map_or(0, |client| client.Id() as u64),
        corner_client_is_cacheable: corner_client.is_some_and(|client| client.IsCacheable()),
        corner_client_is_just_created: corner_client.is_some_and(|client| client.IsJustCreated()),
        uses_overlay_scrollbars: theme.uses_overlay_scrollbars,
        track_color: theme.track_color,
        thumb_color: theme.thumb_color,
        button_color: theme.button_color,
        corner_color: theme.corner_color,
        ..ScrollbarPaintData::default()
    };
    let visible = area.VisibleContentRect(IncludeScrollbarsInRect::kExcludeScrollbars);
    let contents = area.ContentsSize();
    let minimum = area.MinimumScrollOffsetInt();
    let maximum = area.MaximumScrollOffsetInt();
    let current = area.GetScrollOffset();
    let corner_width = if !theme.uses_overlay_scrollbars && area.HasVerticalScrollbar() {
        vertical_thickness
    } else {
        0.0
    };
    let corner_height = if !theme.uses_overlay_scrollbars && area.HasHorizontalScrollbar() {
        horizontal_thickness
    } else {
        0.0
    };

    if area.HasHorizontalScrollbar() {
        let scrollbar = area.HorizontalScrollbar();
        let client = (!scrollbar.is_null()).then(|| unsafe { &*scrollbar }.DisplayItemClient());
        let mut axis = ScrollbarPaintAxis {
            display_item_client_id: client.map_or(0, |client| client.Id() as u64),
            display_item_client_is_cacheable: client.is_some_and(|client| client.IsCacheable()),
            display_item_client_is_just_created: client
                .is_some_and(|client| client.IsJustCreated()),
            horizontal: true,
            track_offset: Offset {
                x: border_left
                    + if !theme.uses_overlay_scrollbars && vertical_on_left {
                        corner_width
                    } else {
                        0.0
                    },
                y: Number(box_fragment.Size().height) - border_bottom - horizontal_thickness,
            },
            track_size: Size {
                width: (Number(box_fragment.Size().width)
                    - border_left
                    - border_right
                    - corner_width)
                    .max(0.0),
                height: horizontal_thickness,
            },
            ..ScrollbarPaintAxis::default()
        };
        SetScrollbarThumb(
            &mut axis,
            visible.width() as f64,
            contents.width() as f64,
            current.x() as f64,
            minimum.x() as f64,
            maximum.x() as f64,
            theme.has_buttons,
            theme.minimum_thumb_length as f64,
        );
        data.horizontal = Some(axis);
    }
    if area.HasVerticalScrollbar() {
        let scrollbar = area.VerticalScrollbar();
        let client = (!scrollbar.is_null()).then(|| unsafe { &*scrollbar }.DisplayItemClient());
        let mut axis = ScrollbarPaintAxis {
            display_item_client_id: client.map_or(0, |client| client.Id() as u64),
            display_item_client_is_cacheable: client.is_some_and(|client| client.IsCacheable()),
            display_item_client_is_just_created: client
                .is_some_and(|client| client.IsJustCreated()),
            horizontal: false,
            track_offset: Offset {
                x: if vertical_on_left {
                    border_left
                } else {
                    Number(box_fragment.Size().width) - border_right - vertical_thickness
                },
                y: border_top,
            },
            track_size: Size {
                width: vertical_thickness,
                height: (Number(box_fragment.Size().height)
                    - border_top
                    - border_bottom
                    - corner_height)
                    .max(0.0),
            },
            ..ScrollbarPaintAxis::default()
        };
        SetScrollbarThumb(
            &mut axis,
            visible.height() as f64,
            contents.height() as f64,
            current.y() as f64,
            minimum.y() as f64,
            maximum.y() as f64,
            theme.has_buttons,
            theme.minimum_thumb_length as f64,
        );
        data.vertical = Some(axis);
    }
    if corner_width > 0.0 && corner_height > 0.0 {
        data.corner_offset = Offset {
            x: if vertical_on_left {
                border_left
            } else {
                Number(box_fragment.Size().width) - border_right - corner_width
            },
            y: Number(box_fragment.Size().height) - border_bottom - corner_height,
        };
        data.corner_size = Size {
            width: corner_width,
            height: corner_height,
        };
    }
    paint.scrollbars = Some(Arc::new(data));
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1280-1315
fn ComputeTableColumnsRect(fragment: &PhysicalBoxFragment) -> PhysicalRect {
    let writing_direction = fragment.Style().GetWritingDirection();
    let mut columns_rect = LogicalRect::default();
    let mut stitched_block_size = LayoutUnit::default();
    let mut fragment_block_offset = LayoutUnit::default();
    let mut has_section = false;
    let layout_box = To::<LayoutBox>(fragment.GetLayoutObject());
    for table_fragment in unsafe { &*layout_box }.PhysicalFragments() {
        if std::ptr::eq(table_fragment, fragment) {
            fragment_block_offset = stitched_block_size;
        }
        let converter = WritingModeConverter::new(writing_direction, table_fragment.Size());
        for child in table_fragment.Children() {
            if !child.IsTableSection() {
                continue;
            }
            let mut section_rect =
                converter.ToLogicalRect(PhysicalRect::new(child.Offset(), child.Size()));
            section_rect.offset.block_offset += stitched_block_size;
            if !has_section {
                columns_rect = section_rect;
                has_section = true;
            } else {
                columns_rect.UniteEvenIfEmpty(&section_rect);
            }
        }
        stitched_block_size += LogicalFragment::new(writing_direction, table_fragment).BlockSize();
    }
    if !has_section {
        return PhysicalRect::default();
    }
    columns_rect.offset.block_offset -= fragment_block_offset;
    WritingModeConverter::new(writing_direction, fragment.Size()).ToPhysicalRect(columns_rect)
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1317-1353
fn ExportTablePaintData(
    box_fragment: &PhysicalBoxFragment,
    sources: &SourceMap,
    paint: &mut PaintProperties,
) {
    if !box_fragment.IsTable() {
        return;
    }
    let mut data = TablePaintData::default();
    let converter = WritingModeConverter::new(
        box_fragment.Style().GetWritingDirection(),
        box_fragment.Size(),
    );
    let grid = converter.ToPhysicalRect(box_fragment.TableGridRect());
    data.grid_offset = Offset {
        x: Number(grid.offset.left),
        y: Number(grid.offset.top),
    };
    data.grid_size = Size {
        width: Number(grid.size.width),
        height: Number(grid.size.height),
    };
    let columns = ComputeTableColumnsRect(box_fragment);
    data.columns_offset = Offset {
        x: Number(columns.offset.left),
        y: Number(columns.offset.top),
    };
    data.columns_size = Size {
        width: Number(columns.size.width),
        height: Number(columns.size.height),
    };
    let geometries = box_fragment.TableColumnGeometries();
    if !geometries.is_null() {
        let geometries = unsafe { &*geometries };
        data.columns.reserve(geometries.len());
        for geometry in geometries.iter() {
            let column_box = geometry.node.GetLayoutBox();
            if column_box.is_null() {
                continue;
            }
            let Some(source) = sources.get(&(column_box as *const LayoutObject)) else {
                continue;
            };
            data.columns.push(TablePaintColumn {
                start_column: geometry.start_column as usize,
                span: geometry.span as usize,
                inline_offset: Number(geometry.inline_offset),
                inline_size: Number(geometry.inline_size),
                node_id: unsafe { &*source.node }.InputId(),
                style: unsafe { &*source.node }.InputStyle().paint.clone(),
            });
        }
    }
    paint.table = Some(Arc::new(data));
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1355-1412
fn ExportMathMLPaintData(box_fragment: &PhysicalBoxFragment, paint: &mut PaintProperties) {
    if let Some(baseline) = box_fragment.FirstBaseline() {
        paint.first_baseline = Some(Number(baseline));
    }
    if !box_fragment.HasExtraMathMLPainting() {
        return;
    }

    let mut data = MathMLPaintData::default();
    let algorithms = LayoutPassScope::Algorithms();
    let math = if algorithms.is_null() {
        None
    } else {
        Some(&unsafe { &*algorithms }.mathml_support)
    };
    if box_fragment.IsMathMLFraction() {
        let Some((axis_height, fraction_rule_thickness)) =
            math.and_then(|support| Some((support.axis_height?, support.fraction_rule_thickness?)))
        else {
            std::panic::panic_any(UnsupportedLayout::new(
                "MathML fraction paint module is not installed",
            ));
        };
        data.kind = MathMLPaintKind::kFraction;
        data.axis_height = Number(axis_height(box_fragment.Style()));
        data.rule_thickness = Number(fraction_rule_thickness(box_fragment.Style()));
        paint.mathml = Some(Arc::new(data));
        return;
    }

    let native = box_fragment.GetMathMLPaintInfo();
    data.kind = if native.IsRadicalOperator() {
        MathMLPaintKind::kRadical
    } else {
        MathMLPaintKind::kOperator
    };
    data.operator_inline_size = Number(native.operator_inline_size);
    data.operator_ascent = Number(native.operator_ascent);
    data.operator_descent = Number(native.operator_descent);
    data.radical_margin_inline_start = Number(native.radical_base_margins.inline_start);
    data.radical_margin_inline_end = Number(native.radical_base_margins.inline_end);
    data.radical_operator_inline_offset = native.radical_operator_inline_offset.map_or(0.0, Number);
    if data.kind == MathMLPaintKind::kRadical {
        let Some(radical_metrics) = math.and_then(|support| support.radical_metrics) else {
            std::panic::panic_any(UnsupportedLayout::new(
                "MathML radical paint module is not installed",
            ));
        };
        let has_index = paint.source_kind == NodeKind::kMathRoot;
        let mut vertical_gap = LayoutUnit::default();
        let mut rule_thickness = LayoutUnit::default();
        radical_metrics(
            box_fragment.Style(),
            has_index,
            &mut vertical_gap,
            &mut rule_thickness,
        );
        data.vertical_gap = Number(vertical_gap);
        data.rule_thickness = Number(rule_thickness);
    }
    let font = box_fragment.Style().GetFont();
    let primary_font = unsafe { &*font }.PrimaryFont();
    let baseline = if primary_font.is_null() {
        0.0
    } else {
        unsafe { &*primary_font }.GetFontMetrics().Ascent() as f64
    };
    ExportShapeResult(
        native.operator_shape_result_view.Get(),
        box_fragment.Style().ComputedFontSize() as f64,
        baseline,
        paint.writing_mode,
        &mut data.operator_glyph_runs,
    );
    paint.mathml = Some(Arc::new(data));
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1414-1437
fn ExportFrameSetPaintData(box_fragment: &PhysicalBoxFragment, paint: &mut PaintProperties) {
    if paint.source_kind != NodeKind::kFrameSet {
        return;
    }
    let native = box_fragment.GetFrameSetLayoutData();
    if native.is_null() {
        return;
    }
    let native = unsafe { &*native };
    let mut data = FrameSetPaintData::default();
    data.column_sizes.reserve(native.col_sizes.len());
    for size in &native.col_sizes {
        data.column_sizes.push(Number(*size));
    }
    data.row_sizes.reserve(native.row_sizes.len());
    for size in &native.row_sizes {
        data.row_sizes.push(Number(*size));
    }
    data.column_allows_border
        .extend(native.col_allow_border.iter().copied());
    data.row_allows_border
        .extend(native.row_allow_border.iter().copied());
    data.border_thickness = native.border_thickness as f64;
    data.has_border_color = native.has_border_color;
    paint.frame_set = Some(Arc::new(data));
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1439-1510
fn ExportFieldsetPaintData(box_fragment: &PhysicalBoxFragment, paint: &mut PaintProperties) {
    if paint.source_kind != NodeKind::kFieldset {
        return;
    }
    let mut data = FieldsetPaintData::default();
    let mut legend_border_box = PhysicalRect::default();
    if let Some(legend) = box_fragment.Children().first() {
        if legend.IsRenderedLegend() {
            data.has_legend = true;
            legend_border_box.size = legend.Size();
            let writing_direction = box_fragment.Style().GetWritingDirection();
            let borders = box_fragment.Borders();
            let padding = box_fragment.Padding();
            let physical_content_size = box_fragment.Size()
                - PhysicalSize::new(
                    borders.HorizontalSum() + padding.HorizontalSum(),
                    borders.VerticalSum() + padding.VerticalSum(),
                );
            let logical_content_size =
                ToLogicalSize(physical_content_size, writing_direction.GetWritingMode());
            let relative_offset =
                ComputeRelativeOffset(legend.Style(), writing_direction, &logical_content_size);
            let logical_offset = WritingModeConverter::new(writing_direction, box_fragment.Size())
                .ToLogicalOffset(legend.Offset(), legend.Size())
                - relative_offset;
            legend_border_box.offset =
                layoutng_geometry::geometry::logical_offset::LogicalOffset::from(logical_offset)
                    .ConvertToPhysical(
                        writing_direction,
                        box_fragment.Size(),
                        legend_border_box.size,
                    );
        }
    }

    let borders = box_fragment.Borders();
    let mut cutout;
    let mut outsets = layoutng_geometry::geometry::box_strut::PhysicalBoxStrut::default();
    if box_fragment.Style().IsHorizontalWritingMode() {
        let legend_size = legend_border_box.size.height;
        let border_size = borders.top;
        let excess = legend_size - border_size;
        if excess > LayoutUnit::default() {
            outsets.top = excess / 2;
        }
        cutout = PhysicalRect::from_units(
            legend_border_box.X(),
            LayoutUnit::default(),
            legend_border_box.Width(),
            legend_size.max(border_size),
        );
    } else {
        let legend_size = legend_border_box.Width();
        let flipped = box_fragment.Style().IsFlippedBlocksWritingMode();
        let border_size = if flipped { borders.right } else { borders.left };
        let excess = legend_size - border_size;
        if excess > LayoutUnit::default() {
            if flipped {
                outsets.right = excess / 2;
            } else {
                outsets.left = excess / 2;
            }
        }
        let total = legend_size.max(border_size);
        cutout = PhysicalRect::from_units(
            LayoutUnit::default(),
            legend_border_box.offset.top,
            total,
            legend_border_box.size.height,
        );
        if flipped {
            cutout.offset.left += box_fragment.Size().width - total;
        }
    }
    data.border_outsets = Edges {
        top: Number(outsets.top),
        right: Number(outsets.right),
        bottom: Number(outsets.bottom),
        left: Number(outsets.left),
    };
    data.legend_cutout_offset = Offset {
        x: Number(cutout.offset.left),
        y: Number(cutout.offset.top),
    };
    data.legend_cutout_size = Size {
        width: Number(cutout.size.width),
        height: Number(cutout.size.height),
    };
    paint.fieldset = Some(Arc::new(data));
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1512-1551
fn ExportColumnRulePaintData(box_fragment: &PhysicalBoxFragment, paint: &mut PaintProperties) {
    let style = &paint.style;
    if style.column_rule_width <= 0.0
        || style.column_rule_style == BorderLineStyle::kNone
        || style.column_rule_color.alpha <= 0.0
    {
        return;
    }
    let geometry = box_fragment.GetGapGeometry();
    if geometry.is_null() || unsafe { &*geometry }.GetContainerType() != ContainerType::kMultiColumn
    {
        return;
    }
    let geometry = unsafe { &*geometry };
    let mut data = ColumnRulePaintData::default();
    let converter = WritingModeConverter::new(
        box_fragment.Style().GetWritingDirection(),
        box_fragment.Size(),
    );
    let block_start = geometry.GetContentBlockStart();
    let block_size = geometry.GetContentBlockEnd() - block_start;
    let thickness = LayoutUnit::from_f64(style.column_rule_width);
    let columns_are_main = geometry.IsMainDirection(GridTrackSizingDirection::kForColumns);
    let gap_count = if columns_are_main {
        geometry.MainGapCount()
    } else {
        geometry.CrossGapCount()
    };
    for index in 0..gap_count {
        if geometry.IsMultiColSpanner(index, GridTrackSizingDirection::kForColumns) {
            continue;
        }
        let center = geometry.GetGapCenterOffset(GridTrackSizingDirection::kForColumns, index);
        let logical =
            LogicalRect::from_units(center - thickness / 2, block_start, thickness, block_size);
        let physical = converter.ToPhysicalRect(logical);
        data.rules.push(ColumnRule {
            offset: Offset {
                x: Number(physical.offset.left),
                y: Number(physical.offset.top),
            },
            size: Size {
                width: Number(physical.size.width),
                height: Number(physical.size.height),
            },
        });
    }
    if !data.rules.is_empty() {
        paint.column_rules = Some(Arc::new(data));
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1553-1562
fn ExportOverflowClipMargin(box_fragment: &PhysicalBoxFragment, paint: &mut PaintProperties) {
    if !box_fragment.ShouldApplyOverflowClipMargin() {
        return;
    }
    let outsets = box_fragment.OverflowClipMarginOutsets();
    paint.overflow_clip_margin_outsets = Some(Edges {
        top: Number(outsets.top),
        right: Number(outsets.right),
        bottom: Number(outsets.bottom),
        left: Number(outsets.left),
    });
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1564-1576
fn ExportBorderStyle(style: EBorderStyle) -> BorderLineStyle {
    match style {
        EBorderStyle::kHidden => BorderLineStyle::kHidden,
        EBorderStyle::kDotted => BorderLineStyle::kDotted,
        EBorderStyle::kDashed => BorderLineStyle::kDashed,
        EBorderStyle::kSolid => BorderLineStyle::kSolid,
        EBorderStyle::kDouble => BorderLineStyle::kDouble,
        EBorderStyle::kGroove => BorderLineStyle::kGroove,
        EBorderStyle::kRidge => BorderLineStyle::kRidge,
        EBorderStyle::kInset => BorderLineStyle::kInset,
        EBorderStyle::kOutset => BorderLineStyle::kOutset,
        _ => BorderLineStyle::kNone,
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1578-1588
fn BorderSideIndex(side: EdgeSide) -> usize {
    match side {
        EdgeSide::kTop | EdgeSide::kDoNotFill => 0,
        EdgeSide::kRight => 1,
        EdgeSide::kBottom => 2,
        EdgeSide::kLeft => 3,
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1590-1598
fn IsStartTableRowFragmented(section: &PhysicalBoxFragment) -> bool {
    for child in section.Children() {
        if child.IsTableRow() {
            let row = To::<PhysicalBoxFragment>(child.get());
            return IsBreakInside(FindPreviousBreakToken(unsafe { &*row }));
        }
    }
    false
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1600-1610
fn IsEndTableRowFragmented(section: &PhysicalBoxFragment) -> bool {
    for child in section.Children().iter().rev() {
        if child.IsTableRow() {
            let token = To::<BlockBreakToken>(child.GetBreakToken());
            return IsBreakInside(token) && !unsafe { &*token }.IsAtBlockEnd();
        }
    }
    false
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1612-1677
fn ExportCollapsedTablePaintData(
    box_fragment: &PhysicalBoxFragment,
    style_sources: &StyleSourceMap,
    paint: &mut PaintProperties,
) {
    if !box_fragment.IsTable() {
        return;
    }
    let borders = box_fragment.TableCollapsedBorders();
    if borders.is_null() || !unsafe { &*borders }.IsCollapsed() {
        return;
    }
    let borders = unsafe { &*borders };
    let geometry = box_fragment.TableCollapsedBordersGeometry();
    if geometry.is_null() {
        return;
    }
    let geometry = unsafe { &*geometry };

    let mut data = CollapsedTablePaintData {
        edges_per_row: borders.EdgesPerRow() as usize,
        ..CollapsedTablePaintData::default()
    };
    data.edges.reserve(borders.EdgeCount() as usize);
    for (edge_index, native) in borders.edges_.iter().enumerate() {
        let mut edge = CollapsedTableEdge {
            exists: !native.style.Get().is_null(),
            box_order: native.box_order as usize,
            ..CollapsedTableEdge::default()
        };
        if edge.exists {
            let native_style = borders.BorderStyle(edge_index as u32);
            edge.can_paint = borders.CanPaint(edge_index as u32);
            edge.width = Number(borders.BorderWidth(edge_index as u32));
            edge.style = ExportBorderStyle(native_style);
            edge.style_rank = native_style as u8;
            if native.edge_side != EdgeSide::kDoNotFill {
                if let Some(&source) =
                    style_sources.get(&(native.style.Get() as *const NativeComputedStyle))
                {
                    edge.color =
                        unsafe { &*source }.paint.border_colors[BorderSideIndex(native.edge_side)];
                }
            }
        }
        data.edges.push(edge);
    }
    data.columns.reserve(geometry.columns.len());
    for column in &geometry.columns {
        data.columns.push(Number(*column));
    }

    for child in box_fragment.Children() {
        if !child.IsTableSection() {
            continue;
        }
        let section = To::<PhysicalBoxFragment>(child.get());
        let section = unsafe { &*section };
        let Some(start_row) = section.TableSectionStartRowIndex() else {
            continue;
        };
        let Some(row_offsets) = section.TableSectionRowOffsets() else {
            continue;
        };
        let mut output = CollapsedTableSection {
            offset: Offset {
                x: Number(child.Offset().left),
                y: Number(child.Offset().top),
            },
            size: Size {
                width: Number(section.Size().width),
                height: Number(section.Size().height),
            },
            start_row: start_row as usize,
            ..CollapsedTableSection::default()
        };
        output.row_offsets.reserve(row_offsets.len());
        for row_offset in row_offsets {
            output.row_offsets.push(Number(*row_offset));
        }
        output.start_row_fragmented = IsStartTableRowFragmented(section);
        output.end_row_fragmented = IsEndTableRowFragmented(section);
        data.sections.push(output);
    }
    paint.collapsed_table = Some(Arc::new(data));
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1704-1734
fn AppendSVGRootChildren(
    object: *const LayoutObject,
    output: &mut Vec<FragmentNode>,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    paint_state_owners: &mut HashSet<*const PhysicalBoxFragment>,
    fragmentainer_instance_id: u64,
    display_item_fragment: u32,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) {
    if object.is_null() || !unsafe { &*object }.IsSVGRoot() {
        return;
    }
    // SVG children are owned by child LayoutBoxes, not linked into the root
    // replaced box's PhysicalBoxFragment::Children().
    let mut child = unsafe { &*object }.SlowFirstChild();
    while !child.is_null() {
        let child_box = DynamicTo::<LayoutBox>(child);
        if !child_box.is_null() {
            let child_box = unsafe { &*child_box };
            let child_offset = child_box.PhysicalLocation();
            for child_fragment in child_box.PhysicalFragments() {
                output.push(ExportFragment(
                    child_fragment,
                    child_offset,
                    sources,
                    style_sources,
                    resources,
                    paint_state_owners,
                    fragmentainer_instance_id,
                    display_item_fragment,
                    next_fragment_instance_id,
                    next_fragmentainer_instance_id,
                ));
            }
        }
        child = unsafe { &*child }.NextSibling();
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1736-1848
fn ExportNativeInkRect(rect: PhysicalRect) -> FragmentPaintRect {
    FragmentPaintRect {
        offset: Offset {
            x: Number(rect.offset.left),
            y: Number(rect.offset.top),
        },
        size: Size {
            width: Number(rect.size.width),
            height: Number(rect.size.height),
        },
    }
}

fn ExportBoxInkOverflow(fragment: &PhysicalBoxFragment, paint: &mut PaintProperties) {
    paint.box_fragment_is_inline_box = fragment.IsInlineBox();
    if fragment.IsInkOverflowComputed() {
        paint.self_ink_overflow = Some(ExportNativeInkRect(fragment.SelfInkOverflowRect()));
        paint.contents_ink_overflow = Some(ExportNativeInkRect(fragment.ContentsInkOverflowRect()));
        paint.ink_overflow = Some(ExportNativeInkRect(fragment.InkOverflowRect()));
    }
    let object = fragment.GetLayoutObject();
    if object.is_null() {
        return;
    }
    let layout_box = DynamicTo::<LayoutBox>(object);
    if !layout_box.is_null() {
        // BoxPainter::VisualRect uses the stitched LayoutBox self rect.
        // The native overflow lifecycle is not yet complete. Its unset
        // border-box default is valid for plain boxes, but cannot be exported
        // as completed shadow/outline overflow.
        let layout_box = unsafe { &*layout_box };
        let plain = paint.style.box_shadows.is_empty()
            && paint.style.outline_width == 0.0
            && paint.style.border_image.is_none()
            && !fragment.IsTableRow();
        if layout_box.VisualOverflowIsSet() || (plain && !fragment.IsInlineBox()) {
            paint.box_self_visual_overflow =
                Some(ExportNativeInkRect(layout_box.SelfVisualOverflowRect()));
        }
    }
}

fn ExportInlineInkOverflow(item: &FragmentItem, paint: &mut PaintProperties) {
    let box_fragment = item.BoxFragment();
    if !box_fragment.is_null() {
        ExportBoxInkOverflow(unsafe { &*box_fragment }, paint);
        return;
    }
    if item.IsInkOverflowComputed() {
        paint.self_ink_overflow = Some(ExportNativeInkRect(item.SelfInkOverflowRect()));
        paint.ink_overflow = Some(ExportNativeInkRect(item.InkOverflowRect()));
        return;
    }
    if !item.IsText() || item.IsSvgText() || item.IsSymbolMarker() {
        return;
    }
    let style = item.Style();
    let decoration = &paint.style.text_decoration;
    // Only the implemented plain-text branch is computed here. Decoration
    // boxes, emphasis, shadows and selection/marker overflow require the
    // original InlinePaintContext lifecycle; do not mark those as computed.
    let plain = !style.HasAppliedTextDecorations()
        && !(decoration.underline || decoration.overline || decoration.line_through)
        && style.TextShadow().is_null()
        && paint.style.text_shadows.is_empty()
        && style.GetTextEmphasisMark() == foundation::TextEmphasisMark::kNone;
    // ink_overflow.cc:439-489 / SetTextInkOverflow:361-369. Glyph bounds are
    // logical baseline coordinates, not a font ascent/descent approximation.
    let size = item.Size();
    let mut rect = PhysicalRect::new(PhysicalOffset::default(), size);
    let shape = item.TextShapeResult();
    if !shape.is_null() {
        let font = item.GetUsedFont();
        let mut bounds = if font.GetFont().ShouldSkipDrawing() {
            gfx::RectF::default()
        } else {
            unsafe { &*shape }.ComputeInkBounds()
        };
        bounds.Scale(font.ScalingFactor(), font.ScalingFactor());
        let mut logical = LogicalRect::EnclosingRect(&bounds);
        logical.offset.block_offset += font.FixedAscent();
        let stroke_width = style.TextStrokeWidth();
        if stroke_width != 0.0 {
            logical.Inflate(LayoutUnit::FromFloatCeil(stroke_width / 2.0));
        }
        let converter = WritingModeConverter::new(
            foundation::WritingDirectionMode::new(
                item.GetWritingMode(),
                foundation::TextDirection::kLtr,
            ),
            size,
        );
        let glyph_rect = converter.ToPhysicalRect(logical);
        paint.text_glyph_ink = Some(ExportNativeInkRect(glyph_rect));
        if !glyph_rect.IsEmpty() && !rect.ContainsRect(&glyph_rect) {
            rect.Unite(&glyph_rect);
            rect.ExpandEdgesToPixelBoundaries();
        }
    }
    if plain {
        paint.self_ink_overflow = Some(ExportNativeInkRect(rect));
        paint.ink_overflow = paint.self_ink_overflow;
    }
}

fn ExportInlineItem(
    item: &FragmentItem,
    fragment_items: &layoutng_fragment_tree::fragment_items::FragmentItems,
    parent_offset: PhysicalOffset,
    containing_line_offset: Option<PhysicalOffset>,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    paint_state_owners: &mut HashSet<*const PhysicalBoxFragment>,
    fragmentainer_instance_id: u64,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) -> FragmentNode {
    let mut output = FragmentNode::default();
    output.fragment_instance_id = *next_fragment_instance_id;
    *next_fragment_instance_id += 1;
    output.fragmentainer_instance_id = fragmentainer_instance_id;
    let object = item.GetLayoutObject();
    output.node_id = InputId(object);
    let effective_style = if item.UsesFirstLineStyle() {
        item.Style() as *const NativeComputedStyle
    } else {
        std::ptr::null()
    };
    output.paint =
        ExportPaintProperties(object, sources, style_sources, effective_style, resources);
    // cpp: core/paint/box_fragment_painter.cc:2025-2026,2071-2072
    // cpp: core/paint/inline_box_fragment_painter.cc:63-64
    output.paint.display_item_fragment = item.FragmentId();
    output.paint.hidden = item.IsHiddenForPaint();
    let rect = *item.RectInContainerFragment();
    output.offset = Offset {
        x: Number(rect.offset.left - parent_offset.left),
        y: Number(rect.offset.top - parent_offset.top),
    };
    output.size = Size {
        width: Number(rect.size.width),
        height: Number(rect.size.height),
    };
    output.content_size = output.size;
    ExportInlineInkOverflow(item, &mut output.paint);
    ExportSvgTextPaintData(item, rect, &mut output.paint);
    match item.Type() {
        ItemType::kText | ItemType::kGeneratedText => {
            output.kind = if item.Type() == ItemType::kText {
                FragmentKind::kText
            } else {
                FragmentKind::kGeneratedText
            };
            if let Some(line_offset) = containing_line_offset {
                output.paint.text_line_top_offset = Some(Number(line_offset.top - rect.offset.top));
            }
            output.text_start = Some(item.StartOffset());
            output.text_end = Some(item.EndOffset());
            ExportListMarkerSymbol(item, &mut output.paint);
            ExportGlyphRuns(item, &mut output.paint);
            if output.paint.text_control_host.is_some() {
                let text = item.Text(fragment_items);
                let view = item.TextShapeResult();
                let shape = (!view.is_null()).then(|| unsafe { &*view }.CreateShapeResult());
                for offset in item.StartOffset()..=item.EndOffset() {
                    let position = if let Some(shape) = shape {
                        use font_engine::fonts::shaping::shape_result_types::AdjustMidCluster;
                        LayoutUnit::FromFloatRound(
                            unsafe { &*shape }.CaretPositionForOffset(
                                offset - item.StartOffset(),
                                &text,
                                AdjustMidCluster::kToEnd,
                            ) * item.GetTextFitScale(),
                        )
                    } else {
                        item.CaretInlinePositionForOffset(text.clone(), offset)
                    };
                    output
                        .paint
                        .text_caret_positions
                        .push((offset, Number(position)));
                }
            }
        }
        ItemType::kLine => {
            output.kind = FragmentKind::kLine;
            output.node_id = 0;
        }
        ItemType::kBox => {
            output.kind = FragmentKind::kBox;
            let box_fragment = item.BoxFragment();
            if !box_fragment.is_null() {
                let box_fragment = unsafe { &*box_fragment };
                ExportBoxPaintGeometry(box_fragment, &mut output.paint);
                ExportOverflowClipMargin(box_fragment, &mut output.paint);
                ExportScrollbarPaintData(box_fragment, &mut output.paint);
                ExportTablePaintData(box_fragment, sources, &mut output.paint);
                if box_fragment.IsTableCell() {
                    output.paint.table_cell_column =
                        Some(box_fragment.TableCellColumnIndex() as usize);
                }
                output.paint.painted_atomically = box_fragment.IsPaintedAtomically();
                output.paint.has_collapsed_borders = box_fragment.HasCollapsedBorders();
                let owner_fragment = item.PostLayoutBoxFragment();
                let owner_fragment = if owner_fragment.is_null() {
                    box_fragment as *const PhysicalBoxFragment
                } else {
                    owner_fragment
                };
                output.paint.establishes_paint_state = paint_state_owners.insert(owner_fragment);
                let content = box_fragment.ContentRect();
                output.content_size = Size {
                    width: Number(content.size.width),
                    height: Number(content.size.height),
                };
                ExportMathMLPaintData(box_fragment, &mut output.paint);
                ExportFrameSetPaintData(box_fragment, &mut output.paint);
                ExportFieldsetPaintData(box_fragment, &mut output.paint);
                ExportColumnRulePaintData(box_fragment, &mut output.paint);
                if item.DescendantsCount() <= 1 {
                    let nested_items = box_fragment.Items();
                    for child in box_fragment.Children() {
                        if !child.is_present() || (!nested_items.is_null() && child.IsLineBox()) {
                            continue;
                        }
                        output.children.push(ExportFragment(
                            child,
                            child.Offset(),
                            sources,
                            style_sources,
                            resources,
                            paint_state_owners,
                            fragmentainer_instance_id,
                            item.FragmentId(),
                            next_fragment_instance_id,
                            next_fragmentainer_instance_id,
                        ));
                    }
                    if !nested_items.is_null() {
                        let span = unsafe { &*nested_items }.Items();
                        let mut index = 0;
                        while index < span.len() {
                            index = AppendInlineItemTree(
                                span,
                                unsafe { &*nested_items },
                                index,
                                span.len(),
                                PhysicalOffset::default(),
                                None,
                                &mut output.children,
                                sources,
                                style_sources,
                                resources,
                                paint_state_owners,
                                fragmentainer_instance_id,
                                next_fragment_instance_id,
                                next_fragmentainer_instance_id,
                            );
                        }
                    }
                }
                AppendSVGRootChildren(
                    object,
                    &mut output.children,
                    sources,
                    style_sources,
                    resources,
                    paint_state_owners,
                    fragmentainer_instance_id,
                    item.FragmentId(),
                    next_fragment_instance_id,
                    next_fragmentainer_instance_id,
                );
            }
        }
        ItemType::kInvalid => unreachable!("invalid inline item"),
    }
    output
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1850-1889
fn AppendInlineItemTree(
    items: &[FragmentItem],
    fragment_items: &layoutng_fragment_tree::fragment_items::FragmentItems,
    index: usize,
    limit: usize,
    parent_offset: PhysicalOffset,
    mut containing_line_offset: Option<PhysicalOffset>,
    output: &mut Vec<FragmentNode>,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    paint_state_owners: &mut HashSet<*const PhysicalBoxFragment>,
    fragmentainer_instance_id: u64,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) -> usize {
    assert!(index < limit);
    let item = &items[index];
    if containing_line_offset.is_none() && item.Type() == ItemType::kLine {
        containing_line_offset = Some(*item.OffsetInContainerFragment());
    }
    let subtree_size = 1_usize.max(item.DescendantsCount() as usize);
    let subtree_end = limit.min(index + subtree_size);
    let mut node = ExportInlineItem(
        item,
        fragment_items,
        parent_offset,
        containing_line_offset,
        sources,
        style_sources,
        resources,
        paint_state_owners,
        fragmentainer_instance_id,
        next_fragment_instance_id,
        next_fragmentainer_instance_id,
    );
    let mut child_index = index + 1;
    while child_index < subtree_end {
        child_index = AppendInlineItemTree(
            items,
            fragment_items,
            child_index,
            subtree_end,
            *item.OffsetInContainerFragment(),
            containing_line_offset,
            &mut node.children,
            sources,
            style_sources,
            resources,
            paint_state_owners,
            fragmentainer_instance_id,
            next_fragment_instance_id,
            next_fragmentainer_instance_id,
        );
    }
    output.push(node);
    subtree_end
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1891-1967
fn ExportFragment(
    fragment: &PhysicalFragment,
    offset: PhysicalOffset,
    sources: &SourceMap,
    style_sources: &StyleSourceMap,
    resources: &Arc<PaintResources>,
    paint_state_owners: &mut HashSet<*const PhysicalBoxFragment>,
    mut fragmentainer_instance_id: u64,
    mut display_item_fragment: u32,
    next_fragment_instance_id: &mut u64,
    next_fragmentainer_instance_id: &mut u64,
) -> FragmentNode {
    let mut output = FragmentNode::default();
    output.fragment_instance_id = *next_fragment_instance_id;
    *next_fragment_instance_id += 1;
    let object = fragment.GetLayoutObject();
    output.node_id = InputId(object);
    let effective_style = if fragment.UsesFirstLineStyle() {
        fragment.Style() as *const NativeComputedStyle
    } else {
        std::ptr::null()
    };
    output.paint =
        ExportPaintProperties(object, sources, style_sources, effective_style, resources);
    output.paint.display_item_fragment = display_item_fragment;
    output.paint.hidden = fragment.IsHiddenForPaint();
    output.kind = if fragment.IsBox() {
        FragmentKind::kBox
    } else {
        FragmentKind::kLine
    };
    output.offset = Offset {
        x: Number(offset.left),
        y: Number(offset.top),
    };
    output.size = Size {
        width: Number(fragment.Size().width),
        height: Number(fragment.Size().height),
    };
    output.content_size = output.size;
    output.fragmentainer_instance_id = fragmentainer_instance_id;
    if !fragment.IsBox() {
        return output;
    }
    let box_fragment = To::<PhysicalBoxFragment>(fragment as *const PhysicalFragment);
    let box_fragment = unsafe { &*box_fragment };
    ExportBoxInkOverflow(box_fragment, &mut output.paint);
    if box_fragment.IsFragmentainerBox() {
        fragmentainer_instance_id = *next_fragmentainer_instance_id;
        *next_fragmentainer_instance_id += 1;
        // cpp: core/paint/box_fragment_painter.cc:398-403,1211-1212
        let token = box_fragment.GetBreakToken();
        display_item_fragment = if token.is_null() {
            0
        } else {
            unsafe { &*token }.SequenceNumber() + 1
        };
    } else if !object.is_null() && unsafe { &*object }.HasLayer() {
        // PaintLayerPainter scopes each physical fragment using its index in
        // the owning LayoutBox, independently of exported preorder numbering.
        // cpp: core/paint/paint_layer_painter.cc:776-778
        let layout_box = DynamicTo::<LayoutBox>(object);
        if !layout_box.is_null() {
            let layout_box = unsafe { &*layout_box };
            if let Some(index) = (0..layout_box.PhysicalFragmentCount())
                .find(|&index| layout_box.GetPhysicalFragment(index) == box_fragment as *const _)
            {
                display_item_fragment = index as u32;
            }
        }
    }
    output.paint.display_item_fragment = display_item_fragment;
    output.fragmentainer_instance_id = fragmentainer_instance_id;
    ExportBoxPaintGeometry(box_fragment, &mut output.paint);
    ExportOverflowClipMargin(box_fragment, &mut output.paint);
    ExportScrollbarPaintData(box_fragment, &mut output.paint);
    ExportTablePaintData(box_fragment, sources, &mut output.paint);
    if box_fragment.IsTableCell() {
        output.paint.table_cell_column = Some(box_fragment.TableCellColumnIndex() as usize);
    }
    output.paint.painted_atomically = fragment.IsPaintedAtomically();
    output.paint.has_collapsed_borders = fragment.HasCollapsedBorders();
    output.paint.establishes_paint_state =
        paint_state_owners.insert(box_fragment as *const PhysicalBoxFragment);
    let content = box_fragment.ContentRect();
    output.content_size = Size {
        width: Number(content.size.width),
        height: Number(content.size.height),
    };
    ExportMathMLPaintData(box_fragment, &mut output.paint);
    ExportFrameSetPaintData(box_fragment, &mut output.paint);
    ExportFieldsetPaintData(box_fragment, &mut output.paint);
    ExportColumnRulePaintData(box_fragment, &mut output.paint);
    ExportCollapsedTablePaintData(box_fragment, style_sources, &mut output.paint);
    output.children.reserve(box_fragment.Children().len());
    let items = box_fragment.Items();
    for child in box_fragment.Children() {
        if !child.is_present() || (!items.is_null() && child.IsLineBox()) {
            continue;
        }
        output.children.push(ExportFragment(
            child,
            child.Offset(),
            sources,
            style_sources,
            resources,
            paint_state_owners,
            fragmentainer_instance_id,
            display_item_fragment,
            next_fragment_instance_id,
            next_fragmentainer_instance_id,
        ));
    }
    if !items.is_null() {
        let span = unsafe { &*items }.Items();
        let mut index = 0;
        while index < span.len() {
            index = AppendInlineItemTree(
                span,
                unsafe { &*items },
                index,
                span.len(),
                PhysicalOffset::default(),
                None,
                &mut output.children,
                sources,
                style_sources,
                resources,
                paint_state_owners,
                fragmentainer_instance_id,
                next_fragment_instance_id,
                next_fragmentainer_instance_id,
            );
        }
    }
    AppendSVGRootChildren(
        object,
        &mut output.children,
        sources,
        style_sources,
        resources,
        paint_state_owners,
        fragmentainer_instance_id,
        display_item_fragment,
        next_fragment_instance_id,
        next_fragmentainer_instance_id,
    );
    output
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:714-726
fn StyleRequiresPaintBackedInlineFragment(style: &ComputedStyle) -> bool {
    let paint = &style.paint;
    paint.background_color.alpha > 0.0
        || !paint.background_images.is_empty()
        || paint.border_image.is_some()
        || !paint.box_shadows.is_empty()
        || (paint.outline_width > 0.0 && paint.outline_style != BorderLineStyle::kNone)
        || paint.opacity != 1.0
        || !paint.visible
        || paint.transform.is_some()
        || paint.clip_path.is_some()
        || !paint.filters.is_empty()
        || !paint.mask_images.is_empty()
        || paint.blend_mode != PaintBlendMode::kNormal
        || paint.isolate_blending
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:728-733
pub(crate) fn RequiresPaintBackedInlineFragment(source: &NativeNodeConstructionData) -> bool {
    StyleRequiresPaintBackedInlineFragment(&source.style)
        || source
            .first_line_style
            .as_ref()
            .is_some_and(StyleRequiresPaintBackedInlineFragment)
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1195-1233
fn SetScrollbarThumb(
    axis: &mut ScrollbarPaintAxis,
    visible_length: f64,
    content_length: f64,
    current_offset: f64,
    minimum_offset: f64,
    maximum_offset: f64,
    has_buttons: bool,
    minimum_thumb_length: f64,
) {
    let track_length = if axis.horizontal {
        axis.track_size.width
    } else {
        axis.track_size.height
    };
    let cross_size = if axis.horizontal {
        axis.track_size.height
    } else {
        axis.track_size.width
    };
    axis.has_buttons = has_buttons && track_length >= 2.0 * cross_size + minimum_thumb_length;
    let button_extent = if axis.has_buttons {
        2.0 * cross_size
    } else {
        0.0
    };
    let available = (track_length - button_extent).max(0.0);
    let ratio = if content_length > 0.0 {
        (visible_length / content_length).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let thumb_length = available.min(minimum_thumb_length.max(available * ratio));
    let range = maximum_offset - minimum_offset;
    let position = if range > 0.0 {
        ((current_offset - minimum_offset) / range).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let start = button_extent / 2.0 + (available - thumb_length) * position;
    axis.thumb_offset = axis.track_offset;
    axis.thumb_size = axis.track_size;
    if axis.horizontal {
        axis.thumb_offset.x += start;
        axis.thumb_size.width = thumb_length;
    } else {
        axis.thumb_offset.y += start;
        axis.thumb_size.height = thumb_length;
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:2085-2135
fn ExportInlineStitchedDecorations(root: &mut FragmentNode) {
    fn gather(
        fragment: &FragmentNode,
        path: &mut Vec<usize>,
        groups: &mut HashMap<(u64, u64), Vec<Vec<usize>>>,
    ) {
        if fragment.node_id != 0
            && fragment.kind == FragmentKind::kBox
            && fragment.paint.has_source
            && fragment.paint.establishes_paint_state
            && fragment.paint.display == Display::kInline
            && fragment.paint.box_decoration_break == BoxDecorationBreak::kSlice
        {
            groups
                .entry((fragment.node_id, fragment.fragmentainer_instance_id))
                .or_default()
                .push(path.clone());
        }
        for (index, child) in fragment.children.iter().enumerate() {
            path.push(index);
            gather(child, path, groups);
            path.pop();
        }
    }
    fn at_path<'a>(root: &'a FragmentNode, path: &[usize]) -> &'a FragmentNode {
        path.iter().fold(root, |node, &index| &node.children[index])
    }
    fn at_path_mut<'a>(root: &'a mut FragmentNode, path: &[usize]) -> &'a mut FragmentNode {
        path.iter()
            .fold(root, |node, index| &mut node.children[*index])
    }

    let mut groups = HashMap::new();
    gather(root, &mut Vec::new(), &mut groups);
    for fragments in groups.values() {
        if fragments.len() < 2 {
            continue;
        }
        let horizontal =
            at_path(root, &fragments[0]).paint.writing_mode == WritingMode::kHorizontalTb;
        let total_inline_size: f64 = fragments
            .iter()
            .map(|path| {
                let fragment = at_path(root, path);
                if horizontal {
                    fragment.size.width
                } else {
                    fragment.size.height
                }
            })
            .sum();
        let mut before = 0.0;
        for path in fragments {
            let fragment = at_path_mut(root, path);
            let inline_size = if horizontal {
                fragment.size.width
            } else {
                fragment.size.height
            };
            let offset = if fragment.paint.direction == TextDirection::kLtr {
                before
            } else {
                total_inline_size - before - inline_size
            };
            fragment.paint.stitched_decoration = Some(StitchedDecorationData {
                fragment_origin: Offset::default(),
                fragment_offset: if horizontal {
                    Offset { x: offset, y: 0.0 }
                } else {
                    Offset { x: 0.0, y: offset }
                },
                size: if horizontal {
                    Size {
                        width: total_inline_size,
                        height: fragment.size.height,
                    }
                } else {
                    Size {
                        width: fragment.size.width,
                        height: total_inline_size,
                    }
                },
            });
            before += inline_size;
        }
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:1969-2083
fn ResolveLogicalParentOccurrences(root: &mut FragmentNode) {
    struct Occurrence {
        // The physical path is used only by the debug preorder assertion.
        #[cfg(debug_assertions)]
        path: Vec<usize>,
        node_id: u64,
        fragment_instance_id: u64,
        fragmentainer_instance_id: u64,
        absolute_offset: Offset,
        size: Size,
        establishes_paint_state: bool,
    }
    fn gather(
        fragment: &FragmentNode,
        path: &mut Vec<usize>,
        parent_offset: Offset,
        all: &mut Vec<Occurrence>,
        by_node_id: &mut HashMap<u64, Vec<usize>>,
    ) {
        let absolute_offset = Offset {
            x: parent_offset.x + fragment.offset.x,
            y: parent_offset.y + fragment.offset.y,
        };
        let index = all.len();
        all.push(Occurrence {
            #[cfg(debug_assertions)]
            path: path.clone(),
            node_id: fragment.node_id,
            fragment_instance_id: fragment.fragment_instance_id,
            fragmentainer_instance_id: fragment.fragmentainer_instance_id,
            absolute_offset,
            size: fragment.size,
            establishes_paint_state: fragment.paint.establishes_paint_state,
        });
        if fragment.node_id != 0
            && fragment.paint.has_source
            && fragment.paint.establishes_paint_state
        {
            by_node_id.entry(fragment.node_id).or_default().push(index);
        }
        for (child_index, child) in fragment.children.iter().enumerate() {
            path.push(child_index);
            gather(child, path, absolute_offset, all, by_node_id);
            path.pop();
        }
    }
    fn distance_squared(candidate: &Occurrence, child_offset: Offset) -> f64 {
        let left = candidate.absolute_offset.x;
        let top = candidate.absolute_offset.y;
        let right = left + candidate.size.width;
        let bottom = top + candidate.size.height;
        let dx = if child_offset.x < left {
            left - child_offset.x
        } else if child_offset.x > right {
            child_offset.x - right
        } else {
            0.0
        };
        let dy = if child_offset.y < top {
            top - child_offset.y
        } else if child_offset.y > bottom {
            child_offset.y - bottom
        } else {
            0.0
        };
        dx * dx + dy * dy
    }
    fn resolve(
        fragment: &mut FragmentNode,
        path: &mut Vec<usize>,
        all: &[Occurrence],
        by_node_id: &HashMap<u64, Vec<usize>>,
        physical_ancestors: &mut Vec<usize>,
        next_index: &mut usize,
    ) {
        let current_index = *next_index;
        *next_index += 1;
        #[cfg(debug_assertions)]
        debug_assert_eq!(all[current_index].path, *path);
        if let Some(parent_node_id) = fragment.paint.logical_parent_node_id {
            let mut resolved = physical_ancestors.iter().rev().copied().find(|&index| {
                all[index].node_id == parent_node_id && all[index].establishes_paint_state
            });
            if resolved.is_none() {
                if let Some(candidates) = by_node_id.get(&parent_node_id) {
                    let mut eligible: Vec<usize> = candidates
                        .iter()
                        .copied()
                        .filter(|&index| {
                            index != current_index
                                && all[index].fragmentainer_instance_id
                                    == fragment.fragmentainer_instance_id
                        })
                        .collect();
                    if eligible.is_empty() {
                        eligible.extend(
                            candidates
                                .iter()
                                .copied()
                                .filter(|&index| index != current_index),
                        );
                    }
                    let child_offset = all[current_index].absolute_offset;
                    resolved = eligible.into_iter().min_by(|&left, &right| {
                        let left_distance = distance_squared(&all[left], child_offset);
                        let right_distance = distance_squared(&all[right], child_offset);
                        if left_distance != right_distance {
                            if left_distance < right_distance {
                                std::cmp::Ordering::Less
                            } else if right_distance < left_distance {
                                std::cmp::Ordering::Greater
                            } else {
                                std::cmp::Ordering::Equal
                            }
                        } else {
                            all[left]
                                .fragment_instance_id
                                .cmp(&all[right].fragment_instance_id)
                        }
                    });
                }
            }
            if let Some(index) = resolved {
                fragment.paint.logical_parent_fragment_instance_id =
                    Some(all[index].fragment_instance_id);
            }
        }
        physical_ancestors.push(current_index);
        for (child_index, child) in fragment.children.iter_mut().enumerate() {
            path.push(child_index);
            resolve(child, path, all, by_node_id, physical_ancestors, next_index);
            path.pop();
        }
        physical_ancestors.pop();
    }

    let mut all = Vec::new();
    let mut by_node_id = HashMap::new();
    gather(
        root,
        &mut Vec::new(),
        Offset::default(),
        &mut all,
        &mut by_node_id,
    );
    let mut next_index = 0;
    resolve(
        root,
        &mut Vec::new(),
        &all,
        &by_node_id,
        &mut Vec::new(),
        &mut next_index,
    );
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:2137-2173
fn UpdateStickyOffsets(
    objects: impl IntoIterator<Item = *mut LayoutObject>,
    reuse_constraints: bool,
) {
    // Logical preorder lets nested sticky offsets observe updated ancestors.
    for object in objects {
        let box_model = DynamicTo::<LayoutBoxModelObject>(object);
        if box_model.is_null()
            || !unsafe { &*box_model }
                .StyleRef()
                .HasStickyConstrainedPosition()
        {
            continue;
        }
        let box_model = unsafe { &mut *box_model };
        let constrained_axes = LayoutBoxModelObject::StickyConstrainedAxes(box_model.StyleRef());
        let mut update_axis = |axis: PhysicalAxis, axis_mask: PhysicalAxes| {
            if !(constrained_axes & axis_mask).is_nonzero() {
                return;
            }
            let scroll_container = box_model.ContainingScrollContainer(axis);
            if scroll_container.is_null() {
                return;
            }
            let cached = box_model.StickyConstraints().AxisData(axis);
            // cpp: core/paint/paint_layer_scrollable_area.cc:2420-2440
            // A layout-clean scroll changes the offset, not the containing
            // geometry. Keep each axis's rooted constraints only for its same
            // native owner; missing/changed owners still build real constraints.
            let can_reuse = reuse_constraints
                && !cached.is_null()
                && unsafe { &*cached }.containing_scroll_container.Get() as *const LayoutBox
                    == scroll_container;
            if !can_reuse {
                let constraints = box_model.ComputeStickyPositionConstraintsForLayout(
                    scroll_container,
                    false,
                    axis_mask,
                );
                box_model.SetStickyConstraints(constraints);
            } else {
                box_model.SetNeedsPaintPropertyUpdate();
            }
            let mut scroll_position = gfx::PointF::default();
            let area = unsafe { &*scroll_container }.GetScrollableArea();
            if !area.is_null() {
                scroll_position = unsafe { &*area }.ScrollPosition();
            } else if unsafe { &*scroll_container }.IsLayoutView() {
                let source = unsafe { &*scroll_container }.GetNode();
                if !source.is_null() && unsafe { &*source }.IsElementNode() {
                    if let Some(data) = unsafe { &*(source as *const Element) }.InputElementData() {
                        scroll_position = gfx::PointF::new(
                            data.scroll_offset.x as f32,
                            data.scroll_offset.y as f32,
                        );
                    }
                }
            }
            box_model
                .StickyConstraints()
                .ComputeStickyOffset(&scroll_position, axis_mask);
        };
        update_axis(PhysicalAxis::kHorizontal, kPhysicalAxesHorizontal);
        update_axis(PhysicalAxis::kVertical, kPhysicalAxesVertical);
    }
}

// Scroll-only paint-property update, corresponding to Blink's scroll translation
// and sticky property update. The caller has verified unchanged LayoutResult;
// styles, fragments, clips and resources keep their existing owned snapshot.
pub fn RefreshScrollPaintProperties(
    root: &mut LayoutObject,
    fragments: &mut FragmentNode,
    reuse_sticky_constraints: bool,
) {
    let root_ptr: *mut LayoutObject = root;
    // Export assigns sourced objects consecutive, one-based preorder ids.
    // Keep that dense mapping without rebuilding a hash table for each scroll.
    let mut sources = Vec::new();
    let mut object = root_ptr;
    while !object.is_null() {
        // The borrowed resident tree owns these objects; the outer heap scope
        // prevents collection during this walk and sticky-property update.
        let current = unsafe { &*object };
        if !current.GetNode().is_null() {
            sources.push(object);
        }
        let next = current.NextInPreOrder(root_ptr);
        // Constraints never change topology. Update in the same logical
        // preorder so nested sticky objects still observe refreshed ancestors,
        // without allocating an all-object vector and walking it again.
        UpdateStickyOffsets(std::iter::once(object), reuse_sticky_constraints);
        object = next;
    }
    fn update(
        fragment: &mut FragmentNode,
        sources: &[*mut LayoutObject],
        retained_geometry: bool,
    ) -> bool {
        let own_changed = RefreshFragmentScrollPaintProperties(fragment, sources);
        let mut changed = own_changed;
        for child in &mut fragment.children {
            changed |= update(child, sources, retained_geometry);
        }
        // The layout boundary's certification covered the exported inputs.
        // Refresh mutated them in place: version exactly these paths so a
        // different resident PaintEngine cannot mistake stale revisions for a
        // fresh layout proof. Unaffected branches retain their certification.
        if changed && retained_geometry {
            layoutng_fragment_tree::fragment_tree::pre_paint_revision::MarkScrollPropertyChange(
                fragment,
                own_changed,
            );
        } else {
            // A refreshed layout is not the retained-geometry proof. Keep the
            // existing unobserved-input fallback until a full certification.
            if own_changed {
                fragment.pre_paint_revision = 0;
            }
            if changed {
                fragment.pre_paint_subtree_revision = 0;
            }
        }
        changed
    }
    update(fragments, &sources, reuse_sticky_constraints);
}

fn RefreshFragmentScrollPaintProperties(
    fragment: &mut FragmentNode,
    sources: &[*mut LayoutObject],
) -> bool {
    let old_scroll = fragment.paint.scroll_offset;
    let old_sticky = fragment.paint.sticky_offset;
    let old_scrollbars = fragment.paint.scrollbars.clone();
    if fragment.paint.has_source {
        if let Some(&object) = fragment
            .paint
            .logical_tree_order
            .checked_sub(1)
            .and_then(|order| usize::try_from(order).ok())
            .and_then(|index| sources.get(index))
        {
            let node = unsafe { &*(*object).GetNode() };
            if node.IsElementNode() {
                let element = unsafe { &*(node as *const Node as *const Element) };
                if let Some(data) = element.InputElementData() {
                    fragment.paint.scroll_offset = data.scroll_offset;
                }
            }
            if fragment.paint.position == Position::kSticky {
                let box_model = DynamicTo::<LayoutBoxModelObject>(object);
                if !box_model.is_null() {
                    let offset = unsafe { &*box_model }.StickyPositionOffset();
                    fragment.paint.sticky_offset = Offset {
                        x: Number(offset.left),
                        y: Number(offset.top),
                    };
                }
            }
            let layout_box = DynamicTo::<LayoutBox>(object);
            if !layout_box.is_null() && fragment.paint.scrollbars.is_some() {
                let layout_box = unsafe { &*layout_box };
                let area = layout_box.GetScrollableArea();
                if !area.is_null() {
                    let area = unsafe { &*area };
                    let theme = layout_box.ScrollbarThemeForLayout();
                    let visible =
                        area.VisibleContentRect(IncludeScrollbarsInRect::kExcludeScrollbars);
                    let contents = area.ContentsSize();
                    let minimum = area.MinimumScrollOffsetInt();
                    let maximum = area.MaximumScrollOffsetInt();
                    let current = area.GetScrollOffset();
                    let data = Arc::make_mut(fragment.paint.scrollbars.as_mut().unwrap());
                    if let Some(axis) = data.horizontal.as_mut() {
                        SetScrollbarThumb(
                            axis,
                            visible.width() as f64,
                            contents.width() as f64,
                            current.x() as f64,
                            minimum.x() as f64,
                            maximum.x() as f64,
                            theme.has_buttons,
                            theme.minimum_thumb_length as f64,
                        );
                    }
                    if let Some(axis) = data.vertical.as_mut() {
                        SetScrollbarThumb(
                            axis,
                            visible.height() as f64,
                            contents.height() as f64,
                            current.y() as f64,
                            minimum.y() as f64,
                            maximum.y() as f64,
                            theme.has_buttons,
                            theme.minimum_thumb_length as f64,
                        );
                    }
                } else if layout_box.IsLayoutView() {
                    let theme = layout_box.ScrollbarThemeForLayout();
                    let visible = fragment
                        .paint
                        .scroll_container
                        .map_or(fragment.size, |container| container.container_rect.size);
                    let contents = Size {
                        width: fragment.paint.scroll_size.width.max(visible.width),
                        height: fragment.paint.scroll_size.height.max(visible.height),
                    };
                    let current = fragment.paint.scroll_offset;
                    let data = Arc::make_mut(fragment.paint.scrollbars.as_mut().unwrap());
                    if let Some(axis) = data.horizontal.as_mut() {
                        SetScrollbarThumb(
                            axis,
                            visible.width,
                            contents.width,
                            current.x,
                            0.0,
                            (contents.width - visible.width).max(0.0),
                            theme.has_buttons,
                            theme.minimum_thumb_length as f64,
                        );
                    }
                    if let Some(axis) = data.vertical.as_mut() {
                        SetScrollbarThumb(
                            axis,
                            visible.height,
                            contents.height,
                            current.y,
                            0.0,
                            (contents.height - visible.height).max(0.0),
                            theme.has_buttons,
                            theme.minimum_thumb_length as f64,
                        );
                    }
                }
            }
        }
    }
    old_scroll != fragment.paint.scroll_offset
        || old_sticky != fragment.paint.sticky_offset
        || old_scrollbars != fragment.paint.scrollbars
}

#[path = "scroll_paint_property_index.rs"]
mod scroll_paint_property_index;
pub(crate) use scroll_paint_property_index::ScrollPaintPropertyIndex;

// cpp: layoutng/internal/boundary/layout_boundary.cc:2545-2605
fn InitialLetterTextStyle(initial_letter_box: &mut LayoutObject) -> *const NativeComputedStyle {
    let box_style = initial_letter_box.StyleRef();
    let paragraph = initial_letter_box.ContainingBlock();
    let box_font = box_style.GetFont();
    if paragraph.is_null() || box_style.InitialLetter().IsNormal() || box_font.is_null() {
        return box_style;
    }
    let paragraph_style = unsafe { &*paragraph }.StyleRef();
    let paragraph_font = paragraph_style.GetFont();
    if paragraph_font.is_null() {
        return box_style;
    }
    let box_primary = (unsafe { &*box_font }).PrimaryFont();
    if box_primary.is_null() {
        return box_style;
    }
    let paragraph_primary = (unsafe { &*paragraph_font }).PrimaryFont();
    if paragraph_primary.is_null() {
        return box_style;
    }
    let cap_height = unsafe { &*box_primary }.GetFontMetrics().CapHeight();
    let paragraph_cap_height = unsafe { &*paragraph_primary }.GetFontMetrics().CapHeight();
    if !(cap_height > 0.0) || !(paragraph_cap_height > 0.0) {
        return box_style;
    }
    let desired_cap_height = paragraph_style.ComputedLineHeight()
        * (box_style.InitialLetter().Size() - 1.0)
        + paragraph_cap_height;
    let source_size = box_style.ComputedFontSize();
    let mut adjusted_size = desired_cap_height * source_size / cap_height;
    if !adjusted_size.is_finite() || adjusted_size <= 1.0 {
        return box_style;
    }
    let node = initial_letter_box.GetNode();
    if node.is_null() {
        return box_style;
    }
    let defaults = ExtendedStyle::default();
    let input = unsafe { &*node }.InputStyle();
    let extra = input.extended.as_ref().unwrap_or(&defaults);
    let mut adjusted_font = std::ptr::null_mut();
    while adjusted_size > 1.0 {
        let request = NativeFontRequest {
            size: adjusted_size as f64,
            specified_size: adjusted_size as f64 / f64::from(extra.effective_zoom),
            letter_spacing: extra.letter_spacing,
            word_spacing: extra.word_spacing,
            writing_mode: PublicWritingMode(box_style.GetWritingMode()),
            language: &extra.language,
            families: &extra.font_families,
            weight: extra.font_weight,
            italic: extra.font_italic,
            orientation: box_style.GetFontDescription().Orientation(),
            smoothing: extra.font_smoothing,
        };
        let candidate = unsafe { &mut *CurrentNativeFontResolver() }.Resolve(&request);
        let primary = candidate.PrimaryFont();
        if primary.is_null()
            || unsafe { &*primary }.GetFontMetrics().CapHeight() <= desired_cap_height
        {
            adjusted_font = candidate;
            break;
        }
        adjusted_size -= 1.0;
    }
    if adjusted_font.is_null() || unsafe { &*adjusted_font }.PrimaryFont().is_null() {
        return box_style;
    }
    let mut builder = ComputedStyleBuilder::from_style(box_style);
    builder.SetFont(Member::from_ptr(adjusted_font));
    let font_height = builder.FontHeight();
    builder.SetLineHeight(&Length::Fixed(font_height));
    builder.SetVerticalAlign(EVerticalAlign::kBaseline);
    builder.TakeStyle()
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:2648-2709
fn RebindFont(
    input: &ComputedStyle,
    native_style: &NativeComputedStyle,
    parent: *const LayoutObject,
) -> *const NativeComputedStyle {
    let zoomed = crate::internal::css_zoom::ZoomedStyle(input);
    let input = zoomed.as_ref();
    let defaults = ExtendedStyle::default();
    let extra = input.extended.as_ref().unwrap_or(&defaults);
    let mut builder = ComputedStyleBuilder::from_style(native_style);
    let mut resolved_font_size = extra.font_size;
    if extra.font_size_math && !parent.is_null() {
        let parent_style = unsafe { &*parent }.StyleRef();
        let parent_font = parent_style.GetFont();
        if !parent_font.is_null() {
            let parent_depth = parent_style.MathDepth() as i32;
            let mut scale = 1.0;
            if extra.math_depth > parent_depth {
                let mut script_scale = 0.71;
                let mut script_script_scale = script_scale * script_scale;
                let data = unsafe { &*parent_font }.PrimaryFont();
                if !data.is_null() {
                    let face = unsafe { &*data }.PlatformData().GetHarfBuzzFace();
                    script_scale = OpenTypeMathSupport::MathConstant(
                        face,
                        MathConstants::kScriptPercentScaleDown,
                    )
                    .map(|value| value as f64)
                    .unwrap_or(script_scale);
                    if !(script_scale > 0.0) {
                        script_scale = 0.71;
                    }
                    script_script_scale = OpenTypeMathSupport::MathConstant(
                        face,
                        MathConstants::kScriptScriptPercentScaleDown,
                    )
                    .map(|value| value as f64)
                    .unwrap_or(script_script_scale);
                    if !(script_script_scale > 0.0) {
                        script_script_scale = script_scale * script_scale;
                    }
                }
                let scale_for_depth = |depth: i32| {
                    if depth <= 0 {
                        1.0
                    } else if depth == 1 {
                        script_scale
                    } else {
                        script_script_scale * 0.71_f64.powi(depth - 2)
                    }
                };
                scale = scale_for_depth(extra.math_depth) / scale_for_depth(parent_depth);
            }
            resolved_font_size = parent_style.ComputedFontSize() as f64 * scale;
        }
    }
    let request = NativeFontRequest {
        size: resolved_font_size,
        specified_size: resolved_font_size / f64::from(extra.effective_zoom),
        letter_spacing: extra.letter_spacing,
        word_spacing: extra.word_spacing,
        writing_mode: PublicWritingMode(builder.GetWritingMode()),
        language: &extra.language,
        families: &extra.font_families,
        weight: extra.font_weight,
        italic: extra.font_italic,
        orientation: builder.ComputeFontOrientation(),
        smoothing: extra.font_smoothing,
    };
    let font = unsafe { &mut *CurrentNativeFontResolver() }.Resolve(&request);
    builder.SetFont(Member::from_ptr(font));
    builder.TakeStyle()
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:2607-2647
// cpp: layoutng/internal/boundary/layout_boundary.cc:2710-2752
fn RebindFontsFromConstraintSpace(root: &mut LayoutObject, only_dirty: bool) {
    let mut rebound = std::collections::HashSet::new();
    let root_ptr: *mut LayoutObject = root;
    let mut object = root_ptr;
    while !object.is_null() {
        let current = unsafe { &mut *object };
        let node = current.GetNode();
        // A parent's resolved font can change even when the child's source
        // style is unchanged (for example relative MathML font sizes).
        let inherited_font_changed = rebound.contains(&current.Parent());
        if only_dirty
            && !node.is_null()
            && !unsafe { &*node }.NeedsInputFontBinding()
            && !inherited_font_changed
        {
            object = current.NextInPreOrder(root_ptr);
            continue;
        }
        let previous_font = if node.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*node }.InputBoundFont()
        };
        if node.is_null() {
            let parent = current.Parent();
            if !parent.is_null() {
                let font = unsafe { &*parent }.StyleRef().GetFont();
                if !font.is_null() && (!only_dirty || current.StyleRef().GetFont() != font) {
                    let mut builder = ComputedStyleBuilder::from_style(current.StyleRef());
                    builder.SetFont(Member::from_ptr(font));
                    current.SetStyleWithChanges(builder.TakeStyle(), ApplyStyleChanges::kNo);
                    current.SetNeedsLayoutAndIntrinsicWidthsRecalc(&raw const kFontsChanged);
                    rebound.insert(object);
                }
            }
        } else if unsafe { &*node }.IsTextNode() {
            let mut logical_parent = unsafe { &*node }.parentNode() as *mut Node;
            while !logical_parent.is_null()
                && unsafe { &*logical_parent }.GetLayoutObject().is_null()
            {
                logical_parent = unsafe { &*logical_parent }.parentNode() as *mut Node;
            }
            if !logical_parent.is_null() {
                let layout_parent = unsafe { &mut *unsafe { &*logical_parent }.GetLayoutObject() };
                let style = if layout_parent.IsInitialLetterBox() {
                    InitialLetterTextStyle(layout_parent)
                } else {
                    layout_parent.StyleRef() as *const NativeComputedStyle
                };
                unsafe { &mut *node }.SetComputedStyle(style);
                current.SetStyleWithChanges(style, ApplyStyleChanges::kNo);
                let text = To::<LayoutText>(object);
                let text = unsafe { &mut *text };
                text.InvalidateInlineItems();
                text.SetNeedsCollectInlines();
                text.SetNeedsLayoutAndIntrinsicWidthsRecalc(&raw const kFontsChanged);
            }
        } else {
            let style = RebindFont(
                unsafe { &*node }.InputStyle(),
                current.StyleRef(),
                current.Parent(),
            );
            unsafe { &mut *node }.SetComputedStyle(style);
            current.SetStyleWithChanges(style, ApplyStyleChanges::kNo);
            let first_line_style = unsafe { &*node }.InputFirstLineStyle();
            if !first_line_style.is_null() {
                if let Some(first_line_input) = unsafe { &*node }.InputFirstLineStyleData() {
                    let rebound = RebindFont(
                        first_line_input,
                        unsafe { &*first_line_style },
                        current.Parent(),
                    );
                    unsafe { &mut *node }.SetInputFirstLineStyle(rebound);
                }
            }
            current.SetNeedsLayoutAndIntrinsicWidthsRecalc(&raw const kFontsChanged);
        }
        if !node.is_null() {
            let resolved_font = current.StyleRef().GetFont() as *mut font_engine::fonts::font::Font;
            // A dirty source node can change width/height while retaining its
            // exact resolved font request. Propagate typography only when the
            // actual binding changes; the node's own layout/style dirtiness
            // and text invalidation remain intact.
            if resolved_font != previous_font {
                rebound.insert(object);
            }
            unsafe { &mut *node }.SetInputBoundFont(resolved_font);
            unsafe { &mut *node }.ClearInputFontBinding();
        }
        object = current.NextInPreOrder(root_ptr);
    }
    object = root_ptr;
    while !object.is_null() {
        let current = unsafe { &mut *object };
        if current.GetNode().is_null() {
            let owner = current.InputOwnerForLayout();
            let owner_object = owner.GetLayoutObject();
            if !owner_object.is_null() {
                let font = unsafe { &*owner_object }.StyleRef().GetFont();
                if !font.is_null() && (!only_dirty || current.StyleRef().GetFont() != font) {
                    let mut builder = ComputedStyleBuilder::from_style(current.StyleRef());
                    builder.SetFont(Member::from_ptr(font));
                    current.SetStyleWithChanges(builder.TakeStyle(), ApplyStyleChanges::kNo);
                    current.SetNeedsLayoutAndIntrinsicWidthsRecalc(&raw const kFontsChanged);
                }
            }
        }
        object = current.NextInPreOrder(root_ptr);
    }
    object = root_ptr;
    while !object.is_null() {
        let current = unsafe { &*object };
        let text = DynamicTo::<LayoutText>(object);
        if !text.is_null() && (!only_dirty || rebound.contains(&object)) {
            unsafe { &mut *text }.TransformAndSecureOriginalText();
        }
        object = current.NextInPreOrder(root_ptr);
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.h:20-33
// cpp: layoutng/internal/boundary/layout_boundary.cc:2516-2520
// cpp: layoutng/internal/boundary/layout_boundary.cc:2521-2521
pub struct NativeLayoutEnvironment {
    storage_: Box<NativeLayoutEnvironmentStorage>,
}

// cpp: layoutng/internal/boundary/layout_boundary.h:30-32
// cpp: layoutng/internal/boundary/layout_boundary.cc:2495-2514
struct NativeLayoutEnvironmentStorage {
    // C++ constructs font, hyphenation, phrase-break, text and image scopes
    // in that order. Rust drops fields top to bottom, so list them in reverse
    // order and retain stable boxes under each external scope.
    text_and_images: InputTextAndImageScopes,
    phrase_break_scope: NativePhraseBreakResolverScope<'static>,
    phrase_break: Box<InputPhraseBreakResolver>,
    hyphenation_scope: NativeHyphenationResolverScope<'static>,
    hyphenation: Box<InputHyphenationResolver>,
    font_binding: FontBinding,
}

impl NativeLayoutEnvironmentStorage {
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2496-2505
    fn new(root: &mut LayoutObject, space: &ConstraintSpace) -> Self {
        let font_binding = FontBinding::new(root, space);
        let mut hyphenation = Box::new(InputHyphenationResolver::new(space.hyphenation.clone()));
        let hyphenation_ptr = hyphenation.as_mut() as *mut dyn NativeHyphenationResolver;
        let hyphenation_scope =
            unsafe { NativeHyphenationResolverScope::new_from_raw(hyphenation_ptr) };
        let mut phrase_break = Box::new(InputPhraseBreakResolver::new(space.phrase_break.clone()));
        let phrase_break_ptr = phrase_break.as_mut() as *mut dyn NativePhraseBreakResolver;
        let phrase_break_scope =
            unsafe { NativePhraseBreakResolverScope::new_from_raw(phrase_break_ptr) };
        let text_and_images = InputTextAndImageScopes::new(space);
        Self {
            text_and_images,
            phrase_break_scope,
            phrase_break,
            hyphenation_scope,
            hyphenation,
            font_binding,
        }
    }
}

impl NativeLayoutEnvironment {
    // cpp: layoutng/internal/boundary/layout_boundary.h:22-22
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2516-2518
    pub fn new(root: &mut LayoutObject, space: &ConstraintSpace) -> Self {
        Self {
            storage_: Box::new(NativeLayoutEnvironmentStorage::new(root, space)),
        }
    }
}

impl LayoutBoundaryEnvironment for NativeLayoutEnvironment {
    // cpp: layoutng/internal/boundary/layout_boundary.h:26-27
    // cpp: layoutng/internal/boundary/layout_boundary.cc:2522-2524
    fn ReusesPreparedFonts(&self) -> bool {
        self.storage_.font_binding.reused
    }

    // cpp: layoutng/internal/boundary/layout_boundary.cc:2526-2528
    fn Commit(&mut self, root: &mut LayoutObject) {
        self.storage_.font_binding.Commit(root)
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.h:34-40
// cpp: layoutng/internal/boundary/layout_boundary.cc:2755-2820
pub fn PrepareLayoutObjectTree(
    root: &mut LayoutObject,
    space: &ConstraintSpace,
    native_space: &NativeConstraintSpace,
    reuse_prepared_fonts: bool,
) {
    if !root.Parent().is_null() {
        panic!("Layout input must reference the tree root");
    }
    let root_node = root.GetNode();
    if root_node.is_null() {
        panic!("Layout root requires standalone metadata");
    }
    NativeNodeMetadataRelations::FinalizeTreeRelations(unsafe { &mut *root_node })
        .unwrap_or_else(|error| panic!("{error:?}"));

    if space.printing {
        let view = DynamicTo::<LayoutView>(root as *mut LayoutObject);
        if view.is_null() {
            panic!("Print layout root must be a LayoutView");
        }
        unsafe { &mut *view }.SetInitialContainingBlockSizeForPrinting(ToPhysicalSize(
            native_space.AvailableSize(),
            native_space.GetWritingMode(),
        ));
    }
    RebindFontsFromConstraintSpace(root, reuse_prepared_fonts);

    let root_ptr: *mut LayoutObject = root;
    let viewport_defining_element = unsafe { &*root_node }.InputViewportDefiningElement();
    let mut has_first_line_styles = false;
    let mut object = root_ptr;
    while !object.is_null() {
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() {
            has_first_line_styles |= !unsafe { &*node }.InputFirstLineStyle().is_null();
        }
        object = current.NextInPreOrder(root_ptr);
    }
    object = root_ptr;
    while !object.is_null() {
        let current = unsafe { &*object };
        let box_ptr = DynamicTo::<LayoutBox>(object);
        if !box_ptr.is_null() {
            let shape = ShapeOutsideInfo::Info(unsafe { &*box_ptr });
            if !shape.is_null() {
                unsafe { &mut *shape }.MarkShapeAsDirty();
                // A shape can depend on mutable host image pixels, which are
                // deliberately absent from the font cache.
                unsafe { &mut *box_ptr }
                    .SetNeedsLayoutAndIntrinsicWidthsRecalc(&raw const kStyleChange);
            }
        }
        let node = current.GetNode();
        if !node.is_null() {
            unsafe { &mut *node }.PrepareStandaloneLayout(
                space,
                viewport_defining_element,
                has_first_line_styles,
            );
        }
        object = current.NextInPreOrder(root_ptr);
    }
    object = root_ptr;
    while !object.is_null() {
        let current = unsafe { &*object };
        let layout_inline = DynamicTo::<LayoutInline>(object);
        let node = current.GetNode();
        if !layout_inline.is_null() && !node.is_null() {
            let node = unsafe { &*node };
            let needs_fragment = StyleRequiresPaintBackedInlineFragment(node.InputStyle())
                || node
                    .InputFirstLineStyleData()
                    .as_ref()
                    .is_some_and(StyleRequiresPaintBackedInlineFragment);
            if needs_fragment {
                let layout_inline = unsafe { &mut *layout_inline };
                layout_inline.SetIsInLayoutNGInlineFormattingContext(true);
                layout_inline.SetShouldCreateBoxFragmentDefault();
            }
        }
        object = current.NextInPreOrder(root_ptr);
    }
}

pub fn PrepareLayoutObjectTreeDefault(
    root: &mut LayoutObject,
    space: &ConstraintSpace,
    native_space: &NativeConstraintSpace,
) {
    PrepareLayoutObjectTree(root, space, native_space, false)
}

// cpp: layoutng/internal/boundary/layout_boundary.h:41-43
// cpp: layoutng/internal/boundary/layout_boundary.cc:2822-2858
pub fn ExportLayoutObjectTree(
    root: &mut LayoutObject,
    result: &LayoutResult,
    space: &ConstraintSpace,
) -> FragmentNode {
    let mut sources = SourceMap::new();
    let mut style_sources = StyleSourceMap::new();
    let mut objects = Vec::new();
    let mut logical_order = 0;
    let root_ptr: *mut LayoutObject = root;
    let mut object = root_ptr;
    while !object.is_null() {
        // Export runs inside LayoutHeapScope; the resident tree already owns
        // these objects. Temporary Persistent cells only retire the same roots
        // and spuriously force another full-heap collection after each export.
        objects.push(object);
        let current = unsafe { &*object };
        let node = current.GetNode();
        if !node.is_null() {
            let input_node = unsafe { &*node };
            logical_order += 1;
            sources.entry(object).or_insert(SourceEntry {
                node,
                logical_tree_order: logical_order,
            });
            style_sources
                .entry(current.StyleRef())
                .or_insert(input_node.InputStyle());
            let first_line_style = input_node.InputFirstLineStyle();
            if !first_line_style.is_null() {
                if let Some(first_line_data) = input_node.InputFirstLineStyleData() {
                    style_sources
                        .entry(first_line_style)
                        .or_insert(first_line_data);
                }
            }
        }
        object = current.NextInPreOrder(root_ptr);
    }
    UpdateStickyOffsets(objects.iter().copied(), false);
    let resources = Arc::new(PaintResources {
        fonts: space.fonts.clone(),
        images: space.images.clone(),
        device_pixel_ratio: space.device_pixel_ratio,
        viewport: space.viewport,
    });
    let mut paint_state_owners = HashSet::new();
    let mut next_fragment_instance_id = 1;
    let mut next_fragmentainer_instance_id = 1;
    let mut output = ExportFragment(
        result.GetPhysicalFragment(),
        PhysicalOffset::default(),
        &sources,
        &style_sources,
        &resources,
        &mut paint_state_owners,
        0,
        0,
        &mut next_fragment_instance_id,
        &mut next_fragmentainer_instance_id,
    );
    ResolveLogicalParentOccurrences(&mut output);
    ExportInlineStitchedDecorations(&mut output);
    output
}
