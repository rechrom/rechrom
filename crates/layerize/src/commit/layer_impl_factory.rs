//! The push-target factory of the main/impl commit.
use compositor_input::scrollbar::ScrollbarOrientation;

/// The impl tree in which a main Layer allocates its LayerImpl: the
/// `LayerTreeImpl*` argument of Chromium's virtual Layer::CreateLayerImpl and
/// its overrides. Each method is one override's constructor call.
/// Source: cc/layers/layer.cc:1619-1622; cc/layers/picture_layer.cc:38-41;
///         cc/layers/solid_color_layer.cc:14-17; cc/layers/surface_layer.cc
///         CreateLayerImpl; cc/layers/painted_scrollbar_layer.cc
///         CreateLayerImpl; cc/layers/solid_color_scrollbar_layer.cc:16-21.
/// The impl half implements it over the actual pending tree; main layers
/// create impl layers only through it and never name the impl tree type.
pub trait LayerImplFactory<L> {
    fn CreateLayer(&mut self, id: i32) -> Box<L>;
    fn CreatePictureLayer(&mut self, id: i32) -> Box<L>;
    fn CreateSolidColorLayer(&mut self, id: i32) -> Box<L>;
    fn CreateSurfaceLayer(&mut self, id: i32) -> Box<L>;
    fn CreatePaintedScrollbarLayer(
        &mut self,
        id: i32,
        orientation: ScrollbarOrientation,
        is_left_side_vertical_scrollbar: bool,
    ) -> Box<L>;
    fn CreateSolidColorScrollbarLayer(
        &mut self,
        id: i32,
        orientation: ScrollbarOrientation,
        thumb_thickness: i32,
        track_start: i32,
        is_left_side_vertical_scrollbar: bool,
    ) -> Box<L>;
}
