// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Document-tree owner for production container-query selection and evaluation.
//!
//! Selection follows container_query_evaluator.cc:136-173 and selector feature
//! collection follows container_selector.cc:15-106. Conditions stay owned by
//! the translated `MediaQueryEvaluator::FromContainerValues` path.
#![allow(non_snake_case, dead_code)]

use crate::{
    css_numeric_literal_value::CSSNumericLiteralValue,
    css_value::CSSValuePayload,
    kleene_value::KleeneValue,
    media_queries::{
        conditional_exp_node::{ConditionalExpNode, ConditionalExpNodeVisitor},
        media_query_backend::{MediaQueryFeatureFlags, MediaQueryValueBackend},
        media_query_container::ContainerQueryEvaluationBackend,
        media_query_evaluator::{MediaQueryEvaluationBackend, MediaQueryEvaluator},
        media_query_exp::{
            MediaQueryExp, MediaQueryExpBounds, MediaQueryExpComparison, MediaQueryExpValue,
        },
        production_container_query::{
            ContainerState, ContainerStyleResolver, DocumentMediaValues,
            RegisteredContainerPropertyResolver,
        },
        MediaValuesCachedData,
    },
    production_css_value::{CSSUnparsedDeclarationValue, Value},
    production_rule_context::ProductionContainerQuery,
    resolver::{
        match_result::MatchResult,
        media_query_result::MediaQueryResultFlags,
        production_style_builder::custom_properties::{
            attributes::{AttributeResolutionState, AttributeResolver, PersistentAttributeSource},
            CustomProperties, EnvironmentVariableResolver, SubstitutionContext,
        },
        style_resolver_state::ComputedStyleHandle,
    },
};
use dom::persistent_document::{DOMNamespace, DOMNodeType};
use foundation::{AtomicString, IsHorizontalWritingMode};
use layoutng_style::style::{computed_style::ComputedStyle, computed_style_constants::PseudoId};
use std::{cell::Cell, collections::HashMap, rc::Rc};

const FEATURE_UNKNOWN: u16 = 1 << 1;
const FEATURE_WIDTH: u16 = 1 << 2;
const FEATURE_HEIGHT: u16 = 1 << 3;
const FEATURE_INLINE_SIZE: u16 = 1 << 4;
const FEATURE_BLOCK_SIZE: u16 = 1 << 5;
const FEATURE_STYLE: u16 = 1 << 6;
const FEATURE_STICKY: u16 = 1 << 7;
const FEATURE_SNAP: u16 = 1 << 8;
const FEATURE_SCROLLABLE: u16 = 1 << 9;
const FEATURE_SCROLLED: u16 = 1 << 10;
const FEATURE_ANCHORED: u16 = 1 << 11;

/// Typed ownership translation of Blink's `ContainerSelector`. Keeping the
/// query beside the name preserves feature axes which cannot be reconstructed
/// from a name-only projection.
#[derive(Clone)]
pub(crate) struct ProductionContainerSelector {
    name: AtomicString,
    flags: u16,
}

impl ProductionContainerSelector {
    pub(crate) fn FromQuery(query: &ProductionContainerQuery) -> Self {
        struct FeatureCollector(u16);
        impl ConditionalExpNodeVisitor<Value, CSSUnparsedDeclarationValue> for FeatureCollector {
            fn EvaluateMediaQueryFeatureExpNode(
                &mut self,
                exp: &MediaQueryExp<Value, CSSUnparsedDeclarationValue>,
            ) -> KleeneValue {
                if exp.HasStyleRange() {
                    self.0 |= FEATURE_STYLE;
                    return KleeneValue::kUnknown;
                }
                if !exp.HasMediaFeature() {
                    self.0 |= FEATURE_UNKNOWN;
                    return KleeneValue::kUnknown;
                }
                let name = exp.MediaFeature();
                self.0 |= if name == "stuck" {
                    FEATURE_STICKY
                } else if name == "snapped" {
                    FEATURE_SNAP
                } else if name == "scrollable" {
                    FEATURE_SCROLLABLE
                } else if name == "scrolled" {
                    FEATURE_SCROLLED
                } else if name == "fallback" {
                    FEATURE_ANCHORED
                } else if exp.IsInlineSizeDependent() {
                    FEATURE_INLINE_SIZE
                } else if exp.IsBlockSizeDependent() {
                    FEATURE_BLOCK_SIZE
                } else {
                    let mut flags = 0;
                    if exp.IsWidthDependent() {
                        flags |= FEATURE_WIDTH;
                    }
                    if exp.IsHeightDependent() {
                        flags |= FEATURE_HEIGHT;
                    }
                    flags
                };
                KleeneValue::kUnknown
            }
            fn EvaluateUnknown(&mut self, _: &foundation::String) -> KleeneValue {
                self.0 |= FEATURE_UNKNOWN;
                KleeneValue::kUnknown
            }
            fn EnterFunction(&mut self, name: &AtomicString) {
                if name == "style" {
                    self.0 |= FEATURE_STYLE;
                }
            }
        }

        let mut collector = FeatureCollector(0);
        if let Some(condition) = query.Query() {
            condition.Evaluate(&mut collector);
        }
        Self {
            name: query.SelectorName(),
            flags: collector.0,
        }
    }

