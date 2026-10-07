#![allow(non_snake_case)]
use crate::style_resolver::{
    background::BackgroundCascadeState,
    cascade::{Origin, ResolveCustomProperties, ResolveDeclarationVariables},
    css_wide::{ApplyCSSWideKeyword, CSSWideKeywordSource},
    finish_style::{NormalizeOverflow, ResolveSVGCurrentColor},
    fonts::{FontSizeOf, ParseComputedFontSize, ParseFontShorthand},
    initial_style::InitialStyle,
    persistent_cascade::{InitialElementStyle, MathPresentationLength, StyleCascadeContext},
    selector::{PseudoTarget, SourceSpace},
    CustomProperties, StyleEnvironment,
};
use crate::{
    persistent_document::{
        DOMNamespace, DOMNode, DOMNodeType, PseudoElement, ResolvedNodeStyle, DOM,
    },
    Document,
};
use cssom::{CSSDeclaration, CSSStyleSheet};
use layoutng_assembly::internal::layout_input::{ComputedStyle, Display, MathStyle, TextAlign};

// Geometry queries do not consume opacity or colors. Keep the existing
// conservative layout comparison, including its inline background transition
// check. Opacity crossing 1 can add/remove a paint-backed inline fragment in
// this implementation, so it is deliberately excluded from this proof.
fn MeasurementGeometryEquivalent(old: &ResolvedNodeStyle, new: &ResolvedNodeStyle) -> bool {
    if old.generates_box != new.generates_box
        || old.own_generates_box != new.own_generates_box
        || old.display_contents != new.display_contents
        || old.own_display_contents != new.own_display_contents
        || old.before != new.before
        || old.after != new.after
        || old.first_letter != new.first_letter
        || old.placeholder != new.placeholder
        || (old.style.paint.opacity != 1.0) != (new.style.paint.opacity != 1.0)
    {
        return false;
    }
    // Called only after the ordinary layout comparison failed. If opacity
    // did not change, its existing conservative rejection already proves that
    // some other geometry input changed; do not compare the whole style twice.
    if old.style.paint.opacity == new.style.paint.opacity {
        return false;
    }
    let mut normalized = old.style.clone();
    normalized.paint.opacity = new.style.paint.opacity;
    normalized.LayoutEquivalent(&new.style)
}

fn Trim(value: &str) -> &str {
    value.trim_matches(|c: char| c.source_space())
}
fn ResolvedDeclaration(
    item: &crate::style_resolver::cascade::CascadedDeclaration,
    custom: &CustomProperties,
) -> CSSDeclaration {
    ResolveDeclarationVariables(&item.declaration, custom).unwrap_or_else(|| CSSDeclaration {
        property: item.declaration.property.clone(),
        value: "unset".into(),
        ..Default::default()
    })
}
// cpp: style_resolver/style_resolver.cc:4618-4998,5122-5132,5600-6283
// The already translated declaration handlers are shared by static and live
// DOM resolution. Values have undergone the source viewport prepass before
// variable substitution, so no second conversion happens in this boundary.
fn ApplyDeclaration(
    style: &mut ComputedStyle,
    generates: &mut bool,
    contents: &mut bool,
    background: &mut BackgroundCascadeState,
    declaration: &CSSDeclaration,
    mode: layoutng_assembly::internal::layout_input::WritingMode,
    direction: layoutng_assembly::internal::layout_input::TextDirection,
    namespace: DOMNamespace,
) {
    if declaration.property == "display" {
        crate::style_resolver::box_properties::ApplyDisplay(
            style,
            generates,
            contents,
            &declaration.value,
        );
    } else if !background.Apply(style, &declaration.property, &declaration.value) {
        crate::style_resolver::apply_with_axes(
            style,
            &declaration.property,
            &declaration.value,
            (f64::NAN, f64::NAN),
            mode,
            direction,
        );
        // cpp: style_resolver/style_resolver.cc:6220-6228
        // Unitless SVG transform lists are accepted only for SVG nodes and
        // only after the CSS transform grammar declines the declaration.
        if namespace == DOMNamespace::kSVG
            && declaration.property == "transform"
            && declaration.value.to_ascii_lowercase() != "none"
            && crate::style_resolver::transform_parser::ParseCSSTransform(
                &declaration.value,
                FontSizeOf(style),
            )
            .is_none()
        {
            if let Some(matrix) =
                crate::style_resolver::transform_parser::ParseSVGTransform(&declaration.value)
            {
                style.paint.transform = Some(
                    layoutng_assembly::internal::paint_input::PaintTransform::from_matrix(&matrix),
                );
            }
        }
    }
}

// cpp: style_resolver/style_resolver.cc:7423-7542
fn ResolvePseudoElement(
    context: &StyleCascadeContext<'_>,
    document: &Document,
    index: usize,
    originating: &ComputedStyle,
    inherited_custom: &CustomProperties,
    environment: &StyleEnvironment,
    originating_generates: bool,
    originating_contents: bool,
    target: PseudoTarget,
) -> Option<PseudoElement> {
    let node = document.Node(index);
    let mut cascade = context.GatherPseudoCascade(index, target);
    if cascade.is_empty() {
        return None;
    }
    if let (Some(width), Some(height)) = (environment.viewport_width, environment.viewport_height) {
        for item in &mut cascade {
            item.declaration.value = crate::style_resolver::resolve_viewport_relative_lengths(
                &item.declaration.value,
                width,
                height,
            );
        }
    }
    let mut style = InitialStyle(document, index, Some(originating), true);
    let mut background = BackgroundCascadeState::default();
    let (mut generates, mut contents) = (true, false);
    let mut content = (false, String::new());
    let custom = ResolveCustomProperties(&cascade, Some(inherited_custom));
    let states = crate::style_resolver::prepare_cascade(
        InitialStyle(document, index, None, true),
        style.clone(),
        Some(originating),
        originating_generates,
        originating_contents,
        &mut style,
        true,
        false,
        &cascade,
        &custom,
    );
    let [initial, inherited, unset, reverted] = states.sources(
        Some(originating),
        originating_generates,
        originating_contents,
    );
    for item in &cascade {
        if item.declaration.property.starts_with("--") {
            continue;
        }
        let declaration = ResolvedDeclaration(item, &custom);
        let property = declaration.property.as_str();
        if matches!(property, "font-size" | "font-weight")
            || (property == "initial-letter" && target != PseudoTarget::FirstLetter)
        {
            continue;
        }
        if property == "content" {
            if CSSWideKeywordSource(&declaration.value, &initial, &inherited, &unset, &reverted)
                .is_some()
            {
                content = (false, String::new());
            } else if let Some(parsed) =
                crate::style_resolver::generated_content::ParseWithAttributes(
                    &declaration.value,
                    node.Namespace(),
                    |name| node.FindAttribute(name).map(|a| a.value.clone()),
                )
            {
                content = parsed;
            }
            continue;
        }
        if property == "all"
            && CSSWideKeywordSource(&declaration.value, &initial, &inherited, &unset, &reverted)
                .is_some()
        {
            content = (false, String::new());
        }
        if ApplyCSSWideKeyword(
            &declaration,
            states.logical_writing_mode,
            states.logical_direction,
            &initial,
            &inherited,
            &unset,
            &reverted,
            &mut style,
            &mut generates,
            &mut contents,
            &mut background,
        ) {
            states.restore_font(&mut style);
            continue;
        }
        ApplyDeclaration(
            &mut style,
            &mut generates,
            &mut contents,
            &mut background,
            &declaration,
            states.logical_writing_mode,
            states.logical_direction,
            node.Namespace(),
        );
    }
    NormalizeOverflow(&mut style);
    ResolveSVGCurrentColor(&mut style);
    background.Export(&mut style);
    ResolveImageResources(document, &mut style);
    if matches!(
        target,
        PseudoTarget::FirstLetter | PseudoTarget::Placeholder
    ) {
        return Some(PseudoElement {
            style,
            text: String::new(),
            display_contents: false,
        });
    }
    if !content.0 || (!generates && !contents) {
        return None;
    }
    Some(PseudoElement {
        style,
        text: content.1,
        display_contents: contents,
    })
}

// A child-list mutation already invalidates the container's whole subtree,
// which covers ordinary child/sibling selectors. Widen to the parent only when
// the mutation can change selector state on the container itself (for example
// `:empty`) and that state may feed a sibling combinator. This is the same
// feature-gated invalidation shape Blink uses instead of a document-wide
// "some selector has a sibling combinator" switch.
fn ChildrenCanKeepContainerScope(document: &Document, index: usize) -> bool {
    let mut ancestor = Some(index);
    while let Some(node) = ancestor {
        let node_ref = document.Node(node);
        // Descendant text/options can affect control values and placeholder
        // matching, even through an unusually nested head in a mutable DOM.
        if node_ref.IsHTMLElement("input")
            || node_ref.IsHTMLElement("textarea")
            || node_ref.IsHTMLElement("select")
        {
            return false;
        }
        ancestor = node_ref.Parent();
    }
    [
        &document.StyleState().rules.user_agent,
        &document.StyleState().rules.author,
    ]
    .iter()
    .filter_map(|r| r.as_ref())
    .all(|r| {
        !r.children_container_dependency_fallback
            && !r.has_dependency_fallback
            && !r.children_container_fallback_hosts.iter().any(|host| {
                crate::style_resolver::persistent_selector::MatchesSelector(document, index, host)
            })
            && !r.empty_container_hosts.iter().any(|host| {
                crate::style_resolver::persistent_selector::MatchesSelector(document, index, host)
            })
    })
}

