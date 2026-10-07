#![allow(non_snake_case)]
use crate::persistent_document::{DOMNamespace, DOMNode};
use crate::style_resolver::cascade::{CascadeLayerOrder, CascadeLess, CascadedDeclaration, Origin};
use crate::style_resolver::initial_style::InitialStyle;
use crate::style_resolver::media::MediaConditionMatches;
use crate::style_resolver::number::Number;
use crate::style_resolver::persistent_selector::MatchCompiledSelectorTarget;
use crate::style_resolver::selector::{PseudoTarget, SourceSpace, Specificity};
use crate::style_resolver::style_rule_index::{BuildCompiledRuleSet, ForEachCompiledCandidateSlot};
use crate::style_resolver::StyleEnvironment;
use crate::Document;
use cssom::compiled_rules::{CompiledRuleSet, MediaRuleCache};
use cssom::{CSSDeclaration, CSSStyleSheet};
use std::sync::Arc;

struct CandidateMarks {
    generation: u32,
    seen: Vec<u32>,
}
impl CandidateMarks {
    fn new(count: usize) -> Self {
        Self {
            generation: 0,
            seen: vec![0; count],
        }
    }
    fn begin(&mut self, count: usize) -> u32 {
        if self.seen.len() != count {
            self.seen.resize(count, 0);
        }
        self.generation = self.generation.wrapping_add(1);
        if self.generation == 0 {
            self.seen.fill(0);
            self.generation = 1;
        }
        self.generation
    }
}

fn Trim(input: &str) -> &str {
    input.trim_matches(|c: char| c.source_space())
}
// cpp: style_resolver/style_resolver.cc:891-903
pub(crate) fn MathPresentationLength(text: &str, allow_negative: bool) -> Option<String> {
    let value = Trim(text);
    let lower = value.to_ascii_lowercase();
    if ["calc(", "min(", "max(", "clamp("]
        .iter()
        .any(|prefix| lower.starts_with(prefix))
    {
        return None;
    }
    crate::style_resolver::border_radius::Length(value, 16.0)
        .filter(|&n| allow_negative || n >= 0.0)
        .map(|_| value.into())
}

