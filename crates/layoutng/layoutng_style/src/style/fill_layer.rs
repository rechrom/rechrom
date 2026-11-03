use std::cell::Cell;

use foundation::{
    gfx::SizeF, BlendMode, CompositeOperator, Length, LengthSize, MakeGarbageCollected, Member,
    RespectImageOrientationEnum, ValuesEquivalent, Visitor,
};

use super::computed_style::ComputedStyle;
use super::computed_style_constants::{
    BackgroundEdgeOrigin, CompositingOperator, EFillAttachment, EFillBox, EFillLayerType,
    EFillMaskMode, EFillRepeat, EFillSizeType, EnclosingFillBox,
};
use super::style_image::StyleImage;
use super::style_mask_source_image::{MaskSourceImage, StyleMaskSourceImage};

impl foundation::Traceable for FillLayer {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FillLayer::Trace(self, visitor);
    }
}

impl foundation::Traceable for FillLayerWrapper {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        FillLayerWrapper::Trace(self, visitor);
    }
}

// cpp: layoutng_style/style/fill_layer.h:40-52
#[derive(Clone, PartialEq)]
pub struct FillSize {
    pub type_: EFillSizeType,
    pub size: LengthSize,
}

#[allow(non_snake_case)]
impl FillSize {
    // cpp: layoutng_style/style/fill_layer.h:44
    pub fn new(type_: EFillSizeType, size: &LengthSize) -> Self {
        Self {
            type_,
            size: size.clone(),
        }
    }
}

// cpp: layoutng_style/style/fill_layer.h:42
impl Default for FillSize {
    fn default() -> Self {
        Self {
            type_: EFillSizeType::kSizeLength,
            size: LengthSize::default(),
        }
    }
}

// cpp: layoutng_style/style/fill_layer.h:54-59
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FillRepeat {
    pub x: EFillRepeat,
    pub y: EFillRepeat,
}

impl Default for FillRepeat {
    fn default() -> Self {
        Self {
            x: EFillRepeat::kRepeatFill,
            y: EFillRepeat::kRepeatFill,
        }
    }
}

// cpp: layoutng_style/style/fill_layer.h:364-383
#[derive(Clone, Copy)]
pub(crate) struct FillLayerCachedProperties {
    pub(crate) layers_clip_max: EFillBox,
    pub(crate) any_layer_uses_content_box: bool,
    pub(crate) any_layer_has_image: bool,
    pub(crate) any_layer_has_url_image: bool,
    pub(crate) any_layer_has_local_attachment: bool,
    pub(crate) any_layer_has_fixed_attachment_image: bool,
    pub(crate) any_layer_has_default_attachment_image: bool,
    pub(crate) any_layer_uses_current_color: bool,
    pub(crate) computed: bool,
}

impl Default for FillLayerCachedProperties {
    fn default() -> Self {
        Self {
            layers_clip_max: EFillBox::kBorder,
            any_layer_uses_content_box: false,
            any_layer_has_image: false,
            any_layer_has_url_image: false,
            any_layer_has_local_attachment: false,
            any_layer_has_fixed_attachment_image: false,
            any_layer_has_default_attachment_image: false,
            any_layer_uses_current_color: false,
            computed: false,
        }
    }
}

// cpp: layoutng_style/style/fill_layer.h:63-67
// cpp: layoutng_style/style/fill_layer.h:317-384
pub struct FillLayer {
    pub(super) next_: Member<FillLayerWrapper>,
    pub(super) image_: Member<StyleImage>,
    pub(super) position_x_: Length,
    pub(super) position_y_: Length,
    pub(super) size_length_: LengthSize,
    pub(super) repeat_: FillRepeat,
    pub(super) attachment_: EFillAttachment,
    pub(super) clip_: EFillBox,
    pub(super) origin_: EFillBox,
    pub(super) compositing_operator_: CompositingOperator,
    pub(super) size_type_: EFillSizeType,
    pub(super) blend_mode_: BlendMode,
    pub(super) background_x_origin_: BackgroundEdgeOrigin,
    pub(super) background_y_origin_: BackgroundEdgeOrigin,
    pub(super) mask_mode_: EFillMaskMode,
    pub(super) image_set_: bool,
    pub(super) attachment_set_: bool,
    pub(super) clip_set_: bool,
    pub(super) origin_set_: bool,
    pub(super) repeat_set_: bool,
    pub(super) mask_mode_set_: bool,
    pub(super) pos_x_set_: bool,
    pub(super) pos_y_set_: bool,
    pub(super) background_x_origin_set_: bool,
    pub(super) background_y_origin_set_: bool,
    pub(super) compositing_operator_set_: bool,
    pub(super) blend_mode_set_: bool,
    pub(super) type_: EFillLayerType,
    pub(crate) cached_properties_: Cell<FillLayerCachedProperties>,
}

// cpp: core/style/fill_layer.cc:49-91
impl FillLayer {
    pub fn new_without_initial_values(type_: EFillLayerType) -> Self {
        Self::new(type_, false)
    }

