//! Native event paths through the production Page dispatch and host bindings.
#[path = "../src/native_test_thread.rs"]
mod native_test_thread;

use browser::page::{Page, ScriptEnvironment};
use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
use std::{cell::RefCell, io, rc::Rc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}
struct Loader;
impl URLLoader for Loader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: "text/html".into(),
            body: b"<!doctype html><html><head><style>body{margin:0;font:16px/20px sans-serif}</style></head><body></body></html>".to_vec(),
            ..Default::default()
        }))))
    }
}
struct XHR;
impl xhr_transport::XMLHttpRequestTransport for XHR {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(io::Error::other("unexpected shadow fixture XHR"))
    }
}

fn evaluate(page: &mut Page, script: &str) {
    let result = page.Evaluate(script, "test:event-path").unwrap();
    assert!(
        result.Succeeded(),
        "event path JavaScript failed: {}",
        result
            .exception
            .as_ref()
            .map(|e| e.message.as_str())
            .unwrap_or("")
    );
}

fn fixture() -> Page {
    let assembly = browser::CreateLayoutAssembly();
    let mut constraints = browser::CreateBrowserConstraints(640, 800);
    constraints.fonts = vec![layoutng_assembly::internal::layout_input::FontFace {
        family: "sans-serif".into(),
        bytes: include_bytes!("../../../assets/fonts/Roboto-Regular.ttf")
            .as_slice()
            .into(),
        ..Default::default()
    }];
    let mut page = Page::Create(
        Rc::new(RefCell::new(Loader)),
        Rc::new(RefCell::new(
            image_decoder::skia_image_decoder::SkiaImageDecoder,
        )),
        Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
        constraints,
        Some(ScriptEnvironment {
            runtime: Box::new(QuickJsJavaScriptRuntime::with_native_stack_budget(
                4 * 1024 * 1024,
            )),
            xhr: Box::new(XHR),
            user_agent: "shadow-publication-fixture".into(),
        }),
        None,
    );
    page.Open("https://shadow-publication.test/", 16384, 4096)
        .unwrap();
    for _ in 0..100 {
        page.RunTask().unwrap();
        if !page.IsLoading() && page.CurrentFrame().is_some() {
            break;
        }
    }
    assert!(!page.IsLoading());
    assert!(page.CurrentFrame().is_some());
    evaluate(
        &mut page,
        r#"
        function check(value, message) { if (!value) throw new Error(message); }
        function near(actual, expected, message) {
            check(Math.abs(actual - expected) < 0.05,
                  message + ': expected ' + expected + ', got ' + actual);
        }
        function element(tag, id, css, parent, text) {
            var node = document.createElement(tag);
            if (id) node.id = id;
            if (css) node.setAttribute('style', css);
            if (text !== undefined) node.appendChild(document.createTextNode(text));
            if (parent) parent.appendChild(node);
            return node;
        }
        function rect(node) {
            var r = node.getBoundingClientRect();
            return [r.x, r.y, r.width, r.height];
        }
        function sameRect(actual, expected, message) {
            for (var i = 0; i < 4; ++i) near(actual[i], expected[i], message + '[' + i + ']');
        }
    "#,
    );
    page
}

#[test]
fn generic_event_target_path_has_dispatch_lifetime() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            const target = new EventTarget(), event = new Event('probe');
            check(event.composedPath().length === 0, 'before dispatch path is empty');
            let count = 0;
            target.addEventListener('probe', e => {
                count++;
                check(e === event && e.target === target && e.currentTarget === target, 'generic identity');
                let path = e.composedPath();
                check(Array.isArray(path) && path.length === 1 && path[0] === target, 'generic singleton path');
                path.length = 0;
                check(e.composedPath().length === 1, 'fresh array');
            });
            target.dispatchEvent(event);
            check(count === 1 && event.composedPath().length === 0 && event.currentTarget === null && event.eventPhase === 0, 'after dispatch path is empty');
        "#,
        );
    });
}

