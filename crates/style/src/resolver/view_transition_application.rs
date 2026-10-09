// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! style_builder_converter.cc:2809-2865 and generated Apply* branches.
//! Native fields store genuine document-root names/groups. Document/shadow
//! TreeScope/HasTreeScopedReference remain external, never fake snapshot owners.
#![allow(non_snake_case)]
use super::*;
use foundation::{HeapVector, MakeGarbageCollected, Member, ScopedCSSNameList};
use layoutng_style::style::{
    style_view_transition_group::StyleViewTransitionGroup,
    style_view_transition_name::StyleViewTransitionName,
};
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kViewTransitionName
            | CSSPropertyID::kViewTransitionClass
            | CSSPropertyID::kViewTransitionGroup
    )
}
fn Name(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<Member<StyleViewTransitionName>, LonghandApplicationError> {
    if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
        return match k.0 {
            CSSValueID::kNone => Ok(Member::default()),
            // Both source variants require state.GetDocument() as TreeScope.
            // That native DOM owner is unbound; never substitute null scope.
            CSSValueID::kAuto | CSSValueID::kMatchElement => {
                Err(LonghandApplicationError::Unsupported(id))
            }
            _ => Err(LonghandApplicationError::InvalidValue(id)),
        };
    }
    let name = super::anchor_application::Name(id, v)?;
    let name = unsafe { &*name.Get() };
    if name.GetName() == "none" || name.GetName() == "auto" {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    Ok(Member::from_ptr(StyleViewTransitionName::Create(
        name.GetName(),
        name.GetTreeScope(),
    )))
}
fn Class(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<Member<ScopedCSSNameList>, LonghandApplicationError> {
    if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==CSSValueID::kNone) {
        return Ok(Member::default());
    }
    let CSSValuePayload::kValueListClass(list) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if list.separator != crate::production_css_value::ListSeparator::Space || list.values.is_empty()
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let names = list
        .values
        .iter()
        .map(|v| {
            let name = super::anchor_application::Name(id, v)?;
            if unsafe { &*name.Get() }.GetName() == "none" {
                return Err(LonghandApplicationError::InvalidValue(id));
            }
            Ok(name)
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(Member::from_ptr(MakeGarbageCollected(
        ScopedCSSNameList::new(HeapVector::from(names)),
    )))
}
fn Group(
    id: CSSPropertyID,
    v: &Value,
) -> std::result::Result<StyleViewTransitionGroup, LonghandApplicationError> {
    if let CSSValuePayload::kIdentifierClass(k) = v.Payload() {
        return match k.0 {
            CSSValueID::kNormal => Ok(StyleViewTransitionGroup::Normal()),
            CSSValueID::kNearest => Ok(StyleViewTransitionGroup::Nearest()),
            CSSValueID::kContain => Ok(StyleViewTransitionGroup::Contain()),
            _ => Err(LonghandApplicationError::InvalidValue(id)),
        };
    }
    let name = super::anchor_application::Name(id, v)?;
    let name = unsafe { &*name.Get() }.GetName();
    if name == "normal" || name == "nearest" || name == "contain" {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    // The source group owns an AtomicString, not ScopedCSSName. Preserve
    // ConvertCustomIdent's native identity before extracting exactly that field.
    Ok(StyleViewTransitionGroup::Create(name))
}
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
) -> Result {
    let inherit = v.IsInheritedValue();
    let initial = v.IsInitialValue() || v.IsUnsetValue() || inherit && parent.is_none();
    match id {
        CSSPropertyID::kViewTransitionName => {
            let value = if initial {
                Member::default()
            } else if inherit {
                parent.unwrap().ViewTransitionName().clone()
            } else {
                Name(id, v)?
            };
            b.SetViewTransitionNameOwned(value);
        }
        CSSPropertyID::kViewTransitionClass => {
            let value = if initial {
                Member::default()
            } else if inherit {
                parent.unwrap().ViewTransitionClass().clone()
            } else {
                Class(id, v)?
            };
            b.SetViewTransitionClassOwned(value);
        }
        CSSPropertyID::kViewTransitionGroup => {
            let value = if initial {
                ComputedStyleInitialValues::InitialViewTransitionGroup()
            } else if inherit {
                parent.unwrap().ViewTransitionGroup().clone()
            } else {
                Group(id, v)?
            };
            b.SetViewTransitionGroupOwned(value);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{
        css_parser_mode::CSSParserMode,
        production_property_parser::{ParseDeclarationList, ParseProperty, PropertyParseErrorKind},
    };
    fn initial() -> &'static ComputedStyle {
        unsafe { &*ComputedStyle::GetInitialStyleSingleton() }
    }
    fn snapshot(b: &ComputedStyleBuilder) -> &ComputedStyle {
        unsafe { &*b.CloneStyle() }
    }
    fn apply(b: &mut ComputedStyleBuilder, parent: Option<&ComputedStyle>, css: &str) {
        let parsed = ParseDeclarationList(
            &foundation::String::from(css),
            CSSParserMode::kHTMLStandardMode,
        );
        assert!(parsed.errors.is_empty(), "{:?}", parsed.errors);
        assert!(!parsed.properties.is_empty());
        for p in parsed.properties {
            super::super::Apply(
                p.PropertyID(),
                b,
                parent,
                p.Value(),
                16.0,
                &MediaValuesCachedData::default(),
            )
            .unwrap();
        }
    }
    fn name(b: &ComputedStyleBuilder) -> &StyleViewTransitionName {
        unsafe { b.ViewTransitionName().GetNonNull().unwrap().as_ref() }
    }
    fn text(name: &foundation::AtomicString) -> std::string::String {
        foundation::String::from_utf16(name.utf16_units().unwrap_or_default()).Utf8()
    }
    fn class_names(b: &ComputedStyleBuilder) -> Vec<std::string::String> {
        let list = unsafe { b.ViewTransitionClass().GetNonNull().unwrap().as_ref() };
        list.GetNames()
            .iter()
            .map(|n| {
                let n = unsafe { &*n.Get() };
                assert!(n.GetTreeScope().is_null());
                text(n.GetName())
            })
            .collect()
    }
    #[test]
    fn production_view_transition_document_root_custom_name_and_class_native_identity() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b,None,"view-transition-name:Hero\\2d Main;view-transition-class:Card card Card auto match-element");
        assert!(name(&b).IsCustom());
        assert_eq!(text(&name(&b).CustomName()), "Hero-Main");
        assert!(name(&b).GetTreeScope().is_null());
        assert_eq!(
            class_names(&b),
            ["Card", "card", "Card", "auto", "match-element"]
        );
        let p = ParseProperty(
            CSSPropertyID::kViewTransitionName,
            &foundation::String::from("Hero"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert!(p[0].Value().State().NeedsTreeScopePopulation());
        let p = ParseProperty(
            CSSPropertyID::kViewTransitionClass,
            &foundation::String::from("Card"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        assert!(p[0].Value().State().NeedsTreeScopePopulation());
        apply(
            &mut b,
            None,
            "view-transition-name:none;view-transition-class:none",
        );
        assert!(
            b.ViewTransitionName().GetNonNull().is_none()
                && b.ViewTransitionClass().GetNonNull().is_none()
        );
    }
    #[test]
    fn production_view_transition_group_keywords_and_custom_native_value() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        for (value, index) in [("normal", 0), ("nearest", 1), ("contain", 2)] {
            apply(&mut b, None, &format!("view-transition-group:{value}"));
            assert_eq!(
                [
                    b.ViewTransitionGroup().IsNormal(),
                    b.ViewTransitionGroup().IsNearest(),
                    b.ViewTransitionGroup().IsContain()
                ][index],
                true
            );
        }
        // Group permits none/auto as custom identifiers, exactly as source.
        for value in ["HeroGroup", "none", "auto", "match-element"] {
            apply(&mut b, None, &format!("view-transition-group:{value}"));
            assert!(b.ViewTransitionGroup().IsCustom());
            assert_eq!(text(&b.ViewTransitionGroup().CustomName()), value);
        }
    }
    #[test]
    fn production_view_transition_css_wide_inheritance_resets_and_copy_on_write() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b,None,"view-transition-name:Hero;view-transition-class:First Second;view-transition-group:Group");
        let parent = snapshot(&b);
        let mut child = ComputedStyleBuilder::from_style(initial());
        apply(&mut child,Some(parent),"view-transition-name:inherit;view-transition-class:inherit;view-transition-group:inherit");
        assert!(child.ViewTransitionName() == parent.ViewTransitionName());
        assert!(child.ViewTransitionClass() == parent.ViewTransitionClass());
        assert!(child.ViewTransitionGroup() == parent.ViewTransitionGroup());
        apply(
            &mut child,
            Some(parent),
            "view-transition-name:none;view-transition-class:Child;view-transition-group:nearest",
        );
        assert_eq!(class_names(&child), ["Child"]);
        assert!(child.ViewTransitionName().GetNonNull().is_none());
        assert_eq!(
            text(&unsafe { &*parent.ViewTransitionName().Get() }.CustomName()),
            "Hero"
        );
        apply(
            &mut child,
            Some(parent),
            "view-transition-name:unset;view-transition-class:initial;view-transition-group:unset",
        );
        assert!(
            child.ViewTransitionName().GetNonNull().is_none()
                && child.ViewTransitionClass().GetNonNull().is_none()
                && child.ViewTransitionGroup().IsNormal()
        );
        apply(&mut child,None,"view-transition-name:inherit;view-transition-class:inherit;view-transition-group:inherit");
        assert!(
            child.ViewTransitionName().GetNonNull().is_none()
                && child.ViewTransitionClass().GetNonNull().is_none()
                && child.ViewTransitionGroup().IsNormal()
        );
    }
    #[test]
    fn production_view_transition_stable_auto_invalid_and_document_owner_boundary() {
        let _heap = foundation::LayoutHeapScope::new();
        for (id, value) in [
            (CSSPropertyID::kViewTransitionName, "auto"),
            (CSSPropertyID::kViewTransitionName, "default"),
            (CSSPropertyID::kViewTransitionName, "one two"),
            (CSSPropertyID::kViewTransitionName, "'name'"),
            (CSSPropertyID::kViewTransitionClass, "none other"),
            (CSSPropertyID::kViewTransitionClass, "first none"),
            (CSSPropertyID::kViewTransitionClass, "default"),
            (CSSPropertyID::kViewTransitionClass, "first,second"),
            (CSSPropertyID::kViewTransitionGroup, "normal nearest"),
            (CSSPropertyID::kViewTransitionGroup, "default"),
            (CSSPropertyID::kViewTransitionGroup, "20px"),
        ] {
            assert_eq!(
                ParseProperty(
                    id,
                    &foundation::String::from(value),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .err()
                .unwrap()
                .kind,
                PropertyParseErrorKind::Invalid,
                "{id:?}:{value}"
            );
        }
        for id in [
            CSSPropertyID::kViewTransitionName,
            CSSPropertyID::kViewTransitionClass,
            CSSPropertyID::kViewTransitionGroup,
        ] {
            assert_eq!(
                ParseProperty(
                    id,
                    &foundation::String::from("ident(card)"),
                    false,
                    CSSParserMode::kHTMLStandardMode
                )
                .err()
                .unwrap()
                .kind,
                PropertyParseErrorKind::Unsupported
            );
        }
        let p = ParseProperty(
            CSSPropertyID::kViewTransitionName,
            &foundation::String::from("match-element"),
            false,
            CSSParserMode::kHTMLStandardMode,
        )
        .unwrap();
        let mut b = ComputedStyleBuilder::from_style(initial());
        apply(&mut b, None, "view-transition-name:Hero");
        let previous = b.ViewTransitionName().clone();
        assert!(matches!(
            Apply(
                CSSPropertyID::kViewTransitionName,
                &mut b,
                None,
                p[0].Value()
            ),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kViewTransitionName
            ))
        ));
        assert!(b.ViewTransitionName() == &previous && name(&b).IsCustom());
    }
}
