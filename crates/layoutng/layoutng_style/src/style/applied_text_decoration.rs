use foundation::{
    Color, EBoxDecorationBreak, ETextDecorationStyle, Length, TextDecorationLine,
    TextDecorationThickness,
};

use super::text_decoration_inset::TextDecorationInset;

// This value only contains scalars and independently owned values; no GC edge.
impl foundation::Traceable for AppliedTextDecoration {
    fn Trace(&self, _visitor: &mut foundation::Visitor<'_>) {}
}

// cpp: layoutng_style/style/applied_text_decoration.h:19-59
#[derive(Clone)]
pub struct AppliedTextDecoration {
    lines_: TextDecorationLine,
    style_: ETextDecorationStyle,
    box_decoration_break_: EBoxDecorationBreak,
    color_: Color,
    thickness_: TextDecorationThickness,
    underline_offset_: Length,
    decoration_inset_: TextDecorationInset,
}

#[allow(non_snake_case)]
impl AppliedTextDecoration {
    // cpp: layoutng_style/style/applied_text_decoration.h:23-29
    // cpp: layoutng_style/style/applied_text_decoration.cc:9-24
    pub fn new(
        line: TextDecorationLine,
        style: ETextDecorationStyle,
        color: Color,
        thickness: TextDecorationThickness,
        underline_offset: Length,
        decoration_inset: TextDecorationInset,
        box_decoration_break: EBoxDecorationBreak,
    ) -> Self {
        Self {
            lines_: line,
            style_: style,
            box_decoration_break_: box_decoration_break,
            color_: color,
            thickness_: thickness,
            underline_offset_: underline_offset,
            decoration_inset_: decoration_inset,
        }
    }

    // cpp: layoutng_style/style/applied_text_decoration.h:31-36
    pub fn Lines(&self) -> TextDecorationLine {
        self.lines_
    }
    pub fn Style(&self) -> ETextDecorationStyle {
        self.style_
    }

    // cpp: layoutng_style/style/applied_text_decoration.h:37-38
    pub fn GetColor(&self) -> Color {
        self.color_.clone()
    }
    pub fn SetColor(&mut self, color: Color) {
        self.color_ = color;
    }

    // cpp: layoutng_style/style/applied_text_decoration.h:40-47
    pub fn Thickness(&self) -> TextDecorationThickness {
        self.thickness_.clone()
    }
    pub fn UnderlineOffset(&self) -> Length {
        self.underline_offset_.clone()
    }
    pub fn DecorationInset(&self) -> &TextDecorationInset {
        &self.decoration_inset_
    }
    pub fn BoxDecorationBreak(&self) -> EBoxDecorationBreak {
        self.box_decoration_break_
    }
}

// cpp: layoutng_style/style/applied_text_decoration.h:49
// cpp: layoutng_style/style/applied_text_decoration.cc:26-27
impl PartialEq for AppliedTextDecoration {
    fn eq(&self, other: &Self) -> bool {
        self.lines_ == other.lines_
            && self.style_ == other.style_
            && self.box_decoration_break_ == other.box_decoration_break_
            && self.color_ == other.color_
            && self.thickness_ == other.thickness_
            && self.underline_offset_ == other.underline_offset_
            && self.decoration_inset_ == other.decoration_inset_
    }
}

// cpp: layoutng_style/style/applied_text_decoration.h:61
pub type AppliedTextDecorationVector = foundation::GCedHeapVector<AppliedTextDecoration>;
