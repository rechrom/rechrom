#![allow(non_snake_case)]

use dom::{Document, DOM};
use html::html_parser::{HTMLDocumentParser, ParserScript};
use javascript::javascript_runtime::{
    JavaScriptException, JavaScriptModuleResolver, JavaScriptRealm, JavaScriptResult,
    JavaScriptRuntime,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use webapi::dom_bindings::DOMJavaScriptBindings;

/// Browser-owned state for classic parser-blocking script tasks. Resource
/// loading and script classification remain with the browser scheduler.
pub struct ParserScriptTasks {
    document: Rc<RefCell<DOM>>,
    bindings: Rc<RefCell<DOMJavaScriptBindings>>,
    running: Rc<Cell<bool>>,
    written_markup: Rc<RefCell<String>>,
    pending_host_errors: Option<Rc<dyn Fn() -> Vec<JavaScriptException>>>,
}

impl ParserScriptTasks {
    pub fn new(document: Rc<RefCell<DOM>>, bindings: Rc<RefCell<DOMJavaScriptBindings>>) -> Self {
        let running = Rc::new(Cell::new(false));
        let written_markup = Rc::new(RefCell::new(String::new()));
        let write_running = running.clone();
        let write_buffer = written_markup.clone();
        bindings
            .borrow_mut()
            .SetDocumentWriteHandler(Box::new(move |markup| {
                if !write_running.get() {
                    return false;
                }
                write_buffer.borrow_mut().push_str(markup);
                true
            }));
        Self {
            document,
            bindings,
            running,
            written_markup,
            pending_host_errors: None,
        }
    }

    pub(crate) fn SetPendingHostErrors(
        &mut self,
        source: Rc<dyn Fn() -> Vec<JavaScriptException>>,
    ) {
        self.pending_host_errors = Some(source);
    }

    fn ReportHostErrors(&self, report_error: &mut dyn FnMut(&JavaScriptException)) {
        if let Some(source) = &self.pending_host_errors {
            for error in source() {
                report_error(&error);
            }
        }
    }

    // cpp: browser/browser.cc:1632-1643
    // cpp: browser/browser.cc:940-943
    // cpp: browser/browser.cc:1715-1722
    pub fn ExecuteClassic(
        &mut self,
        parser: &mut HTMLDocumentParser<'_>,
        script: ParserScript,
        loaded_source: Option<&str>,
        source_name: &str,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> JavaScriptResult {
        assert!(!self.running.get(), "parser script task cannot be nested");
        struct FinishTask {
            bindings: Rc<RefCell<DOMJavaScriptBindings>>,
            running: Rc<Cell<bool>>,
        }
        impl Drop for FinishTask {
            fn drop(&mut self) {
                self.bindings.borrow_mut().SetCurrentScript(0);
                self.running.set(false);
            }
        }
        let result = parser.WithPausedDocument(|tree| {
            let mut inline_source = String::new();
            if loaded_source.is_none() {
                let node = tree
                    .FindNodeById(script.node_id)
                    .expect("parser script node disappeared");
                AppendTextContent(tree, node, &mut inline_source);
            }
            DOM::WithDocument(&self.document, tree, || {
                self.running.set(true);
                let _finish = FinishTask {
                    bindings: self.bindings.clone(),
                    running: self.running.clone(),
                };
                self.bindings.borrow_mut().SetCurrentScript(script.node_id);
                let result =
                    runtime.Evaluate(realm, loaded_source.unwrap_or(&inline_source), source_name);
                // The source clears currentScript before its microtask checkpoint.
                self.bindings.borrow_mut().SetCurrentScript(0);
                // Source listener exceptions are reported before an exception
                // escaping the enclosing script evaluation.
                self.ReportHostErrors(report_error);
                if let Some(error) = &result.exception {
                    report_error(error);
                }
                runtime.PerformMicrotaskCheckpoint();
                self.ReportHostErrors(report_error);
                for error in runtime.TakePendingExceptions(realm) {
                    report_error(&error);
                }
                result
            })
        });
        let markup = std::mem::take(&mut *self.written_markup.borrow_mut());
        if !markup.is_empty() {
            parser.InsertFromScript(&markup);
        }
        result
    }

    // cpp: browser/browser.cc:1615-1643
    // Async/deferred scripts do not have a parser-blocking insertion point.
    pub fn ExecuteNonBlocking(
        &mut self,
        parser: Option<&mut HTMLDocumentParser<'_>>,
        script: ParserScript,
        source: &str,
        source_name: &str,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> JavaScriptResult {
        self.ExecuteNonBlockingKind(
            parser,
            script,
            source,
            source_name,
            None,
            runtime,
            realm,
            report_error,
        )
    }

    pub fn ExecuteModule(
        &mut self,
        parser: Option<&mut HTMLDocumentParser<'_>>,
        script: ParserScript,
        module: &javascript::javascript_runtime::JavaScriptValue,
        source_name: &str,
        loader: Rc<dyn JavaScriptModuleResolver>,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> JavaScriptResult {
        self.ExecuteNonBlockingKind(
            parser,
            script,
            "",
            source_name,
            Some((module, loader)),
            runtime,
            realm,
            report_error,
        )
    }

    fn ExecuteNonBlockingKind(
        &mut self,
        parser: Option<&mut HTMLDocumentParser<'_>>,
        script: ParserScript,
        source: &str,
        source_name: &str,
        module: Option<(
            &javascript::javascript_runtime::JavaScriptValue,
            Rc<dyn JavaScriptModuleResolver>,
        )>,
        runtime: &mut dyn JavaScriptRuntime,
        realm: &JavaScriptRealm,
        report_error: &mut dyn FnMut(&JavaScriptException),
    ) -> JavaScriptResult {
        assert!(!self.running.get(), "parser script task cannot be nested");
        let bindings = self.bindings.clone();
        let run = || {
            struct ClearScript(Rc<RefCell<DOMJavaScriptBindings>>);
            impl Drop for ClearScript {
                fn drop(&mut self) {
                    self.0.borrow_mut().SetCurrentScript(0);
                }
            }
            let _clear = ClearScript(bindings.clone());
            bindings.borrow_mut().SetCurrentScript(if module.is_some() {
                0
            } else {
                script.node_id
            });
            let result = if let Some((module, resolver)) = module.clone() {
                runtime.EvaluateCompiledModule(realm, module, source_name, Some(resolver))
            } else {
                runtime.Evaluate(realm, source, source_name)
            };
            bindings.borrow_mut().SetCurrentScript(0);
            self.ReportHostErrors(report_error);
            if let Some(error) = &result.exception {
                report_error(error);
            }
            runtime.PerformMicrotaskCheckpoint();
            self.ReportHostErrors(report_error);
            for error in runtime.TakePendingExceptions(realm) {
                report_error(&error);
            }
            result
        };
        if let Some(parser) = parser {
            parser.WithDocument(|tree| DOM::WithDocument(&self.document, tree, run))
        } else {
            let mut run = run;
            run()
        }
    }
}

// cpp: browser/browser.cc:179-184
fn AppendTextContent(tree: &Document, node: usize, output: &mut String) {
    if tree.Node(node).Type() == dom::persistent_document::DOMNodeType::kText {
        output.push_str(tree.Node(node).Data());
    }
    for &child in tree.Node(node).Children() {
        AppendTextContent(tree, child, output);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom_mutation::ApplyDOMTreeMutation;
    use html::html_parser::HTMLParserStatus;
    use html::{HTMLParserHost, ParserElementEvent};
    use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;

    struct ParserHost;
    impl HTMLParserHost for ParserHost {
        fn HandleParserElement(&mut self, _event: ParserElementEvent<'_>) {}
    }

    #[test]
    fn classic_task_preserves_parser_order_writes_and_recovery_after_error() {
        let document = Rc::new(RefCell::new(DOM::new()));
        let emit_document = document.clone();
        let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
            document.clone(),
            Box::new(move |mutation| {
                ApplyDOMTreeMutation(&mut emit_document.borrow_mut(), mutation);
            }),
        )));
        let mut tasks = ParserScriptTasks::new(document.clone(), bindings.clone());
        let mut runtime = QuickJsJavaScriptRuntime::new();
        let realm = runtime.CreateRealm(bindings);
        let bootstrap =
            runtime.Evaluate(&realm, webapi::DOMBootstrapSource(), "browser:dom-webidl");
        assert!(bootstrap.Succeeded(), "{:?}", bootstrap.exception);

        let mut parse_owner = DOM::new();
        document
            .borrow_mut()
            .SwapDocument(parse_owner.GetDocumentMut());
        let mut parser_host = ParserHost;
        let mut errors = Vec::new();
        let mut scripts = 0;
        {
            let mut parser =
                HTMLDocumentParser::new(parse_owner.GetDocumentMut(), &mut parser_host);
            parser.Append("<input id=kw><div id=target><span id=old>old</span></div><table id=table></table><template id=template>old</template><script>\
                var field = document.getElementById('kw');\
                if (document.getElementById('later') !== null) throw new Error('parsed too far');\
                if (!document.currentScript || document.currentScript.tagName !== 'SCRIPT') throw new Error('missing currentScript');\
                field.className = 'night';\
                var target = document.getElementById('target'), old = document.getElementById('old');\
                target.innerHTML = '<b id=replacement>new</b><script>throw new Error(\"innerHTML script ran\")<\\/script>';\
                if (target.firstChild.tagName !== 'B' || target.firstChild.textContent !== 'new' || old.parentNode !== null || document.getElementById('old') !== null) throw new Error('innerHTML replacement failed');\
                document.getElementById('table').innerHTML = '<tr><td>cell</td></tr>';\
                if (document.getElementById('table').firstChild.tagName !== 'TBODY') throw new Error('table fragment context lost');\
                document.getElementById('template').innerHTML = '<span>template replacement</span>';\
                if (document.getElementById('template').content.firstChild.textContent !== 'template replacement') throw new Error('template contents lost');\
                document.write('<span id=written></span>');\
                Promise.resolve().then(() => {\
                    if (document.currentScript !== null) throw new Error('currentScript leaked into microtask');\
                    field.setAttribute('data-job', 'done');\
                    document.write('<span id=job-written></span>');\
                });\
                </script><div id=later></div><script src=external.js>throw new Error('inline fallback executed')</script>\
                <script>\
                if (document.getElementById('kw') !== field || field.className !== 'night' ||\
                    field.getAttribute('data-job') !== 'done' || !document.getElementById('later') ||\
                    !document.getElementById('written') || !document.getElementById('job-written'))\
                    throw new Error('DOM state did not survive parser resume');\
                if (field.getAttribute('data-external') !== 'yes') throw new Error('external script did not run');\
                </script>");
            parser.FinishInput();
            while !parser.IsFinished() {
                let parsed = parser.Pump(64);
                if parsed.status != HTMLParserStatus::kWaitingForScript {
                    continue;
                }
                let external_source = (scripts == 1).then_some(
                    "field.setAttribute('data-external','yes'); throw new Error('external failure')",
                );
                let result = tasks.ExecuteClassic(
                    &mut parser,
                    parsed.script.unwrap(),
                    external_source,
                    "page.js",
                    &mut runtime,
                    &realm,
                    &mut |error| errors.push(error.clone()),
                );
                assert_eq!(result.Succeeded(), scripts != 1, "{:?}", result.exception);
                scripts += 1;
                parser.ResumeAfterScript();
            }
        }
        document
            .borrow_mut()
            .SwapDocument(parse_owner.GetDocumentMut());
        assert_eq!(scripts, 3);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("external failure"));
        let outside_write = runtime.Evaluate(&realm, "document.write('<p>late</p>')", "late.js");
        assert!(!outside_write.Succeeded());
        assert!(outside_write
            .exception
            .unwrap()
            .message
            .contains("active parser insertion point"));
    }
}
