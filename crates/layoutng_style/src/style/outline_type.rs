// cpp: layoutng_style/style/outline_type.h:10-15
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum OutlineType {
    kDontIncludeBlockInkOverflow,
    kIncludeBlockInkOverflow,
    kIncludeBlockInkOverflowForAnchor,
}

// cpp: layoutng_style/style/outline_type.h:17-20
#[allow(non_snake_case)]
pub fn ShouldIncludeBlockInkOverflow(outline_type: OutlineType) -> bool {
    outline_type == OutlineType::kIncludeBlockInkOverflow
        || outline_type == OutlineType::kIncludeBlockInkOverflowForAnchor
}

// cpp: layoutng_style/style/outline_type.h:22-24
#[allow(non_snake_case)]
pub fn ShouldIncludeBlockInkOverflowForAnchorOnly(outline_type: OutlineType) -> bool {
    outline_type == OutlineType::kIncludeBlockInkOverflowForAnchor
}
