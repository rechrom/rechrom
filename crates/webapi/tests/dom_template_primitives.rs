use dom::dom_mutation::ApplyDOMMutations;
use javascript::{
    javascript_runtime::JavaScriptRuntime, quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
};
use std::{cell::RefCell, rc::Rc};
use webapi::{dom_bindings::DOMJavaScriptBindings, window_bindings::WindowJavaScriptBindings};

struct NoNetwork;
impl xhr_transport::XMLHttpRequestTransport for NoNetwork {
    fn Start(
        &mut self,
        _: &xhr_transport::XMLHttpRequestData,
    ) -> std::io::Result<Box<dyn xhr_transport::XMLHttpRequestOperation>> {
        Err(std::io::Error::other("fixture has no network"))
    }
}
fn run(script: &str) {
    let document = Rc::new(RefCell::new(html::html_parser::ParseHTML(
        "<main id=root><p id=source data-first=one title=two>text<!--marker--><b>bold</b></p><template id=template><section data-part=x><span>template text</span></section><template><i>nested</i></template></template><svg id=svg viewBox='0 0 10 10' xlink:href='#shape'></svg><div id=host></div></main>",
    )));
    let changed = document.clone();
    let dom = Rc::new(RefCell::new(DOMJavaScriptBindings::new(
        document,
        Box::new(move |mutation| {
            ApplyDOMMutations(
                changed.borrow_mut().GetDocumentMut(),
                std::slice::from_ref(mutation),
            );
        }),
    )));
    let window = Rc::new(RefCell::new(WindowJavaScriptBindings::new(
        dom,
        Box::new(NoNetwork),
        "fixture".into(),
        None,
    )));
    let mut runtime = QuickJsJavaScriptRuntime::new();
    let realm = runtime.CreateRealm(window);
    for (source, name) in [
        (webapi::DOMBootstrapSource().to_owned(), "dom-webidl"),
        (webapi::WindowBootstrapSource(), "window-webidl"),
    ] {
        let result = runtime.Evaluate(&realm, &source, name);
        assert!(result.Succeeded(), "{name}: {:?}", result.exception);
    }
    let result = runtime.Evaluate(
        &realm,
        &format!("function check(v,m){{if(!v)throw Error(m)}};{script}"),
        "dom-template-primitives.js",
    );
    assert!(result.Succeeded(), "{:?}", result.exception);
}
#[test]
fn element_has_attributes_uses_live_native_list() {
    run(r#"
        const e=document.createElement('div');
        check(!e.hasAttributes(),'empty');e.setAttribute('id','x');check(e.hasAttributes(),'added');
        e.removeAttribute('id');check(!e.hasAttributes(),'removed');
        let threw=false;try{Element.prototype.hasAttributes.call(document)}catch(e){threw=e instanceof TypeError}check(threw,'brand');
    "#);
}
#[test]
fn element_attribute_names_are_qualified_ordered_snapshots() {
    run(r#"
        const e=document.getElementById('source'), names=e.getAttributeNames();
        check(Array.isArray(names)&&names.join(',')==='id,data-first,title','ordered array');
        e.setAttribute('data-first','changed');e.setAttribute('length','value');e.removeAttribute('title');
        check(e.getAttributeNames().join(',')==='id,data-first,length','stable update ordering');
        check(names.join(',')==='id,data-first,title'&&names!==e.getAttributeNames(),'snapshot');
        check(document.getElementById('svg').getAttributeNames().join(',')==='id,viewBox,xlink:href','qualified SVG names');
        let threw=false;try{Element.prototype.getAttributeNames.call(document)}catch(e){threw=e instanceof TypeError}check(threw,'brand');
    "#);
}
#[test]
fn document_import_node_clones_into_destination_owner() {
    run(r#"
        const source=document.getElementById('source'), dest=document.implementation.createHTMLDocument('target');
        const shallow=dest.importNode(source), deep=dest.importNode(source,1);
        check(shallow!==source&&shallow.ownerDocument===dest&&shallow.parentNode===null&&!shallow.hasChildNodes(),'shallow default');
        check(deep.ownerDocument===dest&&deep.textContent===source.textContent&&deep.children[0]!==source.children[0],'deep clone');
        for(const n of deep.childNodes)check(n.ownerDocument===dest,'descendant owner');
        check(source.ownerDocument===document&&source.parentNode.id==='root','source unchanged');
        deep.id='copy';check(source.id==='source','independent attrs');
        dest.body.appendChild(deep);check(deep.parentNode===dest.body,'insert imported');
        const clone=deep.cloneNode(true);check(clone.ownerDocument===dest&&clone.firstChild.ownerDocument===dest,'clone retains owner');
    "#);
}
#[test]
fn document_import_node_preserves_nested_template_contents_and_fragments() {
    run(r#"
        const source=document.getElementById('template'), dest=document.implementation.createHTMLDocument();
        const shallow=dest.importNode(source), deep=dest.importNode(source,true), fragment=dest.importNode(source.content,true);
        check(!shallow.content.hasChildNodes()&&shallow.content!==source.content,'shallow template');
        check(deep.content.firstChild!==source.content.firstChild&&deep.content.firstChild.firstChild.textContent==='template text','deep template');
        check(deep.content.lastChild.content.firstChild.textContent==='nested','nested template');
        check(fragment.nodeType===11&&fragment.ownerDocument===dest&&fragment.firstChild.ownerDocument===dest,'fragment ownership');
        check(source.content.childNodes.length===2&&fragment.childNodes.length===2,'source fragment unchanged');
        dest.body.appendChild(fragment);check(fragment.childNodes.length===0&&dest.body.firstChild.tagName==='SECTION','fragment insertion');
    "#);
}
#[test]
fn document_import_node_rejects_documents_shadow_roots_and_invalid_receivers() {
    run(r#"
        const root=document.getElementById('host').attachShadow({mode:'open'});
        for(const n of [document,root]){let threw=false;try{document.importNode(n,true)}catch(e){threw=e instanceof DOMException&&e.name==='NotSupportedError'}check(threw,'not supported');}
        for(const call of [()=>document.importNode(),()=>document.importNode({}),()=>Document.prototype.importNode.call(document.body,document.body)]){
          let threw=false;try{call()}catch(e){threw=e instanceof TypeError}check(threw,'webidl type check');
        }
    "#);
}
#[test]
fn import_options_and_attribute_overrides_keep_native_semantics() {
    run(r#"
        const e=document.getElementById('source');
        Object.defineProperty(e,'attributes',{value:[],configurable:true});
        check(e.hasAttributes()&&e.getAttributeNames().join(',')==='id,data-first,title','native list ignores wrapper override');
        check(document.importNode(e,{selfOnly:true}).childNodes.length===0,'selfOnly');
        check(document.importNode(e,{}).childNodes.length===e.childNodes.length,'dictionary deep default');
        check(document.importNode(e,null).childNodes.length===e.childNodes.length,'null dictionary');
        e.expando=123;const clone=document.importNode(e,true);check(clone.expando===undefined,'new wrapper does not copy expando');
        let unsupported=false;try{document.importNode(e,{customElementRegistry:customElements})}catch(e){unsupported=e.name==='NotSupportedError'}check(unsupported,'explicit registry gap fails clearly');
    "#);
}

#[test]
fn import_node_brand_checks_use_native_identity_after_prototype_changes() {
    run(r#"
        for(const call of [()=>Element.prototype.hasAttributes.call(window),()=>Element.prototype.getAttributeNames.call(window),()=>Document.prototype.importNode.call(window,document.body)]){let threw=false;try{call()}catch(e){threw=e instanceof TypeError}check(threw,'global object incompatible receiver');}
        const e=document.getElementById('source');Object.setPrototypeOf(e,null);
        const copy=document.importNode(e,true);check(copy.localName==='p'&&copy.textContent==='textbold','native element remains valid');
        const root=document.getElementById('host').attachShadow({mode:'open'});Object.setPrototypeOf(root,DocumentFragment.prototype);
        let unsupported=false;try{document.importNode(root)}catch(e){unsupported=e instanceof DOMException&&e.name==='NotSupportedError'}check(unsupported,'native shadow-root identity');
        const dest=document.implementation.createHTMLDocument();Object.setPrototypeOf(dest,null);
        check(Document.prototype.importNode.call(dest,copy).ownerDocument===dest,'native document receiver');
    "#);
}
