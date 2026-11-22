use foundation::ScopedCSSName;

use super::position_area::PositionArea;
use super::style_position_anchor::{StylePositionAnchor, Type};

// cpp: layoutng_style/style/default_anchor_data.h:15-46
#[derive(Clone)]
pub struct DefaultAnchorData {
    position_anchor_: StylePositionAnchor,
    position_area_: PositionArea,
}

#[allow(non_snake_case)]
impl DefaultAnchorData {
    // cpp: layoutng_style/style/default_anchor_data.h:20-22
    pub fn new(position_anchor: &StylePositionAnchor, position_area: PositionArea) -> Self {
        Self {
            position_anchor_: position_anchor.clone(),
            position_area_: position_area,
        }
    }

    // cpp: layoutng_style/style/default_anchor_data.h:26-33
    pub fn GetType(&self) -> Type {
        let type_ = self.position_anchor_.GetType();
        if type_ == Type::kNormal {
            return if self.position_area_.IsNone() {
                Type::kNone
            } else {
                Type::kAuto
            };
        }
        type_
    }

    // cpp: layoutng_style/style/default_anchor_data.h:35-36
    pub fn GetName(&self) -> &ScopedCSSName {
        self.position_anchor_.GetName()
    }
    pub fn GetPositionArea(&self) -> &PositionArea {
        &self.position_area_
    }
}

// cpp: layoutng_style/style/default_anchor_data.h:19
// cpp: layoutng_style/style/default_anchor_data.h:43-45
impl Default for DefaultAnchorData {
    fn default() -> Self {
        Self {
            position_anchor_: StylePositionAnchor::Initial(),
            position_area_: PositionArea::default(),
        }
    }
}

// cpp: layoutng_style/style/default_anchor_data.h:38-41
impl PartialEq for DefaultAnchorData {
    fn eq(&self, other: &Self) -> bool {
        self.position_anchor_ == other.position_anchor_
            && self.position_area_ == other.position_area_
    }
}
