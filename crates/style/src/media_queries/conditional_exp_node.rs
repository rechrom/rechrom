// Copyright 2025 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Source commit: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
// Ledger uses C++ physical / effective / mapped / omitted / pending lines.
// Effective excludes comments, blanks, preprocessor/includes, namespaces and
// lines consisting only of braces/parentheses/semicolons.
// conditional_exp_node.h: 167 / 92 / 69 / 23 / 0.
// conditional_exp_node.cc: 146 / 86 / 79 / 7 / 0.
// Header omitted: 17-23,30,51-54,77,81,85,90,95,97,103,113,123,132,
// 140,150,155,161 (forward/access/default destructor/GC).
// Implementation omitted: 60-63,70-74 (GC Trace only).
// Source ledger:
// - conditional_exp_node.h:29-163: media visitor/compound-node declarations.
// - conditional_exp_node.cc:11-58,65-68,76-144: factories, evaluation, serialization.
// - media_query_exp.cc:820-827: feature leaf dispatch/serialization.
// Omitted: conditional_exp_node.cc:60-63,70-74 and feature Trace (GC only).
// Navigation nodes retain their external type at the visitor boundary.
// Paths are under third_party/blink/renderer/core/css/.
#![allow(non_snake_case)]

use super::media_query_exp::{MediaQueryExp, MediaQueryExpSerialization};
use crate::kleene_value::{KleeneAnd, KleeneNot, KleeneOr, KleeneValue};
use foundation::{AtomicString, String};
use std::rc::Rc;

pub trait ConditionalExpNodeVisitor<V, U = V> {
    // cpp: conditional_exp_node.h:29-38
    fn EvaluateNavigationExpNode<N>(&mut self, _node: &N) -> KleeneValue {
        KleeneValue::kUnknown
    }
    fn EvaluateMediaQuerySet(
        &mut self,
        _set: &super::media_query_set::MediaQuerySet<V, U>,
    ) -> KleeneValue {
        KleeneValue::kUnknown
    }
    // Source's default is unknown, so traversal visits both compound operands.
    fn EvaluateMediaQueryFeatureExpNode(&mut self, _exp: &MediaQueryExp<V, U>) -> KleeneValue {
        KleeneValue::kUnknown
    }
    fn EvaluateUnknown(&mut self, _text: &String) -> KleeneValue {
        KleeneValue::kUnknown
    }
    fn EnterFunction(&mut self, _name: &AtomicString) {}
}

// Rc is the ownership translation of immutable GC Member<const ...> edges.
pub enum ConditionalExpNode<V, U = V> {
    Feature(MediaQueryExp<V, U>),
    Unknown(String),
    NotNode(Rc<Self>),
    NestedNode(Rc<Self>),
    FunctionNode(Rc<Self>, AtomicString),
    AndNode(Rc<Self>, Rc<Self>),
    OrNode(Rc<Self>, Rc<Self>),
}

impl<V, U> ConditionalExpNode<V, U> {
    // cpp: conditional_exp_node.cc:17-58
    pub fn Not(operand: Option<Rc<Self>>) -> Option<Rc<Self>> {
        operand.map(|node| Rc::new(Self::NotNode(node)))
    }
    pub fn Nested(operand: Option<Rc<Self>>) -> Option<Rc<Self>> {
        operand.map(|node| Rc::new(Self::NestedNode(node)))
    }
    pub fn Function(operand: Option<Rc<Self>>, name: &AtomicString) -> Option<Rc<Self>> {
        operand.map(|node| Rc::new(Self::FunctionNode(node, name.clone())))
    }
    pub fn And(left: Option<Rc<Self>>, right: Option<Rc<Self>>) -> Option<Rc<Self>> {
        Some(Rc::new(Self::AndNode(left?, right?)))
    }
    pub fn Or(left: Option<Rc<Self>>, right: Option<Rc<Self>>) -> Option<Rc<Self>> {
        Some(Rc::new(Self::OrNode(left?, right?)))
    }

    // cpp: conditional_exp_node.h:145
    pub fn GetName(&self) -> &AtomicString {
        match self {
            Self::FunctionNode(_, name) => name,
            _ => panic!("not a function node"),
        }
    }

    // cpp: conditional_exp_node.cc:65-68,76-144; media_query_exp.cc:820-824
    pub fn Evaluate(&self, visitor: &mut impl ConditionalExpNodeVisitor<V, U>) -> KleeneValue {
        match self {
            Self::Feature(exp) => visitor.EvaluateMediaQueryFeatureExpNode(exp),
            Self::Unknown(text) => visitor.EvaluateUnknown(text),
            Self::NotNode(operand) => KleeneNot(operand.Evaluate(visitor)),
            Self::NestedNode(operand) => operand.Evaluate(visitor),
            Self::FunctionNode(operand, name) => {
                visitor.EnterFunction(name);
                operand.Evaluate(visitor)
            }
            Self::AndNode(left, right) => {
                let result = left.Evaluate(visitor);
                if result == KleeneValue::kFalse {
                    result
                } else {
                    KleeneAnd(result, right.Evaluate(visitor))
                }
            }
            Self::OrNode(left, right) => {
                let result = left.Evaluate(visitor);
                if result == KleeneValue::kTrue {
                    result
                } else {
                    KleeneOr(result, right.Evaluate(visitor))
                }
            }
        }
    }
}

impl<V: MediaQueryExpSerialization, U: MediaQueryExpSerialization> ConditionalExpNode<V, U> {
    // cpp: conditional_exp_node.cc:11-15,86-90,102-106,113-129,142-144
    pub fn Serialize(&self) -> String {
        let mut result = String::from("");
        self.SerializeTo(&mut result);
        result
    }
    pub fn SerializeTo(&self, output: &mut String) {
        match self {
            Self::Feature(exp) => output.push_string(&exp.Serialize()),
            Self::Unknown(text) => output.push_string(text),
            Self::NotNode(node) => {
                output.push_str("not ");
                node.SerializeTo(output);
            }
            Self::NestedNode(node) => {
                output.push_str("(");
                node.SerializeTo(output);
                output.push_str(")");
            }
            Self::FunctionNode(node, name) => {
                output.push_str(&name.Utf8());
                output.push_str("(");
                node.SerializeTo(output);
                output.push_str(")");
            }
            Self::AndNode(left, right) | Self::OrNode(left, right) => {
                left.SerializeTo(output);
                output.push_str(if matches!(self, Self::AndNode(..)) {
                    " and "
                } else {
                    " or "
                });
                right.SerializeTo(output);
            }
        }
    }
}
