use std::cell::Cell;

use foundation::{MakeGarbageCollected, Member, Visitor};

use super::computed_style_constants::{BackgroundEdgeOrigin, EFillLayerType, EFillSizeType};
use super::fill_layer::{FillLayer, FillLayerCachedProperties};

#[allow(non_snake_case)]
impl FillLayer {
    // cpp: layoutng_style/style/fill_layer.h:67
    pub fn new_without_initial_values(type_: EFillLayerType) -> Self {
        Self::new(type_, false)
    }

    // cpp: layoutng_style/style/fill_layer.h:67
    // cpp: layoutng_style/style/fill_layer_data.cc:24-66
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

    // cpp: layoutng_style/style/fill_layer.h:69
    // cpp: layoutng_style/style/fill_layer_data.cc:107-110
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.next_);
        visitor.Trace(&self.image_);
    }
}

// cpp: layoutng_style/style/fill_layer.h:210
// cpp: layoutng_style/style/fill_layer_data.cc:68-105
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
            repeat_: self.repeat_.clone(),
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