    pub(crate) fn SelectsAnyContainer(&self) -> bool {
        self.flags & FEATURE_UNKNOWN == 0
            && (!self.name.IsNull()
                || self.flags
                    & (FEATURE_WIDTH
                        | FEATURE_HEIGHT
                        | FEATURE_INLINE_SIZE
                        | FEATURE_BLOCK_SIZE
                        | FEATURE_STYLE
                        | FEATURE_STICKY
                        | FEATURE_SNAP
                        | FEATURE_SCROLLABLE
                        | FEATURE_SCROLLED
                        | FEATURE_ANCHORED)
                    != 0)
    }
    pub(crate) fn HasUnknownFeature(&self) -> bool {
        self.flags & FEATURE_UNKNOWN != 0
    }
    pub(crate) fn SelectsSizeContainers(&self) -> bool {
        self.flags & (FEATURE_WIDTH | FEATURE_HEIGHT | FEATURE_INLINE_SIZE | FEATURE_BLOCK_SIZE)
            != 0
    }
    fn SelectsStyleOrNameOnlyContainers(&self) -> bool {
        self.flags & FEATURE_STYLE != 0
            || !self.name.IsNull()
                && self.flags
                    & (FEATURE_WIDTH
                        | FEATURE_HEIGHT
                        | FEATURE_INLINE_SIZE
                        | FEATURE_BLOCK_SIZE
                        | FEATURE_STYLE
                        | FEATURE_STICKY
                        | FEATURE_SNAP
                        | FEATURE_SCROLLABLE
                        | FEATURE_SCROLLED
                        | FEATURE_ANCHORED)
                    == 0
    }
    fn SelectsScrollStateContainers(&self) -> bool {
        self.flags & (FEATURE_STICKY | FEATURE_SNAP | FEATURE_SCROLLABLE | FEATURE_SCROLLED) != 0
    }
    fn SelectsAnchoredContainers(&self) -> bool {
        self.flags & FEATURE_ANCHORED != 0
    }
    fn RequiredType(&self, style: &ComputedStyle) -> u32 {
        let horizontal = IsHorizontalWritingMode(style.GetWritingDirection().GetWritingMode());
        let mut required = 0;
        if self.flags & FEATURE_INLINE_SIZE != 0
            || horizontal && self.flags & FEATURE_WIDTH != 0
            || !horizontal && self.flags & FEATURE_HEIGHT != 0
        {
            required |= 1;
        }
        if self.flags & FEATURE_BLOCK_SIZE != 0
            || horizontal && self.flags & FEATURE_HEIGHT != 0
            || !horizontal && self.flags & FEATURE_WIDTH != 0
        {
            required |= 2;
        }
        if self.SelectsScrollStateContainers() {
            required |= 4;
        }
        if self.SelectsAnchoredContainers() {
            required |= 8;
        }
        required
    }
    fn NameMatches(&self, style: &ComputedStyle) -> bool {
        if self.name.IsNull() {
            return true;
        }
        unsafe { style.ContainerName().Get().as_ref() }.is_some_and(|list| {
            list.GetNames().iter().any(|name| {
                unsafe { name.Get().as_ref() }.is_some_and(|name| name.GetName() == &self.name)
            })
        })
    }
    fn Matches(&self, style: &ComputedStyle) -> bool {
        let required = self.RequiredType(style);
        style.StyleType() == PseudoId::kPseudoIdNone
            && style.ContainerType() & required == required
            && self.NameMatches(style)
    }
    pub(crate) fn SetDependencyFlags(&self, result: &mut MatchResult) {
        if self.SelectsSizeContainers() {
            result.SetDependsOnSizeContainerQueries();
        }
        if self.SelectsStyleOrNameOnlyContainers() {
            result.SetDependsOnStyleContainerQueries();
        }
        if self.SelectsScrollStateContainers() {
            result.SetDependsOnScrollStateContainerQueries();
        }
        if self.SelectsAnchoredContainers() {
            result.SetDependsOnAnchoredContainerQueries();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ProductionContainerChange {
    NearestContainer,
    DescendantContainers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProductionContainerEvaluation {
    pub value: KleeneValue,
    pub unit_flags: u32,
}

/// Immutable collaborators needed by Blink's container selection/evaluation.
/// The engine can assemble this from its document-owned snapshots without
/// exporting DOM or evaluator internals across the public StyleEngine boundary.
pub(crate) struct ProductionContainerRuntime<'a> {
    pub document: &'a dom::Document,
    pub media: &'a MediaValuesCachedData,
    pub states: &'a HashMap<usize, ContainerState>,
    pub stored_custom: &'a HashMap<usize, Rc<CustomProperties>>,
    pub current_styles: Option<&'a HashMap<usize, ComputedStyleHandle>>,
    pub current_custom: Option<&'a HashMap<usize, Rc<CustomProperties>>>,
    pub environment: Option<&'a dyn EnvironmentVariableResolver>,
    pub registered: Option<&'a dyn RegisteredContainerPropertyResolver>,
}

impl ProductionContainerRuntime<'_> {
    fn style(&self, node: usize) -> Option<&ComputedStyle> {
        self.current_styles
            .and_then(|styles| styles.get(&node))
            .and_then(|style| unsafe { style.Get().as_ref() })
            .or_else(|| {
                self.document
                    .ResolvedStyleFor(node)
                    .and_then(|style| unsafe { style.native_style.Get().as_ref() })
            })
    }
    fn custom(&self, node: usize) -> Option<&CustomProperties> {
        self.current_custom
            .and_then(|custom| custom.get(&node))
            .or_else(|| self.stored_custom.get(&node))
            .map(Rc::as_ref)
    }
    /// cpp: flat_tree_traversal.cc:131-159. Container selection walks the
    /// rendered flat tree: a distributed light child belongs to its assigned
    /// slot, a shadow-root child belongs to the host, and fallback slot
    /// children disappear while that slot has assigned nodes.
    fn flat_tree_parent_element(&self, node: usize) -> Option<usize> {
        let parent = self.document.Node(node).Parent()?;
        if self.document.Node(parent).Type() == DOMNodeType::kElement {
            if self.document.ShadowRoot(parent).is_some() {
                return self.document.AssignedSlot(node);
            }
            if self.document.IsSlotSupportingAssignment(parent) {
                let has_assigned_nodes =
                    self.document.OwnerShadowHost(parent).is_some_and(|host| {
                        self.document
                            .Node(host)
                            .Children()
                            .iter()
                            .any(|&child| self.document.AssignedSlot(child) == Some(parent))
                    });
                return (!has_assigned_nodes).then_some(parent);
            }
            return Some(parent);
        }
        self.document.ShadowHost(parent)
    }
    /// cpp: style_recalc_context.h / style_engine traversal. Blink carries
    /// this value down the recalc tree so size queries can begin at the first
    /// possible size container instead of the ordinary flat-tree parent.
    pub(crate) fn NearestSizeContainer(&self, subject: usize) -> Option<usize> {
        self.NearestSizeContainerForStyleRequest(subject, PseudoId::kPseudoIdNone)
    }

    /// cpp: style_recalc_context.cc:15-37,73-77. Pseudo style recalc starts
    /// from the originating element itself; ordinary element recalc starts at
    /// its flat-tree parent. The production requests currently exposed here
    /// (::before/::after/::first-letter/::placeholder) are not layout siblings
    /// of their originating element.
    pub(crate) fn NearestSizeContainerForStyleRequest(
        &self,
        subject: usize,
        pseudo: PseudoId,
    ) -> Option<usize> {
        let mut candidate = if pseudo == PseudoId::kPseudoIdNone {
            self.flat_tree_parent_element(subject)
        } else {
            Some(subject)
        };
        while let Some(node) = candidate {
            if self.style(node).is_some_and(|style| {
                style.StyleType() == PseudoId::kPseudoIdNone && style.ContainerType() & 3 != 0
            }) {
                return Some(node);
            }
            candidate = self.flat_tree_parent_element(node);
        }
        None
    }
    pub(crate) fn DetermineStartingElement(
        &self,
        subject: usize,
        pseudo: PseudoId,
        selector: &ProductionContainerSelector,
        nearest_size_container: Option<usize>,
    ) -> Option<usize> {
        if selector.SelectsSizeContainers() {
            // cpp: container_query_evaluator.cc:160-166. A missing recalc
            // context value means there is no possible size container; do not
            // substitute the ordinary parent walk.
            return nearest_size_container;
        }
        if pseudo != PseudoId::kPseudoIdNone {
            Some(subject)
        } else {
            self.flat_tree_parent_element(subject)
        }
    }
    pub(crate) fn FindContainer(
        &self,
        mut starting: Option<usize>,
        selector: &ProductionContainerSelector,
    ) -> Option<usize> {
        while let Some(node) = starting {
            if self.document.Node(node).Type() == DOMNodeType::kElement {
                if self
                    .style(node)
                    .is_some_and(|style| selector.Matches(style))
                {
                    return Some(node);
                }
            }
            starting = self.flat_tree_parent_element(node);
        }
        None
    }
    fn root_style(&self, mut node: usize) -> Option<&ComputedStyle> {
        let mut root = node;
        while let Some(parent) = self.flat_tree_parent_element(node) {
            root = parent;
            node = parent;
        }
        self.style(root)
    }
    pub(crate) fn Evaluate(
        &self,
        container: usize,
        query: &ProductionContainerQuery,
    ) -> Result<ProductionContainerEvaluation, &'static str> {
        let Some(condition) = query.Query() else {
            return Ok(ProductionContainerEvaluation {
                value: KleeneValue::kTrue,
                unit_flags: 0,
            });
        };
        let style = self.style(container).ok_or("container-style-unavailable")?;
        let values = DocumentMediaValues::ForContainer(
            self.document,
            container,
            self.media,
            style,
            self.states.get(&container),
        );
        let root = self
            .root_style(container)
            .ok_or("container-root-style-unavailable")?;
        let values = values.WithRootStyle(root);
        let empty = CustomProperties::default();
        let custom = self.custom(container).unwrap_or(&empty);
        let parent_custom = self
            .flat_tree_parent_element(container)
            .and_then(|parent| self.custom(parent));
        let source = PersistentAttributeSource::new(
            self.document,
            container,
            self.document.Node(container).Namespace() == DOMNamespace::kHTML,
        );
        struct AttributeState(Cell<bool>);
        impl AttributeResolutionState for AttributeState {
            fn SetHasAttrFunction(&self) {
                self.0.set(true)
            }
        }
        let attribute_state = AttributeState(Cell::new(false));
        let attributes = AttributeResolver::new(&source, &attribute_state, false);
        let unsupported = Cell::new(None);
        let resolver = ContainerStyleResolver {
            node: container,
            values: &values,
            custom,
            parent_custom,
            substitution: SubstitutionContext {
                environment: self.environment,
                attributes: Some(&attributes),
            },
            registered: self.registered,
            unsupported: &unsupported,
        };
        let backend = ProductionContainerValueBackend {
            values: &values,
            unsupported: &unsupported,
        };
        let evaluator: MediaQueryEvaluator<
            '_,
            ContainerQueryEvaluationBackend<'_, ProductionContainerValueBackend<'_>, _>,
        > = MediaQueryEvaluator::FromContainerValues(&values, backend, &resolver);
        let mut flags = MediaQueryResultFlags::default();
        let value = evaluator.EvalNode(condition, Some(&mut flags), None);
        if attribute_state.0.get() {
            unsupported.set(Some("container-query-attr-dependency"));
        }
        if let Some(operation) = unsupported.get() {
            return Err(operation);
        }
        Ok(ProductionContainerEvaluation {
            value,
            unit_flags: flags.unit_flags,
        })
    }
}

struct ProductionContainerValueBackend<'a> {
    values: &'a DocumentMediaValues<'a>,
    unsupported: &'a Cell<Option<&'static str>>,
}

