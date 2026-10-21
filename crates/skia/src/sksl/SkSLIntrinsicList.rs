// Source: src/sksl/SkSLIntrinsicList.h:19-125
impl IntrinsicKind {
    pub const k_abs_IntrinsicKind: Self = Self(0);
    pub const k_acosh_IntrinsicKind: Self = Self(1);
    pub const k_acos_IntrinsicKind: Self = Self(2);
    pub const k_all_IntrinsicKind: Self = Self(3);
    pub const k_any_IntrinsicKind: Self = Self(4);
    pub const k_asinh_IntrinsicKind: Self = Self(5);
    pub const k_asin_IntrinsicKind: Self = Self(6);
    pub const k_atanh_IntrinsicKind: Self = Self(7);
    pub const k_atan_IntrinsicKind: Self = Self(8);
    pub const k_atomicAdd_IntrinsicKind: Self = Self(9);
    pub const k_atomicLoad_IntrinsicKind: Self = Self(10);
    pub const k_atomicMax_IntrinsicKind: Self = Self(11);
    pub const k_atomicMin_IntrinsicKind: Self = Self(12);
    pub const k_atomicStore_IntrinsicKind: Self = Self(13);
    pub const k_bitCount_IntrinsicKind: Self = Self(14);
    pub const k_ceil_IntrinsicKind: Self = Self(15);
    pub const k_clamp_IntrinsicKind: Self = Self(16);
    pub const k_cosh_IntrinsicKind: Self = Self(17);
    pub const k_cos_IntrinsicKind: Self = Self(18);
    pub const k_cross_IntrinsicKind: Self = Self(19);
    pub const k_degrees_IntrinsicKind: Self = Self(20);
    pub const k_determinant_IntrinsicKind: Self = Self(21);
    pub const k_dFdx_IntrinsicKind: Self = Self(22);
    pub const k_dFdy_IntrinsicKind: Self = Self(23);
    pub const k_distance_IntrinsicKind: Self = Self(24);
    pub const k_dot_IntrinsicKind: Self = Self(25);
    pub const k_equal_IntrinsicKind: Self = Self(26);
    pub const k_eval_IntrinsicKind: Self = Self(27);
    pub const k_exp2_IntrinsicKind: Self = Self(28);
    pub const k_exp_IntrinsicKind: Self = Self(29);
    pub const k_faceforward_IntrinsicKind: Self = Self(30);
    pub const k_findLSB_IntrinsicKind: Self = Self(31);
    pub const k_findMSB_IntrinsicKind: Self = Self(32);
    pub const k_floatBitsToInt_IntrinsicKind: Self = Self(33);
    pub const k_floatBitsToUint_IntrinsicKind: Self = Self(34);
    pub const k_floor_IntrinsicKind: Self = Self(35);
    pub const k_fma_IntrinsicKind: Self = Self(36);
    pub const k_fract_IntrinsicKind: Self = Self(37);
    pub const k_frexp_IntrinsicKind: Self = Self(38);
    pub const k_fromLinearSrgb_IntrinsicKind: Self = Self(39);
    pub const k_fwidth_IntrinsicKind: Self = Self(40);
    pub const k_greaterThanEqual_IntrinsicKind: Self = Self(41);
    pub const k_greaterThan_IntrinsicKind: Self = Self(42);
    pub const k_intBitsToFloat_IntrinsicKind: Self = Self(43);
    pub const k_inversesqrt_IntrinsicKind: Self = Self(44);
    pub const k_inverse_IntrinsicKind: Self = Self(45);
    pub const k_isinf_IntrinsicKind: Self = Self(46);
    pub const k_isnan_IntrinsicKind: Self = Self(47);
    pub const k_ldexp_IntrinsicKind: Self = Self(48);
    pub const k_length_IntrinsicKind: Self = Self(49);
    pub const k_lessThanEqual_IntrinsicKind: Self = Self(50);
    pub const k_lessThan_IntrinsicKind: Self = Self(51);
    pub const k_log2_IntrinsicKind: Self = Self(52);
    pub const k_log_IntrinsicKind: Self = Self(53);
    pub const k_matrixCompMult_IntrinsicKind: Self = Self(54);
    pub const k_matrixInverse_IntrinsicKind: Self = Self(55);
    pub const k_max_IntrinsicKind: Self = Self(56);
    pub const k_min_IntrinsicKind: Self = Self(57);
    pub const k_mix_IntrinsicKind: Self = Self(58);
    pub const k_modf_IntrinsicKind: Self = Self(59);
    pub const k_mod_IntrinsicKind: Self = Self(60);
    pub const k_normalize_IntrinsicKind: Self = Self(61);
    pub const k_notEqual_IntrinsicKind: Self = Self(62);
    pub const k_not_IntrinsicKind: Self = Self(63);
    pub const k_outerProduct_IntrinsicKind: Self = Self(64);
    pub const k_packHalf2x16_IntrinsicKind: Self = Self(65);
    pub const k_packSnorm2x16_IntrinsicKind: Self = Self(66);
    pub const k_packSnorm4x8_IntrinsicKind: Self = Self(67);
    pub const k_packUnorm2x16_IntrinsicKind: Self = Self(68);
    pub const k_packUnorm4x8_IntrinsicKind: Self = Self(69);
    pub const k_pow_IntrinsicKind: Self = Self(70);
    pub const k_radians_IntrinsicKind: Self = Self(71);
    pub const k_reflect_IntrinsicKind: Self = Self(72);
    pub const k_refract_IntrinsicKind: Self = Self(73);
    pub const k_roundEven_IntrinsicKind: Self = Self(74);
    pub const k_round_IntrinsicKind: Self = Self(75);
    pub const k_sample_IntrinsicKind: Self = Self(76);
    pub const k_sampleGrad_IntrinsicKind: Self = Self(77);
    pub const k_sampleLod_IntrinsicKind: Self = Self(78);
    pub const k_saturate_IntrinsicKind: Self = Self(79);
    pub const k_sign_IntrinsicKind: Self = Self(80);
    pub const k_sinh_IntrinsicKind: Self = Self(81);
    pub const k_sin_IntrinsicKind: Self = Self(82);
    pub const k_smoothstep_IntrinsicKind: Self = Self(83);
    pub const k_sqrt_IntrinsicKind: Self = Self(84);
    pub const k_step_IntrinsicKind: Self = Self(85);
    pub const k_storageBarrier_IntrinsicKind: Self = Self(86);
    pub const k_subpassLoad_IntrinsicKind: Self = Self(87);
    pub const k_tanh_IntrinsicKind: Self = Self(88);
    pub const k_tan_IntrinsicKind: Self = Self(89);
    pub const k_textureHeight_IntrinsicKind: Self = Self(90);
    pub const k_textureRead_IntrinsicKind: Self = Self(91);
    pub const k_textureSize_IntrinsicKind: Self = Self(92);
    pub const k_textureWidth_IntrinsicKind: Self = Self(93);
    pub const k_textureWrite_IntrinsicKind: Self = Self(94);
    pub const k_toLinearSrgb_IntrinsicKind: Self = Self(95);
    pub const k_transpose_IntrinsicKind: Self = Self(96);
    pub const k_trunc_IntrinsicKind: Self = Self(97);
    pub const k_uintBitsToFloat_IntrinsicKind: Self = Self(98);
    pub const k_unpackHalf2x16_IntrinsicKind: Self = Self(99);
    pub const k_unpackSnorm2x16_IntrinsicKind: Self = Self(100);
    pub const k_unpackSnorm4x8_IntrinsicKind: Self = Self(101);
    pub const k_unpackUnorm2x16_IntrinsicKind: Self = Self(102);
    pub const k_unpackUnorm4x8_IntrinsicKind: Self = Self(103);
    pub const k_workgroupBarrier_IntrinsicKind: Self = Self(104);
    pub const k_workgroupUniformLoad_IntrinsicKind: Self = Self(105);
}
// Source: src/sksl/SkSLIntrinsicList.h:129-135
// Preserve all native fixed-int8 enum casts,including unnamed values.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntrinsicKind(pub i8);
impl IntrinsicKind {
    pub const kNotIntrinsic: Self = Self(-1);
}

