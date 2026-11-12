//! InputEvent IDL projection. DOM owns live/static boundaries; this binding
//! projects them through the existing realm's canonical node wrappers.
//! Chromium core/events/input_event_init.idl:8-17, input_event.cc:121-142,
//! ui_event_init.idl:7-11, ui_event.cc:49-55,68-84; input_event.h:111-115.
//! Window conversion: native_value_traits_impl.h:1417-1437; projection:
//! core/frame/dom_window.cc:248-260, core/events/ui_event.idl:26,31-35.
use super::*;
use interaction::dispatch::core::{
    dom::events::event::EventRef,
    events::{
        input_event::{InputEvent, InputEventInit, RangeExceptionState},
        ui_event_init::UIEventInit,
    },
};

impl DOMJavaScriptBindings {
    pub(super) fn CallInputEventService(
        &mut self,
        receiver: HostObjectId,
        member: &str,
        arguments: &[HostValue],
    ) -> Option<HostResult> {
        if member == "__inputEventCreate" {
            return Some(self.CreateInputEvent(arguments));
        }
        if member == "__inputDataTransferSetDragImage" {
            return Some(self.SetInputDataTransferDragImage(receiver, arguments));
        }
        if member == "__inputEventInitUIEvent" {
            return Some(self.InitInputUIEvent(receiver, arguments));
        }
        if member != "__inputEventAttribute" && member != "__inputEventTargetRanges" {
            return None;
        }
        let Some(native) = self.services.events.borrow().NativeEvent(receiver) else {
            return Some(Self::type_error("Illegal InputEvent invocation"));
        };
        let owner = native.borrow();
        let Some(input) = owner.AsInputEvent() else {
            return Some(Self::type_error("Illegal InputEvent invocation"));
        };
        if member == "__inputEventAttribute" {
            if matches!(arguments.first(), Some(HostValue::String(name)) if name == "view") {
                return Some(self.ProjectInputEventView(receiver, input.view()));
            }
            let value = match arguments.first() {
                Some(HostValue::String(name)) if name == "data" => input
                    .dataUTF16()
                    .map_or(HostValue::Null(JavaScriptNull), HostValue::String16),
                Some(HostValue::String(name)) if name == "inputType" => {
                    HostValue::String(input.inputType().into())
                }
                Some(HostValue::String(name)) if name == "isComposing" => {
                    HostValue::Boolean(input.isComposing())
                }
                Some(HostValue::String(name)) if name == "detail" => {
                    HostValue::Number(input.detail() as f64)
                }
                Some(HostValue::String(name)) if name == "sourceCapabilities" => input
                    .sourceCapabilities()
                    .map_or(HostValue::Null(JavaScriptNull), |capabilities| {
                        self.native_abort
                            .InputCapabilitiesValue(receiver, capabilities)
                    }),
                Some(HostValue::String(name)) if name == "dataTransfer" => input
                    .dataTransfer()
                    .map_or(HostValue::Null(JavaScriptNull), |transfer| {
                        self.native_abort.InputDataTransferValue(receiver, transfer)
                    }),
                _ => return Some(Self::type_error("Invalid InputEvent attribute")),
            };
            return Some(Self::value(value));
        }
        let ranges = input.getTargetRanges();
        let identity = self.document.borrow().GetDocument().RootHandle().Identity();
        let mut values = vec![("length".into(), HostValue::Number(ranges.len() as f64))];
        for (index, range) in ranges.iter().enumerate() {
            let Some(range) = range.Snapshot().filter(|range| range.document == identity) else {
                return Some(Self::type_error(
                    "InputEvent range document is no longer accessible",
                ));
            };
            values.push((
                index.to_string(),
                HostValue::Record(vec![
                    (
                        "startContainer".into(),
                        HostValue::Object(HostObjectRef {
                            id: range.start_container,
                        }),
                    ),
                    (
                        "startOffset".into(),
                        HostValue::Number(range.start_offset as f64),
                    ),
                    (
                        "endContainer".into(),
                        HostValue::Object(HostObjectRef {
                            id: range.end_container,
                        }),
                    ),
                    (
                        "endOffset".into(),
                        HostValue::Number(range.end_offset as f64),
                    ),
                ]),
            ));
        }
        Some(Self::value(HostValue::Record(values)))
    }
    fn CreateInputEvent(&mut self, arguments: &[HostValue]) -> HostResult {
        let [HostValue::String(kind), HostValue::Boolean(bubbles), HostValue::Boolean(cancelable), HostValue::Boolean(composed), view, HostValue::Number(detail), capabilities, HostValue::String(input_type), data, transfer, HostValue::Boolean(is_composing), ranges @ ..] =
            arguments
        else {
            return Self::type_error("Invalid InputEvent initializer");
        };
        let data = match data {
            HostValue::Null(_) => None,
            HostValue::String(value) => Some(value.encode_utf16().collect::<Vec<_>>().into()),
            HostValue::String16(value) => Some(value.clone()),
            _ => return Self::type_error("InputEvent data requires a nullable DOMString"),
        };
        let view = match view {
            HostValue::Null(_) => None,
            value => match self.native_abort.InputViewFromValue(value) {
                Some(view) => Some(view),
                None => return Self::type_error("InputEvent view requires a nullable Window"),
            },
        };
        let data_transfer = match transfer {
            HostValue::Null(_) => None,
            value => match self.native_abort.InputDataTransferFromValue(value) {
                Some(transfer) => Some(transfer),
                None => return Self::type_error("InputEvent dataTransfer requires a DataTransfer"),
            },
        };
        let source_capabilities = match capabilities {
            HostValue::Null(_) => None,
            value => match self.native_abort.InputCapabilitiesFromValue(value) {
                Some(capabilities) => Some(capabilities),
                None => {
                    return Self::type_error(
                        "InputEvent sourceCapabilities requires InputDeviceCapabilities",
                    )
                }
            },
        };
        if ranges.len() % 4 != 0 {
            return Self::type_error("Invalid InputEvent target ranges");
        }
        let mut target_ranges = Vec::with_capacity(ranges.len() / 4);
        for range in ranges.chunks_exact(4) {
            let [HostValue::Object(start), HostValue::Number(start_offset), HostValue::Object(end), HostValue::Number(end_offset)] =
                range
            else {
                return Self::type_error("InputEvent target range requires DOM boundaries");
            };
            let snapshot = dom::DOMStaticRange {
                start: dom::DOMBoundaryPoint {
                    node_id: start.id,
                    offset: *start_offset as usize,
                },
                end: dom::DOMBoundaryPoint {
                    node_id: end.id,
                    offset: *end_offset as usize,
                },
            };
            let range = match interaction::adapter::event::StaticRangeForInputEvent(
                self.document.clone(),
                snapshot,
            ) {
                Ok(range) => range,
                Err(error) => {
                    return Self::type_error(&dom::error::DOMException::from(error).message)
                }
            };
            target_ranges.push(range);
        }
        let mut ui = UIEventInit::default();
        ui.event_init.bubbles = *bubbles;
        ui.event_init.cancelable = *cancelable;
        ui.event_init.composed = *composed;
        ui.view = view.clone();
        ui.detail = *detail as i32;
        ui.source_capabilities = source_capabilities.clone();
        let init = InputEventInit {
            ui_event_init: ui,
            input_type: Some(input_type.clone()),
            data_utf16: Some(data),
            data_transfer: Some(data_transfer.clone()),
            is_composing: Some(*is_composing),
            target_ranges: Some(target_ranges),
            ..Default::default()
        };
        let fallback = services::StandaloneEventClock::new();
        let clock = self
            .services
            .host
            .event_clock
            .as_deref()
            .unwrap_or(&fallback);
        let mut exceptions = RangeExceptionState::default();
        let Some(event) = InputEvent::Create(kind, &init, &mut exceptions, clock) else {
            return Self::type_error(
                &exceptions
                    .TakeException()
                    .expect("InputEvent conversion exception")
                    .message,
            );
        };
        let id = self.services.BindInputEvent(EventRef::new(event));
        if let Some(view) = view {
            self.native_abort.InputViewValue(id, &view);
        }
        if let Some(transfer) = data_transfer {
            self.native_abort.InputDataTransferValue(id, transfer);
        }
        if let Some(capabilities) = source_capabilities {
            self.native_abort.InputCapabilitiesValue(id, capabilities);
        }
        Self::value(HostValue::Object(HostObjectRef { id }))
    }

