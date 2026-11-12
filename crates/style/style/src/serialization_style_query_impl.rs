use super::*;
use crate::serialization_style_query::{SerializationRuleScope, SerializationStyleQuery};
impl DocumentStyleEngine {
    /// The lifecycle caller lends the original Document. Matching remains in
    /// this owner and reuses its retained RuleSets, compiled selectors, scope
    /// bindings and container inputs. There is no Browser selector pass or CSS
    /// text projection, and no document/layout tree is materialized.
    pub fn SerializationStyleQuery<'d>(
        &mut self,
        document: &'d Document,
        index: usize,
        scope: SerializationRuleScope,
        environment: Option<&dyn EnvironmentVariableResolver>,
    ) -> Result<SerializationStyleQuery<'d>, DocumentStyleError> {
        self.EnsureAttached()?;
        if document.RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        if document.StyleState().all_dirty || !document.StyleState().changes.is_empty() {
            return Err(DocumentStyleError::Unsupported(
                "serialization requires the source style-clean lifecycle",
            ));
        }
        let media = self
            .environment
            .as_ref()
            .ok_or(DocumentStyleError::Unsupported(
                "serialization has no prepared viewport environment",
            ))?
            .0
            .clone();
        let active = self.active_sheets.clone();
        let active_scopes = self.active_sheet_scopes.clone();
        // Build only the active-sheet scope index. This walks no DOM or Layout
        // nodes and retains references to the same canonical stylesheet owners.
        let mut roots = HashSet::new();
        roots.insert(document.TreeScopeRoot(index));
        roots.extend(active_scopes.iter().flatten().copied());
        let scoped = roots
            .into_iter()
            .map(|root| {
                let sheets = active
                    .iter()
                    .zip(active_scopes.iter())
                    .filter(|((_, origin), candidate)| {
                        *origin != CascadeOrigin::kAuthor || **candidate == Some(root)
                    })
                    .map(|((sheet, origin), _)| (sheet.as_ref(), *origin))
                    .collect::<Vec<_>>();
                (root, sheets)
            })
            .collect::<HashMap<_, _>>();
        let sheets = scoped
            .get(&document.TreeScopeRoot(index))
            .map_or(&[][..], Vec::as_slice);
        let mut matched = self.CollectRules(
            document,
            index,
            &media,
            sheets,
            &scoped,
            None,
            None,
            None,
            environment,
            Some(scope),
        )?;
        matched
            .serialization_declarations
            .retain(|declaration| scope.Includes(declaration.origin));
        let computed = document
            .ResolvedStyleFor(index)
            .and_then(|style| style.ComputedStyle());
        Ok(SerializationStyleQuery {
            computed,
            declarations: matched.serialization_declarations,
        })
    }
}