    pub fn new(type_: EFillLayerType, use_initial_values: bool) -> Self {
        Self {
            next_: Member::default(),
            image_: Member::from_ptr(Self::InitialFillImage(type_)),
            position_x_: Self::InitialFillPositionX(type_),
            position_y_: Self::InitialFillPositionY(type_),
            size_length_: Self::InitialFillSizeLength(type_),
            repeat_: Self::InitialFillRepeat(type_),
            attachment_: Self::InitialFillAttachment(type_),
            clip_: Self::InitialFillClip(type_),
            origin_: Self::InitialFillOrigin(type_),
            compositing_operator_: Self::InitialFillCompositingOperator(type_),
            size_type_: if use_initial_values {
                Self::InitialFillSizeType(type_)
            } else {
                EFillSizeType::kSizeNone
            },
            blend_mode_: Self::InitialFillBlendMode(type_),
            background_x_origin_: BackgroundEdgeOrigin::kLeft,
            background_y_origin_: BackgroundEdgeOrigin::kTop,
            mask_mode_: Self::InitialFillMaskMode(type_),
            image_set_: use_initial_values,
            attachment_set_: use_initial_values,
            clip_set_: use_initial_values,
            origin_set_: use_initial_values,
            repeat_set_: use_initial_values,
            mask_mode_set_: use_initial_values,
            pos_x_set_: use_initial_values,
            pos_y_set_: use_initial_values,
            background_x_origin_set_: false,
            background_y_origin_set_: false,
            compositing_operator_set_: use_initial_values || type_ == EFillLayerType::kMask,
            blend_mode_set_: use_initial_values,
            type_: type_,
            cached_properties_: Cell::new(FillLayerCachedProperties::default()),
        }
    }

    // cpp: core/style/fill_layer.cc:132-135
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.next_);
        visitor.Trace(&self.image_);
    }
}

// cpp: core/style/fill_layer.cc:93-130
impl Clone for FillLayer {
    fn clone(&self) -> Self {
        let next = self.next_.Get();
        Self {
            next_: if next.is_null() {
                Member::default()
            } else {
                Member::from_ptr(MakeGarbageCollected(unsafe { (&*next).clone() }))
            },
            image_: self.image_.clone(),
            position_x_: self.position_x_.clone(),
            position_y_: self.position_y_.clone(),
            size_length_: self.size_length_.clone(),
            repeat_: self.repeat_,
            attachment_: self.attachment_,
            clip_: self.clip_,
            origin_: self.origin_,
            compositing_operator_: self.compositing_operator_,
            size_type_: self.size_type_,
            blend_mode_: self.blend_mode_,
            background_x_origin_: self.background_x_origin_,
            background_y_origin_: self.background_y_origin_,
            mask_mode_: self.mask_mode_,
            image_set_: self.image_set_,
            attachment_set_: self.attachment_set_,
            clip_set_: self.clip_set_,
            origin_set_: self.origin_set_,
            repeat_set_: self.repeat_set_,
            mask_mode_set_: self.mask_mode_set_,
            pos_x_set_: self.pos_x_set_,
            pos_y_set_: self.pos_y_set_,
            background_x_origin_set_: self.background_x_origin_set_,
            background_y_origin_set_: self.background_y_origin_set_,
            compositing_operator_set_: self.compositing_operator_set_,
            blend_mode_set_: self.blend_mode_set_,
            type_: self.type_,
            cached_properties_: Cell::new(FillLayerCachedProperties::default()),
        }
    }
}

#[allow(non_snake_case)]
impl FillLayer {
    // cpp: layoutng_style/style/fill_layer.h:71
    pub fn GetImage(&self) -> *mut StyleImage {
        self.image_.Get()
    }

    // cpp: layoutng_style/style/fill_layer.h:72
    pub fn PositionX(&self) -> &Length {
        &self.position_x_
    }

    // cpp: layoutng_style/style/fill_layer.h:73
    pub fn PositionY(&self) -> &Length {
        &self.position_y_
    }

    // cpp: layoutng_style/style/fill_layer.h:74-76
    pub fn BackgroundXOrigin(&self) -> BackgroundEdgeOrigin {
        self.background_x_origin_
    }

    // cpp: layoutng_style/style/fill_layer.h:77-79
    pub fn BackgroundYOrigin(&self) -> BackgroundEdgeOrigin {
        self.background_y_origin_
    }

    // cpp: layoutng_style/style/fill_layer.h:80-82
    pub fn Attachment(&self) -> EFillAttachment {
        self.attachment_
    }

    // cpp: layoutng_style/style/fill_layer.h:83
    pub fn Clip(&self) -> EFillBox {
        self.clip_
    }

    // cpp: layoutng_style/style/fill_layer.h:84
    pub fn Origin(&self) -> EFillBox {
        self.origin_
    }

    // cpp: layoutng_style/style/fill_layer.h:85
    pub fn Repeat(&self) -> &FillRepeat {
        &self.repeat_
    }

