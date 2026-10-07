//! V8 console formatter behavior implemented over QuickJS's native callables.
use super::native::*;
use std::rc::Rc;

struct Method {
    start: Option<usize>,
    fast_assert: bool,
    context: bool,
}
impl HostCallable for Method {
    fn call(
        &self,
        ctx: &mut Context,
        _: &Value,
        args: &[Value],
        data: &[Value],
        _: &Value,
    ) -> JsResult<Value> {
        if self.context {
            if let Some(name) = args.first() {
                ctx.to_js_string(name)?;
            }
            return new_console(ctx, true, data);
        }
        if self.fast_assert && args.first().is_some_and(Value::to_boolean) {
            return Ok(Value::Undefined);
        }
        if let Some(start) = self.start {
            format(ctx, args, start, data)?;
        }
        Ok(Value::Undefined)
    }
}

fn format(ctx: &mut Context, args: &[Value], start: usize, intrinsics: &[Value]) -> JsResult<()> {
    if args.len() < start + 2 {
        return Ok(());
    }
    let Some(text) = args[start].as_string() else {
        return Ok(());
    };
    let mut states = vec![(
        text.to_string_lossy().encode_utf16().collect::<Vec<_>>(),
        0usize,
    )];
    let mut index = start + 1;
    while !states.is_empty() && index < args.len() {
        let top = states.len() - 1;
        let (text, offset) = &mut states[top];
        let Some(percent) = text[*offset..]
            .iter()
            .position(|c| *c == b'%' as u16)
            .map(|i| i + *offset)
        else {
            states.pop();
            continue;
        };
        if percent + 1 == text.len() {
            states.pop();
            continue;
        }
        *offset = percent;
        let specifier = text[percent + 1];
        match specifier {
            c if c == b'd' as u16 || c == b'i' as u16 || c == b'f' as u16 => {
                if !args[index].is_symbol() {
                    let function = intrinsics[usize::from(c == b'f' as u16)].clone();
                    ctx.call(
                        function,
                        Value::Undefined,
                        &[args[index].clone(), Value::Int(10)],
                    )?;
                }
                index += 1;
                states[top].1 += 2;
            }
            c if c == b's' as u16 => {
                let converted = ctx.call(
                    intrinsics[2].clone(),
                    Value::Undefined,
                    &[args[index].clone()],
                )?;
                index += 1;
                states[top].1 += 2;
                states.push((
                    converted
                        .as_string()
                        .expect("String intrinsic")
                        .to_string_lossy()
                        .encode_utf16()
                        .collect(),
                    0,
                ));
            }
            c if c == b'c' as u16 || c == b'o' as u16 || c == b'O' as u16 || c == b'_' as u16 => {
                index += 1;
                states[top].1 += 2;
            }
            c if c == b'%' as u16 => states[top].1 += 2,
            _ => states[top].1 += 1,
        }
    }
    Ok(())
}

fn new_console(ctx: &mut Context, nested: bool, intrinsics: &[Value]) -> JsResult<Value> {
    let prototype = ctx.new_object();
    let object = ctx.new_object();
    ctx.set_prototype_of(&object, Some(prototype))?;
    let names: &[&str] = if nested {
        &[
            "dir",
            "dirXml",
            "table",
            "groupEnd",
            "clear",
            "count",
            "countReset",
            "profile",
            "profileEnd",
            "debug",
            "error",
            "info",
            "log",
            "warn",
            "trace",
            "group",
            "groupCollapsed",
            "assert",
            "time",
            "timeLog",
            "timeEnd",
            "timeStamp",
        ]
    } else {
        &[
            "debug",
            "error",
            "info",
            "log",
            "warn",
            "dir",
            "dirxml",
            "table",
            "trace",
            "group",
            "groupCollapsed",
            "groupEnd",
            "clear",
            "count",
            "countReset",
            "assert",
            "profile",
            "profileEnd",
            "time",
            "timeLog",
            "timeEnd",
            "timeStamp",
        ]
    };
    for &name in names {
        let start = match name {
            "assert" => Some(1),
            "debug" | "error" | "info" | "log" | "warn" | "trace" | "group" | "groupCollapsed" => {
                Some(0)
            }
            _ => None,
        };
        let function = ctx.new_host_function(
            Rc::new(Method {
                start,
                fast_assert: !nested && name == "assert",
                context: false,
            }),
            intrinsics.to_vec(),
            name,
            u32::from(nested),
        );
        ctx.define_value(&object, name, Value::Object(function), PropFlags::C_W_E);
    }
    if !nested {
        let function = ctx.new_host_function(
            Rc::new(Method {
                start: None,
                fast_assert: false,
                context: true,
            }),
            intrinsics.to_vec(),
            "context",
            1,
        );
        ctx.define_value(
            &object,
            "context",
            Value::Object(function),
            PropFlags::C_W_E,
        );
        let tag = PropKey::Sym(ctx.well_known.get(WellKnownSymbol::ToStringTag).clone());
        let value = Value::Str(ctx.intern("console"));
        ctx.define_own(&object, tag, Property::data(value, PropFlags::CONFIGURABLE));
    }
    Ok(Value::Object(object))
}

