// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Document-owned, concrete assembly of the translated selector, declaration,
//! cascade priority and native style builder. Document roots every final style.
#![allow(non_snake_case)]
use crate::{
    css_property_name::CSSPropertyName,
    css_selector::{CSSSelector, MatchType, PseudoType, RelationType},
    css_selector_list::CSSSelectorList,
    media_queries::{
        media_query_backend::{MediaQueryValueBackend, ParseMediaQuerySet},
        media_query_evaluator::MediaQueryEvaluator,
        production_container_query::{
            ContainerState, ContainerStyleResolver, DocumentMediaValues,
            EvaluateContainerCondition, RegisteredContainerPropertyResolver, RequiredContainerType,
        },
        MediaValuesCachedData,
    },
    parser::{
        css_parser_mode::CSSParserMode,
        css_selector_parser::{CSSSelectorParser, SelectorParserContext},
        production_property_parser::{ParseDeclarationList, PropertyParseErrorKind},
    },
    persistent_selector::{PersistentSelectorError, PersistentSelectorService},
    production_css_value::{PropertyValue, Value},
    properties::longhand_dispatch::{LonghandApplicationError, ResolvePhysical},
    resolver::{
        cascade_map::CascadeMap,
        cascade_origin::CascadeOrigin,
        cascade_priority::CascadePriority,
        production_style_builder::{
            self,
            custom_properties::{
                attributes::{
                    AttributeResolutionState, AttributeResolver, PersistentAttributeSource,
                },
                ComputedVariableValue, CustomProperties, EnvironmentVariableResolver,
                ResolvePropertyValueWithContext, SubstitutionContext, VariableResolutionError,
            },
        },
    },
};
use cssom::css_style_sheet::CSSStyleSheet;
use dom::{
    persistent_document::{
        CompatibilityMode, DOMNamespace, DOMNodeType, DOMOwnerHandle, PseudoElement,
        ResolvedNodeStyle,
    },
    style_state::{StyleChange, StyleUpdateImpact, StyleUpdateStats},
    Document, DOM,
};
use foundation::{CSSPropertyID, EDisplay, Persistent};
use layoutng_style::style::{
    computed_style::{ComputedStyle, ComputedStyleBuilder, ComputedStyleDifference},
    computed_style_base::{ComputedStyleBase, FieldDifference, IsAtShadowBoundary},
    computed_style_constants::{PseudoId, PseudoIdFlags},
    content_data::TextContentData,
};
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    fmt,
    rc::Rc,
    sync::Arc,
};

// cpp: style_cascade.cc:2351-2390,2924-2930.  Environment lookup is supplied
// by the document host, while the style bits always belong to the current
// builder.  Keeping these per-cascade flags here prevents an environment owner
// from retaining or mutating a builder after the call returns.
struct CascadeEnvironment<'a> {
    upstream: Option<&'a dyn EnvironmentVariableResolver>,
    has_env: Cell<bool>,
    has_safe_area_bottom: Cell<bool>,
}

impl<'a> CascadeEnvironment<'a> {
    fn new(upstream: Option<&'a dyn EnvironmentVariableResolver>) -> Self {
        Self {
            upstream,
            has_env: Cell::new(false),
            has_safe_area_bottom: Cell::new(false),
        }
    }
    fn ApplyState(&self, builder: &mut ComputedStyleBuilder) {
        if self.has_env.get() {
            builder.SetHasEnv();
        }
        if self.has_safe_area_bottom.get() {
            builder.SetHasEnvSafeAreaInsetBottom();
        }
    }
}

impl EnvironmentVariableResolver for CascadeEnvironment<'_> {
    fn ViewportSegmentsEnabled(&self) -> bool {
        self.upstream
            .is_some_and(EnvironmentVariableResolver::ViewportSegmentsEnabled)
    }
    fn ResolveEnvironmentVariable(
        &self,
        name: &foundation::AtomicString,
        indices: &[u32],
    ) -> Option<Rc<crate::production_css_value::CSSVariableData>> {
        self.has_env.set(true);
        if name == &foundation::AtomicString::from_str("safe-area-inset-bottom") {
            self.has_safe_area_bottom.set(true);
        }
        self.upstream
            .and_then(|source| source.ResolveEnvironmentVariable(name, indices))
    }
}

struct CascadeAttributeState {
    has_attr: Cell<bool>,
}

impl CascadeAttributeState {
    fn new() -> Self {
        Self {
            has_attr: Cell::new(false),
        }
    }
    fn ApplyState(&self, builder: &mut ComputedStyleBuilder) {
        if self.has_attr.get() {
            builder.SetHasAttrFunction();
        }
    }
}

impl AttributeResolutionState for CascadeAttributeState {
    fn SetHasAttrFunction(&self) {
        self.has_attr.set(true);
    }
}

#[derive(Debug)]
pub enum DocumentStyleError {
    WrongDocument,
    Selector(PersistentSelectorError),
    Property {
        node: usize,
        property: CSSPropertyID,
        offset: u32,
        operation: &'static str,
        source_file: &'static str,
        source_line: u32,
    },
    Application {
        node: usize,
        error: LonghandApplicationError,
    },
    CascadeIndexOverflow,
    PropertyRegistration(crate::property_registry::PropertyRegistryError),
    Unsupported(&'static str),
}
impl fmt::Display for DocumentStyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "style update: {self:?}")
    }
}
impl std::error::Error for DocumentStyleError {}
struct MatchedRule {
    specificity: u32,
    proximity: u32,
    order: usize,
    origin: CascadeOrigin,
    inline: bool,
    layer: u16,
    properties: Vec<PropertyValue>,
}

// A style request names a pseudo on the originating element, never a DOM query
// for a synthetic element. Other pseudo families require their own lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum PseudoStyleKind {
    Before,
    After,
    FirstLetter,
    Placeholder,
}
impl PseudoStyleKind {
    const ALL: [Self; 4] = [
        Self::Before,
        Self::After,
        Self::FirstLetter,
        Self::Placeholder,
    ];
    fn Id(self) -> PseudoId {
        match self {
            Self::Before => PseudoId::kPseudoIdBefore,
            Self::After => PseudoId::kPseudoIdAfter,
            Self::FirstLetter => PseudoId::kPseudoIdFirstLetter,
            Self::Placeholder => PseudoId::kPseudoIdPlaceholder,
        }
    }
}
struct StyleSelector {
    subject: String,
    compiled_subject: Rc<CSSSelectorList>,
    pseudo: Option<PseudoStyleKind>,
    specificity: u32,
}

