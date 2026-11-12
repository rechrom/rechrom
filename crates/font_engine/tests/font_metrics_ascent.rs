use font_engine::{FontBaseline, FontMetrics};

#[test]
fn integer_ascent_preserves_platform_rounding_for_paint_origin() {
    let mut metrics = FontMetrics::default();
    metrics.SetAscent(14.5);

    assert_eq!(metrics.FloatAscent(), 14.5);
    assert_eq!(metrics.Ascent(), 15);
}

#[test]
fn html_font_height_uses_integer_platform_metrics() {
    let mut metrics = FontMetrics::default();
    metrics.SetAscent(14.5);
    metrics.SetDescent(3.5);

    let float_height = metrics.GetFloatFontHeight(FontBaseline::kAlphabeticBaseline);
    assert_eq!(float_height.ascent.ToDouble(), 14.5);
    assert_eq!(float_height.descent.ToDouble(), 3.5);

    let html_height = metrics.GetFontHeight(FontBaseline::kAlphabeticBaseline);
    // Chromium FontMetrics::GetFontHeight() intentionally uses the integer
    // platform ascent/descent. Fractional metrics are exposed separately by
    // GetFloatFontHeight().
    assert_eq!(html_height.ascent.ToDouble(), 15.0);
    assert_eq!(html_height.descent.ToDouble(), 4.0);
}

#[test]
fn dezoomed_device_metrics_preserve_fractional_css_height() {
    let mut metrics = FontMetrics::default();
    metrics.SetAscent(14.5);
    metrics.SetDescent(4.0);
    metrics.SetLineSpacing(18.5);
    metrics.SetDeviceScaleFactor(2.0);

    let html_height = metrics.GetFontHeight(FontBaseline::kAlphabeticBaseline);
    assert_eq!(html_height.ascent.ToDouble(), 14.5);
    assert_eq!(html_height.descent.ToDouble(), 4.0);
    assert_eq!(metrics.FloatLineSpacing(), 18.5);
    assert_eq!(metrics.FixedLineSpacing().ToDouble(), 18.5);
}
