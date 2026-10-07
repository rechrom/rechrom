//! ICU4X-backed Locale service shared with the browser, without OS lookups.
//! This supplies Intl.Locale; other Intl constructors are not advertised.
use super::native::*;
use icu_locale::{Locale, LocaleExpander};

fn locale(ctx: &mut Context, _: &Value, args: &[Value], _: i32) -> JsResult<Value> {
    let tag = ctx.to_rust_string(args.first().unwrap_or(&Value::Undefined))?;
    let mut locale: Locale = tag
        .parse()
        .map_err(|_| ctx.range_error("Incorrect locale information provided"))?;
    let operation = args.get(1).and_then(Value::as_f64).unwrap_or(0.0) as u8;
    let expander = LocaleExpander::new_extended();
    if operation == 1 {
        expander.maximize(&mut locale.id);
    }
    if operation == 2 {
        expander.minimize(&mut locale.id);
    }
    let out = ctx.new_object();
    for (name, text) in [
        ("tag", locale.to_string()),
        ("baseName", locale.id.to_string()),
        ("language", locale.id.language.to_string()),
    ] {
        let value = Value::Str(ctx.intern(&text));
        ctx.define_value(&out, name, value, PropFlags::C_W_E);
    }
    for (name, text) in [
        ("script", locale.id.script.map(|x| x.to_string())),
        ("region", locale.id.region.map(|x| x.to_string())),
    ] {
        let value = text
            .map(|text| Value::Str(ctx.intern(&text)))
            .unwrap_or(Value::Undefined);
        ctx.define_value(&out, name, value, PropFlags::C_W_E);
    }
    Ok(Value::Object(out))
}

pub(super) fn install(ctx: &mut Context) -> JsResult<()> {
    let service = Value::Object(ctx.new_native_function(locale, "locale", 2, 0));
    let factory = ctx.eval(r#"(service=>{
        const records=new WeakMap();
        class Locale {
            constructor(tag){
                if(typeof tag!=='string' && (tag===null || (typeof tag!=='object' && typeof tag!=='function')))throw new TypeError('Locale identifier must be a string or object');
                records.set(this,service(''+tag,0));
            }
            toString(){return records.get(this).tag}
            get baseName(){return records.get(this).baseName}
            get language(){return records.get(this).language}
            get script(){return records.get(this).script}
            get region(){return records.get(this).region}
            maximize(){return new Locale(service(records.get(this).tag,1).tag)}
            minimize(){return new Locale(service(records.get(this).tag,2).tag)}
        }
        Object.defineProperty(Locale.prototype,Symbol.toStringTag,{value:'Intl.Locale',configurable:true});
        return Locale;
    })"#)?;
    let constructor = ctx.call(factory, Value::Undefined, &[service])?;
    let intl = ctx.new_object();
    ctx.define_value(&intl, "Locale", constructor, PropFlags::C_W);
    let global = ctx.global();
    ctx.define_value(&global, "Intl", Value::Object(intl), PropFlags::C_W);
    Ok(())
}
