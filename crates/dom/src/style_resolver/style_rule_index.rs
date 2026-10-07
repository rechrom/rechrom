#![allow(non_snake_case)]
use crate::persistent_document::DOMNode;
use crate::style_resolver::selector::{
    ExtractTerminalPseudo, ParseSelector, ParsedSelector, SplitSelectorList,
};
use cssom::{CSSStyleRule, CSSStyleSheet};
use std::collections::BTreeMap;

// cpp: style_resolver/style_resolver.cc:674-691
#[derive(Clone, Copy)]
pub(crate) struct IndexedStyleRule<'a> {
    pub rule: &'a CSSStyleRule,
    pub source_order: usize,
}
#[derive(Default)]
pub(crate) struct StyleRuleIndex<'a> {
    universal: Vec<IndexedStyleRule<'a>>,
    ids: BTreeMap<String, Vec<IndexedStyleRule<'a>>>,
    classes: BTreeMap<String, Vec<IndexedStyleRule<'a>>>,
    types: BTreeMap<String, Vec<IndexedStyleRule<'a>>>,
    pub declaration_count: usize,
}
#[derive(PartialEq)]
struct RuleIndexKey {
    kind: u8,
    value: String,
}
// cpp: style_resolver/style_resolver.cc:692-744
fn FastKeyForSelector(selector: &ParsedSelector) -> Option<RuleIndexKey> {
    if !selector.valid {
        return None;
    }
    let compound = selector.compounds.last()?.as_bytes();
    let mut type_name = None;
    let mut first_class = None;
    let mut first_id = None;
    if compound
        .first()
        .is_some_and(|b| !matches!(b, b'*' | b'.' | b'#' | b'[' | b':'))
    {
        let end = compound
            .iter()
            .position(|b| matches!(b, b'.' | b'#' | b'[' | b':'))
            .unwrap_or(compound.len());
        if end > 0 {
            type_name = Some(String::from_utf8_lossy(&compound[..end]).to_ascii_lowercase());
        }
    }
    let (mut square, mut round, mut quote, mut cursor) = (0u32, 0u32, 0u8, 0usize);
    while cursor < compound.len() {
        let character = compound[cursor];
        if quote != 0 {
            if character == b'\\' && cursor + 1 < compound.len() {
                cursor += 1;
            } else if character == quote {
                quote = 0;
            }
        } else if matches!(character, b'\'' | b'"') {
            quote = character;
        } else if character == b'[' {
            square += 1;
        } else if character == b']' {
            square = square.saturating_sub(1);
        } else if character == b'(' {
            round += 1;
        } else if character == b')' {
            round = round.saturating_sub(1);
        } else if square == 0 && round == 0 && matches!(character, b'.' | b'#') {
            let start = cursor + 1;
            let mut end = start;
            while end < compound.len()
                && (compound[end].is_ascii_alphanumeric() || matches!(compound[end], b'-' | b'_'))
            {
                end += 1;
            }
            if end != start {
                let name = String::from_utf8_lossy(&compound[start..end]).into_owned();
                if character == b'#' {
                    first_id = Some(name);
                } else if first_class.is_none() {
                    first_class = Some(name);
                }
                cursor = end - 1;
            }
        }
        cursor += 1;
    }
    first_id
        .map(|value| RuleIndexKey { kind: b'#', value })
        .or_else(|| first_class.map(|value| RuleIndexKey { kind: b'.', value }))
        .or_else(|| type_name.map(|value| RuleIndexKey { kind: b't', value }))
}
// cpp: style_resolver/style_resolver.cc:746-780
pub(crate) fn BuildStyleRuleIndex(sheets: &[CSSStyleSheet]) -> StyleRuleIndex<'_> {
    let mut result = StyleRuleIndex::default();
    for sheet in sheets {
        for rule in &sheet.rules {
            let indexed = IndexedStyleRule {
                rule,
                source_order: result.declaration_count,
            };
            result.declaration_count += rule.declarations.len();
            let mut keys = Vec::new();
            let mut universal = false;
            for item in SplitSelectorList(&rule.selector_text) {
                let mut selector = ParseSelector(&item);
                if !selector.valid {
                    continue;
                }
                ExtractTerminalPseudo(&mut selector);
                let Some(key) = FastKeyForSelector(&selector) else {
                    universal = true;
                    break;
                };
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
            if universal {
                result.universal.push(indexed);
                continue;
            }
            for key in keys {
                let bucket = match key.kind {
                    b'#' => &mut result.ids,
                    b'.' => &mut result.classes,
                    _ => &mut result.types,
                };
                bucket.entry(key.value).or_default().push(indexed);
            }
        }
    }
    result
}
// cpp: style_resolver/style_resolver.cc:782-804
pub(crate) fn CandidateRules<'a>(
    index: &StyleRuleIndex<'a>,
    node: &DOMNode,
) -> Vec<IndexedStyleRule<'a>> {
    candidates(
        index,
        node.FindAttribute("id").map(|a| a.value.as_str()),
        node.FindAttribute("class").map(|a| a.value.as_str()),
        node.Name(),
    )
}