// cpp: document.h:729,2856; document.cc:1103; style_engine.cc:StyleEngine,
// UpdateActiveStyleSheets/UpdateStyleAndLayoutTree; style_resolver.cc:ResolveStyle.
// The source's platform/page services are outside this document-only assembly.
// Unsupported declarations are retained as typed diagnostics and skipped.
// Structural errors keep all previously rooted document styles.
pub struct DocumentStyleEngine {
    document: DOMOwnerHandle,
    selectors: PersistentSelectorService,
    diagnostics: Vec<DocumentStyleError>,
    active_sheets: Arc<Vec<(CSSStyleSheet, CascadeOrigin)>>,
    sheet_revision: Option<u64>,
    environment: Option<(MediaValuesCachedData, CompatibilityMode)>,
    matches: HashMap<usize, Vec<(usize, u32, u32)>>,
    cascade_values: HashMap<usize, HashMap<CSSPropertyID, Rc<Value>>>,
    custom_properties: HashMap<usize, CustomProperties>,
    sheet_dependencies: Vec<SelectorDependencies>,
    style_selectors: HashMap<(String, bool, bool), Vec<StyleSelector>>,
    container_states: HashMap<usize, ContainerState>,
    layout_container_nodes: HashSet<usize>,
    property_registry: crate::property_registry::PropertyRegistry,
    registered_container_properties: Option<Rc<dyn RegisteredContainerPropertyResolver>>,
    color_scheme_settings: production_style_builder::ColorSchemeSettings,
}
impl DocumentStyleEngine {
    pub fn new(owner: &DOM) -> Self {
        Self {
            document: owner.GetDocument().RootHandle(),
            selectors: Default::default(),
            diagnostics: vec![],
            active_sheets: Default::default(),
            sheet_revision: None,
            environment: None,
            matches: HashMap::new(),
            cascade_values: HashMap::new(),
            custom_properties: HashMap::new(),
            sheet_dependencies: vec![],
            style_selectors: HashMap::new(),
            container_states: HashMap::new(),
            layout_container_nodes: HashSet::new(),
            property_registry: Default::default(),
            registered_container_properties: None,
            color_scheme_settings: Default::default(),
        }
    }
    /// Layout/scroll/anchor owners publish the complete state after their update.
    /// Descendant query matches must be revisited even when CSS/DOM are unchanged.
    pub fn SetContainerState(
        &mut self,
        owner: &mut DOM,
        node: usize,
        state: Option<ContainerState>,
    ) -> Result<bool, DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        let changed = self.container_states.get(&node) != state.as_ref();
        if changed {
            if let Some(state) = state {
                self.container_states.insert(node, state);
            } else {
                self.container_states.remove(&node);
            }
            owner.GetDocumentMut().StyleStateMut().all_dirty = true;
        }
        Ok(changed)
    }
    /// Replace the completed layout's size observations. Scroll/anchor state
    /// stays owned by its separate publisher; unavailable observations remain
    /// unavailable rather than becoming a negative query result.
    pub fn SetContainerSizes(
        &mut self,
        owner: &mut DOM,
        sizes: impl IntoIterator<Item = (usize, f64, f64)>,
    ) -> Result<bool, DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        let mut seen = HashSet::new();
        let mut changed = false;
        for (node, width, height) in sizes {
            let Some(native) = owner
                .GetDocument()
                .ResolvedStyleFor(node)
                .and_then(|s| unsafe { s.native_style.Get().as_ref() })
            else {
                continue;
            };
            seen.insert(node);
            let mut state = self
                .container_states
                .get(&node)
                .cloned()
                .unwrap_or_else(|| {
                    ContainerState::SizeOnly(width, height, native.GetWritingDirection())
                });
            state.width = Some(width);
            state.height = Some(height);
            changed |= self.SetContainerState(owner, node, Some(state))?;
        }
        let removed = self
            .layout_container_nodes
            .difference(&seen)
            .copied()
            .collect::<Vec<_>>();
        for node in removed {
            let state = self
                .container_states
                .get(&node)
                .cloned()
                .and_then(|mut state| {
                    state.width = None;
                    state.height = None;
                    (state.scroll_state_available || state.anchored_state_available)
                        .then_some(state)
                });
            changed |= self.SetContainerState(owner, node, state)?;
        }
        self.layout_container_nodes = seen;
        Ok(changed)
    }
    /// The document registration owner supplies actual syntax-computed reads.
    pub fn SetRegisteredContainerProperties(
        &mut self,
        owner: &mut DOM,
        resolver: Option<Rc<dyn RegisteredContainerPropertyResolver>>,
    ) -> Result<(), DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        self.registered_container_properties = resolver;
        owner.GetDocumentMut().StyleStateMut().all_dirty = true;
        Ok(())
    }
    /// The real page/style-engine owner supplies page color-scheme and
    /// force-dark settings; media owns the preferred scheme.
    pub fn SetColorSchemeSettings(
        &mut self,
        owner: &mut DOM,
        settings: production_style_builder::ColorSchemeSettings,
    ) -> Result<(), DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        if self.color_scheme_settings != settings {
            self.color_scheme_settings = settings;
            owner.GetDocumentMut().StyleStateMut().all_dirty = true;
        }
        Ok(())
    }
    pub fn PropertyRegistry(&self) -> &crate::property_registry::PropertyRegistry {
        &self.property_registry
    }
    /// A typed host API for CSS.registerProperty. The script binding remains
    /// separate; no stylesheet declaration is promoted to a script slot.
    pub fn RegisterProperty(
        &mut self,
        owner: &mut DOM,
        name: foundation::AtomicString,
        registration: crate::property_registry::PropertyRegistration,
    ) -> Result<(), DocumentStyleError> {
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        self.property_registry
            .RegisterProperty(name, registration)
            .map_err(DocumentStyleError::PropertyRegistration)?;
        owner.GetDocumentMut().StyleStateMut().all_dirty = true;
        Ok(())
    }
    pub fn Diagnostics(&self) -> &[DocumentStyleError] {
        &self.diagnostics
    }
    pub fn HasContainerQueries(&self) -> bool {
        self.active_sheets.iter().any(|(sheet, _)| {
            sheet
                .rules
                .iter()
                .any(|rule| !rule.container_conditions.is_empty())
        })
    }
    pub fn TakeDiagnostics(&mut self) -> Vec<DocumentStyleError> {
        std::mem::take(&mut self.diagnostics)
    }
    pub fn Update(
        &mut self,
        owner: &mut DOM,
        media: &MediaValuesCachedData,
        ua: &[CSSStyleSheet],
    ) -> Result<StyleUpdateImpact, DocumentStyleError> {
        self.UpdateWithEnvironment(owner, media, ua, None)
    }
    pub fn UpdateWithEnvironment(
        &mut self,
        owner: &mut DOM,
        media: &MediaValuesCachedData,
        ua: &[CSSStyleSheet],
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
    ) -> Result<StyleUpdateImpact, DocumentStyleError> {
        self.diagnostics.clear();
        if owner.GetDocument().RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        let state = owner.GetDocument().StyleState();
        if state.all_dirty
            || !state.dirty_style_elements.is_empty()
            || !state.inserted_style_subtrees.is_empty()
            || state
                .changes
                .iter()
                .any(|c| matches!(c, StyleChange::Children(_)))
        {
            self.SynchronizeStyleSheets(owner);
        }
        let document = owner.GetDocument();
        if document.RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        // Blink's ApplyRuleSetChanges invalidates against old and new rule sets.
        // Retain exact source snapshots: removals and source-order/layer changes
        // are observable even when the DOM and declaration text are unchanged.
        let revision = document.StyleState().sheet_revision;
        let ua_unchanged = self
            .active_sheets
            .iter()
            .filter(|(_, origin)| *origin == CascadeOrigin::kUserAgent)
            .map(|(s, _)| s)
            .eq(ua.iter());
        let active_sheets = if self.sheet_revision == Some(revision) && ua_unchanged {
            self.active_sheets.clone()
        } else {
            Arc::new(
                ua.iter()
                    .cloned()
                    .map(|s| (s, CascadeOrigin::kUserAgent))
                    .chain(
                        document
                            .ActiveStyleSheets()
                            .cloned()
                            .map(|s| (s, CascadeOrigin::kAuthor)),
                    )
                    .collect::<Vec<_>>(),
            )
        };
        let registry_changed = self
            .property_registry
            .UpdateDeclaredProperties(&active_sheets, &DocumentMediaValues::new(document, media));
        let environment = (media.clone(), document.GetCompatibilityMode());
        let all_dirty = registry_changed || document.StyleState().all_dirty || self.environment.as_ref() != Some(&environment)
            // Container ancestry, scope activation and computed custom values are query inputs.
            // Until per-container dependency sets are retained, any DOM/style
            // mutation revisits conditional-rule subjects against current ancestors.
            || (active_sheets.iter().any(|(s,_)|s.rules.iter().any(|r|!r.container_conditions.is_empty() || !r.scope_conditions.is_empty()))
                && !document.StyleState().changes.is_empty());
        let sheet_changes = ChangedSheets(&self.active_sheets, &active_sheets);
        let quirks_changed = self
            .environment
            .as_ref()
            .is_none_or(|(_, mode)| *mode != environment.1);
        let mut rule_sets_built = 0;
        let sheet_dependencies: Vec<_> = active_sheets
            .iter()
            .enumerate()
            .map(|(index, sheet)| {
                if !quirks_changed && self.active_sheets.get(index) == Some(sheet) {
                    if let Some(retained) = self.sheet_dependencies.get(index) {
                        return retained.clone();
                    }
                }
                rule_sets_built += 1;
                SelectorDependencies::ForSheets(document, std::slice::from_ref(sheet))
            })
            .collect();
        let mut dependencies = SelectorDependencies::default();
        for sheet in &sheet_dependencies {
            dependencies.Merge(sheet);
        }
        let sheets: Vec<_> = active_sheets.iter().map(|(s, o)| (s, *o)).collect();
        let mut dirty = HashSet::new();
        let mut candidates = HashSet::new();
        let mut refreshed_matches = HashMap::new();
        if all_dirty {
            AddSubtree(document, document.Root(), &mut dirty);
        } else {
            let state = document.StyleState();
            for change in &state.changes {
                match *change {
                    StyleChange::Node(index) | StyleChange::Attribute(index, _) => {
                        dirty.insert(index);
                        self.InvalidateRelations(document, index, &dependencies, &mut candidates);
                    }
                    StyleChange::InlineStyle(index) => {
                        dirty.insert(index);
                        if dependencies.inline_selector {
                            self.InvalidateRelations(
                                document,
                                index,
                                &dependencies,
                                &mut candidates,
                            );
                        }
                    }
                    StyleChange::Animation(index) => {
                        dirty.insert(index);
                    }
                    StyleChange::Resource(_) => {} // Payload changes retain immutable style.
                    StyleChange::Children(parent) => {
                        candidates.insert(parent);
                        // Child-state subjects such as :empty can be on the
                        // left side of a sibling combinator.
                        if dependencies.sibling {
                            if let Some(grandparent) = document.Node(parent).Parent() {
                                AddSubtree(document, grandparent, &mut candidates);
                            }
                        }
                        if state.non_append_children.contains(&parent)
                            && (dependencies.positional || dependencies.sibling)
                        {
                            AddSubtree(document, parent, &mut candidates);
                        } else if dependencies.reverse_positional {
                            AddSubtree(document, parent, &mut candidates);
                        } else if dependencies.last {
                            for &index in &state.append_affected_subtrees {
                                AddSubtree(document, index, &mut candidates);
                            }
                        }
                        if dependencies.has {
                            self.InvalidateHasAncestors(document, parent, &mut candidates);
                        }
                    }
                }
            }
            for &index in &state.inserted_style_subtrees {
                AddSubtree(document, index, &mut dirty);
            }
            // Rule-set invalidation checks both previously matched rules and
            // current matches. The former survives simultaneous DOM mutations.
            if !sheet_changes.is_empty() {
                let mut changed_selectors = HashSet::new();
                for &i in &sheet_changes {
                    for source in [&self.active_sheets, &active_sheets] {
                        if let Some((sheet, _)) = source.get(i) {
                            changed_selectors
                                .extend(sheet.rules.iter().map(|r| r.selector_text.clone()));
                        }
                    }
                }
                let old_selectors: Vec<_> = self
                    .active_sheets
                    .iter()
                    .flat_map(|(s, _)| s.rules.iter().map(|r| r.selector_text.clone()))
                    .collect();
                let mut elements = HashSet::new();
                AddSubtree(document, document.Root(), &mut elements);
                for index in elements {
                    refreshed_matches.insert(
                        index,
                        self.MatchingRules(
                            document,
                            index,
                            media,
                            &sheets,
                            None,
                            None,
                            environment_variables,
                        ),
                    );
                    let was_matched = self.matches.get(&index).is_some_and(|matches| {
                        matches.iter().any(|(order, _, _)| {
                            old_selectors
                                .get(order - 1)
                                .is_some_and(|s| changed_selectors.contains(s))
                        })
                    });
                    if was_matched
                        || changed_selectors
                            .iter()
                            .any(|selector| self.MatchesAnyStyleTarget(document, index, selector))
                    {
                        dirty.insert(index);
                    }
                }
                // Rule order in the retained match signatures changed. Refresh
                // signatures for candidates instead of comparing unlike revisions.
            }
            for index in candidates {
                if dirty.contains(&index) || !ConnectedElement(document, index) {
                    continue;
                }
                let matches = self.MatchingRules(
                    document,
                    index,
                    media,
                    &sheets,
                    None,
                    None,
                    environment_variables,
                );
                if self.matches.get(&index) != Some(&matches) {
                    dirty.insert(index);
                }
            }
        }
        dirty.retain(|&index| ConnectedElement(document, index));
        // ChildNeedsStyleRecalc equivalent: enter only paths to dirty elements;
        // inherited differences can extend traversal during style resolution.
        let mut visit = HashSet::new();
        for &index in &dirty {
            let mut ancestor = Some(index);
            while let Some(index) = ancestor {
                if !visit.insert(index) {
                    break;
                }
                ancestor = document.Node(index).Parent();
            }
        }
        if visit.is_empty() {
            self.sheet_revision = Some(revision);
            self.active_sheets = active_sheets;
            self.environment = Some(environment);
            self.sheet_dependencies = sheet_dependencies;
            self.matches.extend(refreshed_matches);
            FinishUpdate(
                owner.GetDocumentMut(),
                StyleUpdateStats {
                    rule_sets_built,
                    ..Default::default()
                },
                StyleUpdateImpact::default(),
            );
            return Ok(StyleUpdateImpact::default());
        }
        // Build all immutable styles before publishing. A failure retains the
        // DOM dirty batch, old rule snapshots and every previously rooted style.
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut root_builder = ComputedStyleBuilder::from_style(initial);
        // cpp: style_resolver.cc:2397-2400 initial used-color-scheme setup; page settings
        // are real host inputs, with Chromium's initial defaults when absent.
        production_style_builder::ApplyWithColorSchemeSettings(
            CSSPropertyID::kColorScheme,
            &mut root_builder,
            None,
            &crate::production_css_value::wide(foundation::CSSValueID::kInitial).unwrap(),
            media.em_size,
            media,
            self.color_scheme_settings,
        )
        .map_err(|error| DocumentStyleError::Application {
            node: document.Root(),
            error,
        })?;
        let mut description = root_builder.GetFontDescription().clone();
        description.SetKeywordSize(4);
        description.SetGenericFamily(
            font_engine::fonts::font_description::GenericFamilyType::kStandardFamily,
        );
        description.SetSpecifiedSize(media.em_size);
        description.SetComputedSize(media.em_size);
        production_style_builder::StageFontDescription(&mut root_builder, &description);
        let initial_root = Persistent::from_ptr(root_builder.TakeStyle() as *mut ComputedStyle);
        let initial_root_ref = unsafe { &*initial_root.Get() };
        let mut pending = Vec::<(usize, ResolvedNodeStyle)>::new();
        let mut native = HashMap::<usize, Persistent<ComputedStyle>>::new();
        let mut stack = vec![(document.Root(), None, false, false, false, false)];
        let mut stats = StyleUpdateStats {
            rule_sets_built,
            ..Default::default()
        };
        let mut impact = StyleUpdateImpact::default();
        let mut pending_matches = refreshed_matches;
        let mut pending_values = HashMap::new();
        let mut pending_custom_properties = HashMap::new();
        let mut custom_properties = HashMap::<usize, CustomProperties>::new();
        let mut root_font = media.em_size;
        while let Some((index, parent_index, ancestor_none, inherited, explicit, subtree)) =
            stack.pop()
        {
            let node = document.Node(index);
            let element = node.Type() == DOMNodeType::kElement;
            let mut next_parent = parent_index;
            let mut none = ancestor_none;
            let mut child_inherited = inherited;
            let mut child_explicit = explicit;
            let mut child_subtree = subtree;
            if element {
                let old = document.ResolvedStyleFor(index);
                let must_resolve = dirty.contains(&index)
                    || inherited
                    || subtree
                    || old.is_none_or(|s| s.native_style.Get().is_null())
                    || (explicit && old.is_none_or(|s| s.has_explicit_inheritance != Some(false)));
                if !must_resolve {
                    let old = old.unwrap();
                    native.insert(index, old.native_style.clone());
                    custom_properties.insert(
                        index,
                        self.custom_properties
                            .get(&index)
                            .cloned()
                            .unwrap_or_default(),
                    );
                    none |= unsafe { &*old.native_style.Get() }.Display() == EDisplay::kNone;
                    if parent_index.is_none() {
                        root_font = unsafe { &*old.native_style.Get() }
                            .GetFontDescription()
                            .SpecifiedSize();
                    }
                    next_parent = Some(index);
                    child_inherited = false;
                    child_explicit = false;
                } else {
                    stats.resolved_nodes += 1;
                    let parent = parent_index
                        .and_then(|p| native.get(&p))
                        .map(|p| unsafe { &*p.Get() });
                    let mut builder = if let Some(parent) = parent {
                        ComputedStyleBuilder::from_initial_and_parent(
                            initial_root_ref,
                            parent,
                            IsAtShadowBoundary::kNotAtShadowBoundary,
                        )
                    } else {
                        ComputedStyleBuilder::from_style(initial_root_ref)
                    };
                    let rules = self.CollectRules(
                        document,
                        index,
                        media,
                        &sheets,
                        None,
                        Some(&native),
                        Some(&custom_properties),
                        environment_variables,
                    )?;
                    pending_matches.insert(
                        index,
                        self.MatchingRules(
                            document,
                            index,
                            media,
                            &sheets,
                            Some(&native),
                            Some(&custom_properties),
                            environment_variables,
                        ),
                    );
                    let inherited_custom = parent_index
                        .and_then(|parent| custom_properties.get(&parent))
                        .cloned();
                    let (values, computed_custom) = Self::Cascade(
                        document,
                        index,
                        &rules,
                        &mut builder,
                        parent,
                        inherited_custom.as_ref(),
                        root_font,
                        media,
                        environment_variables,
                        &self.property_registry,
                        self.color_scheme_settings,
                        &mut self.diagnostics,
                    )?;
                    // cpp: MatchResult::SetHasPseudoElementStyle and
                    // StyleResolver::ApplyBaseStyleNoCache (1928). Public pseudo bits
                    // describe matched rules, independently of generated boxes.
                    let mut pseudo_rules = Vec::new();
                    let mut pseudo_flags = PseudoIdFlags::default();
                    for kind in PseudoStyleKind::ALL {
                        let rules = self.CollectRules(
                            document,
                            index,
                            media,
                            &sheets,
                            Some(kind),
                            Some(&native),
                            Some(&custom_properties),
                            environment_variables,
                        )?;
                        if !rules.is_empty() {
                            pseudo_flags.MaybeSet(kind.Id());
                        }
                        pseudo_rules.push((kind, rules));
                    }
                    builder.SetPseudoElementStyles(pseudo_flags.Bits());
                    // cpp: style_adjuster.cc:AdjustStyleForDisplay; root and out-of-flow
                    // elements use the source's equivalent block display.
                    if parent.is_none()
                        || builder.GetPosition() == foundation::EPosition::kAbsolute
                        || builder.GetPosition() == foundation::EPosition::kFixed
                        || builder.Floating() != foundation::EFloat::kNone
                    {
                        let display = builder.Display();
                        builder.SetDisplay(BlockDisplay(display));
                    }
                    let style = Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle);
                    let final_style = unsafe { &*style.Get() };
                    if parent.is_none() {
                        root_font = final_style.GetFontDescription().SpecifiedSize()
                    }
                    let own_none = final_style.Display() == EDisplay::kNone;
                    none |= own_none;
                    let mut resolved = ResolvedNodeStyle::default();
                    resolved.own_display_contents = final_style.Display() == EDisplay::kContents;
                    resolved.display_contents = resolved.own_display_contents;
                    resolved.own_generates_box = !own_none && !resolved.own_display_contents;
                    resolved.generates_box = !none && resolved.own_generates_box;
                    ProjectForLayoutBoundary(final_style, &mut resolved)?;
                    resolved.has_explicit_inheritance = Some(final_style.HasExplicitInheritance());
                    resolved.SetNativeStyle(style.clone());
                    let old_native = old.and_then(|s| unsafe { s.native_style.Get().as_ref() });
                    // cpp: computed_style.cc:407-500; element.cc:5562-5583.
                    // Propagation is decided by the translated native style
                    // difference, not by a hand-maintained projection subset.
                    let native_difference = old_native
                        .map_or(ComputedStyleDifference::kDescendantAffecting, |old| {
                            ComputedStyle::ComputeDifference(old, final_style)
                        });
                    let custom_changed = self
                        .custom_properties
                        .get(&index)
                        .is_none_or(|old| !CustomPropertiesEquivalent(old, &computed_custom));
                    let suppression_changed = old
                        .is_none_or(|s| s.generates_box != resolved.generates_box)
                        || old_native
                            .is_some_and(|old| (old.Display() == EDisplay::kNone) != own_none);
                    child_inherited = native_difference
                        >= ComputedStyleDifference::kIndependentInherited
                        || custom_changed;
                    child_subtree |= suppression_changed
                        || native_difference == ComputedStyleDifference::kDescendantAffecting;
                    // cpp: StyleResolver::ResolveStyle(StyleRequest),
                    // Element::GetCachedPseudoElementStyle. Pseudos inherit from
                    // their originating element and have a separate match result.
                    for (kind, rules) in pseudo_rules {
                        let pseudo = self.ResolvePseudoStyle(
                            document,
                            index,
                            kind,
                            &resolved,
                            &computed_custom,
                            initial_root_ref,
                            root_font,
                            media,
                            environment_variables,
                            &rules,
                            !none,
                        )?;
                        match kind {
                            PseudoStyleKind::Before => resolved.before = pseudo,
                            PseudoStyleKind::After => resolved.after = pseudo,
                            PseudoStyleKind::FirstLetter => resolved.first_letter = pseudo,
                            PseudoStyleKind::Placeholder => resolved.placeholder = pseudo,
                        }
                    }
                    let mut normalized = resolved.clone();
                    if let Some(old) = old {
                        normalized.SetNativeStyle(old.native_style.clone());
                    }
                    // The preliminary difference controls inheritance. Compare
                    // again after cache population so pseudo content changes
                    // participate in the native public-pseudo difference.
                    let final_difference = old_native
                        .map_or(ComputedStyleDifference::kDescendantAffecting, |old| {
                            ComputedStyle::ComputeDifference(old, final_style)
                        });
                    let equal = final_difference == ComputedStyleDifference::kEqual
                        // Generated style equality deliberately excludes these
                        // derived flags. A real page/preference update still
                        // publishes their newly computed native values.
                        && old_native.is_some_and(|old| {
                            old.DarkColorScheme() == final_style.DarkColorScheme()
                                && old.ColorSchemeForced() == final_style.ColorSchemeForced()
                                && old.ColorSchemeFlagsIsNormal() == final_style.ColorSchemeFlagsIsNormal()
                        })
                        // ::placeholder uses an internal PseudoId and is not
                        // covered by ComputedStyle's public-pseudo flag loop.
                        // Its form-control lifecycle still owns a distinct cache.
                        && old_native.is_some_and(|old| PseudoStyleKind::ALL.into_iter().all(|kind| {
                            old.GetCachedPseudoElementStyleWithoutArgument(kind.Id())
                                == final_style.GetCachedPseudoElementStyleWithoutArgument(kind.Id())
                        }))
                        && !subtree
                        && old == Some(&normalized);
                    child_explicit = !equal;
                    pending_values.insert(index, values);
                    pending_custom_properties.insert(index, computed_custom.clone());
                    custom_properties.insert(index, computed_custom);
                    // A root font change also invalidates rem users outside ordinary
                    // inheritance. Until unit dependency flags are retained, visit
                    // the connected document for that source-defined dependency.
                    if parent.is_none()
                        && old_native.is_some_and(|old| {
                            old.GetFontDescription().SpecifiedSize()
                                != final_style.GetFontDescription().SpecifiedSize()
                        })
                    {
                        child_subtree = true;
                    }
                    if equal {
                        // Keep pointer identity and existing Arc for equal styles.
                        let old = old.unwrap();
                        resolved.SetNativeStyle(old.native_style.clone());
                    }
                    native.insert(index, resolved.native_style.clone());
                    if old != Some(&resolved) {
                        stats.changed_nodes += 1;
                        impact.Merge(StyleImpact(old, &resolved));
                        pending.push((index, resolved));
                    }
                    next_parent = Some(index);
                }
            }
            for &child in node.Children().iter().rev() {
                if child_inherited || child_explicit || child_subtree || visit.contains(&child) {
                    stack.push((
                        child,
                        next_parent,
                        none,
                        child_inherited,
                        child_explicit,
                        child_subtree,
                    ))
                }
            }
        }
        drop(sheets);
        self.sheet_revision = Some(revision);
        self.active_sheets = active_sheets;
        self.environment = Some(environment);
        self.sheet_dependencies = sheet_dependencies;
        self.matches.extend(pending_matches);
        self.cascade_values.extend(pending_values);
        self.custom_properties.extend(pending_custom_properties);
        let document = owner.GetDocumentMut();
        for (index, style) in pending {
            document.SetResolvedStyle(index, style)
        }
        FinishUpdate(document, stats, impact);
        Ok(impact)
    }
    // Source's invalidation traversal is separate from the style resolver.
    // Relations widen candidates; retained matched-rule signatures decide which
    // candidates need a cascade. This includes losing a selector match.
    fn InvalidateRelations(
        &self,
        document: &Document,
        index: usize,
        dependencies: &SelectorDependencies,
        candidates: &mut HashSet<usize>,
    ) {
        let state = document.StyleState();
        if dependencies.descendant && state.descendant_sensitive_nodes.contains(&index) {
            AddSubtree(document, index, candidates);
        }
        if (dependencies.sibling || dependencies.positional)
            && state.sibling_sensitive_nodes.contains(&index)
        {
            if let Some(parent) = document.Node(index).Parent() {
                AddSubtree(document, parent, candidates);
            }
        }
        if dependencies.has {
            self.InvalidateHasAncestors(document, index, candidates);
        }
    }
    fn InvalidateHasAncestors(
        &self,
        document: &Document,
        index: usize,
        candidates: &mut HashSet<usize>,
    ) {
        let mut ancestor = Some(index);
        while let Some(index) = ancestor {
            // :has subjects can themselves be ancestors of a final target, or
            // precede it through a sibling relation. Exact matching filters it.
            AddSubtree(document, index, candidates);
            ancestor = document.Node(index).Parent();
        }
    }
    fn MatchingRules(
        &mut self,
        document: &Document,
        index: usize,
        media: &MediaValuesCachedData,
        sheets: &[(&CSSStyleSheet, CascadeOrigin)],
        native: Option<&HashMap<usize, Persistent<ComputedStyle>>>,
        custom: Option<&HashMap<usize, CustomProperties>>,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
    ) -> Vec<(usize, u32, u32)> {
        let values = DocumentMediaValues::new(document, media);
        let evaluator = MediaQueryEvaluator::<MediaQueryValueBackend>::ForMediaValues(&values);
        let mut matches = vec![];
        let mut order = 0;
        for &(sheet, _) in sheets {
            for rule in &sheet.rules {
                order += 1;
                if !ContainerConditionsMatch(
                    document,
                    index,
                    media,
                    &rule.container_conditions,
                    &self.container_states,
                    Some(
                        self.registered_container_properties
                            .as_deref()
                            .unwrap_or(&self.property_registry),
                    ),
                    &self.custom_properties,
                    native,
                    custom,
                    environment_variables,
                    &mut self.diagnostics,
                ) {
                    continue;
                }
                if rule
                    .media_conditions
                    .iter()
                    .all(|c| evaluator.Eval(&ParseMediaQuerySet(c)))
                {
                    for (target, pseudo) in std::iter::once(None)
                        .chain(PseudoStyleKind::ALL.into_iter().map(Some))
                        .enumerate()
                    {
                        if let Some((specificity, proximity)) = self.MatchingRuleSpecificity(
                            document,
                            index,
                            &rule.selector_text,
                            pseudo,
                            &rule.scope_conditions,
                            sheet.owner_node_id,
                        ) {
                            matches.push((order, specificity | ((target as u32) << 24), proximity));
                        }
                    }
                }
            }
        }
        matches
    }
    // cpp: HTMLStyleElement::ProcessStyleSheet; StyleEngine::UpdateActiveStyleSheets.
    // CSSOM owns the parsed sheets; owner-node replacement preserves source order.
    fn SynchronizeStyleSheets(&mut self, owner: &mut DOM) {
        let document = owner.GetDocument();
        let mut stack = vec![document.Root()];
        let mut sheets = vec![];
        while let Some(index) = stack.pop() {
            let node = document.Node(index);
            if node.IsHTMLElement("style") && owner.NeedsStyleSheetParsing(node.Id()) {
                fn text(document: &Document, index: usize, result: &mut String) {
                    let node = document.Node(index);
                    if node.Type() == DOMNodeType::kText {
                        result.push_str(node.Data())
                    }
                    for &child in node.Children() {
                        text(document, child, result)
                    }
                }
                let mut css = String::new();
                text(document, index, &mut css);
                let valid_type = node
                    .FindAttribute("type")
                    .is_none_or(|a| a.value.is_empty() || a.value.eq_ignore_ascii_case("text/css"));
                let mut sheet = crate::ParseCSS(if valid_type { &css } else { "" });
                sheet.owner_node_id = node.Id();
                if let Some(media) = node.FindAttribute("media") {
                    if !media.value.is_empty() {
                        for rule in &mut sheet.rules {
                            rule.media_conditions.push(media.value.clone())
                        }
                    }
                }
                sheets.push(sheet);
            }
            for &child in node.Children().iter().rev() {
                stack.push(child)
            }
        }
        for sheet in sheets {
            owner.GetDocumentMut().AppendStyleSheet(sheet)
        }
    }
    fn CollectRules(
        &mut self,
        document: &Document,
        index: usize,
        media: &MediaValuesCachedData,
        sheets: &[(&CSSStyleSheet, CascadeOrigin)],
        pseudo: Option<PseudoStyleKind>,
        native: Option<&HashMap<usize, Persistent<ComputedStyle>>>,
        custom: Option<&HashMap<usize, CustomProperties>>,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
    ) -> Result<Vec<MatchedRule>, DocumentStyleError> {
        let values = DocumentMediaValues::new(document, media);
        let evaluator = MediaQueryEvaluator::<MediaQueryValueBackend>::ForMediaValues(&values);
        let mut result = vec![];
        let mut layer_names = HashMap::<(u8, String), u16>::new();
        for &(sheet, origin) in sheets {
            for name in &sheet.layer_order {
                let next = layer_names.len() + 1;
                if next >= u16::MAX as usize {
                    return Err(DocumentStyleError::CascadeIndexOverflow);
                }
                layer_names
                    .entry((origin as u8, name.clone()))
                    .or_insert(next as u16);
            }
        }
        let mut order = 0;
        for &(sheet, origin) in sheets {
            for rule in &sheet.rules {
                order += 1;
                if !ContainerConditionsMatch(
                    document,
                    index,
                    media,
                    &rule.container_conditions,
                    &self.container_states,
                    Some(
                        self.registered_container_properties
                            .as_deref()
                            .unwrap_or(&self.property_registry),
                    ),
                    &self.custom_properties,
                    native,
                    custom,
                    environment_variables,
                    &mut self.diagnostics,
                ) {
                    continue;
                }
                if !rule
                    .media_conditions
                    .iter()
                    .all(|condition| evaluator.Eval(&ParseMediaQuerySet(condition)))
                {
                    continue;
                }
                let Some((specificity, proximity)) = self.MatchingRuleSpecificity(
                    document,
                    index,
                    &rule.selector_text,
                    pseudo,
                    &rule.scope_conditions,
                    sheet.owner_node_id,
                ) else {
                    continue;
                };
                let properties = Parse(
                    document,
                    index,
                    &rule.declaration_text,
                    if origin == CascadeOrigin::kUserAgent {
                        CSSParserMode::kUASheetMode
                    } else {
                        DocumentParserMode(document)
                    },
                    &mut self.diagnostics,
                );
                let layer = if rule.layer_name.is_empty() {
                    u16::MAX
                } else {
                    *layer_names
                        .get(&(origin as u8, rule.layer_name.clone()))
                        .ok_or(DocumentStyleError::Unsupported(
                            "CSSOM layer missing source order",
                        ))?
                };
                result.push(MatchedRule {
                    specificity,
                    proximity,
                    order,
                    origin,
                    inline: false,
                    layer,
                    properties,
                });
            }
        }
        // cpp: svg/svg_element.cc:574-592 and
        // svg/svg_transformable_element.cc:45-53. SVG presentation
        // attributes are author-cascade input below ordinary author rules.
        if pseudo.is_none() && document.Node(index).Namespace() == DOMNamespace::kSVG {
            let declarations = SVGPresentationAttributeDeclarations(document.Node(index));
            if !declarations.is_empty() {
                result.push(MatchedRule {
                    specificity: 0,
                    proximity: 0,
                    order: 0,
                    origin: CascadeOrigin::kAuthorPresentationalHint,
                    inline: false,
                    layer: u16::MAX,
                    properties: Parse(
                        document,
                        index,
                        &declarations,
                        CSSParserMode::kSVGAttributeMode,
                        &mut self.diagnostics,
                    ),
                });
            }
        }
        // cpp: ElementRuleCollector::SortMatchedRules and MatchResult::AddMatchedProperties.
        // cpp: element_rule_collector.h:69-83. Scope proximity breaks equal
        // specificity before source order; the source clamps it to 16 bits.
        result.sort_by_key(|r| {
            (
                r.origin as u8,
                r.layer,
                r.specificity,
                u16::MAX - r.proximity.min(u16::MAX as u32) as u16,
                r.order,
            )
        });
        if let Some(inline) = document
            .Node(index)
            .FindAttribute("style")
            .filter(|_| pseudo.is_none())
        {
            result.push(MatchedRule {
                specificity: u32::MAX,
                proximity: u32::MAX,
                order: usize::MAX,
                origin: CascadeOrigin::kAuthor,
                inline: true,
                layer: u16::MAX,
                properties: Parse(
                    document,
                    index,
                    &inline.value,
                    DocumentParserMode(document),
                    &mut self.diagnostics,
                ),
            })
        }
        if result.len() > u16::MAX as usize {
            return Err(DocumentStyleError::CascadeIndexOverflow);
        }
        Ok(result)
    }
    fn MatchesAnyStyleTarget(&mut self, document: &Document, index: usize, selector: &str) -> bool {
        std::iter::once(None)
            .chain(PseudoStyleKind::ALL.into_iter().map(Some))
            .any(|pseudo| {
                self.MatchingStyleSpecificity(document, index, selector, pseudo)
                    .is_some()
            })
    }
    fn MatchingStyleSpecificity(
        &mut self,
        document: &Document,
        index: usize,
        text: &str,
        pseudo: Option<PseudoStyleKind>,
    ) -> Option<u32> {
        self.MatchingRuleSpecificity(document, index, text, pseudo, &[], 0)
            .map(|m| m.0)
    }
    fn MatchingRuleSpecificity(
        &mut self,
        document: &Document,
        index: usize,
        text: &str,
        pseudo: Option<PseudoStyleKind>,
        scopes: &[cssom::CSSStyleScope],
        owner_node_id: u64,
    ) -> Option<(u32, u32)> {
        let quirks = document.GetCompatibilityMode() == CompatibilityMode::kQuirks;
        let key = (text.to_owned(), quirks, !scopes.is_empty());
        if !self.style_selectors.contains_key(&key) {
            let selectors =
                CSSSelectorList::AdoptSelectorVector(CSSSelectorParser::ParseSelectorWithOptions(
                    &foundation::String::from(text),
                    &SelectorParserContext { html: true, quirks },
                    if scopes.is_empty() {
                        crate::parser::css_nesting_type::CSSNestingType::kNone
                    } else {
                        crate::parser::css_nesting_type::CSSNestingType::kScope
                    },
                    &Default::default(),
                ));
            let mut compiled = vec![];
            for complex in selectors.ComplexSelectors() {
                let mut simple: Vec<_> = complex.SimpleSelectors().cloned().collect();
                let positions: Vec<_> = simple
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| s.Match() == MatchType::kPseudoElement)
                    .map(|(i, _)| i)
                    .collect();
                let kind = if positions.is_empty() {
                    None
                } else {
                    let position = positions[0];
                    let compound_end = simple
                        .iter()
                        .position(|s| s.Relation() != RelationType::kSubSelector)
                        .unwrap_or(simple.len() - 1);
                    let kind = match simple[position].GetPseudoType() {
                        PseudoType::kPseudoBefore => Some(PseudoStyleKind::Before),
                        PseudoType::kPseudoAfter => Some(PseudoStyleKind::After),
                        PseudoType::kPseudoFirstLetter => Some(PseudoStyleKind::FirstLetter),
                        PseudoType::kPseudoPlaceholder => Some(PseudoStyleKind::Placeholder),
                        _ => None,
                    };
                    if positions.len() != 1 || position != compound_end || kind.is_none() {
                        self.diagnostics.push(DocumentStyleError::Unsupported(
                            "pseudo style request or pseudo descendants",
                        ));
                        continue;
                    }
                    let removed = simple.remove(position);
                    if position > 0 {
                        simple[position - 1].SetRelation(removed.Relation());
                    }
                    kind
                };
                if simple.is_empty() {
                    simple = CSSSelectorParser::ParseSelector(
                        &foundation::String::from("*"),
                        &SelectorParserContext { html: true, quirks },
                    );
                }
                for s in &mut simple {
                    s.SetLastInComplexSelector(false);
                    s.SetLastInSelectorList(false);
                }
                simple.last_mut().unwrap().SetLastInComplexSelector(true);
                let compiled_subject = CSSSelectorList::AdoptSelectorVector(simple);
                let subject = compiled_subject.SelectorsText().Utf8();
                compiled.push(StyleSelector {
                    subject,
                    compiled_subject,
                    pseudo: kind,
                    specificity: complex.Specificity(),
                });
            }
            self.style_selectors.insert(key.clone(), compiled);
        }
        let mut specificity = None;
        for selector in self
            .style_selectors
            .get(&key)
            .unwrap()
            .iter()
            .filter(|s| s.pseudo == pseudo)
        {
            let matched = if scopes.is_empty() {
                self.selectors
                    .MatchingSpecificity(document, index, &selector.subject)
                    .map(|m| m.map(|_| u32::MAX))
            } else {
                self.selectors
                    .MatchingScopedSpecificity(
                        document,
                        index,
                        &selector.compiled_subject,
                        scopes,
                        owner_node_id,
                    )
                    .map(|m| m.map(|m| m.proximity))
            };
            match matched {
                Ok(Some(proximity)) => {
                    let candidate = (selector.specificity, proximity);
                    if specificity.is_none_or(|old: (u32, u32)| {
                        (candidate.0, std::cmp::Reverse(candidate.1))
                            > (old.0, std::cmp::Reverse(old.1))
                    }) {
                        specificity = Some(candidate);
                    }
                }
                Err(error) => self.diagnostics.push(DocumentStyleError::Selector(error)),
                _ => {}
            }
        }
        specificity
    }
    fn ResolvePseudoStyle(
        &mut self,
        document: &Document,
        index: usize,
        kind: PseudoStyleKind,
        originating: &ResolvedNodeStyle,
        originating_custom: &CustomProperties,
        initial: &ComputedStyle,
        root: f32,
        media: &MediaValuesCachedData,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
        rules: &[MatchedRule],
        has_layout_context: bool,
    ) -> Result<Option<PseudoElement>, DocumentStyleError> {
        let node = document.Node(index);
        let parent = unsafe { &*originating.native_style.Get() };
        if !has_layout_context
            || (kind != PseudoStyleKind::Placeholder && !parent.CanGeneratePseudoElement(kind.Id()))
        {
            return Ok(None);
        }
        if kind == PseudoStyleKind::Placeholder
            && !(node.IsHTMLElement("input") || node.IsHTMLElement("textarea"))
        {
            return Ok(None);
        }
        if rules.is_empty() {
            return Ok(None);
        }
        // Replaced elements have no ::before/::after child layout tree.
        if matches!(kind, PseudoStyleKind::Before | PseudoStyleKind::After)
            && [
                "input", "textarea", "img", "iframe", "video", "audio", "canvas", "select",
            ]
            .iter()
            .any(|name| node.IsHTMLElement(name))
        {
            self.diagnostics.push(DocumentStyleError::Unsupported(
                "generated pseudo on replaced/form element",
            ));
            return Ok(None);
        }
        let mut builder = ComputedStyleBuilder::from_initial_and_parent(
            initial,
            parent,
            IsAtShadowBoundary::kNotAtShadowBoundary,
        );
        builder.SetStyleType(kind.Id());
        Self::Cascade(
            document,
            index,
            rules,
            &mut builder,
            Some(parent),
            Some(originating_custom),
            root,
            media,
            environment_variables,
            &self.property_registry,
            self.color_scheme_settings,
            &mut self.diagnostics,
        )?;
        if builder.GetPosition() == foundation::EPosition::kAbsolute
            || builder.GetPosition() == foundation::EPosition::kFixed
            || builder.Floating() != foundation::EFloat::kNone
        {
            let display = builder.Display();
            builder.SetDisplay(BlockDisplay(display));
        }
        let mut style = Persistent::from_ptr(builder.TakeStyle() as *mut ComputedStyle);
        let native = unsafe { &*style.Get() };
        // cpp: Element::GetCachedPseudoElementStyle (element.cc:10376-10458).
        // Reuse is based on the complete native style, including content and
        // inherited fields, rather than the layout projection or cascade values.
        if let Some(old) = document.ResolvedStyleFor(index) {
            let cached = unsafe { &*old.native_style.Get() }
                .GetCachedPseudoElementStyleWithoutArgument(kind.Id());
            if unsafe { cached.as_ref() }.is_some_and(|old| {
                old == native && old.HasAttrFunction() == native.HasAttrFunction()
            }) {
                style = Persistent::from_ptr(cached as *mut ComputedStyle);
            }
        }
        let native = unsafe { &*style.Get() };
        // Retain computed style before box suppression. Chromium also retains
        // attr-dependent suppressed styles in SetAssociatedPseudoElement so
        // attribute changes invalidate their native cached style.
        parent.AddCachedPseudoElementStyle(style.Get(), kind.Id(), &foundation::g_null_atom);
        if native.Display() == EDisplay::kNone
            || (matches!(kind, PseudoStyleKind::Before | PseudoStyleKind::After)
                && native.ContentPreventsBoxGeneration())
        {
            return Ok(None);
        }
        let mut projection = ResolvedNodeStyle::default();
        ProjectForLayoutBoundary(native, &mut projection)?;
        // DOM/Layout consumes a projection of native ContentData; the native
        // content chain remains the computed value and cache-owned source.
        let mut text = String::new();
        let mut content = native.GetContentData();
        while let Some(data) = content {
            let data_ref = unsafe { &*data };
            if data_ref.IsText() {
                text.push_str(
                    &unsafe { &*(data as *const TextContentData) }
                        .GetText()
                        .Utf8(),
                );
            }
            content = data_ref.Next();
        }
        let mut pseudo = PseudoElement {
            style: projection.style,
            text,
            display_contents: native.Display() == EDisplay::kContents,
            ..Default::default()
        };
        pseudo.SetNativeStyle(style);
        Ok(Some(pseudo))
    }
    fn Cascade(
        document: &Document,
        node: usize,
        rules: &[MatchedRule],
        builder: &mut ComputedStyleBuilder,
        parent: Option<&ComputedStyle>,
        parent_custom: Option<&CustomProperties>,
        root: f32,
        media: &MediaValuesCachedData,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
        registry: &crate::property_registry::PropertyRegistry,
        color_scheme_settings: production_style_builder::ColorSchemeSettings,
        diagnostics: &mut Vec<DocumentStyleError>,
    ) -> Result<(HashMap<CSSPropertyID, Rc<Value>>, CustomProperties), DocumentStyleError> {
        let mut applied = HashMap::new();
        // cpp: style_resolver.cc:593-600,1767-1778. The document element has
        // a scheme-sensitive initial color at the lowest UA cascade priority.
        // Pseudo styles use this same node index with a parent style, so they
        // must keep normal color inheritance instead of resetting it here.
        let is_document_element = parent.is_none()
            && document
                .Node(document.Root())
                .Children()
                .iter()
                .copied()
                .find(|&index| document.Node(index).Type() == DOMNodeType::kElement)
                == Some(node);
        let custom_winners = WinningCustomProperties(rules)?;
        let environment = CascadeEnvironment::new(environment_variables);
        let attribute_source = PersistentAttributeSource::new(
            document,
            node,
            document.Node(node).Namespace() == DOMNamespace::kHTML,
        );
        let attribute_state = CascadeAttributeState::new();
        // RuntimeEnabledFeatures::CSSArgumentGrammar is "test" in the
        // corresponding Chromium source, so the stable document path keeps
        // its legacy attr() first-argument grammar. The typed resolver still
        // owns both branches and can be enabled by the host later.
        let attributes = AttributeResolver::new(&attribute_source, &attribute_state, false);
        let substitution = SubstitutionContext {
            environment: Some(&environment),
            attributes: Some(&attributes),
        };
        let compute_custom = |font: f32| {
            CustomProperties::ComputeWithRegistry(
                parent_custom,
                custom_winners.iter().copied(),
                substitution,
                Some(registry),
                &|n, u| {
                    production_style_builder::Pixels(
                        CSSPropertyID::kVariable,
                        n,
                        u,
                        font,
                        root,
                        media,
                    )
                    .map_err(|_| {
                        VariableResolutionError::Unsupported(
                            "registered property length conversion",
                        )
                    })
                },
            )
        };
        let mut custom = compute_custom(builder.GetFontDescription().ComputedSize());
        for (_, error) in &custom.diagnostics {
            if let VariableResolutionError::Unsupported(operation) = error {
                diagnostics.push(DocumentStyleError::Unsupported(operation));
            }
        }
        // cpp: style_cascade.cc:AnalyzeMatchResult/ApplyHighPriority/ApplyRemaining.
        // Direction-aware properties are mapped only after direction/writing-mode.
        for high in [true, false] {
            // Font-relative registered substitution data belongs to this
            // element's computed font, and remains absolute on inheritance.
            if !high && registry.Registrations().next().is_some() {
                custom = compute_custom(builder.GetFontDescription().ComputedSize());
            }
            let mut map = CascadeMap::new();
            let mut ids = HashSet::new();
            for (ri, rule) in rules.iter().enumerate() {
                if rule.properties.len() > u16::MAX as usize {
                    return Err(DocumentStyleError::CascadeIndexOverflow);
                }
                for (di, p) in rule.properties.iter().enumerate() {
                    let id = p.PropertyID();
                    if id == CSSPropertyID::kVariable {
                        continue;
                    }
                    let is_high = matches!(
                        id,
                        CSSPropertyID::kDirection
                            | CSSPropertyID::kWritingMode
                            | CSSPropertyID::kZoom
                            | CSSPropertyID::kFontSize
                            | CSSPropertyID::kFontWeight
                            | CSSPropertyID::kFontStyle
                            | CSSPropertyID::kFontFamily
                    );
                    if is_high != high {
                        continue;
                    }
                    let id = ResolvePhysical(id, builder.GetWritingDirection());
                    ids.insert(id);
                    map.Add(
                        id,
                        CascadePriority::FromParts(
                            rule.origin,
                            p.IsImportant(),
                            0,
                            rule.inline,
                            false,
                            false,
                            rule.layer,
                            ri as u16,
                            di as u16,
                        ),
                    );
                }
            }
            if !high && is_document_element {
                // ColorScheme sorts before Color. Schedule the native initial
                // color even without a declaration, then let any real winner
                // override it. No synthetic CSS value crosses a crate boundary.
                ids.insert(CSSPropertyID::kColor);
            }
            let mut ids = ids.into_iter().collect::<Vec<_>>();
            ids.sort_by_key(|id| match id {
                CSSPropertyID::kDirection => 0,
                CSSPropertyID::kWritingMode => 1,
                // cpp: style_cascade.cc:625-638 applies zoom before the
                // high-priority font properties because it changes their
                // conversion data and computed metrics.
                CSSPropertyID::kZoom => 2,
                CSSPropertyID::kFontFamily => 3,
                CSSPropertyID::kFontSize => 4,
                _ => *id as i32 + 10,
            });
            for id in ids {
                let name = CSSPropertyName::new(id);
                let mut priority = map.At(&name);
                if id == CSSPropertyID::kColor && is_document_element {
                    let color = builder.InitialColorForColorScheme();
                    builder.SetColor(&color);
                    builder.SetColorIsInherited(false);
                    builder.SetColorIsCurrentColor(false);
                    if !priority.HasOrigin() {
                        continue;
                    }
                }
                let mut value;
                let mut source_id;
                loop {
                    let source_property =
                        &rules[priority.GetRuleIndex()].properties[priority.GetDeclarationIndex()];
                    source_id = source_property.PropertyID();
                    value = source_property.ValueRef();
                    let fallback = if value.IsRevertValue() {
                        let origin = match priority.GetOrigin() {
                            CascadeOrigin::kAuthor | CascadeOrigin::kAuthorPresentationalHint => {
                                CascadeOrigin::kUser
                            }
                            CascadeOrigin::kUser => CascadeOrigin::kUserAgent,
                            _ => CascadeOrigin::kNone,
                        };
                        map.FindForOrigin(&name, origin)
                    } else if value.IsRevertLayerValue() {
                        map.FindRevertLayer(&name, priority.ForLayerComparison())
                    } else {
                        break;
                    };
                    if let Some(next) = fallback {
                        priority = *next
                    } else {
                        value = crate::production_css_value::wide(foundation::CSSValueID::kUnset)
                            .unwrap();
                        break;
                    }
                }
                let value =
                    match ResolvePropertyValueWithContext(source_id, &value, &custom, substitution)
                    {
                        Ok(value) => value,
                        Err(error) => {
                            diagnostics.push(DocumentStyleError::Property {
                                node,
                                property: error.property,
                                offset: error.offset,
                                operation: error.operation,
                                source_file: error.source_file,
                                source_line: error.source_line,
                            });
                            continue;
                        }
                    };
                if let Err(error) = production_style_builder::ApplyWithColorSchemeSettings(
                    id,
                    builder,
                    parent,
                    &value,
                    root,
                    media,
                    color_scheme_settings,
                ) {
                    diagnostics.push(DocumentStyleError::Application { node, error });
                } else {
                    applied.insert(id, value);
                }
            }
        }
        environment.ApplyState(builder);
        attribute_state.ApplyState(builder);
        Ok((applied, custom))
    }
}

