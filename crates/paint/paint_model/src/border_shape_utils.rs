#![allow(non_snake_case)]

use layoutng_assembly::fragment_tree::{FragmentBoxSides, PaintProperties};
use layoutng_assembly::internal::layout_input::{
    Edges, Offset, PaintPathCommand, PaintPathVerb, Size,
};
use layoutng_assembly::internal::paint_input::{
    PaintCornerRadii, PaintCornerRadius, PaintStyleData,
};

use crate::PaintRect;

// cpp: paint/border_shape_utils.cc:11-17
fn ValidateRadius(radius: &PaintCornerRadius) {
    if !radius.x.is_finite() || !radius.y.is_finite() || radius.x < 0.0 || radius.y < 0.0 {
        panic!("border radii must be finite and nonnegative");
    }
}

// cpp: paint/border_shape_utils.cc:19-22
fn MakeDegenerateCornerSquare(radius: &mut PaintCornerRadius) {
    if radius.x == 0.0 || radius.y == 0.0 {
        *radius = PaintCornerRadius::default();
    }
}

// cpp: paint/border_shape_utils.cc:24-48
fn Normalize(radii: &mut PaintCornerRadii, size: Size) {
    if !size.width.is_finite() || !size.height.is_finite() || size.width < 0.0 || size.height < 0.0
    {
        panic!("rounded rectangle size is invalid");
    }
    let mut scale = 1.0_f64;
    let mut constrain = |available: f64, sum: f64| {
        if sum > 0.0 {
            scale = scale.min(available / sum);
        }
    };
    constrain(size.width, radii.top_left.x + radii.top_right.x);
    constrain(size.width, radii.bottom_left.x + radii.bottom_right.x);
    constrain(size.height, radii.top_left.y + radii.bottom_left.y);
    constrain(size.height, radii.top_right.y + radii.bottom_right.y);
    if scale >= 1.0 {
        return;
    }
    for corner in [
        &mut radii.top_left,
        &mut radii.top_right,
        &mut radii.bottom_right,
        &mut radii.bottom_left,
    ] {
        corner.x *= scale;
        corner.y *= scale;
    }
}

// cpp: paint/border_shape_utils.h:7-7
// cpp: paint/border_shape_utils.cc:52-57
pub fn UniformCornerRadii(radius: f64) -> PaintCornerRadii {
    if !radius.is_finite() || radius < 0.0 {
        panic!("border radius must be finite and nonnegative");
    }
    let corner = PaintCornerRadius {
        x: radius,
        y: radius,
    };
    PaintCornerRadii {
        top_left: corner,
        top_right: corner,
        bottom_right: corner,
        bottom_left: corner,
    }
}

// cpp: paint/border_shape_utils.h:12-12
// cpp: paint/border_shape_utils.cc:59-64
pub fn IncludedBorderEdges(paint: &PaintProperties) -> Edges {
    Edges {
        top: if paint.border_sides.top {
            paint.border.top
        } else {
            0.0
        },
        right: if paint.border_sides.right {
            paint.border.right
        } else {
            0.0
        },
        bottom: if paint.border_sides.bottom {
            paint.border.bottom
        } else {
            0.0
        },
        left: if paint.border_sides.left {
            paint.border.left
        } else {
            0.0
        },
    }
}

