#![allow(non_snake_case, non_camel_case_types)]
use crate::style_resolver::selector::{SourceSpace, Specificity};
use cssom::{CSSDeclaration, CSSStyleSheet};
use std::collections::{BTreeMap, HashMap};

// cpp: style_resolver/style_resolver.cc:6914-6922
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Origin {
    kUserAgent,
    kAuthor,
    kAnimation,
}
#[derive(Clone)]
pub(crate) struct CascadedDeclaration {
    pub declaration: CSSDeclaration,
    pub origin: Origin,
    pub specificity: Specificity,
    pub source_order: usize,
    pub layer_priority: usize,
}
// cpp: style_resolver/style_resolver.cc:6924-6954
// A borrowed call context replaces the source's thread-local scope pointer.
// Each resolve owns its context, so nested resolves cannot change its order.
#[derive(Default)]
pub(crate) struct CascadeLayerOrder {
    user_agent: BTreeMap<String, usize>,
    author: BTreeMap<String, usize>,
}
fn CollectLayerOrder(sheets: &[CSSStyleSheet]) -> BTreeMap<String, usize> {
    let mut order = BTreeMap::new();
    for sheet in sheets {
        for name in sheet
            .layer_order
            .iter()
            .chain(sheet.rules.iter().map(|rule| &rule.layer_name))
        {
            if !name.is_empty() && !order.contains_key(name) {
                order.insert(name.clone(), order.len());
            }
        }
    }
    order
}
impl CascadeLayerOrder {
    pub(crate) fn from_compiled(
        ua: Option<&cssom::compiled_rules::CompiledRuleSet>,
        author: Option<&cssom::compiled_rules::CompiledRuleSet>,
        ua_source: &[CSSStyleSheet],
        author_source: &[CSSStyleSheet],
    ) -> Self {
        Self {
            user_agent: ua.map_or_else(|| CollectLayerOrder(ua_source), |r| r.layers.clone()),
            author: author.map_or_else(|| CollectLayerOrder(author_source), |r| r.layers.clone()),
        }
    }

    pub(crate) fn new(user_agent: &[CSSStyleSheet], author: &[CSSStyleSheet]) -> Self {
        Self {
            user_agent: CollectLayerOrder(user_agent),
            author: CollectLayerOrder(author),
        }
    }
    // cpp: style_resolver/style_resolver.cc:6956-6974
    pub(crate) fn LayerPriority(
        &self,
        origin: Origin,
        name: &str,
        important: bool,
        inline: bool,
    ) -> usize {
        if origin == Origin::kAnimation {
            return 0;
        }
        let layers = if origin == Origin::kUserAgent {
            &self.user_agent
        } else {
            &self.author
        };
        if inline {
            return layers.len() + 1;
        }
        match layers.get(name).filter(|_| !name.is_empty()) {
            None => {
                if important {
                    0
                } else {
                    layers.len() + 1
                }
            }
            Some(index) => {
                if important {
                    layers.len() - index
                } else {
                    index + 1
                }
            }
        }
    }
}
// cpp: style_resolver/style_resolver.cc:6976-6981
pub(crate) fn CascadeLevel(important: bool, origin: Origin) -> u8 {
    if important {
        return if origin == Origin::kUserAgent { 4 } else { 3 };
    }
    if origin == Origin::kAnimation {
        return 2;
    }
    if origin == Origin::kAuthor {
        1
    } else {
        0
    }
}
// cpp: style_resolver/style_resolver.cc:6983-6994
pub(crate) fn CascadeLess(
    left: &CascadedDeclaration,
    right: &CascadedDeclaration,
) -> std::cmp::Ordering {
    let key = |item: &CascadedDeclaration| {
        (
            CascadeLevel(item.declaration.important, item.origin),
            item.layer_priority,
            item.specificity,
            item.source_order,
        )
    };
    key(left).cmp(&key(right))
}
// cpp: style_resolver/style_resolver.cc:6996-7022
pub(crate) fn ResolveCustomProperties(
    cascade: &[CascadedDeclaration],
    inherited: Option<&crate::style_resolver::CustomProperties>,
) -> crate::style_resolver::CustomProperties {
    let mut result = inherited.cloned().unwrap_or_default();
    for item in cascade {
        let name = &item.declaration.property;
        if !name.starts_with("--") {
            continue;
        }
        let keyword = item
            .declaration
            .value
            .trim_matches(|c: char| c.source_space())
            .to_ascii_lowercase();
        let value = match keyword.as_str() {
            "initial" => None,
            "inherit" | "unset" | "revert" | "revert-layer" => {
                inherited.and_then(|p| p.get(name)).cloned().flatten()
            }
            _ => Some(item.declaration.value.clone()),
        };
        // The inherited value is immutable. Most nodes and pseudo-elements
        // declare no custom properties, so retain their parent's map and only
        // copy it when this cascade actually changes a binding.
        if result.get(name) != Some(&value) {
            std::sync::Arc::make_mut(&mut result).insert(name.clone(), value);
        }
    }
    result
}
// cpp: style_resolver/style_resolver.cc:7129-7138
pub(crate) fn ResolveDeclarationVariables(
    declaration: &CSSDeclaration,
    properties: &HashMap<String, Option<String>>,
) -> Option<CSSDeclaration> {
    if declaration.property.starts_with("--") {
        return None;
    }
    let value = crate::style_resolver::variables::ResolveVariables(
        &declaration.value,
        properties,
        &mut Vec::new(),
        0,
    )?;
    Some(CSSDeclaration {
        property: declaration.property.clone(),
        value,
        important: declaration.important,
    })
}
