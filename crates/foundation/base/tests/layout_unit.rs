use foundation::blink_geometry::geometry::layout_unit::{
    GetMaxSaturatedSetResultForTesting, GetMinSaturatedSetResultForTesting, InlineLayoutUnit,
    LayoutUnit, SnapSizeToPixel, SnapSizeToPixelAllowingZero, TextRunLayoutUnit,
};

#[test]
fn raw_value_operations_match_cpp_reference() {
    for line in include_str!("layout_unit_raw_reference.tsv").lines() {
        let columns: Vec<i64> = line
            .split('\t')
            .map(|part| part.parse().expect("C++ probe integer"))
            .collect();
        assert_eq!(columns.len(), 10);
        let raw = columns[0] as i32;
        let unit = LayoutUnit::FromRawValue(raw);
        assert_eq!(unit.ToInt() as i64, columns[1], "raw={raw}");
        assert_eq!(unit.Ceil() as i64, columns[2], "raw={raw}");
        assert_eq!(unit.Round() as i64, columns[3], "raw={raw}");
        assert_eq!(unit.Floor() as i64, columns[4], "raw={raw}");
        assert_eq!(unit.Fraction().RawValue() as i64, columns[5], "raw={raw}");
        assert_eq!(unit.HasFraction() as i64, columns[6], "raw={raw}");
        assert_eq!(
            unit.ClampNegativeToZero().RawValue() as i64,
            columns[7],
            "raw={raw}"
        );
        assert_eq!(
            unit.ClampPositiveToZero().RawValue() as i64,
            columns[8],
            "raw={raw}"
        );
        assert_eq!(unit.AddEpsilon().RawValue() as i64, columns[9], "raw={raw}");
    }
}

#[test]
fn constructors_saturation_and_precision_match_cpp() {
    assert_eq!(std::mem::size_of::<LayoutUnit>(), 4);
    assert_eq!(std::mem::size_of::<TextRunLayoutUnit>(), 4);
    assert_eq!(std::mem::size_of::<InlineLayoutUnit>(), 8);
    assert_eq!(LayoutUnit::from_signed(1).RawValue(), 64);
    assert_eq!(LayoutUnit::from_signed(-1).RawValue(), -64);
    assert_eq!(LayoutUnit::from_f64(1.5).RawValue(), 96);
    assert_eq!(LayoutUnit::FromFloatFloor(-0.01).RawValue(), -1);
    assert_eq!(LayoutUnit::FromFloatCeil(0.01).RawValue(), 1);
    assert_eq!(LayoutUnit::from_signed(i32::MAX), LayoutUnit::Max());
    assert_eq!(LayoutUnit::from_unsigned(u32::MAX), LayoutUnit::Max());
    assert_eq!(LayoutUnit::from_f32(f32::NAN).RawValue(), 0);
    assert_eq!(LayoutUnit::from_f64(f64::INFINITY), LayoutUnit::Max());
    assert_eq!(GetMaxSaturatedSetResultForTesting(), i32::MAX);
    assert_eq!(GetMinSaturatedSetResultForTesting(), i32::MIN);
    assert_eq!(TextRunLayoutUnit::FromRawValue(1).ToDouble(), 1.0 / 65536.0);
    assert_eq!(
        InlineLayoutUnit::from_signed(30_000_000).ToDouble(),
        30_000_000.0
    );
    for raw in (-100_000..=100_000).step_by(37) {
        let unit = LayoutUnit::FromRawValue(raw);
        assert_eq!(LayoutUnit::from_f64(unit.ToDouble()).RawValue(), raw);
    }
}

#[test]
fn fixed_point_conversions_keep_cpp_rounding() {
    assert_eq!(LayoutUnit::FromFixed::<16>(1023).RawValue(), 0);
    assert_eq!(LayoutUnit::FromFixed::<16>(1024).RawValue(), 1);
    assert_eq!(LayoutUnit::FromFixed::<16>(-1).RawValue(), -1);
    assert_eq!(TextRunLayoutUnit::FromFixed::<6>(1).RawValue(), 1024);
    let inline = InlineLayoutUnit::FromRawValue(1025);
    let layout: LayoutUnit = inline.To();
    assert_eq!(layout.RawValue(), 1);
    let ceil: LayoutUnit = inline.ToCeil();
    assert_eq!(ceil.RawValue(), 2);
    assert_eq!(
        LayoutUnit::FromFloatEncompassRound(0.01, 1.01),
        (
            LayoutUnit::FromFloatFloor(0.01),
            LayoutUnit::FromFloatCeil(1.01)
        )
    );
}