// cpp: paint/border_shape_utils.h:17-20
// cpp: paint/border_shape_utils.cc:66-130
pub fn ResolveCornerRadii(
    style: &PaintStyleData,
    sides: &FragmentBoxSides,
    outer_size: Size,
    inset: Edges,
) -> PaintCornerRadii {
    let mut radii = style
        .border_radii
        .unwrap_or_else(|| UniformCornerRadii(style.border_radius));
    let percentages = &style.border_radii_percentages;
    radii.top_left.x += percentages.top_left.x * outer_size.width / 100.0;
    radii.top_left.y += percentages.top_left.y * outer_size.height / 100.0;
    radii.top_right.x += percentages.top_right.x * outer_size.width / 100.0;
    radii.top_right.y += percentages.top_right.y * outer_size.height / 100.0;
    radii.bottom_right.x += percentages.bottom_right.x * outer_size.width / 100.0;
    radii.bottom_right.y += percentages.bottom_right.y * outer_size.height / 100.0;
    radii.bottom_left.x += percentages.bottom_left.x * outer_size.width / 100.0;
    radii.bottom_left.y += percentages.bottom_left.y * outer_size.height / 100.0;
    for corner in [
        &mut radii.top_left,
        &mut radii.top_right,
        &mut radii.bottom_right,
        &mut radii.bottom_left,
    ] {
        ValidateRadius(corner);
        MakeDegenerateCornerSquare(corner);
    }
    for value in [inset.top, inset.right, inset.bottom, inset.left] {
        if !value.is_finite() || value < 0.0 {
            panic!("rounded rectangle inset is invalid");
        }
    }
    radii.top_left = PaintCornerRadius {
        x: (radii.top_left.x - inset.left).max(0.0),
        y: (radii.top_left.y - inset.top).max(0.0),
    };
    radii.top_right = PaintCornerRadius {
        x: (radii.top_right.x - inset.right).max(0.0),
        y: (radii.top_right.y - inset.top).max(0.0),
    };
    radii.bottom_right = PaintCornerRadius {
        x: (radii.bottom_right.x - inset.right).max(0.0),
        y: (radii.bottom_right.y - inset.bottom).max(0.0),
    };
    radii.bottom_left = PaintCornerRadius {
        x: (radii.bottom_left.x - inset.left).max(0.0),
        y: (radii.bottom_left.y - inset.bottom).max(0.0),
    };
    for corner in [
        &mut radii.top_left,
        &mut radii.top_right,
        &mut radii.bottom_right,
        &mut radii.bottom_left,
    ] {
        MakeDegenerateCornerSquare(corner);
    }
    if !sides.top || !sides.left {
        radii.top_left = PaintCornerRadius::default();
    }
    if !sides.top || !sides.right {
        radii.top_right = PaintCornerRadius::default();
    }
    if !sides.bottom || !sides.right {
        radii.bottom_right = PaintCornerRadius::default();
    }
    if !sides.bottom || !sides.left {
        radii.bottom_left = PaintCornerRadius::default();
    }
    Normalize(
        &mut radii,
        Size {
            width: (outer_size.width - inset.left - inset.right).max(0.0),
            height: (outer_size.height - inset.top - inset.bottom).max(0.0),
        },
    );
    radii
}

// cpp: paint/border_shape_utils.h:22-24
// cpp: paint/border_shape_utils.cc:132-136
pub fn ExpandCornerRadiiUniform(
    radii: PaintCornerRadii,
    outset: f64,
    expanded_size: Size,
) -> PaintCornerRadii {
    ExpandCornerRadiiAxes(radii, outset, outset, expanded_size)
}

// cpp: paint/border_shape_utils.h:26-29
// cpp: paint/border_shape_utils.cc:138-156
pub fn ExpandCornerRadiiAxes(
    mut radii: PaintCornerRadii,
    horizontal_outset: f64,
    vertical_outset: f64,
    expanded_size: Size,
) -> PaintCornerRadii {
    if !horizontal_outset.is_finite() || !vertical_outset.is_finite() {
        panic!("rounded rectangle outset is invalid");
    }
    for corner in [
        &mut radii.top_left,
        &mut radii.top_right,
        &mut radii.bottom_right,
        &mut radii.bottom_left,
    ] {
        if corner.x > 0.0 && corner.y > 0.0 {
            corner.x = (corner.x + horizontal_outset).max(0.0);
            corner.y = (corner.y + vertical_outset).max(0.0);
        }
    }
    Normalize(&mut radii, expanded_size);
    radii
}

