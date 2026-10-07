use foundation::LayoutUnit;

// cpp: layoutng_style/style/style_overflow_clip_margin.h:16
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum ReferenceBox {
    #[default]
    kBorderBox,
    kPaddingBox,
    kContentBox,
}

// cpp: layoutng_style/style/style_overflow_clip_margin.h:12-37
#[derive(Clone, Copy, Debug)]
pub struct StyleOverflowClipMargin {
    reference_box_: ReferenceBox,
    margin_: LayoutUnit,
}

#[allow(non_snake_case)]
impl StyleOverflowClipMargin {
    // cpp: layoutng_style/style/style_overflow_clip_margin.h:17-18
    pub fn new(reference_box: ReferenceBox, margin: LayoutUnit) -> Self {
        Self {
            reference_box_: reference_box,
            margin_: margin,
        }
    }

    // cpp: layoutng_style/style/style_overflow_clip_margin.h:20-22
    pub fn CreateContent() -> Self {
        Self::new(ReferenceBox::kContentBox, LayoutUnit::default())
    }

    // cpp: layoutng_style/style/style_overflow_clip_margin.h:26-27
    pub fn GetReferenceBox(&self) -> ReferenceBox {
        self.reference_box_
    }
    pub fn GetMargin(&self) -> LayoutUnit {
        self.margin_
    }
}

// cpp: layoutng_style/style/style_overflow_clip_margin.h:24
impl Default for StyleOverflowClipMargin {
    fn default() -> Self {
        Self::new(ReferenceBox::kBorderBox, LayoutUnit::default())
    }
}

// cpp: layoutng_style/style/style_overflow_clip_margin.h:29-32
impl PartialEq for StyleOverflowClipMargin {
    fn eq(&self, other: &Self) -> bool {
        self.GetReferenceBox() == other.GetReferenceBox() && self.GetMargin() == other.GetMargin()
    }
}

impl Eq for StyleOverflowClipMargin {}