fn AppendNeedsNthLastInvalidation(document: &Document, index: usize) -> bool {
    let state = document.StyleState();
    for rules in [&state.rules.user_agent, &state.rules.author]
        .into_iter()
        .filter_map(|rules| rules.as_ref())
    {
        if !rules.depends_on_nth_last_children {
            continue;
        }
        if rules.nth_last_child_host_fallback {
            // A universal/unknown reverse-position target can affect any old
            // element child. New nodes have no stale style and are handled by
            // inserted_style_subtrees.
            if document.Node(index).Children().iter().any(|child| {
                document.Node(*child).Type() == DOMNodeType::kElement
                    && !state.inserted_style_subtrees.contains(child)
            }) {
                return true;
            }
            continue;
        }
        if document.Node(index).Children().iter().any(|child| {
            document.Node(*child).Type() == DOMNodeType::kElement
                && !state.inserted_style_subtrees.contains(child)
                && rules.nth_last_child_hosts.iter().any(|host| {
                    crate::style_resolver::persistent_selector::MatchesSelector(
                        document, *child, host,
                    )
                })
        }) {
            return true;
        }
    }
    false
}

fn HasInsertedAncestor(document: &Document, mut index: usize) -> bool {
    loop {
        if document
            .StyleState()
            .inserted_style_subtrees
            .contains(&index)
        {
            return true;
        }
        let Some(parent) = document.Node(index).Parent() else {
            return false;
        };
        index = parent;
    }
}

fn HasInsertedParent(document: &Document, index: usize) -> bool {
    document
        .Node(index)
        .Parent()
        .is_some_and(|mut parent| loop {
            if document
                .StyleState()
                .inserted_style_subtrees
                .contains(&parent)
            {
                break true;
            }
            let Some(next) = document.Node(parent).Parent() else {
                break false;
            };
            parent = next;
        })
}

fn AddInvalidationPath(
    document: &Document,
    invalidation: &mut StyleInvalidation,
    mut index: usize,
) {
    loop {
        invalidation.paths.insert(index);
        let Some(parent) = document.Node(index).Parent() else {
            break;
        };
        index = parent;
    }
}

// Blink's NthSiblingInvalidationSet stores the features of selector targets
// affected by an :nth-* mutation. Our compiled rule index already stores the
// same target candidate keys, so reuse it instead of invalidating the whole
// container subtree.
fn AddNthLastAppendInvalidations(
    document: &Document,
    parent: usize,
    invalidation: &mut StyleInvalidation,
) {
    use crate::style_resolver::selector::PseudoTarget;
    let state = document.StyleState();
    let rule_sets = [&state.rules.user_agent, &state.rules.author];
    let targets = [
        PseudoTarget::Element,
        PseudoTarget::Before,
        PseudoTarget::After,
        PseudoTarget::FirstLetter,
        PseudoTarget::Placeholder,
    ];
    let mut stack = document.Node(parent).Children().to_vec();
    while let Some(index) = stack.pop() {
        if document.Node(index).Type() != DOMNodeType::kElement {
            continue;
        }
        if state.inserted_style_subtrees.contains(&index) {
            // Freshly inserted subtrees are already resolved from current DOM.
            continue;
        }
        let affected = rule_sets
            .iter()
            .filter_map(|rules| rules.as_ref())
            .any(|rules| {
                targets.iter().any(|&target| {
                    crate::style_resolver::style_rule_index::CompiledCandidateRules(
                        rules,
                        document.Node(index),
                        target,
                    )
                    .into_iter()
                    .any(|rule| rule.depends_on_nth_last_children)
                })
            });
        if affected {
            invalidation.nodes.insert(index);
            AddInvalidationPath(document, invalidation, index);
        }
        stack.extend(document.Node(index).Children().iter().copied());
    }
}

// Equivalent target filtering for the previous :last-* / :only-* sibling.
// The root itself and descendant selector targets are candidate-indexed; an
// actual computed change still propagates inherited values through the normal
// ResolveElement path.
fn AddLastChildInvalidations(
    document: &Document,
    root: usize,
    invalidation: &mut StyleInvalidation,
) {
    use crate::style_resolver::selector::PseudoTarget;
    let state = document.StyleState();
    if state.inserted_style_subtrees.contains(&root) {
        return;
    }
    let rule_sets = [&state.rules.user_agent, &state.rules.author];
    let targets = [
        PseudoTarget::Element,
        PseudoTarget::Before,
        PseudoTarget::After,
        PseudoTarget::FirstLetter,
        PseudoTarget::Placeholder,
    ];
    let mut stack = vec![root];
    while let Some(index) = stack.pop() {
        if document.Node(index).Type() != DOMNodeType::kElement {
            continue;
        }
        let affected = rule_sets
            .iter()
            .filter_map(|rules| rules.as_ref())
            .any(|rules| {
                targets.iter().any(|&target| {
                    crate::style_resolver::style_rule_index::CompiledCandidateRules(
                        rules,
                        document.Node(index),
                        target,
                    )
                    .into_iter()
                    .any(|rule| rule.depends_on_last_children)
                })
            });
        if affected {
            invalidation.nodes.insert(index);
            AddInvalidationPath(document, invalidation, index);
        }
        stack.extend(document.Node(index).Children().iter().copied());
    }
}

fn AttributeHasNoSelectorReader(
    document: &Document,
    kind: cssom::compiled_rules::SelectorOnlyAttribute,
) -> bool {
    [
        &document.StyleState().rules.user_agent,
        &document.StyleState().rules.author,
    ]
    .iter()
    .filter_map(|r| r.as_ref())
    .all(|r| r.attribute_selector_dependencies & kind.mask() == 0)
}

fn AttributeHasNoStyleReader(
    document: &Document,
    index: usize,
    kind: cssom::compiled_rules::SelectorOnlyAttribute,
) -> bool {
    let bit = kind.mask();
    if [
        &document.StyleState().rules.user_agent,
        &document.StyleState().rules.author,
    ]
    .iter()
    .filter_map(|r| r.as_ref())
    .any(|r| r.selector_only_attribute_dependencies & bit != 0)
    {
        return false;
    }
    let Some(old) = document.ResolvedStyleFor(index) else {
        return false;
    };
    let reads = crate::style_resolver::style_rule_index::attribute_dependencies::DeclarationAttributeDependencies;
    // Inherited inline custom properties can carry attr() through var(). The
    // immutable old bindings include every inherited binding; pending ancestor
    // or stylesheet changes still force the normal existing cascade path.
    if old
        .custom_properties
        .values()
        .filter_map(|value| value.as_deref())
        .any(|value| reads(value) & bit != 0)
    {
        return false;
    }
    !document
        .Node(index)
        .FindAttribute("style")
        .is_some_and(|attribute| reads(&attribute.value) & bit != 0)
}

// cpp: style_resolver/style_resolver.cc:7544-7576,7805-8073
fn SameDescendantStyleInputs(a: &ResolvedNodeStyle, b: &ResolvedNodeStyle) -> bool {
    // ComputedStyle::Difference distinguishes kPseudoElementStyle from the
    // inherited/descendant-affecting differences. DOM children do not inherit
    // the owner's generated pseudo styles. Keep every actual child input,
    // including the full style for explicit inherit on non-inherited fields.
    let ResolvedNodeStyle {
        style,
        generates_box,
        own_generates_box,
        own_display_contents,
        display_contents,
        custom_properties,
        has_explicit_inheritance: _,
        before: _,
        after: _,
        first_letter: _,
        placeholder: _,
    } = a;
    style == &b.style
        && *generates_box == b.generates_box
        && *own_generates_box == b.own_generates_box
        && *own_display_contents == b.own_display_contents
        && *display_contents == b.display_contents
        && (std::sync::Arc::ptr_eq(custom_properties, &b.custom_properties)
            || custom_properties == &b.custom_properties)
}

fn NonInheritedChangeDoesNotAffectChildren(
    document: &Document,
    index: usize,
    old: &ResolvedNodeStyle,
    new: &ResolvedNodeStyle,
) -> bool {
    // ComputedStyle::Difference (computed_style.cc:473-476) propagates a
    // non-inherited change when a child has explicit inheritance. Restrict
    // this boundary's kNonInherited subset to opacity/transform; compare every
    // other style, box and custom-property input exactly as before.
    if old.style.paint.opacity == new.style.paint.opacity
        && old.style.paint.transform == new.style.paint.transform
    {
        return false;
    }
    let mut normalized = old.style.clone();
    normalized.paint.opacity = new.style.paint.opacity;
    normalized.paint.transform = new.style.paint.transform.clone();
    if normalized != new.style {
        return false;
    }
    if old.generates_box != new.generates_box
        || old.own_generates_box != new.own_generates_box
        || old.own_display_contents != new.own_display_contents
        || old.display_contents != new.display_contents
        || !(std::sync::Arc::ptr_eq(&old.custom_properties, &new.custom_properties)
            || old.custom_properties == new.custom_properties)
    {
        return false;
    }
    document.Node(index).Children().iter().all(|&child| {
        document.Node(child).Type() != DOMNodeType::kElement
            || document
                .ResolvedStyleFor(child)
                .is_some_and(|style| style.has_explicit_inheritance == Some(false))
    })
}

