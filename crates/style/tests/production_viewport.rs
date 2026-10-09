use foundation::{CSSPropertyID as P, Length, String as CSSString};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    page_size_type::PageSizeType,
};
use style::{
    css_value::CSSValuePayload,
    media_queries::MediaValuesCachedData,
    parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseProperty, PropertyParseErrorKind},
    },
    properties::longhand_dispatch::LonghandApplicationError,
    resolver::production_style_builder::{Apply, StageFontDescription},
    StyleEngine,
};
fn parse(id: P, text: &str) -> std::rc::Rc<style::production_css_value::Value> {
    ParseProperty(
        id,
        &CSSString::from(text),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap()
    .remove(0)
    .ValueRef()
}
fn media() -> MediaValuesCachedData {
    MediaValuesCachedData {
        em_size: 16.,
        media_type: CSSString::from("screen"),
        viewport_width: 800.,
        viewport_height: 600.,
        ..Default::default()
    }
}
fn native<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    let doc = owner.GetDocument();
    let node = (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap();
    unsafe { &*doc.ResolvedStyleFor(node).unwrap().native_style.Get() }
}

#[test]
fn clip_rect_and_page_locale_reach_native_cascade_with_css_wide() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=p><i id=c></i><i id=r></i><i id=u></i><i id=bad></i></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#p{clip:rect(-1px, auto, 3em, calc(2px + 4px));page:ChapterCase;-webkit-locale:'fr-CA';object-view-box:none;zoom:0%} #c{clip:inherit;page:inherit;-webkit-locale:inherit;object-view-box:inherit;zoom:inherit} #r{clip:initial;page:initial;-webkit-locale:initial;object-view-box:initial;zoom:initial} #u{clip:unset;page:unset;-webkit-locale:unset;object-view-box:unset;zoom:unset} #bad{clip:rect(1px 2px 3px 4px);clip:rect(1px,2px 3px,4px);page:SavedName;page:default;-webkit-locale:'ja-JP';-webkit-locale:ja;zoom:100%;zoom:-1}"));
    let sheet = style::production_style_sheet::ParseStyleSheet(
        &CSSString::from("@page ChapterCase { size: a4 landscape; }"),
        CSSParserMode::kHTMLStandardMode,
    );
    // The Size consumer/native fields exist; the actual @page entry point is
    // still an owner boundary. Do not infer a paginated document cascade.
    assert!(sheet
        .diagnostics
        .iter()
        .any(|d| d.kind == style::production_style_sheet::DiagnosticKind::Unsupported));
    assert_eq!(sheet.contents.RuleCount(), 0);
    owner.GetDocumentMut().AppendStyleSheet(
        style::production_style_sheet_projection::ProjectStyleSheet(&sheet),
    );
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let p = native(&owner, "p");
    assert!(!p.HasAutoClip());
    assert_eq!(p.Clip().Top(), &Length::Fixed(-1));
    assert!(p.Clip().Right().IsAuto());
    assert_eq!(p.Clip().Bottom(), &Length::Fixed(48));
    assert_eq!(p.Clip().Left(), &Length::Fixed(6));
    assert_eq!(p.Page().Utf8(), "ChapterCase");
    assert_eq!(p.Locale().Utf8().to_ascii_lowercase(), "fr-ca");
    assert!(!p.GetFontDescription().Locale().is_null());
    let c = native(&owner, "c");
    assert_eq!(c.Clip(), p.Clip());
    assert_eq!(c.Page(), p.Page());
    assert_eq!(
        c.GetFontDescription().Locale(),
        p.GetFontDescription().Locale()
    );
    for id in ["r", "u"] {
        let style = native(&owner, id);
        assert!(style.HasAutoClip());
        assert!(style.Page().IsNull());
        if id == "r" {
            assert!(style.GetFontDescription().Locale().is_null());
        } else {
            assert_eq!(
                style.GetFontDescription().Locale(),
                p.GetFontDescription().Locale()
            );
        }
    }
    for id in ["p", "c", "r", "u", "bad"] {
        let style = native(&owner, id);
        assert_eq!(style.Zoom(), 1.);
        assert_eq!(style.EffectiveZoom(), 1.);
        assert!(style.ObjectViewBox().is_none());
    }
    let bad = native(&owner, "bad");
    assert_eq!(bad.Clip().Top(), &Length::Fixed(1));
    assert_eq!(bad.Clip().Right(), &Length::Fixed(2));
    assert_eq!(bad.Page().Utf8(), "SavedName");
    assert_eq!(bad.Locale().Utf8().to_ascii_lowercase(), "ja-jp");
}

