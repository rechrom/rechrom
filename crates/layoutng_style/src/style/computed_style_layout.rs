use std::sync::OnceLock;

use font_engine::Hyphenation;
use foundation::{
    gfx, AtomicString, CSSPropertyID, Color, EInsideLink, EPosition, ETextTransform,
    ETransformStyle3D, FloatValueForLength, Hyphens, MakeGarbageCollected, Member, PointAndTangent,
    PointForLengthPoint, String, TextOffsetMap,
};

use super::applied_text_decoration::{AppliedTextDecoration, AppliedTextDecorationVector};
use super::basic_shapes::ShapeType;
use super::computed_style::ComputedStyle;
use super::computed_style_base::ComputedStyleBase;
use super::computed_style_constants::{EFillAttachment, EFillBox, OffsetRotationType, PseudoId};
use super::fill_layer::FillLayer;
use super::forward::Element;
use super::gap_data_list::GapDataList;
use super::offset_path_operation::{OperationType, ShapeOffsetPathOperation};
use super::style_cached_data::StyleCachedData;
use super::style_ray::StyleRay;
use super::style_transform::MotionPathBox;
use crate::css::style_color::StyleColor;
use crate::text_transform_services::CurrentNativeTextTransformResolver;

// cpp: layoutng_style/style/computed_style_layout.cc:34-48
#[allow(non_snake_case)]
fn ApplyMathAutoTransform(text: &String, offset_map: Option<&mut TextOffsetMap>) -> String {
    let mut source_units = text.Span16().unwrap_or_default().iter().copied();
    let original = match (source_units.next(), source_units.next()) {
        (Some(unit), None) => unit,
        _ => return text.clone(),
    };
    let transformed_char = foundation::ItalicMathVariant(u32::from(original));
    if transformed_char == u32::from(original) {
        return text.clone();
    }
    let character = char::from_u32(transformed_char).expect("ItalicMathVariant returns a scalar");
    let mut units = [0u16; 2];
    let transformed_text = character.encode_utf16(&mut units);
    let result = String::from_utf16(transformed_text);
    if let Some(offset_map) = offset_map {
        offset_map.Append(1, result.length());
    }
    result
}

#[allow(non_snake_case)]
impl ComputedStyle {
    // cpp: layoutng_style/style/computed_style.h:320
    // cpp: layoutng_style/style/computed_style_layout.cc:52-56
    pub fn EnsureCachedData(&self) -> &mut StyleCachedData {
        if self.cached_data_.get().Get().is_null() {
            self.cached_data_.set(Member::from_ptr(MakeGarbageCollected(
                StyleCachedData::default(),
            )));
        }
        unsafe { &mut *self.cached_data_.get().Get() }
    }

    // cpp: layoutng_style/style/computed_style.h:2798-2799
    // cpp: layoutng_style/style/computed_style_layout.cc:58-63
    pub fn GetCurrentColor(&self, is_current_color: Option<&mut bool>) -> Color {
        debug_assert!(!self.Color().IsCurrentColor());
        if let Some(is_current_color) = is_current_color {
            *is_current_color = self.ColorIsCurrentColor();
        }
        self.Color()
            .Resolve(Color::default(), self.UsedColorScheme(), None)
    }

    // cpp: layoutng_style/style/computed_style.h:2800-2801
    // cpp: layoutng_style/style/computed_style_layout.cc:65-71
    pub fn GetInternalVisitedCurrentColor(&self, is_current_color: Option<&mut bool>) -> Color {
        debug_assert!(!self.InternalVisitedColor().IsCurrentColor());
        if let Some(is_current_color) = is_current_color {
            *is_current_color = self.InternalVisitedColorIsCurrentColor();
        }
        self.InternalVisitedColor()
            .Resolve(Color::default(), self.UsedColorScheme(), None)
    }