// cpp: style_resolver/style_resolver.cc:7544-7576,7805-8073
fn ResolveElement(
    context: &StyleCascadeContext<'_>,
    document: &Document,
    index: usize,
    parent: Option<&ComputedStyle>,
    inherited_custom: Option<&CustomProperties>,
    environment: &StyleEnvironment,
    parent_generates: bool,
    parent_contents: bool,
    ancestor_hidden: bool,
    pending: &mut Vec<(usize, ResolvedNodeStyle)>,
    invalidation: &StyleInvalidation,
    force: bool,
    parent_changed: bool,
) {
    // Selector/structure invalidation remains sticky throughout the subtree.
    // A parent computed-style change requires this element to be recalculated,
    // but only this element's actual resulting change propagates to its children.
    let force = force || invalidation.subtrees.contains(&index);
    let recalc_self = force || parent_changed;
    let old = document.ResolvedStyleFor(index);
    if !recalc_self && !invalidation.paths.contains(&index) && old.is_some() {
        return;
    }
    // Resource completion changes only non-inherited image ids in an already
    // computed style. Keep selector/cascade invalidation for DOM changes; do not
    // promote an image notification through sibling or :has dependencies.
    if !recalc_self
        && invalidation.resources.contains(&index)
        && !invalidation.nodes.contains(&index)
    {
        if let Some(old) = old {
            let mut refreshed = old.clone();
            ResolveImageResources(document, &mut refreshed.style);
            for pseudo in [
                &mut refreshed.before,
                &mut refreshed.after,
                &mut refreshed.first_letter,
                &mut refreshed.placeholder,
            ]
            .into_iter()
            .flatten()
            {
                ResolveImageResources(document, &mut pseudo.style);
            }
            pending.push((index, refreshed));
            for &child in document.Node(index).Children() {
                if document.Node(child).Type() == DOMNodeType::kElement {
                    ResolveElement(
                        context,
                        document,
                        child,
                        Some(&old.style),
                        Some(&old.custom_properties),
                        environment,
                        old.own_generates_box,
                        old.own_display_contents,
                        ancestor_hidden || (!old.own_generates_box && !old.own_display_contents),
                        pending,
                        invalidation,
                        false,
                        false,
                    );
                }
            }
            return;
        }
    }
    if !recalc_self && !invalidation.nodes.contains(&index) {
        if let Some(old) = old {
            for &child in document.Node(index).Children() {
                if document.Node(child).Type() == DOMNodeType::kElement {
                    ResolveElement(
                        context,
                        document,
                        child,
                        Some(&old.style),
                        Some(&old.custom_properties),
                        environment,
                        old.own_generates_box,
                        old.own_display_contents,
                        ancestor_hidden || (!old.own_generates_box && !old.own_display_contents),
                        pending,
                        invalidation,
                        false,
                        false,
                    );
                }
            }
            return;
        }
    }
    let node = document.Node(index);
    let (mut style, mut own_generates, mut contents) = InitialElementStyle(document, index, parent);
    let mut background = BackgroundCascadeState::default();
    let cascade = context.GatherElementCascade(index);
    let computed_custom = ResolveCustomProperties(&cascade, inherited_custom);
    // A forced recalc can reconstruct the root's same bindings. Keep its old
    // immutable map so descendants share identity across passes as well.
    let custom = match old {
        Some(old)
            if std::sync::Arc::ptr_eq(&old.custom_properties, &computed_custom)
                || old.custom_properties == computed_custom =>
        {
            old.custom_properties.clone()
        }
        _ => computed_custom,
    };
    let states = crate::style_resolver::prepare_cascade(
        InitialStyle(document, index, None, true),
        InitialStyle(document, index, parent, true),
        parent,
        parent_generates,
        parent_contents,
        &mut style,
        own_generates,
        contents,
        &cascade,
        &custom,
    );
    let [initial, inherited, unset, reverted] =
        states.sources(parent, parent_generates, parent_contents);
    let mut has_explicit_inheritance = false;
    for item in &cascade {
        if item.declaration.property.starts_with("--") {
            continue;
        }
        let declaration = ResolvedDeclaration(item, &custom);
        // Match StyleBuilder::ApplyProperty after variable resolution. Mark
        // any inherit conservatively, including all:inherit and var fallback;
        // inherited properties need not be excluded to make this proof safe.
        has_explicit_inheritance |= Trim(&declaration.value).eq_ignore_ascii_case("inherit");
        let property = declaration.property.as_str();
        if property == "font-size" {
            let flag = if let Some(source) =
                CSSWideKeywordSource(&declaration.value, &initial, &inherited, &unset, &reverted)
            {
                Some(
                    source
                        .style
                        .extended
                        .as_ref()
                        .is_some_and(|e| e.font_size_math),
                )
            } else if ParseComputedFontSize(&declaration.value, FontSizeOf(inherited.style))
                .is_some()
            {
                Some(false)
            } else {
                None
            };
            if let Some(flag) = flag {
                style
                    .extended
                    .get_or_insert_with(Default::default)
                    .font_size_math = flag;
            }
            continue;
        }
        if property == "font-weight" {
            continue;
        }
        if property == "math-style" {
            let value = Trim(&declaration.value).to_ascii_lowercase();
            let parsed = if let Some(source) =
                CSSWideKeywordSource(&value, &initial, &inherited, &unset, &reverted)
            {
                Some(
                    source
                        .style
                        .extended
                        .as_ref()
                        .map_or(MathStyle::kNormal, |e| e.math_style),
                )
            } else {
                match value.as_str() {
                    "normal" => Some(MathStyle::kNormal),
                    "compact" => Some(MathStyle::kCompact),
                    _ => None,
                }
            };
            if let Some(value) = parsed {
                style
                    .extended
                    .get_or_insert_with(Default::default)
                    .math_style = value;
            }
            continue;
        }
        if property == "math-depth" {
            let value = Trim(&declaration.value).to_ascii_lowercase();
            let parent_extra = parent.and_then(|s| s.extended.as_ref());
            let parent_depth = parent_extra.map_or(0, |e| e.math_depth);
            let parent_math_style = parent_extra.map_or(MathStyle::kNormal, |e| e.math_style);
            let parsed = if let Some(source) =
                CSSWideKeywordSource(&value, &initial, &inherited, &unset, &reverted)
            {
                Some(source.style.extended.as_ref().map_or(0, |e| e.math_depth))
            } else if value == "auto-add" {
                Some(parent_depth + i32::from(parent_math_style == MathStyle::kCompact))
            } else {
                let relative = value.starts_with("add(") && value.ends_with(')');
                let number = if relative {
                    &value[4..value.len() - 1]
                } else {
                    &value
                };
                crate::style_resolver::number::Number(number)
                    .filter(|n| n.trunc() == *n && n.abs() <= 32767.0)
                    .map(|n| {
                        ((n as i32) + if relative { parent_depth } else { 0 }).clamp(-32768, 32767)
                    })
            };
            if let Some(value) = parsed {
                style
                    .extended
                    .get_or_insert_with(Default::default)
                    .math_depth = value;
            }
            continue;
        }
        if property == "initial-letter" {
            continue;
        }
        if ApplyCSSWideKeyword(
            &declaration,
            states.logical_writing_mode,
            states.logical_direction,
            &initial,
            &inherited,
            &unset,
            &reverted,
            &mut style,
            &mut own_generates,
            &mut contents,
            &mut background,
        ) {
            states.restore_font(&mut style);
            continue;
        }
        ApplyDeclaration(
            &mut style,
            &mut own_generates,
            &mut contents,
            &mut background,
            &declaration,
            states.logical_writing_mode,
            states.logical_direction,
            node.Namespace(),
        );
        if node.Namespace() == DOMNamespace::kMathML
            && property == "font"
            && ParseFontShorthand(&declaration.value, FontSizeOf(inherited.style)).is_some()
        {
            style
                .extended
                .get_or_insert_with(Default::default)
                .font_size_math = false;
        }
        let e = style.extended.get_or_insert_with(Default::default);
        let author = item.origin == Origin::kAuthor;
        if author && (property == "background" || property.starts_with("background-")) {
            e.has_author_background = true;
        }
        if author
            && (property == "border-radius"
                || (property.starts_with("border-") && property.ends_with("-radius")))
        {
            e.has_author_border_radius = true;
        } else if author
            && (property == "border"
                || (property.starts_with("border-")
                    && property != "border-collapse"
                    && property != "border-spacing"))
        {
            e.has_author_border = true;
        }
        if author && (property == "outline" || property.starts_with("outline-")) {
            e.has_author_outline = true;
        }
    }
    AdjustElement(node, &mut style);
    NormalizeOverflow(&mut style);
    ResolveSVGCurrentColor(&mut style);
    background.Export(&mut style);
    ResolveImageResources(document, &mut style);
    let input_type = (node.Name() == "input")
        .then(|| node.FindAttribute("type"))
        .flatten();
    if node.Name() == "template"
        || input_type.is_some_and(|a| a.value.eq_ignore_ascii_case("hidden"))
    {
        own_generates = false;
        contents = false;
    }
    if contents
        && node
            .Parent()
            .is_some_and(|p| document.Node(p).Type() == DOMNodeType::kDocument)
    {
        style.display = Display::kBlock;
        own_generates = true;
        contents = false;
    }
    let mut resolved = ResolvedNodeStyle {
        style: style.clone(),
        generates_box: !ancestor_hidden && own_generates,
        own_generates_box: own_generates,
        own_display_contents: contents,
        display_contents: !ancestor_hidden && contents,
        custom_properties: custom.clone(),
        has_explicit_inheritance: Some(has_explicit_inheritance),
        ..Default::default()
    };
    if !ancestor_hidden && (own_generates || contents) {
        for (target, output) in [
            (PseudoTarget::Before, &mut resolved.before),
            (PseudoTarget::After, &mut resolved.after),
            (PseudoTarget::FirstLetter, &mut resolved.first_letter),
        ] {
            *output = ResolvePseudoElement(
                context,
                document,
                index,
                &style,
                &custom,
                environment,
                own_generates,
                contents,
                target,
            );
        }
        if node.IsHTMLElement("input") || node.IsHTMLElement("textarea") {
            resolved.placeholder = ResolvePseudoElement(
                context,
                document,
                index,
                &style,
                &custom,
                environment,
                own_generates,
                contents,
                PseudoTarget::Placeholder,
            );
        }
    }
    let descendants_changed = old.is_none_or(|old| {
        !SameDescendantStyleInputs(old, &resolved)
            && !NonInheritedChangeDoesNotAffectChildren(document, index, old, &resolved)
    });
    pending.push((index, resolved));
    for &child in node.Children() {
        if document.Node(child).Type() == DOMNodeType::kElement {
            ResolveElement(
                context,
                document,
                child,
                Some(&style),
                Some(&custom),
                environment,
                own_generates,
                contents,
                ancestor_hidden || (!own_generates && !contents),
                pending,
                invalidation,
                force,
                descendants_changed,
            );
        }
    }
}

