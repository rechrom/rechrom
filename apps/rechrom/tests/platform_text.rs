// These are rendering-path checks, not a Chromium pixel-parity oracle.
#[test]
fn default_font_metrics_do_not_require_cpp_skia() {
    let constraints = browser::CreateBrowserConstraints(320, 200);
    assert_eq!(
        constraints.font_backend_factory.is_some(),
        false,
        "production font metrics must not inject the C++ Skia backend on any platform"
    );
    assert!(constraints.fonts.iter().any(|font| !font.bytes.is_empty()));
}

#[test]
fn translated_text_raster_is_visible_and_repeatable() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let blank = "<html><body style='margin:0;background:white'></body></html>";
            let text = "<html><body style='margin:0;background:white;color:black;font:48px sans-serif'>Rechrom text</body></html>";
            let empty = browser::RenderHtml(blank, 320, 200);
            let first = browser::RenderHtml(text, 320, 200);
            let second = browser::RenderHtml(text, 320, 200);
            assert_ne!(first, empty, "translated text must produce visible pixels");
            assert_eq!(first, second, "unchanged text must rasterize repeatably");
            let decoder = png::Decoder::new(std::io::Cursor::new(first));
            let mut reader = decoder.read_info().unwrap();
            assert_eq!((reader.info().width, reader.info().height), (320, 200));
            let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
            let output = reader.next_frame(&mut pixels).unwrap();
            assert!(pixels[..output.buffer_size()].iter().any(|&byte| byte == 0));
        })
        .unwrap()
        .join()
        .unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn installed_cjk_fonts_produce_distinct_translated_glyphs() {
    let Ok(output) = std::process::Command::new("fc-match")
        .args(["--format", "%{charset}", ":charset=4e2d,6c34:scalable=true"])
        .output()
    else {
        return;
    };
    let charset = String::from_utf8_lossy(&output.stdout);
    let contains = |codepoint| {
        charset.split_whitespace().any(|range| {
            let mut bounds = range.split('-');
            let Ok(start) = u32::from_str_radix(bounds.next().unwrap_or(""), 16) else {
                return false;
            };
            let end = bounds
                .next()
                .and_then(|s| u32::from_str_radix(s, 16).ok())
                .unwrap_or(start);
            (start..=end).contains(&codepoint)
        })
    };
    if !output.status.success() || !contains(0x4e2d) || !contains(0x6c34) {
        return; // This host has no font covering both requested test glyphs.
    }
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            let page = |text| format!("<html><body style='margin:0;background:white;color:black;font:48px Arial'>{text}</body></html>");
            let first = browser::RenderHtml(&page("中"), 320, 200);
            let second = browser::RenderHtml(&page("水"), 320, 200);
            let empty = browser::RenderHtml(&page(""), 320, 200);
            assert_ne!(first, empty, "the CJK glyph must be visible");
            assert_ne!(second, empty, "the CJK glyph must be visible");
            assert_ne!(first, second, "distinct installed CJK glyphs must not collapse to the same missing-glyph box");
        })
        .unwrap()
        .join()
        .unwrap();
}
