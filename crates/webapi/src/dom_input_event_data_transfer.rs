//! InputEvent's nullable DataTransfer IDL projection. All clipboard state and
//! string normalization remain on the original native DataTransfer owner.
use super::*;
use interaction::dispatch::core::events::input_event::DataTransferRef;
use interaction::dispatch::core::events::ui_event::AbstractViewRef;
use interaction::dispatch::core::events::ui_event::InputDeviceCapabilitiesRef;

#[path = "dom_input_event_transfer_items.rs"]
mod items;

struct NativeInputDataTransfer {
    transfer: DataTransferRef,
    types: RefCell<Option<Value>>,
}
impl ScriptNativeObject for NativeInputDataTransfer {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn ScriptIdentity(&self) -> usize {
        Rc::as_ptr(&self.transfer) as usize
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "DataTransfer"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        if let Some(types) = self.types.borrow().clone() {
            visit(types);
        }
    }
}

struct NativeInputCapabilities(InputDeviceCapabilitiesRef);
struct NativeInputView {
    view: AbstractViewRef,
    script: Value,
}
impl ScriptNativeObject for NativeInputView {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn ScriptIdentity(&self) -> usize {
        Rc::as_ptr(&self.view) as *const () as usize
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "InputEventView"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        visit(self.script.clone());
    }
}
impl ScriptNativeObject for NativeInputCapabilities {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn ScriptIdentity(&self) -> usize {
        Rc::as_ptr(&self.0) as *const () as usize
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "InputDeviceCapabilities"
    }
}

impl AbortBindings {
    pub(in crate::dom_bindings) fn BindInputView(
        &self,
        view: AbstractViewRef,
        script: HostValue,
    ) -> HostValue {
        let native = Rc::new(NativeInputView {
            view,
            script: value(script),
        });
        let id = self.EnsureObject(native);
        let native = self
            .objects
            .borrow()
            .get(&id)
            .cloned()
            .expect("canonical native Window projection");
        self.Trace(native);
        self.NativeValue(id)
    }
    pub(in crate::dom_bindings) fn InputViewFromValue(
        &self,
        value: &HostValue,
    ) -> Option<AbstractViewRef> {
        self.NativeFromValue::<NativeInputView>(value)
            .map(|native| native.view.clone())
    }
    pub(in crate::dom_bindings) fn InputViewValue(
        &self,
        event: u64,
        view: &AbstractViewRef,
    ) -> Option<HostValue> {
        let identity = Rc::as_ptr(view) as *const () as usize;
        let id = self.identities.borrow().get(&identity).copied()?;
        let native =
            self.NativeFromValue::<NativeInputView>(&HostValue::Object(HostObjectRef { id }))?;
        self.Publish([C::Reference {
            owner: event,
            slot: u64::MAX - 10,
            target: Some(id),
        }]);
        Some(host(&native.script))
    }
    fn InputTransferNative(&self, transfer: DataTransferRef) -> Rc<NativeInputDataTransfer> {
        let identity = Rc::as_ptr(&transfer) as usize;
        if let Some(id) = self.identities.borrow().get(&identity).copied() {
            if let Some(native) = self.NativeFromValue(&HostValue::Object(HostObjectRef { id })) {
                return native;
            }
        }
        Rc::new(NativeInputDataTransfer {
            transfer,
            types: Default::default(),
        })
    }
    fn InputNativeValue(&self, object: Rc<dyn ScriptNativeObject>) -> HostValue {
        let id = self.EnsureObject(object.clone());
        self.Trace(object);
        self.NativeValue(id)
    }
    fn RememberInputForeign(&self, v: &HostValue, identity: usize) -> Option<()> {
        if let HostValue::Retained(owner, _) = v {
            let reference = owner
                .downcast_ref::<javascript::javascript_runtime::JavaScriptNativeObjectReference>(
                )?;
            self.foreign_bindings
                .borrow_mut()
                .insert(identity, reference.BindingCell().Downgrade()?);
        }
        Some(())
    }
    pub(in crate::dom_bindings) fn InputCapabilitiesFromValue(
        &self,
        v: &HostValue,
    ) -> Option<InputDeviceCapabilitiesRef> {
        let native = self.NativeFromValue::<NativeInputCapabilities>(v)?;
        self.RememberInputForeign(v, native.ScriptIdentity())?;
        Some(native.0.clone())
    }
    pub(in crate::dom_bindings) fn InputCapabilitiesValue(
        &self,
        event: u64,
        capabilities: InputDeviceCapabilitiesRef,
    ) -> HostValue {
        self.InputMemberValue(
            event,
            u64::MAX - 8,
            Rc::new(NativeInputCapabilities(capabilities)),
        )
    }
    pub(in crate::dom_bindings) fn InputDataTransferFromValue(
        &self,
        value: &HostValue,
    ) -> Option<DataTransferRef> {
        let native = self.NativeFromValue::<NativeInputDataTransfer>(value)?;
        self.RememberInputForeign(value, native.ScriptIdentity())?;
        Some(native.transfer.clone())
    }

    pub(in crate::dom_bindings) fn InputDataTransferValue(
        &self,
        event: u64,
        transfer: DataTransferRef,
    ) -> HostValue {
        self.InputMemberValue(event, u64::MAX - 7, self.InputTransferNative(transfer))
    }