    // cpp: layoutng_style/style/fill_layer.h:86-88
    pub fn MaskMode(&self) -> EFillMaskMode {
        self.mask_mode_
    }

    // cpp: layoutng_style/style/fill_layer.h:89-91
    pub fn CompositingOperator(&self) -> CompositingOperator {
        self.compositing_operator_
    }

    // cpp: layoutng_style/style/fill_layer.h:92
    // cpp: third_party/blink/renderer/core/style/fill_layer.cc:514-553
    pub fn Composite(&self) -> CompositeOperator {
        match self.CompositingOperator() {
            CompositingOperator::kAdd | CompositingOperator::kSourceOver => {
                CompositeOperator::kCompositeSourceOver
            }
            CompositingOperator::kSubtract | CompositingOperator::kSourceOut => {
                CompositeOperator::kCompositeSourceOut
            }
            CompositingOperator::kIntersect | CompositingOperator::kSourceIn => {
                CompositeOperator::kCompositeSourceIn
            }
            CompositingOperator::kExclude | CompositingOperator::kXOR => {
                CompositeOperator::kCompositeXOR
            }
            CompositingOperator::kClear => CompositeOperator::kCompositeClear,
            CompositingOperator::kCopy => CompositeOperator::kCompositeCopy,
            CompositingOperator::kSourceAtop => CompositeOperator::kCompositeSourceAtop,
            CompositingOperator::kDestinationOver => CompositeOperator::kCompositeDestinationOver,
            CompositingOperator::kDestinationIn => CompositeOperator::kCompositeDestinationIn,
            CompositingOperator::kDestinationOut => CompositeOperator::kCompositeDestinationOut,
            CompositingOperator::kDestinationAtop => CompositeOperator::kCompositeDestinationAtop,
            CompositingOperator::kPlusLighter => CompositeOperator::kCompositePlusLighter,
        }
    }

    // cpp: layoutng_style/style/fill_layer.h:93
    pub fn GetBlendMode(&self) -> BlendMode {
        self.blend_mode_
    }

    // cpp: layoutng_style/style/fill_layer.h:94
    pub fn SizeLength(&self) -> &LengthSize {
        &self.size_length_
    }

    // cpp: layoutng_style/style/fill_layer.h:95-97
    pub fn SizeType(&self) -> EFillSizeType {
        self.size_type_
    }

    // cpp: layoutng_style/style/fill_layer.h:98-100
    pub fn Size(&self) -> FillSize {
        FillSize::new(self.size_type_, &self.size_length_)
    }

    // cpp: layoutng_style/style/fill_layer.h:102
    // cpp: layoutng_style/style/fill_layer.h:394-396
    pub fn Next(&self) -> *const FillLayer {
        let next = self.next_.Get();
        if next.is_null() {
            std::ptr::null()
        } else {
            unsafe { &(*next).layer }
        }
    }

