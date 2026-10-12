//! Platform-neutral catalog selection and resolved font caching.
//!
//! The caller supplies immutable font bytes and an optional native backend.
//! Catalog registration validates metadata only; font files and native faces
//! are opened when a request actually selects them.

use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::fonts::font::FontFallbackSource;
use crate::system_fonts::{FontPresentation, FontQueryCollector, SystemFontQuery};
use crate::{
    Font, FontBackend, FontBackendFactory, FontDescription, FontFallbackMetadata, FontFallbackRole,
    FontFamilyType, FontOrientation, FontPlatformData, FontSelectionValue, FontSmoothingMode,
    FontVariation, LayoutLocale, OpenTypeFont, OpenTypeVariation, ResolvedFontFeatures,
    SharedFontFamily, SimpleFontData,
};
use foundation::{AtomicString, Length, MakeGarbageCollected, Persistent};

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontSmoothing {
    kAuto,
    kNone,
    kAntialiased,
    kSubpixelAntialiased,
}

// cpp: layoutng/internal/layout_input.h:1018-1023
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontUnicodeRange {
    pub start: u32,
    pub end: u32,
}
impl Default for FontUnicodeRange {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0x10ffff,
        }
    }
}

// cpp: layoutng/internal/layout_input.h:1031-1052
#[derive(Clone, Debug, PartialEq)]
pub struct FontFace {
    pub family: String,
    pub native_family: String,
    pub metrics_family: String,
    pub weight: f64,
    pub italic: bool,
    // Shared immutable bytes: aliases and layout snapshots must not copy font files.
    pub bytes: crate::SharedFontBytes,
    pub face_index: u32,
    pub variations: Vec<FontVariation>,
    pub unicode_ranges: Vec<FontUnicodeRange>,
    /// Platform-catalog classification consumed by FontEngine's fallback
    /// iterator. This does not affect an explicitly requested family.
    pub fallback_role: FontFallbackRole,
    /// BCP-47 locales for which the platform catalog prefers this face.
    pub fallback_locales: Vec<String>,
}
impl Default for FontFace {
    fn default() -> Self {
        Self {
            family: String::new(),
            native_family: String::new(),
            metrics_family: String::new(),
            weight: 400.0,
            italic: false,
            bytes: crate::SharedFontBytes::default(),
            face_index: 0,
            variations: Vec::new(),
            unicode_ranges: Vec::new(),
            fallback_role: FontFallbackRole::kFamilyOnly,
            fallback_locales: Vec::new(),
        }
    }
}

// cpp: layoutng/internal/layout_font_resolver.h:18-31
pub struct FontRequest<'a> {
    /// The immutable computed description produced by Style. Layout only
    /// rebinds it to the ConstraintSpace's resolved faces; it must not rebuild
    /// the description from the smaller face-selection key.
    pub base_description: Option<&'a FontDescription>,
    pub size: f64,
    pub specified_size: f64,
    pub letter_spacing: f64,
    pub word_spacing: f64,
    pub language: &'a str,
    pub families: &'a [String],
    pub weight: f64,
    pub italic: bool,
    pub orientation: FontOrientation,
    pub smoothing: FontSmoothing,
}

impl<'a> FontRequest<'a> {
    // cpp: layoutng/internal/layout_font_resolver.h:30-30
    pub fn new_auto(
        size: f64,
        letter_spacing: f64,
        word_spacing: f64,
        language: &'a str,
        families: &'a [String],
        weight: f64,
        italic: bool,
        orientation: FontOrientation,
    ) -> Self {
        Self {
            base_description: None,
            size,
            specified_size: size,
            letter_spacing,
            word_spacing,
            language,
            families,
            weight,
            italic,
            orientation,
            smoothing: FontSmoothing::kAuto,
        }
    }