    fn InputMemberValue(
        &self,
        event: u64,
        slot: u64,
        native: Rc<dyn ScriptNativeObject>,
    ) -> HostValue {
        if let Some(foreign) = self.foreign_bindings.borrow().get(&native.ScriptIdentity()) {
            let value = foreign
                .Upgrade()
                .expect("InputEvent retains native binding cell");
            self.Publish([C::ValueReference {
                owner: event,
                slot,
                value: Some(value.clone()),
            }]);
            return HostValue::JavaScriptValue(value);
        }
        let id = self.EnsureObject(native.clone());
        self.Trace(native);
        // InputEvent::Trace visits data_transfer_. A keyed Member edge survives
        // updates of Event's separate dispatch-path ReferenceList.
        self.Publish([C::Reference {
            owner: event,
            slot,
            target: Some(id),
        }]);
        self.NativeValue(id)
    }

    pub(super) fn InvokeInputDataTransfer(
        &self,
        receiver: u64,
        member: &str,
        args: &[HostValue],
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> HostResult {
        let invalid = || Self::InvalidInputDataTransfer();
        let result = |value| HostResult {
            value,
            ..Default::default()
        };
        if member == "__inputDataTransferCreate" && receiver == 0 {
            // data_transfer.cc:281-287: constructor creates writable copy/paste
            // data with dropEffect/effectAllowed equal to "none".
            let native = self.InputTransferNative(
                interaction::dispatch::core::clipboard::data_transfer::DataTransfer::Create(),
            );
            let id = self.EnsureObject(native.clone());
            self.Trace(native);
            return result(self.NativeValue(id));
        }
        if member == "__inputDataTransferCapabilitiesCreate" && receiver == 0 {
            let capabilities = interaction::dispatch::core::input::input_device_capabilities::InputDeviceCapabilities::Create(matches!(args.first(), Some(HostValue::Boolean(true))));
            return result(self.InputNativeValue(Rc::new(NativeInputCapabilities(capabilities))));
        }
        if let Some(capabilities) =
            self.NativeFromValue::<NativeInputCapabilities>(&HostValue::Object(HostObjectRef {
                id: receiver,
            }))
        {
            return match member {
                "__inputDataTransferCapabilitiesBrand" => result(HostValue::Boolean(true)),
                "__inputDataTransferCapabilitiesTouch" => {
                    result(HostValue::Boolean(capabilities.0.firesTouchEvents()))
                }
                _ => invalid(),
            };
        }
        if let Some(result) = self.InvokeInputTransferItems(receiver, member, args, runtime) {
            return result;
        }
        let Some(native) =
            self.NativeFromValue::<NativeInputDataTransfer>(&HostValue::Object(HostObjectRef {
                id: receiver,
            }))
        else {
            return invalid();
        };
        let transfer = &native.transfer;
        let text = |n| match args.get(n) {
            Some(HostValue::String(value)) => Some(value.as_str()),
            _ => None,
        };
        match member {
            "__inputDataTransferBrand" => result(HostValue::Boolean(true)),
            "__inputDataTransferTypesCached" => {
                if transfer.hasDataStoreItemListChanged() {
                    return HostResult::default();
                }
                result(native.types.borrow().as_ref().map(host).unwrap_or_default())
            }
            "__inputDataTransferTypesRemember" => {
                let Some(types) = args.first() else {
                    return invalid();
                };
                *native.types.borrow_mut() = Some(value(types.clone()));
                self.Trace(native);
                result(types.clone())
            }
            "__inputDataTransferGetData" => {
                let Some(kind) = text(0) else {
                    return invalid();
                };
                let value = transfer.getData(kind);
                let units = (0..value.length())
                    .map(|index| value.CodeUnitAt(index))
                    .collect::<Vec<_>>();
                result(HostValue::String16(units.into()))
            }
            "__inputDataTransferSetData" => {
                let Some(kind) = text(0) else {
                    return invalid();
                };
                match args.get(1) {
                    Some(HostValue::String(value)) => transfer.setData(kind, value),
                    Some(HostValue::String16(value)) => transfer.setDataUTF16(kind, value),
                    _ => return invalid(),
                }
                HostResult::default()
            }
            "__inputDataTransferClearData" => {
                if !args.is_empty() && text(0).is_none() {
                    return invalid();
                }
                transfer.clearData(text(0));
                HostResult::default()
            }
            "__inputDataTransferAttribute" => match text(0) {
                Some("dropEffect") => result(HostValue::String(transfer.dropEffect().into())),
                Some("effectAllowed") => result(HostValue::String(transfer.effectAllowed())),
                Some("types") => {
                    let types = transfer.types();
                    let mut values = vec![("length".into(), HostValue::Number(types.len() as f64))];
                    values.extend(
                        types
                            .into_iter()
                            .enumerate()
                            .map(|(i, value)| (i.to_string(), HostValue::String(value))),
                    );
                    result(HostValue::Record(values))
                }
                Some("items") => result(self.InputTransferItemsValue(transfer.items())),
                Some("files") => result(self.InputTransferFilesValue(transfer.files())),
                _ => invalid(),
            },
            "__inputDataTransferSetAttribute" => {
                let Some(value) = text(1) else {
                    return invalid();
                };
                match text(0) {
                    Some("dropEffect") => transfer.setDropEffect(value),
                    Some("effectAllowed") => transfer.setEffectAllowed(value),
                    _ => return invalid(),
                }
                HostResult::default()
            }
            _ => invalid(),
        }
    }

    fn InvalidInputDataTransfer() -> HostResult {
        HostResult::Failure(
            JavaScriptExceptionKind::kTypeError,
            "Illegal DataTransfer invocation",
        )
    }
}