// cpp: style_resolver/style_resolver.cc:8101-8106
// Immutable borrows preserve the source's index/layer scope without an active
// thread-local pointer. A later style pass commits results to the same arena.
pub(crate) struct StyleCascadeContext<'a> {
    document: &'a Document,
    environment: &'a StyleEnvironment,
    user_agent: Arc<CompiledRuleSet>,
    user_agent_source: &'a [CSSStyleSheet],
    author_source: Vec<&'a CSSStyleSheet>,
    author: Arc<CompiledRuleSet>,
    layers: CascadeLayerOrder,
    user_agent_media: Arc<Vec<Vec<bool>>>,
    author_media: Arc<Vec<Vec<bool>>>,
    // RuleSet candidate buckets are persistent and source-ordered. Chromium
    // walks them directly; one generation array suppresses duplicates without
    // allocating and sorting a Vec for every element and pseudo target.
    candidate_marks: [std::cell::RefCell<CandidateMarks>; 2],
}
impl<'a> StyleCascadeContext<'a> {
    pub(crate) fn new(
        document: &'a Document,
        environment: &'a StyleEnvironment,
        user_agent: &'a [CSSStyleSheet],
    ) -> Self {
        let author = document.StyleSheets(None);
        let author_source: Vec<_> = document.ActiveStyleSheets().collect();
        let user_agent_rules = document
            .StyleState()
            .rules
            .user_agent
            .clone()
            .unwrap_or_else(|| Arc::new(BuildCompiledRuleSet(user_agent)));
        let author_rules = document
            .StyleState()
            .rules
            .author
            .clone()
            .unwrap_or_else(|| Arc::new(BuildCompiledRuleSet(document.ActiveStyleSheets())));
        // Exhaustive destructuring keeps the key in step with StyleEnvironment.
        // No DOM/selector state enters media-query evaluation.
        let StyleEnvironment {
            media_type,
            viewport_width,
            viewport_height,
            resolution_dppx,
            preferred_color_scheme,
        } = *environment;
        let environment_key = (
            media_type as u8,
            viewport_width,
            viewport_height,
            resolution_dppx,
            preferred_color_scheme as u8,
        );
        let sheet_revision = document.StyleState().sheet_revision;
        let cached = document
            .StyleState()
            .rules
            .media
            .borrow()
            .as_ref()
            .filter(|cache| {
                cache.environment == environment_key
                    && cache.sheet_revision == sheet_revision
                    && Arc::ptr_eq(&cache.user_agent_rules, &user_agent_rules)
                    && Arc::ptr_eq(&cache.author_rules, &author_rules)
            })
            .map(|cache| {
                (
                    cache.user_agent_matches.clone(),
                    cache.author_matches.clone(),
                )
            });
        let (user_agent_media, author_media) = cached.unwrap_or_else(|| {
            // Recompute only when the immutable rules or complete environment
            // changed, rather than walking every stylesheet for each DOM edit.
            let mut media = std::collections::HashMap::new();
            let mut match_sheets = |sheets: &[&CSSStyleSheet]| -> Vec<Vec<bool>> {
                sheets
                    .iter()
                    .map(|sheet| {
                        sheet
                            .rules
                            .iter()
                            .map(|rule| {
                                rule.media_conditions.iter().all(|condition| {
                                    if let Some(&matched) = media.get(condition.as_str()) {
                                        matched
                                    } else {
                                        let matched = MediaConditionMatches(condition, environment);
                                        media.insert(condition.clone(), matched);
                                        matched
                                    }
                                })
                            })
                            .collect()
                    })
                    .collect()
            };
            let ua = Arc::new(match_sheets(&user_agent.iter().collect::<Vec<_>>()));
            let author = Arc::new(match_sheets(&author_source));
            *document.StyleState().rules.media.borrow_mut() = Some(MediaRuleCache {
                environment: environment_key,
                sheet_revision,
                user_agent_rules: user_agent_rules.clone(),
                author_rules: author_rules.clone(),
                user_agent_matches: ua.clone(),
                author_matches: author.clone(),
            });
            (ua, author)
        });
        let candidate_marks = [
            std::cell::RefCell::new(CandidateMarks::new(user_agent_rules.rules.len())),
            std::cell::RefCell::new(CandidateMarks::new(author_rules.rules.len())),
        ];
        Self {
            document,
            environment,
            user_agent_source: user_agent,
            author_source,
            user_agent_media,
            author_media,
            user_agent: user_agent_rules,
            author: author_rules,
            layers: CascadeLayerOrder::from_compiled(
                document.StyleState().rules.user_agent.as_deref(),
                document.StyleState().rules.author.as_deref(),
                user_agent,
                author,
            ),
            candidate_marks,
        }
    }
    fn GatherRules(
        &self,
        node: usize,
        target: PseudoTarget,
        cascade: &mut Vec<CascadedDeclaration>,
        order: &mut usize,
    ) {
        for (origin_index, (origin, index)) in [
            (Origin::kUserAgent, &self.user_agent),
            (Origin::kAuthor, &self.author),
        ]
        .into_iter()
        .enumerate()
        {
            let mut marks = self.candidate_marks[origin_index].borrow_mut();
            let generation = marks.begin(index.rules.len());
            ForEachCompiledCandidateSlot(index, self.document.Node(node), target, |slot| {
                if marks.seen[slot] == generation {
                    return;
                }
                marks.seen[slot] = generation;
                let indexed = &index.rules[slot];
                let rule = if origin == Origin::kUserAgent {
                    &self.user_agent_source[indexed.sheet_index].rules[indexed.rule_index]
                } else {
                    &self.author_source[indexed.sheet_index].rules[indexed.rule_index]
                };
                let active = if origin == Origin::kUserAgent {
                    self.user_agent_media[indexed.sheet_index][indexed.rule_index]
                } else {
                    self.author_media[indexed.sheet_index][indexed.rule_index]
                };
                if !active {
                    return;
                }
                let Some(specificity) =
                    MatchCompiledSelectorTarget(self.document, node, &indexed.selectors, target)
                else {
                    return;
                };
                for (declaration_index, declaration) in rule.declarations.iter().enumerate() {
                    cascade.push(CascadedDeclaration {
                        declaration: declaration.clone(),
                        origin,
                        specificity,
                        source_order: *order + indexed.source_order + declaration_index,
                        layer_priority: self.layers.LayerPriority(
                            origin,
                            &rule.layer_name,
                            declaration.important,
                            false,
                        ),
                    });
                }
            });
            *order += index.declaration_count;
        }
    }
    // cpp: style_resolver/style_resolver.cc:7378-7421
    pub(crate) fn GatherPseudoCascade(
        &self,
        node: usize,
        target: PseudoTarget,
    ) -> Vec<CascadedDeclaration> {
        let mut cascade = Vec::new();
        self.GatherRules(node, target, &mut cascade, &mut 0);
        cascade.sort_by(CascadeLess);
        cascade
    }
    // cpp: style_resolver/style_resolver.cc:7577-7804
    pub(crate) fn GatherElementCascade(&self, index: usize) -> Vec<CascadedDeclaration> {
        let node = self.document.Node(index);
        let mut cascade = Vec::new();
        for (order, declaration) in PresentationHints(node).into_iter().enumerate() {
            cascade.push(CascadedDeclaration {
                declaration,
                origin: Origin::kAuthor,
                specificity: Specificity::default(),
                source_order: order,
                layer_priority: self.layers.LayerPriority(Origin::kAuthor, "", false, false),
            });
        }
        let mut order = cascade.len();
        self.GatherRules(index, PseudoTarget::Element, &mut cascade, &mut order);
        if let Some(inline) = node.FindAttribute("style") {
            let declarations = {
                let mut cache = self.document.StyleState().rules.inline.borrow_mut();
                let entry = cache.entry(node.Id()).or_insert_with(|| {
                    (
                        inline.value.clone(),
                        cssom::ParseCSSDeclarationList(&inline.value).into(),
                    )
                });
                if entry.0 != inline.value {
                    *entry = (
                        inline.value.clone(),
                        cssom::ParseCSSDeclarationList(&inline.value).into(),
                    );
                }
                entry.1.clone()
            };
            for declaration in declarations.iter().cloned() {
                let layer_priority =
                    self.layers
                        .LayerPriority(Origin::kAuthor, "", declaration.important, true);
                cascade.push(CascadedDeclaration {
                    declaration,
                    origin: Origin::kAuthor,
                    specificity: Specificity {
                        ids: 1 << 20,
                        ..Default::default()
                    },
                    source_order: order,
                    layer_priority,
                });
                order += 1;
            }
        }
        for declarations in self.document.AnimationStyles(node.Id()).values() {
            for declaration in declarations {
                cascade.push(CascadedDeclaration {
                    declaration: declaration.clone(),
                    origin: Origin::kAnimation,
                    specificity: Specificity::default(),
                    source_order: order,
                    layer_priority: 0,
                });
                order += 1;
            }
        }
        cascade.sort_by(CascadeLess);
        for item in &mut cascade {
            if let (Some(width), Some(height)) = (
                self.environment.viewport_width,
                self.environment.viewport_height,
            ) {
                item.declaration.value = crate::style_resolver::resolve_viewport_relative_lengths(
                    &item.declaration.value,
                    width,
                    height,
                );
            }
        }
        cascade
    }
}

