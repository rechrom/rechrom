(function() {
'use strict';
const invoke = __domInvoke, install = __domInstall, wrap = __domWrap;
const callbacks = new WeakMap(), eventStates = new WeakMap();
function callbackFunction(callback) {
  if (callback == null) return null;
  if (typeof callback === 'function') return callback;
  if (typeof callback !== 'object') throw new TypeError('Listener must be an object');
  let wrapped = callbacks.get(callback);
  if (!wrapped) { wrapped = function(event) { return callback.handleEvent(event); }; callbacks.set(callback, wrapped); }
  return wrapped;
}
class EventTarget {
  constructor() {
    const target = invoke(globalThis, '__eventTargetCreate');
    if (new.target !== EventTarget) Object.setPrototypeOf(target, new.target.prototype);
    return target;
  }
  addEventListener(type, callback, options) {
    invoke(this, '__eventTargetBrand');
    const object = options != null && (typeof options === 'object' || typeof options === 'function');
    const capture = object ? !!options.capture : !!options;
    const once = object ? !!options.once : false;
    const passiveValue = object ? options.passive : undefined;
    const passive = passiveValue === undefined ? undefined : !!passiveValue;
    const signal = object ? options.signal : undefined;
    const fn = callbackFunction(callback);
    if (signal != null) invoke(signal, '__abortSignalBrand');
    invoke(this, 'addEventListener', String(type), fn, capture, once, passive, signal);
  }
  removeEventListener(type, callback, options) {
    invoke(this, '__eventTargetBrand');
    const capture = options != null && typeof options === 'object' ? !!options.capture : !!options;
    const fn = callbackFunction(callback);
    invoke(this, 'removeEventListener', String(type), fn, capture);
  }
  dispatchEvent(event) {
    invoke(this, '__eventTargetBrand');
    const state = eventStates.get(event);
    if (!state) invoke(event, '__eventBrand');
    return invoke(this, 'dispatchEvent', String(event.type), !!event.bubbles, !!event.cancelable, !!event.composed, event, state?.native, state ? state.initialized : true, !!state);
  }
}
install('EventTarget', EventTarget.prototype);
Object.defineProperty(EventTarget.prototype, Symbol.toStringTag, {value:'EventTarget', configurable:true});
class Event {
  constructor(type, options = {}) {
    eventStates.set(this, {target:null,current:null,phase:0,dispatching:false,native:null,initialized:true});
    this.type = String(type); this.bubbles = !!options.bubbles; this.cancelable = !!options.cancelable;
    this.composed = !!options.composed; this.defaultPrevented = false; this.isTrusted = false;
    this.timeStamp = globalThis.performance ? performance.now() : 0;
  }
  initEvent(type, bubbles = false, cancelable = false) {
    const state = eventStates.get(this);
    if (!state) { invoke(this, '__eventInit', String(type), !!bubbles, !!cancelable); return; }
    if (state.native ? state.native.currentTarget !== null : state.dispatching) return;
    if (state.native) invoke(state.native, '__eventInit', String(type), !!bubbles, !!cancelable);
    state.initialized = true;
    this.type = String(type); this.bubbles = !!bubbles; this.cancelable = !!cancelable;
    this.defaultPrevented = false; this.isTrusted = false; this._stop = this._immediate = false;
  }
  get target() { const s=eventStates.get(this);return s ? (s.native ? s.native.target : s.target) : invoke(this,'__eventAttribute','target'); }
  get currentTarget() { const s=eventStates.get(this);return s ? (s.native ? s.native.currentTarget : s.current) : invoke(this,'__eventAttribute','currentTarget'); }
  get eventPhase() { const s=eventStates.get(this);return s ? (s.native ? s.native.eventPhase : s.phase) : invoke(this,'__eventAttribute','eventPhase'); }
  composedPath() {
    const state=eventStates.get(this);
    if (state) {
      if (state.native) return Array.from(invoke(state.native, '__eventComposedPath'));
      return state.dispatching ? [state.current] : [];
    }
    return Array.from(invoke(this, '__eventComposedPath'));
  }
  preventDefault() { if (this.cancelable && !this._passive) {this.defaultPrevented = true;const native=eventStates.get(this)?.native;if(native)invoke(native,'preventDefault');} }
  stopPropagation() { this._stop = true;const native=eventStates.get(this)?.native;if(native)invoke(native,'stopPropagation'); }
  stopImmediatePropagation() { this._stop = this._immediate = true;const native=eventStates.get(this)?.native;if(native)invoke(native,'stopImmediatePropagation'); }
}
globalThis.Event = Event;
__domInstall('Event', Event.prototype);
Object.defineProperty(Event.prototype, 'composedPath', {enumerable:true});

// Chromium: core/events/ui_event.{idl,cc} and mouse_event.{idl,cc}.
// Constructed UI events keep their WebIDL-owned state in JavaScript; dispatch
// still crosses the existing Event boundary and therefore preserves the same
// target/path/default-action ownership as Event.
class UIEvent extends Event {
  constructor(type, options = {}) {
    if (options == null) options = {};
    super(type, options);
    this.view = options.view === undefined ? null : options.view;
    this.detail = options.detail === undefined ? 0 : Number(options.detail) | 0;
    this.sourceCapabilities = options.sourceCapabilities === undefined ? null : options.sourceCapabilities;
  }
  initUIEvent(type, bubbles = false, cancelable = false, view = null, detail = 0) {
    const state = eventStates.get(this);
    if (!state) {
      const nativeView=view==null?null:invoke(globalThis,'__inputEventConvertView',view);
      invoke(this, '__inputEventInitUIEvent', String(type), !!bubbles, !!cancelable, nativeView, (+detail) | 0);
      return;
    }
    if (state?.dispatching) return;
    this.type = String(type); this.bubbles = !!bubbles; this.cancelable = !!cancelable;
    this.view = view; this.detail = Number(detail) | 0;
  }
  get which() { return 0; }
}
globalThis.UIEvent = UIEvent;
__domInstall('UIEvent', UIEvent.prototype);
Object.defineProperty(UIEvent.prototype, Symbol.toStringTag, {value:'UIEvent', configurable:true});

const modifierNames = new Map([
  ['Alt','altKey'], ['AltGraph','modifierAltGraph'], ['CapsLock','modifierCapsLock'],
  ['Control','ctrlKey'], ['Fn','modifierFn'], ['Meta','metaKey'],
  ['NumLock','modifierNumLock'], ['ScrollLock','modifierScrollLock'],
  ['Shift','shiftKey'], ['Symbol','modifierSymbol']
]);
class MouseEvent extends UIEvent {
  constructor(type, options = {}) {
    if (options == null) options = {};
    super(type, options);
    this.screenX = Number(options.screenX ?? 0); this.screenY = Number(options.screenY ?? 0);
    this.clientX = Number(options.clientX ?? 0); this.clientY = Number(options.clientY ?? 0);
    this.pageX = this.clientX + (this.view?.scrollX || 0);
    this.pageY = this.clientY + (this.view?.scrollY || 0);
    this.x = this.clientX; this.y = this.clientY;
    this.offsetX = this.clientX; this.offsetY = this.clientY;
    this.movementX = Number(options.movementX ?? 0) | 0;
    this.movementY = Number(options.movementY ?? 0) | 0;
    this.ctrlKey = !!options.ctrlKey; this.shiftKey = !!options.shiftKey;
    this.altKey = !!options.altKey; this.metaKey = !!options.metaKey;
    this.modifierAltGraph = !!options.modifierAltGraph;
    this.modifierCapsLock = !!options.modifierCapsLock;
    this.modifierFn = !!options.modifierFn; this.modifierNumLock = !!options.modifierNumLock;
    this.modifierScrollLock = !!options.modifierScrollLock;
    this.modifierSymbol = !!options.modifierSymbol;
    this.button = Number(options.button ?? 0) << 16 >> 16;
    this.buttons = Number(options.buttons ?? 0) & 0xffff;
    this.relatedTarget = options.relatedTarget === undefined ? null : options.relatedTarget;
    this.layerX = this.pageX | 0; this.layerY = this.pageY | 0;
  }
  get which() { return (this.button + 1) >>> 0; }
  getModifierState(key) { const member = modifierNames.get(String(key)); return member ? !!this[member] : false; }
  initMouseEvent(type, bubbles = false, cancelable = false, view = null, detail = 0,
                 screenX = 0, screenY = 0, clientX = 0, clientY = 0,
                 ctrlKey = false, altKey = false, shiftKey = false, metaKey = false,
                 button = 0, relatedTarget = null) {
    const state = eventStates.get(this);
    if (state?.dispatching) return;
    this.initUIEvent(type, bubbles, cancelable, view, detail);
    this.screenX = Number(screenX); this.screenY = Number(screenY);
    this.clientX = Number(clientX); this.clientY = Number(clientY);
    this.pageX = this.clientX + (view?.scrollX || 0); this.pageY = this.clientY + (view?.scrollY || 0);
    this.x = this.clientX; this.y = this.clientY; this.offsetX = this.clientX; this.offsetY = this.clientY;
    this.ctrlKey = !!ctrlKey; this.altKey = !!altKey; this.shiftKey = !!shiftKey; this.metaKey = !!metaKey;
    this.button = Number(button) << 16 >> 16; this.buttons = 0; this.relatedTarget = relatedTarget;
    this.layerX = this.pageX | 0; this.layerY = this.pageY | 0;
  }
  get fromElement() { return this.type === 'mouseout' || this.type === 'mouseleave' ? this.target : this.relatedTarget; }
  get toElement() { return this.type === 'mouseout' || this.type === 'mouseleave' ? this.relatedTarget : this.target; }
}
globalThis.MouseEvent = MouseEvent;
__domInstall('MouseEvent', MouseEvent.prototype);
Object.defineProperty(MouseEvent.prototype, Symbol.toStringTag, {value:'MouseEvent', configurable:true});

__domWrap('__eventState', (event,target,current,phase,passive,native) => {
  const state=eventStates.get(event);
  if (!state) throw new TypeError('Invalid event dispatch');
  state.target=target;state.current=current;state.phase=phase;
  state.dispatching=current!==null;state.native=native||null;
  event._passive=passive;
  if (current===null) event._stop=event._immediate=false;
});
wrap('__callEventHandler',function(callback,event){if(callback.call(this,event)===false)event.preventDefault();});
const customElementConstructionStack=[];
function constructHTMLElement(newTarget, activeInterface, tag) {
  const entry = customElementConstructionStack.at(-1);
  if (!entry || entry.constructor !== newTarget)
    throw new TypeError('Direct custom element construction is not supported');
  // HTMLConstructor gets the current new.target.prototype at super(), then
  // consumes the construction-stack entry. Never use prototype identity as
  // the native interface brand and never permit another super() to reuse it.
  let prototype = newTarget.prototype;
  if (prototype === null || (typeof prototype !== 'object' && typeof prototype !== 'function'))
    prototype = activeInterface.prototype;
  if (entry.element === null) throw new TypeError('Custom element already constructed');
  const element = entry.element;
  invoke(element, '__htmlConstructionBrand', tag);
  entry.element = null;
  Object.setPrototypeOf(element, prototype);
  return element;
}
class Node extends EventTarget {
  constructor() { super(); throw new TypeError('Illegal constructor'); }
}
class Element extends Node {
  get parentElement() { const p = this.parentNode; return p?.nodeType === 1 ? p : null; }
  hasAttributes() { return invoke(this, 'hasAttributes'); }
  getAttributeNames() { return Array.from(invoke(this, 'getAttributeNames')); }
}
class HTMLElement extends Element {
  constructor() { return constructHTMLElement(new.target, HTMLElement, ''); }
}
class SVGElement extends Element {}
class SVGGeometryElement extends SVGElement {}
class SVGPathElement extends SVGGeometryElement {}
class MathMLElement extends Element {}
class HTMLInputElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLInputElement, 'input'); }}
class HTMLTextAreaElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLTextAreaElement, 'textarea'); }}
class HTMLButtonElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLButtonElement, 'button'); }}
class HTMLFormElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLFormElement, 'form'); }}
class HTMLSelectElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLSelectElement, 'select'); }}
class HTMLOptionElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLOptionElement, 'option'); }}
class HTMLLabelElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLLabelElement, 'label'); }}
class HTMLDetailsElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLDetailsElement, 'details'); }}
class HTMLPreElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLPreElement, 'pre'); }}
class HTMLDialogElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLDialogElement, 'dialog'); }}
class HTMLScriptElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLScriptElement, 'script'); }}
class HTMLStyleElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLStyleElement, 'style'); }}
class HTMLLinkElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLLinkElement, 'link'); }}
class HTMLMetaElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLMetaElement, 'meta'); }}
class HTMLIFrameElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLIFrameElement, 'iframe'); }
  get contentWindow() { return invoke(this, '__frameContentWindow'); }
  get contentDocument() { return invoke(this, '__frameContentDocument'); }
}
for (const member of ['contentWindow', 'contentDocument']) {
  const descriptor = Object.getOwnPropertyDescriptor(HTMLIFrameElement.prototype, member);
  Object.defineProperty(HTMLIFrameElement.prototype, member, {...descriptor, enumerable:true});
}
class HTMLImageElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLImageElement, 'img'); }}
function canvasUnsignedLong(value) {
  // WebIDL unsigned long conversion: ToNumber (which rejects Symbol/BigInt),
  // truncate, then modulo 2^32.
  return (+value) >>> 0;
}
class HTMLCanvasElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLCanvasElement, 'canvas'); }
  get width() { return invoke(this, '__canvasDimension', 'width'); }
  set width(value) { invoke(this, '__canvasSetDimension', 'width', canvasUnsignedLong(value)); }
  get height() { return invoke(this, '__canvasDimension', 'height'); }
  set height(value) { invoke(this, '__canvasSetDimension', 'height', canvasUnsignedLong(value)); }
  getContext(contextId, options = undefined) {
    // Conversion is observable even when no provider supports the requested
    // context. The host owns context-family arbitration and object identity.
    const id = attributeDOMString(contextId);
    let desynchronized = false;
    if (id === 'webgl' || id === 'experimental-webgl' || id === 'webgl2') {
      if (options !== undefined && options !== null) {
        if (typeof options !== 'object' && typeof options !== 'function')
          throw new TypeError('Canvas context attributes must be a dictionary');
        desynchronized = Boolean(options.desynchronized);
      }
    }
    // The requested attribute crosses as a WebIDL-converted scalar. Provider
    // admission computes the effective attribute on the retained context.
    return invoke(this, 'getContext', id, desynchronized);
  }
}
for (const member of ['width', 'height', 'getContext']) {
  const descriptor = Object.getOwnPropertyDescriptor(HTMLCanvasElement.prototype, member);
  Object.defineProperty(HTMLCanvasElement.prototype, member, {...descriptor, enumerable:true});
}
const webglObjectKey = Symbol('WebGLObject');
class WebGLShader {
  constructor(id, key) { if (key !== webglObjectKey) throw new TypeError('Illegal constructor'); this._id=id; }
}
class WebGLProgram {
  constructor(id, key) { if (key !== webglObjectKey) throw new TypeError('Illegal constructor'); this._id=id; }
}
class WebGLBuffer {
  constructor(id, key) { if (key !== webglObjectKey) throw new TypeError('Illegal constructor'); this._id=id; }
}
class WebGLTexture {
  constructor(id, key) { if (key !== webglObjectKey) throw new TypeError('Illegal constructor'); this._id=id; }
}
class WebGLUniformLocation {
  constructor(id, key) { if (key !== webglObjectKey) throw new TypeError('Illegal constructor'); this._id=id; }
}
class WebGLVertexArrayObject {
  constructor(id, key) { if (key !== webglObjectKey) throw new TypeError('Illegal constructor'); this._id=id; }
}
function webglId(value, type) {
  if (!(value instanceof type)) throw new TypeError('WebGL object has the wrong type');
  return value._id;
}
function webglBytes(value) {
  if (!ArrayBuffer.isView(value)) throw new TypeError('WebGL uniform data requires an ArrayBufferView');
  return new Uint8Array(value.buffer,value.byteOffset,value.byteLength).slice().buffer;
}
class WebGLRenderingContext extends EventTarget {
  constructor() { throw new TypeError('Illegal constructor'); }
  createShader(type) { const id=invoke(this,'createShader',type>>>0);return id==null?null:new WebGLShader(id,webglObjectKey); }
  shaderSource(shader, source) { invoke(this,'shaderSource',webglId(shader,WebGLShader),String(source)); }
  compileShader(shader) { invoke(this,'compileShader',webglId(shader,WebGLShader)); }
  getShaderParameter(shader,pname) { return invoke(this,'getShaderParameter',webglId(shader,WebGLShader),pname>>>0); }
  getShaderInfoLog(shader) { return invoke(this,'getShaderInfoLog',webglId(shader,WebGLShader)); }
  createProgram() { return new WebGLProgram(invoke(this,'createProgram'),webglObjectKey); }
  attachShader(program,shader) { invoke(this,'attachShader',webglId(program,WebGLProgram),webglId(shader,WebGLShader)); }
  linkProgram(program) { invoke(this,'linkProgram',webglId(program,WebGLProgram)); }
  getProgramParameter(program,pname) { return invoke(this,'getProgramParameter',webglId(program,WebGLProgram),pname>>>0); }
  getProgramInfoLog(program) { return invoke(this,'getProgramInfoLog',webglId(program,WebGLProgram)); }
  useProgram(program) { invoke(this,'useProgram',webglId(program,WebGLProgram)); }
  getAttribLocation(program,name) { return invoke(this,'getAttribLocation',webglId(program,WebGLProgram),String(name)); }
  getActiveUniform(program,index) { return invoke(this,'getActiveUniform',webglId(program,WebGLProgram),index>>>0); }
  getActiveAttrib(program,index) { return invoke(this,'getActiveAttrib',webglId(program,WebGLProgram),index>>>0); }
  getUniformLocation(program,name) { const id=invoke(this,'getUniformLocation',webglId(program,WebGLProgram),String(name));return id==null?null:new WebGLUniformLocation(id,webglObjectKey); }
  deleteShader(shader) { invoke(this,'deleteShader',webglId(shader,WebGLShader)); }
  createBuffer() { return new WebGLBuffer(invoke(this,'createBuffer'),webglObjectKey); }
  bindBuffer(target,buffer) { invoke(this,'bindBuffer',target>>>0,buffer===null?null:webglId(buffer,WebGLBuffer)); }
  bufferData(target,data,usage) {
    if (!ArrayBuffer.isView(data)) throw new TypeError('bufferData requires an ArrayBufferView');
    const bytes=new Uint8Array(data.buffer,data.byteOffset,data.byteLength);
    invoke(this,'bufferData',target>>>0,bytes.slice().buffer,usage>>>0);
  }
  enableVertexAttribArray(index) { invoke(this,'enableVertexAttribArray',index>>>0); }
  vertexAttribPointer(index,size,type,normalized,stride,offset) { invoke(this,'vertexAttribPointer',index>>>0,Number(size),type>>>0,!!normalized,Number(stride),Number(offset)); }
  vertexAttrib2fv(index,values) { invoke(this,'vertexAttrib2fv',index>>>0,Number(values[0]),Number(values[1])); }
  createVertexArray() { return new WebGLVertexArrayObject(invoke(this,'createVertexArray'),webglObjectKey); }
  bindVertexArray(array) { invoke(this,'bindVertexArray',array===null?null:webglId(array,WebGLVertexArrayObject)); }
  createTexture() { return new WebGLTexture(invoke(this,'createTexture'),webglObjectKey); }
  bindTexture(target,texture) { invoke(this,'bindTexture',target>>>0,texture===null?null:webglId(texture,WebGLTexture)); }
  texParameteri(target,pname,param) { invoke(this,'texParameteri',target>>>0,pname>>>0,Number(param)); }
  texImage2D(target,level,internalFormat,width,height,border,format,type,pixels) {
    let bytes=null;
    if (pixels!==null) {
      if (!ArrayBuffer.isView(pixels)) throw new TypeError('texImage2D requires an ArrayBufferView or null');
      bytes=new Uint8Array(pixels.buffer,pixels.byteOffset,pixels.byteLength).slice().buffer;
    }
    invoke(this,'texImage2D',target>>>0,Number(level),Number(internalFormat),Number(width),Number(height),Number(border),format>>>0,type>>>0,bytes);
  }
  getShaderPrecisionFormat(shaderType,precisionType) { return invoke(this,'getShaderPrecisionFormat',shaderType>>>0,precisionType>>>0); }
  getParameter(pname) {
    pname=pname>>>0;
    const value=invoke(this,'getParameter',pname);
    if (pname===0x0BA2 || pname===0x0C10) return new Int32Array(value);
    if (pname===0x846D || pname===0x846E) return new Float32Array(value);
    return value;
  }
  getExtension(name) { return invoke(this,'getExtension',String(name)); }
  getContextAttributes() { return invoke(this,'getContextAttributes'); }
  viewport(x,y,width,height) { invoke(this,'viewport',Number(x),Number(y),Number(width),Number(height)); }
  clearColor(r,g,b,a) { invoke(this,'clearColor',Number(r),Number(g),Number(b),Number(a)); }
  clearDepth(depth) { invoke(this,'clearDepth',Number(depth)); }
  clearStencil(stencil) { invoke(this,'clearStencil',Number(stencil)); }
  enable(capability) { invoke(this,'enable',capability>>>0); }
  disable(capability) { invoke(this,'disable',capability>>>0); }
  depthFunc(func) { invoke(this,'depthFunc',func>>>0); }
  frontFace(mode) { invoke(this,'frontFace',mode>>>0); }
  cullFace(mode) { invoke(this,'cullFace',mode>>>0); }
  scissor(x,y,width,height) { invoke(this,'scissor',Number(x),Number(y),Number(width),Number(height)); }
  depthMask(flag) { invoke(this,'depthMask',!!flag); }
  colorMask(r,g,b,a) { invoke(this,'colorMask',!!r,!!g,!!b,!!a); }
  blendEquation(mode) { invoke(this,'blendEquation',mode>>>0); }
  blendFuncSeparate(srcRGB,dstRGB,srcAlpha,dstAlpha) { invoke(this,'blendFuncSeparate',srcRGB>>>0,dstRGB>>>0,srcAlpha>>>0,dstAlpha>>>0); }
  bindFramebuffer(target,framebuffer) { if (framebuffer!==null) throw new TypeError('WebGLFramebuffer is not supported yet');invoke(this,'bindFramebuffer',target>>>0,null); }
  drawBuffers(buffers) { invoke(this,'drawBuffers',Number(buffers[0])); }
  clear(mask) { invoke(this,'clear',mask>>>0); }
  drawArrays(mode,first,count) { invoke(this,'drawArrays',mode>>>0,Number(first),Number(count)); }
  drawElements(mode,count,type,offset) { invoke(this,'drawElements',mode>>>0,Number(count),type>>>0,Number(offset)); }
  uniformMatrix2fv(location,transpose,value) { invoke(this,'uniformMatrix2fv',webglId(location,WebGLUniformLocation),!!transpose,webglBytes(value)); }
  uniformMatrix3fv(location,transpose,value) { invoke(this,'uniformMatrix3fv',webglId(location,WebGLUniformLocation),!!transpose,webglBytes(value)); }
  uniformMatrix4fv(location,transpose,value) { invoke(this,'uniformMatrix4fv',webglId(location,WebGLUniformLocation),!!transpose,webglBytes(value)); }
  uniform1fv(location,value) { invoke(this,'uniform1fv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform2fv(location,value) { invoke(this,'uniform2fv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform3fv(location,value) { invoke(this,'uniform3fv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform4fv(location,value) { invoke(this,'uniform4fv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform2f(location,x,y) { invoke(this,'uniform2f',webglId(location,WebGLUniformLocation),Number(x),Number(y)); }
  uniform3f(location,x,y,z) { invoke(this,'uniform3f',webglId(location,WebGLUniformLocation),Number(x),Number(y),Number(z)); }
  uniform4f(location,x,y,z,w) { invoke(this,'uniform4f',webglId(location,WebGLUniformLocation),Number(x),Number(y),Number(z),Number(w)); }
  uniform1f(location,value) { invoke(this,'uniform1f',webglId(location,WebGLUniformLocation),Number(value)); }
  uniform1iv(location,value) { invoke(this,'uniform1iv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform2iv(location,value) { invoke(this,'uniform2iv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform3iv(location,value) { invoke(this,'uniform3iv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform4iv(location,value) { invoke(this,'uniform4iv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform1i(location,value) { invoke(this,'uniform1i',webglId(location,WebGLUniformLocation),Number(value)); }
  uniform2i(location,x,y) { invoke(this,'uniform2i',webglId(location,WebGLUniformLocation),Number(x),Number(y)); }
  uniform3i(location,x,y,z) { invoke(this,'uniform3i',webglId(location,WebGLUniformLocation),Number(x),Number(y),Number(z)); }
  uniform4i(location,x,y,z,w) { invoke(this,'uniform4i',webglId(location,WebGLUniformLocation),Number(x),Number(y),Number(z),Number(w)); }
  uniform1uiv(location,value) { invoke(this,'uniform1uiv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform2uiv(location,value) { invoke(this,'uniform2uiv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform3uiv(location,value) { invoke(this,'uniform3uiv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform4uiv(location,value) { invoke(this,'uniform4uiv',webglId(location,WebGLUniformLocation),webglBytes(value)); }
  uniform1ui(location,value) { invoke(this,'uniform1ui',webglId(location,WebGLUniformLocation),value>>>0); }
  uniform2ui(location,x,y) { invoke(this,'uniform2ui',webglId(location,WebGLUniformLocation),x>>>0,y>>>0); }
  uniform3ui(location,x,y,z) { invoke(this,'uniform3ui',webglId(location,WebGLUniformLocation),x>>>0,y>>>0,z>>>0); }
  uniform4ui(location,x,y,z,w) { invoke(this,'uniform4ui',webglId(location,WebGLUniformLocation),x>>>0,y>>>0,z>>>0,w>>>0); }
  getError() { return invoke(this,'getError'); }
  __snapshot() { return new Uint8Array(invoke(this,'__snapshot')); }
}
class WebGL2RenderingContext extends WebGLRenderingContext {}
for (const [name,value] of Object.entries({VERTEX_SHADER:0x8B31,FRAGMENT_SHADER:0x8B30,
    COMPILE_STATUS:0x8B81,LINK_STATUS:0x8B82,ACTIVE_UNIFORMS:0x8B86,ACTIVE_ATTRIBUTES:0x8B89,
    ARRAY_BUFFER:0x8892,ELEMENT_ARRAY_BUFFER:0x8893,STREAM_DRAW:0x88E0,STATIC_DRAW:0x88E4,DYNAMIC_DRAW:0x88E8,
    TEXTURE_2D:0x0DE1,TEXTURE_CUBE_MAP:0x8513,TEXTURE_CUBE_MAP_POSITIVE_X:0x8515,
    TEXTURE_CUBE_MAP_NEGATIVE_X:0x8516,TEXTURE_CUBE_MAP_POSITIVE_Y:0x8517,TEXTURE_CUBE_MAP_NEGATIVE_Y:0x8518,
    TEXTURE_CUBE_MAP_POSITIVE_Z:0x8519,TEXTURE_CUBE_MAP_NEGATIVE_Z:0x851A,TEXTURE_MIN_FILTER:0x2801,TEXTURE_MAG_FILTER:0x2800,NEAREST:0x2600,
    FLOAT:0x1406,INT:0x1404,BOOL:0x8B56,FLOAT_VEC2:0x8B50,FLOAT_VEC3:0x8B51,FLOAT_VEC4:0x8B52,
    INT_VEC2:0x8B53,INT_VEC3:0x8B54,INT_VEC4:0x8B55,BOOL_VEC2:0x8B57,BOOL_VEC3:0x8B58,BOOL_VEC4:0x8B59,
    FLOAT_MAT2:0x8B5A,FLOAT_MAT3:0x8B5B,FLOAT_MAT4:0x8B5C,SAMPLER_2D:0x8B5E,SAMPLER_CUBE:0x8B60,
    COLOR_BUFFER_BIT:0x4000,DEPTH_BUFFER_BIT:0x0100,STENCIL_BUFFER_BIT:0x0400,TRIANGLES:4,
    UNSIGNED_SHORT:0x1403,RGBA:0x1908,UNSIGNED_BYTE:0x1401,FRAMEBUFFER:0x8D40,BACK:0x0405,
    DEPTH_TEST:0x0B71,CULL_FACE:0x0B44,BLEND:0x0BE2,SCISSOR_TEST:0x0C11,POLYGON_OFFSET_FILL:0x8037,STENCIL_TEST:0x0B90,SAMPLE_ALPHA_TO_COVERAGE:0x809E,
    LEQUAL:0x0203,CW:0x0900,CCW:0x0901,FUNC_ADD:0x8006,ONE:1,SRC_ALPHA:0x0302,ONE_MINUS_SRC_ALPHA:0x0303,
    MAX_TEXTURE_SIZE:0x0D33,MAX_CUBE_MAP_TEXTURE_SIZE:0x851C,MAX_VERTEX_ATTRIBS:0x8869,
    MAX_TEXTURE_IMAGE_UNITS:0x8872,MAX_VERTEX_TEXTURE_IMAGE_UNITS:0x8B4C,MAX_VERTEX_UNIFORM_VECTORS:0x8DFB,
    MAX_VARYING_VECTORS:0x8DFC,MAX_FRAGMENT_UNIFORM_VECTORS:0x8DFD,
    VENDOR:0x1F00,RENDERER:0x1F01,VERSION:0x1F02,SHADING_LANGUAGE_VERSION:0x8B8C,
    VIEWPORT:0x0BA2,SCISSOR_BOX:0x0C10,ALIASED_POINT_SIZE_RANGE:0x846D,ALIASED_LINE_WIDTH_RANGE:0x846E,
    HIGH_FLOAT:0x8DF2,MEDIUM_FLOAT:0x8DF1,
    NO_ERROR:0,INVALID_ENUM:0x0500,INVALID_VALUE:0x0501,INVALID_OPERATION:0x0502})) {
  Object.defineProperty(WebGLRenderingContext.prototype,name,{value,enumerable:true});
  Object.defineProperty(WebGLRenderingContext,name,{value,enumerable:true});
  Object.defineProperty(WebGL2RenderingContext,name,{value,enumerable:true});
}
Object.assign(globalThis,{WebGLRenderingContext,WebGL2RenderingContext,WebGLShader,WebGLProgram,WebGLBuffer,WebGLTexture,WebGLUniformLocation,WebGLVertexArrayObject});
class HTMLTemplateElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLTemplateElement, 'template'); }}
// Blink HTMLSlotElement IDL: identity and assignment remain native DOM state.
class HTMLSlotElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLSlotElement, 'slot'); }
  get name() { return invoke(this, '__slotName'); }
  set name(value) {
    invoke(this, '__slotBrand');
    if (typeof value === 'symbol') throw new TypeError('Cannot convert a Symbol to DOMString');
    invoke(this, '__setSlotName', String(value));
  }
  assignedNodes(options = {}) {
    invoke(this, '__slotBrand');
    return Array.from(invoke(this, 'assignedNodes', slotFlattenOption(options)));
  }
  assignedElements(options = {}) {
    invoke(this, '__slotBrand');
    return Array.from(invoke(this, 'assignedElements', slotFlattenOption(options)));
  }
  assign(...nodes) { invoke(this, 'assign', ...nodes); }
}
for (const member of ['name', 'assignedNodes', 'assignedElements', 'assign']) {
  const descriptor = Object.getOwnPropertyDescriptor(HTMLSlotElement.prototype, member);
  Object.defineProperty(HTMLSlotElement.prototype, member, {...descriptor, enumerable:true});
}
function slotFlattenOption(options) {
  if (options == null) return false;
  if (typeof options !== 'object' && typeof options !== 'function')
    throw new TypeError('AssignedNodesOptions must be a dictionary');
  return !!options.flatten;
}
class HTMLHeadingElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLHeadingElement, 'h1'); }}
// cpp: third_party/blink/renderer/core/html/html_div_element.idl
class HTMLDivElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLDivElement, 'div'); }}
// cpp: third_party/blink/renderer/core/html/html_li_element.{idl,cc}
class HTMLLIElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLLIElement, 'li'); }}
class HTMLAnchorElement extends HTMLElement {
  constructor() { return constructHTMLElement(new.target, HTMLAnchorElement, 'a'); }
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
  constructor() { return constructHTMLElement(new.target, HTMLMediaElement, 'media'); }
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
class HTMLVideoElement extends HTMLMediaElement {
  constructor() { return constructHTMLElement(new.target, HTMLVideoElement, 'video'); }}
class HTMLAudioElement extends HTMLMediaElement {
  constructor() { return constructHTMLElement(new.target, HTMLAudioElement, 'audio'); }}
class Document extends Node {}
Document.prototype.createEvent = function(type) {
  String(type);
  const event = new Event(''); eventStates.get(event).initialized = false;
  event.initCustomEvent = function(name,bubbles,cancelable,detail) { this.initEvent(name,bubbles,cancelable); this.detail=detail; };
  return event;
};
const fontFaceSetToken = {};
class FontFaceSet extends EventTarget {
  constructor(token, ownerDocument) {
    if (token !== fontFaceSetToken) throw new TypeError('Illegal constructor');
    super();
    fontFaceSetDocuments.set(this, ownerDocument);
  }
  get status() {
    const owner = fontFaceSetDocuments.get(this);
    return owner ? invoke(owner, '__fontStatus') : 'loaded';
  }
  get ready() {
    const current = fontFaceSetReady.get(this);
    if (current?.loading && this.status === 'loading') return current.promise;
    if (this.status === 'loaded') {
      const ready = Promise.resolve(this);
      fontFaceSetReady.set(this, {promise:ready, loading:false});
      return ready;
    }
    const ready = new Promise(resolve => {
      const poll = () => {
        if (this.status === 'loaded') { resolve(this); return; }
        setTimeout(poll, 10);
      };
      poll();
    });
    fontFaceSetReady.set(this, {promise:ready, loading:true});
    return ready;
  }
  check(font, text = ' ') { String(font); String(text); return true; }
  load(font, text = ' ') { String(font); String(text); return Promise.resolve([]); }
}
for (const type of ['loading','loadingdone','loadingerror']) Object.defineProperty(FontFaceSet.prototype, 'on'+type, {
  configurable:true, enumerable:true,
  get() { return invoke(this, '__eventTargetHandlerGet', type); },
  set(value) { invoke(this, '__eventTargetHandlerSet', type, value); }
});
Object.defineProperty(FontFaceSet.prototype, Symbol.toStringTag,
  {value:'FontFaceSet', configurable:true});
Object.defineProperty(globalThis, 'FontFaceSet',
  {value:FontFaceSet, writable:true, configurable:true});
const documentFontSets = new WeakMap();
Object.defineProperty(Document.prototype, 'fonts', {configurable:true, enumerable:true, get() {
  let fonts = documentFontSets.get(this);
  if (!fonts) { fonts = new FontFaceSet(fontFaceSetToken, this); documentFontSets.set(this, fonts); }
  return fonts;
}});
class DocumentFragment extends Node {}
class ShadowRoot extends DocumentFragment {
  get referenceTarget() { return invoke(this,'__shadowReferenceTargetGet'); }
  set referenceTarget(value) { invoke(this,'__shadowReferenceTargetSet',value == null ? null : String(value)); }
}
function internalsInvoke(target,member,...args){const result=invoke(target,member,...args);popupMethodResult(result);return result;}
class ElementInternals {
  constructor() { throw new TypeError('Illegal constructor'); }
  setFormValue(value, state) {
    const target=internalsTarget(this); internalsInvoke(target,'__internalsBrand');
    if(!arguments.length)throw new TypeError('One argument is required');
    if(arguments.length>1)invoke(target,'__internalsSetFormValue',value,state);
    else invoke(target,'__internalsSetFormValue',value);
  }
  setValidity(flags, message='', anchor=null) {
    const target=internalsTarget(this); internalsInvoke(target,'__internalsBrand');
    if(!arguments.length)throw new TypeError('One argument is required');
    if(flags!=null&&typeof flags!=='object'&&typeof flags!=='function')throw new TypeError('ValidityStateFlags must be a dictionary');
    let bits=0;
    const names=['valueMissing','typeMismatch','patternMismatch','tooLong','tooShort','rangeUnderflow','rangeOverflow','stepMismatch','badInput','customError'];
    for(const name of [...names].sort())if(flags?.[name])bits|=1<<names.indexOf(name);
    message=attributeDOMString(message);
    if(anchor!==null&&!hasInterface(anchor,'HTMLElement'))throw new TypeError('Validation anchor requires an HTMLElement');
    internalsInvoke(target,'__internalsSetValidity',bits,message,anchor);
  }
  checkValidity() { const target=internalsTarget(this);internalsInvoke(target,'__internalsBrand');return invoke(target,'checkValidity'); }
  reportValidity() { const target=internalsTarget(this);internalsInvoke(target,'__internalsBrand');return invoke(target,'reportValidity'); }
  get form() { return internalsInvoke(internalsTarget(this),'__internalsForm'); }
  get labels() {return internalsInvoke(internalsTarget(this),'__internalsLabels');}
  get willValidate() {const target=internalsTarget(this);internalsInvoke(target,'__internalsBrand');return invoke(target,'__controlValidity','willValidate');}
  get validationMessage() {const target=internalsTarget(this);internalsInvoke(target,'__internalsBrand');return invoke(target,'__controlValidity','validationMessage');}
  get validity() {
    const target=internalsTarget(this);internalsInvoke(target,'__internalsBrand');
    let validity=controlValidities.get(target);
    if(!validity){validity=Object.create(ValidityState.prototype);validityControls.set(validity,target);controlValidities.set(target,validity);}
    return validity;
  }
  get states() {
    internalsTarget(this);
    let states=internalsStates.get(this);
    if(!states){states=Object.create(CustomStateSet.prototype);customStateTargets.set(states,internalsTarget(this));internalsStates.set(this,states);}
    return states;
  }
  get shadowRoot() {return shadowRoots.get(internalsTarget(this))||null;}
}
const internalsTargets=new WeakMap(),internalsStates=new WeakMap(),customStateTargets=new WeakMap();
const customStateIterators=new WeakMap();
function internalsTarget(internals){const target=internalsTargets.get(internals);if(!target)throw new TypeError('Illegal invocation');return target;}
function customStateTarget(states){const target=customStateTargets.get(states);if(!target)throw new TypeError('Illegal invocation');return target;}
class CustomStateSet {
  constructor(){throw new TypeError('Illegal constructor');}
  add(state){const target=customStateTarget(this);if(!arguments.length)throw new TypeError('One argument is required');invoke(target,'__internalsStateAdd',attributeDOMString(state));return undefined;}
  delete(state){
    const target=customStateTarget(this);if(!arguments.length)throw new TypeError('One argument is required');
    const index=invoke(target,'__internalsStateDelete',attributeDOMString(state));
    if(index<0)return false;
    const iterators=customStateIterators.get(this);
    if(iterators)for(const ref of iterators){const iterator=ref.deref();if(!iterator)iterators.delete(ref);else if(index<iterator.index)iterator.index--;}
    return true;
  }
  has(state){const target=customStateTarget(this);if(!arguments.length)throw new TypeError('One argument is required');return invoke(target,'__internalsStateHas',attributeDOMString(state));}
  clear(){invoke(customStateTarget(this),'__internalsStateClear');}
  get size(){let size=0;for(const _value of this)size++;return size;}
  values(){
    const target=customStateTarget(this),cursor={index:0,done:false};
    let iterators=customStateIterators.get(this);if(!iterators){iterators=new Set();customStateIterators.set(this,iterators);}
    const reference=new WeakRef(cursor);iterators.add(reference);
    return {next(){if(cursor.done)return {value:undefined,done:true};const value=invoke(target,'__internalsStateAt',cursor.index);if(value===null){cursor.done=true;iterators.delete(reference);return {value:undefined,done:true};}cursor.index++;return {value,done:false};},[Symbol.iterator](){return this;}};
  }
  keys(){return this.values();}
  *entries(){for(const value of this)yield [value,value];}
  forEach(callback,thisArg){if(typeof callback!=='function')throw new TypeError('Callback must be callable');for(const value of this)callback.call(thisArg,value,value,this);}
  [Symbol.iterator](){return this.values();}
}
Object.defineProperty(CustomStateSet.prototype,Symbol.toStringTag,{value:'CustomStateSet',configurable:true});
Object.defineProperty(globalThis,'CustomStateSet',{value:CustomStateSet,writable:true,configurable:true});
class CharacterData extends Node {}
class Text extends CharacterData {}
class Comment extends CharacterData {}
// Chromium: core/dom/cdata_section.idl and processing_instruction.idl.
// These interfaces are exposed on Window even in HTML documents.  XML-only
// construction does not make their constructors optional: libraries use the
// prototype objects while installing DOM polyfills.
class CDATASection extends Text {}
class ProcessingInstruction extends CharacterData {}
const domTokenListState = new WeakMap(), domTokenLists = new WeakMap();
class DOMTokenList {
  constructor() { throw new TypeError('Illegal constructor'); }
  get length() { return domTokenListTokens(this).length; }
  item(index) { return domTokenListTokens(this)[Number(index)] ?? null; }
  contains(token) { return domTokenListTokens(this).includes(domTokenListToken(token)); }
  add(...tokens) {
    tokens = tokens.map(domTokenListToken);
    const current = domTokenListTokens(this);
    for (const token of tokens) if (!current.includes(token)) current.push(token);
    domTokenListSave(this, current);
  }
  remove(...tokens) {
    tokens = tokens.map(domTokenListToken);
    domTokenListSave(this, domTokenListTokens(this).filter(token => !tokens.includes(token)));
  }
  toggle(token, force) {
    token = domTokenListToken(token);
    const current = domTokenListTokens(this), present = current.includes(token);
    if (force !== undefined ? !!force : !present) {
      if (!present) { current.push(token); domTokenListSave(this, current); }
      return true;
    }
    if (present) domTokenListSave(this, current.filter(value => value !== token));
    return false;
  }
  replace(token, replacement) {
    token = domTokenListToken(token); replacement = domTokenListToken(replacement);
    const current = domTokenListTokens(this), index = current.indexOf(token);
    if (index < 0) return false;
    if (token !== replacement) {
      const existing = current.indexOf(replacement);
      if (existing >= 0) current.splice(existing, 1);
      current[current.indexOf(token)] = replacement;
      domTokenListSave(this, current);
    }
    return true;
  }
  supports() { throw new TypeError('DOMTokenList has no supported tokens'); }
  get value() { const {element, attribute} = domTokenListStateFor(this); return element.getAttribute(attribute) || ''; }
  set value(value) { const {element, attribute} = domTokenListStateFor(this); element.setAttribute(attribute, String(value)); }
  toString() { return this.value; }
  entries() { return domTokenListTokens(this).entries(); }
  keys() { return domTokenListTokens(this).keys(); }
  values() { return domTokenListTokens(this).values(); }
  forEach(callback, thisArg) { let index=0; for (const token of this) callback.call(thisArg, token, index++, this); }
  [Symbol.iterator]() { return this.values(); }
}
function domTokenListStateFor(list) {
  const state = domTokenListState.get(list);
  if (!state) throw new TypeError('Illegal invocation');
  return state;
}
function domTokenListTokens(list) {
  const {element, attribute} = domTokenListStateFor(list);
  return [...new Set((element.getAttribute(attribute) || '').split(/[\t\n\f\r ]+/).filter(Boolean))];
}
function domTokenListToken(value) {
  const token = String(value);
  if (!token) throw new DOMException('The token provided must not be empty.', 'SyntaxError');
  if (/[\t\n\f\r ]/.test(token))
    throw new DOMException('The token provided contains HTML space characters.', 'InvalidCharacterError');
  return token;
}
function domTokenListSave(list, tokens) {
  const {element, attribute} = domTokenListStateFor(list);
  element.setAttribute(attribute, tokens.join(' '));
}
function domTokenList(element, attribute) {
  let lists = domTokenLists.get(element);
  if (!lists) domTokenLists.set(element, lists = new Map());
  if (!lists.has(attribute)) {
    const list = Object.create(DOMTokenList.prototype);
    domTokenListState.set(list, {element, attribute});
    lists.set(attribute, list);
  }
  return lists.get(attribute);
}
class NodeList { constructor() { throw new TypeError('Illegal constructor'); } }
class HTMLCollection { constructor() { throw new TypeError('Illegal constructor'); } }
class HTMLOptionsCollection extends HTMLCollection {}
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
    HTMLSelectElement, HTMLOptionElement, HTMLLabelElement, HTMLDetailsElement, HTMLPreElement,
    HTMLDialogElement, HTMLScriptElement, HTMLStyleElement, HTMLLinkElement,
    HTMLMetaElement, HTMLIFrameElement, HTMLImageElement, HTMLCanvasElement, WebGLRenderingContext, WebGL2RenderingContext, HTMLTemplateElement, HTMLSlotElement,
    HTMLHeadingElement, HTMLDivElement, HTMLLIElement, HTMLAnchorElement, HTMLMediaElement, HTMLVideoElement, HTMLAudioElement,
    Document, DocumentFragment, ShadowRoot, ElementInternals, CharacterData, Text, Comment,
    CDATASection, ProcessingInstruction, DOMTokenList, NodeList, HTMLCollection, HTMLOptionsCollection})) {
  Object.defineProperty(globalThis, name, {value:cls, writable:true, configurable:true});
  Object.defineProperty(cls.prototype, Symbol.toStringTag, {value:name, configurable:true});
  install(name, cls.prototype);
}
// Blink's IDLMemberInstaller installs ordinary members on interface prototypes.
// Let the engine choose own/author/native descriptors, then validate the native
// declaring-interface brand of the actual receiver. Never infer it from `instanceof`.
const attributeString = String, attributeTypeError = TypeError;
// Blink's generated bindings read an immutable native wrapper type tag in the
// same call frame. Rechrom's equivalent tag lives behind the host boundary, so
// retain successful checks by interface instead of crossing that boundary for
// every method and setter on the same wrapper. Only positive results are
// cached: author objects and proxies still take the native validation path on
// every attempted invocation, and changing a prototype never grants a brand.
const interfaceBrands = new Map();
function hasInterface(receiver, name) {
  if ((typeof receiver !== 'object' || receiver === null) && typeof receiver !== 'function') return false;
  let branded = interfaceBrands.get(name);
  if (branded?.has(receiver)) return true;
  if (!invoke(globalThis, '__domHasInterface', receiver, name)) return false;
  if (!branded) interfaceBrands.set(name, branded = new WeakSet());
  branded.add(receiver);
  return true;
}
function requireInterface(receiver, name) {
  if (!hasInterface(receiver, name)) throw new attributeTypeError('Illegal invocation');
}
function attributeDOMString(value) {
  if (typeof value === 'symbol') throw new attributeTypeError('Cannot convert a Symbol to DOMString');
  return attributeString(value);
}
const nullToEmptyString = value => value === null ? '' : attributeDOMString(value);
const nullableString = value => value == null ? '' : attributeDOMString(value);
function nativeAttribute(cls, member, convert, options = {}) {
  const interfaceName = cls.name;
  const descriptor = {configurable:true, enumerable:true,
    get: invoke(globalThis, '__domCreateGetter', interfaceName, member, !!options.lenient)};
  if (convert) descriptor.set = function(value) {
    // Web IDL checks the receiver before invoking any user conversion hooks.
    if (options.lenient && !hasInterface(this, interfaceName)) return;
    requireInterface(this, interfaceName);
    invoke(this, '__domAttributeSet', interfaceName, member, convert(value), !!options.reactions);
  };
  if (descriptor.set) Object.defineProperty(descriptor.set, 'name', {value:'set ' + member, configurable:true});
  Object.defineProperty(cls.prototype, member, descriptor);
}
for (const member of ['nodeType','nodeName','ownerDocument','parentNode','childNodes','firstChild','lastChild'])
  nativeAttribute(Node, member);
