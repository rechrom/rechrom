use crate::src::core::SkPaint::{Cap, Join};
use crate::src::core::SkPaintDefaults::SkPaintDefaults_MiterLimit;

// Source: include/core/SkStrokeRec.h:25-28
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitStyle {
    kHairline_InitStyle,
    kFill_InitStyle,
}
// Source: include/core/SkStrokeRec.h:33-40
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    kHairline_Style,
    kFill_Style,
    kStroke_Style,
    kStrokeAndFill_Style,
}
pub const kStyleCount: i32 = Style::kStrokeAndFill_Style as i32 + 1;
impl SkStrokeRec {
    // Source: include/core/SkStrokeRec.h:43-46
    pub fn getWidth(&self) -> f32 {
        self.fWidth
    }
    pub fn getMiter(&self) -> f32 {
        self.fMiterLimit
    }
    pub fn getCap(&self) -> Cap {
        match self.packed & 0xffff {
            0 => Cap::kButt_Cap,
            1 => Cap::kRound_Cap,
            2 => Cap::kSquare_Cap,
            _ => unreachable!(),
        }
    }
    pub fn getJoin(&self) -> Join {
        match (self.packed >> 16) & 0x7fff {
            0 => Join::kMiter_Join,
            1 => Join::kRound_Join,
            2 => Join::kBevel_Join,
            _ => unreachable!(),
        }
    }
    // Source: include/core/SkStrokeRec.h:48-54
    pub fn isHairlineStyle(&self) -> bool {
        self.getStyle() == Style::kHairline_Style
    }
    pub fn isFillStyle(&self) -> bool {
        self.getStyle() == Style::kFill_Style
    }
    // Source: include/core/SkStrokeRec.h:66-70
    pub fn setStrokeParams(&mut self, cap: Cap, join: Join, miterLimit: f32) {
        self.packed = (self.packed & 0x80000000) | cap as u32 | ((join as u32) << 16);
        self.fMiterLimit = miterLimit;
    }
    // Source: include/core/SkStrokeRec.h:72-79
    pub fn getResScale(&self) -> f32 {
        self.fResScale
    }
    pub fn setResScale(&mut self, rs: f32) {
        debug_assert!(rs > 0.0 && rs.is_finite());
        self.fResScale = rs;
    }
    // Source: include/core/SkStrokeRec.h:85-88
    pub fn needToApply(&self) -> bool {
        matches!(
            self.getStyle(),
            Style::kStroke_Style | Style::kStrokeAndFill_Style
        )
    }
    // Source: include/core/SkStrokeRec.h:131-140
    pub fn hasEqualEffect(&self, other: &Self) -> bool {
        if !self.needToApply() {
            return self.getStyle() == other.getStyle();
        }
        self.fWidth == other.fWidth
            && (self.getJoin() != Join::kMiter_Join || self.fMiterLimit == other.fMiterLimit)
            && self.getCap() == other.getCap()
            && self.getJoin() == other.getJoin()
            && (self.packed >> 31) == (other.packed >> 31)
    }
}
// Source: include/core/SkStrokeRec.h:145-156
// Rust stores the original three bitfields in one u32; no padding participates in equality.
#[derive(Clone, Copy, Debug)]
pub struct SkStrokeRec {
    fResScale: f32,
    fWidth: f32,
    fMiterLimit: f32,
    packed: u32,
}
// Source: src/core/SkStrokeRec.cpp:15-16
const kStrokeRec_FillStyleWidth: f32 = -1.0;
impl SkStrokeRec {
    // Source: src/core/SkStrokeRec.cpp:18-25
    pub fn new(s: InitStyle) -> Self {
        Self {
            fResScale: 1.0,
            fWidth: if s == InitStyle::kFill_InitStyle {
                kStrokeRec_FillStyleWidth
            } else {
                0.0
            },
            fMiterLimit: SkPaintDefaults_MiterLimit,
            packed: (Cap::kDefault_Cap as u32) | ((Join::kDefault_Join as u32) << 16),
        }
    }
    // Source: src/core/SkStrokeRec.cpp:27-33
    pub fn from_paint(paint: &crate::src::core::SkPaint::SkPaint, resScale: f32) -> Self {
        Self::from_paint_style(paint, paint.getStyle(), resScale)
    }
    pub fn from_paint_style(
        paint: &crate::src::core::SkPaint::SkPaint,
        styleOverride: crate::src::core::SkPaint::Style,
        resScale: f32,
    ) -> Self {
        // Initialized Rust storage replaces the native constructor's uninitialized members.
        let mut result = Self {
            fResScale: 0.0,
            fWidth: 0.0,
            fMiterLimit: 0.0,
            packed: 0,
        };
        result.init(paint, styleOverride, resScale);
        result
    }
    // Source: src/core/SkStrokeRec.cpp:35-69
    fn init(
        &mut self,
        paint: &crate::src::core::SkPaint::SkPaint,
        style: crate::src::core::SkPaint::Style,
        resScale: f32,
    ) {
        use crate::src::core::SkPaint::Style as PaintStyle;
        self.fResScale = resScale;
        match style {
            PaintStyle::kFill_Style => {
                self.fWidth = kStrokeRec_FillStyleWidth;
                self.packed &= 0x7fffffff;
            }
            PaintStyle::kStroke_Style => {
                self.fWidth = paint.getStrokeWidth();
                self.packed &= 0x7fffffff;
            }
            PaintStyle::kStrokeAndFill_Style => {
                if paint.getStrokeWidth() == 0.0 {
                    self.fWidth = kStrokeRec_FillStyleWidth;
                    self.packed &= 0x7fffffff;
                } else {
                    self.fWidth = paint.getStrokeWidth();
                    self.packed |= 0x80000000;
                }
            }
        }
        // Rust's closed enum makes the native invalid-style fallback unreachable.
        self.fMiterLimit = paint.getStrokeMiter();
        self.packed = (self.packed & 0xffff0000) | paint.getStrokeCap() as u32;
        self.packed = (self.packed & 0x8000ffff) | ((paint.getStrokeJoin() as u32) << 16);
    }
    // Source: src/core/SkStrokeRec.cpp:72-80
    pub fn getStyle(&self) -> Style {
        if self.fWidth < 0.0 {
            Style::kFill_Style
        } else if self.fWidth == 0.0 {
            Style::kHairline_Style
        } else if self.packed >> 31 != 0 {
            Style::kStrokeAndFill_Style
        } else {
            Style::kStroke_Style
        }
    }
    // Source: src/core/SkStrokeRec.cpp:82-85
    pub fn setFillStyle(&mut self) {
        self.fWidth = kStrokeRec_FillStyleWidth;
        self.packed &= 0x7fffffff;
    }
    // Source: src/core/SkStrokeRec.cpp:87-90
    pub fn setHairlineStyle(&mut self) {
        self.fWidth = 0.0;
        self.packed &= 0x7fffffff;
    }
    // Source: src/core/SkStrokeRec.cpp:92-100
    pub fn setStrokeStyle(&mut self, width: f32, strokeAndFill: bool) {
        if strokeAndFill && width == 0.0 {
            self.setFillStyle();
        } else {
            self.fWidth = width;
            self.packed = (self.packed & 0x7fffffff) | ((strokeAndFill as u32) << 31);
        }
    }
    // Source: src/core/SkStrokeRec.cpp:107-125
    pub fn applyToPath(
        &self,
        dst: &mut crate::src::core::SkPathBuilder::SkPathBuilder,
        src: &crate::src::core::SkPath::SkPath,
    ) -> bool {
        if self.fWidth <= 0.0 {
            return false;
        }
        let mut stroker = crate::src::core::SkStroke::SkStroke::new();
        stroker.setCap(self.getCap());
        stroker.setJoin(self.getJoin());
        stroker.setMiterLimit(self.fMiterLimit);
        stroker.setWidth(self.fWidth);
        stroker.setDoFill(self.packed >> 31 != 0);
        // Production resolution uses the native non-Viewer branch; debug UI knobs are excluded.
        stroker.setResScale(self.fResScale);
        stroker.strokePath(src, dst);
        true
    }
    // Source: src/core/SkStrokeRec.cpp:127-138
    pub fn applyToPaint(&self, paint: &mut crate::src::core::SkPaint::SkPaint) {
        use crate::src::core::SkPaint::Style as PaintStyle;
        if self.fWidth < 0.0 {
            paint.setStyle(PaintStyle::kFill_Style);
            return;
        }
        paint.setStyle(if self.packed >> 31 != 0 {
            PaintStyle::kStrokeAndFill_Style
        } else {
            PaintStyle::kStroke_Style
        });
        paint.setStrokeWidth(self.fWidth);
        paint.setStrokeMiter(self.fMiterLimit);
        paint.setStrokeCap(self.getCap());
        paint.setStrokeJoin(self.getJoin());
    }
    // Source: src/core/SkStrokeRec.cpp:140-142
    pub fn getInflationRadius(&self) -> f32 {
        Self::GetInflationRadius(self.getJoin(), self.fMiterLimit, self.getCap(), self.fWidth)
    }
    // Source: src/core/SkStrokeRec.cpp:144-149
    pub fn GetInflationRadiusForPaint(
        paint: &crate::src::core::SkPaint::SkPaint,
        style: crate::src::core::SkPaint::Style,
    ) -> f32 {
        let width = if style == crate::src::core::SkPaint::Style::kFill_Style {
            -1.0
        } else {
            paint.getStrokeWidth()
        };
        Self::GetInflationRadius(
            paint.getStrokeJoin(),
            paint.getStrokeMiter(),
            paint.getStrokeCap(),
            width,
        )
    }
    // Source: src/core/SkStrokeRec.cpp:151-171
    pub fn GetInflationRadius(join: Join, miterLimit: f32, cap: Cap, strokeWidth: f32) -> f32 {
        if strokeWidth < 0.0 {
            return 0.0;
        } else if strokeWidth == 0.0 {
            return 1.0;
        }
        let mut multiplier = 1.0;
        // std::max keeps its first argument when the comparison is false (including NaN).
        if join == Join::kMiter_Join && multiplier < miterLimit {
            multiplier = miterLimit;
        }
        if cap == Cap::kSquare_Cap && multiplier < std::f32::consts::SQRT_2 {
            multiplier = std::f32::consts::SQRT_2;
        }
        strokeWidth / 2.0 * multiplier
    }
}