// cpp: style_resolver/style_resolver.cc:7582-7755
fn PresentationHints(node: &DOMNode) -> Vec<CSSDeclaration> {
    let mut declarations = Vec::new();
    let mut add = |property: &str, value: String| {
        declarations.push(CSSDeclaration {
            property: property.into(),
            value,
            important: false,
        })
    };
    if node.Namespace() == DOMNamespace::kSVG {
        for property in [
            "color",
            "fill",
            "stroke",
            "stop-color",
            "stop-opacity",
            "stroke-width",
            "opacity",
            "visibility",
            "font-size",
            "font-family",
            "transform",
            "stroke-dasharray",
            "stroke-dashoffset",
            "stroke-linecap",
            "stroke-linejoin",
            "stroke-miterlimit",
            "fill-rule",
            "vector-effect",
            "shape-rendering",
            "paint-order",
        ] {
            if let Some(attribute) = node.FindAttribute(property) {
                let mut value = attribute.value.clone();
                if property == "font-size" && Number(Trim(&value)).is_some() {
                    value.push_str("px");
                }
                add(property, value);
            }
        }
        if matches!(node.Name(), "svg" | "foreignObject") {
            for property in ["width", "height"] {
                if let Some(attribute) = node.FindAttribute(property) {
                    let mut value = Trim(&attribute.value).to_owned();
                    if Number(&value).is_some() {
                        value.push_str("px");
                    }
                    add(property, value);
                }
            }
        }
    }
    if node.Namespace() == DOMNamespace::kMathML {
        if let Some(attribute) = node.FindAttribute("dir") {
            let value = Trim(&attribute.value).to_ascii_lowercase();
            if matches!(value.as_str(), "ltr" | "rtl") {
                add("direction", value);
            }
        }
        for (attribute, property) in [
            ("mathcolor", "color"),
            ("mathbackground", "background-color"),
        ] {
            if let Some(attribute) = node.FindAttribute(attribute) {
                add(property, attribute.value.clone());
            }
        }
        if let Some(attribute) = node.FindAttribute("mathsize") {
            let value = Trim(&attribute.value).to_ascii_lowercase();
            if !matches!(value.as_str(), "medium" | "smaller" | "larger" | "math")
                && !value.ends_with("large")
                && !value.ends_with("small")
            {
                add("font-size", attribute.value.clone());
            }
        }
        if let Some(attribute) = node.FindAttribute("displaystyle") {
            match Trim(&attribute.value).to_ascii_lowercase().as_str() {
                "true" => add("math-style", "normal".into()),
                "false" => add("math-style", "compact".into()),
                _ => {}
            }
        }
        if let Some(attribute) = node.FindAttribute("scriptlevel") {
            let value = Trim(&attribute.value);
            let relative = value.starts_with(['+', '-']);
            if let Some(number) = Number(value) {
                if number.trunc() == number
                    && (relative || number >= 0.0)
                    && number.abs() <= 32767.0
                {
                    add(
                        "math-depth",
                        if relative {
                            format!("add({value})")
                        } else {
                            value.into()
                        },
                    );
                }
            }
        }
        if node.Name() == "mi"
            && node
                .FindAttribute("mathvariant")
                .is_some_and(|a| Trim(&a.value).eq_ignore_ascii_case("normal"))
        {
            add("text-transform", "none".into());
        }
        if node.Name() == "mspace" {
            if let Some(value) = node
                .FindAttribute("width")
                .and_then(|a| MathPresentationLength(&a.value, false))
            {
                add("width", value);
            }
            let height = node
                .FindAttribute("height")
                .and_then(|a| MathPresentationLength(&a.value, false));
            let depth = node
                .FindAttribute("depth")
                .and_then(|a| MathPresentationLength(&a.value, false));
            match (height, depth) {
                (Some(height), Some(depth)) => add("height", format!("calc({height} + {depth})")),
                (Some(value), None) | (None, Some(value)) => add("height", value),
                _ => {}
            }
        }
        if node.Name() == "mpadded" {
            if let Some(value) = node
                .FindAttribute("width")
                .and_then(|a| MathPresentationLength(&a.value, false))
            {
                add("width", value);
            }
        }
    }
    if node.Namespace() == DOMNamespace::kHTML {
        let table_part = matches!(
            node.Name(),
            "thead" | "tbody" | "tfoot" | "tr" | "td" | "th"
        );
        let aligned_container = table_part || matches!(node.Name(), "div" | "p");
        if table_part {
            if let Some(attribute) = node
                .FindAttribute("valign")
                .filter(|a| !Trim(&a.value).is_empty())
            {
                add(
                    "vertical-align",
                    Trim(&attribute.value).to_ascii_lowercase(),
                );
            }
            if let Some(attribute) = node
                .FindAttribute("height")
                .filter(|a| !Trim(&a.value).is_empty())
            {
                let mut value = Trim(&attribute.value).to_owned();
                if Number(&value).is_some() {
                    value.push_str("px");
                }
                add("height", value);
            }
            if let Some(attribute) = node
                .FindAttribute("bgcolor")
                .filter(|a| !Trim(&a.value).is_empty())
            {
                add("background-color", Trim(&attribute.value).into());
            }
            if let Some(attribute) = node
                .FindAttribute("background")
                .filter(|a| !Trim(&a.value).is_empty())
            {
                add("background-image", format!("url(\"{}\")", attribute.value));
            }
        }
        if aligned_container {
            if let Some(attribute) = node.FindAttribute("align") {
                let value = Trim(&attribute.value).to_ascii_lowercase();
                let align = match value.as_str() {
                    "middle" | "center" => Some("-webkit-center"),
                    "absmiddle" if table_part => Some("center"),
                    "left" => Some("-webkit-left"),
                    "right" => Some("-webkit-right"),
                    _ => None,
                };
                if let Some(align) = align {
                    add("text-align", align.into());
                }
            }
        }
        if matches!(node.Name(), "td" | "th") {
            if let Some(attribute) = node
                .FindAttribute("width")
                .filter(|a| !Trim(&a.value).is_empty() && Trim(&a.value) != "0")
            {
                add("width", Trim(&attribute.value).into());
            }
            if node.FindAttribute("nowrap").is_some() {
                add("white-space", "nowrap".into());
            }
        } else if node.Name() == "table" {
            if let Some(attribute) = node
                .FindAttribute("cellspacing")
                .filter(|a| !Trim(&a.value).is_empty())
            {
                let mut value = Trim(&attribute.value).to_owned();
                if Number(&value).is_some_and(|n| n != 0.0) {
                    value.push_str("px");
                }
                add("border-spacing", value);
            }
        }
    }
    declarations
}

