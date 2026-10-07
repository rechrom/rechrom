use foundation::LayoutUnit;

// cpp: layoutng/internal/frame_set_layout_data.h:13-27
#[derive(Clone)]
pub struct FrameSetLayoutData {
    pub col_sizes: Vec<LayoutUnit>,
    pub row_sizes: Vec<LayoutUnit>,
    pub col_allow_border: Vec<bool>,
    pub row_allow_border: Vec<bool>,
    pub border_thickness: i32,
    pub has_border_color: bool,
}