    pub(super) fn ProjectInputEventView(
        &self,
        receiver: HostObjectId,
        view: Option<interaction::dispatch::core::events::ui_event::AbstractViewRef>,
    ) -> HostResult {
        let Some(view) = view else {
            return Self::value(HostValue::Null(JavaScriptNull));
        };
        if let Some(script) = self.native_abort.InputViewValue(receiver, &view) {
            return Self::value(script);
        }
        let current = self
            .services
            .host
            .resolve_event_target
            .as_ref()
            .and_then(|resolve| resolve(0))
            .and_then(|target| target.AsAbstractView());
        let script = if current
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(current, &view))
        {
            Some(HostValue::Object(HostObjectRef { id: 0 }))
        } else {
            self.services
                .host
                .project_window_view
                .as_ref()
                .and_then(|project| project(&view))
        };
        let Some(script) = script else {
            return Self::type_error("UIEvent Window wrapper is unavailable");
        };
        self.native_abort.BindInputView(view.clone(), script);
        Self::value(
            self.native_abort
                .InputViewValue(receiver, &view)
                .expect("retained native Window binding"),
        )
    }

    fn SetInputDataTransferDragImage(
        &mut self,
        receiver: HostObjectId,
        arguments: &[HostValue],
    ) -> HostResult {
        let Some(transfer) = self
            .native_abort
            .InputDataTransferFromValue(&HostValue::Object(HostObjectRef { id: receiver }))
        else {
            return Self::type_error("Illegal DataTransfer invocation");
        };
        let [HostValue::Object(image), HostValue::Number(x), HostValue::Number(y)] = arguments
        else {
            return Self::type_error("DataTransfer drag image requires an Element and coordinates");
        };
        let (is_element, detached_image) = {
            let document = self.document.borrow();
            let document = document.GetDocument();
            match document.FindNodeById(image.id) {
                Some(i) if document.Node(i).Type() == DOMNodeType::kElement => (
                    true,
                    (document.Node(i).IsHTMLElement("img") && !document.IsConnectedToDocument(i))
                        .then(|| {
                            document
                                .ExistingHTMLImageLoader(i)
                                .and_then(|loader| loader.GetContent())
                        }),
                ),
                _ => (false, None),
            }
        };
        if !is_element {
            return Self::type_error("DataTransfer drag image requires an Element");
        }
        // data_transfer.cc:408-426. Copy/paste DataTransfer ignores drag images.
        // HTMLImageElement::CachedImage reads its resident ImageLoader rather
        // than a LayoutImage, which a detached image does not have.
        if transfer.IsForDragAndDrop() && transfer.CanWriteData() {
            let Some(zoom) = self
                .services
                .host
                .read_layout_zoom
                .as_ref()
                .map(|read| read())
            else {
                return HostResult::Failure(
                    JavaScriptExceptionKind::kRuntimeError,
                    "DataTransfer image Frame zoom is unavailable",
                );
            };
            let location = foundation::gfx::ScaleToRoundedPointUniform(
                foundation::gfx::Point::new(*x as i32, *y as i32),
                zoom,
            );
            let location = (location.x(), location.y());
            if let Some(content) = detached_image {
                let resource = match content {
                    Some(content) => {
                        let Some(retain) = self.services.host.retain_drag_image.as_ref() else {
                            return HostResult::Failure(
                                JavaScriptExceptionKind::kRuntimeError,
                                "DataTransfer image resource producer is unavailable",
                            );
                        };
                        Some(retain(content))
                    }
                    None => None,
                };
                if let Some(resource) = resource {
                    transfer.SetDragImageResource(resource, location);
                } else {
                    transfer.ClearDragImage();
                }
                self.host_graph.Publish([
                    javascript::javascript_runtime::HostObjectTraceChange::Reference {
                        owner: receiver,
                        slot: u64::MAX - 9,
                        target: None,
                    },
                ]);
                return HostResult::default();
            }
            let Some(target) = self
                .services
                .host
                .resolve_event_target
                .as_ref()
                .and_then(|resolve| resolve(image.id))
            else {
                return HostResult::Failure(
                    JavaScriptExceptionKind::kRuntimeError,
                    "DataTransfer drag image target is unavailable",
                );
            };
            transfer.SetDragImageElement(target, location);
            self.host_graph.Publish([
                javascript::javascript_runtime::HostObjectTraceChange::Reference {
                    owner: receiver,
                    slot: u64::MAX - 9,
                    target: Some(image.id),
                },
            ]);
        }
        HostResult::default()
    }

    fn InitInputUIEvent(&mut self, receiver: HostObjectId, arguments: &[HostValue]) -> HostResult {
        let [HostValue::String(kind), HostValue::Boolean(bubbles), HostValue::Boolean(cancelable), view, HostValue::Number(detail)] =
            arguments
        else {
            return Self::type_error("Invalid UIEvent initializer");
        };
        let view = match view {
            HostValue::Null(_) => None,
            value => match self.native_abort.InputViewFromValue(value) {
                Some(view) => Some(view),
                None => return Self::type_error("UIEvent view requires a nullable Window"),
            },
        };
        let Some(native) = self.services.events.borrow().NativeEvent(receiver) else {
            return Self::type_error("Illegal UIEvent invocation");
        };
        let mut owner = native.borrow_mut();
        let Some(input) = owner.AsInputEventMut() else {
            return Self::type_error("Illegal UIEvent invocation");
        };
        // ui_event.cc:68-84 guards dispatch and resets only the native base;
        // InputEvent's payload and live Range owners remain untouched.
        input.initUIEvent(kind, *bubbles, *cancelable, view.clone(), *detail as i32);
        if !input.IsBeingDispatched() {
            if let Some(view) = view {
                self.native_abort.InputViewValue(receiver, &view);
            } else {
                self.host_graph.Publish([
                    javascript::javascript_runtime::HostObjectTraceChange::Reference {
                        owner: receiver,
                        slot: u64::MAX - 10,
                        target: None,
                    },
                ]);
            }
            self.host_graph.Publish([
                javascript::javascript_runtime::HostObjectTraceChange::Reference {
                    owner: receiver,
                    slot: u64::MAX - 8,
                    target: None,
                },
                javascript::javascript_runtime::HostObjectTraceChange::ValueReference {
                    owner: receiver,
                    slot: u64::MAX - 8,
                    value: None,
                },
            ]);
        }
        HostResult::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dom::dom_mutation::ApplyDOMMutations;
    use javascript::{
        javascript_runtime::JavaScriptRuntime, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
    };

    #[test]
    fn input_event_target_ranges_snapshot_live_dom_boundaries() {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<p id=target>a😀b</p>",
        )));
        let changed = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document,
            Box::new(move |mutation| {
                ApplyDOMMutations(
                    changed.borrow_mut().GetDocumentMut(),
                    std::slice::from_ref(mutation),
                );
            }),
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings);
        let bootstrap = runtime.Evaluate(&realm, crate::DOMBootstrapSource(), "browser:dom-webidl");
        assert!(bootstrap.Succeeded(), "{:?}", bootstrap.exception);
        let result = runtime.Evaluate(&realm, r#"
            function check(value, message) { if (!value) throw Error(message); }
            const text=document.getElementById('target').firstChild;
            const range=new StaticRange({startContainer:text,startOffset:1,endContainer:text,endOffset:3});
            const transfer=new DataTransfer();
            transfer.setData('text/plain','a\ud800b');
            const capabilities=new InputDeviceCapabilities({firesTouchEvents:true});
            const event=new InputEvent('beforeinput',{view:window,detail:4294967303,sourceCapabilities:capabilities,inputType:'deleteContentBackward',data:null,dataTransfer:transfer,targetRanges:[range]});
            check(event instanceof InputEvent,'native InputEvent prototype');
            check(event.view===window && event.detail===7,'native UIEvent initializer and long conversion');
            check(event.sourceCapabilities===capabilities && capabilities.firesTouchEvents,'native capabilities identity');
            check(event.dataTransfer===transfer && event.dataTransfer===event.dataTransfer,'canonical DataTransfer wrapper');
            check(event.dataTransfer.getData('text/plain')==='a\ud800b','DataTransfer exact UTF-16 owner');
            transfer.setData('text/plain','updated');
            check(event.dataTransfer.getData('text/plain')==='updated','InputEvent retains live DataTransfer owner');
            const defaults=new InputEvent('input');
            check(defaults.view===null && defaults.detail===0 && defaults.dataTransfer===null && defaults.sourceCapabilities===null,'nullable InputEvent defaults');
            const types=transfer.types;
            check(Object.isFrozen(types) && transfer.types===types,'cached frozen native types');
            const items=transfer.items;
            check(items instanceof DataTransferItemList && items.length===1,'native item list projection');
            check(items[0] instanceof DataTransferItem && items[0].kind==='string' && items[0].type==='text/plain','native indexed item projection');
            const item=items.add('other\ud800','application/x-example');
            check(item.kind==='string' && items.length===2 && transfer.types!==types,'native item mutation invalidates types');
            let duplicate=false;
            try { items.add('duplicate','application/x-example'); } catch(error) { duplicate=error.name==='NotSupportedError'; }
            check(duplicate,'native duplicate item exception');
            items.remove(1);
            check(items.length===1 && items[1]===undefined,'live indexed item removal');
            check(transfer.files instanceof FileList && transfer.files===transfer.files && transfer.files.length===0,'canonical FileList identity');
            check(transfer.files.item(0)===null && transfer.files[0]===undefined,'FileList bounds');
            transfer.setDragImage(document.body,2,3);
            check(event.data===null && event.inputType==='deleteContentBackward','native payload');
            const first=event.getTargetRanges();
            check(first.length===1 && first[0] instanceof StaticRange,'StaticRange projection');
            check(first[0].startContainer===text && first[0].endContainer===text,'canonical node wrappers');
            check(first[0].startOffset===1 && first[0].endOffset===3,'UTF-16 range offsets');
            text.textContent='replacement';
            const second=event.getTargetRanges();
            check(second[0].collapsed && second[0].startOffset===0,'canonical live Range mutation');
            check(first[0].startOffset===1 && first[0].endOffset===3,'static snapshot stays immutable');
            check(second[0]!==first[0] && second[0].startContainer===first[0].startContainer,'fresh ranges share stable nodes');
            UIEvent.prototype.initUIEvent.call(event,'input',true,true,null,-4);
            check(event.type==='input' && event.bubbles && event.cancelable && event.view===null && event.detail===-4,'initUIEvent mutates native owner');
            check(event.sourceCapabilities===null,'initUIEvent clears capabilities');
            UIEvent.prototype.initUIEvent.call(event,'beforeinput',false,false,window,9);
            check(event.view===window && event.detail===9,'initUIEvent retains the canonical Window binding');
            check(event.dataTransfer===transfer && event.inputType==='deleteContentBackward','UIEvent initialization preserves InputEvent payload');
            let threw=false;
            try { InputEvent.prototype.getTargetRanges.call({}); } catch (error) { threw=error instanceof TypeError; }
            check(threw,'InputEvent brand check');
            for (const options of [{view:{}},{dataTransfer:{}},{sourceCapabilities:{}}]) {
              threw=false;
              try { new InputEvent('input',options); } catch (error) { threw=error instanceof TypeError; }
              check(threw,'InputEvent nullable interface conversion');
            }
            let windowReads=0;
            const fakeWindow=new Proxy({}, {get(){++windowReads;return window;}});
            threw=false;
            try { new InputEvent('input',{view:fakeWindow}); } catch(error) { threw=error instanceof TypeError; }
            check(threw && windowReads===0,'Window brand conversion never invokes author properties');
        "#, "input-event-ranges.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
    }
}