    // cpp: layoutng_style/style/fill_layer.h:103
    // cpp: layoutng_style/style/fill_layer.h:397-399
    pub fn NextMut(&mut self) -> *mut FillLayer {
        let next = self.next_.Get();
        if next.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &mut (*next).layer }
        }
    }

    // cpp: layoutng_style/style/fill_layer.h:104
    // cpp: layoutng_style/style/fill_layer.h:400-405
    pub fn EnsureNext(&mut self) -> *mut FillLayer {
        if self.next_.Get().is_null() {
            self.next_ =
                Member::from_ptr(MakeGarbageCollected(FillLayerWrapper::new(self.GetType())));
        }
        unsafe { &mut (*self.next_.Get()).layer }
    }

    // cpp: layoutng_style/style/fill_layer.h:106-118
    pub fn IsImageSet(&self) -> bool {
        self.image_set_
    }
    pub fn IsPositionXSet(&self) -> bool {
        self.pos_x_set_
    }
    pub fn IsPositionYSet(&self) -> bool {
        self.pos_y_set_
    }
    pub fn IsBackgroundXOriginSet(&self) -> bool {
        self.background_x_origin_set_
    }
    pub fn IsBackgroundYOriginSet(&self) -> bool {
        self.background_y_origin_set_
    }
    pub fn IsAttachmentSet(&self) -> bool {
        self.attachment_set_
    }
    pub fn IsClipSet(&self) -> bool {
        self.clip_set_
    }
    pub fn IsOriginSet(&self) -> bool {
        self.origin_set_
    }
    pub fn IsRepeatSet(&self) -> bool {
        self.repeat_set_
    }
    pub fn IsMaskModeSet(&self) -> bool {
        self.mask_mode_set_
    }
    pub fn IsCompositingOperatorSet(&self) -> bool {
        self.compositing_operator_set_
    }
    pub fn IsBlendModeSet(&self) -> bool {
        self.blend_mode_set_
    }

    // cpp: layoutng_style/style/fill_layer.h:119-121
    pub fn IsSizeSet(&self) -> bool {
        self.size_type_ != EFillSizeType::kSizeNone
    }

    // cpp: layoutng_style/style/fill_layer.h:123-126
    pub fn SetImage(&mut self, image: *mut StyleImage) {
        self.image_ = Member::from_ptr(image);
        self.image_set_ = true;
    }

    // cpp: layoutng_style/style/fill_layer.h:127-132
    pub fn SetPositionX(&mut self, position: &Length) {
        self.position_x_ = position.clone();
        self.pos_x_set_ = true;
        self.background_x_origin_set_ = false;
        self.background_x_origin_ = BackgroundEdgeOrigin::kLeft;
    }

    // cpp: layoutng_style/style/fill_layer.h:133-138
    pub fn SetPositionY(&mut self, position: &Length) {
        self.position_y_ = position.clone();
        self.pos_y_set_ = true;
        self.background_y_origin_set_ = false;
        self.background_y_origin_ = BackgroundEdgeOrigin::kTop;
    }

    // cpp: layoutng_style/style/fill_layer.h:139-142
    pub fn SetBackgroundXOrigin(&mut self, origin: BackgroundEdgeOrigin) {
        self.background_x_origin_ = origin;
        self.background_x_origin_set_ = true;
    }

    // cpp: layoutng_style/style/fill_layer.h:143-146
    pub fn SetBackgroundYOrigin(&mut self, origin: BackgroundEdgeOrigin) {
        self.background_y_origin_ = origin;
        self.background_y_origin_set_ = true;
    }

    // cpp: layoutng_style/style/fill_layer.h:147-151
    pub fn SetAttachment(&mut self, attachment: EFillAttachment) {
        debug_assert!(!self.cached_properties_.get().computed);
        self.attachment_ = attachment;
        self.attachment_set_ = true;
    }

    // cpp: layoutng_style/style/fill_layer.h:152-156
    pub fn SetClip(&mut self, clip: EFillBox) {
        debug_assert!(!self.cached_properties_.get().computed);
        self.clip_ = clip;
        self.clip_set_ = true;
    }

    // cpp: layoutng_style/style/fill_layer.h:157-161
    pub fn SetOrigin(&mut self, origin: EFillBox) {
        debug_assert!(!self.cached_properties_.get().computed);
        self.origin_ = origin;
        self.origin_set_ = true;
    }

    // cpp: layoutng_style/style/fill_layer.h:162-165
    pub fn SetRepeat(&mut self, repeat: &FillRepeat) {
        self.repeat_ = *repeat;
        self.repeat_set_ = true;
    }

    // cpp: layoutng_style/style/fill_layer.h:166-169
    pub fn SetMaskMode(&mut self, mode: EFillMaskMode) {
        self.mask_mode_ = mode;
        self.mask_mode_set_ = true;
    }

    // cpp: layoutng_style/style/fill_layer.h:170-173
    pub fn SetCompositingOperator(&mut self, operator: CompositingOperator) {
        self.compositing_operator_ = operator;
        self.compositing_operator_set_ = true;
    }

    // cpp: layoutng_style/style/fill_layer.h:174-177
    pub fn SetBlendMode(&mut self, mode: BlendMode) {
        self.blend_mode_ = mode;
        self.blend_mode_set_ = true;
    }

    // cpp: layoutng_style/style/fill_layer.h:178
    pub fn SetSizeType(&mut self, size_type: EFillSizeType) {
        self.size_type_ = size_type;
    }

    // cpp: layoutng_style/style/fill_layer.h:179
    pub fn SetSizeLength(&mut self, size: &LengthSize) {
        self.size_length_ = size.clone();
    }

    // cpp: layoutng_style/style/fill_layer.h:180-183
    pub fn SetSize(&mut self, size: &FillSize) {
        self.size_type_ = size.type_;
        self.size_length_ = size.size.clone();
    }

    // cpp: layoutng_style/style/fill_layer.h:185-188
    pub fn ClearImage(&mut self) {
        self.image_.Clear();
        self.image_set_ = false;
    }

    // cpp: layoutng_style/style/fill_layer.h:189-192
    pub fn ClearPositionX(&mut self) {
        self.pos_x_set_ = false;
        self.background_x_origin_set_ = false;
    }

    // cpp: layoutng_style/style/fill_layer.h:193-196
    pub fn ClearPositionY(&mut self) {
        self.pos_y_set_ = false;
        self.background_y_origin_set_ = false;
    }

    // cpp: layoutng_style/style/fill_layer.h:198-207
    pub fn ClearAttachment(&mut self) {
        self.attachment_set_ = false;
    }
    pub fn ClearClip(&mut self) {
        self.clip_set_ = false;
    }
    pub fn ClearOrigin(&mut self) {
        self.origin_set_ = false;
    }
    pub fn ClearRepeat(&mut self) {
        self.repeat_set_ = false;
    }
    pub fn ClearMaskMode(&mut self) {
        self.mask_mode_set_ = false;
    }
    pub fn ClearCompositingOperator(&mut self) {
        self.compositing_operator_set_ = false;
    }
    pub fn ClearBlendMode(&mut self) {
        self.blend_mode_set_ = false;
    }
    pub fn ClearSize(&mut self) {
        self.size_type_ = EFillSizeType::kSizeNone;
    }
}