for (const member of ['nodeValue','textContent']) nativeAttribute(Node, member, nullableString, {reactions:true});
nativeAttribute(CharacterData, 'data', nullToEmptyString);
nativeAttribute(ProcessingInstruction, 'target');
Object.defineProperty(ProcessingInstruction.prototype, 'sheet', {configurable:true, enumerable:true,
  get() { requireInterface(this, 'ProcessingInstruction'); return null; }});
for (const member of ['tagName','localName','namespaceURI']) nativeAttribute(Element, member);
for (const member of ['id','className']) nativeAttribute(Element, member, attributeDOMString, {reactions:true});
for (const cls of [Element, ShadowRoot]) nativeAttribute(cls, 'innerHTML', nullToEmptyString, {reactions:true});
// The native backend still explicitly rejects unsupported outerHTML/body writes;
// retain their writable IDL descriptor shape rather than disguising them as readonly.
nativeAttribute(Element, 'outerHTML', nullToEmptyString, {reactions:true});
for (const cls of [Element, Document, DocumentFragment]) nativeAttribute(cls, 'children');
for (const member of ['activeElement','readyState','currentScript','defaultView','fullscreenEnabled',
                      'fullscreen','fullscreenElement','documentElement','scrollingElement','head'])
  nativeAttribute(Document, member);