// cpp: style_cascade.cc:AnalyzeMatchResult/ResolveCustomProperty.
// Select custom-property winners with the same origin/layer ordering as native
// properties before var() substitution computes the inherited custom map.
fn WinningCustomProperties<'a>(
    rules: &'a [MatchedRule],
) -> Result<Vec<&'a PropertyValue>, DocumentStyleError> {
    let mut map = CascadeMap::new();
    let mut names = HashSet::new();
    for (ri, rule) in rules.iter().enumerate() {
        if rule.properties.len() > u16::MAX as usize {
            return Err(DocumentStyleError::CascadeIndexOverflow);
        }
        for (di, property) in rule.properties.iter().enumerate() {
            if property.PropertyID() != CSSPropertyID::kVariable {
                continue;
            }
            let name = property.CustomPropertyName().clone();
            names.insert(name.clone());
            map.AddCustom(
                name,
                CascadePriority::FromParts(
                    rule.origin,
                    property.IsImportant(),
                    0,
                    rule.inline,
                    false,
                    false,
                    rule.layer,
                    ri as u16,
                    di as u16,
                ),
            );
        }
    }
    let mut names = names.into_iter().collect::<Vec<_>>();
    names.sort_by_key(|name| name.Utf8());
    let mut winners = Vec::with_capacity(names.len());
    for custom_name in names {
        let name = CSSPropertyName::custom(custom_name);
        let mut priority = map.At(&name);
        loop {
            let property =
                &rules[priority.GetRuleIndex()].properties[priority.GetDeclarationIndex()];
            let value = property.ValueRef();
            let fallback = if value.IsRevertValue() {
                let origin = match priority.GetOrigin() {
                    CascadeOrigin::kAuthor | CascadeOrigin::kAuthorPresentationalHint => {
                        CascadeOrigin::kUser
                    }
                    CascadeOrigin::kUser => CascadeOrigin::kUserAgent,
                    _ => CascadeOrigin::kNone,
                };
                map.FindForOrigin(&name, origin)
            } else if value.IsRevertLayerValue() {
                map.FindRevertLayer(&name, priority.ForLayerComparison())
            } else {
                winners.push(property);
                break;
            };
            let Some(next) = fallback else {
                // Custom properties inherit by default. With no lower origin,
                // revert resolves to the inherited/initial state represented by
                // the absence of a specified winner.
                break;
            };
            priority = *next;
        }
    }
    Ok(winners)
}