#[allow(non_snake_case)]
impl FillLayer {
    // cpp: core/style/fill_layer.cc:137-176
    pub fn copy_from(&mut self, other: &Self) {
        if self.next_.Get() != other.next_.Get() {
            let next = other.next_.Get();
            self.next_ = if next.is_null() {
                Member::default()
            } else {
                Member::from_ptr(MakeGarbageCollected(unsafe { (&*next).clone() }))
            };
        }
        self.image_ = other.image_.clone();
        self.position_x_ = other.position_x_.clone();
        self.position_y_ = other.position_y_.clone();
        self.background_x_origin_ = other.background_x_origin_;
        self.background_y_origin_ = other.background_y_origin_;
        self.mask_mode_ = other.mask_mode_;
        self.background_x_origin_set_ = other.background_x_origin_set_;
        self.background_y_origin_set_ = other.background_y_origin_set_;
        self.size_length_ = other.size_length_.clone();
        self.attachment_ = other.attachment_;
        self.clip_ = other.clip_;
        self.compositing_operator_ = other.compositing_operator_;
        self.blend_mode_ = other.blend_mode_;
        self.origin_ = other.origin_;
        self.repeat_ = other.repeat_;
        self.size_type_ = other.size_type_;
        self.image_set_ = other.image_set_;
        self.attachment_set_ = other.attachment_set_;
        self.clip_set_ = other.clip_set_;
        self.compositing_operator_set_ = other.compositing_operator_set_;
        self.blend_mode_set_ = other.blend_mode_set_;
        self.origin_set_ = other.origin_set_;
        self.repeat_set_ = other.repeat_set_;
        self.mask_mode_set_ = other.mask_mode_set_;
        self.pos_x_set_ = other.pos_x_set_;
        self.pos_y_set_ = other.pos_y_set_;
        self.type_ = other.type_;
        self.cached_properties_
            .set(FillLayerCachedProperties::default());
    }

    // cpp: core/style/fill_layer.cc:195-207
    pub fn VisuallyEqual(&self, other: &Self) -> bool {
        if !self.image_.Get().is_null() || !other.image_.Get().is_null() {
            if !self.LayerPropertiesEqual(other) {
                return false;
            }
        } else if self.clip_ != other.clip_ {
            return false;
        }
        match (unsafe { self.next_.Get().as_ref() }, unsafe {
            other.next_.Get().as_ref()
        }) {
            (Some(a), Some(b)) => a.layer.VisuallyEqual(&b.layer),
            (None, None) => true,
            _ => false,
        }
    }

    // cpp: core/style/fill_layer.cc:415-417
    pub fn ClipOccludesNextLayers(&self) -> bool {
        self.Clip() == self.LayersClipMax()
    }

    // cpp: core/style/fill_layer.cc:419-464
    pub fn ImageOccludesNextLayers(&self, style: &ComputedStyle) -> bool {
        let Some(image) = (unsafe { self.GetImage().as_ref() }) else {
            return false;
        };
        if !image.CanRender() {
            return false;
        }
        let image_tiles_layer = || {
            matches!(
                self.Repeat().x,
                EFillRepeat::kRepeatFill | EFillRepeat::kRoundFill
            ) && matches!(
                self.Repeat().y,
                EFillRepeat::kRepeatFill | EFillRepeat::kRoundFill
            )
        };
        match self.Composite() {
            CompositeOperator::kCompositeClear | CompositeOperator::kCompositeCopy => {
                image_tiles_layer()
            }
            CompositeOperator::kCompositeSourceOver => {
                self.GetBlendMode() == BlendMode::kNormal
                    && image_tiles_layer()
                    && image.KnownToBeOpaque(style)
                    && !image
                        .ImageSize(
                            style.EffectiveZoom(),
                            &SizeF::default(),
                            RespectImageOrientationEnum::kRespectImageOrientation,
                        )
                        .IsEmpty()
            }
            _ => false,
        }
    }

    // cpp: core/style/fill_layer.cc:466-479
    pub fn AllImagesAreInvalid(&self) -> bool {
        let mut has_any_image = false;
        let mut layer = self as *const Self;
        while let Some(current) = unsafe { layer.as_ref() } {
            if let Some(image) = unsafe { current.GetImage().as_ref() } {
                has_any_image = true;
                if image.CanRender() && image.IsLoaded() {
                    return false;
                }
            }
            layer = current.Next();
        }
        has_any_image
    }

    // cpp: core/style/fill_layer.cc:481-495
    pub fn AnyImageIsLoading(&self) -> bool {
        let mut layer = self as *const Self;
        while let Some(current) = unsafe { layer.as_ref() } {
            if let Some(image) = unsafe { current.GetImage().as_ref() } {
                if !image.IsLoaded() && !image.ErrorOccurred() {
                    return true;
                }
            }
            layer = current.Next();
        }
        false
    }

