// C++: font_engine/text_shaper.h. Borrowed request data remains borrowed for
// one call; ShapedRun owns its glyphs as in the source.

// cpp: font_engine/text_shaper.h:11-20
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextDirection {
    #[default]
    kLtr,
    kRtl,
    kTtb,
    kBtt,
}

pub struct ShapeRequest<'a> {
    pub font_bytes: &'a [u8],
    pub utf8: &'a str,
    pub font_size: f64,
    pub face_index: u32,
    pub direction: TextDirection,
    pub language: &'a str,
    pub script: &'a str,
}

impl<'a> ShapeRequest<'a> {
    pub fn new(font_bytes: &'a [u8], utf8: &'a str) -> Self {
        Self {
            font_bytes,
            utf8,
            font_size: 16.0,
            face_index: 0,
            direction: TextDirection::kLtr,
            language: "en",
            script: "",
        }
    }
}

// cpp: font_engine/text_shaper.h:21-33
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShapedGlyph {
    pub id: u32,
    pub cluster: u32,
    pub x_advance: f64,
    pub y_advance: f64,
    pub x_offset: f64,
    pub y_offset: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapedRun {
    pub glyphs: Vec<ShapedGlyph>,
    pub advance: f64,
}

// cpp: font_engine/text_shaper.h:39-43
pub trait TextShaper {
    fn Shape(&self, request: &ShapeRequest<'_>) -> ShapedRun;
}