// cpp: paint/border_shape_utils.h:31-33
// cpp: paint/border_shape_utils.cc:158-181
pub fn ExpandCornerRadiiEdges(
    mut radii: PaintCornerRadii,
    outsets: Edges,
    expanded_size: Size,
) -> PaintCornerRadii {
    for value in [outsets.top, outsets.right, outsets.bottom, outsets.left] {
        if !value.is_finite() {
            panic!("rounded rectangle outset is invalid");
        }
    }
    let expand = |corner: &mut PaintCornerRadius, x: f64, y: f64| {
        if corner.x > 0.0 && corner.y > 0.0 {
            corner.x = (corner.x + x).max(0.0);
            corner.y = (corner.y + y).max(0.0);
        }
    };
    expand(&mut radii.top_left, outsets.left, outsets.top);
    expand(&mut radii.top_right, outsets.right, outsets.top);
    expand(&mut radii.bottom_right, outsets.right, outsets.bottom);
    expand(&mut radii.bottom_left, outsets.left, outsets.bottom);
    Normalize(&mut radii, expanded_size);
    radii
}

// cpp: paint/border_shape_utils.h:35-35
// cpp: paint/border_shape_utils.cc:183-193
pub fn UniformCornerRadius(radii: &PaintCornerRadii) -> f64 {
    let radius = radii.top_left.x;
    if radii.top_left.y != radius
        || radii.top_right.x != radius
        || radii.top_right.y != radius
        || radii.bottom_right.x != radius
        || radii.bottom_right.y != radius
        || radii.bottom_left.x != radius
        || radii.bottom_left.y != radius
    {
        return 0.0;
    }
    radius
}

// cpp: paint/border_shape_utils.h:37-39
// cpp: paint/border_shape_utils.cc:195-248
pub fn RoundedRectPath(rect: &PaintRect, radii: &PaintCornerRadii) -> Vec<PaintPathCommand> {
    const ARC: f64 = 0.5522847498307936;
    let left = rect.x;
    let top = rect.y;
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;
    let mut path = Vec::with_capacity(10);
    path.push(PaintPathCommand {
        point: Offset {
            x: left + radii.top_left.x,
            y: top,
        },
        ..PaintPathCommand::default()
    });
    path.push(PaintPathCommand {
        verb: PaintPathVerb::kLineTo,
        point: Offset {
            x: right - radii.top_right.x,
            y: top,
        },
        ..PaintPathCommand::default()
    });
    if radii.top_right.x > 0.0 && radii.top_right.y > 0.0 {
        path.push(PaintPathCommand {
            verb: PaintPathVerb::kCubicTo,
            control1: Offset {
                x: right - radii.top_right.x * (1.0 - ARC),
                y: top,
            },
            control2: Offset {
                x: right,
                y: top + radii.top_right.y * (1.0 - ARC),
            },
            point: Offset {
                x: right,
                y: top + radii.top_right.y,
            },
            ..PaintPathCommand::default()
        });
    }
    path.push(PaintPathCommand {
        verb: PaintPathVerb::kLineTo,
        point: Offset {
            x: right,
            y: bottom - radii.bottom_right.y,
        },
        ..PaintPathCommand::default()
    });
    if radii.bottom_right.x > 0.0 && radii.bottom_right.y > 0.0 {
        path.push(PaintPathCommand {
            verb: PaintPathVerb::kCubicTo,
            control1: Offset {
                x: right,
                y: bottom - radii.bottom_right.y * (1.0 - ARC),
            },
            control2: Offset {
                x: right - radii.bottom_right.x * (1.0 - ARC),
                y: bottom,
            },
            point: Offset {
                x: right - radii.bottom_right.x,
                y: bottom,
            },
            ..PaintPathCommand::default()
        });
    }
    path.push(PaintPathCommand {
        verb: PaintPathVerb::kLineTo,
        point: Offset {
            x: left + radii.bottom_left.x,
            y: bottom,
        },
        ..PaintPathCommand::default()
    });
    if radii.bottom_left.x > 0.0 && radii.bottom_left.y > 0.0 {
        path.push(PaintPathCommand {
            verb: PaintPathVerb::kCubicTo,
            control1: Offset {
                x: left + radii.bottom_left.x * (1.0 - ARC),
                y: bottom,
            },
            control2: Offset {
                x: left,
                y: bottom - radii.bottom_left.y * (1.0 - ARC),
            },
            point: Offset {
                x: left,
                y: bottom - radii.bottom_left.y,
            },
            ..PaintPathCommand::default()
        });
    }
    path.push(PaintPathCommand {
        verb: PaintPathVerb::kLineTo,
        point: Offset {
            x: left,
            y: top + radii.top_left.y,
        },
        ..PaintPathCommand::default()
    });
    if radii.top_left.x > 0.0 && radii.top_left.y > 0.0 {
        path.push(PaintPathCommand {
            verb: PaintPathVerb::kCubicTo,
            control1: Offset {
                x: left,
                y: top + radii.top_left.y * (1.0 - ARC),
            },
            control2: Offset {
                x: left + radii.top_left.x * (1.0 - ARC),
                y: top,
            },
            point: Offset {
                x: left + radii.top_left.x,
                y: top,
            },
            ..PaintPathCommand::default()
        });
    }
    path.push(PaintPathCommand {
        verb: PaintPathVerb::kClose,
        ..PaintPathCommand::default()
    });
    path
}