pub(crate) fn StaticCandidateRules<'a>(
    index: &StyleRuleIndex<'a>,
    node: &crate::Element,
) -> Vec<IndexedStyleRule<'a>> {
    let classes = node.classes.join(" ");
    candidates(index, node.id.as_deref(), Some(&classes), &node.tag)
}

fn candidates<'a>(
    index: &StyleRuleIndex<'a>,
    id: Option<&str>,
    classes: Option<&str>,
    name: &str,
) -> Vec<IndexedStyleRule<'a>> {
    let mut result = index.universal.clone();
    let mut append = |map: &BTreeMap<String, Vec<IndexedStyleRule<'a>>>, key: &str| {
        if let Some(bucket) = map.get(key) {
            result.extend(bucket.iter().copied());
        }
    };
    if let Some(id) = id {
        append(&index.ids, id);
    }
    if let Some(classes) = classes {
        for name in crate::style_resolver::persistent_selector::split_whitespace(classes) {
            append(&index.classes, name);
        }
    }
    append(&index.types, &name.to_ascii_lowercase());
    result.sort_unstable_by_key(|rule| rule.source_order);
    result.dedup_by(|left, right| std::ptr::eq(left.rule, right.rule));
    result
}
// cpp: style_resolver/style_resolver.cc:806-824
// Resolver-owned borrows replace the thread-local active-index pointer while
// preserving the source's per-call cache lifetime and nested-call isolation.
pub(crate) struct ActiveStyleRuleIndexes<'a> {
    pub user_agent: StyleRuleIndex<'a>,
    pub author: StyleRuleIndex<'a>,
}
impl<'a> ActiveStyleRuleIndexes<'a> {
    pub(crate) fn new(user_agent: &'a [CSSStyleSheet], author: &'a [CSSStyleSheet]) -> Self {
        Self {
            user_agent: BuildStyleRuleIndex(user_agent),
            author: BuildStyleRuleIndex(author),
        }
    }
}

pub(crate) fn CompileSelectors(
    text: &str,
) -> std::sync::Arc<[cssom::compiled_rules::CompiledSelector]> {
    SplitSelectorList(text)
        .into_iter()
        .map(|item| {
            let mut selector = ParseSelector(&item);
            let target = ExtractTerminalPseudo(&mut selector);
            cssom::compiled_rules::CompiledSelector { selector, target }
        })
        .collect::<Vec<_>>()
        .into()
}

#[path = "attribute_dependencies.rs"]
pub(crate) mod attribute_dependencies;
#[path = "bounded_has_dependencies.rs"]
mod bounded_has_dependencies;
#[path = "inline_style_dependencies.rs"]
mod inline_style_dependencies;

