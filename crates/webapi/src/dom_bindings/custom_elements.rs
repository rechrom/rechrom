//! Script/DOM boundary for Blink's custom-element definition and reaction model.
//! Native definitions and queues contain no JavaScript values. The bootstrap
//! retains constructor/callback handles keyed by these document-local IDs.
use super::*;
use dom::custom_elements::{
    CustomElementCallbacks, CustomElementDefinitionId, CustomElementDescriptor,
    CustomElementOptions, CustomElementReaction, IsValidCustomElementName,
};

impl DOMJavaScriptBindings {
    /// Page's existing checkpoint owner schedules unscoped DOM reactions.
    /// Returning a handle never invokes script while the host is borrowed.
    pub fn TakeCustomElementBackupCallback(&mut self) -> Option<JavaScriptFunction> {
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        if self.custom_element_backup_queued
            || document.HasActiveCustomElementReactionScope()
            || !document.HasPendingCustomElementBackupReactions()
        {
            return None;
        }
        let callback = self
            .method_wrappers
            .get("__customElementBackupReactions")?
            .clone();
        self.custom_element_backup_queued = true;
        Some(callback)
    }

    /// Enqueue the original Document's backup element queue on its same Agent.
    /// The installed helper receives that Document explicitly after Window reuse.
    pub fn ScheduleCustomElementBackupReactions(
        &mut self,
    ) -> Result<(), javascript::javascript_runtime::NativeMicrotaskEnqueueError> {
        let Some(callback) = self.TakeCustomElementBackupCallback() else {
            return Ok(());
        };
        let namespace = self
            .HostObjectNamespace()
            .expect("scripted Document namespace")
            .Downgrade();
        let callback = callback.Downgrade();
        let document = {
            let owner = self.document.borrow();
            owner.GetDocument().Node(owner.GetDocument().Root()).Id()
        };
        self.runtime_realm
            .Upgrade()
            .and_then(|realm| realm.NativeMicrotasks())
            .ok_or(javascript::javascript_runtime::NativeMicrotaskEnqueueError::RealmDestroyed)?
            .Enqueue(Box::new(move |runtime| {
                let Some(namespace) = namespace.Upgrade() else {
                    return;
                };
                let Some(callback) = callback.Upgrade() else {
                    return;
                };
                let runtime = runtime.expect("scripted Agent lends its current runtime");
                let _scope = runtime.EnterHostObjectNamespace(&namespace);
                let result = runtime.Call(
                    &callback,
                    &HostValue::default(),
                    &[namespace.Wrap(HostValue::Object(HostObjectRef { id: document }))],
                );
                if let Some(error) = result.exception {
                    let _ = runtime.ReportException(&HostValue::String(error.message));
                }
            }))
    }