    pub fn WithBaseDescription(mut self, description: &'a FontDescription) -> Self {
        self.base_description = Some(description);
        self.specified_size = description.SpecifiedSize() as f64;
        self.smoothing = match description.FontSmoothing() {
            crate::FontSmoothingMode::kNoSmoothing => FontSmoothing::kNone,
            crate::FontSmoothingMode::kAntialiased => FontSmoothing::kAntialiased,
            crate::FontSmoothingMode::kSubpixelAntialiased => FontSmoothing::kSubpixelAntialiased,
            crate::FontSmoothingMode::kAutoSmoothing => FontSmoothing::kAuto,
        };
        self
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
    source: FontFace,
    prepared: OnceCell<PreparedFontFace>,
    unicode_ranges: Vec<(u32, u32)>,
    variations: Vec<FontVariation>,
    fallback_role: FontFallbackRole,
    fallback_locales: Vec<String>,
}

struct PreparedFontFace {
    font: Arc<OpenTypeFont>,
    backend: Option<Arc<dyn FontBackend>>,
    supports_optical_size: bool,
    supports_weight: bool,
}

impl InputFontFace {
    fn Prepare(&self, factory: Option<&dyn FontBackendFactory>) -> &PreparedFontFace {
        self.prepared.get_or_init(|| {
            let face = &self.source;
            let variations: Vec<OpenTypeVariation> = face
                .variations
                .iter()
                .map(|variation| OpenTypeVariation {
                    tag: variation.tag,
                    value: variation.value,
                })
                .collect();
            let font = Arc::new(OpenTypeFont::new_shared(
                face.bytes.clone(),
                face.face_index,
                1.0,
                &variations,
            ));
            // Native system-ui carries the host's optical instance. Do not
            // replace it with axes derived from CSS pixels or weight.
            let native_system_ui = face.family == "system-ui";
            PreparedFontFace {
                supports_optical_size: !native_system_ui && font.HasVariationAxis(OPTICAL_SIZE_TAG),
                supports_weight: !native_system_ui && font.HasVariationAxis(WEIGHT_TAG),
                font,
                backend: factory.map(|factory| {
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
            }
        })
    }
}

struct InputFontEntry {
    description: FontDescription,
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

struct FontCatalog {
    faces: Vec<InputFontFace>,
    backend_factory: Option<Arc<dyn FontBackendFactory>>,
}

#[derive(Clone, Copy)]
struct FontDataLocation {
    source: usize,
    position: usize,
    face_index: usize,
}

struct ResolvedFontData {
    font: Persistent<SimpleFontData>,
    variations: Vec<FontVariation>,
}

// The catalog and initialized data remain owned by the resolved Font even
// when the caller replaces its resolver/catalog. No callback borrows Layout.
struct ResolvedFontSource {
    catalog: Rc<FontCatalog>,
    description: FontDescription,
    specified_size: f64,
    weight: f64,
    italic: bool,
    orientation: FontOrientation,
    indices: Vec<usize>,
    fonts: Vec<OnceCell<ResolvedFontData>>,
    source_index: usize,
    queries: Rc<RefCell<Option<FontQueryCollector>>>,
    locations: Rc<RefCell<HashMap<*const SimpleFontData, FontDataLocation>>>,
}

impl FontFallbackSource for ResolvedFontSource {
    fn FontDataAt(&self, position: usize) -> *const SimpleFontData {
        self.fonts[position]
            .get_or_init(|| {
                let index = self.indices[position];
                let face = &self.catalog.faces[index];
                let prepared = face.Prepare(self.catalog.backend_factory.as_deref());
                let description = &self.description;
                let effective_size = description.EffectiveFontSize();
                let variations = ResolveFontVariations(
                    &face.variations,
                    prepared.supports_optical_size
                        && description.FontOpticalSizing()
                            == crate::OpticalSizing::kAutoOpticalSizing,
                    prepared.supports_weight,
                    // Optical sizing uses the specified CSS size, not the zoomed
                    // or font-size-adjusted effective size.
                    self.specified_size,
                    self.weight,
                    description,
                );
                let native_variations: Vec<OpenTypeVariation> = variations
                    .iter()
                    .map(|variation| OpenTypeVariation {
                        tag: variation.tag,
                        value: variation.value,
                    })
                    .collect();
                let open_type_font = prepared.font.WithSizeVariationsAndSpecifiedSize(
                    effective_size as f64,
                    &native_variations,
                    self.specified_size,
                );
                let backend = prepared.backend.as_ref().map(|backend| {
                    if variations.is_empty() {
                        backend.clone()
                    } else {
                        backend.WithVariations(&variations)
                    }
                });
                let platform = MakeGarbageCollected(FontPlatformData::new(
                    open_type_font,
                    backend,
                    effective_size,
                    self.weight >= 600.0 && face.weight < 600.0 && !prepared.supports_weight,
                    self.italic && !face.italic,
                    description.TextRendering(),
                    ResolvedFontFeatures::default(),
                    self.orientation,
                    face.unicode_ranges.clone(),
                ));
                let simple = SimpleFontData::Create(platform);
                self.locations.borrow_mut().insert(
                    simple,
                    FontDataLocation {
                        source: self.source_index,
                        position,
                        face_index: index,
                    },
                );
                ResolvedFontData {
                    font: Persistent::from_ptr(simple),
                    variations,
                }
            })
            .font
            .Get()
    }

    fn ReportMissing(&self, text: &[u16], priority: crate::FontFallbackPriority) {
        if let Some(queries) = self.queries.borrow().as_ref() {
            if let Some(query) = self.FallbackQuery(text, priority, queries) {
                queries.FallbackReadyLimit(&query, true);
            }
        }
    }

    fn SystemFallbackRanks(
        &self,
        text: &[u16],
        priority: crate::FontFallbackPriority,
    ) -> Option<Vec<Option<usize>>> {
        let queries = self.queries.borrow();
        let queries = queries.as_ref()?;
        let query = self.FallbackQuery(text, priority, queries);
        let limit = query
            .as_ref()
            .map_or(0, |query| queries.FallbackReadyLimit(query, false));
        Some(
            self.indices
                .iter()
                .map(|&index| {
                    let face = &self.catalog.faces[index].source;
                    if queries.FaceId(face).is_none() {
                        Some(usize::MAX)
                    } else {
                        query
                            .as_ref()
                            .and_then(|query| queries.MatchRank(query, face))
                            .filter(|&rank| rank < limit)
                    }
                })
                .collect(),
        )
    }
}

impl ResolvedFontSource {
    fn FallbackQuery(
        &self,
        text: &[u16],
        priority: crate::FontFallbackPriority,
        queries: &FontQueryCollector,
    ) -> Option<SystemFontQuery> {
        use crate::text::native::character::Character;
        let mut meaningful = false;
        for character in char::decode_utf16(text.iter().copied()) {
            let character = character.unwrap_or(char::REPLACEMENT_CHARACTER) as i32;
            if Character::IsPrivateUse(character) || Character::IsNonCharacter(character) {
                return None;
            }
            meaningful |= !Character::IsDefaultIgnorable(character);
        }
        meaningful.then(|| SystemFontQuery::Fallback {
            text: text.to_vec(),
            weight: self.weight,
            italic: self.italic,
            locale: self.description.LocaleOrDefault().Ascii(),
            presentation: if crate::fonts::font_fallback_priority::IsEmojiPresentationEmoji(
                priority,
            ) {
                FontPresentation::Emoji
            } else {
                FontPresentation::Text
            },
            base_face: self
                .indices
                .get(PrimaryFontPosition(&self.catalog.faces, &self.indices))
                .and_then(|&index| queries.FaceId(&self.catalog.faces[index].source)),
        })
    }
}

/// Owns catalog matching, native face admission, and resolved request caches.
///
/// Construction never parses catalog bytes. Resolution admits the primary
/// face; ordered fallback faces are admitted only when Font's iterator selects
/// them for a shaping request.
pub struct FontResolver {
    catalog: Rc<FontCatalog>,
    fonts_: Vec<InputFontEntry>,
    sources: Vec<Rc<ResolvedFontSource>>,
    queries: Rc<RefCell<Option<FontQueryCollector>>>,
    locations: Rc<RefCell<HashMap<*const SimpleFontData, FontDataLocation>>>,
}

// cpp: third_party/blink/renderer/core/css/font_size_functions.cc:132-168
fn FontAspectValue(
    font_data: &SimpleFontData,
    metric: crate::fonts::font_size_adjust::Metric,
) -> Option<f32> {
    use crate::fonts::font_size_adjust::Metric;

    let font_size = font_data.PlatformData().size();
    if font_size == 0.0 {
        return None;
    }
    let metrics = font_data.GetFontMetrics();
    let value = match metric {
        Metric::kCapHeight if metrics.CapHeight() > 0.0 => metrics.CapHeight() / font_size,
        Metric::kChWidth if metrics.HasZeroWidth() => metrics.ZeroWidth() / font_size,
        Metric::kIcWidth => font_data
            .IdeographicAdvanceWidth()
            .map_or(1.0, |value| value / font_size),
        Metric::kIcHeight => font_data
            .IdeographicAdvanceHeight()
            .map_or(1.0, |value| value / font_size),
        Metric::kExHeight if metrics.HasXHeight() => metrics.XHeight() / font_size,
        // CSS Fonts specifies a fallback multiplier of 1 for unavailable
        // metrics; Chromium's AspectValue follows that rule.
        _ => 1.0,
    };
    Some(value)
}

impl FontResolver {
    // cpp: layoutng/internal/boundary/layout_boundary.cc:146-200
    pub fn new(faces: &[FontFace], backend_factory: Option<Arc<dyn FontBackendFactory>>) -> Self {
        assert!(
            !faces.is_empty(),
            "font catalog must contain an OpenType face"
        );
        let faces_ = faces
            .iter()
            .map(|face| {
                ValidateFontFace(face);
                InputFontFace {
                    family: face.family.clone(),
                    weight: face.weight,
                    italic: face.italic,
                    source: face.clone(),
                    prepared: OnceCell::new(),
                    unicode_ranges: face
                        .unicode_ranges
                        .iter()
                        .map(|range| (range.start, range.end))
                        .collect(),
                    variations: face.variations.clone(),
                    fallback_role: face.fallback_role,
                    fallback_locales: face.fallback_locales.clone(),
                }
            })
            .collect();
        Self {
            catalog: Rc::new(FontCatalog {
                faces: faces_,
                backend_factory,
            }),
            fonts_: Vec::new(),
            sources: Vec::new(),
            queries: Rc::new(RefCell::new(None)),
            locations: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    /// Install a pure host-query outbox. No host callback is invoked during
    /// matching or shaping. Existing resolved sources share this outbox.
    pub fn SetQueryCollector(&mut self, collector: Option<FontQueryCollector>) {
        *self.queries.borrow_mut() = collector;
    }

    // cpp: layoutng/internal/boundary/layout_boundary.cc:202-217
    fn CachedFont(
        &self,
        request: &FontRequest<'_>,
        description: &FontDescription,
    ) -> Option<*mut Font> {
        if !request.weight.is_finite() || request.weight < 1.0 || request.weight > 1000.0 {
            panic!("font request weight must be within 1..1000");
        }
        let entry = self.fonts_.iter().find(|entry| {
            entry.description == *description
                && entry.size == request.size
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

    // FontBuilder::UpdateAdjustedSize resolves primary metrics before creating
    // the adjusted instance. Keep both sources alive for stable paint identity.
    fn BuildResolvedFonts(
        &mut self,
        request: &FontRequest<'_>,
        description: &FontDescription,
        order: &[usize],
    ) -> Rc<ResolvedFontSource> {
        let source = Rc::new(ResolvedFontSource {
            catalog: self.catalog.clone(),
            description: description.clone(),
            specified_size: request.specified_size,
            weight: request.weight,
            italic: request.italic,
            orientation: request.orientation,
            indices: order.to_vec(),
            fonts: order.iter().map(|_| OnceCell::new()).collect(),
            source_index: self.sources.len(),
            queries: self.queries.clone(),
            locations: self.locations.clone(),
        });
        self.sources.push(source.clone());
        source
    }

    fn FaceIndexFor(&self, font: *const SimpleFontData) -> usize {
        self.locations
            .borrow()
            .get(&font)
            .expect("font face index")
            .face_index
    }

    fn VariationsFor(&self, font: *const SimpleFontData) -> &[FontVariation] {
        let location = *self.locations.borrow().get(&font).expect("font variations");
        &self.sources[location.source].fonts[location.position]
            .get()
            .expect("resolved font")
            .variations
    }
}

impl FontResolver {
    // cpp: layoutng/internal/boundary/layout_boundary.cc:202-361
    pub fn Resolve(&mut self, request: &FontRequest<'_>) -> &mut Font {
        // Preserve the complete computed description produced by Style. The
        // old reconstruction from `default()` silently discarded translated
        // properties such as font-size-adjust and OpenType feature controls.
        let has_base_description = request.base_description.is_some();
        let mut description = request.base_description.cloned().unwrap_or_default();
        description.SetSpecifiedSize(request.specified_size as f32);
        description.SetComputedSize(request.size as f32);
        description.SetAdjustedSize(request.size as f32);
        description.SetLetterSpacing(&Length::Fixed(request.letter_spacing));
        description.SetWordSpacing(&Length::Fixed(request.word_spacing));
        description.SetOrientation(request.orientation);
        if !has_base_description {
            description.SetWeight(FontSelectionValue::from_double(request.weight));
            description.SetStyle(FontSelectionValue::from_int(if request.italic {
                14
            } else {
                0
            }));
        }
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
        if let Some(queries) = self.queries.borrow().as_ref() {
            for family in request.families {
                if self.catalog.faces.iter().any(|face| {
                    EqualFamilyName(&face.family, family) && queries.FaceId(&face.source).is_none()
                }) {
                    continue;
                }
                queries.Push(FamilyQuery(
                    family,
                    request,
                    &description.LocaleOrDefault().Ascii(),
                ));
            }
        }
        let cache_description = description.clone();
        if let Some(font) = self.CachedFont(request, &cache_description) {
            return unsafe { &mut *font };
        }
        let order = OrderFontFaces(
            &self.catalog.faces,
            request,
            self.queries.borrow().as_ref(),
            &description.LocaleOrDefault().Ascii(),
        );
        let primary_position = PrimaryFontPosition(&self.catalog.faces, &order.indices);
        let mut ordered_fonts = self.BuildResolvedFonts(request, &description, &order.indices);
        let mut primary_font = ordered_fonts.FontDataAt(primary_position);
        if std::env::var_os("RECHROM_TRACE_FONT_BINDING").is_some() {
            let face = &self.catalog.faces[order.indices[primary_position]];
            eprintln!(
                "FONT_BIND families={:?} requested_weight={} italic={} size={} primary={} face_weight={} face_italic={} size_adjust={:?}",
                request.families,
                request.weight,
                request.italic,
                request.size,
                face.family,
                face.weight,
                face.italic,
                description.SizeAdjust(),
            );
        }
        if description.HasSizeAdjust() && description.ComputedSize() != 0.0 {
            let adjust = description.SizeAdjust();
            let aspect = FontAspectValue(unsafe { &*primary_font }, adjust.GetMetric());
            if adjust.IsFromFont()
                && adjust.Value()
                    == crate::fonts::font_size_adjust::FontSizeAdjust::kFontSizeAdjustNone
            {
                description.SetSizeAdjust(
                    &crate::fonts::font_size_adjust::FontSizeAdjust::with_metric_and_type(
                        aspect.unwrap_or(
                            crate::fonts::font_size_adjust::FontSizeAdjust::kFontSizeAdjustNone,
                        ),
                        adjust.GetMetric(),
                        crate::fonts::font_size_adjust::ValueType::kFromFont,
                    ),
                );
            }
            let adjust = description.SizeAdjust();
            if adjust.Value() != crate::fonts::font_size_adjust::FontSizeAdjust::kFontSizeAdjustNone
            {
                if let Some(aspect) = aspect.filter(|value| *value != 0.0) {
                    let adjusted = (adjust.Value() / aspect) * description.ComputedSize();
                    if adjusted.is_finite() && adjusted > 0.0 {
                        description.SetAdjustedSize(adjusted);
                        ordered_fonts =
                            self.BuildResolvedFonts(request, &description, &order.indices);
                        primary_font = ordered_fonts.FontDataAt(primary_position);
                    }
                }
            }
        }
        if std::env::var_os("RECHROM_TRACE_CONTROL_WIDTH").is_some() && request.size == 14.0 {
            let face = &self.catalog.faces[order.indices[primary_position]];
            eprintln!(
                "RESOLVED_PRIMARY family={} variations={:?}",
                face.family,
                self.VariationsFor(primary_font)
            );
        }
        let fallback_metadata: Vec<FontFallbackMetadata> = order
            .indices
            .iter()
            .map(|&index| {
                let face = &self.catalog.faces[index];
                FontFallbackMetadata {
                    role: face.fallback_role,
                    locales: face.fallback_locales.clone(),
                    weight: face.weight as f32,
                    italic: face.italic,
                }
            })
            .collect();
        let font = MakeGarbageCollected(Font::new_with_lazy_fallbacks(
            description,
            ordered_fonts,
            primary_font,
            &fallback_metadata,
            order.font_group_length,
        ));
        self.fonts_.push(InputFontEntry {
            description: cache_description,
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

    pub fn FaceIndex(&self, font_data: *const SimpleFontData) -> usize {
        self.FaceIndexFor(font_data)
    }

    pub fn Variations(&self, font_data: *const SimpleFontData) -> &[FontVariation] {
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
struct FontFaceOrder {
    indices: Vec<usize>,
    font_group_length: usize,
}

fn FamilyQuery(family: &str, request: &FontRequest<'_>, locale: &str) -> SystemFontQuery {
    if matches!(
        family.to_ascii_lowercase().as_str(),
        "serif"
            | "sans-serif"
            | "monospace"
            | "cursive"
            | "fantasy"
            | "system-ui"
            | "ui-serif"
            | "ui-sans-serif"
            | "ui-monospace"
            | "ui-rounded"
            | "math"
    ) {
        SystemFontQuery::MatchGeneric {
            generic: family.to_owned(),
            weight: request.weight,
            italic: request.italic,
            locale: locale.to_owned(),
        }
    } else {
        SystemFontQuery::MatchFamily {
            family: family.to_owned(),
            weight: request.weight,
            italic: request.italic,
            locale: locale.to_owned(),
        }
    }
}

fn OrderFontFaces(
    faces: &[InputFontFace],
    request: &FontRequest<'_>,
    queries: Option<&FontQueryCollector>,
    locale: &str,
) -> FontFaceOrder {
    let mut order = Vec::with_capacity(faces.len());
    let mut selected = vec![false; faces.len()];
    let append_best_face = |family: &str, order: &mut Vec<usize>, selected: &mut [bool]| loop {
        let query = FamilyQuery(family, request, locale);
        let host_rank = |face: &InputFontFace| {
            let Some(queries) = queries else {
                return (0, 0);
            };
            if queries.FaceId(&face.source).is_none() {
                // Caller/CSS-owned faces retain precedence over native aliases.
                (0, 0)
            } else if let Some(rank) = queries.MatchRank(&query, &face.source) {
                (1, rank)
            } else {
                (2, 0)
            }
        };
        let mut best: Option<usize> = None;
        for (index, face) in faces.iter().enumerate() {
            if !selected[index] && !face.family.is_empty() && EqualFamilyName(family, &face.family)
            {
                let rank = (
                    host_rank(face),
                    face.italic != request.italic,
                    FontWeightRank(face.weight, request.weight),
                    index,
                );
                if best.map_or(true, |previous| {
                    let previous_face = &faces[previous];
                    let previous_rank = (
                        host_rank(previous_face),
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
    if order.is_empty()
        && !faces
            .iter()
            .any(|face| face.fallback_role != FontFallbackRole::kFamilyOnly)
    {
        // Small embedders often provide one unclassified face. Preserve that
        // face as the family/.notdef candidate without publishing it as a
        // general platform fallback font.
        selected[0] = true;
        order.push(0);
    }
    let font_group_length = order.len();
    for (index, face) in faces.iter().enumerate() {
        if !selected[index]
            && !matches!(
                face.fallback_role,
                FontFallbackRole::kFamilyOnly | FontFallbackRole::kLastResort
            )
        {
            order.push(index);
        }
    }
    for (index, face) in faces.iter().enumerate() {
        if !selected[index] && face.fallback_role == FontFallbackRole::kLastResort {
            order.push(index);
        }
    }
    FontFaceOrder {
        indices: order,
        font_group_length,
    }
}

// cpp: layoutng/internal/boundary/layout_boundary.cc:296-315
const OPTICAL_SIZE_TAG: u32 = u32::from_be_bytes(*b"opsz");
const WEIGHT_TAG: u32 = u32::from_be_bytes(*b"wght");

fn ResolveFontVariations(
    face_variations: &[FontVariation],
    apply_optical_size: bool,
    supports_weight: bool,
    specified_size: f64,
    request_weight: f64,
    description: &FontDescription,
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
    if apply_optical_size {
        set_variation(OPTICAL_SIZE_TAG, specified_size);
    }
    if supports_weight {
        set_variation(WEIGHT_TAG, request_weight);
    }
    // cpp: font_platform_data_mac.mm:157-180. Registered-axis values derived
    // from high-level CSS properties are installed first. Explicit
    // font-variation-settings are applied last and therefore win, including
    // an explicit `opsz` overriding font-optical-sizing:auto. The face vector
    // contains catalog/@font-face defaults and remains the lowest priority.
    if let Some(settings) = description.VariationSettings() {
        for setting in settings.iter() {
            set_variation(setting.Tag(), setting.Value() as f64);
        }
    }
    variations.sort_unstable_by_key(|setting| setting.tag);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FontBackendGlyphMetrics, FontBackendMetrics};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct CountingFactory(Arc<AtomicUsize>);
    struct TestBackend;

    impl FontBackendFactory for CountingFactory {
        fn Create(
            &self,
            _bytes: &[u8],
            _face_index: u32,
            _variations: &[FontVariation],
            _native_family: &str,
            _metrics_family: &str,
            _weight: f64,
            _italic: bool,
        ) -> Arc<dyn FontBackend> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Arc::new(TestBackend)
        }
    }

    impl FontBackend for TestBackend {
        fn Metrics(&self, size: f32, _bold: bool, _italic: bool) -> FontBackendMetrics {
            FontBackendMetrics {
                ascent: size * 0.8,
                descent: size * 0.2,
                x_height: size * 0.5,
                cap_height: size * 0.7,
                ..Default::default()
            }
        }
        fn GlyphMetrics(
            &self,
            _glyph: u16,
            size: f32,
            _bold: bool,
            _italic: bool,
        ) -> FontBackendGlyphMetrics {
            FontBackendGlyphMetrics {
                advance: size * 0.6,
                ..Default::default()
            }
        }
        fn WithVariations(&self, _variations: &[FontVariation]) -> Arc<dyn FontBackend> {
            Arc::new(Self)
        }
    }

    fn face(family: &str, character: u32, role: FontFallbackRole) -> FontFace {
        FontFace {
            family: family.to_owned(),
            bytes: crate::minimal_font::Build(false, 0, character, false, false).into(),
            fallback_role: role,
            ..Default::default()
        }
    }

    fn request(families: &[String]) -> FontRequest<'_> {
        FontRequest::new_auto(
            16.0,
            0.0,
            0.0,
            "en",
            families,
            400.0,
            false,
            FontOrientation::kHorizontal,
        )
    }

    #[test]
    fn registering_catalog_never_reads_font_bytes_or_creates_native_faces() {
        let calls = Arc::new(AtomicUsize::new(0));
        let faces: Vec<FontFace> = (0..256)
            .map(|index| FontFace {
                family: format!("unopened-{index}"),
                // Intentionally not an OpenType file: registration must not parse.
                bytes: vec![0xde, 0xad].into(),
                fallback_role: FontFallbackRole::kText,
                ..Default::default()
            })
            .collect();
        let resolver = FontResolver::new(&faces, Some(Arc::new(CountingFactory(calls.clone()))));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(resolver
            .catalog
            .faces
            .iter()
            .all(|face| face.prepared.get().is_none()));
    }

    #[test]
    fn resolving_primary_is_lazy_cached_and_keeps_original_paint_index() {
        let _heap = foundation::LayoutHeapScope::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let faces = vec![
            FontFace {
                family: "unrelated".to_owned(),
                bytes: vec![0xde].into(),
                ..Default::default()
            },
            face("requested", 'A' as u32, FontFallbackRole::kFamilyOnly),
            FontFace {
                family: "unopened-system".to_owned(),
                bytes: vec![0xad].into(),
                fallback_role: FontFallbackRole::kText,
                ..Default::default()
            },
        ];
        let mut resolver =
            FontResolver::new(&faces, Some(Arc::new(CountingFactory(calls.clone()))));
        let families = vec!["REQUESTED".to_owned()];
        let request = request(&families);
        let resolved = resolver.Resolve(&request) as *mut Font;
        let primary = unsafe { &*resolved }.PrimaryFont();
        assert_eq!(resolver.FaceIndex(primary), 1);
        assert!(resolver.Variations(primary).is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(resolver.Resolve(&request) as *mut Font, resolved);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(resolver.catalog.faces[0].prepared.get().is_none());
        assert!(resolver.catalog.faces[2].prepared.get().is_none());
        assert_eq!(
            resolver.sources[0]
                .fonts
                .iter()
                .filter(|entry| entry.get().is_some())
                .count(),
            1
        );
    }

    #[test]
    fn fallback_is_initialized_once_on_demand_and_survives_resolver() {
        let _heap = foundation::LayoutHeapScope::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let faces = vec![
            face("requested", 'A' as u32, FontFallbackRole::kFamilyOnly),
            face("fallback", 'B' as u32, FontFallbackRole::kText),
        ];
        let mut resolver =
            FontResolver::new(&faces, Some(Arc::new(CountingFactory(calls.clone()))));
        let families = vec!["requested".to_owned()];
        let resolved = resolver.Resolve(&request(&families)).clone();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let fallback = resolved.FontDataAt(1);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert_eq!(resolved.FontDataAt(1), fallback);
        assert_eq!(resolver.FaceIndex(fallback), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let retained = resolved.CloneWithDescription(resolved.GetFontDescription().clone());
        drop(resolver);
        drop(faces);
        assert_eq!(retained.FontDataAt(1), fallback);
        assert_ne!(unsafe { &*fallback }.GlyphForCharacter('B' as u32), 0);
    }

    #[test]
    fn full_description_and_explicit_variations_remain_in_resolved_cache_key() {
        let _heap = foundation::LayoutHeapScope::new();
        let faces = vec![face("requested", 'A' as u32, FontFallbackRole::kFamilyOnly)];
        let mut resolver = FontResolver::new(&faces, None);
        let families = vec!["requested".to_owned()];
        let mut description = FontDescription::default();
        description.SetSpecifiedSize(16.0);
        let mut settings = crate::FontSettings::default();
        settings.Append(crate::FontVariationAxis::new(OPTICAL_SIZE_TAG, 27.0));
        settings.Append(crate::FontVariationAxis::new(WEIGHT_TAG, 650.0));
        description.SetVariationSettings(Some(Arc::new(settings)));
        description.SetFontSmoothing(FontSmoothingMode::kAntialiased);
        let first =
            resolver.Resolve(&request(&families).WithBaseDescription(&description)) as *mut Font;
        let primary = unsafe { &*first }.PrimaryFont();
        assert_eq!(
            unsafe { &*first }.GetFontDescription().VariationSettings(),
            description.VariationSettings()
        );
        assert_eq!(
            resolver.Variations(primary),
            &[
                FontVariation {
                    tag: OPTICAL_SIZE_TAG,
                    value: 27.0
                },
                FontVariation {
                    tag: WEIGHT_TAG,
                    value: 650.0
                },
            ]
        );
        let mut features = crate::FontSettings::default();
        features.Append(crate::FontFeature::new(u32::from_be_bytes(*b"liga"), 0));
        description.SetFeatureSettings(Some(Arc::new(features)));
        let second =
            resolver.Resolve(&request(&families).WithBaseDescription(&description)) as *mut Font;
        assert_ne!(first, second);
        assert_eq!(
            unsafe { &*second }.GetFontDescription().FeatureSettings(),
            description.FeatureSettings()
        );
    }

    #[test]
    fn matching_uses_css_directional_weight_before_catalog_order() {
        assert!(FontWeightRank(500.0, 400.0) < FontWeightRank(300.0, 400.0));
        assert!(FontWeightRank(500.0, 450.0) < FontWeightRank(400.0, 450.0));
        assert!(FontWeightRank(800.0, 600.0) < FontWeightRank(500.0, 600.0));
        assert!(FontWeightRank(200.0, 300.0) < FontWeightRank(400.0, 300.0));
    }

    #[test]
    fn explicit_axes_override_css_and_catalog_defaults() {
        let mut description = FontDescription::default();
        let mut settings = crate::FontSettings::default();
        settings.Append(crate::FontVariationAxis::new(OPTICAL_SIZE_TAG, 31.0));
        settings.Append(crate::FontVariationAxis::new(WEIGHT_TAG, 720.0));
        description.SetVariationSettings(Some(Arc::new(settings)));
        let resolved = ResolveFontVariations(
            &[FontVariation {
                tag: OPTICAL_SIZE_TAG,
                value: 12.0,
            }],
            true,
            true,
            18.0,
            500.0,
            &description,
        );
        assert_eq!(
            resolved,
            vec![
                FontVariation {
                    tag: OPTICAL_SIZE_TAG,
                    value: 31.0
                },
                FontVariation {
                    tag: WEIGHT_TAG,
                    value: 720.0
                },
            ]
        );
    }

    #[test]
    fn size_adjust_reads_only_primary_and_reuses_admitted_native_face() {
        let _heap = foundation::LayoutHeapScope::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let faces = vec![
            face("requested", 'A' as u32, FontFallbackRole::kFamilyOnly),
            FontFace {
                family: "unopened".to_owned(),
                bytes: vec![0xde].into(),
                fallback_role: FontFallbackRole::kText,
                ..Default::default()
            },
        ];
        let mut resolver =
            FontResolver::new(&faces, Some(Arc::new(CountingFactory(calls.clone()))));
        let families = vec!["requested".to_owned()];
        let mut description = FontDescription::default();
        description.SetSpecifiedSize(16.0);
        description.SetSizeAdjust(&crate::FontSizeAdjust::new(0.75));
        let font = resolver.Resolve(&request(&families).WithBaseDescription(&description));
        assert_eq!(font.GetFontDescription().AdjustedSize(), 24.0);
        assert_eq!(unsafe { &*font.PrimaryFont() }.PlatformData().size(), 24.0);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(resolver.catalog.faces[1].prepared.get().is_none());
    }

    #[test]
    fn missing_family_and_cluster_emit_only_pure_host_queries() {
        let _heap = foundation::LayoutHeapScope::new();
        let faces = vec![face("bootstrap", 'A' as u32, FontFallbackRole::kFamilyOnly)];
        let mut resolver = FontResolver::new(&faces, None);
        let queries = FontQueryCollector::default();
        resolver.SetQueryCollector(Some(queries.clone()));
        let families = vec!["missing-family".to_owned(), "sans-serif".to_owned()];
        let font = resolver.Resolve(&request(&families)).clone();
        let family_queries = queries.Drain();
        assert_eq!(family_queries.len(), 2);
        assert!(
            matches!(&family_queries[0], SystemFontQuery::MatchFamily { family, .. } if family == "missing-family")
        );
        assert!(
            matches!(&family_queries[1], SystemFontQuery::MatchGeneric { generic, .. } if generic == "sans-serif")
        );
        let replacement = FontQueryCollector::default();
        resolver.SetQueryCollector(Some(replacement.clone()));
        font.ReportMissing(
            &[0xd83d, 0xde00, 0x200d, 0xd83d, 0xde80],
            crate::FontFallbackPriority::kEmojiEmoji,
        );
        assert!(queries.Drain().is_empty());
        assert_eq!(
            replacement.Drain(),
            vec![SystemFontQuery::Fallback {
                text: vec![0xd83d, 0xde00, 0x200d, 0xd83d, 0xde80],
                weight: 400.0,
                italic: false,
                locale: "en".to_owned(),
                presentation: FontPresentation::Emoji,
                base_face: None,
            }]
        );
    }

    #[test]
    fn admitted_family_still_requests_new_weight_style_and_locale() {
        let _heap = foundation::LayoutHeapScope::new();
        let native = face("requested", 'A' as u32, FontFallbackRole::kFamilyOnly);
        let queries = FontQueryCollector::default();
        queries.RegisterFace(&native, crate::SystemFontFaceId("native-regular".into()));
        let mut resolver = FontResolver::new(&[native], None);
        resolver.SetQueryCollector(Some(queries.clone()));
        let families = vec!["requested".to_owned()];
        resolver.Resolve(&request(&families));
        assert!(matches!(
            queries.Drain().as_slice(),
            [SystemFontQuery::MatchFamily {
                weight: 400.0,
                italic: false,
                ..
            }]
        ));
        let mut bold = request(&families);
        bold.weight = 700.;
        bold.italic = true;
        bold.language = "ja";
        resolver.Resolve(&bold);
        assert_eq!(
            queries.Drain(),
            vec![SystemFontQuery::MatchFamily {
                family: "requested".into(),
                weight: 700.,
                italic: true,
                locale: "ja".into(),
            }]
        );
        let owned = face("owned-web-face", 'A' as u32, FontFallbackRole::kFamilyOnly);
        let mut resolver = FontResolver::new(&[owned], None);
        resolver.SetQueryCollector(Some(queries.clone()));
        resolver.Resolve(&request(&["owned-web-face".to_owned()]));
        assert!(
            queries.Drain().is_empty(),
            "a CSS-owned family does not discover an unrelated native alias"
        );
    }

    #[test]
    fn exact_locale_match_wins_shared_alias_without_overriding_css_faces() {
        let _heap = foundation::LayoutHeapScope::new();
        let en = face("sans-serif", 'A' as u32, FontFallbackRole::kFamilyOnly);
        let ja = face("sans-serif", 'B' as u32, FontFallbackRole::kFamilyOnly);
        let queries = FontQueryCollector::default();
        for (native, locale) in [(&en, "en"), (&ja, "ja")] {
            let id = crate::SystemFontFaceId(locale.into());
            queries.RegisterFace(native, id.clone());
            queries.RegisterMatches(
                &SystemFontQuery::MatchGeneric {
                    generic: "sans-serif".into(),
                    weight: 400.,
                    italic: false,
                    locale: locale.into(),
                },
                &[crate::SystemFontFace {
                    id,
                    face: FontFace::default(),
                }],
            );
        }
        let mut resolver = FontResolver::new(&[en.clone(), ja.clone()], None);
        resolver.SetQueryCollector(Some(queries.clone()));
        let families = vec!["sans-serif".to_owned()];
        for (locale, index) in [("ja", 1), ("en", 0)] {
            let mut request = request(&families);
            request.language = locale;
            let primary = resolver.Resolve(&request).PrimaryFont();
            assert_eq!(
                resolver.FaceIndex(primary),
                index,
                "the exact locale's native answer wins"
            );
        }
        let css = face("sans-serif", 'C' as u32, FontFallbackRole::kFamilyOnly);
        let mut resolver = FontResolver::new(&[en, ja, css], None);
        resolver.SetQueryCollector(Some(queries));
        let mut request = request(&families);
        request.language = "ja";
        let primary = resolver.Resolve(&request).PrimaryFont();
        assert_eq!(
            resolver.FaceIndex(primary),
            2,
            "CSS-owned faces retain priority"
        );
    }

    #[test]
    fn shaping_admits_only_the_used_primary_and_fallback_faces() {
        let _heap = foundation::LayoutHeapScope::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let faces = vec![
            face("requested", 'A' as u32, FontFallbackRole::kFamilyOnly),
            face("fallback", 'B' as u32, FontFallbackRole::kText),
            FontFace {
                family: "unused-system".to_owned(),
                bytes: vec![0xde].into(),
                fallback_role: FontFallbackRole::kText,
                ..Default::default()
            },
        ];
        let mut resolver =
            FontResolver::new(&faces, Some(Arc::new(CountingFactory(calls.clone()))));
        let families = vec!["requested".to_owned()];
        let font = resolver.Resolve(&request(&families)) as *const Font;
        let shaper = crate::HarfBuzzShaper::new(foundation::String::from_utf16(&['A' as u16]));
        let result = shaper.Shape(font, foundation::TextDirection::kLtr);
        assert_eq!(unsafe { &*result }.NumGlyphs(), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(resolver.catalog.faces[1].prepared.get().is_none());
        assert!(resolver.catalog.faces[2].prepared.get().is_none());
        let shaper = crate::HarfBuzzShaper::new(foundation::String::from_utf16(&['B' as u16]));
        let result = shaper.Shape(font, foundation::TextDirection::kLtr);
        let result = unsafe { &*result };
        assert_eq!(result.NumGlyphs(), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let run = unsafe { &*result.RunsOrParts()[0].Get() };
        assert_ne!(run.glyph_data_[0].glyph(), 0);
        assert_eq!(resolver.FaceIndex(run.font_data_.Get()), 1);
        assert!(resolver.catalog.faces[2].prepared.get().is_none());
    }
}