pub(crate) fn BuildCompiledRuleSet<'a>(
    sheets: impl IntoIterator<Item = &'a CSSStyleSheet>,
) -> cssom::compiled_rules::CompiledRuleSet {
    use cssom::compiled_rules::{CompiledRuleSet, CompiledStyleRule};
    let mut result = CompiledRuleSet::default();
    for (sheet_index, sheet) in sheets.into_iter().enumerate() {
        for name in sheet
            .layer_order
            .iter()
            .chain(sheet.rules.iter().map(|r| &r.layer_name))
        {
            if !name.is_empty() && !result.layers.contains_key(name) {
                result.layers.insert(name.clone(), result.layers.len());
            }
        }
        for (rule_index, rule) in sheet.rules.iter().enumerate() {
            let selectors = CompileSelectors(&rule.selector_text);
            let slot = result.rules.len();
            for kind in cssom::compiled_rules::SelectorOnlyAttribute::ALL {
                if SplitSelectorList(&rule.selector_text)
                    .iter()
                    .zip(selectors.iter())
                    .any(|(text, compiled)| {
                        compiled.selector.valid
                            && compiled.target.index().is_some()
                            && inline_style_dependencies::MayReadAttribute(text, kind.name())
                    })
                {
                    result.attribute_selector_dependencies |= kind.mask();
                    result.selector_only_attribute_dependencies |= kind.mask();
                }
            }
            for declaration in &rule.declarations {
                result.selector_only_attribute_dependencies |=
                    attribute_dependencies::DeclarationAttributeDependencies(&declaration.value);
            }
            // Conservative dependency flags include selectors nested in :is/:not.
            let text = rule.selector_text.to_ascii_lowercase();
            result.depends_on_siblings |= text.contains('+')
                || text.contains('~')
                || text.contains(":nth-")
                || text.contains(":first-")
                || text.contains(":last-")
                || text.contains(":only-");
            result.depends_on_nth_last_children |=
                text.contains(":nth-last-child(") || text.contains(":nth-last-of-type(");
            result.depends_on_descendants |= text.contains(":has(");
            for compiled in selectors
                .iter()
                .filter(|s| s.selector.valid && s.target.index().is_some())
            {
                for (position, combinator) in compiled.selector.combinators.iter().enumerate() {
                    if matches!(combinator, b'+' | b'~') {
                        let host = &compiled.selector.compounds[position];
                        if !result.sibling_invalidation_hosts.contains(host) {
                            result.sibling_invalidation_hosts.push(host.clone());
                        }
                    }
                    if matches!(combinator, b' ' | b'>') {
                        let host = &compiled.selector.compounds[position];
                        if !result.descendant_invalidation_hosts.contains(host) {
                            result.descendant_invalidation_hosts.push(host.clone());
                        }
                    }
                }
                match bounded_has_dependencies::BoundedHasHost(&compiled.selector) {
                    Ok(Some(host)) if !result.bounded_has_hosts.contains(&host) => {
                        result.bounded_has_hosts.push(host)
                    }
                    Ok(_) => {}
                    Err(()) => result.has_dependency_fallback = true,
                }
                for compound in &compiled.selector.compounds {
                    match bounded_has_dependencies::NthLastHost(compound) {
                        Ok(Some(host)) if !result.nth_last_child_hosts.contains(&host) => {
                            result.nth_last_child_hosts.push(host);
                        }
                        Ok(_) => {}
                        Err(()) => result.nth_last_child_host_fallback = true,
                    }
                    match bounded_has_dependencies::ChildStateHost(compound) {
                        Ok(Some(host)) if !result.empty_container_hosts.contains(&host) => {
                            result.empty_container_hosts.push(host)
                        }
                        Ok(_) => {}
                        Err(()) => {
                            if let Some(host) =
                                bounded_has_dependencies::ChildStateFallbackHost(compound)
                            {
                                if !result
                                    .children_container_fallback_hosts
                                    .iter()
                                    .any(|old| old == host)
                                {
                                    result
                                        .children_container_fallback_hosts
                                        .push(host.to_owned());
                                }
                            } else {
                                result.children_container_dependency_fallback = true;
                            }
                        }
                    }
                }
            }
            // Use exactly the selectors admitted to a candidate target. An
            // unsupported terminal pseudo has no matching target and cannot
            // observe a style mutation. Keep proof conservative for every
            // admitted selector, including unknown functions and mixed lists.
            // CompileSelectors preserves SplitSelectorList order one-for-one.
            result.inline_style_may_affect_selectors |= SplitSelectorList(&rule.selector_text)
                .iter()
                .zip(selectors.iter())
                .any(|(text, compiled)| {
                    compiled.selector.valid
                        && compiled.target.index().is_some()
                        && inline_style_dependencies::MayReadStyleAttribute(text)
                });
            for (target, index) in result.targets.iter_mut().enumerate() {
                let mut keys = Vec::new();
                let mut universal = false;
                for compiled in selectors
                    .iter()
                    .filter(|s| s.selector.valid && s.target.index() == Some(target))
                {
                    if let Some(key) = FastKeyForSelector(&compiled.selector) {
                        if !keys.contains(&key) {
                            keys.push(key);
                        }
                    } else {
                        universal = true;
                        break;
                    }
                }
                if universal {
                    index.universal.push(slot);
                } else {
                    for key in keys {
                        let bucket = match key.kind {
                            b'#' => &mut index.ids,
                            b'.' => &mut index.classes,
                            _ => &mut index.types,
                        };
                        bucket.entry(key.value).or_default().push(slot);
                    }
                }
            }
            result.rules.push(CompiledStyleRule {
                sheet_index,
                rule_index,
                selectors,
                source_order: result.declaration_count,
                depends_on_nth_last_children: text.contains(":nth-last-child(")
                    || text.contains(":nth-last-of-type("),
                depends_on_last_children: text.contains(":last-child")
                    || text.contains(":last-of-type")
                    || text.contains(":only-child")
                    || text.contains(":only-of-type"),
            });
            result.declaration_count += rule.declarations.len();
        }
    }
    result
}

