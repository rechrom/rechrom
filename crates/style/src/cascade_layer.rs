#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]
// Copyright 2021 The Chromium Authors. BSD-style license; see Chromium LICENSE.
use foundation::{g_empty_atom, AtomicString, String};
use std::cell::{Ref, RefCell};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

// cpp: third_party/blink/renderer/core/css/cascade_layer.h:18-44
// Shared handles preserve Member<CascadeLayer> node identity. Names use the
// existing AtomicString boundary so null atoms and lone surrogates survive.
#[derive(Clone, Debug)]
pub struct CascadeLayer(Rc<CascadeLayerData>);

impl PartialEq for CascadeLayer {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for CascadeLayer {}
impl Hash for CascadeLayer {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state);
    }
}
#[derive(Debug)]
struct CascadeLayerData {
    name: AtomicString,
    direct_sub_layers: RefCell<Vec<CascadeLayer>>,
}
impl Default for CascadeLayer {
    fn default() -> Self {
        Self::new(&g_empty_atom)
    }
}
impl CascadeLayer {
    // cpp: third_party/blink/renderer/core/css/cascade_layer.h:20-22
    pub fn new(name: &AtomicString) -> Self {
        Self(Rc::new(CascadeLayerData {
            name: name.clone(),
            direct_sub_layers: RefCell::new(Vec::new()),
        }))
    }
    // cpp: third_party/blink/renderer/core/css/cascade_layer.h:24
    pub fn GetName(&self) -> &AtomicString {
        &self.0.name
    }
    // cpp: third_party/blink/renderer/core/css/cascade_layer.h:25-27
    pub fn GetDirectSubLayers(&self) -> Ref<'_, [CascadeLayer]> {
        Ref::map(self.0.direct_sub_layers.borrow(), |layers| {
            layers.as_slice()
        })
    }
    // cpp: third_party/blink/renderer/core/css/cascade_layer.cc:11-22
    fn FindDirectSubLayer(&self, name: &AtomicString) -> Option<Self> {
        if name == &*g_empty_atom {
            return None;
        }
        for sub_layer in &*self.0.direct_sub_layers.borrow() {
            if sub_layer.GetName() == name {
                return Some(sub_layer.clone());
            }
        }
        None
    }
    // cpp: third_party/blink/renderer/core/css/cascade_layer.cc:24-36
    pub fn GetOrAddSubLayer(&self, name: &[AtomicString]) -> Self {
        let mut layer = self.clone();
        for name_part in name {
            let direct_sub_layer = match layer.FindDirectSubLayer(name_part) {
                Some(existing) => existing,
                None => {
                    let new_layer = Self::new(name_part);
                    layer
                        .0
                        .direct_sub_layers
                        .borrow_mut()
                        .push(new_layer.clone());
                    new_layer
                }
            };
            layer = direct_sub_layer;
        }
        layer
    }
    // cpp: third_party/blink/renderer/core/css/cascade_layer.cc:38-42
    pub fn ToStringForTesting(&self) -> String {
        let mut result = Vec::new();
        self.ToStringInternal(&mut result, &[]);
        String::from_utf16(&result)
    }
    // cpp: third_party/blink/renderer/core/css/cascade_layer.cc:44-56
    fn ToStringInternal(&self, result: &mut Vec<u16>, prefix: &[u16]) {
        for sub_layer in &*self.0.direct_sub_layers.borrow() {
            let name = if sub_layer.0.name.length() != 0 {
                sub_layer.0.name.utf16_units().unwrap().to_vec()
            } else {
                "(anonymous)".encode_utf16().collect()
            };
            if !result.is_empty() {
                result.push(b',' as u16);
            }
            result.extend_from_slice(prefix);
            result.extend_from_slice(&name);
            let mut next_prefix = prefix.to_vec();
            next_prefix.extend_from_slice(&name);
            next_prefix.push(b'.' as u16);
            sub_layer.ToStringInternal(result, &next_prefix);
        }
    }
    // cpp: cascade_layer.cc:58-60 is Oilpan tracing only; Rc owns child handles.
}

#[cfg(test)]
mod tests {
    use crate::cascade_layer;
    use foundation::{AtomicString, String};
    fn names(parts: &[&str]) -> Vec<AtomicString> {
        parts.iter().map(|s| AtomicString::from_str(s)).collect()
    }
    #[test]
    fn anonymous_layers_are_distinct_and_ordered() {
        let empty = cascade_layer::CascadeLayer::default();
        let surrogate = AtomicString::from_utf16(&[0xd800]);
        empty.GetOrAddSubLayer(&[surrogate]);
        assert_eq!(
            empty.ToStringForTesting().Span16(),
            String::from_utf16(&[0xd800]).Span16()
        );
        let null_named = cascade_layer::CascadeLayer::default();
        null_named.GetOrAddSubLayer(&[AtomicString::default()]);
        null_named.GetOrAddSubLayer(&[AtomicString::default()]);
        assert_eq!(null_named.GetDirectSubLayers().len(), 1);
        let c = cascade_layer::CascadeLayer::default();
        c.GetOrAddSubLayer(&names(&["foo", "bar"]));
        c.GetOrAddSubLayer(&names(&["foo", "baz"]));
        c.GetOrAddSubLayer(&names(&["foo", "bar"]));
        c.GetOrAddSubLayer(&names(&[""]));
        c.GetOrAddSubLayer(&names(&[""]));
        assert_eq!(
            c.ToStringForTesting().as_str(),
            "foo,foo.bar,foo.baz,(anonymous),(anonymous)"
        );
        assert_eq!(c.GetDirectSubLayers().len(), 3);
        let f = c.GetOrAddSubLayer(&names(&["foo"]));
        f.GetOrAddSubLayer(&names(&["quux"]));
        assert_eq!(
            c.ToStringForTesting().as_str(),
            "foo,foo.bar,foo.baz,foo.quux,(anonymous),(anonymous)"
        );
    }
}