    // cpp: layoutng_style/style/computed_style.h:453
    // cpp: layoutng_style/style/computed_style_layout.cc:73-77
    pub fn GetBaseComputedStyle(&self) -> *const ComputedStyle {
        let base_data = self.BaseData();
        if !base_data.is_null() {
            return unsafe { (&*base_data).GetBaseComputedStyle() };
        }
        std::ptr::null()
    }

    // cpp: layoutng_style/style/computed_style.h:1239-1242
    // cpp: layoutng_style/style/computed_style_layout.cc:79-98
    pub fn ApplyTextTransform(
        &self,
        text: &String,
        previous_character: u16,
        offset_map: Option<&mut TextOffsetMap>,
    ) -> String {
        if self.TextTransform() == ETextTransform::kNone {
            return text.clone();
        }
        let resolver = CurrentNativeTextTransformResolver();
        let Some(resolver) = resolver else {
            return text.clone();
        };
        if self.TextTransform() == ETextTransform::kMathAuto {
            return ApplyMathAutoTransform(text, offset_map);
        }
        unsafe { resolver.as_ref() }.Transform(
            self.TextTransform(),
            self.Locale(),
            text,
            previous_character,
            offset_map,
        )
    }

    // cpp: layoutng_style/style/computed_style.h:1223
    // cpp: layoutng_style/style/computed_style_layout.cc:100-110
    pub fn GetHyphenationWithLimits(&self) -> *mut Hyphenation {
        if self.GetHyphens() != Hyphens::kAuto {
            return std::ptr::null_mut();
        }
        let hyphenation = self.GetHyphenation();
        if hyphenation.is_null() {
            return std::ptr::null_mut();
        }
        let limits = self.HyphenateLimitChars();
        unsafe {
            (&*hyphenation).SetLimits(
                limits.MinBeforeChars(),
                limits.MinAfterChars(),
                limits.MinWordChars(),
            );
        }
        hyphenation
    }

