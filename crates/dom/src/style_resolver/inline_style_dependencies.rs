//! A conservative proof for the inline-declaration mutation path. Selector
//! matching can read the style attribute through attribute selectors, including
//! selectors nested in functional pseudo-classes. This scans syntax, not live
//! matches; failed proof retains the original sibling/relational fallback.
#![allow(non_snake_case)]

pub(super) fn MayReadStyleAttribute(text: &str) -> bool {
    MayReadAttribute(text, "style")
}

pub(super) fn MayReadAttribute(text: &str, attribute: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut parentheses = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            // Escaped names and namespaces require a richer grammar before a
            // negative dependency result is safe.
            b'\\' | b'|' => return true,
            b'[' => {
                i += 1;
                while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
                    i += 1;
                }
                let start = i;
                while bytes
                    .get(i)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
                {
                    i += 1;
                }
                if i == start {
                    return true;
                }
                if text[start..i].eq_ignore_ascii_case(attribute) {
                    return true;
                }
                while bytes.get(i).is_some_and(u8::is_ascii_whitespace) {
                    i += 1;
                }
                // An attribute namespace is distinct from the |= operator.
                if bytes.get(i) == Some(&b'|') && bytes.get(i + 1) != Some(&b'=') {
                    return true;
                }
                if !bytes
                    .get(i)
                    .is_some_and(|b| matches!(b, b']' | b'=' | b'~' | b'|' | b'^' | b'$' | b'*'))
                {
                    return true;
                }
                let mut quote = 0;
                let mut closed = false;
                while i < bytes.len() {
                    let b = bytes[i];
                    if b == b'\\' {
                        return true;
                    }
                    if quote != 0 {
                        if b == quote {
                            quote = 0;
                        }
                    } else if matches!(b, b'\'' | b'"') {
                        quote = b;
                    } else if b == b']' {
                        i += 1;
                        closed = true;
                        break;
                    } else if b == b'[' || b == b'(' || b == b')' {
                        return true;
                    }
                    i += 1;
                }
                if !closed {
                    return true;
                }
                continue;
            }
            b':' => {
                i += 1;
                if bytes.get(i) == Some(&b':') {
                    i += 1;
                }
                let start = i;
                while bytes
                    .get(i)
                    .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'-')
                {
                    i += 1;
                }
                if start == i {
                    return true;
                }
                if bytes.get(i) == Some(&b'(') {
                    let name = &text[start..i];
                    if ![
                        "has",
                        "is",
                        "not",
                        "where",
                        "nth-child",
                        "nth-last-child",
                        "nth-of-type",
                        "nth-last-of-type",
                    ]
                    .iter()
                    .any(|known| name.eq_ignore_ascii_case(known))
                    {
                        return true;
                    }
                }
                continue;
            }
            b'(' => parentheses += 1,
            b')' => {
                if parentheses == 0 {
                    return true;
                }
                parentheses -= 1;
            }
            b']' | b'\'' | b'"' | b'{' | b'}' | b'/' => return true,
            b if !b.is_ascii() => return true,
            _ => {}
        }
        i += 1;
    }
    parentheses != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_style_attribute_is_a_dependency() {
        for selector in [
            "[style]",
            "[STYLE*=height]",
            ".x:has(:is(.other,[style])) + .y",
            ".x:not([style=''])",
            ":nth-child(2n of [style])",
        ] {
            assert!(MayReadStyleAttribute(selector), "{selector}");
        }
    }
    #[test]
    fn unrelated_relational_sibling_and_quoted_text_do_not_read_style() {
        for selector in [
            ".doljm:has(.Egmwab :first-child)",
            ".vH6rvf:has(.jOCUFf) .rNHry[aria-checked=true]",
            ".box + .box",
            ".style #style",
            "[data-x='[style]']:not(.hidden)",
            "[lang|=en]",
        ] {
            assert!(!MayReadStyleAttribute(selector), "{selector}");
        }
    }
    #[test]
    fn escaped_namespace_unrecognized_function_and_malformed_selectors_keep_fallback() {
        for selector in [
            r"[st\79le]",
            "[ns|style]",
            "svg|rect",
            ".x:future(.y)",
            ":is([class=x]",
            "[class='unterminated]",
            ":is(.x))",
        ] {
            assert!(MayReadStyleAttribute(selector), "{selector}");
        }
    }
    #[test]
    fn compiled_inline_style_dependencies_ignore_only_unreachable_targets() {
        use super::super::BuildCompiledRuleSet;
        for selector in [
            "::view-transition-old(sb)",
            "html:active-view-transition-type(aimc)::view-transition-new(sb):only-child",
            "[style]::view-transition-old(sb)",
        ] {
            let sheet = cssom::ParseCSS(&format!("{selector}{{color:red}}"));
            let rules = BuildCompiledRuleSet([&sheet]);
            assert_eq!(rules.rules.len(), 1, "{selector}");
            assert!(rules.rules[0]
                .selectors
                .iter()
                .all(|s| s.target.index().is_none()));
            assert!(!rules.inline_style_may_affect_selectors, "{selector}");
        }
        for selector in [
            "::view-transition-old(sb), .box[style]",
            ".box[style]::before, ::view-transition-new(sb)",
            "::view-transition-new(sb), .x:future(.y)",
            r"::view-transition-new(sb), [st\79le]",
        ] {
            let sheet = cssom::ParseCSS(&format!("{selector}{{color:red}}"));
            assert!(
                BuildCompiledRuleSet([&sheet]).inline_style_may_affect_selectors,
                "{selector}"
            );
        }
    }
    #[test]
    fn captured_google_typed_selectors_prove_no_inline_style_dependency() {
        use super::super::BuildCompiledRuleSet;
        let sheet = cssom::ParseCSS(include_str!("tests/google_typed_selector_dependencies.css"));
        assert_eq!(
            sheet.rules.len(),
            1285,
            "every captured selector group remains present"
        );
        let rules = BuildCompiledRuleSet([&sheet]);
        assert!(rules.depends_on_siblings && rules.depends_on_descendants);
        assert!(rules
            .rules
            .iter()
            .flat_map(|r| r.selectors.iter())
            .any(|s| s.target.index().is_none()));
        assert!(!rules.inline_style_may_affect_selectors);
    }
}