impl ProductionContainerValueBackend<'_> {
    fn Numeric(&self, value: &Value) -> Option<CSSNumericLiteralValue> {
        match value.Payload() {
            CSSValuePayload::kNumericLiteralClass(value) => Some(value.clone()),
            CSSValuePayload::kMathFunctionClass(value) => {
                let unit = value.expression.CanonicalUnit()?;
                let number = value
                    .ComputeValue(
                        &mut |number, unit| {
                            Ok(
                                crate::media_queries::media_values::MediaValues::ComputeLength(
                                    self.values,
                                    number,
                                    unit,
                                ),
                            )
                        },
                        None,
                    )
                    .ok()?;
                Some(CSSNumericLiteralValue::Create(number, unit))
            }
            _ => None,
        }
    }
    fn Value(
        &self,
        value: &MediaQueryExpValue<Value>,
    ) -> Option<MediaQueryExpValue<CSSNumericLiteralValue>> {
        Some(match value {
            MediaQueryExpValue::Invalid => MediaQueryExpValue::Invalid,
            MediaQueryExpValue::Id(id) => MediaQueryExpValue::Id(*id),
            MediaQueryExpValue::Value(value) => {
                MediaQueryExpValue::Value(Rc::new(self.Numeric(value)?))
            }
            MediaQueryExpValue::Ratio(pair) => MediaQueryExpValue::Ratio(Rc::new((
                Rc::new(self.Numeric(&pair.0)?),
                Rc::new(self.Numeric(&pair.1)?),
            ))),
        })
    }
    fn Bounds(
        &self,
        bounds: &MediaQueryExpBounds<Value>,
    ) -> Option<MediaQueryExpBounds<CSSNumericLiteralValue>> {
        Some(MediaQueryExpBounds::new(
            &MediaQueryExpComparison::new(&self.Value(&bounds.left.value)?, bounds.left.op),
            &MediaQueryExpComparison::new(&self.Value(&bounds.right.value)?, bounds.right.op),
        ))
    }
}

