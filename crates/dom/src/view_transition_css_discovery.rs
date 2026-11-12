//! Original ViewTransitionStyleTracker CSS discovery, cc:635-800,890-982.
//! The cursor borrows native paint order and the canonical ComputedStyle.
use super::*;
use layoutng_style::style::{
    computed_style::ComputedStyle,
    computed_style_constants::{IsTransitionPseudoElement, PseudoId},
    style_view_transition_name::StyleViewTransitionNameType,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewTransitionCSSDiscoveryError {
    LifecycleNotReady,
    MissingPaintOrder,
    MissingElement(u64),
    MissingComputedStyle(u64),
    UnboundCompiledTreeScope,
    UnboundPseudoAncestry,
    RandomnessUnavailable,
}
pub struct ViewTransitionCSSLayerFacts<'a> {
    pub node: Option<u64>,
    pub style: &'a ComputedStyle,
    pub fragmented: bool,
    pub child_paint_blocked: bool,
}
pub trait ViewTransitionCSSPaintOrder {
    type Layer: Copy;
    fn Root(&self, scope: Option<u64>) -> Option<Self::Layer>;
    fn Facts(&self, layer: Self::Layer) -> ViewTransitionCSSLayerFacts<'_>;
    fn Children(
        &self,
        layer: Self::Layer,
    ) -> Result<Box<dyn Iterator<Item = Self::Layer> + '_>, ViewTransitionCSSDiscoveryError>;
}
/// base::Token::CreateRandom's only platform input. Token formatting, reuse by
/// ID and per-element stable names stay in the source tracker.
pub trait ViewTransitionRandomToken {
    fn CreateRandom(&self) -> Result<[u64; 2], ViewTransitionCSSDiscoveryError>;
}
fn TokenString(token: [u64; 2]) -> String {
    format!("{:016X}{:016X}", token[0], token[1])
}