nativeAttribute(Document, 'body', value => value, {reactions:true});
nativeAttribute(HTMLTemplateElement, 'content');
for (const member of ['complete','naturalWidth','naturalHeight','currentSrc'])
  nativeAttribute(HTMLImageElement, member);
for (const member of ['checked','indeterminate']) nativeAttribute(HTMLInputElement, member, value => !!value);
nativeAttribute(HTMLInputElement, 'value', nullToEmptyString, {reactions:true});
nativeAttribute(HTMLTextAreaElement, 'value', nullToEmptyString, {reactions:true});
function textControlUnsignedLong(value) {
  let number = Number(value);
  if (!Number.isFinite(number) || number === 0) return 0;
  number = Math.trunc(number);
  number %= 4294967296;
  return number < 0 ? number + 4294967296 : number;
}
function textControlSnapshot(control, interfaceName) {
  requireInterface(control, interfaceName);
  return invoke(control, '__textControlSelectionSnapshot');
}
for (const cls of [HTMLInputElement, HTMLTextAreaElement]) {
  Object.defineProperties(cls.prototype, {
    selectionStart: {configurable:true, enumerable:true,
      get() { return textControlSnapshot(this, cls.name)?.start ?? null; },
      set(value) {
        const snapshot = textControlSnapshot(this, cls.name);
        const start = textControlUnsignedLong(value);
        invoke(this, '__textControlSetSelectionRange', start,
               Math.max(start, snapshot?.end ?? 0), snapshot?.direction ?? 'none');
      }},
    selectionEnd: {configurable:true, enumerable:true,
      get() { return textControlSnapshot(this, cls.name)?.end ?? null; },
      set(value) {
        const snapshot = textControlSnapshot(this, cls.name);
        const end = textControlUnsignedLong(value);
        invoke(this, '__textControlSetSelectionRange', Math.min(end, snapshot?.start ?? 0),
               end, snapshot?.direction ?? 'none');
      }},
    selectionDirection: {configurable:true, enumerable:true,
      get() { return textControlSnapshot(this, cls.name)?.direction ?? null; },
      set(value) {
        const snapshot = textControlSnapshot(this, cls.name);
        const direction = value == null ? 'none' : attributeDOMString(value);
        invoke(this, '__textControlSetSelectionRange', snapshot?.start ?? 0,
               snapshot?.end ?? 0, direction);
      }}
  });
  cls.prototype.setSelectionRange = function(start, end, direction = null) {
    requireInterface(this, cls.name);
    if (arguments.length < 2)
      throw new attributeTypeError('setSelectionRange requires 2 arguments');
    invoke(this, '__textControlSetSelectionRange', textControlUnsignedLong(start),
           textControlUnsignedLong(end),
           direction == null ? 'none' : attributeDOMString(direction));
  };
  cls.prototype.select = function() {
    requireInterface(this, cls.name);
    invoke(this, '__textControlSetSelectionRange', 0, 4294967295, 'none');
  };
}
nativeAttribute(HTMLSelectElement, 'value', attributeDOMString);
nativeAttribute(HTMLSelectElement, 'options');
nativeAttribute(HTMLOptionElement, 'value', attributeDOMString, {reactions:true});
nativeAttribute(HTMLOptionElement, 'selected', value => !!value);
for (const [cls, member, attribute] of [
  [HTMLOptionElement, 'defaultSelected', 'selected'], [HTMLOptionElement, 'disabled', 'disabled'],
  [HTMLSelectElement, 'multiple', 'multiple'], [HTMLSelectElement, 'disabled', 'disabled']
]) Object.defineProperty(cls.prototype, member, {configurable:true, enumerable:true,
  get() {requireInterface(this, cls.name); return this.hasAttribute(attribute)},
  set(value) {requireInterface(this, cls.name); if (value) this.setAttribute(attribute, ''); else this.removeAttribute(attribute)}
});
for (const cls of [HTMLElement, SVGElement, MathMLElement, Document])
  for (const member of ['onload','onerror','onabort','oninput','onchange','onclick','onkeydown','onkeyup',
                        'onkeypress','onfocus','onblur','onsubmit','onreset','onmousedown','onmouseup','onmousemove'])
    nativeAttribute(cls, member, value => value);
