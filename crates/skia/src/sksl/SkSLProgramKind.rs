// Source: src/sksl/SkSLProgramKind.h:19-32
#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramKind {
    kFragment,
    kVertex,
    kCompute,
    kGraphiteFragment,
    kGraphiteVertex,
    kRuntimeColorFilter,
    kRuntimeShader,
    kRuntimeBlender,
    kPrivateRuntimeColorFilter,
    kPrivateRuntimeShader,
    kPrivateRuntimeBlender,
    kMeshVertex,
    kMeshFragment,
}