    // cpp: layoutng_style/style/fill_layer.h:222
    pub fn GetType(&self) -> EFillLayerType {
        self.type_
    }

    // cpp: third_party/blink/renderer/core/style/fill_layer.cc:209-354
    pub fn FillUnsetProperties(&mut self) {
        self.FillPropertyPattern(Self::IsPositionXSet, |current, pattern| {
            current.position_x_ = pattern.position_x_.clone();
            if pattern.IsBackgroundXOriginSet() {
                current.background_x_origin_ = pattern.background_x_origin_;
            }
            if pattern.IsBackgroundYOriginSet() {
                current.background_y_origin_ = pattern.background_y_origin_;
            }
        });
        self.FillPropertyPattern(Self::IsPositionYSet, |current, pattern| {
            current.position_y_ = pattern.position_y_.clone();
            if pattern.IsBackgroundXOriginSet() {
                current.background_x_origin_ = pattern.background_x_origin_;
            }
            if pattern.IsBackgroundYOriginSet() {
                current.background_y_origin_ = pattern.background_y_origin_;
            }
        });
        self.FillPropertyPattern(Self::IsAttachmentSet, |current, pattern| {
            current.attachment_ = pattern.attachment_;
        });
        self.FillPropertyPattern(Self::IsClipSet, |current, pattern| {
            current.clip_ = pattern.clip_;
        });
        self.FillPropertyPattern(Self::IsCompositingOperatorSet, |current, pattern| {
            current.compositing_operator_ = pattern.compositing_operator_;
        });
        self.FillPropertyPattern(Self::IsBlendModeSet, |current, pattern| {
            current.blend_mode_ = pattern.blend_mode_;
        });
        self.FillPropertyPattern(Self::IsOriginSet, |current, pattern| {
            current.origin_ = pattern.origin_;
        });
        self.FillPropertyPattern(Self::IsRepeatSet, |current, pattern| {
            current.repeat_ = pattern.repeat_;
        });
        self.FillPropertyPattern(Self::IsSizeSet, |current, pattern| {
            current.size_type_ = pattern.size_type_;
            current.size_length_ = pattern.size_length_.clone();
        });
        self.FillPropertyPattern(Self::IsMaskModeSet, |current, pattern| {
            current.mask_mode_ = pattern.mask_mode_;
        });
    }

    // Share the identical traversal used for each property in the source.
    // Assign fields directly: repetition must preserve the CSS "set" bits.
    fn FillPropertyPattern(&mut self, is_set: fn(&Self) -> bool, assign: fn(&mut Self, &Self)) {
        let head = self as *mut Self;
        let mut current = head;
        // SAFETY: this is the uniquely borrowed, acyclic Member-owned layer
        // chain. No node is allocated or detached while repeating properties.
        unsafe {
            while !current.is_null() && is_set(&*current) {
                current = (*current).NextMut();
            }
            if current.is_null() || current == head {
                return;
            }
            let mut pattern = head;
            while !current.is_null() {
                // The pattern starts in the set prefix and advances behind
                // current; reset before it can alias the destination node.
                debug_assert_ne!(pattern, current);
                assign(&mut *current, &*pattern);
                pattern = (*pattern).NextMut();
                if pattern.is_null() || pattern == current {
                    pattern = head;
                }
                current = (*current).NextMut();
            }
        }
    }

    // cpp: third_party/blink/renderer/core/style/fill_layer.cc:356-365
    pub fn CullEmptyLayers(&mut self) {
        let mut current = self as *mut Self;
        // SAFETY: the head remains borrowed/rooted. Read the next link before
        // detaching it, and never access that detached suffix afterwards.
        unsafe {
            while !current.is_null() {
                let next = (*current).NextMut();
                if !next.is_null() && !(*next).IsImageSet() {
                    (*current).next_.Clear();
                    break;
                }
                current = next;
            }
        }
    }

    // cpp: core/style/fill_layer.cc:497-512
    pub fn ImagesIdentical(first: *const Self, second: *const Self) -> bool {
        let (mut first, mut second) = (first, second);
        while !first.is_null() && !second.is_null() {
            if unsafe { &*first }.GetImage() != unsafe { &*second }.GetImage() {
                return false;
            }
            first = unsafe { &*first }.Next();
            second = unsafe { &*second }.Next();
        }
        first.is_null() && second.is_null()
    }

    // cpp: layoutng_style/style/fill_layer.h:229-232
    pub fn LayersClipMax(&self) -> EFillBox {
        self.ComputeCachedPropertiesIfNeeded();
        self.cached_properties_.get().layers_clip_max
    }

    // cpp: layoutng_style/style/fill_layer.h:233-236
    pub fn AnyLayerUsesContentBox(&self) -> bool {
        self.ComputeCachedPropertiesIfNeeded();
        self.cached_properties_.get().any_layer_uses_content_box
    }

