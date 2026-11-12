//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: html_image_fallback_helper.cc:214-283; image_input_type.cc:247-281.
use super::*;
use foundation::{AtomicString, Persistent};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder},
    html_image_fallback_helper::{self as helper, ImageFallbackHostFacts},
};
impl PersistentDocument {
    pub fn InputImageClickLocationState(&self, element: usize) -> Rc<std::cell::Cell<(i32, i32)>> {
        self.input_image_click_locations
            .borrow_mut()
            .entry(self.arena[element].id)
            .or_insert_with(|| Rc::new(std::cell::Cell::new((0, 0))))
            .clone()
    }
    pub fn InputImageClickLocation(&self, element: usize) -> (i32, i32) {
        self.InputImageClickLocationState(element).get()
    }

    pub fn InputImageFallbackState(&self, element: usize) -> Rc<std::cell::Cell<bool>> {
        self.input_image_fallback_states
            .borrow_mut()
            .entry(self.arena[element].id)
            .or_insert_with(|| Rc::new(std::cell::Cell::new(false)))
            .clone()
    }
    pub fn ImageInputHasFallbackContent(&self, element: usize) -> bool {
        self.input_image_fallback_states
            .borrow()
            .get(&self.arena[element].id)
            .is_some_and(|state| state.get())
    }
    pub fn BindInputImageLocale(&self, element: usize, locale: Rc<dyn InputAltTextLocale>) {
        self.input_image_locales
            .borrow_mut()
            .insert(self.arena[element].id, locale);
    }
    pub fn CreateImageInputAltTextShadowTree(&mut self, element: usize) {
        let locale = self
            .input_image_locales
            .borrow()
            .get(&self.arena[element].id)
            .cloned()
            .expect("image input retains its source locale");
        let alt = self.TextIteratorAltText(element, locale.as_ref());
        self.CreateHTMLImageAltTextShadowTree(element, &alt);
    }
    pub fn CreateHTMLImageAltTextShadowTree(&mut self, element: usize, alt: &foundation::String) {
        let container = self.CreateElementDefault(DOMNamespace::kHTML, "span".into());
        self.arena[container].class = DOMNodeClass::HTMLAltTextContainerElement;
        self.SetHasCustomStyleCallbacks(container);
        self.SetAttribute(
            container,
            DOMAttribute {
                local_name: "id".into(),
                value: "alttext-container".into(),
                ..Default::default()
            },
        );
        let image = self.CreateElement(
            DOMNamespace::kHTML,
            "img".into(),
            vec![
                DOMAttribute {
                    local_name: "id".into(),
                    value: "alttext-image".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "width".into(),
                    value: "16".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "height".into(),
                    value: "16".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "align".into(),
                    value: "left".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "style".into(),
                    value: "margin:0px".into(),
                    ..Default::default()
                },
            ],
        );
        self.arena[image].class = DOMNodeClass::HTMLAltTextImageElement;
        self.SetHasCustomStyleCallbacks(image);
        self.AppendChild(container, image);
        let text = self.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "id".into(),
                value: "alttext".into(),
                ..Default::default()
            }],
        );
        // Source: Text -> CharacterData, character_data.h:70-87 explicitly
        // normalizes a null WTF String to g_empty_string.
        let units = match alt.Span16() {
            Some(units) => units,
            None => &[],
        };
        let node = self.CreateTextUTF16(units);
        self.AppendChild(text, node);
        self.AppendChild(container, text);
        let root = self
            .ShadowRoot(element)
            .unwrap_or_else(|| self.AttachShadowRoot(element, DOMShadowRootMode::kUserAgent));
        self.AppendChild(root, container);
    }
    pub fn AdjustInputImageStyle(
        &self,
        element: usize,
        style: Persistent<ComputedStyle>,
    ) -> Persistent<ComputedStyle> {
        let image_input = self.InputType(element) == Some(DOMInputType::kImage);
        let html_image = self.arena[element].IsHTMLElement("img")
            && !matches!(
                self.arena[element].class,
                DOMNodeClass::HTMLAltTextImageElement
            );
        let image = image_input || html_image;
        let container = matches!(
            self.arena[element].class,
            DOMNodeClass::HTMLAltTextContainerElement
        );
        let icon = matches!(
            self.arena[element].class,
            DOMNodeClass::HTMLAltTextImageElement
        );
        if !image && !container && !icon {
            return style;
        }
        let source = unsafe { &*style.Get() };
        if image
            && !(if image_input {
                self.ImageInputHasFallbackContent(element)
            } else {
                self.HTMLImageUsesFallbackContent(element)
            })
            && source.UAShadowHostData().is_null()
        {
            return style;
        }
        let quirks = self.GetCompatibilityMode() == CompatibilityMode::kQuirks;
        let mut builder = ComputedStyleBuilder::from_style(source);
        if image {
            let alt = if image_input {
                let locale = self
                    .input_image_locales
                    .borrow()
                    .get(&self.arena[element].id)
                    .cloned()
                    .expect("image input retains its source locale");
                self.TextIteratorAltText(element, locale.as_ref())
            } else {
                self.HTMLImageAltText(element)
            };
            let attribute = |name| {
                self.arena[element]
                    .FindAttribute(name)
                    .map(|a| AtomicString::from_str(a.value.as_str()))
                    .unwrap_or_default()
            };
            let alt_attribute = attribute("alt");
            let src_attribute = attribute("src");
            let root = self.ShadowRoot(element);
            let facts = ImageFallbackHostFacts {
                quirks,
                has_author_shadow: root.is_some_and(|root| {
                    self.ShadowRootMode(root) != Some(DOMShadowRootMode::kUserAgent)
                }),
                has_ua_shadow: root.is_some_and(|root| {
                    self.ShadowRootMode(root) == Some(DOMShadowRootMode::kUserAgent)
                }),
                alt_text: &alt,
                alt_attribute: &alt_attribute,
                src_attribute: &src_attribute,
            };
            if image_input {
                helper::AdjustImageInputStyle(
                    self.ImageInputHasFallbackContent(element),
                    &facts,
                    &mut builder,
                );
            } else {
                helper::AdjustHostStyle(&facts, &mut builder);
            }
        } else if container {
            helper::AdjustAltTextContainerStyle(quirks, &mut builder);
        } else {
            helper::AdjustAltTextImageStyle(quirks, &mut builder);
        }
        Persistent::from_ptr(builder.TakeStyle().cast_mut())
    }
}
