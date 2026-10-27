// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//!
//! Typed request inputs installed on ElementRuleCollector by the production
//! StyleResolver assembly.
#![allow(non_snake_case)]

use crate::{
    container_query::ContainerQuery,
    element_rule_collector::SearchTextRequest,
    production_css_value::{CSSUnparsedDeclarationValue, Value},
};
use foundation::AtomicString;
use layoutng_style::style::computed_style_constants::PseudoId;

pub(crate) type ProductionContainerQuery = ContainerQuery<Value, CSSUnparsedDeclarationValue>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProductionPseudoRequest {
    pub id: PseudoId,
    pub originating_id: PseudoId,
    pub argument: Option<AtomicString>,
    pub ident_list: Vec<AtomicString>,
    pub search_text: SearchTextRequest,
}
impl ProductionPseudoRequest {
    pub fn Element() -> Self {
        Self {
            id: PseudoId::kPseudoIdNone,
            originating_id: PseudoId::kPseudoIdNone,
            argument: None,
            ident_list: Vec::new(),
            search_text: SearchTextRequest::kNone,
        }
    }

    pub fn Pseudo(id: PseudoId, argument: Option<AtomicString>) -> Self {
        Self {
            id,
            originating_id: PseudoId::kPseudoIdNone,
            argument,
            ident_list: Vec::new(),
            search_text: SearchTextRequest::kNone,
        }
    }

    pub fn NestedPseudo(originating_id: PseudoId, id: PseudoId) -> Self {
        Self {
            id,
            originating_id,
            argument: None,
            ident_list: Vec::new(),
            search_text: SearchTextRequest::kNone,
        }
    }

    pub fn SearchText(current: SearchTextRequest) -> Self {
        Self {
            id: PseudoId::kPseudoIdSearchText,
            originating_id: PseudoId::kPseudoIdNone,
            argument: None,
            ident_list: Vec::new(),
            search_text: current,
        }
    }
}

/// Inputs behind ElementRuleCollector's `reject_starting_styles` calculation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ProductionStartingStyleContext {
    pub is_ensuring_style: bool,
    pub has_old_style: bool,
    pub resolving_style: bool,
    pub force_starting_style: bool,
}
impl ProductionStartingStyleContext {
    pub fn InitialResolution() -> Self {
        Self {
            resolving_style: true,
            ..Default::default()
        }
    }
}
