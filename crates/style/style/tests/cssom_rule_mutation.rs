// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
use style::{
    CSSStyleSheetRuleMutationError, DeleteCSSStyleSheetRule, InsertCSSStyleSheetRule, ParseCSS,
    ParseCSSWithBase,
};

#[test]
fn mutation_rebuilds_current_typed_owner_from_utf16_source() {
    let css = "/* 😀 */ .first{color:red}.last{background:url(icon.svg)}";
    let mut sheet = ParseCSSWithBase(css, "https://example.test/css/site.css");
    sheet.SetLinkMetadata(73, false, false, "theme".to_owned(), "screen".to_owned());

    let inserted = InsertCSSStyleSheetRule(&sheet, 1, ".middle{width:7px}")
        .expect("insertRule keeps an authoritative typed owner");
    assert!(inserted.HasCurrentSource());
    assert_eq!(inserted.owner_node_id, 73);
    assert_eq!(inserted.title, "theme");
    assert_eq!(inserted.owner_media_conditions, ["screen"]);
    assert_eq!(inserted.rules.len(), 3);
    assert_eq!(inserted.rules[1].selector_text, ".middle");
    assert_eq!(inserted.rules[1].media_conditions, ["screen"]);
    assert_eq!(
        inserted.SourceText(),
        "/* 😀 */ .first{color:red}.middle{width:7px}\n.last{background:url(icon.svg)}"
    );
    assert!(inserted.rules[2]
        .declarations
        .iter()
        .any(|declaration| declaration
            .value
            .contains("https://example.test/css/icon.svg")));

    let deleted = DeleteCSSStyleSheetRule(&inserted, 1)
        .expect("deleteRule keeps an authoritative typed owner");
    assert!(deleted.HasCurrentSource());
    assert_eq!(deleted.rules.len(), 2);
    assert_eq!(deleted.rules[0].selector_text, ".first");
    assert_eq!(deleted.rules[1].selector_text, ".last");
}

#[test]
fn mutation_rejects_non_rule_and_range_errors() {
    let sheet = ParseCSS(".first{color:red}");
    assert_eq!(
        InsertCSSStyleSheetRule(&sheet, 2, ".last{color:blue}").unwrap_err(),
        CSSStyleSheetRuleMutationError::IndexSize
    );
    assert_eq!(
        InsertCSSStyleSheetRule(&sheet, 1, "not a rule").unwrap_err(),
        CSSStyleSheetRuleMutationError::Syntax
    );
    assert_eq!(
        DeleteCSSStyleSheetRule(&sheet, 1).unwrap_err(),
        CSSStyleSheetRuleMutationError::IndexSize
    );
}