nativeAttribute(Document, 'onreadystatechange', value => value, {lenient:true});
for (const cls of [Element, Document])
  for (const member of ['onfullscreenchange','onfullscreenerror']) nativeAttribute(cls, member, value => value);
function nativeStringMethod(cls, name, arity = 1) {
  const interfaceName = cls.name;
  // Generated Web IDL bindings convert only their declared arguments, in
  // order. These methods have one or two DOMString arguments; avoid creating
  // rest/slice/map/spread arrays on every call into the native DOM owner.
  const method = arity === 2 ? function(first, second) {
    requireInterface(this, interfaceName);
    if (arguments.length < 2) throw new attributeTypeError(name + ' requires 2 arguments');
    return invoke(this, name, attributeDOMString(first), attributeDOMString(second));
  } : function(first) {
    requireInterface(this, interfaceName);
    if (arguments.length < 1) throw new attributeTypeError(name + ' requires 1 arguments');
    return invoke(this, name, attributeDOMString(first));
  };
  Object.defineProperties(method, {name:{value:name, configurable:true}, length:{value:arity, configurable:true}});
  Object.defineProperty(cls.prototype, name, {configurable:true, enumerable:true, writable:true, value:method});
}
for (const cls of [Element, Document, DocumentFragment])
  for (const name of ['querySelector','querySelectorAll']) nativeStringMethod(cls, name);