#[test]
fn page_size_typed_values_convert_to_native_unzoomed_dimensions() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    let mut font = b.GetFontDescription().clone();
    font.SetSpecifiedSize(16.);
    font.SetComputedSize(32.);
    StageFontDescription(&mut b, &font);
    b.SetEffectiveZoom(2.);
    for (text, kind, width, height) in [
        ("letter landscape", PageSizeType::kFixed, 1056., 816.),
        ("portrait legal", PageSizeType::kFixed, 816., 1344.),
        ("ledger", PageSizeType::kFixed, 1056., 1632.),
        ("2em 1in", PageSizeType::kFixed, 32., 96.),
        ("calc(1in + 1in)", PageSizeType::kFixed, 192., 192.),
        ("auto", PageSizeType::kAuto, 0., 0.),
        ("landscape", PageSizeType::kLandscape, 0., 0.),
        ("portrait", PageSizeType::kPortrait, 0., 0.),
    ] {
        Apply(
            P::kSize,
            &mut b,
            None,
            &parse(P::kSize, text),
            16.,
            &media(),
        )
        .unwrap();
        assert_eq!(b.GetPageSizeType(), kind, "{text}");
        assert_eq!(b.PageSize().width(), width, "{text}");
        assert_eq!(b.PageSize().height(), height, "{text}");
    }
    for (text, mm_width, mm_height) in [
        ("a3", 297., 420.),
        ("a4", 210., 297.),
        ("a5", 148., 210.),
        ("b4", 250., 353.),
        ("b5", 176., 250.),
        ("jis-b4", 257., 364.),
        ("jis-b5", 182., 257.),
    ] {
        Apply(
            P::kSize,
            &mut b,
            None,
            &parse(P::kSize, text),
            16.,
            &media(),
        )
        .unwrap();
        assert_eq!(
            b.PageSize().width(),
            (mm_width * ((96.0f64 / 2.54) / 10.0)) as f32
        );
        assert_eq!(
            b.PageSize().height(),
            (mm_height * ((96.0f64 / 2.54) / 10.0)) as f32
        );
    }
    assert_eq!(parse(P::kSize, "portrait A4").CssText().Utf8(), "a4");
    assert_eq!(
        parse(P::kSize, "landscape A4").CssText().Utf8(),
        "a4 landscape"
    );
    let saved = b.PageSize().clone();
    let kind = b.GetPageSizeType();
    let mut parent_builder = ComputedStyleBuilder::from_style(initial);
    let parent = unsafe { &*parent_builder.TakeStyle() };
    for wide in ["initial", "inherit", "unset"] {
        Apply(
            P::kSize,
            &mut b,
            Some(parent),
            &parse(P::kSize, wide),
            16.,
            &media(),
        )
        .unwrap();
        assert_eq!(b.GetPageSizeType(), kind);
        assert_eq!(b.PageSize(), &saved);
    }
    // Size's Chromium ApplyInitial/ApplyInherit are genuinely empty methods.
}

#[test]
fn zoom_keeps_real_font_dirty_boundary_and_clip_keeps_typed_quad() {
    let _heap = foundation::LayoutHeapScope::new();
    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
    let mut b = ComputedStyleBuilder::from_style(initial);
    for text in [
        "normal",
        "1",
        "100%",
        "0",
        "0%",
        "calc(100%)",
        "calc(1 + 0)",
        "calc(-1)",
    ] {
        Apply(
            P::kZoom,
            &mut b,
            None,
            &parse(P::kZoom, text),
            16.,
            &media(),
        )
        .unwrap();
        assert_eq!(b.Zoom(), 1.);
        assert_eq!(b.EffectiveZoom(), 1.);
    }
    let font = b.GetFontDescription().clone();
    for text in ["2", "50%", "calc(2)", "calc(200%)"] {
        assert_eq!(
            Apply(
                P::kZoom,
                &mut b,
                None,
                &parse(P::kZoom, text),
                16.,
                &media()
            ),
            Err(LonghandApplicationError::Unsupported(P::kZoom))
        );
        assert_eq!(b.Zoom(), 1.);
        assert_eq!(b.EffectiveZoom(), 1.);
        assert!(b.GetFontDescription() == &font);
    }
    let value = parse(P::kClip, "rect(1px auto -3px 4px)");
    let CSSValuePayload::kQuadClass(rect) = value.Payload() else {
        panic!("typed quad required")
    };
    assert_eq!(rect.sides.len(), 4);
    assert_eq!(value.CssText().Utf8(), "rect(1px, auto, -3px, 4px)");
    b.SetEffectiveZoom(2.);
    Apply(P::kClip, &mut b, None, &value, 16., &media()).unwrap();
    assert_eq!(b.Clip().Top(), &Length::Fixed(2));
    assert_eq!(b.Clip().Bottom(), &Length::Fixed(-6));
    assert_eq!(
        ParseProperty(
            P::kObjectViewBox,
            &CSSString::from("inset(1px)"),
            false,
            CSSParserMode::kHTMLStandardMode
        )
        .err()
        .unwrap()
        .kind,
        PropertyParseErrorKind::Unsupported
    );
}

#[test]
fn viewport_grammar_invalid_values_and_quirks_are_precise() {
    for (id, text) in [
        (P::kZoom, "reset"),
        (P::kZoom, "auto"),
        (P::kZoom, "-1"),
        (P::kZoom, "-10%"),
        (P::kZoom, "1px"),
        (P::kClip, "rect(1px 2px 3px)"),
        (P::kClip, "rect(1px,2px 3px,4px)"),
        (P::kClip, "rect(1%,2px,3px,4px)"),
        (P::kClip, "rect(1,2,3,4)"),
        (P::kSize, "-1px"),
        (P::kSize, "100%"),
        (P::kSize, "letter legal"),
        (P::kSize, "auto landscape"),
        (P::kSize, "a4 portrait landscape"),
        (P::kPage, "default"),
        (P::kPage, "ident(Chapter)"),
        (P::kPage, "'Chapter'"),
        (P::kPage, "two names"),
        (P::kWebkitLocale, "en-US"),
        (P::kWebkitLocale, "'en' 'fr'"),
        (P::kObjectViewBox, "polygon(0 0,1px 1px,0 1px)"),
        (P::kObjectViewBox, "circle(2px)"),
    ] {
        assert_eq!(
            ParseProperty(
                id,
                &CSSString::from(text),
                false,
                CSSParserMode::kHTMLStandardMode
            )
            .err()
            .unwrap()
            .kind,
            PropertyParseErrorKind::Invalid,
            "{id:?}: {text}"
        );
    }
    let quirky = ParseProperty(
        P::kClip,
        &CSSString::from("rect(1 2 3 4)"),
        false,
        CSSParserMode::kHTMLQuirksMode,
    )
    .unwrap();
    assert_eq!(
        quirky[0].ValueRef().CssText().Utf8(),
        "rect(1px, 2px, 3px, 4px)"
    );
}