    // cpp: layoutng_style/style/computed_style.h:1222
    // cpp: layoutng_style/style/computed_style_layout.cc:112-115
    pub fn GetHyphenation(&self) -> *mut Hyphenation {
        let locale = self.GetFontDescription().Locale();
        if locale.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { (&*locale).GetHyphenation() }
        }
    }

    // cpp: layoutng_style/style/computed_style.h:1224
    // cpp: layoutng_style/style/computed_style_layout.cc:117-131
    pub fn HyphenString(&self) -> &AtomicString {
        let specified = self.HyphenationString();
        if !specified.IsNull() {
            return specified;
        }
        static HYPHEN_MINUS: OnceLock<AtomicString> = OnceLock::new();
        static HYPHEN: OnceLock<AtomicString> = OnceLock::new();
        let hyphen_minus = HYPHEN_MINUS.get_or_init(|| AtomicString::from_utf16(&[0x002D]));
        let hyphen = HYPHEN.get_or_init(|| AtomicString::from_utf16(&[0x2010]));
        let primary_font = unsafe { (&*self.GetFont()).PrimaryFont() };
        debug_assert!(!primary_font.is_null());
        if !primary_font.is_null() && unsafe { (&*primary_font).GlyphForCharacter(0x2010) != 0 } {
            hyphen
        } else {
            hyphen_minus
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2398
    // cpp: core/style/computed_style.cc:2499-2520
    pub fn HasBackground(&self) -> bool {
        if !self
            .BackgroundColor()
            .Resolve(self.GetCurrentColor(None), self.UsedColorScheme(), None)
            .IsFullyTransparent()
        {
            return true;
        }
        // Composite background-color animation state is not represented yet;
        // the ordinary native image predicate is the final Chromium branch.
        self.HasBackgroundImage()
    }

    // cpp: layoutng_style/style/computed_style.h:722
    // cpp: layoutng_style/style/computed_style_layout.cc:142-146
    pub fn HasCustomScrollbarStyle(&self, _element: *mut Element) -> bool {
        false
    }

    // cpp: layoutng_style/style/computed_style.h:1210
    // cpp: layoutng_style/style/computed_style_layout.cc:148-151
    pub fn HasWillChangeProperty(&self, id: CSSPropertyID) -> bool {
        debug_assert!(id != CSSPropertyID::kInvalid);
        let will_change = self.WillChange();
        !will_change.is_null() && unsafe { (&*will_change).resolved_longhand_ids().Has(id) }
    }

    // cpp: layoutng_style/style/computed_style.h:2816-2817
    // cpp: layoutng_style/style/computed_style_layout.cc:153-159
    pub fn ResolvedColor(&self, color: &StyleColor, is_current_color: Option<&mut bool>) -> Color {
        let visited = self.InsideLink() == EInsideLink::kInsideVisitedLink;
        let current = if visited {
            self.GetInternalVisitedCurrentColor(None)
        } else {
            self.GetCurrentColor(None)
        };
        color.Resolve(current, self.UsedColorScheme(), is_current_color)
    }

    // cpp: layoutng_style/style/computed_style.h:2872-2873
    // cpp: layoutng_style/style/computed_style_layout.cc:161-175
    pub fn GapRuleColorIsTransparent(&self, colors: &GapDataList<StyleColor>) -> bool {
        let current = self.GetCurrentColor(None);
        let scheme = self.UsedColorScheme();
        colors.GetGapDataList().iter().all(|data| {
            if !data.IsRepeaterData() {
                return data
                    .GetValue()
                    .Resolve(current.clone(), scheme, None)
                    .IsFullyTransparent();
            }
            data.GetValueRepeater()
                .RepeatedValues()
                .iter()
                .all(|value| {
                    value
                        .Resolve(current.clone(), scheme, None)
                        .IsFullyTransparent()
                })
        })
    }

    // cpp: layoutng_style/style/computed_style.h:329-330
    // cpp: layoutng_style/style/computed_style_layout.cc:177-193
    pub fn EnsureAppliedTextDecorationsCache(&self) -> *mut AppliedTextDecorationVector {
        debug_assert!(self.IsDecoratingBox());
        let cache = self.cached_data_.get().Get();
        if cache.is_null() || unsafe { (&*cache).applied_text_decorations_.Get().is_null() } {
            let decorations = MakeGarbageCollected(AppliedTextDecorationVector::default());
            let base = self.BaseTextDecorationData();
            if !base.is_null() {
                unsafe {
                    (&mut *decorations).ReserveInitialCapacity((&*base).size() + 1);
                    *decorations = (&*base).clone();
                }
            }
            unsafe {
                (&mut *decorations).emplace_back(AppliedTextDecoration::new(
                    self.GetTextDecorationLine(),
                    self.TextDecorationStyle(),
                    self.ResolvedColor(self.TextDecorationColor(), None),
                    self.GetTextDecorationThickness().clone(),
                    self.TextUnderlineOffset().clone(),
                    self.GetTextDecorationInset().clone(),
                    self.BoxDecorationBreak(),
                ));
            }
            self.EnsureCachedData().applied_text_decorations_ = Member::from_ptr(decorations);
        }
        unsafe {
            (&*self.cached_data_.get().Get())
                .applied_text_decorations_
                .Get()
        }
    }

    // cpp: layoutng_style/style/computed_style.h:2870
    // cpp: layoutng_style/style/computed_style_layout.cc:195-207
    pub fn CalculateIsStackingContextWithoutContainment(&self) -> bool {
        if self.TransformStyle3D() == ETransformStyle3D::kPreserve3d
            || self.ForcesStackingContext()
            || self.StyleType() == PseudoId::kPseudoIdBackdrop
            || self.HasTransformRelatedProperty()
            || self.HasStackingGroupingProperty(self.HasBoxReflect())
            || self.GetPosition() == EPosition::kFixed
            || self.GetPosition() == EPosition::kSticky
        {
            return true;
        }
        // cpp: core/style/computed_style.cc:1301-1339,2941-2949
        let will_change = self.WillChange();
        if !will_change.is_null() {
            let properties = unsafe { &*will_change }.resolved_longhand_ids();
            let creates_context = [
                CSSPropertyID::kOpacity,
                CSSPropertyID::kTransform,
                CSSPropertyID::kTransformStyle,
                CSSPropertyID::kPerspective,
                CSSPropertyID::kTranslate,
                CSSPropertyID::kRotate,
                CSSPropertyID::kScale,
                CSSPropertyID::kOffsetPath,
                CSSPropertyID::kOffsetPosition,
                CSSPropertyID::kMaskImage,
                CSSPropertyID::kWebkitMaskBoxImageSource,
                CSSPropertyID::kClipPath,
                CSSPropertyID::kWebkitBoxReflect,
                CSSPropertyID::kFilter,
                CSSPropertyID::kBackdropFilter,
                CSSPropertyID::kPosition,
                CSSPropertyID::kMixBlendMode,
                CSSPropertyID::kIsolation,
                CSSPropertyID::kContain,
                CSSPropertyID::kViewTransitionName,
            ]
            .into_iter()
            .any(|id| properties.Has(id));
            if creates_context || (self.AllowsZIndex() && properties.Has(CSSPropertyID::kZIndex)) {
                return true;
            }
        }
        self.ShouldCompositeForCurrentAnimations()
    }

    // cpp: layoutng_style/style/computed_style.h:2766-2770
    // cpp: layoutng_style/style/computed_style_layout.cc:220-228
    pub fn ApplyMotionPathTransform<B: MotionPathBox>(
        &self,
        origin_x: f32,
        origin_y: f32,
        box_: Option<&B>,
        bounding_box: &gfx::RectF,
        transform: &mut gfx::Transform,
    ) {
        let Some(offset_path) = self.OffsetPath().GetNonNull() else {
            return;
        };
        let offset_path = unsafe { offset_path.as_ref() };
        let Some(box_) = box_ else {
            return;
        };

        let reference_box = box_.MotionPathReferenceBox(offset_path.GetCoordBox());
        let offset_from_containing_block = box_.MotionPathOffsetFromContainingBlock();
        let offset_from_reference_box = gfx::PointF::new(
            offset_from_containing_block.x() - reference_box.x(),
            offset_from_containing_block.y() - reference_box.y(),
        );
        let reference_size = reference_box.size();

        let starting_point = |position: &foundation::LengthPoint| {
            if position.X().IsAuto() {
                offset_from_reference_box
            } else if position.X().IsNone() {
                gfx::PointF::new(reference_size.width() / 2.0, reference_size.height() / 2.0)
            } else {
                PointForLengthPoint(position, &reference_size)
            }
        };

        let mut path_position = match offset_path.GetType() {
            OperationType::kShape => {
                let shape_operation = unsafe {
                    &*(offset_path as *const dyn super::offset_path_operation::OffsetPathOperation
                        as *const ShapeOffsetPathOperation)
                };
                let shape = shape_operation.GetBasicShape();
                match shape.GetType() {
                    ShapeType::kStyleRayType => {
                        let ray = unsafe {
                            &*(shape as *const dyn super::basic_shapes::BasicShape
                                as *const StyleRay)
                        };
                        let start = if ray.HasExplicitCenter() || self.OffsetPosition().X().IsNone()
                        {
                            PointForLengthPoint(ray.Center(), &reference_size)
                        } else {
                            starting_point(self.OffsetPosition())
                        };
                        let mut ray_length = ray.CalculateRayPathLength(&start, &reference_size);
                        if ray.Contain() {
                            let element_size = box_.MotionPathBorderBoxSize();
                            ray_length = (ray_length
                                - element_size.width().max(element_size.height()) / 2.0)
                                .max(0.0);
                        }
                        let distance = FloatValueForLength(self.OffsetDistance(), ray_length);
                        ray.PointAndNormalAtLength(&start, distance)
                    }
                    ShapeType::kStylePathType => {
                        let mut position = box_
                            .MotionPathPointAndTangent(
                                shape,
                                &gfx::PointF::default(),
                                &reference_size,
                                self.OffsetDistance(),
                                self.EffectiveZoom(),
                            )
                            .unwrap_or(PointAndTangent {
                                point: gfx::PointF::default(),
                                tangent_in_degrees: 0.0,
                            });
                        // Unlike reference-box basic shapes, path() coordinates
                        // are local to the transformed element.  Compensate the
                        // common containing-block conversion below.
                        position.point = gfx::PointF::new(
                            position.point.x() + offset_from_reference_box.x(),
                            position.point.y() + offset_from_reference_box.y(),
                        );
                        position
                    }
                    ShapeType::kBasicShapeCircleType
                    | ShapeType::kBasicShapeEllipseType
                    | ShapeType::kBasicShapeInsetType
                    | ShapeType::kBasicShapePolygonType
                    | ShapeType::kStyleShapeType => {
                        let start = starting_point(self.OffsetPosition());
                        box_.MotionPathPointAndTangent(
                            shape,
                            &start,
                            &reference_size,
                            self.OffsetDistance(),
                            self.EffectiveZoom(),
                        )
                        .unwrap_or(PointAndTangent {
                            point: gfx::PointF::default(),
                            tangent_in_degrees: 0.0,
                        })
                    }
                }
            }
            OperationType::kCoordBox => {
                if !box_.MotionPathHasContainingBlock() {
                    return;
                }
                box_.MotionPathCoordBoxPointAndTangent(&reference_size, self.OffsetDistance())
                    .unwrap_or_else(|| {
                        PointAndTangentOnRect(&reference_size, self.OffsetDistance())
                    })
            }
            // URL paths remain rejected by the production parser, so no live
            // ReferenceOffsetPathOperation can currently reach this boundary.
            OperationType::kReference => return,
        };

        // The point produced above is relative to the containing block's
        // reference box.  Chromium converts it back to the transformed box.
        path_position.point = gfx::PointF::new(
            path_position.point.x() - offset_from_reference_box.x(),
            path_position.point.y() - offset_from_reference_box.y(),
        );
        if self.OffsetRotate().r#type == OffsetRotationType::kFixed {
            path_position.tangent_in_degrees = 0.0;
        }

        transform.Translate3d(
            path_position.point.x() - origin_x,
            path_position.point.y() - origin_y,
            0.0,
        );
        transform.Rotate(f64::from(
            path_position.tangent_in_degrees + self.OffsetRotate().angle,
        ));

        let anchor = self.OffsetAnchor();
        if !anchor.X().IsAuto() {
            let mut anchor_point = PointForLengthPoint(anchor, &bounding_box.size());
            anchor_point = gfx::PointF::new(
                anchor_point.x() + bounding_box.x(),
                anchor_point.y() + bounding_box.y(),
            );
            transform.Translate3d(
                origin_x - anchor_point.x(),
                origin_y - anchor_point.y(),
                0.0,
            );
        }
    }
}