// Source: src/sksl/SkSLIntrinsicList.h:138-138
// Adapted string_view key uses immutable borrowed byte slices and native Hash32.
pub type IntrinsicMap =
    crate::src::core::SkTHash::THashMap<&'static [u8], IntrinsicKind, IntrinsicHash>;
#[derive(Default)]
pub struct IntrinsicHash;
impl crate::src::core::SkTHash::NativeHash<&'static [u8]> for IntrinsicHash {
    fn hash(&self, value: &&'static [u8]) -> u32 {
        crate::src::core::SkChecksum::Hash32(value, 0)
    }
}
// Source: src/sksl/SkSLIntrinsicList.cpp:16-24
pub fn GetIntrinsicMap() -> &'static IntrinsicMap {
    static MAP: std::sync::OnceLock<IntrinsicMap> = std::sync::OnceLock::new();
    MAP.get_or_init(|| {
        IntrinsicMap::from_pairs(&[
            (b"abs".as_slice(), IntrinsicKind::k_abs_IntrinsicKind),
            (b"acosh".as_slice(), IntrinsicKind::k_acosh_IntrinsicKind),
            (b"acos".as_slice(), IntrinsicKind::k_acos_IntrinsicKind),
            (b"all".as_slice(), IntrinsicKind::k_all_IntrinsicKind),
            (b"any".as_slice(), IntrinsicKind::k_any_IntrinsicKind),
            (b"asinh".as_slice(), IntrinsicKind::k_asinh_IntrinsicKind),
            (b"asin".as_slice(), IntrinsicKind::k_asin_IntrinsicKind),
            (b"atanh".as_slice(), IntrinsicKind::k_atanh_IntrinsicKind),
            (b"atan".as_slice(), IntrinsicKind::k_atan_IntrinsicKind),
            (
                b"atomicAdd".as_slice(),
                IntrinsicKind::k_atomicAdd_IntrinsicKind,
            ),
            (
                b"atomicLoad".as_slice(),
                IntrinsicKind::k_atomicLoad_IntrinsicKind,
            ),
            (
                b"atomicMax".as_slice(),
                IntrinsicKind::k_atomicMax_IntrinsicKind,
            ),
            (
                b"atomicMin".as_slice(),
                IntrinsicKind::k_atomicMin_IntrinsicKind,
            ),
            (
                b"atomicStore".as_slice(),
                IntrinsicKind::k_atomicStore_IntrinsicKind,
            ),
            (
                b"bitCount".as_slice(),
                IntrinsicKind::k_bitCount_IntrinsicKind,
            ),
            (b"ceil".as_slice(), IntrinsicKind::k_ceil_IntrinsicKind),
            (b"clamp".as_slice(), IntrinsicKind::k_clamp_IntrinsicKind),
            (b"cosh".as_slice(), IntrinsicKind::k_cosh_IntrinsicKind),
            (b"cos".as_slice(), IntrinsicKind::k_cos_IntrinsicKind),
            (b"cross".as_slice(), IntrinsicKind::k_cross_IntrinsicKind),
            (
                b"degrees".as_slice(),
                IntrinsicKind::k_degrees_IntrinsicKind,
            ),
            (
                b"determinant".as_slice(),
                IntrinsicKind::k_determinant_IntrinsicKind,
            ),
            (b"dFdx".as_slice(), IntrinsicKind::k_dFdx_IntrinsicKind),
            (b"dFdy".as_slice(), IntrinsicKind::k_dFdy_IntrinsicKind),
            (
                b"distance".as_slice(),
                IntrinsicKind::k_distance_IntrinsicKind,
            ),
            (b"dot".as_slice(), IntrinsicKind::k_dot_IntrinsicKind),
            (b"equal".as_slice(), IntrinsicKind::k_equal_IntrinsicKind),
            (b"eval".as_slice(), IntrinsicKind::k_eval_IntrinsicKind),
            (b"exp2".as_slice(), IntrinsicKind::k_exp2_IntrinsicKind),
            (b"exp".as_slice(), IntrinsicKind::k_exp_IntrinsicKind),
            (
                b"faceforward".as_slice(),
                IntrinsicKind::k_faceforward_IntrinsicKind,
            ),
            (
                b"findLSB".as_slice(),
                IntrinsicKind::k_findLSB_IntrinsicKind,
            ),
            (
                b"findMSB".as_slice(),
                IntrinsicKind::k_findMSB_IntrinsicKind,
            ),
            (
                b"floatBitsToInt".as_slice(),
                IntrinsicKind::k_floatBitsToInt_IntrinsicKind,
            ),
            (
                b"floatBitsToUint".as_slice(),
                IntrinsicKind::k_floatBitsToUint_IntrinsicKind,
            ),
            (b"floor".as_slice(), IntrinsicKind::k_floor_IntrinsicKind),
            (b"fma".as_slice(), IntrinsicKind::k_fma_IntrinsicKind),
            (b"fract".as_slice(), IntrinsicKind::k_fract_IntrinsicKind),
            (b"frexp".as_slice(), IntrinsicKind::k_frexp_IntrinsicKind),
            (
                b"fromLinearSrgb".as_slice(),
                IntrinsicKind::k_fromLinearSrgb_IntrinsicKind,
            ),
            (b"fwidth".as_slice(), IntrinsicKind::k_fwidth_IntrinsicKind),
            (
                b"greaterThanEqual".as_slice(),
                IntrinsicKind::k_greaterThanEqual_IntrinsicKind,
            ),
            (
                b"greaterThan".as_slice(),
                IntrinsicKind::k_greaterThan_IntrinsicKind,
            ),
            (
                b"intBitsToFloat".as_slice(),
                IntrinsicKind::k_intBitsToFloat_IntrinsicKind,
            ),
            (
                b"inversesqrt".as_slice(),
                IntrinsicKind::k_inversesqrt_IntrinsicKind,
            ),
            (
                b"inverse".as_slice(),
                IntrinsicKind::k_inverse_IntrinsicKind,
            ),
            (b"isinf".as_slice(), IntrinsicKind::k_isinf_IntrinsicKind),
            (b"isnan".as_slice(), IntrinsicKind::k_isnan_IntrinsicKind),
            (b"ldexp".as_slice(), IntrinsicKind::k_ldexp_IntrinsicKind),
            (b"length".as_slice(), IntrinsicKind::k_length_IntrinsicKind),
            (
                b"lessThanEqual".as_slice(),
                IntrinsicKind::k_lessThanEqual_IntrinsicKind,
            ),
            (
                b"lessThan".as_slice(),
                IntrinsicKind::k_lessThan_IntrinsicKind,
            ),
            (b"log2".as_slice(), IntrinsicKind::k_log2_IntrinsicKind),
            (b"log".as_slice(), IntrinsicKind::k_log_IntrinsicKind),
            (
                b"matrixCompMult".as_slice(),
                IntrinsicKind::k_matrixCompMult_IntrinsicKind,
            ),
            (
                b"matrixInverse".as_slice(),
                IntrinsicKind::k_matrixInverse_IntrinsicKind,
            ),
            (b"max".as_slice(), IntrinsicKind::k_max_IntrinsicKind),
            (b"min".as_slice(), IntrinsicKind::k_min_IntrinsicKind),
            (b"mix".as_slice(), IntrinsicKind::k_mix_IntrinsicKind),
            (b"modf".as_slice(), IntrinsicKind::k_modf_IntrinsicKind),
            (b"mod".as_slice(), IntrinsicKind::k_mod_IntrinsicKind),
            (
                b"normalize".as_slice(),
                IntrinsicKind::k_normalize_IntrinsicKind,
            ),
            (
                b"notEqual".as_slice(),
                IntrinsicKind::k_notEqual_IntrinsicKind,
            ),
            (b"not".as_slice(), IntrinsicKind::k_not_IntrinsicKind),
            (
                b"outerProduct".as_slice(),
                IntrinsicKind::k_outerProduct_IntrinsicKind,
            ),
            (
                b"packHalf2x16".as_slice(),
                IntrinsicKind::k_packHalf2x16_IntrinsicKind,
            ),
            (
                b"packSnorm2x16".as_slice(),
                IntrinsicKind::k_packSnorm2x16_IntrinsicKind,
            ),
            (
                b"packSnorm4x8".as_slice(),
                IntrinsicKind::k_packSnorm4x8_IntrinsicKind,
            ),
            (
                b"packUnorm2x16".as_slice(),
                IntrinsicKind::k_packUnorm2x16_IntrinsicKind,
            ),
            (
                b"packUnorm4x8".as_slice(),
                IntrinsicKind::k_packUnorm4x8_IntrinsicKind,
            ),
            (b"pow".as_slice(), IntrinsicKind::k_pow_IntrinsicKind),
            (
                b"radians".as_slice(),
                IntrinsicKind::k_radians_IntrinsicKind,
            ),
            (
                b"reflect".as_slice(),
                IntrinsicKind::k_reflect_IntrinsicKind,
            ),
            (
                b"refract".as_slice(),
                IntrinsicKind::k_refract_IntrinsicKind,
            ),
            (
                b"roundEven".as_slice(),
                IntrinsicKind::k_roundEven_IntrinsicKind,
            ),
            (b"round".as_slice(), IntrinsicKind::k_round_IntrinsicKind),
            (b"sample".as_slice(), IntrinsicKind::k_sample_IntrinsicKind),
            (
                b"sampleGrad".as_slice(),
                IntrinsicKind::k_sampleGrad_IntrinsicKind,
            ),
            (
                b"sampleLod".as_slice(),
                IntrinsicKind::k_sampleLod_IntrinsicKind,
            ),
            (
                b"saturate".as_slice(),
                IntrinsicKind::k_saturate_IntrinsicKind,
            ),
            (b"sign".as_slice(), IntrinsicKind::k_sign_IntrinsicKind),
            (b"sinh".as_slice(), IntrinsicKind::k_sinh_IntrinsicKind),
            (b"sin".as_slice(), IntrinsicKind::k_sin_IntrinsicKind),
            (
                b"smoothstep".as_slice(),
                IntrinsicKind::k_smoothstep_IntrinsicKind,
            ),
            (b"sqrt".as_slice(), IntrinsicKind::k_sqrt_IntrinsicKind),
            (b"step".as_slice(), IntrinsicKind::k_step_IntrinsicKind),
            (
                b"storageBarrier".as_slice(),
                IntrinsicKind::k_storageBarrier_IntrinsicKind,
            ),
            (
                b"subpassLoad".as_slice(),
                IntrinsicKind::k_subpassLoad_IntrinsicKind,
            ),
            (b"tanh".as_slice(), IntrinsicKind::k_tanh_IntrinsicKind),
            (b"tan".as_slice(), IntrinsicKind::k_tan_IntrinsicKind),
            (
                b"textureHeight".as_slice(),
                IntrinsicKind::k_textureHeight_IntrinsicKind,
            ),
            (
                b"textureRead".as_slice(),
                IntrinsicKind::k_textureRead_IntrinsicKind,
            ),
            (
                b"textureSize".as_slice(),
                IntrinsicKind::k_textureSize_IntrinsicKind,
            ),
            (
                b"textureWidth".as_slice(),
                IntrinsicKind::k_textureWidth_IntrinsicKind,
            ),
            (
                b"textureWrite".as_slice(),
                IntrinsicKind::k_textureWrite_IntrinsicKind,
            ),
            (
                b"toLinearSrgb".as_slice(),
                IntrinsicKind::k_toLinearSrgb_IntrinsicKind,
            ),
            (
                b"transpose".as_slice(),
                IntrinsicKind::k_transpose_IntrinsicKind,
            ),
            (b"trunc".as_slice(), IntrinsicKind::k_trunc_IntrinsicKind),
            (
                b"uintBitsToFloat".as_slice(),
                IntrinsicKind::k_uintBitsToFloat_IntrinsicKind,
            ),
            (
                b"unpackHalf2x16".as_slice(),
                IntrinsicKind::k_unpackHalf2x16_IntrinsicKind,
            ),
            (
                b"unpackSnorm2x16".as_slice(),
                IntrinsicKind::k_unpackSnorm2x16_IntrinsicKind,
            ),
            (
                b"unpackSnorm4x8".as_slice(),
                IntrinsicKind::k_unpackSnorm4x8_IntrinsicKind,
            ),
            (
                b"unpackUnorm2x16".as_slice(),
                IntrinsicKind::k_unpackUnorm2x16_IntrinsicKind,
            ),
            (
                b"unpackUnorm4x8".as_slice(),
                IntrinsicKind::k_unpackUnorm4x8_IntrinsicKind,
            ),
            (
                b"workgroupBarrier".as_slice(),
                IntrinsicKind::k_workgroupBarrier_IntrinsicKind,
            ),
            (
                b"workgroupUniformLoad".as_slice(),
                IntrinsicKind::k_workgroupUniformLoad_IntrinsicKind,
            ),
        ])
    })
}
// Source: src/sksl/SkSLIntrinsicList.cpp:26-34
pub fn FindIntrinsicKind(mut name: &[u8]) -> IntrinsicKind {
    if name.first() == Some(&b'$') {
        name = &name[1..];
    }
    let map = GetIntrinsicMap();
    // Language-only lease: immutable borrowed lookup key is not retained by find.
    let key: &'static [u8] = unsafe { core::mem::transmute(name) };
    map.find(&key)
        .copied()
        .unwrap_or(IntrinsicKind::kNotIntrinsic)
}
