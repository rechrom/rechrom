// Source: src/sksl/ir/SkSLLayout.h:17-17
pub use crate::src::sksl::SkSLContext::Context;
// Source: src/sksl/ir/SkSLLayout.h:18-18
pub use crate::src::sksl::SkSLPosition::Position;
// Source: src/sksl/ir/SkSLLayout.h:20-63
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutFlag(pub i32);
impl LayoutFlag {
    pub const kNone: Self = Self(0);
    pub const kAll: Self = Self(!0);
    pub const kOriginUpperLeft: Self = Self(1 << 0);
    pub const kPushConstant: Self = Self(1 << 1);
    pub const kBlendSupportAllEquations: Self = Self(1 << 2);
    pub const kColor: Self = Self(1 << 3);
    pub const kLocation: Self = Self(1 << 4);
    pub const kOffset: Self = Self(1 << 5);
    pub const kBinding: Self = Self(1 << 6);
    pub const kTexture: Self = Self(1 << 7);
    pub const kSampler: Self = Self(1 << 8);
    pub const kIndex: Self = Self(1 << 9);
    pub const kSet: Self = Self(1 << 10);
    pub const kBuiltin: Self = Self(1 << 11);
    pub const kInputAttachmentIndex: Self = Self(1 << 12);
    pub const kVulkan: Self = Self(1 << 13);
    pub const kMetal: Self = Self(1 << 14);
    pub const kWebGPU: Self = Self(1 << 15);
    pub const kDirect3D: Self = Self(1 << 16);
    pub const kAllBackends: Self =
        Self(Self::kVulkan.0 | Self::kMetal.0 | Self::kWebGPU.0 | Self::kDirect3D.0);
    pub const kRGBA8: Self = Self(1 << 17);
    pub const kRGBA32F: Self = Self(1 << 18);
    pub const kR32F: Self = Self(1 << 19);
    pub const kAllPixelFormats: Self = Self(Self::kRGBA8.0 | Self::kRGBA32F.0 | Self::kR32F.0);
    pub const kLocalSizeX: Self = Self(1 << 20);
    pub const kLocalSizeY: Self = Self(1 << 21);
    pub const kLocalSizeZ: Self = Self(1 << 22);
}
// Source: src/sksl/ir/SkSLLayout.h:68-68
pub type LayoutFlags = LayoutFlag;
// Source: src/sksl/ir/SkSLLayout.h:112-130
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub fFlags: LayoutFlags,
    pub fLocation: i32,
    pub fOffset: i32,
    pub fBinding: i32,
    pub fTexture: i32,
    pub fSampler: i32,
    pub fIndex: i32,
    pub fSet: i32,
    pub fBuiltin: i32,
    pub fInputAttachmentIndex: i32,
    pub fLocalSizeX: i32,
    pub fLocalSizeY: i32,
    pub fLocalSizeZ: i32,
}
impl Layout {
    // Source: src/sksl/ir/SkSLLayout.h:76-85
    pub fn new(
        flags: LayoutFlags,
        location: i32,
        offset: i32,
        binding: i32,
        index: i32,
        set: i32,
        builtin: i32,
        inputAttachmentIndex: i32,
    ) -> Self {
        Self {
            fFlags: flags,
            fLocation: location,
            fOffset: offset,
            fBinding: binding,
            fIndex: index,
            fSet: set,
            fBuiltin: builtin,
            fInputAttachmentIndex: inputAttachmentIndex,
            ..Self::default()
        }
    }
}
// Source: src/sksl/ir/SkSLLayout.h:87-87
impl Default for Layout {
    fn default() -> Self {
        Self {
            fFlags: LayoutFlag::kNone,
            fLocation: -1,
            fOffset: -1,
            fBinding: -1,
            fTexture: -1,
            fSampler: -1,
            fIndex: -1,
            fSet: -1,
            fBuiltin: -1,
            fInputAttachmentIndex: -1,
            fLocalSizeX: -1,
            fLocalSizeY: -1,
            fLocalSizeZ: -1,
        }
    }
}
impl Layout {
    // Source: src/sksl/ir/SkSLLayout.h:89-93
    pub fn builtin(builtin: i32) -> Self {
        let mut result = Self::default();
        result.fBuiltin = builtin;
        result
    }
}
// Rust native bitmask substitutions; no backend implementation is introduced.
impl core::ops::BitOr for LayoutFlag {
    type Output = Self;
    fn bitor(self, r: Self) -> Self {
        Self(self.0 | r.0)
    }
}
impl core::ops::BitAnd for LayoutFlag {
    type Output = Self;
    fn bitand(self, r: Self) -> Self {
        Self(self.0 & r.0)
    }
}
impl core::ops::Not for LayoutFlag {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

impl Layout {
    // Source: src/sksl/ir/SkSLLayout.cpp:19-95
    pub fn paddedDescription(&self) -> Vec<u8> {
        let mut result = Vec::new();
        let mut separator = crate::src::sksl::SkSLString::Separator();
        if self.fFlags.0 & LayoutFlag::kVulkan.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"vulkan");
        }
        if self.fFlags.0 & LayoutFlag::kMetal.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"metal");
        }
        if self.fFlags.0 & LayoutFlag::kWebGPU.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"webgpu");
        }
        if self.fFlags.0 & LayoutFlag::kDirect3D.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"direct3d");
        }
        if self.fFlags.0 & LayoutFlag::kRGBA8.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"rgba8");
        }
        if self.fFlags.0 & LayoutFlag::kRGBA32F.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"rgba32f");
        }
        if self.fFlags.0 & LayoutFlag::kR32F.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"r32f");
        }
        if self.fLocation >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"location = ");
            result.extend_from_slice(self.fLocation.to_string().as_bytes());
        }
        if self.fOffset >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"offset = ");
            result.extend_from_slice(self.fOffset.to_string().as_bytes());
        }
        if self.fBinding >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"binding = ");
            result.extend_from_slice(self.fBinding.to_string().as_bytes());
        }
        if self.fTexture >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"texture = ");
            result.extend_from_slice(self.fTexture.to_string().as_bytes());
        }
        if self.fSampler >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"sampler = ");
            result.extend_from_slice(self.fSampler.to_string().as_bytes());
        }
        if self.fIndex >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"index = ");
            result.extend_from_slice(self.fIndex.to_string().as_bytes());
        }
        if self.fSet >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"set = ");
            result.extend_from_slice(self.fSet.to_string().as_bytes());
        }
        if self.fBuiltin >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"builtin = ");
            result.extend_from_slice(self.fBuiltin.to_string().as_bytes());
        }
        if self.fInputAttachmentIndex >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"input_attachment_index = ");
            result.extend_from_slice(self.fInputAttachmentIndex.to_string().as_bytes());
        }
        if self.fFlags.0 & LayoutFlag::kOriginUpperLeft.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"origin_upper_left");
        }
        if self.fFlags.0 & LayoutFlag::kBlendSupportAllEquations.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"blend_support_all_equations");
        }
        if self.fFlags.0 & LayoutFlag::kPushConstant.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"push_constant");
        }
        if self.fFlags.0 & LayoutFlag::kColor.0 != 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"color");
        }
        if self.fLocalSizeX >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"local_size_x = ");
            result.extend_from_slice(self.fLocalSizeX.to_string().as_bytes());
        }
        if self.fLocalSizeY >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"local_size_y = ");
            result.extend_from_slice(self.fLocalSizeY.to_string().as_bytes());
        }
        if self.fLocalSizeZ >= 0 {
            result.extend_from_slice(separator());
            result.extend_from_slice(b"local_size_z = ");
            result.extend_from_slice(self.fLocalSizeZ.to_string().as_bytes());
        }
        if !result.is_empty() {
            let mut wrapped = b"layout (".to_vec();
            wrapped.extend_from_slice(&result);
            wrapped.extend_from_slice(b") ");
            result = wrapped;
        }
        result
    }
    // Source: src/sksl/ir/SkSLLayout.cpp:97-103
    pub fn description(&self) -> Vec<u8> {
        let mut s = self.paddedDescription();
        if !s.is_empty() {
            s.pop();
        }
        s
    }
    // Source: src/sksl/ir/SkSLLayout.cpp:105-180
    pub unsafe fn checkPermittedLayout(
        &self,
        context: &crate::src::sksl::SkSLContext::Context,
        pos: crate::src::sksl::SkSLPosition::Position,
        permitted: LayoutFlags,
    ) -> bool {
        use crate::src::sksl::SkSLErrorReporter::ErrorReporterExt;
        const FLAGS: &[(LayoutFlag, &[u8])] = &[
            (LayoutFlag::kOriginUpperLeft, b"origin_upper_left"),
            (LayoutFlag::kPushConstant, b"push_constant"),
            (
                LayoutFlag::kBlendSupportAllEquations,
                b"blend_support_all_equations",
            ),
            (LayoutFlag::kColor, b"color"),
            (LayoutFlag::kLocation, b"location"),
            (LayoutFlag::kOffset, b"offset"),
            (LayoutFlag::kBinding, b"binding"),
            (LayoutFlag::kTexture, b"texture"),
            (LayoutFlag::kSampler, b"sampler"),
            (LayoutFlag::kIndex, b"index"),
            (LayoutFlag::kSet, b"set"),
            (LayoutFlag::kBuiltin, b"builtin"),
            (LayoutFlag::kInputAttachmentIndex, b"input_attachment_index"),
            (LayoutFlag::kVulkan, b"vulkan"),
            (LayoutFlag::kMetal, b"metal"),
            (LayoutFlag::kWebGPU, b"webgpu"),
            (LayoutFlag::kDirect3D, b"direct3d"),
            (LayoutFlag::kRGBA8, b"rgba8"),
            (LayoutFlag::kRGBA32F, b"rgba32f"),
            (LayoutFlag::kR32F, b"r32f"),
            (LayoutFlag::kLocalSizeX, b"local_size_x"),
            (LayoutFlag::kLocalSizeY, b"local_size_y"),
            (LayoutFlag::kLocalSizeZ, b"local_size_z"),
        ];
        let mut success = true;
        let mut layout = self.fFlags.0;
        let mut permitted = permitted.0;
        let mut errors = context.fErrors;
        let backends = layout & LayoutFlag::kAllBackends.0;
        if backends.count_ones() > 1 {
            errors
                .as_mut()
                .error(pos, b"only one backend qualifier can be used");
            success = false;
        }
        let formats = layout & LayoutFlag::kAllPixelFormats.0;
        if formats.count_ones() > 1 {
            errors
                .as_mut()
                .error(pos, b"only one pixel format qualifier can be used");
            success = false;
        }
        if layout & (LayoutFlag::kTexture.0 | LayoutFlag::kSampler.0) != 0
            && layout & LayoutFlag::kBinding.0 != 0
        {
            errors.as_mut().error(
                pos,
                b"'binding' modifier cannot coexist with 'texture'/'sampler'",
            );
            success = false;
        }
        if layout & (LayoutFlag::kMetal.0 | LayoutFlag::kWebGPU.0 | LayoutFlag::kDirect3D.0) == 0 {
            permitted &= !LayoutFlag::kTexture.0;
            permitted &= !LayoutFlag::kSampler.0;
        }
        if layout & (LayoutFlag::kVulkan.0 | LayoutFlag::kWebGPU.0) == 0 {
            permitted &= !LayoutFlag::kPushConstant.0;
        }
        if layout & LayoutFlag::kMetal.0 != 0 {
            permitted &= !LayoutFlag::kSet.0;
        }
        for (flag, name) in FLAGS {
            if layout & flag.0 != 0 {
                if permitted & flag.0 == 0 {
                    let mut message = b"layout qualifier '".to_vec();
                    message.extend_from_slice(name);
                    message.extend_from_slice(b"' is not permitted here");
                    errors.as_mut().error(pos, &message);
                    success = false;
                }
                layout &= !flag.0;
            }
        }
        debug_assert_eq!(layout, 0);
        success
    }
}

// Source: src/sksl/ir/SkSLLayout.cpp:182-196
impl PartialEq for Layout {
    fn eq(&self, other: &Self) -> bool {
        self.fFlags == other.fFlags
            && self.fLocation == other.fLocation
            && self.fOffset == other.fOffset
            && self.fBinding == other.fBinding
            && self.fTexture == other.fTexture
            && self.fSampler == other.fSampler
            && self.fIndex == other.fIndex
            && self.fSet == other.fSet
            && self.fBuiltin == other.fBuiltin
            && self.fInputAttachmentIndex == other.fInputAttachmentIndex
            && self.fLocalSizeX == other.fLocalSizeX
            && self.fLocalSizeY == other.fLocalSizeY
            && self.fLocalSizeZ == other.fLocalSizeZ
    }
}
