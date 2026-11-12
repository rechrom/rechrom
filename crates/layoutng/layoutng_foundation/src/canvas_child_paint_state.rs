//! Shared CanvasChildPaintState source value; one definition serves Canvas, Paint and Compositor.
use crate::{LayoutOffset as Offset, TransformMatrix};
use std::{collections::BTreeMap, sync::Arc};

/// Source: platform/graphics/canvas_child_paint_state.h:22-42. Geometry stays
/// in source units; image ids are the source signed PaintImage ids.
#[derive(Clone, Debug)]
pub struct CanvasChildPaintState {
    pub effective_zoom: f32,
    pub transform_origin: [f32; 3],
    pub box_size: [f32; 2],
    pub reference_box_offset: Offset,
    pub canvas_content_size: [f32; 2],
    pub canvas_device_pixel_content_box: [i32; 2],
    pub canvas_node_id: i32,
    pub canvas_child_node_id: i32,
    pub animated_image_frame_index_map: Option<Arc<BTreeMap<i32, usize>>>,
}
impl PartialEq for CanvasChildPaintState {
    fn eq(&self, other: &Self) -> bool {
        // Source: canvas_child_paint_state.cc:12-27. The child id is owned by
        // EffectPaintPropertyNode::CanvasChildState and is intentionally not
        // part of this value comparison.
        self.effective_zoom == other.effective_zoom
            && self.transform_origin == other.transform_origin
            && self.box_size == other.box_size
            && self.reference_box_offset == other.reference_box_offset
            && self.canvas_content_size == other.canvas_content_size
            && self.canvas_device_pixel_content_box == other.canvas_device_pixel_content_box
            && self.canvas_node_id == other.canvas_node_id
            && self.animated_image_frame_index_map == other.animated_image_frame_index_map
    }
}
impl Default for CanvasChildPaintState {
    fn default() -> Self {
        Self {
            effective_zoom: 1.0,
            transform_origin: [0.0; 3],
            box_size: [0.0; 2],
            reference_box_offset: Offset::default(),
            canvas_content_size: [0.0; 2],
            canvas_device_pixel_content_box: [0; 2],
            canvas_node_id: 0,
            canvas_child_node_id: 0,
            animated_image_frame_index_map: None,
        }
    }
}
impl CanvasChildPaintState {
    /// Source: canvas_child_paint_state.cc:64-78.
    pub fn GetCanvasGridScaleFactor(&self, canvas_size: [i32; 2]) -> [f32; 2] {
        if canvas_size != self.canvas_device_pixel_content_box
            && self.canvas_content_size[0] > 0.0
            && self.canvas_content_size[1] > 0.0
        {
            [
                canvas_size[0] as f32 / self.canvas_content_size[0],
                canvas_size[1] as f32 / self.canvas_content_size[1],
            ]
        } else {
            [1.0, 1.0]
        }
    }

    /// Source: canvas_child_paint_state.cc:29-62.
    pub fn GetElementTransform(
        &self,
        canvas_size: [i32; 2],
        draw_transform: &crate::gfx::Transform,
        element_canvas_transform_enabled: bool,
    ) -> crate::gfx::Transform {
        fn scale(x: f32, y: f32) -> crate::gfx::Transform {
            let mut values = TransformMatrix::default().values;
            values[0] = f64::from(x);
            values[5] = f64::from(y);
            crate::gfx::Transform::ColMajor(&values)
        }

        let physical_to_canvas_grid = self.GetCanvasGridScaleFactor(canvas_size);
        let physical_to_css = 1.0 / self.effective_zoom;
        let canvas_grid_to_css_x = physical_to_css / physical_to_canvas_grid[0];
        let canvas_grid_to_css_y = physical_to_css / physical_to_canvas_grid[1];

        let mut css_transform = scale(canvas_grid_to_css_x, canvas_grid_to_css_y);
        css_transform.PreConcat(draw_transform);
        css_transform.PreConcat(&scale(
            1.0 / canvas_grid_to_css_x,
            1.0 / canvas_grid_to_css_y,
        ));
        if element_canvas_transform_enabled {
            return css_transform;
        }

        let mut result = crate::gfx::Transform::default();
        result.Translate3d(
            -self.transform_origin[0],
            -self.transform_origin[1],
            -self.transform_origin[2],
        );
        result.PreConcat(&css_transform);
        result.Translate3d(
            self.transform_origin[0],
            self.transform_origin[1],
            self.transform_origin[2],
        );
        result
    }
}
