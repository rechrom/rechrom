//! Exact original repository V8 public exception fields, independently captured
//! by quickjs_exception_metadata_oracle.cc. Stack stays an engine-owned value.
use javascript::{javascript_runtime::*, quickjs_javascript_runtime::QuickJsJavaScriptRuntime};
use std::{cell::RefCell, rc::Rc};
#[derive(Default)]
struct Host {
    captured: Option<JavaScriptFunction>,
}
impl JavaScriptHostBindings for Host {
    fn GlobalNames(&self) -> Vec<String> {
        ["nativeGetter", "nativeObject", "capture"]
            .map(str::to_owned)
            .to_vec()
    }
    fn Invoke(&mut self, call: &HostCall<'_>) -> HostResult {
        match (call.operation, call.receiver, call.member) {
            (HostOperation::kGet, 0, "nativeGetter") => {
                HostResult::Failure(JavaScriptExceptionKind::kTypeError, "native getter failure")
            }
            (HostOperation::kGet, 0, "nativeObject") => HostResult {
                value: HostValue::Object(HostObjectRef { id: 1 }),
                ..Default::default()
            },
            (HostOperation::kGet, 1, "valueOf") => HostResult {
                value: HostValue::Method(HostMethodRef {
                    receiver: 1,
                    name: "valueOf".into(),
                }),
                ..Default::default()
            },
            (HostOperation::kCall, 1, "valueOf") => HostResult::Failure(
                JavaScriptExceptionKind::kTypeError,
                "native coercion failure",
            ),
            (HostOperation::kGet, 0, "capture") => HostResult {
                value: HostValue::Method(HostMethodRef {
                    receiver: 0,
                    name: "capture".into(),
                }),
                ..Default::default()
            },
            (HostOperation::kCall, 0, "capture") => {
                if let HostValue::JavaScriptFunction(function) = &call.arguments[0] {
                    self.captured = Some(function.clone());
                } else {
                    panic!("capture requires function");
                }
                HostResult::default()
            }
            _ => HostResult {
                handled: false,
                ..Default::default()
            },
        }
    }
}
fn row(name: &str, error: &JavaScriptException) -> String {
    format!(
        "{name}\t{}\t{}\t{}\t{}\t{}",
        error.source_name, error.line, error.column, error.source_line, error.message
    )
}
#[test]
fn actual_exception_fields_match_original_cpp_v8() {
    let host = Rc::new(RefCell::new(Host::default()));
    let mut runtime = QuickJsJavaScriptRuntime::new();
    let realm = runtime.CreateRealm(host.clone());
    let cases=[
        ("primitive","\nthrow 'primitive';"),
        ("finally-caught","try {\n throw 'original';\n} finally {\n try { throw 'cleanup'; } catch(e) {}\n}"),
        ("finally-overrides","try {\n throw 'original';\n} finally {\n throw 'replacement';\n}"),
        ("nested-finally","try {\n try { throw 'nested'; } finally {var a=1;}\n} finally {var b=2;}"),
        ("caught-same","try {throw 'same'} catch(e) {}\nthrow 'same';"),
        ("crlf","var x=1;\r\n throw 'crlf';"),
        ("unicode","var s='😀'; throw 'unicode';"),
        ("separator","var x=1;  throw 'separator';"),

        ("old-error","var old = new Error('old');\n\nthrow old;"),
        ("caught-new","try { throw 'caught'; } catch(e) {}\nthrow 'new';"),
        ("rethrow","try {\n throw 'caught';\n} catch(e) {\n throw e;\n}"),
        ("finally","try {\n throw 'original';\n} finally {\n var cleanup=1;\n}"),
        ("iterator-cleanup","for(const x of {[Symbol.iterator](){return {next(){return {value:1,done:false}},return(){throw 'cleanup'}}}}){\n throw 'original';\n}"),
        ("getter","\nnativeGetter;"),
        ("coercion","\n+nativeObject;"),
        ("nested","function inner(){\n throw 'inner';\n}\nfunction outer(){inner()}\nouter();"),
    ];
    let mut rows = Vec::new();
    for (name, source) in cases {
        let result = runtime.Evaluate(&realm, source, &format!("https://test/{name}.js"));
        let error = result
            .exception
            .unwrap_or_else(|| panic!("{name} succeeded"));
        if name == "old-error" {
            assert!(
                error.stack.contains("https://test/old-error.js:1:"),
                "creation stack changed: {}",
                error.stack
            );
        }
        rows.push(row(name, &error));
    }
    assert!(runtime
        .Evaluate(
            &realm,
            "capture(function invoked(){\n throw 'called';\n});",
            "https://test/call.js"
        )
        .Succeeded());
    let callback = host.borrow().captured.clone().unwrap();
    let result = runtime.Call(&realm, &callback, &HostValue::default(), &[]);
    rows.push(row("call", &result.exception.unwrap()));
    for (name, source) in [
        (
            "microtask",
            "capture(function queued(){\n throw 'microtask';\n});",
        ),
        (
            "microtask-error",
            "capture(function queuedError(){\n throw new Error('microtask error');\n});",
        ),
        (
            "microtask-old",
            "capture(function queuedOld(){\n throw old;\n});",
        ),
    ] {
        assert!(runtime
            .Evaluate(&realm, source, &format!("https://test/{name}.js"))
            .Succeeded());
        let callback = host.borrow().captured.clone().unwrap();
        assert!(runtime.EnqueueMicrotask(&realm, &callback).Succeeded());
        runtime.PerformMicrotaskCheckpoint();
        let errors = runtime.TakePendingExceptions(&realm);
        assert_eq!(errors.len(), 1);
        rows.push(row(name, &errors[0]));
    }
    assert!(runtime
        .Evaluate(
            &realm,
            "var unthrown = new Error('unthrown');",
            "https://test/unthrown.js"
        )
        .Succeeded());
    assert!(runtime
        .Evaluate(
            &realm,
            "capture(function queuedUnthrown(){\n throw unthrown;\n});",
            "https://test/microtask-unthrown.js"
        )
        .Succeeded());
    let callback = host.borrow().captured.clone().unwrap();
    assert!(runtime.EnqueueMicrotask(&realm, &callback).Succeeded());
    runtime.PerformMicrotaskCheckpoint();
    for error in runtime.TakePendingExceptions(&realm) {
        rows.push(row("microtask-unthrown", &error));
    }
    assert!(runtime.Evaluate(&realm,"Promise.reject(old);var readStack=new Error('read stack');readStack.stack;Promise.reject(readStack);","https://test/rejection-read.js").Succeeded());
    runtime.PerformMicrotaskCheckpoint();
    for error in runtime.TakePendingExceptions(&realm) {
        rows.push(row("rejection-read", &error));
    }
    assert!(runtime.Evaluate(&realm,"var rejectionOld=new Error('rejected old');\nPromise.reject(rejectionOld);\nPromise.reject('primitive rejection');","https://test/rejection.js").Succeeded());
    runtime.PerformMicrotaskCheckpoint();
    for error in runtime.TakePendingExceptions(&realm) {
        rows.push(row("rejection", &error));
    }
    let got = rows.join("\n") + "\n";
    std::fs::write("/tmp/quickjs-exception-rust.txt", &got).unwrap();
    assert_eq!(
        got,
        include_str!("quickjs_exception_metadata_cpp_expected.tsv")
    );
}
