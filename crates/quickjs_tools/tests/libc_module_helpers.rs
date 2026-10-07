use quickjs::{quickjs::*, quickjs_header::*};
use quickjs_tools::quickjs_libc::*;
use std::{ffi::{CStr,CString},ptr};
unsafe fn show(ctx:*mut JSContext,name:&str,mut value:JSValue) {
    if JS_IsException(value)!=0 {value=JS_GetException(ctx);}
    let text=JS_ToCString(ctx,value);
    println!("{}:{}",name,if text.is_null(){"<conversion exception>".into()}else{CStr::from_ptr(text).to_string_lossy()});
    JS_FreeCString(ctx,text); JS_FreeValue(ctx,value);
}
unsafe fn inspect_json(ctx:*mut JSContext,_:JSValueConst,_:i32,argv:*mut JSValueConst)->JSValue {
    let n=js_module_test_json(ctx,*argv);
    if JS_HasException(ctx)!=0 {JS_EXCEPTION}else{JS_NewInt32(ctx,n)}
}
unsafe fn inspect_attrs(ctx:*mut JSContext,_:JSValueConst,_:i32,argv:*mut JSValueConst)->JSValue {
    if js_module_check_attributes(ctx,ptr::null_mut(),*argv)<0 {JS_EXCEPTION}else{JS_NewInt32(ctx,0)}
}
unsafe fn inspect_option(ctx:*mut JSContext,_:JSValueConst,_:i32,argv:*mut JSValueConst)->JSValue {
    let mut value=9;
    if get_bool_option(ctx,&mut value,*argv,c"enabled".as_ptr())<0 {JS_EXCEPTION}else{JS_NewInt32(ctx,value)}
}
unsafe fn evaluate(ctx:*mut JSContext,name:&str,source:&str) {
    let text=CString::new(source).unwrap();
    show(ctx,name,JS_Eval(ctx,text.as_ptr(),source.len(),c"helpers.js".as_ptr(),0));
}
fn main() {
    let filename=std::env::args().nth(1).unwrap();
    std::thread::Builder::new().stack_size(16*1024*1024).spawn(move||unsafe {
        let rt=JS_NewRuntime();JS_SetMaxStackSize(rt,4*1024*1024);
        let ctx=JS_NewContext(rt);let global=JS_GetGlobalObject(ctx);
        JS_SetPropertyStr(ctx,global,c"jsonType".as_ptr(),JS_NewCFunction(ctx,Some(inspect_json),c"jsonType".as_ptr(),1));
        JS_SetPropertyStr(ctx,global,c"attrs".as_ptr(),JS_NewCFunction(ctx,Some(inspect_attrs),c"attrs".as_ptr(),1));
        let program=CString::new("helper-oracle").unwrap();
        let file_arg=CString::new(filename.as_bytes()).unwrap();
        let mut args=[program.as_ptr().cast_mut(),file_arg.as_ptr().cast_mut()];
        js_std_add_helpers(ctx,2,args.as_mut_ptr());
        for (name,func,length) in [
            (c"getEnv",js_std_getenv as JSCFunction,1),
            (c"setEnv",js_std_setenv as JSCFunction,2),
            (c"unsetEnv",js_std_unsetenv as JSCFunction,1),
            (c"getEnviron",js_std_getenviron as JSCFunction,0),
            (c"collect",js_std_gc as JSCFunction,0),
            (c"loadText",js_std_loadFile as JSCFunction,1),
            (c"option",inspect_option as JSCFunction,1),
        ] { JS_SetPropertyStr(ctx,global,name.as_ptr(),JS_NewCFunction(ctx,Some(func),name.as_ptr(),length)); }
        if std::env::args().nth(2).as_deref()==Some("--exit") {
            JS_SetPropertyStr(ctx,global,c"stdExit".as_ptr(),JS_NewCFunction(ctx,Some(js_std_exit),c"stdExit".as_ptr(),1));
            evaluate(ctx,"exit-case",&std::env::args().nth(3).unwrap());
            std::process::exit(2);
        }
        for (name,source) in [
            ("json-types",r"[jsonType(undefined),jsonType({}),jsonType({type:'json'}),jsonType({type:'json5'}),jsonType({type:'JSON'}),jsonType({type:'json\0'})].join(',')"),
            ("attrs-values","attrs({get type(){throw Error('must not read')}})"),
            ("attrs-hidden","attrs(Object.defineProperty({[Symbol()]:1},'other',{value:1}))"),
            ("attrs-reject","try{attrs({other:1})}catch(e){e.name+':'+e.message}"),
            ("attrs-nul",r"try{attrs({'other\0':1})}catch(e){e.name+':'+e.message}"),
            ("json-getter","try{jsonType({get type(){throw Error('json getter')}})}catch(e){e.message}"),
            ("attrs-proxy","try{attrs(new Proxy({},{ownKeys(){throw Error('own keys')}}))}catch(e){e.message}"),
            ("environment",r"(()=>{let k='QJS_TRANSLATED_TOOLS_ORACLE_ENV';unsetEnv(k);let a=getEnv(k);setEnv(k,'héllo\0ignored');let b=getEnv(k),c=getEnviron()[k];unsetEnv(k);return [a===undefined,b,c,getEnv(k)===undefined].join('|')})()"),
            ("environment-coercion","(()=>{let seen=[];setEnv({toString(){seen.push('key');return ''}},{toString(){seen.push('value');return 'x'}});return seen.join(',')})()"),
            ("environment-error","try{setEnv('QJS_ORACLE_TMP',{toString(){throw Error('value')}})}catch(e){e.message}"),
            ("gc","collect()===undefined"),
            ("helpers","[typeof print,typeof console.log,typeof __loadScript,typeof performance.now,performance.now()>0,scriptArgs.slice(1).length].join(',')"),
            ("printer",r"print('embedded\0text',{answer:42},[1,2],undefined,42n);console.log('console',false);'done'"),
            ("option","[option({}),option({enabled:false}),option({enabled:'x'}),option({get enabled(){return null}})].join(',')"),
            ("option-error","try{option({get enabled(){throw Error('option getter')}})}catch(e){e.message}"),
            ("load-text","JSON.stringify([loadText(scriptArgs[1]),loadText('__quickjs_nonexistent_file__')])"),
        ] {evaluate(ctx,name,source);}
        let filename=CString::new(filename).unwrap();let mut length=999;
        let buffer=js_load_file(ctx,&mut length,filename.as_ptr());
        println!("file:{}:{}:{}",length,(!buffer.is_null()&&*buffer.add(length)==0)as i32,(!buffer.is_null()&&std::slice::from_raw_parts(buffer,4)==b"a\0b\n")as i32);
        js_free(ctx,buffer.cast());
        let buffer=js_load_file(ptr::null_mut(),&mut length,filename.as_ptr());
        println!("file-null:{}:{}",length,(!buffer.is_null()&&*buffer.add(length)==0)as i32);
        js_free_file_buffer(ptr::null_mut(),buffer,length);
        let buffer=js_load_file(ctx,&mut length,c"__quickjs_nonexistent_file__".as_ptr());
        println!("file-missing:{}",buffer.is_null()as i32);
        for (i,name) in [c"relative name.mjs",c"scheme:module",&filename,c"__quickjs_nonexistent_file__"].into_iter().enumerate() {
            let module=JS_Eval(ctx,c"export {};".as_ptr(),10,name.as_ptr(),JS_EVAL_TYPE_MODULE|JS_EVAL_FLAG_COMPILE_ONLY);
            let result=js_module_set_import_meta(ctx,module,(i>=2)as i32,(i==0)as i32);
            println!("meta-{}-result:{}",i,result);
            if result<0 {show(ctx,"meta-error",JS_EXCEPTION);}else {
                let meta=JS_GetImportMeta(ctx,JS_VALUE_GET_PTR(module).cast());
                JS_SetPropertyStr(ctx,global,c"meta".as_ptr(),meta);
                evaluate(ctx,"meta-value","JSON.stringify([meta.url,meta.main,Object.keys(meta),Object.getOwnPropertyDescriptor(meta,'main')])");
            }
            JS_FreeValue(ctx,module);
        }
        let json=JS_ParseJSON(ctx,c"{\"answer\":42}".as_ptr(),13,c"json:fixture".as_ptr());
        create_json_module(ctx,c"json:fixture".as_ptr(),json);
        let source=c"import x from 'json:fixture'; globalThis.result=x.answer;";
        let value=JS_Eval(ctx,source.as_ptr(),source.to_bytes().len(),c"json:consumer".as_ptr(),JS_EVAL_TYPE_MODULE);
        if JS_IsException(value)!=0 {show(ctx,"json-module",value);}else {JS_FreeValue(ctx,value);evaluate(ctx,"json-module","result");}
        let source=c"throw Error('reported')";
        let value=JS_Eval(ctx,source.as_ptr(),source.to_bytes().len(),c"error.js".as_ptr(),0);
        if JS_IsException(value)!=0 {js_std_dump_error(ctx);}JS_FreeValue(ctx,value);
        JS_FreeValue(ctx,global);JS_FreeContext(ctx);JS_FreeRuntime(rt);
    }).unwrap().join().unwrap();
}
