use foundation::blink_geometry::geometry::{LayoutUnit, MarginStrut};

#[test]
fn margin_strut_state_matches_cpp_reference() {
    let mut strut = MarginStrut::default();
    let reference = include_str!("margin_strut_reference.tsv");
    let mut lines = reference.lines();
    for step in 0..=6 {
        match step {
            1 => strut.Append(&LayoutUnit::FromRawValue(128), false),
            2 => strut.Append(&LayoutUnit::FromRawValue(-64), false),
            3 => strut.Append(&LayoutUnit::FromRawValue(256), true),
            4 => {
                strut.is_quirky_container_start = true;
                strut.Append(&LayoutUnit::FromRawValue(512), true);
            }
            5 => {
                strut.trim_leading_margins = true;
                strut.Append(&LayoutUnit::FromRawValue(-256), false);
            }
            6 => strut.discard_margins = true,
            _ => {}
        }
        let actual = format!(
            "{step}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            strut.positive_margin.RawValue(),
            strut.negative_margin.RawValue(),
            strut.quirky_positive_margin.RawValue(),
            strut.is_quirky_container_start as i32,
            strut.discard_margins as i32,
            strut.trim_leading_margins as i32,
            strut.Sum().RawValue(),
            strut.QuirkyContainerSum().RawValue(),
            strut.IsEmpty() as i32,
        );
        assert_eq!(actual, lines.next().unwrap());
    }
    let mut copy = strut;
    assert_eq!(
        format!("equal\t{}", (copy == strut) as i32),
        lines.next().unwrap()
    );
    copy.trim_leading_margins = false;
    assert_eq!(
        format!("different\t{}", (copy == strut) as i32),
        lines.next().unwrap()
    );
    assert!(lines.next().is_none());
}