impl MediaQueryEvaluationBackend for ProductionContainerValueBackend<'_> {
    type Value = Value;
    type UnparsedValue = CSSUnparsedDeclarationValue;
    type ResultFlags = MediaQueryResultFlags;
    type CustomMediaRulesMap = ();
    fn EvalFeature(
        &self,
        feature: &MediaQueryExp<Value, CSSUnparsedDeclarationValue>,
        flags: Option<&mut Self::ResultFlags>,
        _: Option<&Self::CustomMediaRulesMap>,
    ) -> KleeneValue {
        if !feature.HasMediaFeature() {
            self.unsupported
                .set(Some("container-query-non-media-feature"));
            return KleeneValue::kUnknown;
        }
        let Some(bounds) = self.Bounds(feature.Bounds()) else {
            self.unsupported
                .set(Some("container-query-value-conversion"));
            return KleeneValue::kUnknown;
        };
        let numeric = ConditionalExpNode::Feature(MediaQueryExp::CreateWithBounds(
            feature.MediaFeature(),
            &bounds,
        ));
        MediaQueryEvaluator::<MediaQueryValueBackend<'_>>::ForMediaValuesWithFeatures(
            self.values,
            MediaQueryFeatureFlags {
                scrolled_container_queries: true,
                ..Default::default()
            },
        )
        .EvalNode(&numeric, flags, None)
    }
}

