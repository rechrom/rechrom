use foundation::blink_geometry::geometry::physical_size::AspectRatioFit;
use foundation::blink_geometry::geometry::{LayoutRatioFromSizeF, LayoutUnit, PhysicalSize};
use foundation::gfx_geometry::size_conversions::ToRoundedSize;
use foundation::gfx_geometry::{
    ScaleToCeiledSize, ScaleToFlooredSize, ScaleToRoundedSize, Size, SizeF,
};

fn size_line(name: &str, size: Size) -> String {
    format!(
        "{name}\t{}\t{}\t{}\t{}\t{}\t{}",
        size.width(),
        size.height(),
        size.IsEmpty() as i32,
        size.IsZero() as i32,
        size.Area64(),
        size.ToString(),
    )
}

fn size_f_line(name: &str, size: SizeF) -> String {
    format!(
        "{name}\t{}\t{}\t{}\t{}\t{}\t{}",
        size.width(),
        size.height(),
        size.IsEmpty() as i32,
        size.IsZero() as i32,
        size.GetArea(),
        size.ToString(),
    )
}

fn physical_line(name: &str, size: PhysicalSize) -> String {
    format!(
        "{name}\t{}\t{}\t{}\t{}\t{}",
        size.width.RawValue(),
        size.height.RawValue(),
        size.IsEmpty() as i32,
        size.IsZero() as i32,
        size.HasFraction() as i32,
    )
}

#[test]
fn size_and_aspect_ratio_match_cpp_probe() {
    let mut actual = Vec::new();
    actual.push(size_line("size0", Size::new(-3, 4)));
    let mut size = Size::new(7, 11);
    actual.push(size_line("size1", size));
    size.Enlarge(3, -2);
    actual.push(size_line("size2", size));
    actual.push(size_line("size3", ScaleToCeiledSize(size, 1.25, 0.5)));
    actual.push(size_line("size4", ScaleToFlooredSize(size, 1.25, 0.5)));
    actual.push(size_line("size5", ScaleToRoundedSize(size, 1.25, 0.5)));
    size.SetToMax(&Size::new(100, 2));
    actual.push(size_line("size6", size));
    size.Transpose();
    actual.push(size_line("size7", size));

    actual.push(size_f_line("sizef0", SizeF::new(-1.0, 0.0000001)));
    let mut size_f = SizeF::new(2.5, 3.75);
    actual.push(size_f_line("sizef1", size_f));
    size_f.Enlarge(1.25, -0.5);
    actual.push(size_f_line("sizef2", size_f));
    size_f.Scale(2.0, 0.5);
    actual.push(size_f_line("sizef3", size_f));
    actual.push(size_line("sizef_round", ToRoundedSize(size_f)));

    actual.push(physical_line("physical0", PhysicalSize::default()));
    let physical = PhysicalSize::new(LayoutUnit::from_signed(300), LayoutUnit::from_signed(200));
    let aspect = PhysicalSize::new(LayoutUnit::from_signed(16), LayoutUnit::from_signed(9));
    actual.push(physical_line("physical1", physical));
    actual.push(physical_line(
        "physical_shrink",
        physical.FitToAspectRatio(&aspect, AspectRatioFit::kAspectRatioFitShrink),
    ));
    actual.push(physical_line(
        "physical_grow",
        physical.FitToAspectRatio(&aspect, AspectRatioFit::kAspectRatioFitGrow),
    ));
    for ratio in [
        SizeF::new(16.0, 9.0),
        SizeF::new(1.6, 1.0),
        SizeF::new(355.0, 113.0),
        SizeF::new(0.0, 0.0),
        SizeF::new(1.4142135, 1.0),
    ] {
        actual.push(physical_line("ratio", LayoutRatioFromSizeF(ratio)));
    }
    let mut expected = include_str!("size_reference.tsv").lines();
    for got in &actual {
        let want = expected.next().unwrap();
        assert_eq!(got, want);
    }
    for line in expected {
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(columns[0], "ratio_case");
        let width = f32::from_bits(columns[1].parse().unwrap());
        let height = f32::from_bits(columns[2].parse().unwrap());
        let converted = LayoutRatioFromSizeF(SizeF::new(width, height));
        assert_eq!(
            converted.width.RawValue(),
            columns[3].parse::<i32>().unwrap(),
            "{line}"
        );
        assert_eq!(
            converted.height.RawValue(),
            columns[4].parse::<i32>().unwrap(),
            "{line}"
        );
    }
}
