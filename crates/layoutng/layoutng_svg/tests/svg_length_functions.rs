use foundation::gfx::SizeF;
use foundation::{Length, RuntimeEnabledFeatures};
use layoutng_svg::svg_length_functions::{ValueForLength, VectorForLengthPair};

#[test]
fn zoom_and_percentage_follow_source_branch() {
    assert!(!RuntimeEnabledFeatures::SvgNewZoomEnabled());
    assert_eq!(ValueForLength(&Length::Fixed(10.0), 2.0, 200.0), 5.0);
    assert_eq!(ValueForLength(&Length::Percent(25.0), 2.0, 200.0), 50.0);
    assert_eq!(ValueForLength(&Length::Auto(), 2.0, 200.0), 0.0);
}

#[test]
fn auto_axis_uses_zero_viewport_dimension() {
    let pair = VectorForLengthPair(
        &Length::Auto(),
        &Length::Percent(50.0),
        1.0,
        &SizeF::new(100.0, 40.0),
    );
    assert_eq!((pair.x(), pair.y()), (0.0, 20.0));
}
