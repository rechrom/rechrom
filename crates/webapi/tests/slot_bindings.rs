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
        "slot-bindings.js",
    );
    assert!(result.Succeeded(), "{:?}", result.exception);
}

#[test]
fn slot_interface_uses_native_prototypes_and_receiver_brands() {
    run(r#"
      const slot=document.createElement('slot');
      check(slot instanceof HTMLSlotElement&&slot instanceof HTMLElement,'native slot prototype');
      for(const member of ['name','assignedNodes','assignedElements','assign'])check(Object.getOwnPropertyDescriptor(HTMLSlotElement.prototype,member).enumerable,'IDL enumerable '+member);
      check(!(document.createElementNS('http://www.w3.org/2000/svg','slot') instanceof HTMLSlotElement),'namespace brand');
      for(const call of [()=>new HTMLSlotElement(),()=>HTMLSlotElement(),()=>new(class extends HTMLSlotElement{})()]){let ok=false;try{call()}catch(e){ok=e instanceof TypeError}check(ok,'illegal constructor');}
      const methods=['assignedNodes','assignedElements','assign'];
      for(const m of methods)for(const obj of [window,{},document,document.body,Object.create(HTMLSlotElement.prototype)]){let ok=false;try{HTMLSlotElement.prototype[m].call(obj)}catch(e){ok=e instanceof TypeError}check(ok,'native receiver '+m);}
      Object.setPrototypeOf(slot,null);check(HTMLSlotElement.prototype.assignedNodes.call(slot).length===0,'prototype is not brand');
    "#);
}
#[test]
fn slot_name_reflects_through_native_brand_and_mutation() {
    run(r#"
      const s=document.createElement('slot'),d=Object.getOwnPropertyDescriptor(HTMLSlotElement.prototype,'name');
      check(s.name==='','default name');s.name=42;check(s.getAttribute('name')==='42','reflected write');s.setAttribute('name','next');check(s.name==='next','reflected read');
      s.name=null;check(s.name==='null','DOMString');
      for(const f of [()=>d.get.call(document.body),()=>d.set.call(document.body,'bad'),()=>d.get.call(window),()=>{s.name=Symbol()}]){let ok=false;try{f()}catch(e){ok=e instanceof TypeError}check(ok,'name validation');}
      Object.setPrototypeOf(s,null);check(d.get.call(s)==='null','native brand');
    "#);
}
#[test]
fn named_slot_snapshots_filter_comments_and_follow_first_slot() {
    run(r#"
      const host=document.getElementById('host'),root=host.attachShadow({mode:'open'});
      for(const name of ['', 'a', 'a']){const s=document.createElement('slot');s.setAttribute('name',name);root.appendChild(s);}
      const [def,a,duplicate]=root.children, text=document.createTextNode('text'),one=document.createElement('b'),two=document.createElement('i');
      one.setAttribute('slot','a');two.setAttribute('slot','missing');host.appendChild(text);host.appendChild(document.createComment('skip'));host.appendChild(one);host.appendChild(two);
      check(def.assignedNodes().length===1&&def.assignedNodes()[0]===text,'only slotables');
      const snapshot=a.assignedNodes();check(Array.isArray(snapshot)&&snapshot[0]===one&&a.assignedElements()[0]===one,'fresh sequence');check(snapshot!==a.assignedNodes(),'new array');
      check(duplicate.assignedNodes().length===0,'first named slot');one.setAttribute('slot','');check(a.assignedNodes().length===0&&snapshot[0]===one&&def.assignedElements()[0]===one,'live reassign static result');
      a.name='missing';check(a.assignedNodes()[0]===two,'slot name mutation');root.removeChild(a);check(a.assignedNodes().length===0,'detached slot');
    "#);
}
#[test]
fn flattened_slots_recurse_fallback_and_preserve_light_dom_slot_nodes() {
    run(r#"
      const host=document.getElementById('host'),root=host.attachShadow({mode:'open'});
      const o=document.createElement('slot'),n=document.createElement('slot');o.setAttribute('name','outer');n.setAttribute('name','inner');o.appendChild(document.createTextNode('before'));o.appendChild(document.createComment('skip'));n.appendChild(document.createElement('b'));o.appendChild(n);o.appendChild(document.createElement('i'));root.appendChild(o);
      const outer=root.firstChild,inner=outer.querySelector('slot');
      check(outer.assignedNodes().length===0,'unflattened omits fallback');
      let flat=outer.assignedNodes({flatten:true});check(flat.length===3&&flat[0].nodeType===3&&flat[1].tagName==='B'&&flat[2].tagName==='I','nested fallback');
      const assigned=document.createElement('em');assigned.setAttribute('slot','inner');host.appendChild(assigned);check(outer.assignedElements({flatten:true})[0]===assigned,'nested assignment');
      const lightSlot=document.createElement('slot');lightSlot.setAttribute('slot','outer');lightSlot.appendChild(document.createElement('b'));host.appendChild(lightSlot);
      check(outer.assignedNodes({flatten:true})[0]===lightSlot,'only assignment-supporting slots flatten');
      check(lightSlot.assignedNodes({flatten:true}).length===0,'light slot does not support assignment');
    "#);
}
#[test]
fn assigned_options_dictionary_and_native_brand_order() {
    run(r#"
      const s=document.createElement('slot');
      for(const x of [undefined,null,{},()=>{}])check(s.assignedNodes(x).length===0,'empty dictionary');
      for(const x of [1,true,'x',Symbol()]){let ok=false;try{s.assignedNodes(x)}catch(e){ok=e instanceof TypeError}check(ok,'dictionary conversion');}
      let reads=0;s.assignedElements({get flatten(){reads++;return 1}});check(reads===1,'one getter');
      let ok=false;try{HTMLSlotElement.prototype.assignedNodes.call(document.body,{get flatten(){throw Error('wrong order')}})}catch(e){ok=e instanceof TypeError}check(ok,'receiver before options');
    "#);
}
#[test]
fn manual_assignment_is_retained_atomic_ordered_and_deduplicated() {
    run(r#"
      const host=document.getElementById('host'),root=host.attachShadow({mode:'closed',slotAssignment:'manual'}),s=document.createElement('slot'),other=document.createElement('slot');
      const a=document.createElement('b'),b=document.createTextNode('b'),outside=document.createElement('i');
      s.assign(b,a,b,outside);check(s.assignedNodes().length===0,'retain while detached');root.appendChild(s);root.appendChild(other);host.appendChild(a);host.appendChild(b);
      check(s.assignedNodes().length===2&&s.assignedNodes()[0]===b&&s.assignedNodes()[1]===a,'order and duplicate');
      check(s.assignedElements().length===1&&s.assignedElements()[0]===a,'element filter');
      for(const invalid of [document,root,document.createComment('bad'),{},null]){let ok=false;try{s.assign(a,invalid)}catch(e){ok=e instanceof TypeError}check(ok,'union conversion');check(s.assignedNodes()[0]===b,'atomic failure');}
      other.assign(a);check(s.assignedNodes().length===1&&other.assignedNodes()[0]===a,'move previous owner');
      host.removeChild(b);check(s.assignedNodes().length===0,'removed child inactive');host.appendChild(b);check(s.assignedNodes()[0]===b,'retained relation');
      s.assign();check(s.assignedNodes().length===0,'clear');
    "#);
}

#[test]
fn slot_methods_ignore_script_projections_and_reject_autonomous_slot_constructor() {
    run(r#"
      const host=document.getElementById('host'),root=host.attachShadow({mode:'open'}),slot=document.createElement('slot'),a=document.createElement('b');
      root.appendChild(slot);host.appendChild(a);
      Object.defineProperty(slot,'childNodes',{value:[]});Object.defineProperty(host,'childNodes',{value:[]});
      const getAttribute=slot.getAttribute;slot.getAttribute=()=> 'fake';slot.setAttribute=()=>{};slot.name='real';
      check(slot.name==='real'&&getAttribute.call(slot,'name')==='real','native name');
      slot.name='';check(slot.assignedNodes()[0]===a,'native assignment');
      slot.assign();check(slot.assignedNodes()[0]===a,'manual assignment does not override named mode');
      const get=HTMLSlotElement.prototype.assignedNodes;Object.setPrototypeOf(slot,null);check(get.call(slot)[0]===a,'native slot identity remains');
      let converted=false;try{get.call(slot,{get flatten(){throw Error('options getter')}})}catch(e){converted=e.message==='options getter'}check(converted,'getter exception propagated');
      const nativeSlot=document.createElement('slot');
      __domUpgradeCustomElement(nativeSlot,class extends HTMLSlotElement {constructor(){super();let threw=false;try{new HTMLSlotElement()}catch(e){threw=e instanceof TypeError}check(threw,'direct constructor during upgrade');}});
      class Wrong extends HTMLSlotElement{};const element=document.createElement('x-wrong');let rejected=false;
      try{__domUpgradeCustomElement(element,Wrong)}catch(e){rejected=e instanceof TypeError}check(rejected,'autonomous element cannot construct built-in slot');
    "#);
}