/// PointAndNormalAtLength for the zero-radius inset generated by a coordinate
/// box offset-path.  Path::MakeRoundedRect uses this clockwise contour order
/// when the containing block has the common zero border-radius case.
fn PointAndTangentOnRect(size: &gfx::SizeF, distance: &foundation::Length) -> PointAndTangent {
    let width = size.width().max(0.0);
    let height = size.height().max(0.0);
    let path_length = 2.0 * (width + height);
    if path_length <= 0.0 {
        return PointAndTangent {
            point: gfx::PointF::default(),
            tangent_in_degrees: 0.0,
        };
    }
    let mut offset = FloatValueForLength(distance, path_length) % path_length;
    if offset < 0.0 {
        offset += path_length;
    }
    if offset <= width {
        PointAndTangent {
            point: gfx::PointF::new(offset, 0.0),
            tangent_in_degrees: 0.0,
        }
    } else if offset <= width + height {
        PointAndTangent {
            point: gfx::PointF::new(width, offset - width),
            tangent_in_degrees: 90.0,
        }
    } else if offset <= 2.0 * width + height {
        PointAndTangent {
            point: gfx::PointF::new(2.0 * width + height - offset, height),
            tangent_in_degrees: 180.0,
        }
    } else {
        PointAndTangent {
            point: gfx::PointF::new(0.0, path_length - offset),
            tangent_in_degrees: -90.0,
        }
    }
}

