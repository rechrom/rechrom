use crate::include::sksl::SkSLVersion::Version;
use crate::src::sksl::{
    SkSLDefines::kDefaultInlineThreshold, SkSLModule::ModuleType, SkSLProgramKind::ProgramKind,
};
// Source: src/sksl/SkSLProgramSettings.h:26-80
// Adapted: flat native field order is preserved, with exact in-class defaults
// initialized by Rust Default below; scalar defaults are never invented.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ProgramSettings {
    pub fFragColorIsInOut: bool,
    pub fForceHighPrecision: bool,
    pub fSharpenTextures: bool,
    pub fForceNoRTFlip: bool,
    pub fRTFlipOffset: i32,
    pub fRTFlipBinding: i32,
    pub fRTFlipSet: i32,
    pub fDefaultUniformSet: i32,
    pub fDefaultUniformBinding: i32,
    pub fOptimize: bool,
    pub fRemoveDeadFunctions: bool,
    pub fRemoveDeadVariables: bool,
    pub fInlineThreshold: i32,
    pub fForceNoInline: bool,
    pub fAllowNarrowingConversions: bool,
    pub fValidateSPIRV: bool,
    pub fUseVulkanPushConstantsForGaneshRTAdjust: bool,
    pub fMaxVersionAllowed: Version,
    pub fUseMemoryPool: bool,
}
impl Default for ProgramSettings {
    fn default() -> Self {
        Self {
            fFragColorIsInOut: false,
            fForceHighPrecision: false,
            fSharpenTextures: false,
            fForceNoRTFlip: false,
            fRTFlipOffset: -1,
            fRTFlipBinding: -1,
            fRTFlipSet: -1,
            fDefaultUniformSet: 0,
            fDefaultUniformBinding: 0,
            fOptimize: true,
            fRemoveDeadFunctions: true,
            fRemoveDeadVariables: true,
            fInlineThreshold: kDefaultInlineThreshold,
            fForceNoInline: false,
            fAllowNarrowingConversions: false,
            fValidateSPIRV: true,
            fUseVulkanPushConstantsForGaneshRTAdjust: false,
            fMaxVersionAllowed: Version::k100,
            fUseMemoryPool: true,
        }
    }
}
// Source: src/sksl/SkSLProgramSettings.h:85-92
// Source: src/sksl/SkSLProgramSettings.h:100-100
// Adapted contiguous Rust storage retains fModuleType/fKind/fSettings/required
// version order. Methods94-175 stay in their original method order below.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ProgramConfig {
    pub fModuleType: ModuleType,
    pub fKind: ProgramKind,
    pub fSettings: ProgramSettings,
    pub fRequiredSkSLVersion: Version,
}
impl ProgramConfig {
    // Rust aggregate initialization helper: native first three fields are
    // mandatory inputs, never fabricated defaults; remaining native default100.
    pub fn new(module_type: ModuleType, kind: ProgramKind, settings: ProgramSettings) -> Self {
        Self {
            fModuleType: module_type,
            fKind: kind,
            fSettings: settings,
            fRequiredSkSLVersion: Version::k100,
        }
    }
    // Source: src/sksl/SkSLProgramSettings.h:94-96
    pub fn isBuiltinCode(&self) -> bool {
        self.fModuleType != ModuleType::program
    }
    // Source: src/sksl/SkSLProgramSettings.h:102-104
    pub fn enforcesSkSLVersion(&self) -> bool {
        Self::IsRuntimeEffect(self.fKind)
    }
    // Source: src/sksl/SkSLProgramSettings.h:106-112
    pub fn strictES2Mode(&self) -> bool {
        self.fSettings.fMaxVersionAllowed == Version::k100
            && self.fRequiredSkSLVersion == Version::k100
            && self.enforcesSkSLVersion()
    }
    // Source: src/sksl/SkSLProgramSettings.h:114-122
    pub fn versionDescription(&self) -> &'static str {
        if self.enforcesSkSLVersion() {
            match self.fRequiredSkSLVersion {
                Version::k100 => return "#version 100\n",
                Version::k300 => return "#version 300\n",
            }
        }
        ""
    }
    // Source: src/sksl/SkSLProgramSettings.h:124-127
    pub fn IsFragment(kind: ProgramKind) -> bool {
        kind == ProgramKind::kFragment || kind == ProgramKind::kGraphiteFragment
    }
    // Source: src/sksl/SkSLProgramSettings.h:129-132
    pub fn IsVertex(kind: ProgramKind) -> bool {
        kind == ProgramKind::kVertex || kind == ProgramKind::kGraphiteVertex
    }
    // Source: src/sksl/SkSLProgramSettings.h:134-136
    pub fn IsCompute(kind: ProgramKind) -> bool {
        kind == ProgramKind::kCompute
    }
    // Source: src/sksl/SkSLProgramSettings.h:138-147
    pub fn IsRuntimeEffect(kind: ProgramKind) -> bool {
        (kind == ProgramKind::kRuntimeColorFilter
            || kind == ProgramKind::kRuntimeShader
            || kind == ProgramKind::kRuntimeBlender
            || kind == ProgramKind::kPrivateRuntimeColorFilter
            || kind == ProgramKind::kPrivateRuntimeShader
            || kind == ProgramKind::kPrivateRuntimeBlender
            || kind == ProgramKind::kMeshVertex
            || kind == ProgramKind::kMeshFragment)
    }
    // Source: src/sksl/SkSLProgramSettings.h:149-152
    pub fn IsRuntimeShader(kind: ProgramKind) -> bool {
        (kind == ProgramKind::kRuntimeShader || kind == ProgramKind::kPrivateRuntimeShader)
    }
    // Source: src/sksl/SkSLProgramSettings.h:154-157
    pub fn IsRuntimeColorFilter(kind: ProgramKind) -> bool {
        (kind == ProgramKind::kRuntimeColorFilter
            || kind == ProgramKind::kPrivateRuntimeColorFilter)
    }
    // Source: src/sksl/SkSLProgramSettings.h:159-162
    pub fn IsRuntimeBlender(kind: ProgramKind) -> bool {
        (kind == ProgramKind::kRuntimeBlender || kind == ProgramKind::kPrivateRuntimeBlender)
    }
    // Source: src/sksl/SkSLProgramSettings.h:164-167
    pub fn IsMesh(kind: ProgramKind) -> bool {
        (kind == ProgramKind::kMeshVertex || kind == ProgramKind::kMeshFragment)
    }
    // Source: src/sksl/SkSLProgramSettings.h:169-175
    pub fn AllowsPrivateIdentifiers(kind: ProgramKind) -> bool {
        (kind != ProgramKind::kRuntimeColorFilter
            && kind != ProgramKind::kRuntimeShader
            && kind != ProgramKind::kRuntimeBlender
            && kind != ProgramKind::kMeshVertex
            && kind != ProgramKind::kMeshFragment)
    }
}