    // cpp: layoutng_style/style/fill_layer.h:237-240
    pub fn AnyLayerHasImage(&self) -> bool {
        self.ComputeCachedPropertiesIfNeeded();
        self.cached_properties_.get().any_layer_has_image
    }

    // cpp: layoutng_style/style/fill_layer.h:241-244
    pub fn AnyLayerHasUrlImage(&self) -> bool {
        self.ComputeCachedPropertiesIfNeeded();
        self.cached_properties_.get().any_layer_has_url_image
    }

    // cpp: layoutng_style/style/fill_layer.h:245-248
    pub fn AnyLayerHasLocalAttachment(&self) -> bool {
        self.ComputeCachedPropertiesIfNeeded();
        self.cached_properties_.get().any_layer_has_local_attachment
    }

    // cpp: layoutng_style/style/fill_layer.h:249-253
    pub fn AnyLayerHasLocalAttachmentImage(&self) -> bool {
        self.ComputeCachedPropertiesIfNeeded();
        let cache = self.cached_properties_.get();
        cache.any_layer_has_local_attachment && cache.any_layer_has_image
    }

    // cpp: layoutng_style/style/fill_layer.h:254-257
    pub fn AnyLayerHasFixedAttachmentImage(&self) -> bool {
        self.ComputeCachedPropertiesIfNeeded();
        self.cached_properties_
            .get()
            .any_layer_has_fixed_attachment_image
    }

    // cpp: layoutng_style/style/fill_layer.h:258-261
    pub fn AnyLayerHasDefaultAttachmentImage(&self) -> bool {
        self.ComputeCachedPropertiesIfNeeded();
        self.cached_properties_
            .get()
            .any_layer_has_default_attachment_image
    }

    // cpp: layoutng_style/style/fill_layer.h:262-265
    pub fn AnyLayerUsesCurrentColor(&self) -> bool {
        self.ComputeCachedPropertiesIfNeeded();
        self.cached_properties_.get().any_layer_uses_current_color
    }

    // cpp: layoutng_style/style/fill_layer.h:267-269
    pub fn InitialFillAttachment(_: EFillLayerType) -> EFillAttachment {
        EFillAttachment::kScroll
    }

    // cpp: layoutng_style/style/fill_layer.h:270
    pub fn InitialFillClip(_: EFillLayerType) -> EFillBox {
        EFillBox::kBorder
    }

    // cpp: layoutng_style/style/fill_layer.h:271-274
    pub fn InitialFillOrigin(type_: EFillLayerType) -> EFillBox {
        if type_ == EFillLayerType::kBackground {
            EFillBox::kPadding
        } else {
            EFillBox::kBorder
        }
    }

    // cpp: layoutng_style/style/fill_layer.h:275-277
    pub fn InitialFillRepeat(_: EFillLayerType) -> FillRepeat {
        FillRepeat::default()
    }

    // cpp: layoutng_style/style/fill_layer.h:278-280
    pub fn InitialFillMaskMode(_: EFillLayerType) -> EFillMaskMode {
        EFillMaskMode::kMatchSource
    }

    // cpp: layoutng_style/style/fill_layer.h:281-284
    pub fn InitialFillCompositingOperator(_: EFillLayerType) -> CompositingOperator {
        CompositingOperator::kAdd
    }

    // cpp: layoutng_style/style/fill_layer.h:285-287
    pub fn InitialFillBlendMode(_: EFillLayerType) -> BlendMode {
        BlendMode::kNormal
    }

    // cpp: layoutng_style/style/fill_layer.h:288-290
    pub fn InitialFillSizeType(_: EFillLayerType) -> EFillSizeType {
        EFillSizeType::kSizeLength
    }

    // cpp: layoutng_style/style/fill_layer.h:291-293
    pub fn InitialFillSizeLength(_: EFillLayerType) -> LengthSize {
        LengthSize::default()
    }

    // cpp: layoutng_style/style/fill_layer.h:294-296
    pub fn InitialFillSize(type_: EFillLayerType) -> FillSize {
        FillSize::new(
            Self::InitialFillSizeType(type_),
            &Self::InitialFillSizeLength(type_),
        )
    }

    // cpp: layoutng_style/style/fill_layer.h:297-299
    pub fn InitialFillPositionX(_: EFillLayerType) -> Length {
        Length::Percent(0.0)
    }

    // cpp: layoutng_style/style/fill_layer.h:300-302
    pub fn InitialFillPositionY(_: EFillLayerType) -> Length {
        Length::Percent(0.0)
    }

    // cpp: layoutng_style/style/fill_layer.h:303
    pub fn InitialFillImage(_: EFillLayerType) -> *mut StyleImage {
        std::ptr::null_mut()
    }

    // cpp: layoutng_style/style/fill_layer.h:305-315
    pub fn IterateFillLayersInReverseOrder<F>(
        start: *const FillLayer,
        end: *const FillLayer,
        mut callback: F,
    ) where
        F: FnMut(&FillLayer) + Clone,
    {
        if start != end {
            Self::IterateFillLayersInReverseOrder(
                unsafe { (&*start).Next() },
                end,
                callback.clone(),
            );
        }
        if !start.is_null() {
            callback(unsafe { &*start });
        }
    }