// cpp: style_resolver/style_resolver.cc:7553-7575
pub(crate) fn InitialElementStyle(
    document: &Document,
    index: usize,
    parent: Option<&layoutng_assembly::internal::layout_input::ComputedStyle>,
) -> (
    layoutng_assembly::internal::layout_input::ComputedStyle,
    bool,
    bool,
) {
    let node = document.Node(index);
    let html = node.Namespace() == DOMNamespace::kHTML;
    let svg_resource = node.Namespace() == DOMNamespace::kSVG
        && matches!(
            node.Name(),
            "defs"
                | "linearGradient"
                | "radialGradient"
                | "stop"
                | "pattern"
                | "clipPath"
                | "mask"
                | "marker"
                | "symbol"
        );
    let hidden_html = html
        && matches!(
            node.Name(),
            "area"
                | "base"
                | "basefont"
                | "datalist"
                | "head"
                | "link"
                | "meta"
                | "noembed"
                | "noframes"
                | "param"
                | "rp"
                | "script"
                | "source"
                | "style"
                | "template"
                | "title"
                | "track"
        );
    let mut own_generates_box = !svg_resource && !hidden_html;
    let mut display_contents = html && node.Name() == "slot";
    if display_contents {
        own_generates_box = false;
    }
    if node.FindAttribute("hidden").is_some()
        || (html && node.Name() == "dialog" && node.FindAttribute("open").is_none())
        || (html && node.Name() == "audio" && node.FindAttribute("controls").is_none())
    {
        own_generates_box = false;
        display_contents = false;
    }
    (
        InitialStyle(document, index, parent, false),
        own_generates_box,
        display_contents,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistent_document::DOMAttribute;
    use crate::style_resolver::cascade::{ResolveCustomProperties, ResolveDeclarationVariables};

    fn node(document: &Document, id: &str) -> usize {
        (0..document.NodeCount())
            .find(|&index| {
                document
                    .Node(index)
                    .FindAttribute("id")
                    .is_some_and(|a| a.value == id)
            })
            .unwrap()
    }
    fn winner(cascade: &[CascadedDeclaration], property: &str) -> Option<String> {
        let custom = ResolveCustomProperties(cascade, None);
        cascade
            .iter()
            .filter(|item| item.declaration.property == property)
            .filter_map(|item| ResolveDeclarationVariables(&item.declaration, &custom))
            .last()
            .map(|declaration| declaration.value)
    }

    #[test]
    fn grouped_targets_keep_specificity_source_order_and_live_pseudo_styles() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<html><body><div id=a class='x shared'>Hello</div><div id=b class=shared>World</div><input id=c class=x placeholder=search></body></html>");
        owner.GetDocumentMut().AppendStyleSheet(cssom::ParseCSS(
            "*, .x::before { color:red; content:'before' }
             .x, .x::after, .x::first-letter { color:green; content:'group' }
             .shared, .shared::before { color:blue; content:'shared' }
             .x::after { color:purple; content:'after' }
             #a::before, .x::before { color:orange; content:'specific' }
             .shared::before { color:black; content:'late' }
             .shared::first-letter { color:blue }
             .x::placeholder { color:cyan }",
        ));
        let a = node(owner.GetDocument(), "a");
        let b = node(owner.GetDocument(), "b");
        let c = node(owner.GetDocument(), "c");
        let environment = StyleEnvironment::default();
        crate::style_resolver::ResolveComputedStyles(&mut owner, &environment, &[]);
        {
            let context = StyleCascadeContext::new(owner.GetDocument(), &environment, &[]);
            let cascade = context.GatherPseudoCascade(a, PseudoTarget::Before);
            assert_eq!(winner(&cascade, "content").as_deref(), Some("'specific'"));
            assert_eq!(
                cascade
                    .iter()
                    .filter(|item| item.declaration.value == "'specific'")
                    .count(),
                1,
                "a rule matching both id and class contributes once, at the greater specificity"
            );
            assert_eq!(
                winner(
                    &context.GatherPseudoCascade(b, PseudoTarget::Before),
                    "content"
                )
                .as_deref(),
                Some("'late'")
            );
        }
        let color = |name: &str| layoutng_assembly::css_color_parser::ParseCSSColor(name).unwrap();
        let resolved = owner.GetDocument().ResolvedStyleFor(a).unwrap();
        assert_eq!(resolved.style.paint.color, color("blue"));
        assert_eq!(resolved.before.as_ref().unwrap().text, "specific");
        assert_eq!(
            resolved.before.as_ref().unwrap().style.paint.color,
            color("orange")
        );
        assert_eq!(resolved.after.as_ref().unwrap().text, "after");
        assert_eq!(
            resolved.first_letter.as_ref().unwrap().style.paint.color,
            color("blue")
        );
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(c)
                .unwrap()
                .placeholder
                .as_ref()
                .unwrap()
                .style
                .paint
                .color,
            color("cyan")
        );
        owner.GetDocumentMut().SetAttribute(
            a,
            DOMAttribute {
                local_name: "class".into(),
                value: "other".into(),
                ..Default::default()
            },
        );
        owner.GetDocumentMut().SetAttribute(
            b,
            DOMAttribute {
                local_name: "class".into(),
                value: "x shared shared".into(),
                ..Default::default()
            },
        );
        crate::style_resolver::ResolveComputedStyles(&mut owner, &environment, &[]);
        let resolved = owner.GetDocument().ResolvedStyleFor(a).unwrap();
        assert_eq!(resolved.style.paint.color, color("red"));
        assert_eq!(resolved.before.as_ref().unwrap().text, "specific");
        assert!(resolved.after.is_none() && resolved.first_letter.is_none());
        let resolved = owner.GetDocument().ResolvedStyleFor(b).unwrap();
        assert_eq!(resolved.before.as_ref().unwrap().text, "late");
        assert_eq!(resolved.after.as_ref().unwrap().text, "after");
        assert_eq!(
            resolved.first_letter.as_ref().unwrap().style.paint.color,
            color("blue")
        );
        // A later sheet rebuilds every target index from the new CSSOM revision.
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS(".x::after { content:'replacement' }"));
        crate::style_resolver::ResolveComputedStyles(&mut owner, &environment, &[]);
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(b)
                .unwrap()
                .after
                .as_ref()
                .unwrap()
                .text,
            "replacement"
        );
    }

    #[test]
    fn media_results_follow_environment_and_cssom_revision() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<html><body><div id=target class=box></div></body></html>",
        );
        owner.GetDocumentMut().AppendStyleSheet(cssom::ParseCSS(".box { color:black } @media screen and (min-width:600px) { .box { color:red } .box::before { content:'wide' } } @media (prefers-color-scheme:dark) { .box { color:white } }"));
        let index = node(owner.GetDocument(), "target");
        let mut environment = StyleEnvironment {
            viewport_width: Some(800.0),
            preferred_color_scheme: crate::style_resolver::PreferredColorScheme::Light,
            ..Default::default()
        };
        let context = StyleCascadeContext::new(owner.GetDocument(), &environment, &[]);
        assert_eq!(
            winner(&context.GatherElementCascade(index), "color").as_deref(),
            Some("red")
        );
        assert_eq!(
            winner(
                &context.GatherPseudoCascade(index, PseudoTarget::Before),
                "content"
            )
            .as_deref(),
            Some("'wide'")
        );
        drop(context);
        environment.viewport_width = Some(400.0);
        let context = StyleCascadeContext::new(owner.GetDocument(), &environment, &[]);
        assert_eq!(
            winner(&context.GatherElementCascade(index), "color").as_deref(),
            Some("black")
        );
        assert!(context
            .GatherPseudoCascade(index, PseudoTarget::Before)
            .is_empty());
        drop(context);
        environment.preferred_color_scheme = crate::style_resolver::PreferredColorScheme::Dark;
        owner.GetDocumentMut().AppendStyleSheet(cssom::ParseCSS(
            "@media (max-width:500px) { .box { color:blue } }",
        ));
        let context = StyleCascadeContext::new(owner.GetDocument(), &environment, &[]);
        assert_eq!(
            winner(&context.GatherElementCascade(index), "color").as_deref(),
            Some("blue")
        );
    }

    #[test]
    fn persistent_rules_read_mutations_control_state_layers_and_animation_order() {
        let mut owner = crate::test_html::html_parser::ParseHTML("<!doctype html><html><body><input id='target' class='one two' placeholder='search' style='--page-color: blue; color: var(--page-color)'><div id='other'></div></body></html>");
        let document = owner.GetDocumentMut();
        document.AppendStyleSheet(cssom::ParseCSS("@layer first, second; .one, .two { width: 20vw; } @layer first { .one { color: red !important; } } @layer second { .two { color: green !important; } } input:placeholder-shown::placeholder { color: gray; } #target::before { content: 'x'; } #other { color: yellow; }") );
        let target = node(document, "target");
        let environment = StyleEnvironment {
            viewport_width: Some(1024.0),
            viewport_height: Some(768.0),
            resolution_dppx: Some(1.0),
            ..Default::default()
        };
        let ua = [cssom::ParseCSS("#target { color: black; }")];
        let context = StyleCascadeContext::new(document, &environment, &ua);
        let cascade = context.GatherElementCascade(target);
        assert_eq!(
            cascade
                .iter()
                .filter(|item| item.declaration.property == "width")
                .count(),
            1,
            "selector-list buckets must not duplicate a declaration"
        );
        assert_eq!(winner(&cascade, "width").as_deref(), Some("204.800000px"));
        assert_eq!(winner(&cascade, "color").as_deref(), Some("red"));
        assert_eq!(
            winner(
                &context.GatherPseudoCascade(target, PseudoTarget::Before),
                "content"
            )
            .as_deref(),
            Some("'x'")
        );
        assert_eq!(
            winner(
                &context.GatherPseudoCascade(target, PseudoTarget::Placeholder),
                "color"
            )
            .as_deref(),
            Some("gray")
        );
        document.SetControlValue(target, "typed".into());
        document.SetAttribute(
            target,
            DOMAttribute {
                local_name: "class".into(),
                value: "two".into(),
                ..Default::default()
            },
        );
        let node_id = document.Node(target).Id();
        document.SetAnimationStyle(
            node_id,
            1,
            vec![CSSDeclaration {
                property: "color".into(),
                value: "purple".into(),
                important: false,
            }],
        );
        let context = StyleCascadeContext::new(document, &environment, &ua);
        assert!(context
            .GatherPseudoCascade(target, PseudoTarget::Placeholder)
            .is_empty());
        assert!(
            crate::style_resolver::persistent_selector::MatchesSelector(
                document,
                target,
                "input:placeholder-shown"
            ),
            "the public matcher keeps the source's attribute-only default"
        );
        assert_eq!(
            winner(&context.GatherElementCascade(target), "color").as_deref(),
            Some("green"),
            "important author beats animation"
        );
        document.SetAttribute(
            target,
            DOMAttribute {
                local_name: "class".into(),
                value: "".into(),
                ..Default::default()
            },
        );
        assert_eq!(
            winner(
                &StyleCascadeContext::new(document, &environment, &ua).GatherElementCascade(target),
                "color"
            )
            .as_deref(),
            Some("purple"),
            "animation beats normal inline author"
        );
        let important_ua = [cssom::ParseCSS("#target { color: orange !important; }")];
        assert_eq!(
            winner(
                &StyleCascadeContext::new(document, &environment, &important_ua)
                    .GatherElementCascade(target),
                "color"
            )
            .as_deref(),
            Some("orange")
        );
    }

    #[test]
    fn presentation_hints_and_html_box_semantics_use_source_namespaces() {
        let mut owner = crate::test_html::html_parser::ParseHTML("<!doctype html><html><head><title id='title'>x</title></head><body><table id='table' cellspacing='2'><tr><td id='cell' align='absmiddle' height='20' width='0' nowrap background=' a.png '>x</td></tr></table><svg id='svg' width='40' font-size='20'><defs id='defs'></defs><g id='group' hidden></g></svg><math><mspace id='space' width='3em' height='4px' depth='2px'/><mi id='mi' mathvariant='normal' scriptlevel='+2' mathsize='larger'>x</mi></math><dialog id='dialog'></dialog><slot id='slot'></slot></body></html>");
        let document = owner.GetDocumentMut();
        document.AppendStyleSheet(cssom::ParseCSS("svg { width: 60px; }"));
        let environment = StyleEnvironment::default();
        let context = StyleCascadeContext::new(document, &environment, &[]);
        let cell = context.GatherElementCascade(node(document, "cell"));
        assert_eq!(winner(&cell, "text-align").as_deref(), Some("center"));
        assert_eq!(winner(&cell, "height").as_deref(), Some("20px"));
        assert_eq!(winner(&cell, "width"), None);
        assert_eq!(winner(&cell, "white-space").as_deref(), Some("nowrap"));
        assert_eq!(
            winner(&cell, "background-image").as_deref(),
            Some("url(\" a.png \")")
        );
        assert_eq!(
            winner(
                &context.GatherElementCascade(node(document, "svg")),
                "width"
            )
            .as_deref(),
            Some("60px")
        );
        assert_eq!(
            winner(
                &context.GatherElementCascade(node(document, "space")),
                "height"
            )
            .as_deref(),
            Some("calc(4px + 2px)")
        );
        let mi = context.GatherElementCascade(node(document, "mi"));
        assert_eq!(winner(&mi, "math-depth").as_deref(), Some("add(+2)"));
        assert_eq!(winner(&mi, "text-transform").as_deref(), Some("none"));
        assert_eq!(winner(&mi, "font-size"), None);
        for id in ["title", "defs", "group", "dialog"] {
            assert!(
                !InitialElementStyle(document, node(document, id), None).1,
                "{id}"
            );
        }
        let (_, generates_box, display_contents) =
            InitialElementStyle(document, node(document, "slot"), None);
        assert!(!generates_box && display_contents);
        assert_eq!(MathPresentationLength("calc(2px)", false), None);
        assert_eq!(MathPresentationLength("20%", false), None);
        assert_eq!(MathPresentationLength("-2px", false), None);
        assert_eq!(
            MathPresentationLength("-2px", true).as_deref(),
            Some("-2px")
        );
    }
}