fn Parse(
    _document: &Document,
    node: usize,
    text: &str,
    mode: CSSParserMode,
    diagnostics: &mut Vec<DocumentStyleError>,
) -> Vec<PropertyValue> {
    let parsed = ParseDeclarationList(&foundation::String::from(text), mode);
    for e in parsed
        .errors
        .iter()
        .filter(|e| e.kind == PropertyParseErrorKind::Unsupported)
    {
        diagnostics.push(DocumentStyleError::Property {
            node,
            property: e.property,
            offset: e.offset,
            operation: e.operation,
            source_file: e.source_file,
            source_line: e.source_line,
        });
    }
    parsed.properties
}

fn SVGPresentationAttributeDeclarations(node: &dom::persistent_document::DOMNode) -> String {
    // Geometry-only SVG attributes remain owned by DOM/Layout metadata. These
    // names are the CSS-facing animated presentation properties collected by
    // SVGElement and its graphics-element subclasses in Chromium.
    const PROPERTIES: &[&str] = &[
        "alignment-baseline",
        "baseline-shift",
        "clip-path",
        "clip-rule",
        "color",
        "color-interpolation",
        "cursor",
        "display",
        "dominant-baseline",
        "fill",
        "fill-opacity",
        "fill-rule",
        "filter",
        "flood-color",
        "flood-opacity",
        "font-family",
        "font-size",
        "font-style",
        "font-weight",
        "image-rendering",
        "marker-end",
        "marker-mid",
        "marker-start",
        "mask",
        "opacity",
        "overflow",
        "paint-order",
        "pointer-events",
        "shape-rendering",
        "stop-color",
        "stop-opacity",
        "stroke",
        "stroke-dasharray",
        "stroke-dashoffset",
        "stroke-linecap",
        "stroke-linejoin",
        "stroke-miterlimit",
        "stroke-opacity",
        "stroke-width",
        "text-anchor",
        "text-decoration",
        "transform",
        "transform-origin",
        "vector-effect",
        "visibility",
        "white-space",
    ];
    let mut declarations = String::new();
    for attribute in node.Attributes() {
        if !attribute.namespace_uri.is_empty()
            || !PROPERTIES.contains(&attribute.local_name.as_str())
        {
            continue;
        }
        let value = if attribute.local_name == "transform" {
            NormalizeSVGTransform(&attribute.value)
        } else {
            attribute.value.clone()
        };
        declarations.push_str(&attribute.local_name);
        declarations.push(':');
        declarations.push_str(&value);
        declarations.push(';');
    }
    declarations
}