#[test]
fn arithmetic_saturates_like_platform_test() {
    assert_eq!(
        LayoutUnit::Max() + LayoutUnit::from_signed(1),
        LayoutUnit::Max()
    );
    assert_eq!(
        LayoutUnit::Min() - LayoutUnit::from_signed(1),
        LayoutUnit::Min()
    );
    assert_eq!(-LayoutUnit::Min(), LayoutUnit::Max());
    let large = LayoutUnit::from_signed(30_000_000);
    assert_eq!(
        large.MulDiv(LayoutUnit::from_signed(2), LayoutUnit::from_signed(2)),
        large
    );
    assert_eq!(
        LayoutUnit::from_signed(2) * LayoutUnit::from_signed(3),
        LayoutUnit::from_signed(6)
    );
    assert_eq!(
        LayoutUnit::from_signed(6) / LayoutUnit::from_signed(2),
        LayoutUnit::from_signed(3)
    );
}

#[test]
fn arithmetic_matches_cpp_reference() {
    for line in include_str!("layout_unit_arithmetic_reference.tsv").lines() {
        let columns: Vec<i64> = line
            .split('\t')
            .map(|part| part.parse().expect("C++ probe integer"))
            .collect();
        assert_eq!(columns.len(), 11);
        let a = LayoutUnit::FromRawValue(columns[0] as i32);
        let b = LayoutUnit::FromRawValue(columns[1] as i32);
        let pair = (columns[0], columns[1]);
        assert_eq!((a + b).RawValue() as i64, columns[2], "{pair:?} add");
        assert_eq!((a - b).RawValue() as i64, columns[3], "{pair:?} sub");
        assert_eq!((-a).RawValue() as i64, columns[4], "{pair:?} neg");
        assert_eq!((a * b).RawValue() as i64, columns[5], "{pair:?} mul");
        assert_eq!((a / b).RawValue() as i64, columns[6], "{pair:?} div");
        assert_eq!((a % b).RawValue() as i64, columns[7], "{pair:?} rem");
        assert_eq!(
            a.MulDiv(b, LayoutUnit::from_signed(2)).RawValue() as i64,
            columns[8],
            "{pair:?} MulDiv"
        );
        assert_eq!(SnapSizeToPixel(a, b) as i64, columns[9], "{pair:?} snap");
        assert_eq!(
            SnapSizeToPixelAllowingZero(a, b) as i64,
            columns[10],
            "{pair:?} snap allowing zero"
        );
    }
}

#[test]
fn floating_conversions_match_cpp_reference() {
    for line in include_str!("layout_unit_float_reference.tsv").lines() {
        let columns: Vec<&str> = line.split('\t').collect();
        match columns[0] {
            "f32" => {
                assert_eq!(columns.len(), 6);
                let bits: u32 = columns[1].parse().unwrap();
                let value = f32::from_bits(bits);
                assert_eq!(
                    LayoutUnit::from_f32(value).RawValue(),
                    columns[2].parse::<i32>().unwrap(),
                    "bits={bits} constructor"
                );
                assert_eq!(
                    LayoutUnit::FromFloatCeil(value).RawValue(),
                    columns[3].parse::<i32>().unwrap(),
                    "bits={bits} ceil"
                );
                assert_eq!(
                    LayoutUnit::FromFloatFloor(value).RawValue(),
                    columns[4].parse::<i32>().unwrap(),
                    "bits={bits} floor"
                );
                assert_eq!(
                    LayoutUnit::FromFloatRound(value).RawValue(),
                    columns[5].parse::<i32>().unwrap(),
                    "bits={bits} round"
                );
            }
            "f64" => {
                assert_eq!(columns.len(), 4);
                let bits: u64 = columns[1].parse().unwrap();
                let value = f64::from_bits(bits);
                assert_eq!(
                    LayoutUnit::from_f64(value).RawValue(),
                    columns[2].parse::<i32>().unwrap(),
                    "bits={bits} constructor"
                );
                assert_eq!(
                    LayoutUnit::FromDoubleRound(value).RawValue(),
                    columns[3].parse::<i32>().unwrap(),
                    "bits={bits} round"
                );
            }
            _ => panic!("unknown C++ reference row"),
        }
    }
}
