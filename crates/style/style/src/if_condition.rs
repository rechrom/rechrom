// Copyright 2025 The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typed nodes used by CSS `if()` conditions.
//!
//! This is the Rust ownership translation of `if_condition.{h,cc}`.  Media
//! tests retain a real `MediaQuerySet`, style tests retain the production
//! container-query expression, and supports/else tests carry their source
//! Boolean result.  Compound nodes preserve Kleene short-circuit evaluation.
#![allow(non_snake_case)]

use crate::{
    kleene_value::{KleeneAnd, KleeneNot, KleeneOr, KleeneValue},
    media_queries::media_query_set::MediaQuerySet,
    production_container_parser::ContainerCondition,
    production_style_sheet::QuerySet,
};
use foundation::String;
use std::rc::Rc;

pub enum IfCondition {
    Supports(bool),
    Media(Rc<QuerySet>),
    Style(Rc<ContainerCondition>),
    Else,
    Unknown(String),
    Not(Rc<Self>),
    Nested(Rc<Self>),
    And(Rc<Self>, Rc<Self>),
    Or(Rc<Self>, Rc<Self>),
}

pub trait IfConditionEvaluator {
    fn EvaluateMedia(&mut self, query: &QuerySet) -> KleeneValue;
    fn EvaluateStyle(&mut self, query: &ContainerCondition) -> KleeneValue;
}

impl IfCondition {
    pub fn MediaFromCondition(
        condition: Rc<
            crate::media_queries::conditional_exp_node::ConditionalExpNode<
                crate::css_numeric_literal_value::CSSNumericLiteralValue,
            >,
        >,
    ) -> Self {
        use crate::media_queries::media_query::{MediaQuery, RestrictorType};
        Self::Media(Rc::new(MediaQuerySet::FromQueries(vec![Rc::new(
            MediaQuery::new(RestrictorType::kNone, String::from("all"), Some(condition)),
        )])))
    }

    pub fn Evaluate(&self, evaluator: &mut impl IfConditionEvaluator) -> KleeneValue {
        match self {
            Self::Supports(value) => {
                if *value {
                    KleeneValue::kTrue
                } else {
                    KleeneValue::kFalse
                }
            }
            Self::Media(query) => evaluator.EvaluateMedia(query),
            Self::Style(query) => evaluator.EvaluateStyle(query),
            Self::Else => KleeneValue::kTrue,
            Self::Unknown(_) => KleeneValue::kUnknown,
            Self::Not(operand) => KleeneNot(operand.Evaluate(evaluator)),
            Self::Nested(operand) => operand.Evaluate(evaluator),
            Self::And(left, right) => {
                let result = left.Evaluate(evaluator);
                if result == KleeneValue::kFalse {
                    result
                } else {
                    KleeneAnd(result, right.Evaluate(evaluator))
                }
            }
            Self::Or(left, right) => {
                let result = left.Evaluate(evaluator);
                if result == KleeneValue::kTrue {
                    result
                } else {
                    KleeneOr(result, right.Evaluate(evaluator))
                }
            }
        }
    }
}
