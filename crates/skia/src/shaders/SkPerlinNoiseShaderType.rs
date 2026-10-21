// Source: src/shaders/SkPerlinNoiseShaderType.h:18-18
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkPerlinNoiseShaderType {
    kFractalNoise = 0,
    kTurbulence = 1,
}
impl SkPerlinNoiseShaderType {
    pub const kLast: Self = Self::kTurbulence;
}
