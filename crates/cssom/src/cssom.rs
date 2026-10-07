#![allow(non_snake_case)]

use crate::CSSStyleSheet;
use std::{collections::HashMap, hash::Hash};

pub enum CSSOMMutation<Owner = ()> {
    AddStyleSheet {
        sheet: CSSStyleSheet,
        detached_owner: Option<Owner>,
    },
}

pub enum CSSOMEvent {
    StyleSheetChanged,
}

/// The document-owned, authoritative stylesheet collection. The owner key is
/// supplied by DOM without making CSSOM depend on DOM or the layout assembly.
/// This ownership follows Document::style_engine_ and
/// StyleEngine::document_style_sheet_collection_ in Blink.
pub struct CSSOM<Owner: Eq + Hash = ()> {
    style_sheets: Vec<CSSStyleSheet>,
    detached_style_sheets: HashMap<Owner, Vec<CSSStyleSheet>>,
    generation: u64,
}

impl<Owner: Eq + Hash> Default for CSSOM<Owner> {
    fn default() -> Self {
        Self {
            style_sheets: Vec::new(),
            detached_style_sheets: HashMap::new(),
            generation: 0,
        }
    }
}

impl<Owner: Eq + Hash> CSSOM<Owner> {
    pub fn Generation(&self) -> u64 {
        self.generation
    }

    pub fn GetStyleSheets(&self, owner: Option<&Owner>) -> &[CSSStyleSheet] {
        match owner {
            None => &self.style_sheets,
            Some(owner) => self
                .detached_style_sheets
                .get(owner)
                .map_or(&[], Vec::as_slice),
        }
    }

    /// Preserve AppendStyleSheet's owner-node replacement and same-value no-op.
    /// Ownerless sheets retain source order and are always appended.
    pub fn ApplyMutation(
        &mut self,
        mutation: CSSOMMutation<Owner>,
        receive: impl FnOnce(CSSOMEvent),
    ) -> bool {
        let CSSOMMutation::AddStyleSheet {
            sheet,
            detached_owner,
        } = mutation;
        let sheets = if let Some(owner) = detached_owner {
            self.detached_style_sheets.entry(owner).or_default()
        } else {
            &mut self.style_sheets
        };
        if sheet.owner_node_id != 0 {
            if let Some(existing) = sheets
                .iter_mut()
                .find(|existing| existing.owner_node_id == sheet.owner_node_id)
            {
                if *existing == sheet {
                    return false;
                }
                *existing = sheet;
            } else {
                sheets.push(sheet);
            }
        } else {
            sheets.push(sheet);
        }
        self.generation = self
            .generation
            .checked_add(1)
            .expect("CSSOM generation exhausted");
        receive(CSSOMEvent::StyleSheetChanged);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_noop_and_owner_scopes_preserve_source_order() {
        let mut catalog = CSSOM::<u64>::default();
        let mut sheet = crate::ParseCSS("div { color: red }");
        sheet.owner_node_id = 7;
        let mut notifications = 0;
        let mut apply = |catalog: &mut CSSOM<u64>, sheet, detached_owner| {
            catalog.ApplyMutation(
                CSSOMMutation::AddStyleSheet {
                    sheet,
                    detached_owner,
                },
                |_| notifications += 1,
            )
        };
        assert!(apply(&mut catalog, sheet.clone(), None));
        assert!(!apply(&mut catalog, sheet.clone(), None));
        let mut replacement = crate::ParseCSS("div { color: blue }");
        replacement.owner_node_id = 7;
        assert!(apply(&mut catalog, replacement.clone(), None));
        assert!(apply(&mut catalog, sheet.clone(), Some(42)));
        let ownerless = crate::ParseCSS("span { color: green }");
        assert!(apply(&mut catalog, ownerless.clone(), None));
        assert!(apply(&mut catalog, ownerless.clone(), None));
        assert_eq!(
            catalog.GetStyleSheets(None),
            &[replacement, ownerless.clone(), ownerless]
        );
        assert_eq!(catalog.GetStyleSheets(Some(&42)), &[sheet]);
        assert!(catalog.GetStyleSheets(Some(&43)).is_empty());
        assert_eq!(catalog.Generation(), 5);
        assert_eq!(notifications, 5);
    }
}