pub(crate) fn CompiledCandidateRules<'a>(
    rules: &'a cssom::compiled_rules::CompiledRuleSet,
    node: &DOMNode,
    target: crate::style_resolver::selector::PseudoTarget,
) -> Vec<&'a cssom::compiled_rules::CompiledStyleRule> {
    let Some(target) = target.index() else {
        return Vec::new();
    };
    let index = &rules.targets[target];
    if index.universal.is_empty()
        && index.ids.is_empty()
        && index.classes.is_empty()
        && index.types.is_empty()
    {
        return Vec::new();
    }
    let mut slots = index.universal.clone();
    let mut append = |bucket: &BTreeMap<String, Vec<usize>>, key: &str| {
        if let Some(items) = bucket.get(key) {
            slots.extend_from_slice(items);
        }
    };
    if let Some(id) = node.FindAttribute("id") {
        append(&index.ids, &id.value);
    }
    if let Some(classes) = node.FindAttribute("class") {
        for class in crate::style_resolver::persistent_selector::split_whitespace(&classes.value) {
            append(&index.classes, class);
        }
    }
    append(&index.types, &node.Name().to_ascii_lowercase());
    slots.sort_unstable();
    slots.dedup();
    slots.into_iter().map(|slot| &rules.rules[slot]).collect()
}