pub(crate) fn ApplyContainerUnitDependencies(unit_flags: u32, result: &mut MatchResult) {
    if unit_flags & MediaQueryExpValue::<()>::kDynamicViewport != 0 {
        result.SetDependsOnDynamicViewportUnits();
    }
    if unit_flags
        & (MediaQueryExpValue::<()>::kStaticViewport | MediaQueryExpValue::<()>::kContainer)
        != 0
    {
        result.SetDependsOnStaticViewportUnits();
    }
    if unit_flags & MediaQueryExpValue::<()>::kRootRelative != 0 {
        result.SetDependsOnRootUnitContainerQueries();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parser::{
            container_query_parser::ContainerQueryParser,
            css_parser_context::{SecureContextMode, StrictCSSParserContext},
        },
        production_container_parser::ProductionContainerBackend,
        production_style_sheet::Backend,
    };
    use foundation::{Persistent, String};
    use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};

    fn node_with_id(document: &dom::Document, id: &str) -> usize {
        (0..document.NodeCount())
            .find(|&node| {
                document
                    .Node(node)
                    .FindAttribute("id")
                    .is_some_and(|attribute| attribute.value == id)
            })
            .unwrap()
    }

    fn size_container_style() -> ComputedStyleHandle {
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut builder = ComputedStyleBuilder::from_style(initial);
        builder.SetContainerType(1);
        Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle)
    }

    fn width_query() -> Rc<ProductionContainerQuery> {
        let context = StrictCSSParserContext::<Backend>(SecureContextMode::kInsecureContext);
        let set = ContainerQueryParser::new(ProductionContainerBackend::new(&context))
            .ParseContainerQuerySetString(&String::from("(width > 400px)"))
            .expect("typed container query");
        set.Queries()[0].clone()
    }

    fn evaluate_width(width: Option<f64>) -> KleeneValue {
        let owner =
            html::html_parser::ParseHTML("<div id=container><span id=subject></span></div>");
        let document = owner.GetDocument();
        let container = node_with_id(document, "container");
        let subject = node_with_id(document, "subject");
        let query = width_query();
        let selector = ProductionContainerSelector::FromQuery(&query);
        let mut styles = HashMap::new();
        styles.insert(container, size_container_style());
        let mut root = container;
        while let Some(parent) = document.Node(root).Parent() {
            if document.Node(parent).Type() == DOMNodeType::kElement {
                root = parent;
            } else {
                break;
            }
        }
        styles.entry(root).or_insert_with(|| {
            let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
            let mut builder = ComputedStyleBuilder::from_style(initial);
            Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle)
        });
        let mut state = ContainerState::SizeOnly(
            width.unwrap_or_default(),
            200.0,
            unsafe { &*styles[&container].Get() }.GetWritingDirection(),
        );
        state.width = width;
        let states = HashMap::from([(container, state)]);
        let custom = HashMap::new();
        let media = MediaValuesCachedData {
            media_type: String::from("screen"),
            viewport_width: 800.0,
            viewport_height: 600.0,
            em_size: 16.0,
            ..Default::default()
        };
        let runtime = ProductionContainerRuntime {
            document,
            media: &media,
            states: &states,
            stored_custom: &custom,
            current_styles: Some(&styles),
            current_custom: None,
            environment: None,
            registered: None,
        };
        assert_eq!(
            runtime.DetermineStartingElement(subject, PseudoId::kPseudoIdNone, &selector, None,),
            None,
            "a size query must consume StyleRecalcContext::size_container"
        );
        let nearest = runtime.NearestSizeContainer(subject);
        assert_eq!(nearest, Some(container));
        let starting =
            runtime.DetermineStartingElement(subject, PseudoId::kPseudoIdNone, &selector, nearest);
        assert_eq!(runtime.FindContainer(starting, &selector), Some(container));
        runtime.Evaluate(container, &query).unwrap().value
    }

    #[test]
    fn functional_container_uses_translated_evaluator_for_true_false_and_unknown() {
        let _heap = foundation::LayoutHeapScope::new();
        assert_eq!(evaluate_width(Some(500.0)), KleeneValue::kTrue);
        assert_eq!(evaluate_width(Some(300.0)), KleeneValue::kFalse);
        assert_eq!(evaluate_width(None), KleeneValue::kUnknown);
    }

    #[test]
    fn typed_selector_marks_size_dependency_before_evaluation() {
        let query = width_query();
        let selector = ProductionContainerSelector::FromQuery(&query);
        let mut result = MatchResult::default();
        selector.SetDependencyFlags(&mut result);
        assert!(selector.SelectsAnyContainer());
        assert!(selector.SelectsSizeContainers());
        assert!(result.DependsOnSizeContainerQueries());
        assert!(!result.DependsOnStyleContainerQueries());
    }

    #[test]
    fn nearest_size_container_walks_slots_and_shadow_hosts_in_the_flat_tree() {
        let _heap = foundation::LayoutHeapScope::new();
        let mut owner = html::html_parser::ParseHTML(
            "<!doctype html><x-host id=host><span id=light slot=named></span><span id=unslotted slot=missing></span></x-host>",
        );
        let document = owner.GetDocumentMut();
        let host = node_with_id(document, "host");
        let light = node_with_id(document, "light");
        let unslotted = node_with_id(document, "unslotted");
        let shadow =
            document.AttachShadowRoot(host, dom::persistent_document::DOMShadowRootMode::kOpen);
        let container = document.CreateElementDefault(DOMNamespace::kHTML, "section".into());
        let slot = document.CreateElement(
            DOMNamespace::kHTML,
            "slot".into(),
            vec![dom::persistent_document::DOMAttribute {
                local_name: "name".into(),
                value: "named".into(),
                ..Default::default()
            }],
        );
        let fallback = document.CreateElementDefault(DOMNamespace::kHTML, "i".into());
        let shadow_child = document.CreateElementDefault(DOMNamespace::kHTML, "aside".into());
        document.AppendChild(shadow, container);
        document.AppendChild(container, slot);
        document.AppendChild(slot, fallback);
        document.AppendChild(shadow, shadow_child);

        let mut styles = HashMap::from([
            (container, size_container_style()),
            (host, size_container_style()),
        ]);
        let mut ancestor = host;
        while let Some(parent) = document.Node(ancestor).Parent() {
            ancestor = parent;
            if document.Node(ancestor).Type() == DOMNodeType::kElement {
                styles.entry(ancestor).or_insert_with(|| {
                    let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
                    let mut builder = ComputedStyleBuilder::from_style(initial);
                    Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle)
                });
            }
        }
        let states = HashMap::from([(
            container,
            ContainerState::SizeOnly(
                500.0,
                200.0,
                unsafe { &*styles[&container].Get() }.GetWritingDirection(),
            ),
        )]);
        let custom = HashMap::new();
        let media = MediaValuesCachedData::default();
        let runtime = ProductionContainerRuntime {
            document,
            media: &media,
            states: &states,
            stored_custom: &custom,
            current_styles: Some(&styles),
            current_custom: None,
            environment: None,
            registered: None,
        };

        assert_eq!(document.AssignedSlot(light), Some(slot));
        assert_eq!(runtime.flat_tree_parent_element(light), Some(slot));
        assert_eq!(runtime.flat_tree_parent_element(slot), Some(container));
        assert_eq!(runtime.NearestSizeContainer(light), Some(container));
        let query = width_query();
        let selector = ProductionContainerSelector::FromQuery(&query);
        let starting = runtime.DetermineStartingElement(
            light,
            PseudoId::kPseudoIdNone,
            &selector,
            runtime.NearestSizeContainer(light),
        );
        let selected = runtime.FindContainer(starting, &selector);
        assert_eq!(selected, Some(container));
        assert_eq!(
            runtime.Evaluate(selected.unwrap(), &query).unwrap().value,
            KleeneValue::kTrue
        );
        assert_eq!(runtime.flat_tree_parent_element(shadow_child), Some(host));
        assert_eq!(runtime.NearestSizeContainer(shadow_child), Some(host));

        assert_eq!(document.AssignedSlot(unslotted), None);
        assert_eq!(runtime.flat_tree_parent_element(unslotted), None);
        assert_eq!(runtime.NearestSizeContainer(unslotted), None);
        assert_eq!(runtime.flat_tree_parent_element(fallback), None);
    }
}
