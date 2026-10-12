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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSOMAdoptionError {
    StyleSheetHasNoIdentity,
    StyleSheetHasOwnerNode,
    UnknownConstructedStyleSheet,
    InvalidTreeScope,
}

/// The document-owned, authoritative stylesheet collection. The owner key is
/// supplied by DOM without making CSSOM depend on DOM or the layout assembly.
/// This ownership follows Document::style_engine_ and
/// StyleEngine::document_style_sheet_collection_ in Blink.
pub struct CSSOM<Owner: Eq + Hash = ()> {
    style_sheets: Vec<CSSStyleSheet>,
    detached_style_sheets: HashMap<Owner, Vec<CSSStyleSheet>>,
    // cpp: tree_scope.h:182-190; css_style_sheet.h:200-203,321.
    // Constructed sheets belong to their constructor Document (this CSSOM
    // instance) and may be referenced, in order and with multiplicity, by any
    // Document/ShadowRoot TreeScope owned by that Document.
    constructed_style_sheets: HashMap<u64, CSSStyleSheet>,
    adopted_style_sheet_ids: HashMap<Owner, Vec<u64>>,
    generation: u64,
}

impl<Owner: Eq + Hash> Default for CSSOM<Owner> {
    fn default() -> Self {
        Self {
            style_sheets: Vec::new(),
            detached_style_sheets: HashMap::new(),
            constructed_style_sheets: HashMap::new(),
            adopted_style_sheet_ids: HashMap::new(),
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

    /// Registers the backing created by `new CSSStyleSheet()` in this
    /// Document. Registration does not activate the sheet; TreeScope adoption
    /// is a separate ordered relation below.
    pub fn RegisterConstructedStyleSheet(
        &mut self,
        sheet: CSSStyleSheet,
    ) -> Result<bool, CSSOMAdoptionError> {
        if sheet.owner_node_id != 0 {
            return Err(CSSOMAdoptionError::StyleSheetHasOwnerNode);
        }
        let identity = sheet.Identity();
        if identity == 0 {
            return Err(CSSOMAdoptionError::StyleSheetHasNoIdentity);
        }
        let changed = self
            .constructed_style_sheets
            .get(&identity)
            .is_none_or(|existing| existing != &sheet);
        if changed {
            self.constructed_style_sheets.insert(identity, sheet);
        }
        Ok(changed)
    }

    pub fn GetConstructedStyleSheet(&self, identity: u64) -> Option<&CSSStyleSheet> {
        self.constructed_style_sheets.get(&identity)
    }

    pub fn SetAdoptedStyleSheets(
        &mut self,
        scope: Owner,
        identities: Vec<u64>,
        receive: impl FnOnce(CSSOMEvent),
    ) -> Result<bool, CSSOMAdoptionError> {
        if identities
            .iter()
            .any(|identity| !self.constructed_style_sheets.contains_key(identity))
        {
            return Err(CSSOMAdoptionError::UnknownConstructedStyleSheet);
        }
        if self.adopted_style_sheet_ids.get(&scope) == Some(&identities) {
            return Ok(false);
        }
        if identities.is_empty() {
            self.adopted_style_sheet_ids.remove(&scope);
        } else {
            self.adopted_style_sheet_ids.insert(scope, identities);
        }
        self.generation = self
            .generation
            .checked_add(1)
            .expect("CSSOM generation exhausted");
        receive(CSSOMEvent::StyleSheetChanged);
        Ok(true)
    }

    pub fn GetAdoptedStyleSheets(&self, scope: &Owner) -> Vec<&CSSStyleSheet> {
        self.adopted_style_sheet_ids
            .get(scope)
            .into_iter()
            .flatten()
            .filter_map(|identity| self.constructed_style_sheets.get(identity))
            .collect()
    }

    pub fn IsStyleSheetAdoptedBy(&self, identity: u64, scope: &Owner) -> bool {
        self.adopted_style_sheet_ids
            .get(scope)
            .is_some_and(|identities| identities.contains(&identity))
    }

    pub fn IsConstructedStyleSheetAdopted(&self, identity: u64) -> bool {
        self.adopted_style_sheet_ids
            .values()
            .any(|identities| identities.contains(&identity))
    }

    /// Preserve AppendStyleSheet's owner-node replacement and same-value no-op.
    /// Ownerless sheets retain source order and are always appended.
    pub fn ApplyMutation(
        &mut self,
        mutation: CSSOMMutation<Owner>,
        receive: impl FnOnce(CSSOMEvent),
    ) -> bool {
        let CSSOMMutation::AddStyleSheet {
            mut sheet,
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
                sheet.PrepareOwnerReplacement(existing);
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
    use std::rc::Rc;

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

    #[test]
    fn owner_replacement_preserves_identity_and_advances_revision() {
        let mut catalog = CSSOM::<u64>::default();
        let mut first = crate::ParseCSS("div { color: red }");
        first.owner_node_id = 7;
        first.AttachParsedBacking("div { color: red }", Rc::new(11_u32));
        let identity = first.Identity();
        assert!(catalog.ApplyMutation(
            CSSOMMutation::AddStyleSheet {
                sheet: first.clone(),
                detached_owner: None,
            },
            |_| {},
        ));
        assert!(!catalog.ApplyMutation(
            CSSOMMutation::AddStyleSheet {
                sheet: first,
                detached_owner: None,
            },
            |_| {},
        ));

        let mut replacement = crate::ParseCSS("div { color: blue }");
        replacement.owner_node_id = 7;
        replacement.AttachParsedBacking("div { color: blue }", Rc::new(22_u32));
        assert!(catalog.ApplyMutation(
            CSSOMMutation::AddStyleSheet {
                sheet: replacement,
                detached_owner: None,
            },
            |_| {},
        ));
        let retained = &catalog.GetStyleSheets(None)[0];
        assert_eq!((retained.Identity(), retained.Revision()), (identity, 2));
        assert_eq!(retained.SourceText(), "div { color: blue }");
        assert!(retained.HasCurrentSource());
        assert_eq!(*retained.ParsedBacking::<u32>().unwrap(), 22);
        assert!(!format!("{retained:?}").contains("parsed_backing"));

        let mut projected_edit = retained.clone();
        projected_edit.rules[0].declaration_text = " color: green ".into();
        assert!(catalog.ApplyMutation(
            CSSOMMutation::AddStyleSheet {
                sheet: projected_edit,
                detached_owner: None,
            },
            |_| {},
        ));
        let retained = &catalog.GetStyleSheets(None)[0];
        assert_eq!((retained.Identity(), retained.Revision()), (identity, 3));
        assert!(!retained.HasCurrentSource());
        assert!(retained.ParsedBacking::<u32>().is_none());
    }

    #[test]
    fn constructed_sheet_adoption_retains_order_multiplicity_and_scope_identity() {
        let mut catalog = CSSOM::<u64>::default();
        let mut first = crate::ParseCSS("@scope { .first { color: red } }");
        first.AttachParsedBacking("@scope { .first { color: red } }", Rc::new(()));
        let mut second = crate::ParseCSS(".second { color: blue }");
        second.AttachParsedBacking(".second { color: blue }", Rc::new(()));
        let first_id = first.Identity();
        let second_id = second.Identity();
        assert_ne!(first_id, 0);
        assert_ne!(second_id, 0);
        assert!(catalog.RegisterConstructedStyleSheet(first).unwrap());
        assert!(catalog.RegisterConstructedStyleSheet(second).unwrap());
        assert!(catalog.GetStyleSheets(None).is_empty());

        let mut notifications = 0;
        assert!(catalog
            .SetAdoptedStyleSheets(10, vec![first_id, second_id, first_id], |_| {
                notifications += 1
            })
            .unwrap());
        assert!(catalog
            .SetAdoptedStyleSheets(20, vec![first_id], |_| notifications += 1)
            .unwrap());
        assert!(!catalog
            .SetAdoptedStyleSheets(20, vec![first_id], |_| notifications += 1)
            .unwrap());
        assert_eq!(notifications, 2);
        assert_eq!(
            catalog
                .GetAdoptedStyleSheets(&10)
                .iter()
                .map(|sheet| sheet.Identity())
                .collect::<Vec<_>>(),
            [first_id, second_id, first_id]
        );
        assert!(catalog.IsStyleSheetAdoptedBy(first_id, &10));
        assert!(catalog.IsStyleSheetAdoptedBy(first_id, &20));
        assert!(!catalog.IsStyleSheetAdoptedBy(second_id, &20));
        assert_eq!(catalog.Generation(), 2);

        assert_eq!(
            catalog.SetAdoptedStyleSheets(30, vec![u64::MAX], |_| {}),
            Err(CSSOMAdoptionError::UnknownConstructedStyleSheet)
        );
        let mut owned = crate::ParseCSS("div{}");
        owned.owner_node_id = 7;
        assert_eq!(
            catalog.RegisterConstructedStyleSheet(owned),
            Err(CSSOMAdoptionError::StyleSheetHasOwnerNode)
        );
    }
}
