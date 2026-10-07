// C++: src/foundation/style_values/style/display_adjustment.h:8-18
// The free function declared in lines 20-22 is implemented by the
// layoutng_style package; keeping it there avoids a Cargo dependency cycle.

#[derive(Clone, Copy, Debug)]
pub struct DisplayAdjustmentContext {
    pub is_root: bool,
    pub is_immediate_canvas_child: bool,
    pub is_input_file_shadow_child: bool,
    pub should_be_inlinified: bool,
    pub is_at_media_shadow_boundary: bool,
}

impl Default for DisplayAdjustmentContext {
    fn default() -> Self {
        Self {
            is_root: false,
            is_immediate_canvas_child: false,
            is_input_file_shadow_child: false,
            should_be_inlinified: true,
            is_at_media_shadow_boundary: false,
        }
    }
}
