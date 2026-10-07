//! Subset of upstream SkRasterPipelineOpList.h.
//! Variant numbers remain the migrated function-table indices; they are not C++ ABI values.
//! `compat_` variants are fused or legacy operations without a one-to-one upstream op.
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum SkRasterPipelineOp {
    move_src_dst = 0,
    move_dst_src,
    compat_clamp_0,
    compat_clamp_a,
    premul,
    uniform_color,
    seed_shader,
    load_8888_dst,
    store_8888,
    load_a8_dst,
    store_a8,
    gather_8888,
    compat_load_mask_u8,
    compat_mask_u8,
    scale_u8,
    lerp_u8,
    scale_1_float,
    lerp_1_float,
    dstatop,
    dstin,
    dstout,
    dstover,
    srcatop,
    srcin,
    srcout,
    srcover,
    clear,
    modulate,
    multiply,
    plus_,
    screen,
    xor_,
    colorburn,
    colordodge,
    darken,
    difference,
    exclusion,
    hardlight,
    lighten,
    overlay,
    softlight,
    hue,
    saturation,
    color,
    luminosity,
    srcover_rgba_8888,
    matrix_2x3,
    compat_mirror_xy,
    compat_repeat_xy,
    compat_bilinear,
    compat_bicubic,
    clamp_x_1,
    mirror_x_1,
    repeat_x_1,
    gradient,
    evenly_spaced_2_stop_gradient,
    xy_to_unit_angle,
    xy_to_radius,
    xy_to_2pt_conical_focal_on_circle,
    xy_to_2pt_conical_well_behaved,
    xy_to_2pt_conical_smaller,
    xy_to_2pt_conical_greater,
    xy_to_2pt_conical_strip,
    compat_mask2pt_conical_nan,
    compat_mask2pt_conical_degenerates,
    apply_vector_mask,
    compat_alter2pt_conical_compensate_focal,
    compat_alter2pt_conical_unswap,
    negate_x,
    compat_apply_concentric_scale_bias,
    compat_gamma_expand2,
    compat_gamma_expand_destination2,
    compat_gamma_compress2,
    compat_gamma_expand22,
    compat_gamma_expand_destination22,
    compat_gamma_compress22,
    compat_gamma_expand_srgb,
    compat_gamma_expand_destination_srgb,
    compat_gamma_compress_srgb,
}