#[test]
fn synthetic_open_shadow_path_retargets_and_survives_tree_mutation() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            const host = element('div', 'host', '', document.body), shadow = host.attachShadow({mode:'open'});
            const target = element('button', 'target', '', shadow), parent = host.parentNode;
            const path = [target, shadow, host, document.body, document.documentElement, document, window];
            let seen = [], event = new Event('probe', {bubbles:true,composed:true});
            for (const node of path) for (const capture of [true,false]) node.addEventListener('probe', e => {
                let got = e.composedPath();
                check(got.length === path.length && got.every((n,i) => n === path[i]), 'open snapshot path');
                check(e.target === (node === target || node === shadow ? target : host), 'retargeted target');
                check(e.currentTarget === node, 'currentTarget');
                check(e.eventPhase === (node === target || node === host ? 2 : capture ? 1 : 3), 'host at-target phase');
                seen.push(path.indexOf(node)+':'+capture);
                if (node === target && capture) parent.removeChild(host);
            }, capture);
            target.dispatchEvent(event);
            check(seen.join(',') === '6:true,5:true,4:true,3:true,2:true,1:true,0:true,0:false,1:false,2:false,3:false,4:false,5:false,6:false', 'dispatch order '+seen);
            check(event.target === host && event.currentTarget === null && event.eventPhase === 0 && event.composedPath().length === 0, 'postdispatch target and path');
        "#,
        );
    });
}

#[test]
fn closed_shadow_slots_filter_paths_without_retargeting_light_dom() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            const host = element('div', 'host', '', document.body), shadow = host.attachShadow({mode:'closed'});
            const slot = element('slot', 'slot', '', shadow), target = element('button', 'target', '', host);
            const inside = [target,slot,shadow,host,document.body,document.documentElement,document,window];
            const outside = [target,host,document.body,document.documentElement,document,window];
            let called=0;
            for (const node of inside) node.addEventListener('probe', e => {
                called++;
                const expected = node === slot || node === shadow ? inside : outside;
                const path=e.composedPath();
                check(path.length===expected.length && path.every((n,i)=>n===expected[i]),'closed slot observer path');
                check(e.target===target,'slotted target remains light DOM');
            });
            target.dispatchEvent(new Event('probe',{bubbles:true,composed:false}));
            check(called===8,'noncomposed light target crosses its slot root');
        "#,
        );
    });
}

#[test]
fn noncomposed_internal_event_stays_inside_and_clears_target() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            const host=element('div','host','',document.body), shadow=host.attachShadow({mode:'open'}), target=element('b','target','',shadow);
            let log=[],event=new Event('probe',{bubbles:true});
            host.addEventListener('probe',()=>log.push('outside'));
            for (const node of [target,shadow]) node.addEventListener('probe',e=>{
                log.push(node===target?'target':'root');
                const p=e.composedPath(); check(p.length===2&&p[0]===target&&p[1]===shadow,'noncomposed root boundary');
                check(e.target===target,'internal target');
            });
            target.dispatchEvent(event);
            check(log.join(',')==='target,root','only internal listeners');
            check(event.target===null&&event.currentTarget===null&&event.composedPath().length===0,'clear shadow target after dispatch');
            let detached=document.createElement('div'), plain=new Event('plain');
            detached.dispatchEvent(plain);
            check(plain.target===detached&&plain.composedPath().length===0,'detached plain target retained');
        "#,
        );
    });
}

#[test]
fn composed_nonbubbling_event_invokes_shadow_host_at_target() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            const host=element('div','host','',document.body), shadow=host.attachShadow({mode:'closed'}), target=element('b','target','',shadow);
            let log=[],event=new Event('probe',{composed:true});
            for (const [node,name] of [[window,'window'],[document,'document'],[host,'host'],[shadow,'root'],[target,'target']]) for (const capture of [true,false]) node.addEventListener('probe',e=>{
                log.push(name+':'+capture+':'+e.eventPhase);
                if(node===host||node===document||node===window) check(!e.composedPath().includes(target)&&!e.composedPath().includes(shadow),'closed root hidden');
            },capture);
            target.dispatchEvent(event);
            check(log.join(',')==='window:true:1,document:true:1,host:true:2,root:true:1,target:true:2,target:false:2,host:false:2','composed nonbubble order '+log);
            check(event.target===host,'outer target after nonbubble dispatch');
        "#,
        );
    });
}