pub(super) fn install(ctx: &mut Context) -> JsResult<()> {
    let intrinsics = [
        ctx.global_value("parseInt"),
        ctx.global_value("parseFloat"),
        ctx.global_value("String"),
    ];
    let console = new_console(ctx, false, &intrinsics)?;
    let global = ctx.global();
    ctx.define_value(&global, "console", console, PropFlags::C_W);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_console_preserves_formatter_effects_shape_and_intrinsics() {
        let mut context = Context::new();
        install(&mut context).unwrap();
        let result=context.eval(r#"
            function check(v,s){if(!v)throw Error(s)}
            var effects=[];function obj(label,result){return {toString(){effects.push(label);return result}}}
            console.log('%s:%i:%f',obj('outer','%i'),obj('nested','42'),obj('integer','3'),obj('float','4.5'));
            console.log('%o%c%O%_%% %q %s',obj('skip1','x'),obj('skip2','x'),obj('skip3','x'),obj('skip4','x'),obj('string','done'));
            console.assert(true,'%s',obj('unreachable','x'));console.assert(false,'%s',obj('assert','x'));
            console.context().assert(true,'%s',obj('context-assert','x'));
            console.log('%d',Symbol('s'));console.log('%s',Symbol('s'));
            try{console.context(Symbol('s'))}catch(e){effects.push(e.name)}
            var originalString=String, originalParseInt=parseInt;String=()=>{throw Error('wrong String')};parseInt=()=>{throw Error('wrong parseInt')};console.log('%s:%i',obj('intrinsic-string','s'),obj('intrinsic-int','1'));String=originalString;parseInt=originalParseInt;
            console.log(obj('unformatted','x'),obj('unformatted-argument','x'));console.time(obj('timer-unconverted','x'));
            check(JSON.stringify(effects)===JSON.stringify(['outer','nested','integer','float','string','assert','context-assert','TypeError','intrinsic-string','intrinsic-int']),'conversion order');
            check(Object.keys(console).join(',')==='debug,error,info,log,warn,dir,dirxml,table,trace,group,groupCollapsed,groupEnd,clear,count,countReset,assert,profile,profileEnd,time,timeLog,timeEnd,timeStamp,context','global keys');
            check(Object.prototype.toString.call(console)==='[object console]' && Object.getPrototypeOf(console)!==Object.prototype && console.constructor===Object,'global prototype');
            for(const name of Object.keys(console)){check(console[name].name===name && console[name].length===(name==='context'?1:0),'method '+name);check(!Object.hasOwn(console[name],'prototype'),'not constructible');}
            var nested=console.context('name');check(Object.keys(nested).join(',')==='dir,dirXml,table,groupEnd,clear,count,countReset,profile,profileEnd,debug,error,info,log,warn,trace,group,groupCollapsed,assert,time,timeLog,timeEnd,timeStamp','context keys');
            check(Object.prototype.toString.call(nested)==='[object Object]' && nested.constructor===Object && nested!==console.context(),'context object');
            for(const name of Object.keys(nested))check(nested[name].length===1,'context arity');
            var thrown=false;try{console.log('%s',{toString(){throw new RangeError('conversion')}})}catch(e){thrown=e instanceof RangeError}check(thrown,'conversion error');
            var before=effects.length;console.log('trailing%');console.log('%s');check(effects.length===before,'missing conversion argument');
            var descriptor=Object.getOwnPropertyDescriptor(globalThis,'console');check(!descriptor.enumerable && descriptor.writable && descriptor.configurable,'global descriptor');
        "#);
        assert!(result.is_ok(), "{result:?}");
    }
}
