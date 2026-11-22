//! Source trusted_html.cc, trusted_script.cc, trusted_script_url.cc;
//! trusted_type_policy.cc createHTML/createScript/createScriptURL and Internal;
//! trusted_type_policy_factory.cc createPolicy; trusted_types_util.cc:195-207,
//! 217-293,384-450,917-924. Native brands and policy maps retain their original
//! Window/ScriptState owner; Document's compiled CSP decides permission/reporting.
use super::*;
use foundation::content_security_policy::{CSPRequestPolicy, CSPSourceLocation, CSPWorld};

struct NativeTrustedHTML(Rc<[u16]>);
struct NativeTrustedScript(Rc<[u16]>);
struct NativeTrustedScriptURL(Rc<[u16]>);
struct NativeTrustedTypePolicy {
    name: Rc<[u16]>,
    create_html: Option<Value>,
    create_script: Option<Value>,
    create_script_url: Option<Value>,
}
pub(super) struct NativeTrustedTypeFactory {
    world: i32,
    policies: RefCell<HashMap<Vec<u16>, Rc<NativeTrustedTypePolicy>>>,
    empty_html: Rc<NativeTrustedHTML>,
    empty_script: Rc<NativeTrustedScript>,
}
impl ScriptNativeObject for NativeTrustedHTML {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn ScriptIdentity(&self) -> usize {
        self as *const Self as usize
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "TrustedHTML"
    }
}
impl ScriptNativeObject for NativeTrustedScript {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn ScriptIdentity(&self) -> usize {
        self as *const Self as usize
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "TrustedScript"
    }
}
impl ScriptNativeObject for NativeTrustedScriptURL {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn ScriptIdentity(&self) -> usize {
        self as *const Self as usize
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "TrustedScriptURL"
    }
}
impl ScriptNativeObject for NativeTrustedTypePolicy {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn ScriptIdentity(&self) -> usize {
        self as *const Self as usize
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "TrustedTypePolicy"
    }
    fn VisitScriptValues(&self, visit: &mut dyn FnMut(Value)) {
        if let Some(callback) = &self.create_html {
            visit(callback.clone());
        }
        if let Some(callback) = &self.create_script {
            visit(callback.clone());
        }
        if let Some(callback) = &self.create_script_url {
            visit(callback.clone());
        }
    }
}
impl ScriptNativeObject for NativeTrustedTypeFactory {
    fn AsAny(&self) -> &dyn Any {
        self
    }
    fn ScriptIdentity(&self) -> usize {
        self as *const Self as usize
    }
    fn ScriptInterfaceName(&self) -> &'static str {
        "TrustedTypePolicyFactory"
    }
    fn VisitNativeObjects(&self, visit: &mut dyn FnMut(Rc<dyn ScriptNativeObject>)) {
        visit(self.empty_html.clone());
        visit(self.empty_script.clone());
        for policy in self.policies.borrow().values() {
            visit(policy.clone());
        }
    }
}
fn world(id: i32) -> CSPWorld {
    if id == 0 {
        CSPWorld::Main
    } else {
        CSPWorld::Isolated(id as u64)
    }
}
fn location(runtime: &dyn JavaScriptHostRuntime) -> CSPSourceLocation {
    runtime
        .CaptureSourceLocation(0)
        .map_or(CSPSourceLocation::Unknown, |location| {
            CSPSourceLocation::Script {
                url: location.source_name,
                line: location.line.min(u32::MAX as usize) as u32,
                column: location.column.min(u32::MAX as usize) as u32,
            }
        })
}
fn string(
    runtime: &mut dyn JavaScriptHostRuntime,
    value: &HostValue,
) -> Result<Rc<[u16]>, HostResult> {
    let result = runtime.ToString(value);
    if result.exception.is_some() || result.thrown.is_some() {
        return Err(result);
    }
    runtime.PrimitiveString(&result.value).ok_or_else(|| {
        HostResult::Failure(
            JavaScriptExceptionKind::kTypeError,
            "DOMString conversion failed",
        )
    })
}
#[derive(Clone, Copy)]
enum TrustedTypeKind {
    Html,
    Script,
    ScriptUrl,
}
impl TrustedTypeKind {
    fn callback<'a>(self, policy: &'a NativeTrustedTypePolicy) -> Option<&'a Value> {
        match self {
            Self::Html => policy.create_html.as_ref(),
            Self::Script => policy.create_script.as_ref(),
            Self::ScriptUrl => policy.create_script_url.as_ref(),
        }
    }
    fn callback_name(self) -> &'static str {
        match self {
            Self::Html => "createHTML",
            Self::Script => "createScript",
            Self::ScriptUrl => "createScriptURL",
        }
    }
}
fn invoke_policy(
    policy: &NativeTrustedTypePolicy,
    kind: TrustedTypeKind,
    input: Rc<[u16]>,
    args: &[HostValue],
    runtime: &mut dyn JavaScriptHostRuntime,
) -> Result<Option<Rc<[u16]>>, HostResult> {
    let Some(callback) = kind.callback(policy) else {
        let message = format!(
            "TrustedTypePolicyOptions did not specify a '{}' member.",
            kind.callback_name()
        );
        return Err(HostResult::Failure(
            JavaScriptExceptionKind::kTypeError,
            &message,
        ));
    };
    let mut arguments = Vec::with_capacity(args.len() + 1);
    arguments.push(HostValue::String16(input));
    arguments.extend_from_slice(args);
    let result = runtime.CallValue(
        &host(callback),
        &HostValue::Null(JavaScriptNull),
        &arguments,
    );
    if result.exception.is_some() || result.thrown.is_some() {
        return Err(result);
    }
    if runtime.IsNull(&result.value) || runtime.IsUndefined(&result.value) {
        return Ok(None);
    }
    let output = string(runtime, &result.value)?;
    if matches!(kind, TrustedTypeKind::ScriptUrl) {
        // CreateURLCallback returns USVString, so replace unpaired UTF-16
        // surrogates after the callback's ordinary ToString conversion.
        return Ok(Some(
            String::from_utf16_lossy(&output)
                .encode_utf16()
                .collect::<Vec<_>>()
                .into(),
        ));
    }
    Ok(Some(output))
}
impl AbortBindings {
    fn TrustedTypeFactory(&self, world: i32) -> Rc<NativeTrustedTypeFactory> {
        if let Some(factory) = self.trusted_types.borrow().get(&world).cloned() {
            return factory;
        }
        let factory = Rc::new(NativeTrustedTypeFactory {
            world,
            policies: Default::default(),
            empty_html: Rc::new(NativeTrustedHTML(Rc::from([]))),
            empty_script: Rc::new(NativeTrustedScript(Rc::from([]))),
        });
        self.trusted_types
            .borrow_mut()
            .insert(world, factory.clone());
        self.Trace(factory.clone());
        self.Publish([C::Root {
            id: self.EnsureObject(factory.clone()),
            reason: 12,
            rooted: !self.destroyed.get(),
        }]);
        factory
    }
    fn TrustedValue(&self, native: Rc<dyn ScriptNativeObject>) -> HostValue {
        self.Trace(native.clone());
        self.NativeValue(self.EnsureObject(native))
    }
    pub(in crate::dom_bindings) fn CheckExecCommandValue(
        &self,
        input: &HostValue,
        html_sink: bool,
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> Result<String, HostResult> {
        // Document's optional String-or-TrustedHTML union does not check
        // missing/null values. Native identity is checked before DOMString.
        if runtime.IsNull(input) || runtime.IsUndefined(input) {
            return Ok(String::new());
        }
        if let Some(html) = self.NativeFromValue::<NativeTrustedHTML>(input) {
            return Ok(String::from_utf16_lossy(&html.0));
        }
        let original = string(runtime, input)?;
        if !html_sink {
            return Ok(String::from_utf16_lossy(&original));
        }
        let Some(document) = self.document.borrow().upgrade() else {
            return Ok(String::from_utf16_lossy(&original));
        };
        let policy = document.borrow().GetDocument().ContentSecurityPolicy();
        let world_id = runtime.CallbackContext().WorldId();
        let world = world(world_id);
        if policy.ShouldBypassMainWorldForTrustedTypes(world)
            || !policy.RequiresTrustedTypesForScript(world)
        {
            return Ok(String::from_utf16_lossy(&original));
        }
        let factory = self.TrustedTypeFactory(world_id);
        let default = factory
            .policies
            .borrow()
            .get(&"default".encode_utf16().collect::<Vec<_>>())
            .cloned();
        let message = if let Some(default) = default {
            if default.create_html.is_some() {
                let args = [
                    HostValue::String("TrustedHTML".into()),
                    HostValue::String("Document execCommand".into()),
                ];
                if let Some(checked) = invoke_policy(
                    &default,
                    TrustedTypeKind::Html,
                    original.clone(),
                    &args,
                    runtime,
                )? {
                    return Ok(String::from_utf16_lossy(&checked));
                }
                "This document requires 'TrustedHTML' assignment and the 'default' policy failed to execute."
            } else {
                "This document requires 'TrustedHTML' assignment and no 'default' policy for 'TrustedHTML' has been defined."
            }
        } else {
            "This document requires 'TrustedHTML' assignment."
        };
        document
            .borrow()
            .GetDocument()
            .CountUse(dom::use_counter::WebFeature::kTrustedTypesAssignmentError);
        if policy.AllowTrustedTypeAssignmentFailure(
            message,
            &original,
            "Document execCommand",
            world,
            location(runtime),
        ) {
            Ok(String::from_utf16_lossy(&original))
        } else {
            Err(HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                message,
            ))
        }
    }
    pub(super) fn InvokeTrustedTypes(
        &self,
        receiver: u64,
        member: &str,
        args: &[HostValue],
        runtime: &mut dyn JavaScriptHostRuntime,
    ) -> HostResult {
        let error = || {
            HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "Illegal Trusted Types invocation",
            )
        };
        if member == "__trustedTypesFactory" {
            if self.destroyed.get() {
                return error();
            }
            let Some(document) = self.document.borrow().upgrade() else {
                return error();
            };
            if receiver != document.borrow().GetDocument().RootHandle().Id() {
                return error();
            }
            return HostResult {
                value: self
                    .TrustedValue(self.TrustedTypeFactory(runtime.CallbackContext().WorldId())),
                ..Default::default()
            };
        }
        if matches!(
            member,
            "__trustedTypesIsHTML" | "__trustedTypesIsScript" | "__trustedTypesIsScriptURL"
        ) {
            return HostResult {
                value: HostValue::Boolean(args.first().is_some_and(|value| {
                    match member {
                        "__trustedTypesIsHTML" => {
                            self.NativeFromValue::<NativeTrustedHTML>(value).is_some()
                        }
                        "__trustedTypesIsScript" => {
                            self.NativeFromValue::<NativeTrustedScript>(value).is_some()
                        }
                        _ => self
                            .NativeFromValue::<NativeTrustedScriptURL>(value)
                            .is_some(),
                    }
                })),
                ..Default::default()
            };
        }
        if matches!(
            member,
            "__trustedTypesHTMLString"
                | "__trustedTypesScriptString"
                | "__trustedTypesScriptURLString"
        ) {
            let value = HostValue::Object(HostObjectRef { id: receiver });
            let output = match member {
                "__trustedTypesHTMLString" => self
                    .NativeFromValue::<NativeTrustedHTML>(&value)
                    .map(|value| value.0.clone()),
                "__trustedTypesScriptString" => self
                    .NativeFromValue::<NativeTrustedScript>(&value)
                    .map(|value| value.0.clone()),
                _ => self
                    .NativeFromValue::<NativeTrustedScriptURL>(&value)
                    .map(|value| value.0.clone()),
            };
            let Some(output) = output else {
                return error();
            };
            return HostResult {
                value: HostValue::String16(output),
                ..Default::default()
            };
        }
        if self.destroyed.get() {
            return error();
        }
        if matches!(
            member,
            "__trustedTypesPolicyName"
                | "__trustedTypesCreateHTML"
                | "__trustedTypesCreateScript"
                | "__trustedTypesCreateScriptURL"
        ) {
            let Some(policy) = self.NativeFromValue::<NativeTrustedTypePolicy>(&HostValue::Object(
                HostObjectRef { id: receiver },
            )) else {
                return error();
            };
            if member == "__trustedTypesPolicyName" {
                return HostResult {
                    value: HostValue::String16(policy.name.clone()),
                    ..Default::default()
                };
            }
            let input = match string(runtime, args.first().unwrap_or(&HostValue::default())) {
                Ok(input) => input,
                Err(error) => return error,
            };
            let kind = match member {
                "__trustedTypesCreateHTML" => TrustedTypeKind::Html,
                "__trustedTypesCreateScript" => TrustedTypeKind::Script,
                _ => TrustedTypeKind::ScriptUrl,
            };
            return match invoke_policy(
                &policy,
                kind,
                input,
                args.get(1..).unwrap_or_default(),
                runtime,
            ) {
                Ok(output) => {
                    let output = output.unwrap_or_else(|| Rc::from([]));
                    let native: Rc<dyn ScriptNativeObject> = match kind {
                        TrustedTypeKind::Html => Rc::new(NativeTrustedHTML(output)),
                        TrustedTypeKind::Script => Rc::new(NativeTrustedScript(output)),
                        TrustedTypeKind::ScriptUrl => Rc::new(NativeTrustedScriptURL(output)),
                    };
                    HostResult {
                        value: self.TrustedValue(native),
                        ..Default::default()
                    }
                }
                Err(error) => error,
            };
        }
        let Some(factory) =
            self.NativeFromValue::<NativeTrustedTypeFactory>(&HostValue::Object(HostObjectRef {
                id: receiver,
            }))
        else {
            return error();
        };
        match member {
            "__trustedTypesFactoryBrand" => HostResult::default(),
            "__trustedTypesEmptyHTML" => HostResult {
                value: self.TrustedValue(factory.empty_html.clone()),
                ..Default::default()
            },
            "__trustedTypesEmptyScript" => HostResult {
                value: self.TrustedValue(factory.empty_script.clone()),
                ..Default::default()
            },
            "__trustedTypesDefaultPolicy" => HostResult {
                value: factory
                    .policies
                    .borrow()
                    .get(&"default".encode_utf16().collect::<Vec<_>>())
                    .cloned()
                    .map_or(HostValue::Null(JavaScriptNull), |policy| {
                        self.TrustedValue(policy)
                    }),
                ..Default::default()
            },
            "__trustedTypesCreatePolicy" => {
                let name = match string(runtime, args.first().unwrap_or(&HostValue::default())) {
                    Ok(name) => name,
                    Err(error) => return error,
                };
                let mut callback = |index: usize, name: &'static str| {
                    let callback = args.get(index).cloned().unwrap_or_default();
                    if runtime.IsUndefined(&callback) {
                        Ok(None)
                    } else if runtime.IsCallable(&callback) {
                        Ok(Some(value(callback)))
                    } else {
                        Err(HostResult::Failure(
                            JavaScriptExceptionKind::kTypeError,
                            name,
                        ))
                    }
                };
                let create_html = match callback(1, "createHTML is not a callback") {
                    Ok(callback) => callback,
                    Err(error) => return error,
                };
                let create_script = match callback(2, "createScript is not a callback") {
                    Ok(callback) => callback,
                    Err(error) => return error,
                };
                let create_script_url = match callback(3, "createScriptURL is not a callback") {
                    Ok(callback) => callback,
                    Err(error) => return error,
                };
                let Some(document) = self.document.borrow().upgrade() else {
                    return error();
                };
                let policy_owner = document.borrow().GetDocument().ContentSecurityPolicy();
                document
                    .borrow()
                    .GetDocument()
                    .CountUse(dom::use_counter::WebFeature::kTrustedTypesCreatePolicy);
                if name.is_empty() {
                    document.borrow().GetDocument().CountUse(
                        dom::use_counter::WebFeature::kTrustedTypesCreatePolicyWithEmptyName,
                    );
                }
                let duplicate = factory.policies.borrow().contains_key(name.as_ref());
                let (allowed, details) = policy_owner.AllowTrustedTypePolicy(
                    &name,
                    duplicate,
                    world(factory.world),
                    location(runtime),
                );
                if !allowed
                    || duplicate && name.as_ref() == "default".encode_utf16().collect::<Vec<_>>()
                {
                    let message = if duplicate && name.as_ref() == "default".encode_utf16().collect::<Vec<_>>() || details == dom::content_security_policy::TrustedTypePolicyDetails::DisallowedDuplicateName { format!("Policy with name \"{}\" already exists.", String::from_utf16_lossy(&name)) } else { format!("Policy \"{}\" disallowed.", String::from_utf16_lossy(&name)) };
                    return HostResult::Failure(JavaScriptExceptionKind::kTypeError, &message);
                }
                let policy = Rc::new(NativeTrustedTypePolicy {
                    name: name.clone(),
                    create_html,
                    create_script,
                    create_script_url,
                });
                document
                    .borrow()
                    .GetDocument()
                    .CountUse(dom::use_counter::WebFeature::kTrustedTypesPolicyCreated);
                if name.as_ref() == "default".encode_utf16().collect::<Vec<_>>() {
                    document
                        .borrow()
                        .GetDocument()
                        .CountUse(dom::use_counter::WebFeature::kTrustedTypesDefaultPolicyCreated);
                }
                factory
                    .policies
                    .borrow_mut()
                    .insert(name.to_vec(), policy.clone());
                self.Trace(factory);
                HostResult {
                    value: self.TrustedValue(policy),
                    ..Default::default()
                }
            }
            _ => error(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
    fn run(enforce: bool, source: &str) -> Vec<String> {
        let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
            "<html><body></body></html>",
        )));
        let header = "require-trusted-types-for 'script'; trusted-types safe default".to_owned();
        document
            .borrow()
            .GetDocument()
            .AddResponseContentSecurityPolicies(
                "https://example.test/",
                if enforce {
                    std::slice::from_ref(&header)
                } else {
                    &[]
                },
                if enforce {
                    &[]
                } else {
                    std::slice::from_ref(&header)
                },
            );
        let written = Rc::new(RefCell::new(Vec::new()));
        let output = written.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::WithHost(
            document.clone(),
            Box::new(|_| {}),
            services::DOMBindingsHost {
                execute_document_editor_command: Some(Rc::new(move |name, check, runtime| {
                    match check(name.eq_ignore_ascii_case("InsertHTML"), runtime) {
                        Ok(value) => {
                            output.borrow_mut().push(value);
                            HostResult {
                                value: HostValue::Boolean(true),
                                ..Default::default()
                            }
                        }
                        Err(error) => error,
                    }
                })),
                ..Default::default()
            },
        )));
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings);
        let bootstrap = runtime.Evaluate(&realm, crate::DOMBootstrapSource(), "bootstrap.js");
        assert!(bootstrap.Succeeded(), "{:?}", bootstrap.exception);
        let result = runtime.Evaluate(&realm, source, "trusted-types.js");
        assert!(result.Succeeded(), "{:?}", result.exception);
        assert!(
            !document
                .borrow()
                .GetDocument()
                .ContentSecurityPolicy()
                .TakeLocalViolationDeliveryEffects()
                .is_empty(),
            "sink/policy violations use the Document CSP owner"
        );
        let result = written.borrow().clone();
        result
    }
    #[test]
    fn exec_command_trusted_html_native_brand_and_default_policy() {
        let values = run(
            true,
            r#"
            function check(v,s){if(!v)throw Error(s)}
            function typeError(fn){try{fn()}catch(e){check(e instanceof TypeError,'TypeError');return}throw Error('expected TypeError')}
            check(trustedTypes===trustedTypes && trustedTypes.defaultPolicy===null,'Window factory owner');
            typeError(()=>new TrustedHTML());
            typeError(()=>TrustedHTML.prototype.toString.call({}));
            typeError(()=>trustedTypes.createPolicy('forbidden',{createHTML:s=>s}));
            typeError(()=>document.execCommand('insertHTML',false,'plain'));
            check(document.execCommand('insertHTML') && document.execCommand('insertHTML',false,null) && document.execCommand('insertHTML',false,undefined),'null optional union');
            check(document.execCommand('fontName',false,'serif'),'non HTML value');
            const policy=trustedTypes.createPolicy('safe',{createHTML:(s,...rest)=>{check(rest[0]===7,'policy extra callback argument');return '<b>'+s+'</b>'}});
            check(policy.name==='safe' && policy instanceof TrustedTypePolicy,'native policy');
            const html=policy.createHTML('x',7);
            check(trustedTypes.isHTML(html) && html instanceof TrustedHTML && String(html)==='<b>x</b>' && html.toJSON()==='<b>x</b>','native HTML value');
            html.toString=()=>{throw Error('must not stringify native union')};
            check(document.execCommand('INSERTHTML',false,html),'native union bypass');
            const forged=Object.create(TrustedHTML.prototype);
            check(!trustedTypes.isHTML(forged),'prototype is not authority');
            typeError(()=>document.execCommand('insertHTML',false,forged));
            check(trustedTypes.isHTML(trustedTypes.emptyHTML) && trustedTypes.emptyHTML===trustedTypes.emptyHTML,'empty native HTML');
            check(document.execCommand('insertHTML',false,trustedTypes.emptyHTML),'empty native union');
            const thrown={marker:1};let calls=0;
            const defaultPolicy=trustedTypes.createPolicy('default',{createHTML:(input,type,sink)=>{calls++;check(type==='TrustedHTML'&&sink==='Document execCommand','source callback args');if(input==='throw')throw thrown;if(input==='null-result')return null;return '<i>'+input+'</i>'}});
            check(trustedTypes.defaultPolicy===defaultPolicy,'retained default policy');
            check(document.execCommand('insertHTML',false,'clean'),'default transform');
            typeError(()=>document.execCommand('insertHTML',false,'null-result'));
            try{document.execCommand('insertHTML',false,'throw');throw Error('lost callback exception')}catch(e){check(e===thrown,'original thrown identity')}
            const before=calls;check(document.execCommand('insertHTML')&&document.execCommand('insertHTML',false,null)&&document.execCommand('bold'),'missing/null/non HTML skip');check(calls===before,'no extra sink callback');
            typeError(()=>trustedTypes.createPolicy('default',{createHTML:s=>s}));
        "#,
        );
        assert_eq!(
            values,
            [
                "",
                "",
                "",
                "serif",
                "<b>x</b>",
                "",
                "<i>clean</i>",
                "",
                "",
                ""
            ]
        );
    }
    #[test]
    fn exec_command_report_only_missing_default_callback_preserves_value() {
        let values = run(
            false,
            r#"
            function check(v,s){if(!v)throw Error(s)}
            const missing=trustedTypes.createPolicy('default',{});
            check(document.execCommand('insertHTML',false,'report-only'),'report-only permits original value');
            let failed=false;try{missing.createHTML('x')}catch(e){failed=e instanceof TypeError}check(failed,'direct createHTML still requires callback');
        "#,
        );
        assert_eq!(values, ["report-only"]);
    }
}