// cpp: style_resolver/style_resolver.cc:3038-3042
// Rust background parsing shares its CSS grammar with the static adapter;
// native DOM resolution supplies the document-owned immutable image ids.
fn ResolveImageResources(document: &Document, style: &mut ComputedStyle) {
    let paint = &mut *style.paint;
    for layer in paint
        .background_images
        .iter_mut()
        .chain(paint.mask_images.iter_mut().map(|mask| &mut mask.image))
    {
        if !layer.source_url.is_empty() {
            if let Some(resource) = document.ImageResourceFor(&layer.source_url) {
                layer.resource_id = resource.id;
            }
        }
    }
}

// cpp: style_resolver/style_resolver.cc:7957-8006
fn AdjustElement(node: &DOMNode, style: &mut ComputedStyle) {
    let font_size = FontSizeOf(style);
    let e = style.extended.get_or_insert_with(Default::default);
    if node.IsHTMLElement("table")
        && matches!(
            e.text_align,
            TextAlign::kWebkitLeft | TextAlign::kWebkitCenter | TextAlign::kWebkitRight
        )
    {
        e.text_align = TextAlign::kStart;
    }
    if node.Namespace() != DOMNamespace::kMathML {
        return;
    }
    let length = |name: &str, negative: bool| {
        node.FindAttribute(name)
            .and_then(|a| MathPresentationLength(&a.value, negative))
            .and_then(|value| crate::style_resolver::border_radius::Length(&value, font_size))
    };
    match node.Name() {
        "mspace" => {
            if let Some(height) = length("height", false) {
                e.math_baseline = Some(height);
            }
        }
        "mpadded" => {
            e.math_baseline = length("height", false);
            e.math_padded_depth = length("depth", false);
            e.math_lspace = length("lspace", false);
            e.math_padded_voffset = length("voffset", true);
        }
        "mfrac" => e.math_fraction_bar_thickness = length("linethickness", true),
        "mo" => {
            e.math_lspace = length("lspace", true);
            e.math_rspace = length("rspace", true);
            e.math_min_size = length("minsize", true);
            e.math_max_size = length("maxsize", true);
        }
        _ => {}
    }
}

#[derive(Default)]
struct StyleInvalidation {
    resources: std::collections::HashSet<usize>,
    nodes: std::collections::HashSet<usize>,
    subtrees: std::collections::HashSet<usize>,
    paths: std::collections::HashSet<usize>,
}