// cpp: paint/border_shape_utils.h:43-45
// cpp: paint/border_shape_utils.cc:250-277
pub fn ConicRoundedRectPath(rect: &PaintRect, radii: &PaintCornerRadii) -> Vec<PaintPathCommand> {
    const QUARTER_CIRCLE_WEIGHT: f64 = 0.7071067811865475;
    let left = rect.x;
    let top = rect.y;
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;
    let mut path = Vec::with_capacity(10);
    path.push(PaintPathCommand {
        point: Offset {
            x: left + radii.top_left.x,
            y: top,
        },
        ..PaintPathCommand::default()
    });
    fn line_to(path: &mut Vec<PaintPathCommand>, point: Offset) {
        if path.last().is_none_or(|last| last.point != point) {
            path.push(PaintPathCommand {
                verb: PaintPathVerb::kLineTo,
                point,
                ..PaintPathCommand::default()
            });
        }
    }
    line_to(
        &mut path,
        Offset {
            x: right - radii.top_right.x,
            y: top,
        },
    );
    if radii.top_right.x > 0.0 && radii.top_right.y > 0.0 {
        path.push(PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset { x: right, y: top },
            point: Offset {
                x: right,
                y: top + radii.top_right.y,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..PaintPathCommand::default()
        });
    }
    line_to(
        &mut path,
        Offset {
            x: right,
            y: bottom - radii.bottom_right.y,
        },
    );
    if radii.bottom_right.x > 0.0 && radii.bottom_right.y > 0.0 {
        path.push(PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset {
                x: right,
                y: bottom,
            },
            point: Offset {
                x: right - radii.bottom_right.x,
                y: bottom,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..PaintPathCommand::default()
        });
    }
    line_to(
        &mut path,
        Offset {
            x: left + radii.bottom_left.x,
            y: bottom,
        },
    );
    if radii.bottom_left.x > 0.0 && radii.bottom_left.y > 0.0 {
        path.push(PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset { x: left, y: bottom },
            point: Offset {
                x: left,
                y: bottom - radii.bottom_left.y,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..PaintPathCommand::default()
        });
    }
    line_to(
        &mut path,
        Offset {
            x: left,
            y: top + radii.top_left.y,
        },
    );
    if radii.top_left.x > 0.0 && radii.top_left.y > 0.0 {
        path.push(PaintPathCommand {
            verb: PaintPathVerb::kConicTo,
            control1: Offset { x: left, y: top },
            point: Offset {
                x: left + radii.top_left.x,
                y: top,
            },
            conic_weight: QUARTER_CIRCLE_WEIGHT,
            ..PaintPathCommand::default()
        });
    }
    path.push(PaintPathCommand {
        verb: PaintPathVerb::kClose,
        ..PaintPathCommand::default()
    });
    path
}