#[allow(non_upper_case_globals)]
impl SkRasterPipelineOp {
    pub const MoveSourceToDestination: Self = Self::move_src_dst;
    pub const MoveDestinationToSource: Self = Self::move_dst_src;
    pub const Clamp0: Self = Self::compat_clamp_0;
    pub const ClampA: Self = Self::compat_clamp_a;
    pub const Premultiply: Self = Self::premul;
    pub const UniformColor: Self = Self::uniform_color;
    pub const SeedShader: Self = Self::seed_shader;
    pub const LoadDestination: Self = Self::load_8888_dst;
    pub const Store: Self = Self::store_8888;
    pub const LoadDestinationU8: Self = Self::load_a8_dst;
    pub const StoreU8: Self = Self::store_a8;
    pub const Gather: Self = Self::gather_8888;
    pub const LoadMaskU8: Self = Self::compat_load_mask_u8;
    pub const MaskU8: Self = Self::compat_mask_u8;
    pub const ScaleU8: Self = Self::scale_u8;
    pub const LerpU8: Self = Self::lerp_u8;
    pub const Scale1Float: Self = Self::scale_1_float;
    pub const Lerp1Float: Self = Self::lerp_1_float;
    pub const DestinationAtop: Self = Self::dstatop;
    pub const DestinationIn: Self = Self::dstin;
    pub const DestinationOut: Self = Self::dstout;
    pub const DestinationOver: Self = Self::dstover;
    pub const SourceAtop: Self = Self::srcatop;
    pub const SourceIn: Self = Self::srcin;
    pub const SourceOut: Self = Self::srcout;
    pub const SourceOver: Self = Self::srcover;
    pub const Clear: Self = Self::clear;
    pub const Modulate: Self = Self::modulate;
    pub const Multiply: Self = Self::multiply;
    pub const Plus: Self = Self::plus_;
    pub const Screen: Self = Self::screen;
    pub const Xor: Self = Self::xor_;
    pub const ColorBurn: Self = Self::colorburn;
    pub const ColorDodge: Self = Self::colordodge;
    pub const Darken: Self = Self::darken;
    pub const Difference: Self = Self::difference;
    pub const Exclusion: Self = Self::exclusion;
    pub const HardLight: Self = Self::hardlight;
    pub const Lighten: Self = Self::lighten;
    pub const Overlay: Self = Self::overlay;
    pub const SoftLight: Self = Self::softlight;
    pub const Hue: Self = Self::hue;
    pub const Saturation: Self = Self::saturation;
    pub const Color: Self = Self::color;
    pub const Luminosity: Self = Self::luminosity;
    pub const SourceOverRgba: Self = Self::srcover_rgba_8888;
    pub const Transform: Self = Self::matrix_2x3;
    pub const Reflect: Self = Self::compat_mirror_xy;
    pub const Repeat: Self = Self::compat_repeat_xy;
    pub const Bilinear: Self = Self::compat_bilinear;
    pub const Bicubic: Self = Self::compat_bicubic;
    pub const PadX1: Self = Self::clamp_x_1;
    pub const ReflectX1: Self = Self::mirror_x_1;
    pub const RepeatX1: Self = Self::repeat_x_1;
    pub const Gradient: Self = Self::gradient;
    pub const EvenlySpaced2StopGradient: Self = Self::evenly_spaced_2_stop_gradient;
    pub const XYToUnitAngle: Self = Self::xy_to_unit_angle;
    pub const XYToRadius: Self = Self::xy_to_radius;
    pub const XYTo2PtConicalFocalOnCircle: Self = Self::xy_to_2pt_conical_focal_on_circle;
    pub const XYTo2PtConicalWellBehaved: Self = Self::xy_to_2pt_conical_well_behaved;
    pub const XYTo2PtConicalSmaller: Self = Self::xy_to_2pt_conical_smaller;
    pub const XYTo2PtConicalGreater: Self = Self::xy_to_2pt_conical_greater;
    pub const XYTo2PtConicalStrip: Self = Self::xy_to_2pt_conical_strip;
    pub const Mask2PtConicalNan: Self = Self::compat_mask2pt_conical_nan;
    pub const Mask2PtConicalDegenerates: Self = Self::compat_mask2pt_conical_degenerates;
    pub const ApplyVectorMask: Self = Self::apply_vector_mask;
    pub const Alter2PtConicalCompensateFocal: Self = Self::compat_alter2pt_conical_compensate_focal;
    pub const Alter2PtConicalUnswap: Self = Self::compat_alter2pt_conical_unswap;
    pub const NegateX: Self = Self::negate_x;
    pub const ApplyConcentricScaleBias: Self = Self::compat_apply_concentric_scale_bias;
    pub const GammaExpand2: Self = Self::compat_gamma_expand2;
    pub const GammaExpandDestination2: Self = Self::compat_gamma_expand_destination2;
    pub const GammaCompress2: Self = Self::compat_gamma_compress2;
    pub const GammaExpand22: Self = Self::compat_gamma_expand22;
    pub const GammaExpandDestination22: Self = Self::compat_gamma_expand_destination22;
    pub const GammaCompress22: Self = Self::compat_gamma_compress22;
    pub const GammaExpandSrgb: Self = Self::compat_gamma_expand_srgb;
    pub const GammaExpandDestinationSrgb: Self = Self::compat_gamma_expand_destination_srgb;
    pub const GammaCompressSrgb: Self = Self::compat_gamma_compress_srgb;
}