/// One update entry; all persistent caches and results belong to the document.
/// Keeping ownership there also lets script style reads share the frame cache.
pub struct StyleEngine;
impl StyleEngine {
    pub fn Update(
        document: &mut Document,
        environment: &StyleEnvironment,
        user_agent: &[CSSStyleSheet],
    ) {
        use crate::style_state::{StyleChange, StyleUpdateImpact, StyleUpdateStats};
        use std::sync::Arc;
        let mut trace = browser_tracing::span("style", "StyleEngine.Update");
        // Diagnostic only: disabled runs retain the original style lifecycle.
        // Snapshot at most eight queued records; counts cover the whole queue.
        let invalidation_profile = std::env::var_os("BROWSER_PROFILE_STYLE_INVALIDATION")
            .is_some()
            .then(std::time::Instant::now);
        let dirty_before = document.StyleState().all_dirty;
        let queued = document.StyleState().changes.len();
        let profile_changes = invalidation_profile.map(|_| {
            document
                .StyleState()
                .changes
                .iter()
                .take(8)
                .map(|&change| {
                    let index = match change {
                        crate::style_state::StyleChange::Node(index)
                        | crate::style_state::StyleChange::Attribute(index, _)
                        | crate::style_state::StyleChange::InlineStyle(index)
                        | crate::style_state::StyleChange::Children(index)
                        | crate::style_state::StyleChange::Animation(index)
                        | crate::style_state::StyleChange::Resource(index) => index,
                    };
                    (
                        change,
                        document.Node(index).Id(),
                        document.Node(index).Name().to_owned(),
                    )
                })
                .collect::<Vec<_>>()
        });
        let key = (
            environment.media_type as u8,
            environment.viewport_width,
            environment.viewport_height,
            environment.resolution_dppx,
            environment.preferred_color_scheme as u8,
        );
        let ua_changed = document.StyleState().rules.user_agent.is_none()
            || document.StyleState().rules.user_agent_source != user_agent;
        let author_changed = document.StyleState().rules.author_revision
            != Some(document.StyleState().sheet_revision);
        let environment_changed = document.StyleState().environment != Some(key);
        trace.set("dirty_before", dirty_before as u8 as f64);
        trace.set("queued_changes", queued as f64);
        trace.set("ua_changed", ua_changed as u8 as f64);
        trace.set("author_changed", author_changed as u8 as f64);
        trace.set("environment_changed", environment_changed as u8 as f64);
        // Cache refreshes must not consume invalidation if calculation fails.
        if ua_changed || author_changed || environment_changed {
            document.StyleStateMut().all_dirty = true;
        }
        let mut stats = StyleUpdateStats::default();
        if ua_changed {
            let _rules = browser_tracing::span("style", "StyleEngine.BuildUserAgentRules");
            let rules =
                Arc::new(crate::style_resolver::style_rule_index::BuildCompiledRuleSet(user_agent));
            let cache = &mut document.StyleStateMut().rules;
            cache.user_agent = Some(rules);
            cache.user_agent_source = user_agent.to_vec();
            stats.rule_sets_built += 1;
        }
        if author_changed {
            let _rules = browser_tracing::span("style", "StyleEngine.BuildAuthorRules");
            let rules = Arc::new(
                crate::style_resolver::style_rule_index::BuildCompiledRuleSet(
                    document.ActiveStyleSheets(),
                ),
            );
            let revision = document.StyleState().sheet_revision;
            let cache = &mut document.StyleStateMut().rules;
            cache.author = Some(rules);
            cache.author_revision = Some(revision);
            stats.rule_sets_built += 1;
        }
        if ua_changed || author_changed {
            document.StyleState().rules.selectors.borrow_mut().clear();
        }
        if environment_changed {
            let state = document.StyleStateMut();
            state.measurement_geometry_revision = state
                .measurement_geometry_revision
                .checked_add(1)
                .expect("measurement geometry revision overflow");
            document
                .StyleStateMut()
                .impact
                .Merge(StyleUpdateImpact::LAYOUT);
        }
        let all =
            document.StyleState().all_dirty || ua_changed || author_changed || environment_changed;
        trace.set("all_dirty", all as u8 as f64);
        if !all && document.StyleState().changes.is_empty() {
            let state = document.StyleStateMut();
            state.stats = stats;
            state.sibling_sensitive_nodes.clear();
            state.descendant_sensitive_nodes.clear();
            state.append_only_children.clear();
            state.non_append_children.clear();
            state.inserted_style_subtrees.clear();
            state.append_affected_subtrees.clear();
            trace.set("no_work", 1.0);
            if let Some(started) = invalidation_profile {
                eprintln!("style-invalidation-profile root_id={} all_dirty_before={} all={} ua_changed={} author_changed={} environment_changed={} queued={} no_work=true resolved_nodes=0 changed_nodes=0 rule_sets_built={} total_ms={:.3}",
                    document.Node(document.Root()).Id(), dirty_before, all,
                    ua_changed, author_changed, environment_changed, queued,
                    stats.rule_sets_built, started.elapsed().as_secs_f64()*1000.0);
            }
            return;
        }
        let mut invalidation = StyleInvalidation::default();
        for &index in &document.StyleState().inserted_style_subtrees {
            // A later mutation in the same batch may have detached an earlier
            // append target. Detached nodes are not reachable from the document
            // and therefore need no style work in this update.
            if document.Node(index).Parent().is_none() || HasInsertedParent(document, index) {
                continue;
            }
            invalidation.subtrees.insert(index);
            AddInvalidationPath(document, &mut invalidation, index);
        }
        for &index in &document.StyleState().append_affected_subtrees {
            if document.Node(index).Parent().is_none() {
                continue;
            }
            AddLastChildInvalidations(document, index, &mut invalidation);
        }
        let sibling = [
            &document.StyleState().rules.user_agent,
            &document.StyleState().rules.author,
        ]
        .iter()
        .any(|r| r.as_ref().is_some_and(|r| r.depends_on_siblings));
        let relational = [
            &document.StyleState().rules.user_agent,
            &document.StyleState().rules.author,
        ]
        .iter()
        .any(|r| r.as_ref().is_some_and(|r| r.depends_on_descendants));
        let has_dependency_fallback = [
            &document.StyleState().rules.user_agent,
            &document.StyleState().rules.author,
        ]
        .iter()
        .any(|r| r.as_ref().is_some_and(|r| r.has_dependency_fallback));
        let inline_style_selector_dependency = [
            &document.StyleState().rules.user_agent,
            &document.StyleState().rules.author,
        ]
        .iter()
        .any(|r| {
            r.as_ref()
                .is_some_and(|r| r.inline_style_may_affect_selectors)
        });
        trace.set("sibling_dependencies", sibling as u8 as f64);
        trace.set("relational_dependencies", relational as u8 as f64);
        trace.set(
            "has_dependency_fallback",
            has_dependency_fallback as u8 as f64,
        );
        trace.set(
            "inline_style_selector_dependency",
            inline_style_selector_dependency as u8 as f64,
        );
        let mut profile_kind_counts = [0usize; 6];
        let mut profile_attribute_skips = 0usize;
        let mut profile_attribute_local = 0usize;
        let mut profile_sibling_promotions = 0usize;
        let mut profile_root_promotions = 0usize;
        let mut invalidation_trace =
            browser_tracing::span("style", "StyleEngine.BuildInvalidation");
        for change in &document.StyleState().changes {
            let kind = match change {
                StyleChange::Node(_) => 0,
                StyleChange::Children(_) => 1,
                StyleChange::Animation(_) => 2,
                StyleChange::Resource(_) => 3,
                StyleChange::InlineStyle(_) => 4,
                StyleChange::Attribute(_, _) => 5,
            };
            profile_kind_counts[kind] += 1;
            let change_index = match *change {
                StyleChange::Node(index)
                | StyleChange::Attribute(index, _)
                | StyleChange::InlineStyle(index)
                | StyleChange::Children(index)
                | StyleChange::Animation(index)
                | StyleChange::Resource(index) => index,
            };
            // Mutations made while assembling a newly inserted subtree cannot
            // invalidate stale style inside it: it has no prior style. The
            // outer inserted root is already a subtree invalidation seed.
            if HasInsertedAncestor(document, change_index) {
                continue;
            }
            let (mut index, subtree) = match *change {
                StyleChange::Node(index)
                    if document
                        .StyleState()
                        .descendant_sensitive_nodes
                        .contains(&index) =>
                {
                    (index, true)
                }
                StyleChange::Node(index) => {
                    invalidation.nodes.insert(index);
                    AddInvalidationPath(document, &mut invalidation, index);
                    continue;
                }
                StyleChange::Attribute(index, kind)
                    if AttributeHasNoStyleReader(document, index, kind) =>
                {
                    if invalidation_profile.is_some() {
                        profile_attribute_skips += 1;
                    }
                    continue;
                }
                StyleChange::Attribute(index, kind)
                    if AttributeHasNoSelectorReader(document, kind) =>
                {
                    // attr()/var() may read the changed attribute, but no
                    // selector can observe it. Recompute this node, including
                    // a newly inserted script with no old style, then let exact
                    // computed changes propagate through the existing path.
                    if invalidation_profile.is_some() {
                        profile_attribute_local += 1;
                    }
                    invalidation.nodes.insert(index);
                    let mut ancestor = Some(index);
                    while let Some(node) = ancestor {
                        invalidation.paths.insert(node);
                        ancestor = document.Node(node).Parent();
                    }
                    continue;
                }
                StyleChange::Attribute(index, _)
                    if document
                        .StyleState()
                        .descendant_sensitive_nodes
                        .contains(&index) =>
                {
                    (index, true)
                }
                StyleChange::Attribute(index, _) => {
                    invalidation.nodes.insert(index);
                    AddInvalidationPath(document, &mut invalidation, index);
                    continue;
                }
                StyleChange::InlineStyle(index) if !inline_style_selector_dependency => {
                    // No selector can observe this attribute, including nested
                    // :has/:is/:not. Resolve this element's declarations; the
                    // existing descendants_changed path propagates inheritance,
                    // custom properties, display and other computed changes.
                    invalidation.nodes.insert(index);
                    let mut ancestor = Some(index);
                    while let Some(node) = ancestor {
                        invalidation.paths.insert(node);
                        ancestor = document.Node(node).Parent();
                    }
                    continue;
                }
                StyleChange::InlineStyle(index)
                    if document
                        .StyleState()
                        .descendant_sensitive_nodes
                        .contains(&index) =>
                {
                    (index, true)
                }
                StyleChange::InlineStyle(index) => {
                    invalidation.nodes.insert(index);
                    AddInvalidationPath(document, &mut invalidation, index);
                    continue;
                }
                StyleChange::Children(index) => (index, true),
                StyleChange::Animation(index) => (index, false),
                StyleChange::Resource(index) => {
                    invalidation.resources.insert(index);
                    let mut ancestor = Some(index);
                    while let Some(node) = ancestor {
                        invalidation.paths.insert(node);
                        ancestor = document.Node(node).Parent();
                    }
                    continue;
                }
            };
            let changed_index = index;
            let append_scoped = matches!(*change, StyleChange::Children(_))
                && document.StyleState().append_only_children.contains(&index)
                && !document.StyleState().non_append_children.contains(&index)
                && ChildrenCanKeepContainerScope(document, index);
            if append_scoped && AppendNeedsNthLastInvalidation(document, index) {
                AddNthLastAppendInvalidations(document, index, &mut invalidation);
            }
            let keep_container_scope = matches!(*change, StyleChange::Children(_))
                && ChildrenCanKeepContainerScope(document, index);
            let affects_siblings = match *change {
                StyleChange::Children(_) => !keep_container_scope,
                StyleChange::Resource(_) | StyleChange::Animation(_) => false,
                _ => document
                    .StyleState()
                    .sibling_sensitive_nodes
                    .contains(&changed_index),
            };
            if subtree && sibling && affects_siblings {
                let parent = document.Node(index).Parent().unwrap_or(index);
                if parent != index {
                    profile_sibling_promotions += 1;
                }
                index = parent;
            }
            if relational && subtree {
                if has_dependency_fallback {
                    // Unknown/nonlocal grammar retains the original full fallback.
                    if index != document.Root() {
                        profile_root_promotions += 1;
                    }
                    index = document.Root();
                } else {
                    // Keep the ordinary Node/Children sibling-parent subtree.
                    // For proven first-compound descendant :has, only an ancestor
                    // host can observe this mutation. Recalculate its whole subtree
                    // to preserve descendant targets, pseudo cascade and specificity.
                    // Changed/lost host class membership is covered by the ordinary
                    // subtree. DOM moves queue both old and new parent Children.
                    let mut ancestor = Some(changed_index);
                    while let Some(node) = ancestor {
                        let is_host = [
                            &document.StyleState().rules.user_agent,
                            &document.StyleState().rules.author,
                        ]
                        .iter()
                        .filter_map(|r| r.as_ref())
                        .any(|r| {
                            r.bounded_has_hosts.iter().any(|host| {
                                crate::style_resolver::persistent_selector::MatchesSelector(
                                    document, node, host,
                                )
                            })
                        });
                        if is_host {
                            invalidation.subtrees.insert(node);
                            let mut path = Some(node);
                            while let Some(node) = path {
                                invalidation.paths.insert(node);
                                path = document.Node(node).Parent();
                            }
                        }
                        ancestor = document.Node(node).Parent();
                    }
                }
            }
            if subtree && !append_scoped {
                invalidation.subtrees.insert(index);
            } else if !subtree {
                invalidation.nodes.insert(index);
            }
            let mut ancestor = Some(index);
            while let Some(node) = ancestor {
                invalidation.paths.insert(node);
                ancestor = document.Node(node).Parent();
            }
        }
        invalidation_trace.set(
            "root_subtree",
            invalidation.subtrees.contains(&document.Root()) as u8 as f64,
        );
        invalidation_trace.set("invalidated_subtrees", invalidation.subtrees.len() as f64);
        invalidation_trace.set("invalidated_nodes", invalidation.nodes.len() as f64);
        invalidation_trace.set("invalidated_resources", invalidation.resources.len() as f64);
        invalidation_trace.set("sibling_promotions", profile_sibling_promotions as f64);
        invalidation_trace.set("root_promotions", profile_root_promotions as f64);
        for (name, count) in [
            ("node_changes", profile_kind_counts[0]),
            ("children_changes", profile_kind_counts[1]),
        ] {
            invalidation_trace.set(name, count as f64);
        }
        drop(invalidation_trace);
        trace.set(
            "root_subtree",
            invalidation.subtrees.contains(&document.Root()) as u8 as f64,
        );
        trace.set("invalidated_subtrees", invalidation.subtrees.len() as f64);
        trace.set("invalidated_nodes", invalidation.nodes.len() as f64);
        trace.set("invalidated_resources", invalidation.resources.len() as f64);
        let mut pending = Vec::new();
        {
            let mut recalc = browser_tracing::span("style", "StyleEngine.RecalcStyles");
            let context = StyleCascadeContext::new(document, environment, user_agent);
            let force = all || invalidation.subtrees.contains(&document.Root());
            for &child in document.Node(document.Root()).Children() {
                if document.Node(child).Type() == DOMNodeType::kElement {
                    ResolveElement(
                        &context,
                        document,
                        child,
                        None,
                        None,
                        environment,
                        true,
                        false,
                        false,
                        &mut pending,
                        &invalidation,
                        force,
                        false,
                    );
                }
            }
            recalc.set("resolved_nodes", pending.len() as f64);
        }
        stats.resolved_nodes = pending.len();
        {
            let mut commit = browser_tracing::span("style", "StyleEngine.CommitStyles");
            for (node, style) in pending {
                let old = document.ResolvedStyleFor(node);
                if old == Some(&style) {
                    continue;
                }
                stats.changed_nodes += 1;
                let reattach = old.is_none_or(|old| {
                    old.generates_box != style.generates_box
                        || old.display_contents != style.display_contents
                        || old.style.display != style.style.display
                        || old.before != style.before
                        || old.after != style.after
                        || old.first_letter != style.first_letter
                        || old.placeholder != style.placeholder
                });
                let layout =
                    reattach || old.is_none_or(|old| !old.style.LayoutEquivalent(&style.style));
                let geometry_changed = old.is_none_or(|old| {
                    old.own_generates_box != style.own_generates_box
                        || old.own_display_contents != style.own_display_contents
                        || (layout && !MeasurementGeometryEquivalent(old, &style))
                });
                if geometry_changed {
                    let state = document.StyleStateMut();
                    state.measurement_geometry_revision = state
                        .measurement_geometry_revision
                        .checked_add(1)
                        .expect("measurement geometry revision overflow");
                }
                document.StyleStateMut().impact.Merge(StyleUpdateImpact {
                    reattach,
                    layout,
                    paint: true,
                });
                document.SetResolvedStyle(node, style);
            }
            commit.set("resolved_nodes", stats.resolved_nodes as f64);
            commit.set("changed_nodes", stats.changed_nodes as f64);
        }
        // Commit dirty state only after successful calculation. A panicking
        // declaration leaves changes queued so the next update can retry.
        let state = document.StyleStateMut();
        state.environment = Some(key);
        state.all_dirty = false;
        state.changes.clear();
        state.sibling_sensitive_nodes.clear();
        state.descendant_sensitive_nodes.clear();
        state.append_only_children.clear();
        state.non_append_children.clear();
        state.inserted_style_subtrees.clear();
        state.append_affected_subtrees.clear();
        state.stats = stats;
        trace.set("resolved_nodes", stats.resolved_nodes as f64);
        trace.set("changed_nodes", stats.changed_nodes as f64);
        trace.set("rule_sets_built", stats.rule_sets_built as f64);
        if let Some(started) = invalidation_profile {
            // Capture the complete update before diagnostic formatting/output.
            let elapsed = started.elapsed();
            eprintln!("style-invalidation-profile root_id={} all_dirty_before={} all={} ua_changed={} author_changed={} environment_changed={} queued={} node_changes={} children_changes={} animation_changes={} resource_changes={} inline_style_changes={} attribute_changes={} attribute_skips={} attribute_local={} inline_style_selector_dependency={} sibling_dependencies={} relational_dependencies={} sibling_promotions={} root_promotions={} root_subtree={} subtrees={} nodes={} resources={} resolved_nodes={} changed_nodes={} rule_sets_built={} queued_sample={:?} total_ms={:.3}",
                document.Node(document.Root()).Id(), dirty_before, all,
                ua_changed, author_changed, environment_changed, queued,
                profile_kind_counts[0], profile_kind_counts[1], profile_kind_counts[2], profile_kind_counts[3],
                profile_kind_counts[4], profile_kind_counts[5], profile_attribute_skips, profile_attribute_local, inline_style_selector_dependency,
                sibling, relational, profile_sibling_promotions, profile_root_promotions,
                invalidation.subtrees.contains(&document.Root()), invalidation.subtrees.len(),
                invalidation.nodes.len(), invalidation.resources.len(),
                stats.resolved_nodes, stats.changed_nodes, stats.rule_sets_built,
                profile_changes.as_deref().unwrap_or(&[]), elapsed.as_secs_f64()*1000.0);
        }
    }
}

