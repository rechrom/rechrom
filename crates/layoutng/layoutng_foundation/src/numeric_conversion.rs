// C++: foundation/style_values/css/numeric_conversion.h.

// cpp: foundation/style_values/css/numeric_conversion.h:34-41
#[unsafe(no_mangle)]
pub extern "Rust" fn RoundForImpreciseConversionI16(value: f64) -> i16 {
    let adjusted = value + if value < 0.0 { -0.01 } else { 0.01 };
    if adjusted > i16::MAX as f64 || adjusted < i16::MIN as f64 {
        0
    } else {
        adjusted as i16
    }
}
// Source core/css/css_primitive_value.h:46-53, int instantiation.
pub fn RoundForImpreciseConversionI32(value: f64) -> i32 {
    let adjusted = value + if value < 0.0 { -0.01 } else { 0.01 };
    if adjusted > i32::MAX as f64 || adjusted < i32::MIN as f64 {
        0
    } else {
        adjusted as i32
    }
}

#[cfg(test)]
mod tests {
    use super::RoundForImpreciseConversionI16 as round;

    #[test]
    fn source_near_integer_and_overflow_rules() {
        assert_eq!(round(44.99998), 45);
        assert_eq!(round(-44.99998), -45);
        assert_eq!(round(32768.0), 0);
        assert_eq!(round(-32769.0), 0);
    }
}
