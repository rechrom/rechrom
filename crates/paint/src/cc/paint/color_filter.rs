//! Canonical cc ColorFilter owner/factories/codec. The native filter value is
//! constructed by real Skia primitives; no compositor/host capability enters.
#![allow(non_snake_case, non_camel_case_types)]
use super::{blend_mode::BlendMode, color::Color4f};
use std::sync::Arc;
// Source: cc/paint/color_filter.h:51-64.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    kNull,
    kMatrix,
    kBlend,
    kSRGBToLinearGamma,
    kLinearToSRGBGamma,
    kLuma,
    kTableARGB,
    kHighContrast,
}
pub trait NativeColorFilterValue {
    fn filterColor4f(&self, color: Color4f) -> Color4f;
    fn asAColorMatrix(&self, matrix: &mut [f32; 20]) -> bool;
}
pub trait NativeColorTableValue {
    fn alphaTable(&self) -> &[u8; 256];
    fn redTable(&self) -> &[u8; 256];
    fn greenTable(&self) -> &[u8; 256];
    fn blueTable(&self) -> &[u8; 256];
}
pub trait ColorFilterBackend {
    type Filter: NativeColorFilterValue;
    type ColorTable: NativeColorTableValue;
    type HighContrastConfig: Copy;
    fn Matrix(matrix: &[f32; 20]) -> Option<Self::Filter>;
    fn Blend(color: Color4f, mode: BlendMode) -> Option<Self::Filter>;
    fn SRGBToLinearGamma() -> Option<Self::Filter>;
    fn LinearToSRGBGamma() -> Option<Self::Filter>;
    fn Luma() -> Option<Self::Filter>;
    fn MakeColorTable(
        a: &[u8; 256],
        r: &[u8; 256],
        g: &[u8; 256],
        b: &[u8; 256],
    ) -> Arc<Self::ColorTable>;
    fn Table(table: &Self::ColorTable) -> Option<Self::Filter>;
    fn HighContrast(config: &Self::HighContrastConfig) -> Option<Self::Filter>;
}
pub trait ColorFilterWriter<B: ColorFilterBackend> {
    fn SerializedSizeScalarElements(count: usize) -> usize;
    fn SerializedSizeColor() -> usize;
    fn SerializedSizeBlendMode() -> usize;
    fn SerializedSizeOfBytes(count: usize) -> usize;
    fn SerializedSizeConfig() -> usize;
    fn WriteScalar(&mut self, value: f32);
    fn WriteColor(&mut self, value: Color4f);
    fn WriteBlendMode(&mut self, value: BlendMode);
    fn WriteData(&mut self, value: &[u8]);
    fn WriteConfig(&mut self, value: B::HighContrastConfig);
}
pub trait ColorFilterReader<B: ColorFilterBackend> {
    fn valid(&self) -> bool;
    fn ReadScalar(&mut self) -> f32;
    fn ReadColor(&mut self) -> Color4f;
    fn ReadBlendMode(&mut self) -> BlendMode;
    fn ReadData(&mut self, value: &mut [u8]);
    fn ReadConfig(&mut self) -> B::HighContrastConfig;
}
// Source: cc/paint/color_filter.cc:23-131. The native subclasses become
// exhaustive payload variants; Matrix has no second matrix copy in its owner.
enum Data<B: ColorFilterBackend> {
    Matrix,
    Blend {
        color: Color4f,
        blend_mode: BlendMode,
    },
    SRGBToLinear,
    LinearToSRGB,
    Luma,
    Table {
        table: Arc<B::ColorTable>,
    },
    HighContrast {
        config: B::HighContrastConfig,
    },
}
// Source: cc/paint/color_filter.h:23-26,66-74; .cc:135-141.
pub struct ColorFilter<B: ColorFilterBackend> {
    data: Data<B>,
    pub sk_color_filter_: Option<B::Filter>,
}
impl<B: ColorFilterBackend> ColorFilter<B> {
    pub fn kind(&self) -> Type {
        match self.data {
            Data::Matrix => Type::kMatrix,
            Data::Blend { .. } => Type::kBlend,
            Data::SRGBToLinear => Type::kSRGBToLinearGamma,
            Data::LinearToSRGB => Type::kLinearToSRGBGamma,
            Data::Luma => Type::kLuma,
            Data::Table { .. } => Type::kTableARGB,
            Data::HighContrast { .. } => Type::kHighContrast,
        }
    }
    // Source: cc/paint/color_filter.cc:25-28,144-146.
    pub fn MakeMatrix(matrix: &[f32; 20]) -> Arc<Self> {
        Arc::new(Self {
            data: Data::Matrix,
            sk_color_filter_: B::Matrix(matrix),
        })
    }
    // Source: cc/paint/color_filter.cc:52-59,148-152.
    pub fn MakeBlend(color: Color4f, blend_mode: BlendMode) -> Arc<Self> {
        Arc::new(Self {
            data: Data::Blend { color, blend_mode },
            sk_color_filter_: B::Blend(color, blend_mode),
        })
    }
    // Source: cc/paint/color_filter.cc:76-87,154-160.
    pub fn MakeSRGBToLinearGamma() -> Arc<Self> {
        Arc::new(Self {
            data: Data::SRGBToLinear,
            sk_color_filter_: B::SRGBToLinearGamma(),
        })
    }
    pub fn MakeLinearToSRGBGamma() -> Arc<Self> {
        Arc::new(Self {
            data: Data::LinearToSRGB,
            sk_color_filter_: B::LinearToSRGBGamma(),
        })
    }
    // Source: cc/paint/color_filter.cc:89-92,174-176.
    pub fn MakeLuma() -> Arc<Self> {
        Arc::new(Self {
            data: Data::Luma,
            sk_color_filter_: B::Luma(),
        })
    }
    // Source: cc/paint/color_filter.cc:94-116,162-172.
    pub fn MakeTableARGB(a: &[u8; 256], r: &[u8; 256], g: &[u8; 256], b: &[u8; 256]) -> Arc<Self> {
        Self::MakeTable(B::MakeColorTable(a, r, g, b))
    }
    pub fn MakeTable(table: Arc<B::ColorTable>) -> Arc<Self> {
        let native = B::Table(&table);
        Arc::new(Self {
            data: Data::Table { table },
            sk_color_filter_: native,
        })
    }
    // Source: cc/paint/color_filter.cc:118-131,178-181.
    pub fn MakeHighContrast(config: B::HighContrastConfig) -> Arc<Self> {
        Arc::new(Self {
            data: Data::HighContrast { config },
            sk_color_filter_: B::HighContrast(&config),
        })
    }
    // Source: cc/paint/color_filter.cc:183-187.
    pub fn FilterColor(&self, color: Color4f) -> Color4f {
        self.sk_color_filter_
            .as_ref()
            .map_or(color, |f| f.filterColor4f(color))
    }
    // Source: cc/paint/color_filter.cc:188-190. Source equality deliberately
    // compares subclass type only; do not replace with a payload comparison.
    pub fn EqualsForTesting(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
    // Source: cc/paint/color_filter.cc:31-34,61-64,101-103,124-126,192-194.
    pub fn SerializedDataSize<W: ColorFilterWriter<B>>(&self) -> usize {
        match self.data {
            Data::Matrix => W::SerializedSizeScalarElements(20),
            Data::Blend { .. } => W::SerializedSizeColor() + W::SerializedSizeBlendMode(),
            Data::Table { .. } => W::SerializedSizeOfBytes(256 * 4),
            Data::HighContrast { .. } => W::SerializedSizeConfig(),
            _ => 0,
        }
    }
    // Source: cc/paint/color_filter.cc:35-48,65-68,104-113,127-129,196.
    pub fn SerializeData<W: ColorFilterWriter<B>>(&self, writer: &mut W) {
        match &self.data {
            Data::Matrix => {
                let mut matrix = [
                    1., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 1., 0., 0., 0., 0., 0., 1., 0.,
                ];
                if let Some(filter) = &self.sk_color_filter_ {
                    filter.asAColorMatrix(&mut matrix);
                }
                for value in matrix {
                    writer.WriteScalar(value)
                }
            }
            Data::Blend { color, blend_mode } => {
                writer.WriteColor(*color);
                writer.WriteBlendMode(*blend_mode)
            }
            Data::Table { table } => {
                writer.WriteData(table.alphaTable());
                writer.WriteData(table.redTable());
                writer.WriteData(table.greenTable());
                writer.WriteData(table.blueTable());
            }
            Data::HighContrast { config } => writer.WriteConfig(*config),
            _ => {}
        }
    }
    // Source: cc/paint/color_filter.cc:198-240.
    pub fn Deserialize<R: ColorFilterReader<B>>(reader: &mut R, kind: Type) -> Option<Arc<Self>> {
        match kind {
            Type::kMatrix => {
                let mut matrix = [0.; 20];
                for value in &mut matrix {
                    *value = reader.ReadScalar();
                    if !reader.valid() {
                        return None;
                    }
                }
                Some(Self::MakeMatrix(&matrix))
            }
            Type::kBlend => {
                let color = reader.ReadColor();
                let blend = reader.ReadBlendMode();
                if !reader.valid() {
                    return None;
                }
                Some(Self::MakeBlend(color, blend))
            }
            Type::kSRGBToLinearGamma => Some(Self::MakeSRGBToLinearGamma()),
            Type::kLinearToSRGBGamma => Some(Self::MakeLinearToSRGBGamma()),
            Type::kLuma => Some(Self::MakeLuma()),
            Type::kTableARGB => {
                let mut a = [0; 256];
                let mut r = [0; 256];
                let mut g = [0; 256];
                let mut b = [0; 256];
                reader.ReadData(&mut a);
                reader.ReadData(&mut r);
                reader.ReadData(&mut g);
                reader.ReadData(&mut b);
                if !reader.valid() {
                    return None;
                }
                Some(Self::MakeTableARGB(&a, &r, &g, &b))
            }
            Type::kHighContrast => Some(Self::MakeHighContrast(reader.ReadConfig())),
            Type::kNull => unreachable!("source NOTREACHED null color filter"),
        }
    }
}
impl<B: ColorFilterBackend> super::paint_flags::PaintValueEquals for ColorFilter<B> {
    fn EqualsForTesting(&self, other: &Self) -> bool {
        Self::EqualsForTesting(self, other)
    }
}
// Source scaffolding: cc/paint/color_filter.h:1-22,76-78; .cc:1-22,133,242.
// Translation status: complete