for (const cls of [Element, Document])
  for (const name of ['getElementsByTagName','getElementsByClassName']) nativeStringMethod(cls, name);
for (const cls of [Document, DocumentFragment]) nativeStringMethod(cls, 'getElementById');
for (const name of ['matches','closest','getAttribute','hasAttribute','removeAttribute'])
  nativeStringMethod(Element, name);
nativeStringMethod(Element, 'setAttribute', 2);
function pointerCaptureResult(result) {
  if (result && result.exceptionName) throw new DOMException(result.message, result.exceptionName);
  return result;
}
for (const name of ['setPointerCapture','releasePointerCapture','hasPointerCapture']) {
  const method = function(pointerId) {
    requireInterface(this, 'Element');
    if (!arguments.length) throw new TypeError(name + ' requires 1 argument');
    const result = pointerCaptureResult(invoke(this, name, Number(pointerId) | 0));
    return name === 'hasPointerCapture' ? !!result : undefined;
  };
  Object.defineProperties(method, {name:{value:name, configurable:true}, length:{value:1, configurable:true}});
  Object.defineProperty(Element.prototype, name, {configurable:true, enumerable:true, writable:true, value:method});
}
// cpp: third_party/blink/renderer/core/dom/element.idl setAttributeNS;
//      element.cc Element::ParseAttributeName / setAttributeNS.
Object.defineProperty(Element.prototype, 'setAttributeNS', {
  configurable: true,
  enumerable: true,
  writable: true,
  value: function setAttributeNS(namespaceURI, qualifiedName, value) {
    requireInterface(this, 'Element');
    if (arguments.length < 3) throw new attributeTypeError('setAttributeNS requires 3 arguments');
    return invoke(this, 'setAttributeNS', namespaceURI === null ? null : attributeDOMString(namespaceURI),
                  attributeDOMString(qualifiedName), attributeDOMString(value));
  }
});
Node.prototype.isEqualNode = function(other) {
  requireInterface(this, 'Node');
  if (!arguments.length) throw new TypeError('isEqualNode requires 1 argument');
  if (other != null) requireInterface(other, 'Node');
  return invoke(this, 'isEqualNode', other == null ? null : other);
};
SVGGeometryElement.prototype.getTotalLength=function(){return invoke(this,'getTotalLength');};
globalThis.__domUpgradeCustomElement=function(element,constructor){
  customElementConstructionStack.push({element, constructor});
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
  const slotAssignment=init.slotAssignment===undefined?'named':String(init.slotAssignment);
  if(slotAssignment!=='named'&&slotAssignment!=='manual')throw new TypeError('Shadow root slot assignment must be named or manual');
  const rawReferenceTarget=init.referenceTarget;
  const referenceTarget=rawReferenceTarget == null ? null : String(rawReferenceTarget);
  const root=invoke(this,'attachShadow',mode,slotAssignment,!!init.delegatesFocus,referenceTarget);
  Object.setPrototypeOf(root,ShadowRoot.prototype);
  Object.defineProperties(root,{host:{value:this},mode:{value:mode},delegatesFocus:{value:!!init.delegatesFocus},slotAssignment:{value:slotAssignment}});
  shadowRoots.set(this,root); return root;
};
function popupMethodResult(result){if(result&&result.exceptionName)throw new DOMException(result.message,result.exceptionName);}
HTMLDialogElement.prototype.showModal=function(){popupMethodResult(invoke(this,'showModal'));};
HTMLDialogElement.prototype.close=function(value){popupMethodResult(arguments.length?invoke(this,'close',attributeDOMString(value)):invoke(this,'close'));};
HTMLElement.prototype.showPopover=function(options={}){const source=options?.source??null;if(source!==null&&!hasInterface(source,'Element'))throw new TypeError('source requires an Element');popupMethodResult(invoke(this,'showPopover',source));};
HTMLElement.prototype.hidePopover=function(){popupMethodResult(invoke(this,'hidePopover'));};
Element.prototype.requestFullscreen=function(options={}) {
  if(options!==null && typeof options!=='object')throw new TypeError('FullscreenOptions must be an object');
  try {
    invoke(this,'requestFullscreen');
    this.ownerDocument.dispatchEvent(new Event('fullscreenchange'));
    return Promise.resolve();
  } catch(error) {
    this.ownerDocument.dispatchEvent(new Event('fullscreenerror'));
    return Promise.reject(error);
  }
};
Document.prototype.exitFullscreen=function() {
  try {
    invoke(this,'exitFullscreen');
    this.dispatchEvent(new Event('fullscreenchange'));
    return Promise.resolve();
  } catch(error) {
    this.dispatchEvent(new Event('fullscreenerror'));
    return Promise.reject(error);
  }
};
Object.defineProperty(Element.prototype,'shadowRoot',{get(){const root=shadowRoots.get(this);return root?.mode==='open'?root:null;}});
HTMLElement.prototype.attachInternals=function() {
  requireInterface(this,'HTMLElement');internalsInvoke(this,'__internalsAttach');
  const internals=Object.create(ElementInternals.prototype);
  internalsTargets.set(internals,this);
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
  Node.prototype[name] = function(...args) {
    try {
      return invoke(this, name, ...args);
    } catch (error) {
      // ContainerNode::RemoveChild reports NotFoundError when oldChild is no
      // longer a direct child. The native mutation boundary carries the DOM
      // validation result without panicking the page thread; restore the Web
      // IDL exception class at this wrapper boundary.
      if (name === 'removeChild' && error instanceof TypeError && error.message === 'child is not in parent')
        throw new DOMException('The node to be removed is not a child of this node.', 'NotFoundError');
      throw error;
    }
  };
for (const name of ['createTextNode','createComment','createDocumentFragment'])
  Document.prototype[name] = function(...args) { return invoke(this, name, ...args.map(x => x == null ? x : String(x))); };
Document.prototype.createElement = function(name, options) {
  const is = options != null && typeof options === 'object' ? options.is : undefined;
  return invoke(this,'createElement',String(name),is === undefined ? undefined : String(is));
};
Document.prototype.createElementNS = function(ns,name,options) {
  const is = options != null && typeof options === 'object' ? options.is : undefined;
  return invoke(this,'createElementNS',ns == null ? null : String(ns),String(name),is === undefined ? undefined : String(is));
};
// Blink Document::importNode: Web IDL conversion belongs here; the native DOM
// owns cloning and destination-document identity through one typed mutation.
Document.prototype.importNode = function(node, options = false) {
  // Native identities, not JavaScript prototype chains, implement Web IDL
  // brands. A valid Node remains valid after Object.setPrototypeOf.
  const supported = invoke(this, '__validateImportNode', node);
  let deep;
  if (options === null || typeof options === 'object' || typeof options === 'function') {
    if (options?.customElementRegistry !== undefined) throw new DOMException('Explicit custom element registries are not supported', 'NotSupportedError');
    deep = !options?.selfOnly;
  } else deep = !!options;
  if (!supported)
    throw new DOMException('Document and ShadowRoot nodes cannot be imported', 'NotSupportedError');
  return invoke(this, 'importNode', node, deep);
};
Document.prototype.write = function(...text) { popupMethodResult(invoke(this, 'write', text.map(String).join(''))); };
Document.prototype.writeln = function(...text) { popupMethodResult(invoke(this, 'write', text.map(String).join('') + '\n')); };
// Blink implements Node::contains against the native DOM tree. Keep the
// ancestry walk on the host side as well: MutationObserver subtree matching
// calls this for every mutation, and walking through parentNode here would
// cross the JavaScript/host boundary once per ancestor.
Node.prototype.contains = function(other) { return invoke(this, 'contains', other); };
Node.prototype.hasChildNodes = function() { return this.firstChild !== null; };
// cpp: third_party/blink/renderer/core/dom/node.cc:1379-1394
// Normalize every descendant container, then merge adjacent Text siblings and
// remove empty Text nodes. Use the ordinary mutation surface so custom-element
// reactions and MutationObserver records retain their existing task boundary.
Node.prototype.normalize = function() {
  requireInterface(this, 'Node');
  let child = this.firstChild;
  while (child) {
    if (child.nodeType === 3) {
      let next = child.nextSibling;
      while (next?.nodeType === 3) {
        child.data += next.data;
        this.removeChild(next);
        next = child.nextSibling;
      }
      if (child.data === '') {
        next = child.nextSibling;
        this.removeChild(child);
        child = next;
      } else child = child.nextSibling;
    } else {
      child.normalize();
      child = child.nextSibling;
    }
  }
};
Node.prototype.getRootNode = function(options = {}) {
  if (options !== null && typeof options !== 'object' && typeof options !== 'function')
    throw new TypeError('GetRootNodeOptions must be an object');
  let n = this;
  while (n.parentNode) n = n.parentNode;
  if (options?.composed && n instanceof ShadowRoot)
    return n.host.getRootNode(options);
  return n;
};
Object.defineProperty(Node.prototype, 'isConnected', {get() { return this.getRootNode({composed:true}).nodeType === 9; }});
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
const validityControls = new WeakMap(), controlValidities = new WeakMap();
class ValidityState {
  constructor() { throw new TypeError('Illegal constructor'); }
}
for (const property of ['valueMissing','typeMismatch','patternMismatch','tooLong','tooShort','rangeUnderflow','rangeOverflow','stepMismatch','badInput','customError','valid'])
  Object.defineProperty(ValidityState.prototype,property,{enumerable:true,configurable:true,get(){
    const control=validityControls.get(this);
    if(!control)throw new TypeError('Illegal invocation');
    return invoke(control,'__controlValidity',property);
  }});
Object.defineProperty(ValidityState.prototype,Symbol.toStringTag,{value:'ValidityState',configurable:true});
Object.defineProperty(globalThis,'ValidityState',{value:ValidityState,writable:true,configurable:true});
for (const [prototype, expected] of [[HTMLInputElement.prototype,'input'], [HTMLTextAreaElement.prototype,'textarea'],
                         [HTMLButtonElement.prototype,'button'], [HTMLSelectElement.prototype,'select']]) {
  Object.defineProperty(prototype,'validity',{enumerable:true,configurable:true,get(){
    invoke(this,'__validityBrand',expected);
    let validity=controlValidities.get(this);
    if(!validity){validity=Object.create(ValidityState.prototype);validityControls.set(validity,this);controlValidities.set(this,validity);}
    return validity;
  }});
  for(const property of ['willValidate','validationMessage'])
    Object.defineProperty(prototype,property,{enumerable:true,configurable:true,get(){invoke(this,'__validityBrand',expected);return invoke(this,'__controlValidity',property);}});
  prototype.checkValidity=function(){invoke(this,'__validityBrand',expected);return invoke(this,'checkValidity');};
  prototype.reportValidity=function(){invoke(this,'__validityBrand',expected);return invoke(this,'reportValidity');};
  prototype.setCustomValidity=function(message){
    invoke(this,'__validityBrand',expected);
    if(arguments.length===0)throw new attributeTypeError('1 argument required');
    if(typeof message==='symbol')throw new attributeTypeError('Cannot convert a Symbol value to a string');
    invoke(this,'setCustomValidity',attributeString(message));
  };
}
for (const prototype of [HTMLInputElement.prototype, HTMLTextAreaElement.prototype,
                         HTMLButtonElement.prototype, HTMLSelectElement.prototype])
  Object.defineProperty(prototype,'form',{get(){const owner=this.getAttribute('form');return owner!==null?(this.ownerDocument.getElementById(owner) instanceof HTMLFormElement?this.ownerDocument.getElementById(owner):null):this.closest('form');}});
Object.defineProperty(HTMLFormElement.prototype,'elements',{get(){return invoke(this,'__formElements');}});
HTMLFormElement.prototype.checkValidity=function(){invoke(this,'__validityBrand','form');return invoke(this,'checkValidity');};
HTMLFormElement.prototype.reportValidity=function(){invoke(this,'__validityBrand','form');return invoke(this,'reportValidity');};
HTMLFormElement.prototype.requestSubmit=function(submitter=null){
  if(submitter!==null && !((submitter instanceof HTMLButtonElement && !['button','reset'].includes((submitter.getAttribute('type')||'').toLowerCase())) || (submitter instanceof HTMLInputElement && ['submit','image'].includes((submitter.getAttribute('type')||'').toLowerCase()))))throw new TypeError('Submitter must be a submit button');
  if(submitter!==null && submitter.form!==this)throw new DOMException('Submitter does not belong to this form','NotFoundError');
  invoke(this,'requestSubmit',submitter);
};
HTMLFormElement.prototype.submit=function(){invoke(this,'submit');};
Document.prototype.hasFocus = function() { return invoke(this, 'hasFocus'); };
// The native Window factory owns policy callbacks and immutable TrustedHTML.
// Prototype inheritance alone never supplies a Trusted Types brand.
function trustedTypesString(value) {
  if (typeof value === 'symbol') throw new attributeTypeError('Cannot convert a Symbol value to a string');
  return attributeString(value);
}
class TrustedHTML {
  constructor() { throw new attributeTypeError('Illegal constructor'); }
  toString() { return invoke(this, '__trustedTypesHTMLString'); }
  toJSON() { return invoke(this, '__trustedTypesHTMLString'); }
}
class TrustedScript {
  constructor() { throw new attributeTypeError('Illegal constructor'); }
  toString() { return invoke(this, '__trustedTypesScriptString'); }
  toJSON() { return invoke(this, '__trustedTypesScriptString'); }
}
class TrustedScriptURL {
  constructor() { throw new attributeTypeError('Illegal constructor'); }
  toString() { return invoke(this, '__trustedTypesScriptURLString'); }
  toJSON() { return invoke(this, '__trustedTypesScriptURLString'); }
}
class TrustedTypePolicy {
  constructor() { throw new attributeTypeError('Illegal constructor'); }
  get name() { return invoke(this, '__trustedTypesPolicyName'); }
  createHTML(input, ...args) {
    invoke(this, '__trustedTypesPolicyName');
    if (!arguments.length) throw new attributeTypeError('1 argument required');
    return invoke(this, '__trustedTypesCreateHTML', trustedTypesString(input), ...args);
  }
  createScript(input, ...args) {
    invoke(this, '__trustedTypesPolicyName');
    if (!arguments.length) throw new attributeTypeError('1 argument required');
    return invoke(this, '__trustedTypesCreateScript', trustedTypesString(input), ...args);
  }
  createScriptURL(input, ...args) {
    invoke(this, '__trustedTypesPolicyName');
    if (!arguments.length) throw new attributeTypeError('1 argument required');
    return invoke(this, '__trustedTypesCreateScriptURL', trustedTypesString(input), ...args);
  }
}
class TrustedTypePolicyFactory {
  constructor() { throw new attributeTypeError('Illegal constructor'); }
  createPolicy(name, options = {}) {
    invoke(this, '__trustedTypesFactoryBrand');
    if (!arguments.length) throw new attributeTypeError('1 argument required');
    name = trustedTypesString(name);
    if (options == null) options = {};
    if (typeof options !== 'object' && typeof options !== 'function') throw new attributeTypeError('Policy options require a dictionary');
    const createHTML = options.createHTML;
    const createScript = options.createScript;
    const createScriptURL = options.createScriptURL;
    if (createHTML !== undefined && typeof createHTML !== 'function') throw new attributeTypeError('createHTML is not a callback');
    if (createScript !== undefined && typeof createScript !== 'function') throw new attributeTypeError('createScript is not a callback');
    if (createScriptURL !== undefined && typeof createScriptURL !== 'function') throw new attributeTypeError('createScriptURL is not a callback');
    return invoke(this, '__trustedTypesCreatePolicy', name, createHTML, createScript, createScriptURL);
  }
  isHTML(value) { invoke(this, '__trustedTypesFactoryBrand'); if(!arguments.length)throw new attributeTypeError('1 argument required'); return invoke(globalThis, '__trustedTypesIsHTML', value); }
  isScript(value) { invoke(this, '__trustedTypesFactoryBrand'); if(!arguments.length)throw new attributeTypeError('1 argument required'); return invoke(globalThis, '__trustedTypesIsScript', value); }
  isScriptURL(value) { invoke(this, '__trustedTypesFactoryBrand'); if(!arguments.length)throw new attributeTypeError('1 argument required'); return invoke(globalThis, '__trustedTypesIsScriptURL', value); }
  get emptyHTML() { return invoke(this, '__trustedTypesEmptyHTML'); }
  get emptyScript() { return invoke(this, '__trustedTypesEmptyScript'); }
  get defaultPolicy() { return invoke(this, '__trustedTypesDefaultPolicy'); }
}
for (const [name, constructor] of [['TrustedHTML',TrustedHTML],['TrustedScript',TrustedScript],['TrustedScriptURL',TrustedScriptURL],['TrustedTypePolicy',TrustedTypePolicy],['TrustedTypePolicyFactory',TrustedTypePolicyFactory]]) {
  install(name, constructor.prototype);
  Object.defineProperty(constructor.prototype, Symbol.toStringTag, {value:name, configurable:true});
  Object.defineProperty(globalThis, name, {value:constructor, writable:true, configurable:true});
}
Object.defineProperty(globalThis, 'trustedTypes', {get() { return invoke(document, '__trustedTypesFactory'); }, enumerable:true, configurable:true});
function documentEditorCommand(document, member, command, value) {
  const result = invoke(document, member, command, value);
  if (result && result.exceptionName) throw new DOMException(result.message, result.exceptionName);
  return result;
}
Document.prototype.execCommand = function(command, showUI = false, value = null) {
  invoke(this, '__editorCommandBrand');
  if (!arguments.length) throw new attributeTypeError('1 argument required');
  command = trustedTypesString(command);
  const union = arguments.length < 3 || value == null ? null : invoke(globalThis, '__trustedTypesIsHTML', value) ? value : trustedTypesString(value);
  return documentEditorCommand(this, 'execCommand', command, union);
};
for (const member of ['queryCommandSupported','queryCommandEnabled','queryCommandState','queryCommandIndeterm','queryCommandValue']) {
  Document.prototype[member] = function(command) {
    invoke(this, '__editorCommandBrand');
    if (!arguments.length) throw new attributeTypeError('1 argument required');
    if (typeof command === 'symbol') throw new attributeTypeError('Cannot convert a Symbol value to a string');
    return documentEditorCommand(this, member, attributeString(command), '');
  };
}
for (const [property,tag] of [['scripts','script'],['images','img'],['forms','form'],['embeds','embed'],['plugins','embed']])
  Object.defineProperty(Document.prototype,property,{configurable:true,enumerable:true,get(){return this.getElementsByTagName(tag);}});
Document.prototype.getElementsByName = function(name) {
  const escaped=String(name).replace(/([\\"])/g,'\\$1');
  return this.querySelectorAll('[name="'+escaped+'"]');
};
class Range {
  constructor(document = globalThis.document) {
    const range=invoke(document,'__rangeCreate');
    if(new.target!==Range)Object.setPrototypeOf(range,new.target.prototype);
    return range;
  }
  get startContainer(){return invoke(this,'__rangeSnapshot').startContainer;}
  get startOffset(){return invoke(this,'__rangeSnapshot').startOffset;}
  get endContainer(){return invoke(this,'__rangeSnapshot').endContainer;}
  get endOffset(){return invoke(this,'__rangeSnapshot').endOffset;}
  get collapsed(){return invoke(this,'__rangeSnapshot').collapsed;}
  get commonAncestorContainer(){
    let node=this.startContainer,end=this.endContainer;
    const ancestors=new Set();for(let current=node;current;current=current.parentNode)ancestors.add(current);
    while(end&&!ancestors.has(end))end=end.parentNode;return end;
  }
  setStart(node,offset){if(!(node instanceof Node))throw new TypeError('Range node required');invoke(this,'__rangeSetStart',node,Number(offset));}
  setEnd(node,offset){if(!(node instanceof Node))throw new TypeError('Range node required');invoke(this,'__rangeSetEnd',node,Number(offset));}
  selectNode(node) { if(!(node instanceof Node))throw new TypeError('Range node required');invoke(this,'__rangeSelectNode',node); }
  selectNodeContents(node) { if(!(node instanceof Node))throw new TypeError('Range node required');invoke(this,'__rangeSelectNodeContents',node); }
  collapse(toStart=false) { invoke(this,'__rangeCollapse',!!toStart); }
  deleteContents(){invoke(this,'__rangeDeleteContents');}
  detach(){}
  cloneRange(){const clone=new Range(this.startContainer.ownerDocument||document);clone.setStart(this.startContainer,this.startOffset);clone.setEnd(this.endContainer,this.endOffset);return clone;}
  createContextualFragment(markup) {
    const start=this.startContainer;
    const document=start?.ownerDocument||globalThis.document;
    const context=start?.nodeType===1?start:(start?.parentElement||document.body||document.documentElement);
    const container=document.createElement(context?.tagName?.toLowerCase()||'body');
    container.innerHTML=String(markup);
    const fragment=document.createDocumentFragment();
    while(container.firstChild)fragment.appendChild(container.firstChild);
    return fragment;
  }
}
// InputEvent snapshots retain their actual DOM node wrappers in this realm.
// Each getTargetRanges call creates new StaticRanges from the native live
// Range owners; an already returned StaticRange never follows later mutation.
const staticRangeStates = new WeakMap();
class StaticRange {
  constructor(init) {
    if (init == null) throw new TypeError('StaticRange initializer required');
    const startContainer = init.startContainer;
    if (init.startOffset === undefined) throw new TypeError('startOffset is required');
    const startOffset = (+init.startOffset) >>> 0;
    const endContainer = init.endContainer;
    if (init.endOffset === undefined) throw new TypeError('endOffset is required');
    const endOffset = (+init.endOffset) >>> 0;
    if (!(startContainer instanceof Node) || !(endContainer instanceof Node))
      throw new TypeError('StaticRange boundaries require Nodes');
    if ([startContainer, endContainer].some(node => node.nodeType === 2 || node.nodeType === 10))
      throw new DOMException('Invalid StaticRange container', 'InvalidNodeTypeError');
    staticRangeStates.set(this, {startContainer, startOffset, endContainer, endOffset});
  }
  get startContainer() { return staticRangeState(this).startContainer; }
  get startOffset() { return staticRangeState(this).startOffset; }
  get endContainer() { return staticRangeState(this).endContainer; }
  get endOffset() { return staticRangeState(this).endOffset; }
  get collapsed() { const s=staticRangeState(this);return s.startContainer===s.endContainer && s.startOffset===s.endOffset; }
}
function staticRangeState(range) {
  const state=staticRangeStates.get(range);
  if (!state) throw new TypeError('Illegal StaticRange invocation');
  return state;
}
Object.defineProperty(StaticRange.prototype, Symbol.toStringTag, {value:'StaticRange', configurable:true});
globalThis.StaticRange=StaticRange;
class InputDeviceCapabilities {
  constructor(options = {}) {
    return invoke(globalThis,'__inputDataTransferCapabilitiesCreate',!!options?.firesTouchEvents);
  }
  get firesTouchEvents() { return invoke(this,'__inputDataTransferCapabilitiesTouch'); }
}
Object.defineProperty(InputDeviceCapabilities.prototype,Symbol.toStringTag,{value:'InputDeviceCapabilities',configurable:true});
globalThis.InputDeviceCapabilities=InputDeviceCapabilities;
install('InputDeviceCapabilities',InputDeviceCapabilities.prototype);
// A thin IDL wrapper over the native clipboard owner retained by InputEvent.
class DataTransfer {
  constructor() { return invoke(globalThis, '__inputDataTransferCreate'); }
  get dropEffect() { return invoke(this, '__inputDataTransferAttribute', 'dropEffect'); }
  set dropEffect(value) { invoke(this, '__inputDataTransferSetAttribute', 'dropEffect', String(value)); }
  get effectAllowed() { return invoke(this, '__inputDataTransferAttribute', 'effectAllowed'); }
  set effectAllowed(value) { invoke(this, '__inputDataTransferSetAttribute', 'effectAllowed', String(value)); }
  get types() {
    const cached=invoke(this,'__inputDataTransferTypesCached');
    if (cached!==undefined) return cached;
    const types=Object.freeze(Array.from(invoke(this,'__inputDataTransferAttribute','types')));
    return invoke(this,'__inputDataTransferTypesRemember',types);
  }
  get items() { return invoke(this,'__inputDataTransferAttribute','items'); }
  get files() { return invoke(this,'__inputDataTransferAttribute','files'); }
  setDragImage(image,x,y) {
    invoke(this,'__inputDataTransferBrand');
    if (arguments.length<3) throw new TypeError('Three arguments are required');
    return invoke(this,'__inputDataTransferSetDragImage',image,(+x)|0,(+y)|0);
  }
  getData(format) { return invoke(this, '__inputDataTransferGetData', String(format)); }
  setData(format, data) { invoke(this, '__inputDataTransferSetData', String(format), String(data)); }
  clearData(format) { if (format === undefined) invoke(this, '__inputDataTransferClearData'); else invoke(this, '__inputDataTransferClearData', String(format)); }
}
Object.defineProperty(DataTransfer.prototype, Symbol.toStringTag, {value:'DataTransfer', configurable:true});
globalThis.DataTransfer=DataTransfer;
install('DataTransfer', DataTransfer.prototype);
class DataTransferItemList {
  constructor() { throw new TypeError('Illegal constructor'); }
  get length() { return invoke(this,'__inputDataTransferListLength'); }
  add(data,type) {
    invoke(this,'__inputDataTransferListBrand');
    if (arguments.length===0) throw new TypeError('One argument is required');
    if (arguments.length===1) return invoke(this,'__inputDataTransferListAddFile',data);
    return invoke(this,'__inputDataTransferListAdd',String(data),String(type));
  }
  remove(index) {
    invoke(this,'__inputDataTransferListBrand');
    if (arguments.length===0) throw new TypeError('One argument is required');
    return invoke(this,'__inputDataTransferListRemove',(+index)>>>0);
  }
  clear() { return invoke(this,'__inputDataTransferListClear'); }
  *[Symbol.iterator]() { for(let i=0;i<this.length;++i) yield this[i]; }
}
class DataTransferItem {
  constructor() { throw new TypeError('Illegal constructor'); }
  get kind() { return invoke(this,'__inputDataTransferItemKind'); }
  get type() { return invoke(this,'__inputDataTransferItemType'); }
  getAsFile() { return invoke(this,'__inputDataTransferItemFile'); }
  getAsString(callback) {
    if(arguments.length===0) throw new TypeError('One argument is required');
    if(callback!=null && typeof callback!=='function') throw new TypeError('Callback must be callable');
    return invoke(this,'__inputDataTransferItemString',callback??null);
  }
}
class FileList {
  constructor() { throw new TypeError('Illegal constructor'); }
  get length() { return invoke(this,'__inputDataTransferListLength'); }
  item(index) {
    if(arguments.length===0) throw new TypeError('One argument is required');
    return invoke(this,'__inputDataTransferFileListItem',(+index)>>>0);
  }
  *[Symbol.iterator]() { for(let i=0;i<this.length;++i) yield this.item(i); }
}
for(const [name,constructor] of [['DataTransferItemList',DataTransferItemList],['DataTransferItem',DataTransferItem],['FileList',FileList]]) {
  Object.defineProperty(constructor.prototype,Symbol.toStringTag,{value:name,configurable:true});
  globalThis[name]=constructor;
  install(name,constructor.prototype);
}
class InputEvent extends UIEvent {
  constructor(type, options = {}) {
    type=String(type);
    if (options == null) options = {};
    // Returning the canonical native wrapper does not require constructing a
    // temporary JS UIEvent or copying its fields onto the real event owner.
    const bubbles=!!options.bubbles, cancelable=!!options.cancelable, composed=!!options.composed;
    const detailValue=options.detail;
    const detail=detailValue===undefined?0:(+detailValue)|0;
    const capabilitiesValue=options.sourceCapabilities;
    const sourceCapabilities=capabilitiesValue==null?null:capabilitiesValue;
    if(sourceCapabilities!==null) invoke(sourceCapabilities,'__inputDataTransferCapabilitiesBrand');
    const viewValue=options.view;
    const view=viewValue==null?null:viewValue;
    const nativeView=view===null?null:invoke(globalThis,'__inputEventConvertView',view);
    const dataValue=options.data;
    const data=dataValue==null?null:String(dataValue);
    const transferValue=options.dataTransfer;
    const dataTransfer=transferValue==null?null:transferValue;
    if (dataTransfer!==null) invoke(dataTransfer,'__inputDataTransferBrand');
    const inputTypeValue=options.inputType;
    const inputType=inputTypeValue===undefined?'':String(inputTypeValue);
    const isComposing=!!options.isComposing;
    const ranges=[];
    for (const range of options.targetRanges ?? []) {
      const s=staticRangeState(range);
      ranges.push(s.startContainer,s.startOffset,s.endContainer,s.endOffset);
    }
    return invoke(globalThis,'__inputEventCreate',type,bubbles,cancelable,composed,nativeView,detail,sourceCapabilities,inputType,data,dataTransfer,isComposing,...ranges);
  }
  get view() { return invoke(this,'__inputEventAttribute','view'); }
  get detail() { return invoke(this,'__inputEventAttribute','detail'); }
  get sourceCapabilities() { return invoke(this,'__inputEventAttribute','sourceCapabilities'); }
  get data() { return invoke(this,'__inputEventAttribute','data'); }
  get dataTransfer() { return invoke(this,'__inputEventAttribute','dataTransfer'); }
  get inputType() { return invoke(this,'__inputEventAttribute','inputType'); }
  get isComposing() { return invoke(this,'__inputEventAttribute','isComposing'); }
  getTargetRanges() { return Array.from(invoke(this,'__inputEventTargetRanges'), init => new StaticRange(init)); }
}
Object.defineProperty(InputEvent.prototype, Symbol.toStringTag, {value:'InputEvent', configurable:true});
globalThis.InputEvent=InputEvent;
install('InputEvent',InputEvent.prototype);
Object.defineProperty(Range.prototype,Symbol.toStringTag,{value:'Range',configurable:true});
install('Range',Range.prototype);
globalThis.Range=Range;
Document.prototype.createRange=function(){return new Range(this);};
const NodeFilter=Object.freeze({
  FILTER_ACCEPT:1,FILTER_REJECT:2,FILTER_SKIP:3,
  SHOW_ALL:0xffffffff,SHOW_ELEMENT:1,SHOW_ATTRIBUTE:2,SHOW_TEXT:4,
  SHOW_CDATA_SECTION:8,SHOW_ENTITY_REFERENCE:16,SHOW_ENTITY:32,
  SHOW_PROCESSING_INSTRUCTION:64,SHOW_COMMENT:128,SHOW_DOCUMENT:256,
  SHOW_DOCUMENT_TYPE:512,SHOW_DOCUMENT_FRAGMENT:1024,SHOW_NOTATION:2048
});
// cpp: NodeIteratorBase::AcceptNode. Use the native node type and its bit,
// without allocating a table or observing a shadowed JavaScript nodeType.
const nativeNodeType = Function.prototype.call.bind(Object.getOwnPropertyDescriptor(Node.prototype, 'nodeType').get);
const nodeTypeMask=node=>1 << (nativeNodeType(node) - 1);
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
const cssStyleSheetToken = {}, cssStyleSheetState = new WeakMap(), constructedStyleSheets = new Map();
class CSSStyleSheet {
  constructor(options, internalState) {
    if (options === cssStyleSheetToken) {
      cssStyleSheetState.set(this, internalState);
      return;
    }
    if (options !== undefined && options !== null && typeof options !== 'object')
      throw new TypeError('CSSStyleSheet options must be a dictionary');
    const identity = invoke(document, 'stylesheetConstruct');
    cssStyleSheetState.set(this, {document, identity, constructed:true});
    constructedStyleSheets.set(identity, this);
  }
  get ownerNode() {
    const state=cssStyleSheetState.get(this);
    return state.constructed ? null : invoke(state.document, 'stylesheetOwner', state.index);
  }
  get cssRules() {
    const state=cssStyleSheetState.get(this), rules=[];
    const prefix=state.constructed?'constructedStylesheet':'stylesheet';
    const key=state.constructed?state.identity:state.index;
    for (let i=0; i<invoke(state.document,prefix+'RuleCount',key); ++i)
      rules.push(new CSSStyleRule(invoke(state.document,prefix+'RuleText',key,i)));
    return rules;
  }
  get rules() { return this.cssRules; }
  insertRule(rule,index=0) {
    const state=cssStyleSheetState.get(this), prefix=state.constructed?'constructedStylesheet':'stylesheet';
    return invoke(state.document,prefix+'InsertRule',state.constructed?state.identity:state.index,Number(index),String(rule));
  }
  deleteRule(index) {
    const state=cssStyleSheetState.get(this), prefix=state.constructed?'constructedStylesheet':'stylesheet';
    invoke(state.document,prefix+'DeleteRule',state.constructed?state.identity:state.index,Number(index));
  }
  replaceSync(text) {
    const state=cssStyleSheetState.get(this);
    if (!state.constructed) throw new DOMException("Can't call replaceSync on non-constructed CSSStyleSheets.",'NotAllowedError');
    invoke(state.document,'stylesheetReplaceSync',state.identity,String(text));
  }
  replace(text) {
    try { this.replaceSync(text); return Promise.resolve(this); }
    catch (error) { return Promise.reject(error); }
  }
}
globalThis.CSSStyleRule = CSSStyleRule; globalThis.CSSStyleSheet = CSSStyleSheet;
function constructedStyleSheetObject(ownerDocument, identity) {
  let sheet=constructedStyleSheets.get(identity);
  if (!sheet) {
    sheet=new CSSStyleSheet(cssStyleSheetToken,{document:ownerDocument,identity,constructed:true});
    constructedStyleSheets.set(identity,sheet);
  }
  return sheet;
}
function adoptedStyleSheetsFor(scope) {
  const ownerDocument=scope instanceof Document ? scope : scope.ownerDocument;
  const result=[], count=invoke(scope,'adoptedStyleSheetCount');
  for(let i=0;i<count;i++) result.push(constructedStyleSheetObject(ownerDocument,invoke(scope,'adoptedStyleSheetIdentity',i)));
  return result;
}
function setAdoptedStyleSheets(scope, value) {
  const sheets=Array.from(value), ownerDocument=scope instanceof Document ? scope : scope.ownerDocument, identities=[];
  for(const sheet of sheets) {
    const state=cssStyleSheetState.get(sheet);
    if (!state?.constructed || state.document!==ownerDocument)
      throw new DOMException('Sharing constructed stylesheets in multiple documents is not allowed','NotAllowedError');
    identities.push(state.identity);
  }
  invoke(scope,'setAdoptedStyleSheets',...identities);
}
for(const prototype of [Document.prototype,ShadowRoot.prototype]) Object.defineProperty(prototype,'adoptedStyleSheets',{
  configurable:true,enumerable:true,get(){return adoptedStyleSheetsFor(this)},set(value){setAdoptedStyleSheets(this,value)}
});
const sheetLists = new WeakMap();
Object.defineProperty(Document.prototype, 'styleSheets', {get() {
  if(!sheetLists.has(this)) {
    const document=this, objects=[];
    const list=new Proxy({}, {get(_, key) {
      const count=invoke(document,'stylesheetCount');
      if(key==='length')return count;
      if(key===Symbol.iterator)return function*(){for(let i=0;i<count;i++)yield list[i];};
      if(key==='item')return index=>list[index]||null;
      if(/^\d+$/.test(String(key)) && Number(key)<count)return objects[key] ||= new CSSStyleSheet(cssStyleSheetToken,{index:Number(key),document,constructed:false});
    }});sheetLists.set(this,list);
  }return sheetLists.get(this);
}});
Object.defineProperty(HTMLElement.prototype,'sheet',{get(){
  if(!['STYLE','LINK'].includes(this.tagName))return undefined;
  return [...this.ownerDocument.styleSheets].find(sheet=>sheet.ownerNode===this)||null;
}});
const datasets = new WeakMap(), styles = new WeakMap();
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
    has(target, key) { if (key in target) return true;
      return typeof key === 'string' && invoke(node, 'hasStyle', cssName(key)); },
    set(target, key, value) { if (key === 'cssText') node.setAttribute('style', String(value));
      else if (typeof key === 'string' && invoke(node, 'hasStyle', cssName(key))) {
        invoke(node, 'setStyle', cssName(key), String(value ?? ''), '');
      } else target[key] = value;
      return true; }
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
Object.defineProperty(Element.prototype, 'classList', {
  get() { return domTokenList(this, 'class'); },
  set(value) { domTokenList(this, 'class').value = value; }
});
Object.defineProperty(HTMLIFrameElement.prototype, 'sandbox', {
  get() { return domTokenList(this, 'sandbox'); },
  set(value) { domTokenList(this, 'sandbox').value = value; }
});
const observers = new Set(), observerIds = new WeakMap(), observerDocuments = new WeakMap(), mutationRecordLists = new WeakMap();
function observerId(observer) {
  const id=observerIds.get(observer);
  if(id===undefined)throw new TypeError('Illegal invocation');
  return id;
}
class MutationRecord {
  constructor(values) {
    const {addedNodes,removedNodes,...attributes}=values;
    Object.assign(this,attributes);
    mutationRecordLists.set(this,{addedNodes,removedNodes});
  }
}
// RecordWithEmptyNodeLists allocates each stable empty list only when read.
for(const name of ['addedNodes','removedNodes'])Object.defineProperty(MutationRecord.prototype,name,{
  configurable:true,enumerable:true,get(){
    const lists=mutationRecordLists.get(this);
    if(!lists)throw new TypeError('Illegal invocation');
    return lists[name]??=invoke(document,'__mutationEmptyNodeList');
  }
});
class MutationObserver {
  constructor(callback){
    if(typeof callback!=='function')throw new TypeError('MutationObserver callback required');
    this._callback=callback;this._targets=new Set();
    observerIds.set(this,invoke(document,'__mutationObserverId'));
    observerDocuments.set(this,document);
  }
  observe(target,options={}) {
    const id=observerId(this);
    if(!(target instanceof Node))throw new TypeError('MutationObserver target must be a Node');
    const o={...options};
    if(o.attributes===undefined && (o.attributeOldValue!==undefined || o.attributeFilter!==undefined))o.attributes=true;
    if(o.characterData===undefined && o.characterDataOldValue!==undefined)o.characterData=true;
    if(!(o.childList||o.attributes||o.characterData) || (!o.attributes&&(o.attributeOldValue||o.attributeFilter)) || (!o.characterData&&o.characterDataOldValue))throw new TypeError('Invalid mutation observer options');
    const filter=o.attributeFilter===undefined?null:Array.from(o.attributeFilter,String);
    invoke(observerDocuments.get(this),'__mutationObserve',id,target,!!o.childList,!!o.attributes,!!o.characterData,
      !!o.subtree,!!o.attributeOldValue,!!o.characterDataOldValue,filter!==null,...(filter||[]));
    this._targets.add(target);observers.add(this);
  }
  disconnect(){
    invoke(observerDocuments.get(this),'__mutationDisconnect',observerId(this));
    this._targets.clear();observers.delete(this);
  }
  takeRecords(){return Array.from(invoke(observerDocuments.get(this),'__mutationTakeRecords',observerId(this)),values=>new MutationRecord(values));}
}
Object.assign(globalThis,{MutationObserver,MutationRecord});
wrap('__deliverMutationObserver',(id,sourceDocument)=>{
  for(const observer of observers){
    if(observerId(observer)!==id)continue;
    const records=observer.takeRecords();
    if(records.length)try{observer._callback(records,observer);}catch(error){invoke(sourceDocument,'__ceReportException',error);}
    break;
  }
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
for (const property of ['offsetWidth','offsetHeight','offsetLeft','offsetTop'])
  Object.defineProperty(HTMLElement.prototype,property,{get(){return invoke(this,'metric',property);}});
for (const property of ['clientWidth','clientHeight','clientLeft','clientTop','scrollWidth','scrollHeight'])
  Object.defineProperty(Element.prototype,property,{get(){return invoke(this,'metric',property);}});
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
  const brand = invoke(receiver, '__domCollectionBrand');
  if (brand !== expected && !(expected === 'HTMLCollection' && brand === 'HTMLOptionsCollection'))
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
const fontFaceSetDocuments = new WeakMap(), fontFaceSetReady = new WeakMap();
