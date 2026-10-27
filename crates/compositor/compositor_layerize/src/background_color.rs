use compositor_tree::geometry::RectF;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BackgroundEffect {
    SameEffect,
    OpacityOnly(f32),
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackgroundSample<C> {
    pub color: C,
    pub covered_area: f32,
    pub effect: BackgroundEffect,
}

pub trait BackgroundColorOperations<C: Copy + Default> {
    fn Alpha(&self, color: C) -> f32;
    fn MultiplyAlpha(&self, color: C, opacity: f32) -> C;
    fn SourceOver(&self, foreground: C, background: C) -> C;
}

pub fn ComputeBackgroundColor<C: Copy + Default>(
    layer_bounds: RectF,
    samples_back_to_front: impl IntoIterator<Item = BackgroundSample<C>>,
    operations: &dyn BackgroundColorOperations<C>,
) -> C {
    const K_MIN_BACKGROUND_COLOR_COVERAGE_RATIO: f32 = 0.5;
    let min_background_area =
        K_MIN_BACKGROUND_COLOR_COVERAGE_RATIO * layer_bounds.width() * layer_bounds.height();
    let mut background_colors = Vec::with_capacity(4);
    for sample in samples_back_to_front {
        if operations.Alpha(sample.color) == 0.0 || sample.covered_area < min_background_area {
            continue;
        }
        let color = match sample.effect {
            BackgroundEffect::SameEffect => sample.color,
            BackgroundEffect::OpacityOnly(opacity) => {
                operations.MultiplyAlpha(sample.color, opacity)
            }
            BackgroundEffect::Unsupported => continue,
        };
        background_colors.push(color);
        if operations.Alpha(color) == 1.0 {
            break;
        }
    }

    let Some(mut background_color) = background_colors.pop() else {
        return C::default();
    };
    for color in background_colors.into_iter().rev() {
        background_color = operations.SourceOver(color, background_color);
    }
    background_color
}