#[allow(non_snake_case)]
impl ComputedStyleBase {
    // cpp: layoutng_style/style/computed_style_base.h:2763
    // cpp: layoutng_style/style/computed_style_layout.cc:209-218
    pub fn IsStackingContextWithoutContainment(&self) -> bool {
        let cached = self.data_.is_stacking_context_without_containment_bits();
        if cached == 0 {
            let style = unsafe { &*(self as *const Self as *const ComputedStyle) };
            let value = style.CalculateIsStackingContextWithoutContainment();
            self.data_
                .set_is_stacking_context_without_containment_bits(0b10 | (u32::from(value) & 1));
        }
        self.data_.is_stacking_context_without_containment_bits() & 1 != 0
    }
}

#[allow(non_snake_case)]
impl FillLayer {
    // cpp: layoutng_style/style/computed_style_layout.cc:230-254
    pub(crate) fn ComputeCachedProperties(&self) {
        debug_assert!(!self.cached_properties_.get().computed);
        let image = unsafe { self.GetImage().as_ref() };
        let effective_clip = self.Clip();
        let mut cache = self.cached_properties_.get();
        cache.layers_clip_max = effective_clip;
        cache.any_layer_uses_content_box =
            effective_clip == EFillBox::kContent || self.Origin() == EFillBox::kContent;
        cache.any_layer_has_image = image.is_some();
        cache.any_layer_has_url_image = image.is_some_and(|image| {
            // StyleImage::CssValue returns its live CSS value, as in Blink.
            unsafe { &*image.CssValue() }.MayContainUrl()
        });
        cache.any_layer_has_local_attachment = self.Attachment() == EFillAttachment::kLocal;
        cache.any_layer_has_fixed_attachment_image =
            cache.any_layer_has_image && self.Attachment() == EFillAttachment::kFixed;
        cache.any_layer_has_default_attachment_image =
            cache.any_layer_has_image && self.Attachment() == EFillAttachment::kScroll;
        // StyleGeneratedImage::DependsOnCurrentColor overrides the base with
        // IsUsingCurrentColor; use that real virtual dispatch after its tag check.
        cache.any_layer_uses_current_color =
            image.is_some_and(|image| image.IsGeneratedImage() && image.DependsOnCurrentColor());
        cache.computed = true;
        self.cached_properties_.set(cache);

        let next = self.Next();
        if !next.is_null() {
            let next = unsafe { &*next };
            next.ComputeCachedPropertiesIfNeeded();
            let current_clip = self.LayersClipMax();
            let next_clip = next.LayersClipMax();
            cache.layers_clip_max = if (current_clip as u32) < (next_clip as u32) {
                next_clip
            } else {
                current_clip
            };
            cache.any_layer_uses_content_box |=
                next.cached_properties_.get().any_layer_uses_content_box;
            cache.any_layer_has_image |= next.cached_properties_.get().any_layer_has_image;
            cache.any_layer_has_url_image |= next.cached_properties_.get().any_layer_has_url_image;
            cache.any_layer_has_local_attachment |=
                next.cached_properties_.get().any_layer_has_local_attachment;
            cache.any_layer_has_fixed_attachment_image |= next
                .cached_properties_
                .get()
                .any_layer_has_fixed_attachment_image;
            cache.any_layer_has_default_attachment_image |= next
                .cached_properties_
                .get()
                .any_layer_has_default_attachment_image;
            cache.any_layer_uses_current_color |=
                next.cached_properties_.get().any_layer_uses_current_color;
            self.cached_properties_.set(cache);
        }
    }
}
