use foundation::{AtomicString, Color, Member, Visitor};

use super::style_svg_resource::StyleSVGResource;
use crate::css::style_color::StyleColor;

// cpp: layoutng_style/style/svg_paint.h:41-49
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SVGPaintType {
    kColor,
    kNone,
    kContextFill,
    kContextStroke,
    kUriNone,
    kUriColor,
    kUri,
}

// cpp: layoutng_style/style/svg_paint.h:51-90
// cpp: layoutng_style/style/svg_paint.cc:37-41
#[derive(Clone)]
pub struct SVGPaint {
    pub color: StyleColor,
    pub resource: Member<StyleSVGResource>,
    pub paint_type: SVGPaintType,
    pub is_initial_value: bool,
}

// cpp: layoutng_style/style/svg_paint.h:55
// cpp: layoutng_style/style/svg_paint.cc:35
impl Default for SVGPaint {
    fn default() -> Self {
        Self {
            color: StyleColor::default(),
            resource: Member::default(),
            paint_type: SVGPaintType::kNone,
            is_initial_value: false,
        }
    }
}

#[allow(non_snake_case)]
impl SVGPaint {
    // cpp: layoutng_style/style/svg_paint.h:56
    // cpp: layoutng_style/style/svg_paint.cc:36
    pub fn from_color(color: Color) -> Self {
        Self {
            color: StyleColor::from_color(color),
            paint_type: SVGPaintType::kColor,
            ..Default::default()
        }
    }

    // cpp: layoutng_style/style/svg_paint.h:61-64
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.color);
        visitor.Trace(&self.resource);
    }

    // cpp: layoutng_style/style/svg_paint.h:68-69
    // cpp: layoutng_style/style/svg_paint.cc:48-58
    pub fn CreateInitial() -> Self {
        let mut result = Self::default();
        result.is_initial_value = true;
        result
    }

    pub fn CreateInitialBlack() -> Self {
        let mut result = Self::from_color(Color::kBlack);
        result.is_initial_value = true;
        result
    }

    // cpp: layoutng_style/style/svg_paint.h:71-73
    pub fn IsNone(&self) -> bool {
        self.paint_type == SVGPaintType::kNone
    }
    pub fn IsInitial(&self) -> bool {
        self.is_initial_value
    }
    pub fn IsColor(&self) -> bool {
        self.paint_type == SVGPaintType::kColor
    }

    // cpp: layoutng_style/style/svg_paint.h:75-78
    pub fn EqualTypeOrColor(&self, other: &Self) -> bool {
        self.paint_type == other.paint_type
            && (self.paint_type != SVGPaintType::kColor || self.color == other.color)
    }

    // cpp: layoutng_style/style/svg_paint.h:79-82
    pub fn HasColor(&self) -> bool {
        self.IsColor() || self.paint_type == SVGPaintType::kUriColor
    }
    pub fn HasUrl(&self) -> bool {
        self.paint_type >= SVGPaintType::kUriNone
    }
    pub fn HasCurrentColor(&self) -> bool {
        self.HasColor() && self.color.IsCurrentColor()
    }
    pub fn Resource(&self) -> *mut StyleSVGResource {
        self.resource.Get()
    }

    // cpp: layoutng_style/style/svg_paint.h:84-85
    // cpp: layoutng_style/style/svg_paint.cc:60-62
    pub fn GetColor(&self) -> &StyleColor {
        &self.color
    }
    pub fn GetUrl(&self) -> &AtomicString {
        unsafe { (&*self.Resource()).Url() }
    }
}

// cpp: layoutng_style/style/svg_paint.h:66
// cpp: layoutng_style/style/svg_paint.cc:43-46
impl PartialEq for SVGPaint {
    fn eq(&self, other: &Self) -> bool {
        self.paint_type == other.paint_type
            && self.color == other.color
            && foundation::ValuesEquivalent(&self.resource, &other.resource)
    }
}
