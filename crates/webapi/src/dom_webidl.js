(function() {
'use strict';
const invoke = __domInvoke, install = __domInstall, wrap = __domWrap;
const callbacks = new WeakMap(), eventTargets = new WeakMap();
function callbackFunction(callback) {
  if (callback == null) return null;
  if (typeof callback === 'function') return callback;
  if (typeof callback !== 'object') throw new TypeError('Listener must be an object');
  let wrapped = callbacks.get(callback);
  if (!wrapped) { wrapped = function(event) { return callback.handleEvent(event); }; callbacks.set(callback, wrapped); }
  return wrapped;
}
class EventTarget {
  constructor() { eventTargets.set(this, []); }
  addEventListener(type, callback, options) {
    const object = options != null && (typeof options === 'object' || typeof options === 'function');
    const capture = object ? !!options.capture : !!options;
    const once = object ? !!options.once : false;
    const passive = object ? !!options.passive : false;
    const signal = object ? options.signal : undefined;
    const fn = callbackFunction(callback);
    if (!fn || signal?.aborted) return;
    if (eventTargets.has(this)) {
      const list = eventTargets.get(this); type = String(type);
      if (!list.some(l => l.type === type && l.fn === fn && l.capture === capture)) list.push({type, fn, capture, once, passive});
    } else invoke(this, 'addEventListener', String(type), fn, capture, once, passive);
    if (signal) signal.addEventListener('abort', () => this.removeEventListener(type, fn, capture), {once:true});
  }
  removeEventListener(type, callback, options) {
    const capture = options != null && typeof options === 'object' ? !!options.capture : !!options;
    const fn = callbackFunction(callback);
    if (!fn) return;
    if (eventTargets.has(this)) {
      const list = eventTargets.get(this), index = list.findIndex(l => l.type === String(type) && l.fn === fn && l.capture === capture);
      if (index >= 0) list.splice(index,1);
    } else invoke(this, 'removeEventListener', String(type), fn, capture);
  }
  dispatchEvent(event) {
    if (!event || !event.type || event.currentTarget !== null) throw new TypeError('Invalid event dispatch');
    if (!eventTargets.has(this)) return invoke(this, 'dispatchEvent', String(event.type), !!event.bubbles, !!event.cancelable, !!event.composed, event);
    const list = eventTargets.get(this);
    event.target = event.currentTarget = this; event.eventPhase = 2;
    for (const listener of [...list]) {
      if (listener.type !== event.type || !list.includes(listener)) continue;
      if (listener.once) list.splice(list.indexOf(listener),1);
      event._passive = listener.passive;
      listener.fn.call(this,event);
      if (event._immediate) break;
    }
    if (!event._immediate && typeof this['on'+event.type] === 'function') this['on'+event.type](event);
    event.currentTarget = null; event.eventPhase = 0; event._passive = false;
    return !event.defaultPrevented;
  }
}
wrap('__callEventHandler',function(callback,event){if(callback.call(this,event)===false)event.preventDefault();});
const customElementConstructionStack=[];
class Node extends EventTarget {
  constructor() {
    super();
    if(customElementConstructionStack.length)return customElementConstructionStack.at(-1);
    throw new TypeError('Illegal constructor');
  }
}
class Element extends Node {
  get parentElement() { const p = this.parentNode; return p?.nodeType === 1 ? p : null; }
}
class HTMLElement extends Element {}
class SVGElement extends Element {}
class SVGGeometryElement extends SVGElement {}
class SVGPathElement extends SVGGeometryElement {}
class MathMLElement extends Element {}
class HTMLInputElement extends HTMLElement {}
class HTMLTextAreaElement extends HTMLElement {}
class HTMLButtonElement extends HTMLElement {}
class HTMLFormElement extends HTMLElement {}
class HTMLSelectElement extends HTMLElement {}
class HTMLOptionElement extends HTMLElement {}
class HTMLLabelElement extends HTMLElement {}
class HTMLDetailsElement extends HTMLElement {}
class HTMLDialogElement extends HTMLElement {}
class HTMLScriptElement extends HTMLElement {}
class HTMLStyleElement extends HTMLElement {}
class HTMLLinkElement extends HTMLElement {}
class HTMLMetaElement extends HTMLElement {}
class HTMLIFrameElement extends HTMLElement {}
class HTMLImageElement extends HTMLElement {}
class HTMLCanvasElement extends HTMLElement {
  // A conforming canvas returns null for a context type that the user agent
  // does not support. Keep feature detection deterministic until a rendering
  // backend is installed; callers must not observe a missing method.
  getContext(contextId) { String(contextId); return null; }
}
class HTMLTemplateElement extends HTMLElement {}
class HTMLHeadingElement extends HTMLElement {}
class HTMLAnchorElement extends HTMLElement {
  get href() { const value=this.getAttribute('href'); if(value===null)return ''; try{return new URL(value,document.URL).href;}catch{return value;} }
  set href(value) { this.setAttribute('href',String(value)); }
  get protocol() { try{return new URL(this.href).protocol;}catch{return ':';} }
  get host() { try{return new URL(this.href).host;}catch{return '';} }
  get hostname() { try{return new URL(this.href).hostname;}catch{return '';} }
  get port() { try{return new URL(this.href).port;}catch{return '';} }
  get pathname() { try{return new URL(this.href).pathname;}catch{return '';} }
  get search() { try{return new URL(this.href).search;}catch{return '';} }
  get hash() { try{return new URL(this.href).hash;}catch{return '';} }
  get origin() { try{return new URL(this.href).origin;}catch{return '';} }
}
class HTMLMediaElement extends HTMLElement {
  // No media decoder is supplied by this host yet. Unsupported formats must
  // report the empty string so the site's own fallback can run.
  canPlayType(type) { String(type); return ''; }
  load() {}
  pause() { this._paused=true; }
  play() { this._paused=false; return Promise.resolve(); }
  get paused() { return this._paused!==false; }
  get ended() { return false; }
  get readyState() { return 0; }
  get networkState() { return 0; }
  get duration() { return NaN; }
}
class HTMLVideoElement extends HTMLMediaElement {}
class HTMLAudioElement extends HTMLMediaElement {}
class Document extends Node {}
const fontFaceSetToken = {};
class FontFaceSet extends EventTarget {
  constructor(token) {
    if (token !== fontFaceSetToken) throw new TypeError('Illegal constructor');
    super();
    this.status = 'loaded';
    this.ready = Promise.resolve(this);
  }
  check(font, text = ' ') { String(font); String(text); return true; }
  load(font, text = ' ') { String(font); String(text); return Promise.resolve([]); }
}
Object.defineProperty(FontFaceSet.prototype, Symbol.toStringTag,
  {value:'FontFaceSet', configurable:true});
Object.defineProperty(globalThis, 'FontFaceSet',
  {value:FontFaceSet, writable:true, configurable:true});
const documentFontSets = new WeakMap();
Object.defineProperty(Document.prototype, 'fonts', {configurable:true, enumerable:true, get() {
  let fonts = documentFontSets.get(this);
  if (!fonts) { fonts = new FontFaceSet(fontFaceSetToken); documentFontSets.set(this, fonts); }
  return fonts;
}});
class DocumentFragment extends Node {}
class ShadowRoot extends DocumentFragment {}
class ElementInternals {
  constructor() { throw new TypeError('Illegal constructor'); }
  setFormValue() {}
  setValidity() {}
  checkValidity() { return true; }
  reportValidity() { return true; }
}
class Text extends Node {}
class Comment extends Node {}
class NodeList { constructor() { throw new TypeError('Illegal constructor'); } }
class HTMLCollection { constructor() { throw new TypeError('Illegal constructor'); } }
for (const [name, value] of Object.entries({ELEMENT_NODE:1, ATTRIBUTE_NODE:2, TEXT_NODE:3,
  CDATA_SECTION_NODE:4, PROCESSING_INSTRUCTION_NODE:7, COMMENT_NODE:8, DOCUMENT_NODE:9,
  DOCUMENT_TYPE_NODE:10, DOCUMENT_FRAGMENT_NODE:11, DOCUMENT_POSITION_DISCONNECTED:1,
  DOCUMENT_POSITION_PRECEDING:2, DOCUMENT_POSITION_FOLLOWING:4,
  DOCUMENT_POSITION_CONTAINS:8, DOCUMENT_POSITION_CONTAINED_BY:16,
  DOCUMENT_POSITION_IMPLEMENTATION_SPECIFIC:32})) {
  Object.defineProperty(Node, name, {value, enumerable:true});
  Object.defineProperty(Node.prototype, name, {value, enumerable:true});
}
for (const [name, cls] of Object.entries({EventTarget, Node, Element, HTMLElement, SVGElement,
    SVGGeometryElement, SVGPathElement, MathMLElement,
    HTMLInputElement, HTMLTextAreaElement, HTMLButtonElement, HTMLFormElement,
    HTMLSelectElement, HTMLOptionElement, HTMLLabelElement, HTMLDetailsElement,
    HTMLDialogElement, HTMLScriptElement, HTMLStyleElement, HTMLLinkElement,
    HTMLMetaElement, HTMLIFrameElement, HTMLImageElement, HTMLCanvasElement, HTMLTemplateElement,
    HTMLHeadingElement, HTMLAnchorElement, HTMLMediaElement, HTMLVideoElement, HTMLAudioElement,
    Document, DocumentFragment, ShadowRoot, ElementInternals, Text, Comment, NodeList, HTMLCollection})) {
  Object.defineProperty(globalThis, name, {value:cls, writable:true, configurable:true});
  Object.defineProperty(cls.prototype, Symbol.toStringTag, {value:name, configurable:true});
  install(name, cls.prototype);
}
SVGGeometryElement.prototype.getTotalLength=function(){return invoke(this,'getTotalLength');};
globalThis.__domUpgradeCustomElement=function(element,constructor){
  Object.setPrototypeOf(element,constructor.prototype);
  customElementConstructionStack.push(element);
  try {
    const result=Reflect.construct(constructor,[],constructor);
    if(result!==element)throw new TypeError('Custom element constructor returned another object');
  } finally { customElementConstructionStack.pop(); }
  return element;
};
const shadowRoots=new WeakMap(), elementInternals=new WeakMap();
Element.prototype.attachShadow=function(init={}) {
  if(shadowRoots.has(this))throw new DOMException('Shadow root already attached','NotSupportedError');
  const mode=String(init.mode||'');
  if(mode!=='open'&&mode!=='closed')throw new TypeError('Shadow root mode must be open or closed');
  const root=this.ownerDocument.createDocumentFragment();
  Object.setPrototypeOf(root,ShadowRoot.prototype);
  Object.defineProperties(root,{host:{value:this},mode:{value:mode},delegatesFocus:{value:!!init.delegatesFocus},slotAssignment:{value:init.slotAssignment||'named'}});
  shadowRoots.set(this,root); return root;
};
Object.defineProperty(Element.prototype,'shadowRoot',{get(){const root=shadowRoots.get(this);return root?.mode==='open'?root:null;}});
HTMLElement.prototype.attachInternals=function() {
  let internals=elementInternals.get(this); if(internals)return internals;
  internals=Object.create(ElementInternals.prototype);
  Object.defineProperties(internals,{shadowRoot:{get:()=>shadowRoots.get(this)||null},form:{get:()=>this.closest('form')},labels:{get:()=>[]},willValidate:{value:true},validity:{value:{valid:true}},validationMessage:{value:''}});
  elementInternals.set(this,internals); return internals;
};
function Image(width,height) {
  const image=document.createElement('img');
  if(width!==undefined)image.width=Number(width);
  if(height!==undefined)image.height=Number(height);
  return image;
}
Image.prototype=HTMLImageElement.prototype;
Object.defineProperty(Image.prototype,'constructor',{value:Image,writable:true,configurable:true});
globalThis.Image=Image;
for (const name of ['appendChild', 'insertBefore', 'removeChild', 'cloneNode'])
  Node.prototype[name] = function(...args) { return invoke(this, name, ...args); };
for (const name of ['createElement','createElementNS','createTextNode','createComment','createDocumentFragment'])
  Document.prototype[name] = function(...args) { return invoke(this, name, ...args.map(x => x == null ? x : String(x))); };
Document.prototype.write = function(...text) { invoke(this, 'write', text.map(String).join('')); };
Document.prototype.writeln = function(...text) { invoke(this, 'write', text.map(String).join('') + '\n'); };
// Blink implements Node::contains against the native DOM tree. Keep the
// ancestry walk on the host side as well: MutationObserver subtree matching
// calls this for every mutation, and walking through parentNode here would
// cross the JavaScript/host boundary once per ancestor.
Node.prototype.contains = function(other) { return invoke(this, 'contains', other); };
Node.prototype.hasChildNodes = function() { return this.firstChild !== null; };
Node.prototype.getRootNode = function() { let n = this; while (n.parentNode) n = n.parentNode; return n; };
Object.defineProperty(Node.prototype, 'isConnected', {get() { return this.getRootNode().nodeType === 9; }});
for (const [name, reverse, elements] of [['nextSibling',false,false],['previousSibling',true,false],
    ['nextElementSibling',false,true],['previousElementSibling',true,true]])
  Object.defineProperty(Node.prototype, name, {get() {
    return invoke(this, '__sibling', reverse, elements);
  }});
Object.defineProperty(Node.prototype, 'firstElementChild', {get() { return this.children?.[0] || null; }});
Object.defineProperty(Node.prototype, 'lastElementChild', {get() { return this.children?.[this.children.length-1] || null; }});
Object.defineProperty(Node.prototype, 'childElementCount', {get() { return this.children?.length || 0; }});
Element.prototype.append = function(...nodes) { for (let node of nodes) this.appendChild(node instanceof Node ? node : document.createTextNode(String(node))); };
Element.prototype.prepend = function(...nodes) { const first = this.firstChild; for (let node of nodes) this.insertBefore(node instanceof Node ? node : document.createTextNode(String(node)), first); };
Element.prototype.replaceChildren = function(...nodes) { while (this.firstChild) this.removeChild(this.firstChild); this.append(...nodes); };
for(const prototype of [Document.prototype,DocumentFragment.prototype]) {
  prototype.append=Element.prototype.append;
  prototype.prepend=Element.prototype.prepend;
  prototype.replaceChildren=Element.prototype.replaceChildren;
}
function childNodeValue(node) {
  return node instanceof Node ? node : document.createTextNode(String(node));
}
function childBefore(...nodes) {
  const parent=this.parentNode;
  if(!parent)return;
  for(const node of nodes)parent.insertBefore(childNodeValue(node),this);
}
function childAfter(...nodes) {
  const parent=this.parentNode;
  if(!parent)return;
  const reference=this.nextSibling;
  for(const node of nodes)parent.insertBefore(childNodeValue(node),reference);
}
function childReplaceWith(...nodes) {
  const parent=this.parentNode;
  if(!parent)return;
  childBefore.apply(this,nodes);
  if(this.parentNode===parent)parent.removeChild(this);
}
function childRemove() { if(this.parentNode)this.parentNode.removeChild(this); }
for(const prototype of [Element.prototype,Text.prototype,Comment.prototype]) {
  prototype.before=childBefore;
  prototype.after=childAfter;
  prototype.replaceWith=childReplaceWith;
  prototype.remove=childRemove;
}
Element.prototype.toggleAttribute = function(name, force) {
  name=String(name); const present=this.hasAttribute(name);
  const enabled=force===undefined?!present:!!force;
  if(enabled&&!present)this.setAttribute(name,''); else if(!enabled&&present)this.removeAttribute(name);
  return enabled;
};
HTMLElement.prototype.focus = function() { invoke(this, 'focus'); };
HTMLElement.prototype.blur = function() { invoke(this, 'blur'); };
for (const prototype of [HTMLInputElement.prototype, HTMLTextAreaElement.prototype,
                         HTMLButtonElement.prototype, HTMLSelectElement.prototype]) {
  prototype.checkValidity=function(){return true;};
  prototype.reportValidity=function(){return this.checkValidity();};
  prototype.setCustomValidity=function(message){this._validationMessage=String(message);};
}
for (const prototype of [HTMLInputElement.prototype, HTMLTextAreaElement.prototype,
                         HTMLButtonElement.prototype, HTMLSelectElement.prototype])
  Object.defineProperty(prototype,'form',{get(){const owner=this.getAttribute('form');return owner!==null?(this.ownerDocument.getElementById(owner) instanceof HTMLFormElement?this.ownerDocument.getElementById(owner):null):this.closest('form');}});
Object.defineProperty(HTMLFormElement.prototype,'elements',{get(){return this.querySelectorAll('button,input,select,textarea');}});
HTMLFormElement.prototype.checkValidity=function(){return [...this.elements].every(control=>typeof control.checkValidity!=='function'||control.checkValidity());};
HTMLFormElement.prototype.reportValidity=function(){return this.checkValidity();};
HTMLFormElement.prototype.requestSubmit=function(submitter=null){
  if(submitter!==null && !((submitter instanceof HTMLButtonElement && !['button','reset'].includes((submitter.getAttribute('type')||'').toLowerCase())) || (submitter instanceof HTMLInputElement && ['submit','image'].includes((submitter.getAttribute('type')||'').toLowerCase()))))throw new TypeError('Submitter must be a submit button');
  if(submitter!==null && submitter.form!==this)throw new DOMException('Submitter does not belong to this form','NotFoundError');
  invoke(this,'requestSubmit',submitter);
};
HTMLFormElement.prototype.submit=function(){invoke(this,'submit');};
Document.prototype.hasFocus = function() { return invoke(this, 'hasFocus'); };
for (const [property,tag] of [['scripts','script'],['images','img'],['forms','form'],['embeds','embed'],['plugins','embed']])
  Object.defineProperty(Document.prototype,property,{configurable:true,enumerable:true,get(){return this.getElementsByTagName(tag);}});
Document.prototype.getElementsByName = function(name) {
  const escaped=String(name).replace(/([\\"])/g,'\\$1');
  return this.querySelectorAll('[name="'+escaped+'"]');
};
class Range {
  constructor(document) { this._document=document;this._context=document.body||document.documentElement;this.collapsed=true; }
  selectNode(node) { if(!(node instanceof Node))throw new TypeError('Range node required');this._context=node.parentElement||node;this.collapsed=false; }
  selectNodeContents(node) { if(!(node instanceof Node))throw new TypeError('Range node required');this._context=node;this.collapsed=false; }
  collapse(toStart=false) { this.collapsed=true; }
  createContextualFragment(markup) {
    const document=this._context?.ownerDocument||this._document;
    const context=this._context?.nodeType===1?this._context:(document.body||document.documentElement);
    const container=document.createElement(context?.tagName?.toLowerCase()||'body');
    container.innerHTML=String(markup);
    const fragment=document.createDocumentFragment();
    while(container.firstChild)fragment.appendChild(container.firstChild);
    return fragment;
  }
}
Object.defineProperty(Range.prototype,Symbol.toStringTag,{value:'Range',configurable:true});
globalThis.Range=Range;
Document.prototype.createRange=function(){return new Range(this);};
const NodeFilter=Object.freeze({
  FILTER_ACCEPT:1,FILTER_REJECT:2,FILTER_SKIP:3,
  SHOW_ALL:0xffffffff,SHOW_ELEMENT:1,SHOW_ATTRIBUTE:2,SHOW_TEXT:4,
  SHOW_CDATA_SECTION:8,SHOW_ENTITY_REFERENCE:16,SHOW_ENTITY:32,
  SHOW_PROCESSING_INSTRUCTION:64,SHOW_COMMENT:128,SHOW_DOCUMENT:256,
  SHOW_DOCUMENT_TYPE:512,SHOW_DOCUMENT_FRAGMENT:1024,SHOW_NOTATION:2048
});
const nodeTypeMask=node=>({1:1,2:2,3:4,4:8,5:16,6:32,7:64,8:128,9:256,10:512,11:1024,12:2048}[node.nodeType]||0);
class TreeWalker {
  constructor(root,whatToShow=NodeFilter.SHOW_ALL,filter=null) {
    if(!(root instanceof Node))throw new TypeError('TreeWalker root must be a Node');
    this.root=root;this.whatToShow=Number(whatToShow)>>>0;this.filter=filter;this.currentNode=root;
  }
  _decision(node) {
    if(!(this.whatToShow&nodeTypeMask(node)))return NodeFilter.FILTER_SKIP;
    if(this.filter==null)return NodeFilter.FILTER_ACCEPT;
    const result=typeof this.filter==='function'?this.filter(node):this.filter.acceptNode(node);
    const decision=Number(result);
    if(decision!==1&&decision!==2&&decision!==3)throw new TypeError('Invalid NodeFilter result');
    return decision;
  }
  _firstVisibleWithin(parent) {
    for(let child=parent.firstChild;child;child=child.nextSibling) {
      const decision=this._decision(child);
      if(decision===NodeFilter.FILTER_ACCEPT)return child;
      if(decision===NodeFilter.FILTER_SKIP) {const nested=this._firstVisibleWithin(child);if(nested)return nested;}
    }
    return null;
  }
  firstChild() {
    const result=this._firstVisibleWithin(this.currentNode);
    if(result)this.currentNode=result;
    return result;
  }
  nextSibling() {
    const original=this.currentNode;
    for(let sibling=original.nextSibling;sibling;sibling=sibling.nextSibling) {
      const decision=this._decision(sibling);
      if(decision===NodeFilter.FILTER_ACCEPT){this.currentNode=sibling;return sibling;}
      if(decision===NodeFilter.FILTER_SKIP) {const nested=this._firstVisibleWithin(sibling);if(nested){this.currentNode=nested;return nested;}}
    }
    return null;
  }
  parentNode() {
    for(let parent=this.currentNode.parentNode;parent;parent=parent.parentNode) {
      if(parent===this.root) {
        if(this._decision(parent)===NodeFilter.FILTER_ACCEPT){this.currentNode=parent;return parent;}
        return null;
      }
      if(this._decision(parent)===NodeFilter.FILTER_ACCEPT){this.currentNode=parent;return parent;}
    }
    return null;
  }
  nextNode() {
    const original=this.currentNode;
    const descendant=this._firstVisibleWithin(original);
    if(descendant){this.currentNode=descendant;return descendant;}
    for(let node=original;node&&node!==this.root;node=node.parentNode) {
      for(let sibling=node.nextSibling;sibling;sibling=sibling.nextSibling) {
        const decision=this._decision(sibling);
        if(decision===NodeFilter.FILTER_ACCEPT){this.currentNode=sibling;return sibling;}
        if(decision===NodeFilter.FILTER_SKIP) {const nested=this._firstVisibleWithin(sibling);if(nested){this.currentNode=nested;return nested;}}
      }
    }
    return null;
  }
}
Object.defineProperty(TreeWalker.prototype,Symbol.toStringTag,{value:'TreeWalker',configurable:true});
Object.assign(globalThis,{NodeFilter,TreeWalker});
Document.prototype.createTreeWalker=function(root,whatToShow=NodeFilter.SHOW_ALL,filter=null){return new TreeWalker(root,whatToShow,filter);};
class XMLSerializer {
  serializeToString(node) {
    if(!(node instanceof Node))throw new TypeError('XMLSerializer requires a Node');
    return invoke(node,'serializeNode');
  }
}
Object.defineProperty(XMLSerializer.prototype,Symbol.toStringTag,{value:'XMLSerializer',configurable:true});
globalThis.XMLSerializer=XMLSerializer;
Object.defineProperty(Element.prototype,'attributes',{get(){
  const raw=invoke(this,'attributeList'),result=[];
  for(let i=0;i<raw.length;i++){
    const attribute=raw[i];
    result.push(attribute);
    Object.defineProperty(result,attribute.name,{value:attribute,enumerable:false,configurable:true});
  }
  Object.defineProperties(result,{
    item:{value:index=>result[index]||null},
    getNamedItem:{value:name=>result.find(attribute=>attribute.name===String(name))||null}
  });
  return result;
}});
class CSSStyleRule { constructor(text) { this.cssText = text; this.type = 1; this.selectorText = text.slice(0,text.indexOf('{')).trim(); } }
class CSSStyleSheet {
  constructor(index, document) { this._index = index; this._document = document; }
  get ownerNode() { return invoke(this._document, 'stylesheetOwner', this._index); }
  get cssRules() { const rules = []; for (let i = 0; i < invoke(this._document,'stylesheetRuleCount',this._index); ++i)
    rules.push(new CSSStyleRule(invoke(this._document,'stylesheetRuleText',this._index,i))); return rules; }
  get rules() { return this.cssRules; }
  insertRule(rule,index=0) {return invoke(this._document,'stylesheetInsertRule',this._index,Number(index),String(rule));}
  deleteRule(index) {invoke(this._document,'stylesheetDeleteRule',this._index,Number(index));}
}
globalThis.CSSStyleRule = CSSStyleRule; globalThis.CSSStyleSheet = CSSStyleSheet;
const sheetLists = new WeakMap();
Object.defineProperty(Document.prototype, 'styleSheets', {get() {
  if(!sheetLists.has(this)) {
    const document=this, objects=[];
    const list=new Proxy({}, {get(_, key) {
      const count=invoke(document,'stylesheetCount');
      if(key==='length')return count;
      if(key===Symbol.iterator)return function*(){for(let i=0;i<count;i++)yield list[i];};
      if(key==='item')return index=>list[index]||null;
      if(/^\d+$/.test(String(key)) && Number(key)<count)return objects[key] ||= new CSSStyleSheet(Number(key),document);
    }});sheetLists.set(this,list);
  }return sheetLists.get(this);
}});
Object.defineProperty(HTMLElement.prototype,'sheet',{get(){
  if(!['STYLE','LINK'].includes(this.tagName))return undefined;
  return [...this.ownerDocument.styleSheets].find(sheet=>sheet.ownerNode===this)||null;
}});
const datasets = new WeakMap(), styles = new WeakMap(), classes = new WeakMap();
const dataName = key => 'data-' + key.replace(/[A-Z]/g, c => '-' + c.toLowerCase());
const datasetDescriptor = {get() {
  if (!datasets.has(this)) { const node = this; datasets.set(this, new Proxy({}, {
    get(_, key) { if (typeof key !== 'string') return undefined; return node.getAttribute(dataName(key)) ?? undefined; },
    set(_, key, value) { node.setAttribute(dataName(key), String(value)); return true; },
    deleteProperty(_, key) { node.removeAttribute(dataName(key)); return true; },
    has(_, key) { return node.hasAttribute(dataName(key)); }
  })); } return datasets.get(this);
}};
// HTMLOrSVGElement supplies dataset to both element families in Chromium.
Object.defineProperty(HTMLElement.prototype, 'dataset', datasetDescriptor);
Object.defineProperty(SVGElement.prototype, 'dataset', datasetDescriptor);
const cssName = key => key === 'cssFloat' ? 'float' : key.replace(/[A-Z]/g, c => '-' + c.toLowerCase());
Object.defineProperty(Element.prototype, 'style', {get() {
  if (!styles.has(this)) { const node = this; const methods = {
    getPropertyValue(name) { return invoke(node, 'getStyle', String(name)); },
    setProperty(name, value, priority = '') { invoke(node, 'setStyle', String(name), String(value ?? ''), String(priority)); },
    removeProperty(name) { const old = this.getPropertyValue(name); this.setProperty(name, ''); return old; }
  }; styles.set(this, new Proxy(methods, {
    get(target, key) { if (key in target || typeof key !== 'string') return target[key];
      if (key === 'cssText') return node.getAttribute('style') || '';
      return invoke(node, 'getStyle', cssName(key)); },
    set(_, key, value) { if (key === 'cssText') node.setAttribute('style', String(value));
      else invoke(node, 'setStyle', cssName(key), String(value ?? ''), ''); return true; }
  })); } return styles.get(this);
}, set(value) { this.setAttribute('style', String(value)); }});
for (const property of ['name','type','src','href','rel','as','title','alt','nonce','dir','lang','action','method','target','enctype','placeholder','autocomplete'])
  Object.defineProperty(HTMLElement.prototype, property, {get() { return this.getAttribute(property) || ''; }, set(value) { this.setAttribute(property, String(value)); }});
for (const [property,attribute] of [['acceptCharset','accept-charset']])
  Object.defineProperty(HTMLFormElement.prototype,property,{get(){return this.getAttribute(attribute)||'';},set(value){this.setAttribute(attribute,String(value));}});
Object.defineProperty(HTMLFormElement.prototype,'action',{get(){
  const document=this.ownerDocument, raw=this.getAttribute('action');
  if(!raw)return document.URL;
  const base=document.querySelector('base[href]')?.getAttribute('href');
  try{return new URL(raw,base?new URL(base,document.URL).href:document.URL).href;}catch{return raw;}
},set(value){this.setAttribute('action',String(value));}});
for (const [property,allowed,fallback] of [['method',['get','post','dialog'],'get'],['enctype',['application/x-www-form-urlencoded','multipart/form-data','text/plain'],'application/x-www-form-urlencoded']])
  Object.defineProperty(HTMLFormElement.prototype,property,{get(){const value=(this.getAttribute(property)||'').toLowerCase();return allowed.includes(value)?value:fallback;},set(value){this.setAttribute(property,String(value));}});
for (const prototype of [HTMLInputElement.prototype,HTMLButtonElement.prototype])
  for (const [property,attribute] of [['formAction','formaction'],['formMethod','formmethod'],['formTarget','formtarget'],['formEnctype','formenctype']])
    Object.defineProperty(prototype,property,{get(){return this.getAttribute(attribute)||'';},set(value){this.setAttribute(attribute,String(value));}});
for (const [prototype,property,attribute] of [[HTMLFormElement.prototype,'noValidate','novalidate'],[HTMLInputElement.prototype,'formNoValidate','formnovalidate'],[HTMLButtonElement.prototype,'formNoValidate','formnovalidate']])
  Object.defineProperty(prototype,property,{get(){return this.hasAttribute(attribute);},set(value){if(value)this.setAttribute(attribute,'');else this.removeAttribute(attribute);}});
for (const property of ['hidden','disabled','multiple','required','autofocus'])
  Object.defineProperty(HTMLElement.prototype, property, {get() { return this.hasAttribute(property); }, set(value) { if (value) this.setAttribute(property, ''); else this.removeAttribute(property); }});
Object.defineProperty(Element.prototype, 'classList', {get() {
  if (!classes.has(this)) { const node = this;
    const tokens = () => (node.getAttribute('class') || '').split(/\s+/).filter(Boolean);
    const save = list => node.setAttribute('class', [...new Set(list)].join(' '));
    classes.set(this, {contains(token) { return tokens().includes(String(token)); },
      add(...items) { save([...tokens(), ...items.map(String)]); },
      remove(...items) { save(tokens().filter(t => !items.map(String).includes(t))); },
      toggle(token, force) { const present = this.contains(token); const add = force === undefined ? !present : !!force;
        if (add) this.add(token); else this.remove(token); return add; },
      get value() { return node.className; }, set value(v) { node.className = String(v); },
      [Symbol.iterator]() { return tokens()[Symbol.iterator](); }
    }); } return classes.get(this);
}});
const observers = new Set(); let mutationDeliveryQueued=false;
class MutationRecord {constructor(values){Object.assign(this,values);}}
class MutationObserver {
  constructor(callback){if(typeof callback!=='function')throw new TypeError('MutationObserver callback required');this._callback=callback;this._targets=new Map();this._records=[];}
  observe(target,options={}) {
    if(!(target instanceof Node))throw new TypeError('MutationObserver target must be a Node');
    const o={...options};
    if(o.attributes===undefined && (o.attributeOldValue!==undefined || o.attributeFilter!==undefined))o.attributes=true;
    if(o.characterData===undefined && o.characterDataOldValue!==undefined)o.characterData=true;
    if(!(o.childList||o.attributes||o.characterData) || (!o.attributes&&(o.attributeOldValue||o.attributeFilter)) || (!o.characterData&&o.characterDataOldValue))throw new TypeError('Invalid mutation observer options');
    if(o.attributeFilter!==undefined)o.attributeFilter=Array.from(o.attributeFilter,String);
    this._targets.set(target,o);const changed=!observers.has(this);observers.add(this);if(changed)invoke(document,'mutationObserverState',observers.size);
  }
  disconnect(){this._targets.clear();this._records=[];if(observers.delete(this))invoke(document,'mutationObserverState',observers.size);}
  takeRecords(){const records=this._records;this._records=[];return records;}
}
Object.assign(globalThis,{MutationObserver,MutationRecord});
wrap('__mutationRecord',(target,type,attributeName,oldValue,addedNodes,removedNodes,previousSibling,nextSibling)=>{
  let queued=false;
  for(const observer of observers){let matched=false,keepOld=false;
    for(const [node,o]of observer._targets){
      if(node!==target && !(o.subtree && node.contains(target)))continue;
      if(!o[type] || (type==='attributes'&&o.attributeFilter&&!o.attributeFilter.includes(attributeName)))continue;
      matched=true;keepOld ||= type==='attributes'?!!o.attributeOldValue:!!o.characterDataOldValue;
    }
    if(matched){observer._records.push(new MutationRecord({target,type,attributeName,attributeNamespace:null,oldValue:keepOld?oldValue:null,addedNodes,removedNodes,previousSibling,nextSibling}));queued=true;}
  }
  if(queued&&!mutationDeliveryQueued){mutationDeliveryQueued=true;queueMicrotask(()=>{
    mutationDeliveryQueued=false;
    for(const observer of [...observers]){const records=observer.takeRecords();if(records.length)try{observer._callback(records,observer);}catch(error){setTimeout(()=>{throw error;},0);}}
  });}
});
const implementations = new WeakMap();
Object.defineProperty(Document.prototype,'implementation',{get(){
  if(!implementations.has(this)) {const document=this;implementations.set(this,{
    createHTMLDocument(title) {return arguments.length ? invoke(document,'createHTMLDocument',String(title)) : invoke(document,'createHTMLDocument');}
  });}return implementations.get(this);
}});
const disconnectedOrder = new WeakMap(); let nextDisconnectedOrder = 1;
Node.prototype.compareDocumentPosition = function(other) {
  if (!(other instanceof Node)) throw new TypeError('compareDocumentPosition requires a Node');
  if (this === other) return 0;
  const path = node => {const result=[];for(;node;node=node.parentNode)result.unshift(node);return result;};
  const a=path(this), b=path(other);
  if(a[0]!==b[0]) {
    for(const root of [a[0],b[0]])if(!disconnectedOrder.has(root))disconnectedOrder.set(root,nextDisconnectedOrder++);
    return 1|32|(disconnectedOrder.get(a[0])<disconnectedOrder.get(b[0])?4:2);
  }
  let i=0;while(i<a.length && i<b.length && a[i]===b[i])++i;
  if(i===a.length)return 4|16;
  if(i===b.length)return 2|8;
  const siblings=Array.from(a[i-1].childNodes);
  return siblings.indexOf(a[i])<siblings.indexOf(b[i])?4:2;
};
const computedStyles = new WeakMap();
globalThis.getComputedStyle = function(node) {
  if (!(node instanceof Element)) throw new TypeError('getComputedStyle requires an Element');
  if (!computedStyles.has(node)) {
    const get = name => invoke(node,'computedStyle',String(name));
    computedStyles.set(node,new Proxy({getPropertyValue:get}, {
      get(target,key) { if(key in target || typeof key !== 'string') return target[key];return get(cssName(key)); },
      set() { throw new DOMException('Computed styles are read-only','NoModificationAllowedError'); }
    }));
  } return computedStyles.get(node);
};
for (const property of ['offsetWidth','offsetHeight','offsetLeft','offsetTop','clientWidth','clientHeight','clientLeft','clientTop','scrollWidth','scrollHeight'])
  Object.defineProperty(HTMLElement.prototype,property,{get(){return invoke(this,'metric',property);}});
for (const [property,axis] of [['scrollLeft','x'],['scrollTop','y']])
  Object.defineProperty(Element.prototype,property,{configurable:true,enumerable:true,get(){return invoke(this,'scrollOffset',axis);},set(value){invoke(this,'scrollOffset',axis,Number(value));}});
Element.prototype.scroll = Element.prototype.scrollTo = function(x=0,y=0) {
  if(x && typeof x==='object') {if(x.left!==undefined)this.scrollLeft=x.left;if(x.top!==undefined)this.scrollTop=x.top;}
  else {this.scrollLeft=x;this.scrollTop=y;}
};
Element.prototype.scrollBy = function(x=0,y=0) {
  if(x && typeof x==='object') {this.scrollLeft+=Number(x.left??0);this.scrollTop+=Number(x.top??0);}
  else {this.scrollLeft+=Number(x);this.scrollTop+=Number(y);}
};
class DOMRectReadOnly {
  constructor(x=0,y=0,width=0,height=0) { this.x=Number(x); this.y=Number(y); this.width=Number(width); this.height=Number(height); }
  get left() { return Math.min(this.x,this.x+this.width); } get right() { return Math.max(this.x,this.x+this.width); }
  get top() { return Math.min(this.y,this.y+this.height); } get bottom() { return Math.max(this.y,this.y+this.height); }
  toJSON() { return {x:this.x,y:this.y,width:this.width,height:this.height,top:this.top,right:this.right,bottom:this.bottom,left:this.left}; }
  static fromRect(r={}) { return new this(r.x,r.y,r.width,r.height); }
}
class DOMRect extends DOMRectReadOnly {}
Object.assign(globalThis,{DOMRectReadOnly,DOMRect});
Element.prototype.getClientRects = function() {
  const raw=invoke(this,'clientRects'), result=[];
  for(let i=0;i<raw.length;i++) result.push(DOMRect.fromRect(raw[i]));
  result.item=function(i) { return this[i] || null; }; return result;
};
Element.prototype.getBoundingClientRect = function() {
  const rects=this.getClientRects(); if(!rects.length) return new DOMRect();
  const nonempty=rects.filter(r=>r.width || r.height); if(!nonempty.length) return rects[0];
  const left=Math.min(...nonempty.map(r=>r.left)), top=Math.min(...nonempty.map(r=>r.top));
  return new DOMRect(left,top,Math.max(...nonempty.map(r=>r.right))-left,Math.max(...nonempty.map(r=>r.bottom))-top);
};
wrap('addEventListener', EventTarget.prototype.addEventListener);
wrap('removeEventListener', EventTarget.prototype.removeEventListener);
wrap('dispatchEvent', EventTarget.prototype.dispatchEvent);
for (const name of ['addEventListener','removeEventListener','dispatchEvent']) {
  const method = EventTarget.prototype[name];
  wrap('__window_' + name,(...args)=>method.call(globalThis,...args));
}
// DOM-only browser semantic repair, before NodeList.prototype.forEach.
// It intentionally replaces the previous per-get host method identity with
// standard per-interface prototype identity, separate from cache performance.
const collectionString = String, collectionTypeError = TypeError;
function collectionReceiver(receiver, expected) {
  if (invoke(receiver, '__domCollectionBrand') !== expected)
    throw new collectionTypeError('Illegal invocation');
}
function collectionArgumentCount(count, method, expected) {
  if (!count) throw new collectionTypeError("Failed to execute '" + method + "' on '" + expected + "': 1 argument required");
}
function collectionDOMString(value) {
  if (typeof value === 'symbol') throw new collectionTypeError('Cannot convert a Symbol value to a string');
  return collectionString(value);
}
const nodeListMethods = {
  item(index) {
    collectionReceiver(this, 'NodeList');
    collectionArgumentCount(arguments.length, 'item', 'NodeList');
    // Unary + is ToNumber, including throwing for BigInt and Symbol. >>>0
    // performs unsigned-long truncation/modulo after that single coercion.
    return invoke(this, 'item', (+index) >>> 0);
  }
};
const htmlCollectionMethods = {
  item(index) {
    collectionReceiver(this, 'HTMLCollection');
    collectionArgumentCount(arguments.length, 'item', 'HTMLCollection');
    return invoke(this, 'item', (+index) >>> 0);
  },
  namedItem(name) {
    collectionReceiver(this, 'HTMLCollection');
    collectionArgumentCount(arguments.length, 'namedItem', 'HTMLCollection');
    return invoke(this, 'namedItem', collectionDOMString(name));
  }
};
Object.defineProperty(NodeList.prototype, 'item', {value:nodeListMethods.item, writable:true, configurable:true, enumerable:true});
Object.defineProperty(HTMLCollection.prototype, 'item', {value:htmlCollectionMethods.item, writable:true, configurable:true, enumerable:true});
Object.defineProperty(HTMLCollection.prototype, 'namedItem', {value:htmlCollectionMethods.namedItem, writable:true, configurable:true, enumerable:true});
// Mark completed DOM-only installation. Actual reads use the original Reflect
// path so standard own/prototype replacement and getter exceptions still work.
wrap('__NodeList_item', nodeListMethods.item);
wrap('__HTMLCollection_item', htmlCollectionMethods.item);
wrap('__HTMLCollection_namedItem', htmlCollectionMethods.namedItem);

NodeList.prototype.forEach = Array.prototype.forEach;
NodeList.prototype.entries = Array.prototype.entries;
NodeList.prototype.keys = Array.prototype.keys;
NodeList.prototype.values = Array.prototype.values;
})()