// cpp: style_resolver/style_resolver.cc:8087-8112
pub fn ResolveComputedStyles(
    owner: &mut DOM,
    environment: &StyleEnvironment,
    user_agent: &[CSSStyleSheet],
) {
    ResolveDocumentStyles(owner.GetDocumentMut(), environment, user_agent);
}
// cpp: style_resolver/style_resolver.cc:8100-8112
// Resolve from immutable references into the existing arena, then commit style
// records after the borrow ends. The source never reads resolved records during
// the pass, so staging records does not alter selector or cascade behavior.
pub fn ResolveDocumentStyles(
    document: &mut Document,
    environment: &StyleEnvironment,
    user_agent: &[CSSStyleSheet],
) {
    StyleEngine::Update(document, environment, user_agent);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistent_document::DOMAttribute;
    use std::collections::BTreeMap;
    #[derive(Debug, PartialEq)]
    enum Token {
        Number(f64),
        Text(String),
        None,
    }
    #[derive(Default)]
    struct Dump(Vec<Token>);
    impl Dump {
        fn n(&mut self, value: f64) {
            self.0.push(Token::Number(value));
        }
        fn b(&mut self, value: bool) {
            self.n(value as u8 as f64);
        }
        fn optional(&mut self, value: Option<f64>) {
            self.0.push(value.map_or(Token::None, Token::Number));
        }
        fn text(&mut self, value: &str) {
            self.0.push(Token::Text(value.into()));
        }
        fn color(&mut self, value: layoutng_assembly::internal::layout_input_types::Color) {
            for v in [value.red, value.green, value.blue, value.alpha] {
                self.n(v as f64);
            }
        }
        fn style(&mut self, resolved: Option<&ResolvedNodeStyle>) {
            self.b(resolved.is_some());
            let Some(r) = resolved else {
                return;
            };
            let s = &r.style;
            let e = s.extended.as_ref().unwrap();
            self.b(r.generates_box);
            self.b(r.display_contents);
            for v in [s.display as i32, s.direction as i32, s.writing_mode as i32] {
                self.n(v as f64);
            }
            self.n(e.font_size);
            self.n(e.font_weight);
            self.b(e.font_italic);
            for v in [
                e.line_height,
                e.line_height_percent,
                s.width,
                s.height,
                e.width_percent,
                e.height_percent,
            ] {
                self.optional(v);
            }
            self.b(e.width_calculated);
            self.b(e.height_calculated);
            for edges in [s.margin, s.padding] {
                for v in [edges.top, edges.right, edges.bottom, edges.left] {
                    self.n(v);
                }
            }
            self.n(e.math_depth as f64);
            self.n(e.math_style as i32 as f64);
            self.b(e.font_size_math);
            for v in [
                e.math_baseline,
                e.math_padded_depth,
                e.math_lspace,
                e.math_rspace,
                e.math_min_size,
                e.math_max_size,
                e.math_padded_voffset,
                e.math_fraction_bar_thickness,
            ] {
                self.optional(v);
            }
            for v in [
                e.text_align as i32,
                e.overflow_x as i32,
                e.overflow_y as i32,
            ] {
                self.n(v as f64);
            }
            self.n(s.flex_grow as f64);
            self.n(s.flex_shrink as f64);
            self.optional(s.flex_basis);
            self.optional(e.flex_basis_percent);
            self.b(e.flex_basis_calculated);
            self.n(e.flex_basis_sizing as i32 as f64);
            self.optional(e.justify_items.map(|v| v as i32 as f64));
            self.optional(e.justify_self.map(|v| v as i32 as f64));
            self.b(e.justify_items_legacy);
            self.n(e.justify_self_overflow as i32 as f64);
            for v in [
                e.has_author_background,
                e.has_author_border,
                e.has_author_border_radius,
                e.has_author_outline,
            ] {
                self.b(v);
            }
            self.color(s.paint.color);
            self.color(s.paint.background_color);
            for v in [s.paint.svg_fill, s.paint.svg_stroke] {
                self.b(v.is_some());
                if let Some(v) = v {
                    self.color(v);
                }
            }
            self.n(e.initial_letter.size);
            self.n(e.initial_letter.sink as f64);
            self.n(e.initial_letter.sink_type as i32 as f64);
            self.n(e.font_families.len() as f64);
            for f in &e.font_families {
                self.text(f);
            }
            self.text(&e.language);
            let mut names: Vec<_> = r.custom_properties.keys().collect();
            names.sort();
            self.n(names.len() as f64);
            for name in names {
                self.text(name);
                let value = &r.custom_properties[name];
                self.b(value.is_some());
                if let Some(value) = value {
                    self.text(value);
                }
            }
            for p in [&r.before, &r.after, &r.first_letter, &r.placeholder] {
                self.b(p.is_some());
                if let Some(p) = p {
                    let e = p.style.extended.as_ref().unwrap();
                    self.text(&p.text);
                    self.b(p.display_contents);
                    self.n(p.style.display as i32 as f64);
                    self.n(e.font_size);
                    self.n(e.font_weight);
                    self.b(e.font_italic);
                    for v in [e.line_height, e.line_height_percent, p.style.width] {
                        self.optional(v);
                    }
                    self.n(e.initial_letter.size);
                    self.n(e.initial_letter.sink as f64);
                    self.n(e.initial_letter.sink_type as i32 as f64);
                    self.color(p.style.paint.color);
                    for v in [
                        p.style.margin.top,
                        p.style.margin.right,
                        p.style.margin.bottom,
                        p.style.margin.left,
                    ] {
                        self.n(v);
                    }
                }
            }
        }
    }
    fn decode(s: &str) -> String {
        String::from_utf8(
            (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect(),
        )
        .unwrap()
    }

    fn find_id(document: &Document, index: usize, id: &str) -> Option<usize> {
        if document
            .Node(index)
            .FindAttribute("id")
            .is_some_and(|attribute| attribute.value == id)
        {
            return Some(index);
        }
        document
            .Node(index)
            .Children()
            .iter()
            .find_map(|&child| find_id(document, child, id))
    }

    #[test]
    fn child_list_invalidation_stays_at_container_for_unrelated_sibling_rules() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<main><div id='container'><span></span></div><p></p></main>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS(".unrelated + .sibling { color:red }"));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        let container = find_id(document, document.Root(), "container").unwrap();
        let child = document.CreateElementDefault(DOMNamespace::kHTML, "span".into());
        document.AppendChild(container, child);
        ResolveDocumentStyles(document, &environment, &[]);
        // No rule observes last/only state, so only the fresh child resolves.
        assert_eq!(document.StyleState().stats.resolved_nodes, 1);
    }

    #[test]
    fn append_updates_previous_last_child_without_revisiting_older_siblings() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<main id='container'><div id='untouched'><i></i></div>\
             <div id='old' class='item'><i id='old-desc' class='desc'></i></div></main>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS(".item:last-child .desc { color:red }"));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        let container = find_id(document, document.Root(), "container").unwrap();
        let old_desc = find_id(document, document.Root(), "old-desc").unwrap();
        let before = document
            .ResolvedStyleFor(old_desc)
            .unwrap()
            .style
            .paint
            .color;
        let child = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        document.SetAttribute(
            child,
            DOMAttribute {
                local_name: "class".into(),
                value: "item".into(),
                ..Default::default()
            },
        );
        let desc = document.CreateElementDefault(DOMNamespace::kHTML, "i".into());
        document.SetAttribute(
            desc,
            DOMAttribute {
                local_name: "class".into(),
                value: "desc".into(),
                ..Default::default()
            },
        );
        document.AppendChild(child, desc);
        document.AppendChild(container, child);
        ResolveDocumentStyles(document, &environment, &[]);
        assert_ne!(
            document
                .ResolvedStyleFor(old_desc)
                .unwrap()
                .style
                .paint
                .color,
            before
        );
        assert!(document.ResolvedStyleFor(child).is_some());
        assert!(document.ResolvedStyleFor(desc).is_some());
        assert!(document.StyleState().stats.resolved_nodes < 6);
    }

    #[test]
    fn nth_last_child_append_retains_container_subtree_invalidation() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<main id='container'><i></i><i></i><i></i></main>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS("i:nth-last-child(odd) { color:red }"));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        let container = find_id(document, document.Root(), "container").unwrap();
        let child = document.CreateElementDefault(DOMNamespace::kHTML, "i".into());
        document.AppendChild(container, child);
        ResolveDocumentStyles(document, &environment, &[]);
        // The four children are target candidates; the unchanged container is
        // only a traversal path, matching Blink's NthSiblingInvalidationSet.
        assert_eq!(document.StyleState().stats.resolved_nodes, 4);
    }

    #[test]
    fn unrelated_nth_last_rule_does_not_disable_scoped_append() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<main id='container'><span></span><span></span><span></span></main>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS("li.hot:nth-last-child(odd) { color:red }"));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        let container = find_id(document, document.Root(), "container").unwrap();
        let child = document.CreateElementDefault(DOMNamespace::kHTML, "span".into());
        document.AppendChild(container, child);
        ResolveDocumentStyles(document, &environment, &[]);
        assert_eq!(document.StyleState().stats.resolved_nodes, 2);
    }

    #[test]
    fn child_state_feeding_sibling_selector_still_widens_to_parent() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<main><div id='container'></div><p id='outside'></p></main>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS("#container:empty + #outside { color:red }"));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        let container = find_id(document, document.Root(), "container").unwrap();
        assert!(!ChildrenCanKeepContainerScope(document, container));
        let child = document.CreateElementDefault(DOMNamespace::kHTML, "span".into());
        document.AppendChild(container, child);
        ResolveDocumentStyles(document, &environment, &[]);
        assert!(document.StyleState().stats.resolved_nodes > 2);
    }

    #[test]
    fn unrelated_attribute_change_does_not_widen_for_sibling_rules() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<main><div><span id='changed' class='other'></span><i></i></div></main>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS(".trigger + .target { color:red }"));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        let changed = find_id(document, document.Root(), "changed").unwrap();
        document.SetAttribute(
            changed,
            DOMAttribute {
                local_name: "class".into(),
                value: "still-unrelated".into(),
                ..Default::default()
            },
        );
        ResolveDocumentStyles(document, &environment, &[]);
        assert_eq!(document.StyleState().stats.resolved_nodes, 1);
    }

    #[test]
    fn unrelated_attribute_change_does_not_recalculate_descendants() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<main id='changed' class='other'><span><i></i></span></main>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS(".trigger .target { color:red }"));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        let changed = find_id(document, document.Root(), "changed").unwrap();
        document.SetAttribute(
            changed,
            DOMAttribute {
                local_name: "class".into(),
                value: "still-unrelated".into(),
                ..Default::default()
            },
        );
        ResolveDocumentStyles(document, &environment, &[]);
        assert_eq!(document.StyleState().stats.resolved_nodes, 1);
    }

    #[test]
    fn ancestor_selector_change_recalculates_descendant_targets() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<main id='changed' class='trigger'><span id='target' class='target'></span></main>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS(".trigger .target { color:red }"));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        let changed = find_id(document, document.Root(), "changed").unwrap();
        let target = find_id(document, document.Root(), "target").unwrap();
        let before = document.ResolvedStyleFor(target).unwrap().style.paint.color;
        document.RemoveAttributeDefault(changed, "class");
        ResolveDocumentStyles(document, &environment, &[]);
        assert_ne!(
            document.ResolvedStyleFor(target).unwrap().style.paint.color,
            before
        );
        assert!(document.StyleState().stats.resolved_nodes >= 2);
    }

    #[test]
    fn removing_sibling_trigger_uses_old_state_to_widen() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<main><div><span id='trigger' class='trigger'></span>\
             <span id='target' class='target'></span></div></main>",
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(cssom::ParseCSS(".trigger + .target { color:red }"));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        let trigger = find_id(document, document.Root(), "trigger").unwrap();
        let target = find_id(document, document.Root(), "target").unwrap();
        let before = document.ResolvedStyleFor(target).unwrap().style.paint.color;
        document.RemoveAttributeDefault(trigger, "class");
        ResolveDocumentStyles(document, &environment, &[]);
        assert_ne!(
            document.ResolvedStyleFor(target).unwrap().style.paint.color,
            before
        );
        assert!(document.StyleState().stats.resolved_nodes >= 3);
    }

    #[test]
    fn non_inherited_parent_change_respects_explicit_inheritance() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<div id='plain' style='opacity:0.8;transform:translateX(1px)'>\
             <span id='plain-child'>plain</span></div>\
             <div id='inherited' style='opacity:0.8'>\
             <span id='explicit'>explicit</span><span id='variable'>variable</span>\
             <span id='all'>all</span></div>",
        );
        owner.GetDocumentMut().AppendStyleSheet(cssom::ParseCSS(
            "#explicit {opacity:inherit} #variable {opacity:var(--absent, inherit)} \
             #all {all:inherit}",
        ));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        fn find(document: &Document, index: usize, id: &str) -> Option<usize> {
            if document
                .Node(index)
                .FindAttribute("id")
                .is_some_and(|attribute| attribute.value == id)
            {
                return Some(index);
            }
            document
                .Node(index)
                .Children()
                .iter()
                .find_map(|&child| find(document, child, id))
        }
        let plain = find(document, document.Root(), "plain").unwrap();
        let child = find(document, document.Root(), "plain-child").unwrap();
        let child_style = document.ResolvedStyleFor(child).unwrap().clone();
        assert_eq!(child_style.has_explicit_inheritance, Some(false));
        document.StyleStateMut().impact = Default::default();
        document.SetAttribute(
            plain,
            DOMAttribute {
                local_name: "style".into(),
                value: "opacity:0.4;transform:translateX(2px)".into(),
                ..Default::default()
            },
        );
        ResolveDocumentStyles(document, &environment, &[]);
        assert_eq!(document.StyleState().stats.resolved_nodes, 1);
        assert!(document.ResolvedStyleFor(child) == Some(&child_style));
        assert!(document.StyleState().impact.layout);
        assert!(document.StyleState().impact.paint);

        let inherited = find(document, document.Root(), "inherited").unwrap();
        let dependent =
            ["explicit", "variable", "all"].map(|id| find(document, document.Root(), id).unwrap());
        for &index in &dependent {
            let style = document.ResolvedStyleFor(index).unwrap();
            assert_eq!(style.has_explicit_inheritance, Some(true));
            assert_eq!(style.style.paint.opacity, 0.8);
        }
        document.SetAttribute(
            inherited,
            DOMAttribute {
                local_name: "style".into(),
                value: "opacity:0.4".into(),
                ..Default::default()
            },
        );
        ResolveDocumentStyles(document, &environment, &[]);
        assert_eq!(document.StyleState().stats.resolved_nodes, 4);
        for &index in &dependent {
            assert_eq!(
                document
                    .ResolvedStyleFor(index)
                    .unwrap()
                    .style
                    .paint
                    .opacity,
                0.4
            );
        }

        // An external/old cached style cannot authorize the non-inherited path.
        let mut unknown = child_style;
        unknown.has_explicit_inheritance = None;
        document.SetResolvedStyle(child, unknown);
        document.SetAttribute(
            plain,
            DOMAttribute {
                local_name: "style".into(),
                value: "opacity:0.2;transform:translateX(2px)".into(),
                ..Default::default()
            },
        );
        ResolveDocumentStyles(document, &environment, &[]);
        assert_eq!(document.StyleState().stats.resolved_nodes, 2);
        assert_eq!(
            document
                .ResolvedStyleFor(child)
                .unwrap()
                .has_explicit_inheritance,
            Some(false)
        );
    }

    #[test]
    fn pseudo_only_change_keeps_child_style_and_layout_notification() {
        let mut owner = crate::test_html::html_parser::ParseHTML(
            "<div id='owner' data-pcr='before'><span id='child'>text</span></div>",
        );
        owner.GetDocumentMut().AppendStyleSheet(cssom::ParseCSS(
            "#owner::before {content:attr(data-pcr)} #owner::first-letter {color:red}",
        ));
        let environment = StyleEnvironment::default();
        ResolveComputedStyles(&mut owner, &environment, &[]);
        let document = owner.GetDocumentMut();
        fn find(document: &Document, index: usize, id: &str) -> Option<usize> {
            if document
                .Node(index)
                .FindAttribute("id")
                .is_some_and(|attribute| attribute.value == id)
            {
                return Some(index);
            }
            document
                .Node(index)
                .Children()
                .iter()
                .find_map(|&child| find(document, child, id))
        }
        let parent = find(document, document.Root(), "owner").unwrap();
        let child = find(document, document.Root(), "child").unwrap();
        let previous = document.ResolvedStyleFor(parent).unwrap().clone();
        let child_style = document.ResolvedStyleFor(child).unwrap().clone();
        document.StyleStateMut().impact = Default::default();
        document.SetAttribute(
            parent,
            DOMAttribute {
                local_name: "data-pcr".into(),
                value: "after".into(),
                ..Default::default()
            },
        );
        ResolveDocumentStyles(document, &environment, &[]);
        let updated = document.ResolvedStyleFor(parent).unwrap();
        assert!(previous.before != updated.before);
        assert!(SameDescendantStyleInputs(&previous, updated));
        assert!(document.ResolvedStyleFor(child) == Some(&child_style));
        assert_eq!(document.StyleState().stats.resolved_nodes, 1);
        assert_eq!(document.StyleState().stats.changed_nodes, 1);
        assert!(document.StyleState().impact.reattach);
        assert!(document.StyleState().impact.layout);
        assert!(document.StyleState().impact.paint);
    }
    #[test]
    fn persistent_full_cascade_pseudos_math_and_live_mutations_match_cpp() {
        let mut owner = crate::test_html::html_parser::ParseHTML(include_str!(
            "../../../../artifacts/cpp-reference/persistent-style.html"
        ));
        crate::style_resolver::AddStyleSheet(
            &mut owner,
            cssom::ParseCSS(include_str!(
                "../../../../artifacts/cpp-reference/persistent-style.css"
            )),
        );
        let ua = [cssom::ParseCSS(include_str!(
            "../../../../artifacts/cpp-reference/persistent-style-ua.css"
        ))];
        let environment = StyleEnvironment {
            viewport_width: Some(1024.0),
            viewport_height: Some(768.0),
            resolution_dppx: Some(1.0),
            ..Default::default()
        };
        fn gather(d: &Document, i: usize, nodes: &mut BTreeMap<String, usize>) {
            if let Some(id) = d.Node(i).FindAttribute("id") {
                nodes.insert(id.value.clone(), i);
            }
            for &c in d.Node(i).Children() {
                gather(d, c, nodes);
            }
        }
        let mut nodes = BTreeMap::new();
        gather(owner.GetDocument(), owner.GetDocument().Root(), &mut nodes);
        let identities: Vec<_> = nodes
            .values()
            .map(|&i| owner.GetDocument().Node(i).Id())
            .collect();
        for stage in 0..2 {
            if stage == 1 {
                let d = owner.GetDocumentMut();
                let target = nodes["target"];
                d.SetAttribute(
                    target,
                    DOMAttribute {
                        local_name: "class".into(),
                        value: "mutated".into(),
                        ..Default::default()
                    },
                );
                d.SetAttribute(target,DOMAttribute {local_name:"style".into(),value:"font-size:28px; width:var(--space); direction:ltr; flex:4 2 10px; justify-self:unsafe left".into(),..Default::default()});
                d.SetControlValue(nodes["input"], "typed".into());
                d.AppendChild(nodes["contents"], nodes["moved"]);
                d.SetAnimationStyle(
                    d.Node(target).Id(),
                    1,
                    cssom::ParseCSSDeclarationList("font-weight:800; height:12px"),
                );
            }
            ResolveComputedStyles(&mut owner, &environment, &ua);
            let d = owner.GetDocument();
            assert_eq!(
                identities,
                nodes.values().map(|&i| d.Node(i).Id()).collect::<Vec<_>>()
            );
            let mut count = 0;
            for line in
                include_str!("../../../../artifacts/cpp-reference/persistent-style-results.tsv")
                    .lines()
            {
                let (key, encoded) = line.split_once('\t').unwrap();
                let (expected_stage, id) = key.split_once(':').unwrap();
                if expected_stage.parse::<usize>().unwrap() != stage {
                    continue;
                }
                let mut dump = Dump::default();
                dump.style(d.ResolvedStyleFor(nodes[id]));
                let expected: Vec<Token> = encoded
                    .split(',')
                    .map(|s| {
                        if s == "none" {
                            Token::None
                        } else if let Some(s) = s.strip_prefix("n:") {
                            Token::Number(s.parse().unwrap())
                        } else {
                            Token::Text(decode(s.strip_prefix("s:").unwrap()))
                        }
                    })
                    .collect();
                assert_eq!(dump.0, expected, "stage {} node {}", stage, id);
                count += 1;
            }
            assert_eq!(count, nodes.len());
        }
    }
}