impl DocumentStyleEngine {
    /// Source StyleResolver::ComputeValue. Uses the live computed/custom values
    /// and the original viewport/font/container/registration owners; the one
    /// temporary style is never published onto the element.
    pub fn ResolveSerializationValue(
        &self,
        document: &Document,
        index: usize,
        name: &CSSPropertyName,
        value: Rc<Value>,
        environment_variables: Option<&dyn EnvironmentVariableResolver>,
    ) -> Result<Option<Rc<Value>>, crate::serialization_style_query::SerializationValueError> {
        use crate::resolver::production_style_cascade::{
            ComputeSerializationValue, ProductionCascadeInputs,
        };
        use crate::serialization_style_query::SerializationValueError;
        self.EnsureAttached()?;
        if document.RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument.into());
        }
        if document.StyleState().all_dirty || !document.StyleState().changes.is_empty() {
            return Err(DocumentStyleError::Unsupported(
                "serialization requires the source style-clean lifecycle",
            )
            .into());
        }
        let Some(base) = document
            .ResolvedStyleFor(index)
            .map(|style| style.computed_style_owner.clone())
        else {
            return Ok(None);
        };
        let media = self
            .environment
            .as_ref()
            .ok_or(SerializationValueError::Style(
                DocumentStyleError::Unsupported(
                    "serialization has no prepared viewport environment",
                ),
            ))?
            .0
            .clone();
        let parent_index = document.FlatTreeParentElement(index);
        let parent = parent_index
            .and_then(|parent| document.ResolvedStyleFor(parent))
            .map(|style| style.computed_style_owner.clone());
        let root_style = document
            .DocumentElement()
            .and_then(|root| document.ResolvedStyleFor(root))
            .map(|style| style.computed_style_owner.clone());
        let root_font_size = unsafe { &*root_style.as_ref().unwrap_or(&base).Get() }
            .GetFontDescription()
            .ComputedSize();
        let environment = CascadeEnvironment::new(environment_variables);
        let attribute_source = PersistentAttributeSource::new(
            document,
            index,
            document.Node(index).Namespace() == DOMNamespace::kHTML,
        );
        let attribute_state = CascadeAttributeState::new();
        let attributes = AttributeResolver::new(&attribute_source, &attribute_state, false);
        let images = DocumentURLImageResolver {
            document,
            originating_index: index,
        };
        let containers =
            crate::resolver::production_container_runtime::ProductionContainerRuntime {
                document,
                media: &media,
                states: &self.container_states,
                snapped_dependencies: Some(&self.snapped_query_dependencies),
                stored_custom: &self.custom_properties,
                current_styles: None,
                current_custom: None,
                environment: environment_variables,
                registered: Some(
                    self.registered_container_properties
                        .as_deref()
                        .unwrap_or(&self.property_registry),
                ),
            };
        let function_rules = self
            .active_sheets
            .iter()
            .filter_map(|(sheet, _)| {
                self.translated_rule_sets
                    .get(&(sheet.Identity(), sheet.Revision()))
                    .cloned()
            })
            .collect::<Vec<_>>();
        let functions = crate::production_style_sheet::ProductionFunctionRegistry::FromRuleSets(
            &function_rules,
        );
        ComputeSerializationValue(ProductionCascadeInputs {
            document:Some(document),media:media.clone(),base,parent,root_style,match_result:MatchResult::default(),
            root_font_size,color_scheme_settings:self.color_scheme_settings,
            system_colors:crate::resolver::production_style_resolver::ProductionSystemColorContext::default(),
            text_link_text_color:self.resolver_dependencies.text_link_text_color,
            link_color:self.resolver_dependencies.link_color,visited_link_color:self.resolver_dependencies.visited_link_color,
            active_link_color:self.resolver_dependencies.active_link_color,
            is_document_element:document.DocumentElement()==Some(index),pseudo_id:PseudoId::kPseudoIdNone,
            originating_style:None,subject_node:Some(index),container_runtime:Some(&containers),
            custom_properties:self.custom_properties.get(&index).map(Rc::as_ref),
            parent_custom_properties:parent_index.and_then(|parent| self.custom_properties.get(&parent)).map(Rc::as_ref),
            substitution_context:SubstitutionContext { environment:Some(&environment),attributes:Some(&attributes) },
            image_resolver:Some(&images),property_registry:Some(&self.property_registry),function_registry:Some(&functions),
            animations:None,transitions:None,font_metrics:self.font_metrics.as_deref(),
        },name,value)
    }
}

impl DocumentStyleEngine {
    /// Element's original inline declaration handle, including a CSSOM write's
    /// retained compiled backing. No CSSOM string projection is involved.
    pub fn InlineSerializationStyle(
        &self,
        document: &Document,
        index: usize,
    ) -> Result<
        Option<Rc<CSSPropertyValueSetRuleHandle<ProductionCSSValueDispatch>>>,
        DocumentStyleError,
    > {
        self.EnsureAttached()?;
        if document.RootHandle() != self.document {
            return Err(DocumentStyleError::WrongDocument);
        }
        let node = document.Node(index);
        if node.FindAttribute("style").is_none() {
            return Ok(None);
        }
        let inline = node.InlineStyle();
        let parsed = self
            .CSSOMSource()
            .ContextForCurrentElement(document, index)
            .as_ref()
            .map_or_else(
                || {
                    crate::production_style_sheet_projection::ResolveInlineStyle(
                        &inline,
                        DocumentParserMode(document),
                    )
                },
                |context| {
                    crate::production_style_sheet_projection::ResolveInlineStyleWithContext(
                        &inline, context,
                    )
                },
            );
        Ok(Some(parsed.Declarations().as_ref().clone()))
    }
}