impl ViewTransitionStyleTracker {
    fn GenerateAutoName(
        &mut self,
        document: &PersistentDocument,
        node: usize,
        scope: usize,
        allow_from_id: bool,
        random: &dyn ViewTransitionRandomToken,
    ) -> Result<String, ViewTransitionCSSDiscoveryError> {
        if allow_from_id && document.TreeScopeRoot(node) == scope {
            if let Some(id) = document.Node(node).FindAttribute("id") {
                if let Some(name) = self.id_to_auto_name.get(&id.value) {
                    return Ok(name.clone());
                }
                let name = format!("-ua-auto-{}", TokenString(random.CreateRandom()?));
                self.id_to_auto_name.insert(id.value.clone(), name.clone());
                return Ok(name);
            }
        }
        if self.auto_name_token.is_none() {
            self.auto_name_token = Some(random.CreateRandom()?);
        }
        Ok(format!(
            "-ua-auto-{}-{}",
            TokenString(self.auto_name_token.unwrap()),
            document.Node(node).Id()
        ))
    }
}
impl PersistentDocument {
    fn HasViewTransitionContainmentBoundary(
        &self,
        root: Option<u64>,
        child: Option<u64>,
        pseudo: PseudoId,
        scope: Option<DOMOwnerHandle>,
    ) -> Result<bool, ViewTransitionCSSDiscoveryError> {
        if child.is_none() {
            return Ok(false);
        }
        if IsTransitionPseudoElement(pseudo) || pseudo == PseudoId::kPseudoIdFirstLetter {
            return Ok(true);
        }
        if pseudo == PseudoId::kPseudoIdScrollMarker {
            return Err(ViewTransitionCSSDiscoveryError::UnboundPseudoAncestry);
        }
        let mut node = child
            .and_then(|id| self.FindNodeById(id))
            .ok_or(ViewTransitionCSSDiscoveryError::UnboundPseudoAncestry)?;
        let root = match root {
            Some(id) => Some(
                self.FindNodeById(id)
                    .ok_or(ViewTransitionCSSDiscoveryError::UnboundPseudoAncestry)?,
            ),
            None => None,
        };
        while Some(node) != root {
            if self.Node(node).Type() == DOMNodeType::kElement {
                let style = self
                    .ResolvedStyleFor(node)
                    .and_then(|style| unsafe { style.computed_style_owner.Get().as_ref() })
                    .ok_or(ViewTransitionCSSDiscoveryError::MissingComputedStyle(
                        self.Node(node).Id(),
                    ))?;
                if Some(self.NodeHandle(node)) != scope
                    && style.ViewTransitionScope() == foundation::EViewTransitionScope::kAll
                {
                    return Ok(true);
                }
                if IsTransitionPseudoElement(style.StyleType())
                    || style.StyleType() == PseudoId::kPseudoIdFirstLetter
                {
                    return Ok(true);
                }
            }
            let Some(parent) = self.FlatTreeParent(node) else {
                debug_assert!(
                    root.is_none(),
                    "source flat ancestry must reach the containing paint layer"
                );
                break;
            };
            node = parent;
        }
        Ok(false)
    }
    /// The original source's discovery commit. This only fills pending names;
    /// participants become active at Capture/Start, never from the CSS value.
    pub fn AddViewTransitionElementsFromCSS<S: ViewTransitionCSSPaintOrder>(
        &self,
        transition: &ViewTransition,
        order: &S,
        random: &dyn ViewTransitionRandomToken,
    ) -> Result<(), ViewTransitionCSSDiscoveryError> {
        use crate::document_lifecycle::LifecycleState;
        if self.Lifecycle().GetState() < LifecycleState::CompositingInputsClean {
            return Err(ViewTransitionCSSDiscoveryError::LifecycleNotReady);
        }
        assert_eq!(transition.document, self.RootHandle());
        let scope_node = transition
            .tracker_element
            .and_then(|scope| self.FindNodeById(scope.Id()));
        let document_scope = scope_node.is_none() || scope_node == self.DocumentElement();
        let tree_scope = if document_scope {
            self.Root()
        } else {
            self.TreeScopeRoot(scope_node.unwrap())
        };
        let mut tracker = transition.tracker.borrow_mut();
        if let Some(scope) = scope_node.filter(|_| !document_scope) {
            if let Some(style) = self
                .ResolvedStyleFor(scope)
                .and_then(|style| unsafe { style.computed_style_owner.Get().as_ref() })
            {
                if style.OverflowX() != foundation::EOverflow::kVisible
                    || style.OverflowY() != foundation::EOverflow::kVisible
                {
                    tracker.apply_overflow_clip = true;
                }
            }
        }
        let Some(root) = order.Root(
            scope_node
                .filter(|_| !document_scope)
                .map(|node| self.Node(node).Id()),
        ) else {
            return Ok(());
        };
        fn Visit<S: ViewTransitionCSSPaintOrder>(
            document: &PersistentDocument,
            transition: &ViewTransition,
            tracker: &mut ViewTransitionStyleTracker,
            order: &S,
            layer: S::Layer,
            scope: usize,
            stack: &mut Vec<String>,
            nearest_contain: Option<String>,
            random: &dyn ViewTransitionRandomToken,
        ) -> Result<(), ViewTransitionCSSDiscoveryError> {
            let facts = order.Facts(layer);
            let name = unsafe { facts.style.ViewTransitionName().Get().as_ref() };
            let mut current_name = None;
            if let Some(name) = name.filter(|_| !facts.fragmented) {
                // Non-null compiled scopes need the actual Style/DOM owner
                // bridge. The present production style domain uses null; no
                // pointer-to-DOM-id inference or reparsing is permitted here.
                if !name.GetTreeScope().is_null() {
                    return Err(ViewTransitionCSSDiscoveryError::UnboundCompiledTreeScope);
                }
                let id = facts
                    .node
                    .ok_or(ViewTransitionCSSDiscoveryError::UnboundPseudoAncestry)?;
                let node = document
                    .FindNodeById(id)
                    .filter(|node| document.Node(*node).Type() == DOMNodeType::kElement)
                    .ok_or(ViewTransitionCSSDiscoveryError::MissingElement(id))?;
                let generated = match name.GetType() {
                    StyleViewTransitionNameType::kAuto => {
                        tracker.GenerateAutoName(document, node, scope, true, random)?
                    }
                    StyleViewTransitionNameType::kMatchElement => {
                        tracker.GenerateAutoName(document, node, scope, false, random)?
                    }
                    StyleViewTransitionNameType::kCustom => name.CustomName().Utf8(),
                };
                tracker.AddTransitionElementWithGroups(
                    document.NodeHandle(node),
                    generated.clone(),
                    stack.last().cloned(),
                    nearest_contain.clone(),
                );
                current_name = Some(generated);
            }
            if facts.child_paint_blocked {
                return Ok(());
            }
            if let Some(name) = &current_name {
                stack.push(name.clone());
            }
            for child in order.Children(layer)? {
                let child_facts = order.Facts(child);
                if document.HasViewTransitionContainmentBoundary(
                    facts.node,
                    child_facts.node,
                    child_facts.style.StyleType(),
                    transition.tracker_element,
                )? {
                    continue;
                }
                let next_contain = if facts.style.ViewTransitionGroup().IsNormal() {
                    nearest_contain.clone()
                } else {
                    current_name.clone()
                };
                Visit(
                    document,
                    transition,
                    tracker,
                    order,
                    child,
                    scope,
                    stack,
                    next_contain,
                    random,
                )?;
            }
            if current_name.is_some() {
                stack.pop();
            }
            Ok(())
        }
        Visit(
            self,
            transition,
            &mut tracker,
            order,
            root,
            tree_scope,
            &mut Vec::new(),
            None,
            random,
        )
    }
}
