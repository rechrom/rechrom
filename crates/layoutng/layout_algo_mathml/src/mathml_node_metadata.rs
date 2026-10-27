// Only the complete value type needed by the block-node boundary is mapped
// here. MathML node construction and algorithms remain untranslated.
// cpp: layoutng_mathml/mathml_node_metadata.h:113-115
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathScriptType {
    kSub = 0,
    kSuper = 1,
    kSubSup = 2,
    kMultiscripts = 3,
    kUnder = 4,
    kOver = 5,
    kUnderOver = 6,
}
