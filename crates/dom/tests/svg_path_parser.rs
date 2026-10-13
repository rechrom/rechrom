use dom::svg_path_parser::ParseSVGPathDefault;
use layoutng_assembly::internal::layout_input::{Offset, PaintPathVerb};

fn near(actual: f64, expected: f64, tolerance: f64) -> bool {
    (actual - expected).abs() <= tolerance
}
fn expect_point(actual: Offset, x: f64, y: f64) {
    assert!(near(actual.x, x, 1e-6) && near(actual.y, y, 1e-6));
}

// cpp: dom/svg_path_parser_test.cc:24-66
#[test]
fn retained_commands_flattened_curves_and_invalid_paths() {
    let cubic = ParseSVGPathDefault("M0 0 C0 10 10 10 10 0 S20 -10 20 0").unwrap();
    assert_eq!(cubic.commands.len(), 3);
    assert_eq!(cubic.commands[1].verb, PaintPathVerb::kCubicTo);
    assert_eq!(cubic.commands[2].verb, PaintPathVerb::kCubicTo);
    assert!(cubic.flattened_points.len() > 3);
    expect_point(cubic.commands[2].control1, 10.0, -10.0);
    expect_point(cubic.commands[2].point, 20.0, 0.0);

    let quadratic = ParseSVGPathDefault("M0 0 Q5 10 10 0 T20 0").unwrap();
    assert_eq!(quadratic.commands.len(), 3);
    assert_eq!(quadratic.commands[2].verb, PaintPathVerb::kQuadraticTo);
    expect_point(quadratic.commands[2].control1, 15.0, -10.0);

    let relative = ParseSVGPathDefault("m1 2 h3 v4 l-3 0 z").unwrap();
    assert_eq!(relative.commands.len(), 5);
    assert_eq!(
        relative.commands.last().unwrap().verb,
        PaintPathVerb::kClose
    );
    expect_point(relative.commands[3].point, 1.0, 6.0);
    expect_point(*relative.flattened_points.last().unwrap(), 1.0, 2.0);

    let arc = ParseSVGPathDefault("M0 0 A10 10 0 0 1 20 0").unwrap();
    assert_eq!(arc.commands.len(), 3);
    assert_eq!(arc.commands[1].verb, PaintPathVerb::kConicTo);
    assert_eq!(arc.commands[2].verb, PaintPathVerb::kConicTo);
    assert!(arc.flattened_points.len() >= 5);
    expect_point(arc.commands.last().unwrap().point, 20.0, 0.0);
    assert!(near(arc.bounds_width, 20.0, 0.01));
    assert!(near(arc.bounds_height, 10.0, 0.1));

    let multiple = ParseSVGPathDefault("M0 0 L1 1 M2 2 L3 3").unwrap();
    assert!(!multiple.single_subpath);

    for invalid in ["L0 0", "M0 0 A1 1 0 2 0 3 3", "M0 0 C1 2", "M0 0 Lnan 1"] {
        assert!(
            ParseSVGPathDefault(invalid).is_none(),
            "accepted: {invalid}"
        );
    }
}

#[test]
fn repeated_relative_moveto_pairs_remain_relative_lines() {
    let path = ParseSVGPathDefault("M0 0zm7.251 10.324.004-5.073-.002-2.253").unwrap();
    let tail = &path.commands[path.commands.len() - 3..];
    assert_eq!(tail[0].verb, PaintPathVerb::kMoveTo);
    assert_eq!(tail[1].verb, PaintPathVerb::kLineTo);
    assert_eq!(tail[2].verb, PaintPathVerb::kLineTo);
    expect_point(tail[0].point, 7.251, 10.324);
    expect_point(tail[1].point, 7.255, 5.251);
    expect_point(tail[2].point, 7.253, 2.998);
}

#[test]
fn consecutive_explicit_moves_replace_the_empty_contour() {
    for (source, normalized) in [
        (
            "M337.5,337.5 m-320,0 a320,320 0 1,1 640,0 a320,320 0 1,1 -640,0",
            "M17.5,337.5 a320,320 0 1,1 640,0 a320,320 0 1,1 -640,0",
        ),
        ("M900 900 M10 20 m20 30 L100 80", "M30 50 L100 80"),
        ("M900 900 M10 20 Q30 50 100 20", "M10 20 Q30 50 100 20"),
        (
            "M900 900 M10 20 C30 50 60 50 100 20 Z",
            "M10 20 C30 50 60 50 100 20 Z",
        ),
        (
            "M0 0 L10 0 M900 900 M20 20 L30 20",
            "M0 0 L10 0 M20 20 L30 20",
        ),
        ("M900 900 M10 20", "M10 20"),
    ] {
        let actual = ParseSVGPathDefault(source).unwrap();
        let expected = ParseSVGPathDefault(normalized).unwrap();
        assert_eq!(actual.commands, expected.commands, "{source}");
        assert_eq!(
            actual.flattened_points, expected.flattened_points,
            "{source}"
        );
        assert_eq!(actual.single_subpath, expected.single_subpath, "{source}");
        assert_eq!(actual.bounds_offset, expected.bounds_offset, "{source}");
        assert_eq!(actual.bounds_width, expected.bounds_width, "{source}");
        assert_eq!(actual.bounds_height, expected.bounds_height, "{source}");
    }
}

#[test]
fn moveto_after_a_close_or_drawn_contour_stays_a_separate_contour() {
    for source in ["M10 20 Z M30 40 L50 40", "M10 20 L30 20 M40 40 L50 40"] {
        let path = ParseSVGPathDefault(source).unwrap();
        assert!(!path.single_subpath, "{source}");
        assert_eq!(
            path.commands
                .iter()
                .filter(|c| c.verb == PaintPathVerb::kMoveTo)
                .count(),
            2
        );
    }
}
