use dom::dom_mutation::ApplyDOMMutations;
use javascript::javascript_runtime::JavaScriptRuntime;
use javascript::quickjs_javascript_runtime::QuickJsJavaScriptRuntime;
use std::cell::RefCell;
use std::rc::Rc;
use webapi::dom_bindings::DOMJavaScriptBindings;
#[test]
fn mapped_selector_api_uses_real_dom_and_errors() {
    let document=Rc::new(RefCell::new(html::html_parser::ParseHTML("<main id=root><p id=a class='one two'></p><!--gap--><p id=b class=two></p><section><span class=leaf></span></section></main>")));
    let changed = document.clone();
    let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
        document,
        Box::new(move |mutation| {
            ApplyDOMMutations(
                changed.borrow_mut().GetDocumentMut(),
                std::slice::from_ref(mutation),
            )
        }),
    )));
    let mut runtime = QuickJsJavaScriptRuntime::new();
    let realm = runtime.CreateRealm(bindings);
    let bootstrap = runtime.Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl");
    assert!(bootstrap.Succeeded(), "{:?}", bootstrap.exception);
    let result=runtime.Evaluate(&realm,r#"
        function check(v,s){if(!v)throw Error(s)}
        const root=document.querySelector('#root'),a=root.querySelector('p.one'),b=root.querySelector('#b');
        check(root===document.getElementById('root')&&root.querySelector('#root')===null,'receiver exclusion');
        check(a.matches('main > p.one.two:first-child')&&b.matches('p + p:nth-child(2)')&&b.closest('main')===root,'matcher/closest');
        check(root.matches(':has(> section .leaf)')&&!root.matches(':has(> .leaf)'),'relative has');
        const snap=root.querySelectorAll(':scope > p');check(snap.length===2&&snap[0]===a&&snap[1]===b,'ordered snapshot');
        root.removeChild(a);root.appendChild(a);a.className='changed';
        check(root.querySelector('p')===b&&!a.matches('.one')&&a.matches('.changed:last-child'),'mutation/cache');
        check(snap[0]===a&&snap[1]===b,'snapshot preserved');
        for(const operation of [()=>a.matches('p >'),()=>a.closest(''),()=>root.querySelector('['),()=>root.querySelectorAll('p,')]){
            let threw=false;try{operation()}catch(e){threw=e instanceof SyntaxError}check(threw,'syntax error');
        }
        let pending=false;try{root.querySelector(':playing')}catch(e){pending=String(e).includes('NotSupportedError')}check(pending,'explicit unsupported collaborator');
    "#,"mapped-selector-test.js");
    assert!(result.Succeeded(), "{:?}", result.exception);
}