fn NormalizeSVGTransform(value: &str) -> String {
    // SVGTransformList has SVG-specific argument grammar (including optional
    // rotate centers). Chromium exposes the concatenated AffineTransform to
    // style. Lowering to matrix() preserves that exact boundary and avoids
    // pretending the attribute itself used CSS transform-function syntax.
    type Matrix = [f64; 6];
    fn multiply(left: Matrix, right: Matrix) -> Matrix {
        [
            left[0] * right[0] + left[2] * right[1],
            left[1] * right[0] + left[3] * right[1],
            left[0] * right[2] + left[2] * right[3],
            left[1] * right[2] + left[3] * right[3],
            left[0] * right[4] + left[2] * right[5] + left[4],
            left[1] * right[4] + left[3] * right[5] + left[5],
        ]
    }
    fn operation(name: &str, arguments: &[f64]) -> Option<Matrix> {
        match name.to_ascii_lowercase().as_str() {
            "matrix" if arguments.len() == 6 => arguments.try_into().ok(),
            "translate" if (1..=2).contains(&arguments.len()) => Some([
                1.0,
                0.0,
                0.0,
                1.0,
                arguments[0],
                arguments.get(1).copied().unwrap_or(0.0),
            ]),
            "scale" if (1..=2).contains(&arguments.len()) => Some([
                arguments[0],
                0.0,
                0.0,
                arguments.get(1).copied().unwrap_or(arguments[0]),
                0.0,
                0.0,
            ]),
            "rotate" if arguments.len() == 1 || arguments.len() == 3 => {
                let radians = arguments[0].to_radians();
                let rotation = [
                    radians.cos(),
                    radians.sin(),
                    -radians.sin(),
                    radians.cos(),
                    0.0,
                    0.0,
                ];
                if arguments.len() == 1 {
                    Some(rotation)
                } else {
                    let to_center = [1.0, 0.0, 0.0, 1.0, arguments[1], arguments[2]];
                    let from_center = [1.0, 0.0, 0.0, 1.0, -arguments[1], -arguments[2]];
                    Some(multiply(multiply(to_center, rotation), from_center))
                }
            }
            "skewx" if arguments.len() == 1 => {
                Some([1.0, 0.0, arguments[0].to_radians().tan(), 1.0, 0.0, 0.0])
            }
            "skewy" if arguments.len() == 1 => {
                Some([1.0, arguments[0].to_radians().tan(), 0.0, 1.0, 0.0, 0.0])
            }
            _ => None,
        }
    }

    let bytes = value.as_bytes();
    let mut cursor = 0;
    let mut result = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut count = 0;
    while cursor < bytes.len() {
        while cursor < bytes.len() && (bytes[cursor].is_ascii_whitespace() || bytes[cursor] == b',')
        {
            cursor += 1;
        }
        let name_start = cursor;
        while cursor < bytes.len() && bytes[cursor].is_ascii_alphabetic() {
            cursor += 1;
        }
        let name = &value[name_start..cursor];
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if name.is_empty() || bytes.get(cursor) != Some(&b'(') {
            return value.to_owned();
        }
        cursor += 1;
        let arguments_start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b')' {
            cursor += 1;
        }
        if cursor == bytes.len() {
            return value.to_owned();
        }
        let arguments: Option<Vec<f64>> = value[arguments_start..cursor]
            .split(|character: char| character == ',' || character.is_ascii_whitespace())
            .filter(|argument| !argument.is_empty())
            .map(|argument| argument.parse().ok())
            .collect();
        let Some(matrix) = arguments.and_then(|arguments| operation(name, &arguments)) else {
            return value.to_owned();
        };
        result = multiply(result, matrix);
        count += 1;
        cursor += 1;
    }
    if count == 0 {
        return value.to_owned();
    }
    format!(
        "matrix({},{},{},{},{},{})",
        result[0], result[1], result[2], result[3], result[4], result[5]
    )
}
// cpp: StyleAdjuster::EquivalentBlockDisplay; inline variants preserve their inner display.
fn BlockDisplay(display: EDisplay) -> EDisplay {
    match display {
        EDisplay::kInline | EDisplay::kInlineBlock => EDisplay::kBlock,
        EDisplay::kInlineFlex => EDisplay::kFlex,
        EDisplay::kInlineGrid => EDisplay::kGrid,
        EDisplay::kInlineTable => EDisplay::kTable,
        EDisplay::kInlineListItem => EDisplay::kListItem,
        EDisplay::kInlineFlowRootListItem => EDisplay::kFlowRootListItem,
        EDisplay::kRuby => EDisplay::kBlockRuby,
        EDisplay::kMath => EDisplay::kBlockMath,
        EDisplay::kInlineGridLanes => EDisplay::kGridLanes,
        EDisplay::kInlineLayoutCustom => EDisplay::kLayoutCustom,
        _ => display,
    }
}
// The layout boundary rebinds the native font to ConstraintSpace's font faces.
// It must receive the computed description from the same native style.
fn ProjectForLayoutBoundary(
    native: &ComputedStyle,
    resolved: &mut ResolvedNodeStyle,
) -> Result<(), DocumentStyleError> {
    let font = native.GetFontDescription();
    let mut ext = layoutng::internal::layout_input::ExtendedStyle::default();
    use layoutng::internal::layout_input::{Display, FloatSide, Position, WhiteSpace};
    resolved.style.display = match native.Display() {
        EDisplay::kInline => Display::kInline,
        EDisplay::kBlock => Display::kBlock,
        EDisplay::kInlineBlock => Display::kInlineBlock,
        EDisplay::kListItem
        | EDisplay::kInlineListItem
        | EDisplay::kFlowRootListItem
        | EDisplay::kInlineFlowRootListItem => Display::kListItem,
        EDisplay::kFlex | EDisplay::kWebkitBox => Display::kFlex,
        EDisplay::kInlineFlex | EDisplay::kWebkitInlineBox => Display::kInlineFlex,
        EDisplay::kGrid => Display::kGrid,
        EDisplay::kInlineGrid => Display::kInlineGrid,
        EDisplay::kTable => Display::kTable,
        EDisplay::kInlineTable => Display::kInlineTable,
        EDisplay::kTableRowGroup => Display::kTableSection,
        EDisplay::kTableHeaderGroup => Display::kTableHeaderGroup,
        EDisplay::kTableFooterGroup => Display::kTableFooterGroup,
        EDisplay::kTableRow => Display::kTableRow,
        EDisplay::kTableColumn => Display::kTableColumn,
        EDisplay::kTableColumnGroup => Display::kTableColumnGroup,
        EDisplay::kTableCell => Display::kTableCell,
        EDisplay::kTableCaption => Display::kTableCaption,
        EDisplay::kFlowRoot => Display::kFlowRoot,
        EDisplay::kMath => Display::kMath,
        EDisplay::kBlockMath => Display::kBlockMath,
        EDisplay::kRuby => Display::kRuby,
        EDisplay::kBlockRuby => Display::kBlockRuby,
        EDisplay::kRubyText => Display::kRubyText,
        EDisplay::kGridLanes | EDisplay::kInlineGridLanes => Display::kGridLanes,
        EDisplay::kLayoutCustom | EDisplay::kInlineLayoutCustom => Display::kCustom,
        // No LayoutObject is constructed for either value; own_generates_box
        // and display_contents preserve their structural distinction.
        EDisplay::kNone | EDisplay::kContents => Display::kInline,
    };
    resolved.style.position = match native.GetPosition() {
        foundation::EPosition::kStatic => Position::kStatic,
        foundation::EPosition::kRelative => Position::kRelative,
        foundation::EPosition::kAbsolute => Position::kAbsolute,
        foundation::EPosition::kFixed => Position::kFixed,
        foundation::EPosition::kSticky => Position::kSticky,
    };
    resolved.style.floating = match native.Floating() {
        foundation::EFloat::kNone => FloatSide::kNone,
        foundation::EFloat::kLeft => FloatSide::kLeft,
        foundation::EFloat::kRight => FloatSide::kRight,
        foundation::EFloat::kInlineStart => FloatSide::kInlineStart,
        foundation::EFloat::kInlineEnd => FloatSide::kInlineEnd,
    };
    use layoutng_style::css::white_space::EWhiteSpace;
    ext.white_space = match native.WhiteSpace() {
        EWhiteSpace::kNormal => WhiteSpace::kNormal,
        EWhiteSpace::kNowrap => WhiteSpace::kNowrap,
        EWhiteSpace::kPre => WhiteSpace::kPre,
        EWhiteSpace::kPreLine => WhiteSpace::kPreLine,
        EWhiteSpace::kPreWrap => WhiteSpace::kPreWrap,
        EWhiteSpace::kBreakSpaces => WhiteSpace::kBreakSpaces,
        value if value.bits() == 2 => WhiteSpace::kPreserveBreaksNowrap,
        value if value.bits() == 3 => WhiteSpace::kBreakSpacesNowrap,
        _ => unreachable!("white-space collapse/wrap fields exceeded their bit widths"),
    };

    ext.font_size = font.ComputedSize() as f64;
    ext.font_weight = font.Weight().ToFloat() as f64;
    ext.font_italic = font.Style().ToFloat() != 0.0;
    // cpp: font_description_data.cc:127-145. The computed CSS value keeps a
    // Length/percentage, while Layout consumes the used spacing resolved from
    // the current computed font size.
    ext.letter_spacing = font.LetterSpacing() as f64;
    ext.word_spacing = font.WordSpacing() as f64;
    ext.effective_zoom = native.EffectiveZoom();
    ext.zoom = native.Zoom();
    let mut family = font.Family() as *const font_engine::FontFamily;
    while !family.is_null() {
        let f = unsafe { &*family };
        if !f.FamilyName().empty() {
            ext.font_families.push(f.FamilyName().Utf8())
        }
        family = f.Next()
    }

    // cpp: ComputedStyle::GetCurrentColor/BorderWidth; StyleColor::Resolve.
    // Paint's neutral projection shares these scalar results with the native
    // object used by layout; complex paint payloads stay in their own mapping.
    let current = native.GetCurrentColor(None);
    let color=|value:&layoutng_style::css::style_color::StyleColor|->Result<layoutng::internal::layout_input_types::Color,DocumentStyleError>{
        let value=value.Resolve(current,native.UsedColorScheme(),None);
        let scale=match value.GetColorSpace(){foundation::color::ColorSpace::kSRGBLegacy=>1.0/255.0,foundation::color::ColorSpace::kSRGB=>1.0,_=>return Err(DocumentStyleError::Unsupported("paint projection color-space conversion"))};
        Ok(layoutng::internal::layout_input_types::Color{red:value.Param0()*scale,green:value.Param1()*scale,blue:value.Param2()*scale,alpha:value.Alpha()})
    };
    resolved.style.paint.color = color(native.Color())?;
    resolved.style.paint.background_color = color(native.BackgroundColor())?;
    resolved.style.paint.border_colors = [
        color(native.BorderTopColor())?,
        color(native.BorderRightColor())?,
        color(native.BorderBottomColor())?,
        color(native.BorderLeftColor())?,
    ];
    resolved.style.paint.outline_color = color(native.OutlineColor())?;
    resolved.style.paint.outline_width = if native.OutlineStyle() == foundation::EBorderStyle::kNone
    {
        0.0
    } else {
        *native.OutlineWidth() as f64
    };
    resolved.style.paint.outline_offset = *native.OutlineOffset() as f64;
    resolved.style.paint.opacity = native.Opacity();
    resolved.style.paint.visible = native.Visibility() == foundation::EVisibility::kVisible;
    resolved.style.paint.pointer_events_none =
        native.PointerEvents() == foundation::EPointerEvents::kNone;
    resolved.style.paint.cursor = native.Cursor();
    resolved.style.paint.z_index = (!native.HasAutoZIndex()).then(|| native.ZIndex());
    resolved.style.border.top = native.BorderTopWidth() as f64;
    resolved.style.border.right = native.BorderRightWidth() as f64;
    resolved.style.border.bottom = native.BorderBottomWidth() as f64;
    resolved.style.border.left = native.BorderLeftWidth() as f64;
    fn border(
        style: foundation::EBorderStyle,
    ) -> layoutng::internal::layout_input::BorderLineStyle {
        use foundation::EBorderStyle as N;
        use layoutng::internal::layout_input::BorderLineStyle as P;
        match style {
            N::kNone => P::kNone,
            N::kHidden => P::kHidden,
            N::kSolid => P::kSolid,
            N::kDashed => P::kDashed,
            N::kDotted => P::kDotted,
            N::kDouble => P::kDouble,
            N::kGroove => P::kGroove,
            N::kRidge => P::kRidge,
            N::kInset => P::kInset,
            N::kOutset => P::kOutset,
        }
    }
    resolved.style.border_styles = [
        border(native.BorderTopStyle()),
        border(native.BorderRightStyle()),
        border(native.BorderBottomStyle()),
        border(native.BorderLeftStyle()),
    ];
    resolved.style.paint.outline_style = border(native.OutlineStyle());
    crate::resolver::layout_projection::ProjectGeometry(native, resolved, &mut ext);
    resolved.style.extended = Some(ext);
    Ok(())
}

