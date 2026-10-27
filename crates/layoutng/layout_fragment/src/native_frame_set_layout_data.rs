use foundation::LayoutUnit;

// cpp: layoutng/internal/frame_set_layout_data.h:13-27
pub struct FrameSetLayoutData {
    pub col_sizes: Vec<LayoutUnit>,
    pub row_sizes: Vec<LayoutUnit>,
    pub col_allow_border: Vec<bool>,
    pub row_allow_border: Vec<bool>,
    pub border_thickness: i32,
    pub has_border_color: bool,
}

impl Clone for FrameSetLayoutData {
    #[inline(never)]
    fn clone(&self) -> Self {
        Self {
            col_sizes: self.col_sizes.clone(),
            row_sizes: self.row_sizes.clone(),
            col_allow_border: self.col_allow_border.clone(),
            row_allow_border: self.row_allow_border.clone(),
            border_thickness: self.border_thickness,
            has_border_color: self.has_border_color,
        }
    }
}