    pub(super) fn CustomElementCall(
        &mut self,
        receiver: HostObjectId,
        member: &str,
        arguments: &[HostValue],
    ) -> HostResult {
        let mut owner = self.document.borrow_mut();
        let document = owner.GetDocumentMut();
        let Some(node) = document.FindNodeById(receiver) else {
            return Self::type_error("Custom element operation requires a native Node");
        };
        let document_operation = !matches!(member, "__ceUpgrade" | "__ceBegin" | "__ceComplete");
        if document_operation && node != document.Root() {
            return Self::type_error("Custom element registry requires its Document");
        }
        let id_argument = || match arguments.first() {
            Some(HostValue::Number(id)) if *id > 0.0 && id.fract() == 0.0 => {
                Some(CustomElementDefinitionId(*id as u64))
            }
            _ => None,
        };
        match member {
            "__ceBackupStart" => {
                self.custom_element_backup_queued = false;
                HostResult::default()
            }
            "__ceValidName" => Self::value(HostValue::Boolean(matches!(
                arguments.first(), Some(HostValue::String(name)) if IsValidCustomElementName(name)
            ))),
            "__ceFind" => {
                let Some(HostValue::String(name)) = arguments.first() else {
                    return Self::type_error("Custom element name must be a string");
                };
                Self::value(HostValue::Number(
                    document
                        .CustomElementDefinitionForName(name)
                        .map_or(0.0, |definition| definition.id.0 as f64),
                ))
            }
            "__ceDefinitionName" => {
                let name = id_argument()
                    .and_then(|id| document.CustomElementDefinitionForId(id))
                    .map(|definition| definition.descriptor.name.clone());
                Self::value(name.map_or(HostValue::Null(JavaScriptNull), HostValue::String))
            }
            "__ceRegister" => {
                let [HostValue::String(name), HostValue::Number(flags), observed @ ..] = arguments
                else {
                    return Self::type_error("Invalid custom element definition");
                };
                let Some(observed_attributes) = observed
                    .iter()
                    .map(|value| match value {
                        HostValue::String(value) => Some(value.clone()),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>()
                else {
                    return Self::type_error("Observed attributes must be DOMStrings");
                };
                let flags = *flags as u32;
                match document.RegisterCustomElementDefinitionWithOptions(
                    CustomElementDescriptor {
                        name: name.clone(),
                        local_name: name.clone(),
                    },
                    observed_attributes,
                    CustomElementCallbacks {
                        connected: flags & 1 != 0,
                        disconnected: flags & 2 != 0,
                        connected_move: flags & 4 != 0,
                        adopted: flags & 8 != 0,
                        attribute_changed: flags & 16 != 0,
                        form_associated: flags & 32 != 0,
                        form_disabled: flags & 64 != 0,
                        form_reset: flags & 128 != 0,
                        form_state_restore: flags & 1024 != 0,
                    },
                    CustomElementOptions {
                        form_associated: flags & 256 != 0,
                        disable_internals: flags & 512 != 0,
                    },
                ) {
                    Ok(id) => Self::value(HostValue::Number(id.0 as f64)),
                    Err(error) => {
                        Self::type_error(&format!("Invalid custom element definition: {error:?}"))
                    }
                }
            }
            "__cePush" => {
                document.PushCustomElementReactionScope();
                HostResult::default()
            }
            "__cePop" => {
                if !document.PopCustomElementReactionScope() {
                    return Self::type_error("Custom element reaction scope is not drained");
                }
                HostResult::default()
            }
            "__ceUpgrade" => {
                document.EnqueueCustomElementUpgradeSubtree(node);
                HostResult::default()
            }
            "__ceBegin" => {
                let Some(id) = id_argument() else {
                    return Self::type_error("Invalid definition ID");
                };
                Self::value(HostValue::Boolean(
                    document.BeginCustomElementUpgrade(node, id),
                ))
            }
            "__ceComplete" => {
                let Some(id) = id_argument() else {
                    return Self::type_error("Invalid definition ID");
                };
                document.CompleteCustomElementUpgrade(
                    node,
                    id,
                    matches!(arguments.get(1), Some(HostValue::Boolean(true))),
                );
                drop(owner);
                dom::attribute_lifecycle::DrainAttributeLifecycle(&self.document);
                HostResult::default()
            }
            "__ceNext" => {
                let Some(reaction) = document.PopCustomElementReaction() else {
                    return Self::value(HostValue::Null(JavaScriptNull));
                };
                let node = reaction.Element();
                let (kind, id) = match &reaction {
                    CustomElementReaction::Upgrade { id, .. } => ("upgrade", *id),
                    CustomElementReaction::AttributeChanged { id, .. } => {
                        ("attributeChangedCallback", *id)
                    }
                    CustomElementReaction::Connected { id, .. } => ("connectedCallback", *id),
                    CustomElementReaction::Disconnected { id, .. } => ("disconnectedCallback", *id),
                    CustomElementReaction::FormAssociated { id, .. } => {
                        ("formAssociatedCallback", *id)
                    }
                    CustomElementReaction::FormDisabled { id, .. } => ("formDisabledCallback", *id),
                    CustomElementReaction::FormReset { id, .. } => ("formResetCallback", *id),
                    CustomElementReaction::FormStateRestore { id, .. } => {
                        ("formStateRestoreCallback", *id)
                    }
                };
                let mut fields = vec![
                    ("kind".into(), HostValue::String(kind.into())),
                    ("definition".into(), HostValue::Number(id.0 as f64)),
                    (
                        "element".into(),
                        HostValue::Object(HostObjectRef {
                            id: document.Node(node).Id(),
                        }),
                    ),
                ];
                match &reaction {
                    CustomElementReaction::FormAssociated { form, .. } => fields.push((
                        "form".into(),
                        form.map_or(HostValue::Null(JavaScriptNull), |form| {
                            HostValue::Object(HostObjectRef {
                                id: document.Node(form).Id(),
                            })
                        }),
                    )),
                    CustomElementReaction::FormDisabled { disabled, .. } => {
                        fields.push(("disabled".into(), HostValue::Boolean(*disabled)))
                    }
                    _ => {}
                }
                match reaction {
                    CustomElementReaction::AttributeChanged {
                        name,
                        old_value,
                        new_value,
                        namespace,
                        ..
                    } => {
                        fields.extend([
                            ("name".into(), HostValue::String(name)),
                            (
                                "oldValue".into(),
                                old_value
                                    .map_or(HostValue::Null(JavaScriptNull), HostValue::String),
                            ),
                            (
                                "newValue".into(),
                                new_value
                                    .map_or(HostValue::Null(JavaScriptNull), HostValue::String),
                            ),
                            (
                                "namespace".into(),
                                namespace
                                    .map_or(HostValue::Null(JavaScriptNull), HostValue::String),
                            ),
                        ]);
                    }
                    CustomElementReaction::FormStateRestore { state, .. } => {
                        fields.push((
                            "state".into(),
                            self.native_abort.ElementInternalsStateValue(state),
                        ));
                    }
                    _ => {}
                }
                Self::value(HostValue::Record(fields))
            }
            _ => Self::type_error("Unknown custom element operation"),
        }
    }
}