fn DocumentParserMode(document: &Document) -> CSSParserMode {
    if document.GetCompatibilityMode() == dom::persistent_document::CompatibilityMode::kQuirks {
        CSSParserMode::kHTMLQuirksMode
    } else {
        CSSParserMode::kHTMLStandardMode
    }
}

#[derive(Clone, Default)]
struct SelectorDependencies {
    descendant: bool,
    sibling: bool,
    has: bool,
    positional: bool,
    reverse_positional: bool,
    last: bool,
    inline_selector: bool,
}
impl SelectorDependencies {
    fn Merge(&mut self, other: &Self) {
        self.descendant |= other.descendant;
        self.sibling |= other.sibling;
        self.has |= other.has;
        self.positional |= other.positional;
        self.reverse_positional |= other.reverse_positional;
        self.last |= other.last;
        self.inline_selector |= other.inline_selector;
    }

    fn ForSheets(document: &Document, sheets: &[(CSSStyleSheet, CascadeOrigin)]) -> Self {
        let context = SelectorParserContext {
            html: true,
            quirks: document.GetCompatibilityMode()
                == dom::persistent_document::CompatibilityMode::kQuirks,
        };
        let mut result = Self::default();
        for (sheet, _) in sheets {
            for rule in &sheet.rules {
                for selector in CSSSelectorParser::ParseSelector(
                    &foundation::String::from(rule.selector_text.as_str()),
                    &context,
                ) {
                    result.Add(&selector);
                }
            }
        }
        result
    }
    fn Add(&mut self, selector: &CSSSelector) {
        match selector.Relation() {
            RelationType::kDescendant
            | RelationType::kChild
            | RelationType::kRelativeDescendant
            | RelationType::kRelativeChild => self.descendant = true,
            RelationType::kDirectAdjacent
            | RelationType::kIndirectAdjacent
            | RelationType::kRelativeDirectAdjacent
            | RelationType::kRelativeIndirectAdjacent => self.sibling = true,
            _ => {}
        }
        use PseudoType::*;
        match selector.GetPseudoType() {
            kPseudoHas => self.has = true,
            kPseudoNthLastChild | kPseudoNthLastOfType => {
                self.positional = true;
                self.reverse_positional = true;
            }
            kPseudoLastChild | kPseudoLastOfType | kPseudoOnlyChild | kPseudoOnlyOfType => {
                self.positional = true;
                self.last = true;
            }
            kPseudoFirstChild | kPseudoFirstOfType | kPseudoNthChild | kPseudoNthOfType => {
                self.positional = true
            }
            // Form container state depends on descendants too.
            kPseudoHasDatalist => self.has = true,
            kPseudoLang | kPseudoDisabled | kPseudoEnabled => self.descendant = true,
            _ => {}
        }
        if selector.IsAttributeSelector()
            && selector
                .Attribute()
                .LocalName()
                .Utf8()
                .eq_ignore_ascii_case("style")
        {
            self.inline_selector = true;
        }
        if let Some(list) = selector.SelectorList() {
            for complex in list.ComplexSelectors() {
                for selector in complex.SimpleSelectors() {
                    self.Add(selector);
                }
            }
        }
    }
}
fn ConnectedElement(document: &Document, index: usize) -> bool {
    if index >= document.NodeCount() || document.Node(index).Type() != DOMNodeType::kElement {
        return false;
    }
    let mut root = index;
    while let Some(parent) = document.Node(root).Parent() {
        root = parent;
    }
    root == document.Root()
}
fn AddSubtree(document: &Document, root: usize, targets: &mut HashSet<usize>) {
    let mut stack = vec![root];
    while let Some(index) = stack.pop() {
        let node = document.Node(index);
        if node.Type() == DOMNodeType::kElement {
            targets.insert(index);
        }
        stack.extend_from_slice(node.Children());
    }
}
fn ChangedSheets(
    old: &[(CSSStyleSheet, CascadeOrigin)],
    new: &[(CSSStyleSheet, CascadeOrigin)],
) -> Vec<usize> {
    if std::ptr::eq(old, new) {
        return vec![];
    }
    let layers_changed = old
        .iter()
        .flat_map(|(s, o)| s.layer_order.iter().map(move |name| (o, name)))
        .collect::<Vec<_>>()
        != new
            .iter()
            .flat_map(|(s, o)| s.layer_order.iter().map(move |name| (o, name)))
            .collect::<Vec<_>>();
    (0..old.len().max(new.len()))
        .filter(|&i| layers_changed || old.get(i) != new.get(i))
        .collect()
}
fn StyleImpact(old: Option<&ResolvedNodeStyle>, new: &ResolvedNodeStyle) -> StyleUpdateImpact {
    let Some(old) = old else {
        return StyleUpdateImpact::TREE;
    };
    let mut pseudo_impact = StyleUpdateImpact::default();
    for (old, new) in [
        (&old.before, &new.before),
        (&old.after, &new.after),
        (&old.first_letter, &new.first_letter),
        (&old.placeholder, &new.placeholder),
    ] {
        match (old, new) {
            (None, None) => {}
            (Some(old), Some(new))
                if old.text == new.text
                    && old.display_contents == new.display_contents
                    && old.style.display == new.style.display
                    && old.style.position == new.style.position
                    && old.style.floating == new.style.floating =>
            {
                let fields = ComputedStyleBase::FieldInvalidationDiff(
                    unsafe { &*old.native_style.Get() },
                    unsafe { &*new.native_style.Get() },
                );
                let layout = fields & FieldDifference::kLayout.bits() != 0
                    || !old.style.LayoutEquivalent(&new.style);
                pseudo_impact.layout |= layout;
                pseudo_impact.paint |= layout || fields != 0 || old.style.paint != new.style.paint;
            }
            _ => return StyleUpdateImpact::TREE,
        }
    }
    if old.generates_box != new.generates_box
        || old.own_generates_box != new.own_generates_box
        || old.own_display_contents != new.own_display_contents
        || old.style.display != new.style.display
        || old.style.position != new.style.position
        || old.style.floating != new.style.floating
    {
        return StyleUpdateImpact::TREE;
    }
    let old_native = unsafe { &*old.native_style.Get() };
    let new_native = unsafe { &*new.native_style.Get() };
    let native_difference = ComputedStyle::ComputeDifference(old_native, new_native);
    let fields = ComputedStyleBase::FieldInvalidationDiff(old_native, new_native);
    let layout =
        fields & FieldDifference::kLayout.bits() != 0 || !old.style.LayoutEquivalent(&new.style);
    StyleUpdateImpact {
        reattach: false,
        layout: pseudo_impact.layout || layout,
        paint: pseudo_impact.paint
            || native_difference != ComputedStyleDifference::kEqual
            || fields != 0
            || old.style.paint != new.style.paint
            || layout,
    }
}