#[test]
fn nested_dispatch_restores_outer_path_and_listener_removal() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            const host=element('div','host','',document.body), shadow=host.attachShadow({mode:'open'}), target=element('b','target','',shadow), other=element('i','other','',document.body);
            let outer=new Event('outer',{bubbles:true,composed:true}),inner=new Event('inner',{bubbles:true}),log=[];
            function removed(){log.push('removed')}
            target.addEventListener('outer',e=>{
                const before=e.composedPath();
                other.dispatchEvent(inner);
                const after=e.composedPath();
                check(before.length===after.length&&before.every((n,i)=>n===after[i]),'nested path restored');
                check(e.currentTarget===target&&e.target===target,'outer target restored');
                target.removeEventListener('outer',removed);
                log.push('outer');
            });
            target.addEventListener('outer',removed);
            other.addEventListener('inner',e=>{check(e.composedPath()[0]===other,'inner path');log.push('inner')});
            target.dispatchEvent(outer);
            check(log.join(',')==='inner,outer','nested and removed listeners');
            check(inner.composedPath().length===0&&outer.composedPath().length===0,'both paths cleared');
        "#,
        );
    });
}

#[test]
fn document_window_and_load_paths_match_dispatch_endpoints() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            let log=[];
            for (const node of [document,window]) for (const capture of [true,false]) node.addEventListener('probe',e=>{
                const p=e.composedPath();
                check(p.length===2&&p[0]===document&&p[1]===window,'document path includes window');
                log.push((node===window?'w':'d')+':'+capture+':'+e.eventPhase);
            },capture);
            document.dispatchEvent(new Event('probe',{bubbles:true}));
            check(log.join(',')==='w:true:1,d:true:2,d:false:2,w:false:3','window phases '+log);
            let wc=0,dc=0,load=new Event('load',{bubbles:true});
            window.addEventListener('load',()=>wc++,true);
            document.addEventListener('load',e=>{dc++;check(e.composedPath().length===1&&e.composedPath()[0]===document,'node load window exclusion')});
            document.dispatchEvent(load);
            check(wc===0&&dc===1&&load.target===null,'load postdispatch target');
            let direct=new Event('direct');
            window.addEventListener('direct',e=>{check(e.composedPath().length===1&&e.composedPath()[0]===window,'direct window singleton')});
            window.dispatchEvent(direct);
            check(direct.target===window&&direct.currentTarget===null&&direct.composedPath().length===0,'direct window lifetime');
        "#,
        );
    });
}

