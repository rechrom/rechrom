//! Browser legacy RegExp statics observed through public native RegExp calls.
//! Matching remains in QuickJS; this host service owns only the legacy record.
use super::native::*;
use std::{cell::RefCell, rc::Rc};
struct Accessor {
    state: Rc<RefCell<Vec<Value>>>,
    index: usize,
    set: bool,
}
impl HostCallable for Accessor {
    fn call(
        &self,
        ctx: &mut Context,
        _: &Value,
        args: &[Value],
        _: &[Value],
        _: &Value,
    ) -> JsResult<Value> {
        if self.set {
            if self.index == 0 {
                let value = ctx.to_js_string(args.first().unwrap_or(&Value::Undefined))?;
                self.state.borrow_mut()[0] = Value::Str(value);
            }
            Ok(Value::Undefined)
        } else {
            Ok(self.state.borrow()[self.index].clone())
        }
    }
}
struct Exec {
    state: Rc<RefCell<Vec<Value>>>,
}
impl HostCallable for Exec {
    fn call(
        &self,
        ctx: &mut Context,
        this: &Value,
        args: &[Value],
        data: &[Value],
        _: &Value,
    ) -> JsResult<Value> {
        // The original source getter performs brand validation before coercion.
        ctx.call(data[1].clone(), this.clone(), &[])?;
        let input = Value::Str(ctx.to_js_string(args.first().unwrap_or(&Value::Undefined))?);
        let result = ctx.call(data[0].clone(), this.clone(), &[input.clone()])?;
        if result.is_null() {
            return Ok(result);
        }
        let key = PropKey::from_string(&ctx.intern("length"));
        let length = ctx.get_property(&result, &key)?.as_number().unwrap_or(0.0) as usize;
        let key = PropKey::from_string(&ctx.intern("index"));
        let index = ctx.get_property(&result, &key)?.as_number().unwrap_or(0.0);
        let matched = ctx.get_property(&result, &PropKey::Index(0))?;
        let key = PropKey::from_string(&ctx.intern("length"));
        let width = ctx.get_property(&matched, &key)?.as_number().unwrap_or(0.0);
        let left = ctx.call(
            data[2].clone(),
            input.clone(),
            &[Value::Int(0), Value::number(index)],
        )?;
        let right = ctx.call(
            data[2].clone(),
            input.clone(),
            &[Value::number(index + width)],
        )?;
        let empty = Value::Str(ctx.intern(""));
        let mut record = vec![empty.clone(); 14];
        record[0] = input;
        record[1] = matched;
        record[3] = left;
        record[4] = right;
        for i in 1..length {
            let value = ctx.get_property(&result, &PropKey::Index(i as u32))?;
            let value = if matches!(value, Value::Undefined) {
                empty.clone()
            } else {
                value
            };
            record[2] = value.clone();
            if i <= 9 {
                record[i + 4] = value;
            }
        }
        *self.state.borrow_mut() = record;
        Ok(result)
    }
}
struct NativeSource;
impl HostCallable for NativeSource {
    fn call(
        &self,
        ctx: &mut Context,
        this: &Value,
        args: &[Value],
        data: &[Value],
        _: &Value,
    ) -> JsResult<Value> {
        if Value::strict_equals(this, &data[1]) {
            Ok(Value::Str(ctx.intern("function exec() { [native code] }")))
        } else {
            ctx.call(data[0].clone(), this.clone(), args)
        }
    }
}
pub(super) fn install(ctx: &mut Context) -> JsResult<()> {
    let empty = Value::Str(ctx.intern(""));
    let state = Rc::new(RefCell::new(vec![empty; 14]));
    let ctor = ctx.global_value("RegExp");
    let constructor = ctor.as_object().expect("RegExp constructor").clone();
    let key = PropKey::from_string(&ctx.intern("prototype"));
    let proto = ctx.get_property(&ctor, &key)?;
    let prototype = proto.as_object().unwrap().clone();
    let key = PropKey::from_string(&ctx.intern("exec"));
    let original = ctx.get_property(&proto, &key)?;
    let brand = ctx.eval("Object.getOwnPropertyDescriptor(RegExp.prototype,'source').get")?;
    let slice = ctx.eval("String.prototype.slice")?;
    let exec = Value::Object(ctx.new_host_function(
        Rc::new(Exec {
            state: state.clone(),
        }),
        vec![original, brand, slice],
        "exec",
        1,
    ));
    ctx.define_value(&prototype, "exec", exec.clone(), PropFlags::C_W);
    let mut names = vec![
        ("input".to_owned(), 0),
        ("$_".into(), 0),
        ("lastMatch".into(), 1),
        ("$&".into(), 1),
        ("lastParen".into(), 2),
        ("$+".into(), 2),
        ("leftContext".into(), 3),
        ("$`".into(), 3),
        ("rightContext".into(), 4),
        ("$'".into(), 4),
    ];
    for i in 1..=9 {
        names.push((format!("${i}"), i + 4));
    }
    for (name, index) in names {
        let get = Value::Object(ctx.new_host_function(
            Rc::new(Accessor {
                state: state.clone(),
                index,
                set: false,
            }),
            Vec::new(),
            &format!("get {name}"),
            0,
        ));
        let set = Value::Object(ctx.new_host_function(
            Rc::new(Accessor {
                state: state.clone(),
                index,
                set: true,
            }),
            Vec::new(),
            &format!("set {name}"),
            1,
        ));
        let key = PropKey::from_string(&ctx.intern(&name));
        ctx.define_own(
            &constructor,
            key,
            Property::accessor(Some(get), Some(set), PropFlags::CONFIGURABLE),
        );
    }
    // Native source spelling is a browser compatibility surface, outside matching.
    let function_proto = ctx.eval("Function.prototype")?;
    let key = PropKey::from_string(&ctx.intern("toString"));
    let original = ctx.get_property(&function_proto, &key)?;
    let source = Value::Object(ctx.new_host_function(
        Rc::new(NativeSource),
        vec![original, exec],
        "toString",
        0,
    ));
    ctx.define_value(
        function_proto.as_object().unwrap(),
        "toString",
        source,
        PropFlags::C_W,
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        javascript_runtime::*,
        quickjs_javascript_runtime::{JsValue, QuickJsJavaScriptRuntime},
    };
    use std::{cell::RefCell, rc::Rc};
    struct Host;
    impl JavaScriptHostBindings for Host {
        fn GlobalNames(&self) -> Vec<String> {
            Vec::new()
        }
        fn Invoke(&mut self, _: &HostCall<'_>) -> HostResult {
            HostResult {
                handled: false,
                ..Default::default()
            }
        }
    }
    #[test]
    fn legacy_regexp_state_matches_original_v8_27_observations() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(Rc::new(RefCell::new(Host)));
        let result = runtime.Evaluate(
            &realm,
            include_str!("../tests/fixtures/regexp_statics.js"),
            "regexp-reference.js",
        );
        assert!(result.Succeeded(), "{:?}", result.exception);
        let text = result
            .value
            .Implementation::<JsValue>()
            .unwrap()
            .as_string()
            .unwrap()
            .to_string_lossy();
        assert_eq!(
            text,
            include_str!("../tests/fixtures/regexp_statics_v8.json").trim()
        );
    }
    #[test]
    fn legacy_regexp_state_isolated_and_retained_across_evaluations() {
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let first = runtime.CreateRealm(Rc::new(RefCell::new(Host)));
        let second = runtime.CreateRealm(Rc::new(RefCell::new(Host)));
        assert!(runtime
            .Evaluate(&first, "/(first)/.test('xfirsty')", "first.js")
            .Succeeded());
        assert!(runtime
            .Evaluate(
                &second,
                "if(RegExp.$1 !== '') throw Error('leaked'); /(second)/.test('second');",
                "second.js"
            )
            .Succeeded());
        assert!(runtime.Evaluate(&first,"if(RegExp.$1 !== 'first') throw Error('lost'); /(never)/.test('no'); if(RegExp.$1 !== 'first') throw Error('cleared');","retained.js").Succeeded());
    }
}