/// Visit the retained index buckets without building and sorting a temporary
/// candidate vector. Callers which combine buckets must suppress duplicate
/// slots; a rule can be indexed by more than one class/id in a selector list.
pub(crate) fn ForEachCompiledCandidateSlot(
    rules: &cssom::compiled_rules::CompiledRuleSet,
    node: &DOMNode,
    target: crate::style_resolver::selector::PseudoTarget,
    mut visit: impl FnMut(usize),
) {
    let Some(target) = target.index() else {
        return;
    };
    let index = &rules.targets[target];
    for &slot in &index.universal {
        visit(slot);
    }
    if let Some(id) = node.FindAttribute("id") {
        if let Some(bucket) = index.ids.get(&id.value) {
            for &slot in bucket {
                visit(slot);
            }
        }
    }
    if let Some(classes) = node.FindAttribute("class") {
        for class in crate::style_resolver::persistent_selector::split_whitespace(&classes.value) {
            if let Some(bucket) = index.classes.get(class) {
                for &slot in bucket {
                    visit(slot);
                }
            }
        }
    }
    if let Some(bucket) = index.types.get(&node.Name().to_ascii_lowercase()) {
        for &slot in bucket {
            visit(slot);
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use crate::style_resolver::selector::PseudoTarget;
    use cssom::compiled_rules::{CompiledRuleIndex, CompiledRuleSet};

    // Reproduce the previous target-independent candidate path for an in-process
    // CPU comparison, using identical CSS, DOM, compiler and cache temperature.
    fn legacy_index(rules: &CompiledRuleSet) -> CompiledRuleIndex {
        let mut index = CompiledRuleIndex::default();
        for (slot, rule) in rules.rules.iter().enumerate() {
            let mut keys = Vec::new();
            let mut universal = false;
            for selector in rule.selectors.iter().filter(|s| s.selector.valid) {
                if let Some(key) = FastKeyForSelector(&selector.selector) {
                    if !keys.contains(&key) {
                        keys.push(key);
                    }
                } else {
                    universal = true;
                }
            }
            if universal {
                index.universal.push(slot);
            } else {
                for key in keys {
                    let map = match key.kind {
                        b'#' => &mut index.ids,
                        b'.' => &mut index.classes,
                        _ => &mut index.types,
                    };
                    map.entry(key.value).or_default().push(slot);
                }
            }
        }
        index
    }

    fn legacy_candidates<'a>(
        rules: &'a CompiledRuleSet,
        index: &CompiledRuleIndex,
        node: &DOMNode,
    ) -> Vec<&'a cssom::compiled_rules::CompiledStyleRule> {
        let mut slots = index.universal.clone();
        let mut append = |map: &BTreeMap<String, Vec<usize>>, key: &str| {
            if let Some(items) = map.get(key) {
                slots.extend_from_slice(items);
            }
        };
        if let Some(id) = node.FindAttribute("id") {
            append(&index.ids, &id.value);
        }
        if let Some(classes) = node.FindAttribute("class") {
            for class in
                crate::style_resolver::persistent_selector::split_whitespace(&classes.value)
            {
                append(&index.classes, class);
            }
        }
        append(&index.types, &node.Name().to_ascii_lowercase());
        slots.sort_unstable();
        slots.dedup();
        slots.into_iter().map(|slot| &rules.rules[slot]).collect()
    }

    #[test]
    #[ignore = "CPU comparison; set BROWSER_STYLE_BENCH_HTML and BROWSER_STYLE_BENCH_CSS to captured files"]
    fn captured_pseudo_candidates_cpu_comparison() {
        use std::{hint::black_box, time::Instant};
        let html =
            std::fs::read_to_string(std::env::var("BROWSER_STYLE_BENCH_HTML").unwrap()).unwrap();
        let css =
            std::fs::read_to_string(std::env::var("BROWSER_STYLE_BENCH_CSS").unwrap()).unwrap();
        let owner = crate::test_html::html_parser::ParseHTML(&html);
        let document = owner.GetDocument();
        let sheet = cssom::ParseCSS(&css);
        let rules = BuildCompiledRuleSet([&sheet]);
        let legacy = legacy_index(&rules);
        let nodes: Vec<_> = (0..document.NodeCount())
            .filter(|&i| {
                document.Node(i).Type() == crate::persistent_document::DOMNodeType::kElement
            })
            .collect();
        let targets = [
            PseudoTarget::Element,
            PseudoTarget::Before,
            PseudoTarget::After,
            PseudoTarget::FirstLetter,
            PseudoTarget::Placeholder,
        ];
        // Check matched source order and specificity separately from timing.
        // Candidate pruning may remove failures but must retain every match.
        for &node in &nodes {
            for target in targets {
                let matched = |candidates: Vec<&cssom::compiled_rules::CompiledStyleRule>| {
                    candidates
                        .into_iter()
                        .filter_map(|rule| {
                            crate::style_resolver::persistent_selector::MatchCompiledSelectorTarget(
                                document,
                                node,
                                &rule.selectors,
                                target,
                            )
                            .map(|specificity| (rule.source_order, specificity))
                        })
                        .collect::<Vec<_>>()
                };
                assert_eq!(
                    matched(legacy_candidates(&rules, &legacy, document.Node(node))),
                    matched(CompiledCandidateRules(&rules, document.Node(node), target))
                );
            }
        }
        let mut before = Vec::new();
        let mut after = Vec::new();
        let mut counts = [0usize; 2];
        // Alternate order across repetitions to avoid first-run bias.
        for repetition in 0..7 {
            for version in if repetition % 2 == 0 { [0, 1] } else { [1, 0] } {
                let start = Instant::now();
                let mut count = 0;
                for _ in 0..20 {
                    for &node in &nodes {
                        for target in targets {
                            // ResolveElement asks for placeholder style only on
                            // text controls; don't inflate the timing workload.
                            if target == PseudoTarget::Placeholder
                                && !document.Node(node).IsHTMLElement("input")
                                && !document.Node(node).IsHTMLElement("textarea")
                            {
                                continue;
                            }
                            let candidates = if version == 0 {
                                legacy_candidates(&rules, &legacy, document.Node(node))
                            } else {
                                CompiledCandidateRules(&rules, document.Node(node), target)
                            };
                            count += black_box(candidates).len();
                        }
                    }
                }
                counts[version] = count;
                let times = if version == 0 {
                    &mut before
                } else {
                    &mut after
                };
                times.push(start.elapsed().as_secs_f64() * 1000.0);
            }
        }
        before.sort_by(f64::total_cmp);
        after.sort_by(f64::total_cmp);
        eprintln!("pseudo-candidate-cpu nodes={} rules={} turns=20 targets=4+control_placeholder old_median_ms={:.3} new_median_ms={:.3} old_candidates={} new_candidates={} matched_specificity_equal=true",
            nodes.len(), rules.rules.len(), before[3], after[3], counts[0], counts[1]);
    }
}

#[cfg(test)]
mod invalidation_tests {
    use super::*;

    #[test]
    fn collects_only_compounds_left_of_sibling_combinators() {
        let sheet = cssom::ParseCSS(
            ".trigger + .target, .outer .switch ~ .target .child { color:red } \
             .ordinary .descendant { color:blue }",
        );
        let rules = BuildCompiledRuleSet([&sheet]);
        assert_eq!(rules.sibling_invalidation_hosts, [".trigger", ".switch"]);
    }
}