fn CustomPropertiesEquivalent(a: &CustomProperties, b: &CustomProperties) -> bool {
    if a.registered_values.len() != b.registered_values.len()
        || !a.registered_values.iter().all(|(name, value)| {
            b.registered_values
                .get(name)
                .is_some_and(|other| value.as_ref() == other.as_ref())
        })
    {
        return false;
    }
    if a.values.len() != b.values.len() {
        return false;
    }
    a.values.iter().all(|(name, left)| {
        let Some(right) = b.values.get(name) else {
            return false;
        };
        match (left, right) {
            (ComputedVariableValue::Invalid, ComputedVariableValue::Invalid)
            | (ComputedVariableValue::Cyclic, ComputedVariableValue::Cyclic) => true,
            (ComputedVariableValue::Data(left), ComputedVariableValue::Data(right)) => {
                Rc::ptr_eq(left, right)
                    || left.original_text == right.original_text
                        && left.features == right.features
                        && left.is_animation_tainted == right.is_animation_tainted
                        && left.is_attr_tainted == right.is_attr_tainted
            }
            _ => false,
        }
    })
}

fn FinishUpdate(document: &mut Document, stats: StyleUpdateStats, impact: StyleUpdateImpact) {
    let state = document.StyleStateMut();
    state.all_dirty = false;
    state.changes.clear();
    state.sibling_sensitive_nodes.clear();
    state.descendant_sensitive_nodes.clear();
    state.append_only_children.clear();
    state.non_append_children.clear();
    state.inserted_style_subtrees.clear();
    state.append_affected_subtrees.clear();
    state.stats = stats;
    if impact.layout {
        state.measurement_geometry_revision = state.measurement_geometry_revision.wrapping_add(1);
    }
    state.impact.Merge(impact);
}

