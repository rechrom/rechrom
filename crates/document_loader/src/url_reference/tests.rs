use super::ResolveUrl;

#[test]
fn resolves_web_resource_references() {
    let base = "https://example.test/docs/page.html?view=1#section";
    assert_eq!(
        ResolveUrl(base, "../assets/site.css").unwrap(),
        "https://example.test/assets/site.css"
    );
    assert_eq!(
        ResolveUrl(base, "//cdn.example.test/css/../site.css").unwrap(),
        "https://cdn.example.test/site.css"
    );
    assert_eq!(
        ResolveUrl(base, "/styles/site.css?v=2").unwrap(),
        "https://example.test/styles/site.css?v=2"
    );
    assert_eq!(
        ResolveUrl(base, "?view=2").unwrap(),
        "https://example.test/docs/page.html?view=2"
    );
    assert_eq!(
        ResolveUrl(base, "#next").unwrap(),
        "https://example.test/docs/page.html?view=1#next"
    );
}

#[test]
fn css_url_rewrites_preserve_non_url_text_quotes_and_unicode() {
    use super::ResolveCSSURLs;
    let base = "https://example.test/docs/page.html";
    for text in [
        "color: red",
        "rgba(1,2,3,0.5)",
        "myurl(a.png)",
        "url('')",
        "url('unfinished",
    ] {
        let mut value = text.to_owned();
        assert!(ResolveCSSURLs(&mut value, base).unwrap().is_empty());
        assert_eq!(value, text);
    }
    let mut value = "é前缀 URL('../a.png') rgb(1,2,3) url( b.png ) 尾部".to_owned();
    assert_eq!(
        ResolveCSSURLs(&mut value, base).unwrap(),
        [
            "https://example.test/a.png",
            "https://example.test/docs/b.png"
        ]
    );
    assert_eq!(value, "é前缀 url(\"https://example.test/a.png\") rgb(1,2,3) url(\"https://example.test/docs/b.png\") 尾部");
}