    // cpp: core/style/fill_layer.cc:178-188
    fn LayerPropertiesEqual(&self, other: &Self) -> bool {
        ValuesEquivalent(&self.image_, &other.image_)
            && self.position_x_ == other.position_x_
            && self.position_y_ == other.position_y_
            && self.background_x_origin_ == other.background_x_origin_
            && self.background_y_origin_ == other.background_y_origin_
            && self.mask_mode_ == other.mask_mode_
            && self.attachment_ == other.attachment_
            && self.clip_ == other.clip_
            && self.compositing_operator_ == other.compositing_operator_
            && self.blend_mode_ == other.blend_mode_
            && self.origin_ == other.origin_
            && self.repeat_ == other.repeat_
            && self.size_type_ == other.size_type_
            && self.size_length_ == other.size_length_
            && self.type_ == other.type_
    }

    // cpp: core/style/fill_layer.cc:367-377
    fn EffectiveClip(&self) -> EFillBox {
        if self.GetType() == EFillLayerType::kMask
            && unsafe { self.GetImage().as_ref() }
                .and_then(MaskSourceImage)
                .is_some_and(StyleMaskSourceImage::HasSVGMask)
        {
            EFillBox::kNoClip
        } else {
            self.Clip()
        }
    }

    // cpp: layoutng_style/style/fill_layer.h:324-328
    pub(crate) fn ComputeCachedPropertiesIfNeeded(&self) {
        if !self.cached_properties_.get().computed {
            self.ComputeCachedProperties();
        }
    }

    // cpp: core/style/fill_layer.cc:379-413
    pub(crate) fn ComputeCachedProperties(&self) {
        debug_assert!(!self.cached_properties_.get().computed);
        let image = unsafe { self.GetImage().as_ref() };
        let effective_clip = self.EffectiveClip();
        let mut cache = self.cached_properties_.get();
        cache.layers_clip_max = effective_clip;
        cache.any_layer_uses_content_box =
            effective_clip == EFillBox::kContent || self.Origin() == EFillBox::kContent;
        cache.any_layer_has_image = image.is_some();
        cache.any_layer_has_url_image =
            image.is_some_and(|image| unsafe { &*image.CssValue() }.MayContainUrl());
        cache.any_layer_has_local_attachment = self.Attachment() == EFillAttachment::kLocal;
        cache.any_layer_has_fixed_attachment_image =
            cache.any_layer_has_image && self.Attachment() == EFillAttachment::kFixed;
        cache.any_layer_has_default_attachment_image =
            cache.any_layer_has_image && self.Attachment() == EFillAttachment::kScroll;
        cache.any_layer_uses_current_color =
            image.is_some_and(|image| image.IsGeneratedImage() && image.DependsOnCurrentColor());
        cache.computed = true;
        self.cached_properties_.set(cache);

        if let Some(next) = unsafe { self.Next().as_ref() } {
            next.ComputeCachedPropertiesIfNeeded();
            cache.layers_clip_max = EnclosingFillBox(self.LayersClipMax(), next.LayersClipMax());
            let next_cache = next.cached_properties_.get();
            cache.any_layer_uses_content_box |= next_cache.any_layer_uses_content_box;
            cache.any_layer_has_image |= next_cache.any_layer_has_image;
            cache.any_layer_has_url_image |= next_cache.any_layer_has_url_image;
            cache.any_layer_has_local_attachment |= next_cache.any_layer_has_local_attachment;
            cache.any_layer_has_fixed_attachment_image |=
                next_cache.any_layer_has_fixed_attachment_image;
            cache.any_layer_has_default_attachment_image |=
                next_cache.any_layer_has_default_attachment_image;
            cache.any_layer_uses_current_color |= next_cache.any_layer_uses_current_color;
            self.cached_properties_.set(cache);
        }
    }
}

// cpp: core/style/fill_layer.cc:190-193
impl PartialEq for FillLayer {
    fn eq(&self, other: &Self) -> bool {
        if !self.LayerPropertiesEqual(other) {
            return false;
        }
        match (unsafe { self.Next().as_ref() }, unsafe {
            other.Next().as_ref()
        }) {
            (Some(a), Some(b)) => a == b,
            (None, None) => true,
            _ => false,
        }
    }
}

// cpp: layoutng_style/style/fill_layer.h:386-392
#[derive(Clone)]
pub struct FillLayerWrapper {
    pub layer: FillLayer,
}

#[allow(non_snake_case)]
impl FillLayerWrapper {
    // cpp: layoutng_style/style/fill_layer.h:388
    pub fn new(type_: EFillLayerType) -> Self {
        Self {
            layer: FillLayer::new(type_, false),
        }
    }

    // cpp: layoutng_style/style/fill_layer.h:390
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.layer);
    }
}