// cpp: ElementRuleCollector::MatchContainerQuerySet; ContainerSelector::FindContainer.
// An immutable typed projection is evaluated against the closest eligible live
// ancestor. Current traversal values take precedence over last-commit caches.
fn ContainerConditionsMatch(
    document: &Document,
    subject: usize,
    media: &MediaValuesCachedData,
    conditions: &[cssom::CSSContainerQuerySet],
    states: &HashMap<usize, ContainerState>,
    registered: Option<&dyn RegisteredContainerPropertyResolver>,
    stored_custom: &HashMap<usize, CustomProperties>,
    native: Option<&HashMap<usize, Persistent<ComputedStyle>>>,
    current_custom: Option<&HashMap<usize, CustomProperties>>,
    environment_variables: Option<&dyn EnvironmentVariableResolver>,
    diagnostics: &mut Vec<DocumentStyleError>,
) -> bool {
    use crate::kleene_value::{KleeneOr, KleeneValue};
    conditions.iter().all(|set| {
        let mut result = KleeneValue::kFalse;
        for query in &set.queries {
            let mut ancestor = document.Node(subject).Parent();
            let mut matched = KleeneValue::kFalse;
            while let Some(index) = ancestor {
                let node = document.Node(index);
                ancestor = node.Parent();
                if node.Type() != DOMNodeType::kElement {
                    continue;
                }
                let style = native
                    .and_then(|n| n.get(&index))
                    .and_then(|s| unsafe { s.Get().as_ref() })
                    .or_else(|| {
                        document
                            .ResolvedStyleFor(index)
                            .and_then(|s| unsafe { s.native_style.Get().as_ref() })
                    });
                let Some(style) = style else { continue };
                if !query.name.IsNull() && !query.name.Utf8().is_empty() {
                    let names = unsafe { style.ContainerName().Get().as_ref() };
                    if !names.is_some_and(|list| {
                        list.GetNames().iter().any(|name| {
                            unsafe { name.Get().as_ref() }
                                .is_some_and(|name| name.GetName() == &query.name)
                        })
                    }) {
                        continue;
                    }
                }
                let required = query
                    .condition
                    .as_ref()
                    .map_or(0, |c| RequiredContainerType(c, style));
                if style.ContainerType() & required != required {
                    continue;
                }
                let Some(condition) = &query.condition else {
                    matched = KleeneValue::kTrue;
                    break;
                };
                let empty = CustomProperties::default();
                let custom = current_custom
                    .and_then(|c| c.get(&index))
                    .or_else(|| stored_custom.get(&index))
                    .unwrap_or(&empty);
                let values = DocumentMediaValues::ForContainer(
                    document,
                    index,
                    media,
                    style,
                    states.get(&index),
                );
                let mut root_index = index;
                let mut parent = node.Parent();
                while let Some(index) = parent {
                    parent = document.Node(index).Parent();
                    if document.Node(index).Type() == DOMNodeType::kElement {
                        root_index = index;
                    }
                }
                let root = native
                    .and_then(|n| n.get(&root_index))
                    .and_then(|s| unsafe { s.Get().as_ref() })
                    .or_else(|| {
                        document
                            .ResolvedStyleFor(root_index)
                            .and_then(|s| unsafe { s.native_style.Get().as_ref() })
                    })
                    .unwrap_or(style);
                let values = values.WithRootStyle(root);
                let environment = CascadeEnvironment::new(environment_variables);
                let attribute_source = PersistentAttributeSource::new(
                    document,
                    index,
                    node.Namespace() == DOMNamespace::kHTML,
                );
                let attribute_state = CascadeAttributeState::new();
                let attributes = AttributeResolver::new(&attribute_source, &attribute_state, false);
                let unsupported = Cell::new(None);
                let parent_custom = node.Parent().and_then(|parent| {
                    current_custom
                        .and_then(|c| c.get(&parent))
                        .or_else(|| stored_custom.get(&parent))
                });
                let resolver = ContainerStyleResolver {
                    node: index,
                    values: &values,
                    custom,
                    parent_custom,
                    substitution: SubstitutionContext {
                        environment: Some(&environment),
                        attributes: Some(&attributes),
                    },
                    registered,
                    unsupported: &unsupported,
                };
                matched = EvaluateContainerCondition(condition, &resolver);
                if let Some(operation) = unsupported.get() {
                    diagnostics.push(DocumentStyleError::Unsupported(operation))
                }
                break;
            }
            result = KleeneOr(result, matched);
            if result == KleeneValue::kTrue {
                break;
            }
        }
        result == KleeneValue::kTrue
    })
}
