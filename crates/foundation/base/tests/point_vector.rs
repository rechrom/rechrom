use foundation::blink_geometry::geometry::PhysicalOffset;
use foundation::gfx_geometry::point_conversions::{ToCeiledPoint, ToFlooredPoint, ToRoundedPoint};
use foundation::gfx_geometry::vector2d_f::{CrossProduct, DotProduct};
use foundation::gfx_geometry::{Point, PointF, ScaleToRoundedPoint, Vector2d, Vector2dF};

fn point_line(name: &str, point: Point) -> String {
    format!(
        "{name}\t{}\t{}\t{}\t{}",
        point.x(),
        point.y(),
        point.IsOrigin() as i32,
        point.ToString()
    )
}

fn vector_line(name: &str, vector: Vector2d) -> String {
    format!(
        "{name}\t{}\t{}\t{}\t{}\t{}\t{}",
        vector.x(),
        vector.y(),
        vector.IsZero() as i32,
        vector.LengthSquared(),
        vector.Length().to_bits(),
        vector.ToString(),
    )
}

fn offset_line(name: &str, offset: PhysicalOffset) -> String {
    format!(
        "{name}\t{}\t{}\t{}\t{}",
        offset.left.RawValue(),
        offset.top.RawValue(),
        offset.IsZero() as i32,
        offset.HasFraction() as i32,
    )
}

#[test]
fn points_vectors_and_offsets_match_cpp_probe() {
    let mut actual = Vec::new();
    let mut point = Point::new(10, -20);
    actual.push(point_line("point0", point));
    point.Offset(3, 4);
    actual.push(point_line("point1", point));
    point += Vector2d::new(5, -2);
    actual.push(point_line("point2", point));
    actual.push(point_line("point3", ScaleToRoundedPoint(point, 1.25, -0.5)));
    actual.push(point_line(
        "point4",
        ToFlooredPoint(PointF::new(-1.25, 2.75)),
    ));
    actual.push(point_line(
        "point5",
        ToCeiledPoint(PointF::new(-1.25, 2.75)),
    ));

    let mut vector = Vector2d::new(3, 4);
    actual.push(vector_line("vector0", vector));
    vector.Add(&Vector2d::new(-10, 2));
    actual.push(vector_line("vector1", vector));
    actual.push(vector_line("vector2", -vector));

    let vector_f = Vector2dF::new(3.0, 4.0);
    actual.push(format!(
        "vector_f\t{}\t{}\t{}\t{}\t{}",
        vector_f.Length().to_bits(),
        vector_f.SlopeAngleRadians().to_bits(),
        CrossProduct(&vector_f, &Vector2dF::new(2.0, 5.0)),
        DotProduct(&vector_f, &Vector2dF::new(2.0, 5.0)),
        vector_f.ToString(),
    ));

    actual.push(offset_line("offset0", PhysicalOffset::default()));
    let mut offset = PhysicalOffset::FromPointFRound(&PointF::new(1.25, -2.75));
    actual.push(offset_line("offset1", offset));
    actual.push(point_line(
        "offset_point",
        ToRoundedPoint(PointF::from(offset)),
    ));
    offset.Scale(2.0);
    actual.push(offset_line("offset2", offset));
    actual.push(offset_line("offset3", -offset));

    let expected: Vec<&str> = include_str!("point_vector_reference.tsv").lines().collect();
    assert_eq!(actual.len(), expected.len());
    for (got, want) in actual.iter().zip(expected) {
        assert_eq!(got, want);
    }
}
