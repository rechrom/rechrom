// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Generated content data; resource loading remains in URLImageResolver.
#![allow(non_snake_case)]
use super::*;
pub(super) fn Apply(
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    images: Option<&dyn URLImageResolver>,
) -> Result {
    let inherit = v.IsInheritedValue();
    let initial = v.IsInitialValue() || v.IsUnsetValue() || inherit && parent.is_none();
    ApplyContent(b, v, inherit, initial, images)?;
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

// cpp: longhands_custom.cc:3158-3294 Content::Apply*.
// The value converter is the custom longhand application. Shared cascade
// substitution already converted attr() into typed strings before this call.
fn ApplyContent(
    b: &mut ComputedStyleBuilder,
    v: &Value,
    inherit: bool,
    initial: bool,
    images: Option<&dyn URLImageResolver>,
) -> Result {
    use layoutng_style::style::computed_style_constants::QuoteType;
    use layoutng_style::style::content_data::{
        AltCounterContentData, AltTextContentData, ContentData, CounterContentData, CounterData,
        ImageContentData, NoneContentData, QuoteContentData, TextContentData,
    };
    fn allocate<T: ContentData + foundation::Traceable + 'static>(
        value: T,
    ) -> *mut dyn ContentData {
        foundation::MakeGarbageCollected(value) as *mut dyn ContentData
    }
    fn counter_data(
        value: &crate::production_css_value::CSSCounterContentValue,
    ) -> std::result::Result<CounterData, LonghandApplicationError> {
        let CSSValuePayload::kCustomIdentClass(identifier) = value.identifier.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(
                CSSPropertyID::kContent,
            ));
        };
        let CSSValuePayload::kCustomIdentClass(style) = value.list_style.Payload() else {
            return Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kContent,
            ));
        };
        let separator = if value.separator.IsNull() {
            foundation::AtomicString::default()
        } else {
            foundation::AtomicString::from_utf16(value.separator.Span16().unwrap_or_default())
        };
        // Native null scope represents the production document-root path.
        // Shadow TreeScope population remains an explicit CSSValue dependency.
        Ok(CounterData::new(
            &identifier.name,
            &style.name,
            &separator,
            std::ptr::null(),
            std::ptr::null(),
        ))
    }
    let id = CSSPropertyID::kContent;
    if initial {
        b.SetContentOwned(None);
        return Ok(());
    }
    if inherit {
        // Chromium Content::ApplyInherit explicitly remains a no-op.
        return Ok(());
    }
    if let CSSValuePayload::kIdentifierClass(value) = v.Payload() {
        match value.0 {
            CSSValueID::kNormal => b.SetContentOwned(None),
            CSSValueID::kNone => b.SetContentOwned(Some(foundation::Member::from_ptr(allocate(
                NoneContentData::new(),
            )))),
            _ => return Err(LonghandApplicationError::InvalidValue(id)),
        }
        return Ok(());
    }
    let CSSValuePayload::kValueListClass(outer) = v.Payload() else {
        return Err(LonghandApplicationError::InvalidValue(id));
    };
    if outer.separator != crate::production_css_value::ListSeparator::Slash
        || !(1..=2).contains(&outer.values.len())
    {
        return Err(LonghandApplicationError::InvalidValue(id));
    }
    let mut first: Option<*mut dyn ContentData> = None;
    let mut previous: Option<*mut dyn ContentData> = None;
    for (group, items) in outer.values.iter().enumerate() {
        let CSSValuePayload::kValueListClass(items) = items.Payload() else {
            return Err(LonghandApplicationError::InvalidValue(id));
        };
        if items.separator != crate::production_css_value::ListSeparator::Space
            || items.values.is_empty()
        {
            return Err(LonghandApplicationError::InvalidValue(id));
        }
        for item in &items.values {
            let next = match item.Payload() {
                CSSValuePayload::kImageClass(_) | CSSValuePayload::kLinearGradientClass(_)
                    if group == 0 =>
                {
                    // GetStyleImage remains a typed resource-owner boundary.
                    allocate(ImageContentData::new(super::ResolveStyleImage(
                        id, item, images,
                    )?))
                }
                CSSValuePayload::kCounterContentClass(value) => {
                    let data = counter_data(value)?;
                    if group == 0 {
                        allocate(CounterContentData::from_data(data))
                    } else {
                        allocate(AltCounterContentData::from_data(data))
                    }
                }
                CSSValuePayload::kIdentifierClass(value) if group == 0 => {
                    let quote = match value.0 {
                        CSSValueID::kOpenQuote => QuoteType::kOpen,
                        CSSValueID::kCloseQuote => QuoteType::kClose,
                        CSSValueID::kNoOpenQuote => QuoteType::kNoOpen,
                        CSSValueID::kNoCloseQuote => QuoteType::kNoClose,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    };
                    allocate(QuoteContentData::new(quote))
                }
                CSSValuePayload::kStringClass(value) => {
                    if group == 1 {
                        allocate(AltTextContentData::new(&value.0))
                    } else {
                        if let Some(previous) = previous {
                            if unsafe { (&*previous).IsText() } {
                                let text = unsafe { &mut *(previous as *mut TextContentData) };
                                let mut joined = text.GetText().clone();
                                joined.push_string(&value.0);
                                text.SetText(&joined);
                                continue;
                            }
                        }
                        allocate(TextContentData::new(&value.0))
                    }
                }
                _ => return Err(LonghandApplicationError::InvalidValue(id)),
            };
            if let Some(previous) = previous {
                unsafe {
                    (&mut *previous).SetNext(Some(next));
                }
            } else {
                first = Some(next);
            }
            previous = Some(next);
        }
    }
    b.SetContentOwned(first.map(foundation::Member::from_ptr));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use layoutng_style::style::content_data::ImageContentData;
    fn value(css: &str) -> std::rc::Rc<Value> {
        crate::parser::production_property_parser::ParseProperty(
            CSSPropertyID::kContent,
            &foundation::String::from(css),
            false,
            crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
        )
        .unwrap()[0]
            .ValueRef()
    }
    #[test]
    fn generated_images_and_resource_errors_preserve_content_ownership() {
        struct MissingResource(std::cell::Cell<usize>);
        impl URLImageResolver for MissingResource {
            fn ResolveImage(
                &self,
                _v: &crate::production_css_value::CSSImageValue,
            ) -> std::result::Result<FetchedImageBinding, LonghandApplicationError> {
                self.0.set(self.0.get() + 1);
                Err(LonghandApplicationError::Unsupported(
                    CSSPropertyID::kBackgroundImage,
                ))
            }
        }
        let heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        let gradient = value("linear-gradient(red, blue) / \"icon\"");
        Apply(&mut b, None, &gradient, None).unwrap();
        let content = b.GetContentData().unwrap();
        assert!(unsafe { (&*content).IsImage() });
        let image = unsafe { &*(content as *const ImageContentData) };
        assert!(unsafe { &*image.GetImage() }.IsGeneratedImage());
        let previous = b.GetContentData();
        let url = value("\"before\" url(icon.svg)");
        assert_eq!(
            Apply(&mut b, None, &url, None),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kContent
            ))
        );
        assert_eq!(b.GetContentData(), previous);
        let resolver = MissingResource(std::cell::Cell::new(0));
        assert_eq!(
            super::super::ApplyWithImageResolver(
                CSSPropertyID::kContent,
                &mut b,
                None,
                &url,
                16.0,
                &MediaValuesCachedData::default(),
                &resolver
            ),
            Err(LonghandApplicationError::Unsupported(
                CSSPropertyID::kContent
            ))
        );
        assert_eq!(resolver.0.get(), 1);
        assert_eq!(b.GetContentData(), previous);
        let retained = foundation::Persistent::from_ptr(b.TakeStyle() as *mut ComputedStyle);
        drop(heap);
        foundation::CollectLayoutHeapForTesting();
        let content = unsafe { &*retained.Get() }.GetContentData().unwrap();
        let image = unsafe { &*(content as *const ImageContentData) };
        assert!(unsafe { &*image.GetImage() }.IsGeneratedImage());
    }
}