#[test]
fn real_mouse_click_traverses_slot_and_keeps_native_event_identity() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            var host=element('div','host','display:block;width:120px;height:60px',document.body);
            var shadow=host.attachShadow({mode:'open'}), slot=element('slot','slot','display:contents',shadow);
            var button=element('div','button','display:block;width:80px;height:40px',host);
            var panel=element('div','panel','',shadow);panel.hidden=true;
            var clicks=0, saved=null, nativeLog=[];
            for (const node of [window,document,host,shadow,slot,button]) node.addEventListener('click',e=>{
                if (e.target!==button) return;
                nativeLog.push(node===window?'window':node===document?'document':node===shadow?'shadow':node.id);
                const p=e.composedPath();
                check(p[0]===button&&p[1]===slot&&p[2]===shadow&&p[3]===host&&p[p.length-1]===window,'native mouse slot path');
                check(e.target===button&&e.currentTarget===node&&e.isTrusted,'native mouse identity and trust');
                for(const name of ['target','currentTarget','eventPhase'])check(Object.getOwnPropertyDescriptor(Event.prototype,name).get.call(e)===e[name],'native accessor brand '+name);
                if(saved)check(saved===e,'same native wrapper across path'); else saved=e;
                p.length=0;check(e.composedPath().length===8,'native fresh snapshot');
                if(node===slot){clicks++;panel.hidden=!panel.hidden;}
            });
        "#,
        );
        for click in 1..=2 {
            for kind in [interaction::input_event::MouseEventType::kClick] {
                page.Dispatch(&interaction::input_event::InputEvent::Mouse(
                    interaction::input_event::MouseEvent {
                        r#type: kind,
                        position: layoutng_assembly::internal::layout_input::Offset {
                            x: 40.0,
                            y: 20.0,
                        },
                        button: interaction::input_event::MouseButton::kPrimary,
                        ..Default::default()
                    },
                ))
                .unwrap();
            }
            evaluate(
                &mut page,
                &format!(
                    r#"
                check(clicks==={click},'trusted slot listener toggled panel');
                check(panel.hidden==={},'mouse toggled hidden state');
                check(nativeLog.join(',')==='button,slot,shadow,host,document,window','native bubble route '+nativeLog);
                check(saved.currentTarget===null&&saved.eventPhase===0&&saved.composedPath().length===0&&saved.target===button,'retained native event cleanup');
                if (clicks===1) {{
                    var retained=saved, alternate=element('div','alternate','',document.body), reentered=false;
                    alternate.addEventListener('click',e=>{{
                        reentered=true;check(e===retained&&!e.isTrusted&&e.target===alternate&&e.currentTarget===alternate,'native redispatch identity');
                        check(e.composedPath()[0]===alternate,'native redispatch path');
                        e.preventDefault();
                    }},{{once:true}});
                    check(alternate.dispatchEvent(retained)===false&&reentered,'native redispatch cancellation');
                    check(retained.target===alternate&&retained.composedPath().length===0,'native redispatch cleanup');
                }}
                nativeLog=[];saved=null;
            "#,
                    if click == 1 { "false" } else { "true" }
                ),
            );
        }
    });
}

#[test]
fn path_lifetime_is_private_and_nested_actions_target_their_event() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            for(const name of ['target','currentTarget','eventPhase','composedPath']) {
                const d=Object.getOwnPropertyDescriptor(Event.prototype,name), method=d.get||d.value;
                for(const fake of [{},window,document,Object.create(Event.prototype)]){let rejected=false;try{method.call(fake)}catch(_){rejected=true}check(rejected,'event receiver brand '+name)}
            }
            const target=new EventTarget(),event=new Event('generic');
            Object.defineProperty(event,'currentTarget',{value:document,writable:true});
            check(event.composedPath().length===0,'public override cannot fabricate a path');
            target.addEventListener('generic',e=>{
                e.currentTarget=null;
                check(e.composedPath().length===1&&e.composedPath()[0]===target,'private generic dispatch path');
                let denied=false;try{target.dispatchEvent(e)}catch(_){denied=true}check(denied,'same event reentrancy rejected');
            });
            target.dispatchEvent(event);
            event.currentTarget=document;check(event.composedPath().length===0,'postdispatch override ignored');
            const one=element('div','one','',document.body),two=element('div','two','',document.body);
            const outer=new Event('outer',{bubbles:true,cancelable:true}),inner=new Event('inner',{cancelable:true});
            let outside=0;
            document.addEventListener('outer',()=>outside++);
            two.addEventListener('inner',()=>{outer.preventDefault();outer.stopPropagation()});
            one.addEventListener('outer',()=>{check(two.dispatchEvent(inner),'outer cancellation must not cancel inner')});
            check(!one.dispatchEvent(outer)&&outer.defaultPrevented&&!inner.defaultPrevented&&outside===0,'nested actions target outer event');
        "#,
        );
    });
}

#[test]
fn synthetic_path_is_cleared_before_nested_activation_events() {
    native_test_thread::run(|| {
        let mut page = fixture();
        evaluate(
            &mut page,
            r#"
            const input=element('input','check','',document.body);input.setAttribute('type','checkbox');
            const event=new Event('click',{bubbles:true,cancelable:true,composed:true});
            let called=0;
            input.addEventListener('click',e=>{check(e.composedPath()[0]===input,'click active path')});
            input.addEventListener('input',()=>{called++;check(event.currentTarget===null&&event.eventPhase===0&&event.composedPath().length===0,'click lifetime ended before default activation')});
            input.dispatchEvent(event);
            check(called===1,'checkbox activation input');
        "#,
        );
    });
}
