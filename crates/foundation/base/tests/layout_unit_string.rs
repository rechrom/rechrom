use foundation::blink_geometry::geometry::layout_unit::{
    InlineLayoutUnit, LayoutUnit, TextRunLayoutUnit,
};

#[test]
fn fixed_point_string_matches_cpp_reference() {
    for line in include_str!("layout_unit_string_reference.tsv").lines() {
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(columns.len(), 3, "{line}");
        let actual = match columns[0] {
            "layout" => LayoutUnit::FromRawValue(columns[1].parse().unwrap()).ToString(),
            "text" => TextRunLayoutUnit::FromRawValue(columns[1].parse().unwrap()).ToString(),
            "inline" => InlineLayoutUnit::FromRawValue(columns[1].parse().unwrap()).ToString(),
            other => panic!("unknown C++ type: {other}"),
        };
        assert_eq!(actual, columns[2], "{line}");
    }
}
