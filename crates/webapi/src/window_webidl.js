(function() {
'use strict';
// Window CSSOM View, LocalDOMWindow::scrollTo/scrollBy. Native viewport state
// and writes use the same scroll owner as input and Element scrolling.
const scrollInvoke = __domInvoke, queueScrollCompletion = __queueScrollCompletion;
const ScrollPromise = Promise;
function windowScroll(receiver, args, relative) {
  try {
    if (receiver != null && receiver !== window && receiver !== globalThis)
      throw new TypeError('Illegal invocation');
    let left, top, behavior = 'auto';
    if (args.length > 1) {
      left = +args[0]; top = +args[1];
    } else {
      const options = args[0];
      if (options != null && typeof options !== 'object' && typeof options !== 'function')
        throw new TypeError('Scroll options must be an object');
      if (options != null) {
        const requestedBehavior = options.behavior;
        if (requestedBehavior !== undefined) behavior = `${requestedBehavior}`;
        if (!['auto','instant','smooth'].includes(behavior))
          throw new TypeError('Invalid scroll behavior');
        const requestedLeft = options.left;
        if (requestedLeft !== undefined) left = +requestedLeft;
        const requestedTop = options.top;
        if (requestedTop !== undefined) top = +requestedTop;
      }
    }
    const unsupported = scrollInvoke(window,'viewportScroll',relative,left,top,behavior);
    if (unsupported) throw new DOMException(unsupported,'NotSupportedError');
    // The instant source path completes its native offset change synchronously,
    // then posts ScrollPromiseResolver settlement to the DOM-manipulation queue.
    return new ScrollPromise(resolve => queueScrollCompletion(resolve));
  } catch (error) {
    // The pinned Window IDL is Promise-returning; conversion/receiver errors
    // become rejections through ExceptionToRejectPromiseScope.
    return ScrollPromise.reject(error);
  }
}
globalThis.scroll = function scroll() { return windowScroll(this,arguments,false); };
globalThis.scrollTo = function scrollTo() { return windowScroll(this,arguments,false); };
globalThis.scrollBy = function scrollBy() { return windowScroll(this,arguments,true); };
for (const [property,axis] of [['scrollX','x'],['pageXOffset','x'],['scrollY','y'],['pageYOffset','y']]) {
  Object.defineProperty(globalThis,property,{
    configurable:true, enumerable:true,
    get() {
      if (this != null && this !== window && this !== globalThis) throw new TypeError('Illegal invocation');
      return scrollInvoke(window,'viewportScrollOffset',axis);
    },
    // Window IDL marks these readonly attributes Replaceable.
    set(value) {
      if (this != null && this !== window && this !== globalThis) throw new TypeError('Illegal invocation');
      Object.defineProperty(this == null ? globalThis : this,property,{value,writable:true,enumerable:true,configurable:true});
    }
  });
}

const request = __request;
const cancelRequest = __cancelRequest;
const randomHex = __randomHex;
const queueTask = __queueTask;

class MessageEvent extends Event {
  constructor(type, options = {}) {
    super(type, options);
    this.data = options.data === undefined ? null : options.data;
    this.origin = options.origin === undefined ? '' : String(options.origin);
    this.lastEventId = options.lastEventId === undefined ? '' : String(options.lastEventId);
    this.source = options.source === undefined ? null : options.source;
    this.ports = options.ports === undefined ? [] : Array.from(options.ports);
  }
}
globalThis.MessageEvent = MessageEvent;

class MessagePort extends EventTarget {
  constructor(token) {
    super();
    if (token !== messagePortToken) throw new TypeError('Illegal constructor');
    this._peer = null;
    this._started = false;
    this._closed = false;
    this._pending = [];
    this._onmessage = null;
    this._onmessageerror = null;
  }
  postMessage(message, transfer = []) {
    if (this._closed || !this._peer || this._peer._closed) return;
    const transfers = transfer == null ? [] : Array.from(transfer);
    if (transfers.length) throw new DOMException('Transferable objects are not supported', 'DataCloneError');
    const data = structuredClone(message);
    const target = this._peer;
    queueTask(() => {
      if (target._closed) return;
      if (!target._started) { target._pending.push(data); return; }
      target.dispatchEvent(new MessageEvent('message', {data}));
    });
  }
  start() {
    if (this._closed || this._started) return;
    this._started = true;
    const pending = this._pending.splice(0);
    for (const data of pending)
      queueTask(() => { if (!this._closed) this.dispatchEvent(new MessageEvent('message', {data})); });
  }
  close() { this._closed = true; this._pending.length = 0; }
  get onmessage() { return this._onmessage; }
  set onmessage(callback) {
    this._onmessage = typeof callback === 'function' ? callback : null;
    if (this._onmessage) this.start();
  }
  get onmessageerror() { return this._onmessageerror; }
  set onmessageerror(callback) { this._onmessageerror = typeof callback === 'function' ? callback : null; }
}
const messagePortToken = {};
globalThis.MessagePort = MessagePort;

class MessageChannel {
  constructor() {
    this.port1 = new MessagePort(messagePortToken);
    this.port2 = new MessagePort(messagePortToken);
    this.port1._peer = this.port2;
    this.port2._peer = this.port1;
  }
}
globalThis.MessageChannel = MessageChannel;

let nextIdleCallback = 1;
const idleCallbacks = new Map();
globalThis.requestIdleCallback = function(callback, options = {}) {
  if (typeof callback !== 'function')
    throw new TypeError('requestIdleCallback requires a function');
  const id = nextIdleCallback++;
  const began = performance.now();
  const timeout = options && Number.isFinite(Number(options.timeout))
      ? Math.max(0, Number(options.timeout)) : 1;
  const timer = setTimeout(() => {
    if (!idleCallbacks.delete(id)) return;
    const deadline = {
      didTimeout: timeout === 0,
      timeRemaining() {
        return Math.max(0, 50 - (performance.now() - began));
      }
    };
    callback(deadline);
  }, Math.min(timeout, 1));
  idleCallbacks.set(id, timer);
  return id;
};
globalThis.cancelIdleCallback = function(id) {
  const timer = idleCallbacks.get(Number(id));
  if (timer === undefined) return;
  idleCallbacks.delete(Number(id));
  clearTimeout(timer);
};
const timeline = [];
const performanceObservers = new Set();
let performanceDeliveryQueued = false;
class PerformanceObserverEntryList {
  constructor(entries) { this._entries = entries; }
  getEntries() { return this._entries.slice(); }
  getEntriesByType(type) { return this._entries.filter(entry => entry.entryType === String(type)); }
  getEntriesByName(name, type) { return this._entries.filter(entry => entry.name === String(name) && (type === undefined || entry.entryType === String(type))); }
}
class PerformanceObserver {
  constructor(callback) {
    if (typeof callback !== 'function') throw new TypeError('PerformanceObserver callback must be a function');
    this._callback = callback; this._types = new Set(); this._records = [];
  }
  observe(options = {}) {
    const types = options.entryTypes !== undefined ? Array.from(options.entryTypes, String) :
      options.type !== undefined ? [String(options.type)] : [];
    if (!types.length) throw new TypeError('PerformanceObserver requires type or entryTypes');
    this._types = new Set(types); performanceObservers.add(this);
    if (options.buffered) {
      this._records.push(...timeline.filter(entry => this._types.has(entry.entryType)));
      queuePerformanceDelivery();
    }
  }
  disconnect() { performanceObservers.delete(this); this._records = []; this._types.clear(); }
  takeRecords() { const records = this._records; this._records = []; return records; }
}
PerformanceObserver.supportedEntryTypes = Object.freeze(['mark','measure','resource','navigation','paint']);
globalThis.PerformanceObserver = PerformanceObserver;
globalThis.PerformanceObserverEntryList = PerformanceObserverEntryList;
function queuePerformanceDelivery() {
  if (performanceDeliveryQueued) return;
  performanceDeliveryQueued = true;
  queueMicrotask(() => {
    performanceDeliveryQueued = false;
    for (const observer of [...performanceObservers]) {
      const records = observer.takeRecords();
      if (records.length) observer._callback(new PerformanceObserverEntryList(records), observer);
    }
  });
}
function addPerformanceEntry(entry) {
  timeline.push(entry);
  for (const observer of performanceObservers)
    if (observer._types.has(entry.entryType)) observer._records.push(entry);
  queuePerformanceDelivery();
  return entry;
}
performance.getEntries = () => timeline.slice();
performance.getEntriesByType = type => timeline.filter(entry => entry.entryType === String(type));
performance.getEntriesByName = (name,type) => timeline.filter(entry => entry.name === String(name) && (type === undefined || entry.entryType === type));
performance.mark = name => addPerformanceEntry({name:String(name), entryType:'mark', startTime:performance.now(), duration:0});
performance.measure = (name,start,end) => {
  const resolve = value => typeof value === 'number' ? value : timeline.findLast(e=>e.name===value&&e.entryType==='mark')?.startTime;
  let a=0,b=performance.now();
  if (start && typeof start === 'object') {a=start.start===undefined?0:resolve(start.start);b=start.end===undefined?b:resolve(start.end);if(start.duration!==undefined)b=a+Number(start.duration);}
  else {if(start!==undefined)a=resolve(start);if(end!==undefined)b=resolve(end);}
  if(a===undefined||b===undefined)throw new DOMException('Mark not found','SyntaxError');
  return addPerformanceEntry({name:String(name),entryType:'measure',startTime:a,duration:b-a});
};
performance.clearMeasures = name => {for(let i=timeline.length-1;i>=0;i--)if(timeline[i].entryType==='measure'&&(name===undefined||timeline[i].name===name))timeline.splice(i,1);};
performance.clearMarks = name => { for (let i=timeline.length-1;i>=0;i--) if (timeline[i].entryType === 'mark' && (name === undefined || timeline[i].name === name)) timeline.splice(i,1); };
performance.clearResourceTimings = () => { for (let i=timeline.length-1;i>=0;i--) if (timeline[i].entryType === 'resource') timeline.splice(i,1); };
globalThis.CSS = {
  supports(property,value) {
    if (arguments.length === 1) { const match = String(property).match(/^\s*\(?\s*([\w-]+)\s*:\s*(.*?)\s*\)?\s*$/);
      if (!match) return false; property = match[1]; value = match[2]; }
    return __domInvoke(window, 'cssSupports', String(property), String(value));
  },
  escape(value) {
    const text = String(value); let result = '';
    for (let i=0;i<text.length;i++) { const code = text.charCodeAt(i), c=text[i];
      if (code === 0) result += '\ufffd';
      else if (code <= 31 || code === 127 || (i===0 && code>=48 && code<=57) ||
               (i===1 && code>=48 && code<=57 && text[0]==='-')) result += '\\' + code.toString(16) + ' ';
      else if (i===0 && c==='-' && text.length===1) result += '\\-';
      else if (code>=128 || /[a-zA-Z0-9_-]/.test(c)) result += c;
      else result += '\\'+c;
    } return result;
  }
};
class MediaQueryList extends EventTarget {
  constructor(media) { super(); this.media=String(media); this.onchange=null; }
  get matches() {
    return __mediaQueryMatches(this.media);
  }
  addListener(callback) { this.addEventListener('change',callback); }
  removeListener(callback) { this.removeEventListener('change',callback); }
}
globalThis.MediaQueryList=MediaQueryList;
globalThis.matchMedia=query=>new MediaQueryList(query);
const historyEntries = [{state:null, url:location.href}]; let historyIndex = 0;
globalThis.history = {
  get state() { return historyEntries[historyIndex].state; },
  get length() { return historyEntries.length; }, scrollRestoration:'auto',
  replaceState(state, unused, url) {
    if (url != null && String(url) !== '') __historyUpdate(String(url));
    historyEntries[historyIndex] = {state:structuredClone(state), url:location.href};
  },
  pushState(state, unused, url) {
    if (url != null && String(url) !== '') __historyUpdate(String(url));
    historyEntries.splice(++historyIndex); historyEntries.push({state:structuredClone(state), url:location.href});
  }
};
class TextEncoder {
  get encoding() { return 'utf-8'; }
  encode(value = '') {
    const bytes = [];
    for (const char of String(value)) {
      let cp = char.codePointAt(0); if (cp >= 0xd800 && cp <= 0xdfff) cp = 0xfffd;
      if (cp <= 0x7f) bytes.push(cp);
      else if (cp <= 0x7ff) bytes.push(0xc0 | cp >> 6, 0x80 | cp & 63);
      else if (cp <= 0xffff) bytes.push(0xe0 | cp >> 12, 0x80 | cp >> 6 & 63, 0x80 | cp & 63);
      else bytes.push(0xf0 | cp >> 18, 0x80 | cp >> 12 & 63, 0x80 | cp >> 6 & 63, 0x80 | cp & 63);
    } return new Uint8Array(bytes);
  }
  encodeInto(value, destination) {
    let read = 0, written = 0;
    for (const char of String(value)) { const bytes = this.encode(char); if (written + bytes.length > destination.length) break;
      destination.set(bytes,written); written += bytes.length; read += char.length; }
    return {read, written};
  }
}
class TextDecoder {
  constructor(label = 'utf-8', options = {}) {
    if (!['utf-8','utf8','unicode-1-1-utf-8'].includes(String(label).trim().toLowerCase())) throw new RangeError('Unsupported encoding');
    this.encoding = 'utf-8'; this.fatal = !!options.fatal; this.ignoreBOM = !!options.ignoreBOM;
    this._pending = []; this._bomSeen = false;
  }
  decode(input = new Uint8Array(), options = {}) {
    const bytes = [...this._pending, ...new Uint8Array(input.buffer || input, input.byteOffset || 0, input.byteLength)];
    this._pending = []; let result = '', i = 0;
    while (i < bytes.length) {
      const first = bytes[i]; let count = first <= 127 ? 1 : first >= 194 && first <= 223 ? 2 : first >= 224 && first <= 239 ? 3 : first >= 240 && first <= 244 ? 4 : 0;
      let cp = count === 1 ? first : first & (0x7f >> count), used = 1, valid = count > 0;
      for (let n=1;valid && n<count;n++) {
        if (i+n >= bytes.length) { if (options.stream) { this._pending = bytes.slice(i); return result; } valid=false; break; }
        const b = bytes[i+n];
        if (b < 128 || b > 191 || (n===1 && ((first===224 && b<160) || (first===237 && b>159) || (first===240 && b<144) || (first===244 && b>143)))) {valid=false;break;}
        cp = cp << 6 | b & 63; ++used;
      }
      if (!valid) { if (this.fatal) throw new TypeError('Invalid UTF-8 sequence'); cp = 0xfffd; }
      i += used;
      if (!this._bomSeen) { this._bomSeen = true; if (cp === 0xfeff && !this.ignoreBOM) continue; }
      result += String.fromCodePoint(cp);
    }
    if (!options.stream) this._bomSeen = false;
    return result;
  }
}
globalThis.TextEncoder = TextEncoder; globalThis.TextDecoder = TextDecoder;
function blobPartBytes(part) {
  if (part instanceof Blob) return part._bytes;
  if (part instanceof ArrayBuffer) return new Uint8Array(part);
  if (ArrayBuffer.isView(part))
    return new Uint8Array(part.buffer, part.byteOffset, part.byteLength);
  return new TextEncoder().encode(String(part));
}
class Blob {
  constructor(parts = [], options = {}) {
    const chunks = Array.from(parts, blobPartBytes);
    const size = chunks.reduce((total, chunk) => total + chunk.byteLength, 0);
    this._bytes = new Uint8Array(size);
    let offset = 0;
    for (const chunk of chunks) { this._bytes.set(chunk, offset); offset += chunk.byteLength; }
    const type = String(options.type || '').toLowerCase();
    this.type = /^[\x20-\x7e]*$/.test(type) ? type : '';
  }
  get size() { return this._bytes.byteLength; }
  slice(start = 0, end = this.size, type = '') {
    const normalize = value => value < 0 ? Math.max(this.size + value, 0) : Math.min(value, this.size);
    const from = normalize(Number(start) || 0), to = Math.max(from, normalize(Number(end)));
    return new Blob([this._bytes.slice(from, to)], {type});
  }
  async arrayBuffer() {
    return this._bytes.buffer.slice(this._bytes.byteOffset,
                                    this._bytes.byteOffset + this._bytes.byteLength);
  }
  async text() { return new TextDecoder().decode(this._bytes); }
  stream() { throw new DOMException('Blob streams are not implemented', 'NotSupportedError'); }
}
class File extends Blob {
  constructor(parts, name, options = {}) {
    super(parts, options); this.name = String(name);
    this.lastModified = options.lastModified === undefined ? Date.now() : Number(options.lastModified);
  }
}
globalThis.Blob = Blob; globalThis.File = File;
const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
globalThis.btoa = value => {
  const text = String(value); let bits=0, count=0, result='';
  for (let i=0;i<text.length;i++) { const byte=text.charCodeAt(i); if (byte>255) throw new DOMException('Invalid character', 'InvalidCharacterError');
    bits = bits << 8 | byte; count += 8; while (count >= 6) { count -= 6; result += alphabet[bits >> count & 63]; } }
  if (count) result += alphabet[bits << (6-count) & 63];
  while (result.length % 4) result += '='; return result;
};
globalThis.atob = value => {
  let text = String(value).replace(/[\t\n\f\r ]/g,'');
  if (text.length % 4 === 0) text = text.replace(/={1,2}$/,'');
  if (text.length % 4 === 1 || /[^A-Za-z0-9+/]/.test(text)) throw new DOMException('Invalid character','InvalidCharacterError');
  let bits=0,count=0,result=''; for (const char of text) {bits=bits<<6|alphabet.indexOf(char);count+=6;
    if (count>=8) {count-=8;result+=String.fromCharCode(bits>>count&255);} } return result;
};
// URLSearchParams takes USVString input before native percent/UTF-8 decoding.
// Capture the intrinsic conversion and decoder: author codecs/prototypes must
// not change parsing, and UTF-8 decode without BOM handling preserves U+FEFF.
const formScalarString = Function.prototype.call.bind(String.prototype.toWellFormed);
const nativeDecodeForm = __urlDecodeForm;
const decodeForm = text => nativeDecodeForm(formScalarString(text));
const encodeForm = text => encodeURIComponent(String(text)).replace(/[!'()~]/g,c=>'%'+c.charCodeAt(0).toString(16).toUpperCase()).replace(/%20/g,'+');
class URLSearchParams {
  constructor(init = '') {
    this._pairs=[];
    if (typeof init === 'string') {
      for (const part of init.replace(/^\?/,'').split('&')) if (part) {const eq=part.indexOf('=');
        this._pairs.push([decodeForm(eq<0?part:part.slice(0,eq)),decodeForm(eq<0?'':part.slice(eq+1))]);}
    } else if (init[Symbol.iterator]) {
      for (const pair of init) {const values=[...pair];if(values.length!==2)throw new TypeError('Expected pair');this._pairs.push(values.map(String));}
    } else for (const key of Object.keys(init)) this._pairs.push([key,String(init[key])]);
  }
  get size() { return this._pairs.length; }
  append(name,value) {this._pairs.push([String(name),String(value)]);this._changed?.();}
  delete(name,value) {this._pairs=this._pairs.filter(p=>p[0]!==String(name)||(arguments.length>1&&p[1]!==String(value)));this._changed?.();}
  get(name) {return this._pairs.find(p=>p[0]===String(name))?.[1]??null;}
  getAll(name) {return this._pairs.filter(p=>p[0]===String(name)).map(p=>p[1]);}
  has(name,value) {return this._pairs.some(p=>p[0]===String(name)&&(arguments.length<2||p[1]===String(value)));}
  set(name,value) {name=String(name);value=String(value);const first=this._pairs.findIndex(p=>p[0]===name);
    if(first<0)this._pairs.push([name,value]);else this._pairs=this._pairs.filter((p,i)=>p[0]!==name||i===first).map(p=>p[0]===name?[name,value]:p);this._changed?.();}
  sort() {this._pairs.sort((a,b)=>a[0]<b[0]?-1:a[0]>b[0]?1:0);this._changed?.();}
  *entries(){for(let i=0;i<this._pairs.length;i++)yield [...this._pairs[i]];}
  *keys(){for(const p of this)yield p[0];} *values(){for(const p of this)yield p[1];}
  [Symbol.iterator](){return this.entries();}
  forEach(callback,thisArg){for(const [k,v] of this)callback.call(thisArg,v,k,this);}
  toString(){return this._pairs.map(([k,v])=>encodeForm(k)+'='+encodeForm(v)).join('&');}
}
globalThis.URLSearchParams=URLSearchParams;
// The URL store is owned by this Window realm and released with the realm.
const objectURLs = new Map();
class URL {
  constructor(input,base) {this._parts=__urlParts(String(input),base===undefined?String(input):String(base));this._params=new URLSearchParams(this._parts.search);
    this._params._changed=()=>{this.search=this._params.toString();};}
  get href(){return this._parts.href;} set href(value){this._parts=__urlParts(String(value));this._params._pairs=new URLSearchParams(this._parts.search)._pairs;}
  get origin(){return this._parts.origin;} get searchParams(){return this._params;}
  toString(){return this.href;} toJSON(){return this.href;}
  static createObjectURL(blob) {
    if (!(blob instanceof Blob)) throw new TypeError('createObjectURL requires a Blob');
    const url = 'blob:' + location.origin + '/' + crypto.randomUUID();
    objectURLs.set(url, new Blob([blob._bytes], {type:blob.type}));
    return url;
  }
  static revokeObjectURL(url) { objectURLs.delete(String(url)); }
}
for(const key of ['protocol','host','hostname','port','pathname','username','password','search','hash'])
  Object.defineProperty(URL.prototype,key,{get(){return this._parts[key];},set(value){
    const p={};for(const k of ['protocol','host','hostname','port','pathname','username','password','search','hash'])p[k]=this._parts[k];
    p[key]=String(value);if(key==='hostname'||key==='port')p.host=p.hostname+(p.port?':'+p.port:'');
    if(key==='search'&&p.search&&!p.search.startsWith('?'))p.search='?'+p.search;
    if(key==='hash'&&p.hash&&!p.hash.startsWith('#'))p.hash='#'+p.hash;
    this.href=p.protocol+'//'+(p.username?p.username+(p.password?':'+p.password:'')+'@':'')+p.host+p.pathname+p.search+p.hash;
  }});
globalThis.URL=URL;
Document.prototype.createEvent = function(type) { const event=new Event(''); event.initEvent=function(name,bubbles,cancelable){this.type=String(name);this.bubbles=!!bubbles;this.cancelable=!!cancelable;}; event.initCustomEvent=function(name,bubbles,cancelable,detail){this.initEvent(name,bubbles,cancelable);this.detail=detail;};return event; };
class DOMException extends Error {
  constructor(message = '', name = 'Error') { super(message); this.name = name; }
}
globalThis.DOMException = DOMException;
globalThis.crypto = {
  getRandomValues(array) {
    if (!ArrayBuffer.isView(array) || array instanceof DataView ||
        /^Float/.test(array.constructor.name) || array.byteLength > 65536)
      throw new DOMException('Invalid random value buffer','QuotaExceededError');
    const hex=randomHex(array.byteLength);
    const bytes=new Uint8Array(array.buffer,array.byteOffset,array.byteLength);
    for(let index=0;index<bytes.length;++index)bytes[index]=parseInt(hex.slice(index*2,index*2+2),16);
    return array;
  },
  randomUUID() {
    const bytes=this.getRandomValues(new Uint8Array(16));
    bytes[6]=bytes[6]&15|64; bytes[8]=bytes[8]&63|128;
    const hex=[...bytes].map(value=>value.toString(16).padStart(2,'0')).join('');
    return hex.slice(0,8)+'-'+hex.slice(8,12)+'-'+hex.slice(12,16)+'-'+hex.slice(16,20)+'-'+hex.slice(20);
  }
};
class AbortSignal extends EventTarget {
  #aborted = false; #reason;
  get aborted() { return this.#aborted; }
  get reason() { return this.#reason; }
  _abort(reason) { if (this.#aborted) return; this.#aborted = true;
    this.#reason = reason === undefined ? new DOMException('The operation was aborted.', 'AbortError') : reason;
    this.dispatchEvent(new Event('abort')); }
  throwIfAborted() { if (this.#aborted) throw this.#reason; }
  static abort(reason) { const signal = new AbortSignal(); signal._abort(reason); return signal; }
  static timeout(ms) { const signal = new AbortSignal(); setTimeout(() => signal._abort(new DOMException('The operation timed out.', 'TimeoutError')), ms); return signal; }
  static any(signals) { const result = new AbortSignal(); for (const signal of signals) {
    if (signal.aborted) { result._abort(signal.reason); break; }
    signal.addEventListener('abort', () => result._abort(signal.reason), {once:true});
  } return result; }
}
class AbortController { constructor() { this.signal = new AbortSignal(); } abort(reason) { this.signal._abort(reason); } }
globalThis.AbortSignal = AbortSignal; globalThis.AbortController = AbortController;
class CustomEvent extends Event { constructor(type, options = {}) { super(type, options); this.detail = options.detail ?? null; } }
globalThis.CustomEvent = CustomEvent;
class Headers {
  constructor(init = undefined) {
    this._values = new Map();
    if (init instanceof Headers) for (const [name,value] of init) this.append(name,value);
    else if (init != null && typeof init[Symbol.iterator] === 'function')
      for (const pair of init) { if (!pair || pair.length !== 2) throw new TypeError('Invalid header pair'); this.append(pair[0],pair[1]); }
    else if (init != null) for (const [name,value] of Object.entries(init)) this.append(name,value);
  }
  _name(name) { name=String(name).trim().toLowerCase(); if (!name || !/^[!#$%&'*+.^_`|~0-9a-z-]+$/.test(name)) throw new TypeError('Invalid header name'); return name; }
  append(name,value) { name=this._name(name); value=String(value).trim(); this._values.set(name,this._values.has(name)?this._values.get(name)+', '+value:value); }
  set(name,value) { this._values.set(this._name(name),String(value).trim()); }
  get(name) { return this._values.get(this._name(name)) ?? null; }
  has(name) { return this._values.has(this._name(name)); }
  delete(name) { this._values.delete(this._name(name)); }
  entries() { return this._values.entries(); }
  keys() { return this._values.keys(); }
  values() { return this._values.values(); }
  forEach(callback,thisArg) { for (const [name,value] of this) callback.call(thisArg,value,name,this); }
  [Symbol.iterator]() { return this.entries(); }
}
class Body {
  _initBody(body) { this._body=body == null ? '' : body instanceof Blob ? body : String(body); this.bodyUsed=false; }
  _consume() { if(this.bodyUsed)return Promise.reject(new TypeError('Body already used'));this.bodyUsed=true;return Promise.resolve(this._body); }
  text() { return this._consume().then(body=>body instanceof Blob?body.text():String(body)); }
  json() { return this.text().then(JSON.parse); }
  blob() { return this._consume().then(body=>body instanceof Blob?body:new Blob([body],{type:this.headers?.get('content-type')||''})); }
  arrayBuffer() { return this.blob().then(blob=>blob.arrayBuffer()); }
}
class Request extends Body {
  constructor(input, init = {}) {
    super(); const source=input instanceof Request?input:null;
    const address=source?source.url:String(input);
    this.url=/^[a-zA-Z][a-zA-Z0-9+.-]*:/.test(address)?address:new URL(address,location.href).href;
    this.method=String(init.method??source?.method??'GET').toUpperCase();
    this.headers=new Headers(init.headers??source?.headers);
    this.signal=init.signal??source?.signal??new AbortController().signal;
    this.credentials=init.credentials??source?.credentials??'same-origin';
    this.mode=init.mode??source?.mode??'cors'; this.cache=init.cache??source?.cache??'default';
    this.redirect=init.redirect??source?.redirect??'follow'; this.referrer=init.referrer??source?.referrer??'about:client';
    this._initBody(init.body??(source?source._body:null));
  }
  clone() { if(this.bodyUsed)throw new TypeError('Body already used');return new Request(this); }
}
class Response extends Body {
  constructor(body = null, init = {}) {
    super(); this.status=init.status===undefined?200:Number(init.status); this.statusText=String(init.statusText||'');
    this.headers=new Headers(init.headers); this.url=String(init.url||''); this.redirected=!!init.redirected; this.type=init.type||'basic';
    this._initBody(body);
  }
  get ok() { return this.status>=200&&this.status<=299; }
  clone() { if(this.bodyUsed)throw new TypeError('Body already used');return new Response(this._body,{status:this.status,statusText:this.statusText,headers:this.headers,url:this.url,redirected:this.redirected,type:this.type}); }
  static error() { return new Response(null,{status:0,type:'error'}); }
  static redirect(url,status=302) { return new Response(null,{status,headers:{location:new URL(url,location.href).href}}); }
  static json(value,init={}) { const headers=new Headers(init.headers);if(!headers.has('content-type'))headers.set('content-type','application/json');return new Response(JSON.stringify(value),{...init,headers}); }
}
globalThis.Headers=Headers; globalThis.Request=Request; globalThis.Response=Response;
globalThis.fetch=function(input,init={}) {
  let resource; try { resource=new Request(input,init); } catch(error) { return Promise.reject(error); }
  if(resource.signal?.aborted)return Promise.reject(resource.signal.reason??new DOMException('Aborted','AbortError'));
  if (resource.url.startsWith('blob:')) {
    const blob = objectURLs.get(resource.url);
    if (!blob || resource.method !== 'GET') return Promise.reject(new TypeError('Failed to fetch'));
    return Promise.resolve(new Response(blob, {status:200,statusText:'OK',url:resource.url,headers:{'content-type':blob.type,'content-length':String(blob.size)}}));
  }
  return new Promise((resolve,reject)=>{
    let settled=false, id=0;
    const abort=()=>{if(settled)return;settled=true;if(id)cancelRequest(id);reject(resource.signal.reason??new DOMException('Aborted','AbortError'));};
    resource.signal?.addEventListener('abort',abort,{once:true});
    id=request(resource.method,resource.url,resource._body instanceof Blob?'':String(resource._body||''),
      [...resource.headers].map(([name,value])=>name+':'+value).join('\n'),
      (status,bytes,url,mime)=>{if(settled)return;settled=true;const headers=new Headers();if(mime)headers.set('content-type',mime);resolve(new Response(new Blob([bytes],{type:mime}),{status,url,headers}));},
      ()=>{if(settled)return;settled=true;reject(new TypeError('Failed to fetch'));});
  });
};
class XMLHttpRequest extends EventTarget {
  constructor() {
    super();
    this.readyState = 0; this.responseText = ''; this.response = ''; this.responseType = '';
    this.status = 0; this.statusText = ''; this.responseURL = ''; this.timeout = 0;
    this.withCredentials = false; this._generation = 0; this._headers = {};
  }
  _fire(type) { this.dispatchEvent(new Event(type)); }
  open(method, url, async = true) {
    if (!async) throw new Error('Synchronous XMLHttpRequest is unsupported');
    this._cancel();
    this._generation++; this._method = String(method).toUpperCase(); this._url = String(url);
    this.readyState = 1; this._sent = false; this._fire('readystatechange');
  }
  setRequestHeader(name, value) { if (/[\r\n]/.test(String(name)+String(value))) throw new TypeError('Invalid header'); this._headers[String(name).toLowerCase()] = String(value); }
  getResponseHeader(name) { return String(name).toLowerCase() === 'content-type' ? this._mime || null : null; }
  getAllResponseHeaders() { return this._mime ? 'content-type: ' + this._mime + '\r\n' : ''; }
  overrideMimeType(value) { this._overrideMime = String(value); }
  get timeout() { return this._timeout || 0; }
  set timeout(value) {
    this._timeout = Number(value) >>> 0;
    if (this._sent) this._armTimeout();
  }
  _armTimeout() {
    clearTimeout(this._timeoutId);
    if (!this.timeout) return;
    const generation = this._generation;
    this._timeoutId = setTimeout(() => {
      if (generation === this._generation && this._sent) this._fail('timeout');
    }, Math.max(0, this.timeout - (performance.now() - this._started)));
  }
  _cancel() { if (this._requestId) cancelRequest(this._requestId); this._requestId = 0; clearTimeout(this._timeoutId); }
  _fail(type) {
    this._cancel(); this._generation++; this.status = 0;
    this.responseText = ''; this.response = ''; this.responseURL = '';
    this.readyState = 4; this._sent = false;
    this._fire('readystatechange'); this._fire(type); this._fire('loadend');
  }
  abort() {
    if (this._sent && this.readyState !== 4) this._fail('abort');
    else { this._cancel(); this._generation++; }
    this.readyState = 0;
  }
  send(body = null) {
    if (this.readyState !== 1 || this._sent) throw new Error('InvalidStateError');
    this._sent = true; this._fire('loadstart'); const generation = this._generation;
    const started = this._started = performance.now();
    this._requestId = request(this._method, this._url, body == null ? '' : String(body), Object.entries(this._headers).map(([k,v])=>k+':'+v).join('\n'), (status, bytes, url, mime) => {
      if (generation !== this._generation) return;
      timeline.push({name:url, entryType:'resource', initiatorType:'xmlhttprequest',
                     startTime:started, duration:performance.now()-started});
      this._requestId = 0; clearTimeout(this._timeoutId);
      this.status = status; this.responseURL = url; this._mime = this._overrideMime || mime;
      this.readyState = 2; this._fire('readystatechange');
      const text = new TextDecoder().decode(bytes);
      this.responseText = text; this.readyState = 3; this._fire('readystatechange'); this._fire('progress');
      this.response = this.responseType === 'arraybuffer' ? bytes.slice(0)
        : this.responseType === 'blob' ? new Blob([bytes], {type:this._mime})
        : this.responseType === 'json' ? JSON.parse(text) : text;
      this.readyState = 4; this._sent = false; this._fire('readystatechange'); this._fire('load'); this._fire('loadend');
    }, () => {
      if (generation !== this._generation) return;
      this._fail('error');
    });
    this._armTimeout();
  }
}
for (const [name,value] of Object.entries({UNSENT:0, OPENED:1, HEADERS_RECEIVED:2, LOADING:3, DONE:4})) {
  Object.defineProperty(XMLHttpRequest, name, {value});
  Object.defineProperty(XMLHttpRequest.prototype, name, {value});
}
globalThis.XMLHttpRequest = XMLHttpRequest;
class Storage {
  constructor() { this._data = new Map(); }
  get length() { return this._data.size; }
  key(index) { return [...this._data.keys()][Number(index)] ?? null; }
  getItem(key) { key=String(key); return this._data.has(key) ? this._data.get(key) : null; }
  setItem(key,value) { this._data.set(String(key),String(value)); }
  removeItem(key) { this._data.delete(String(key)); }
  clear() { this._data.clear(); }
}
globalThis.Storage = Storage;
globalThis.localStorage = new Storage();
globalThis.sessionStorage = new Storage();
class Selection {
  constructor() { this._ranges = []; }
  get anchorNode() { return this._ranges[0]?._context || null; }
  get focusNode() { return this.anchorNode; }
  get anchorOffset() { return 0; }
  get focusOffset() { return 0; }
  get isCollapsed() { return this._ranges.length === 0 || this._ranges[0].collapsed; }
  get rangeCount() { return this._ranges.length; }
  get type() { return this._ranges.length === 0 ? 'None' : this.isCollapsed ? 'Caret' : 'Range'; }
  addRange(range) { if (!(range instanceof Range)) throw new TypeError('Range required'); this._ranges = [range]; }
  getRangeAt(index) { if (Number(index) !== 0 || !this._ranges.length) throw new RangeError('IndexSizeError'); return this._ranges[0]; }
  removeAllRanges() { this._ranges = []; }
  empty() { this.removeAllRanges(); }
  collapse(node, offset = 0) { const range = document.createRange(); range.selectNodeContents(node); range.collapse(true); this._ranges = [range]; }
  selectAllChildren(node) { const range = document.createRange(); range.selectNodeContents(node); this._ranges = [range]; }
  containsNode() { return false; }
  deleteFromDocument() {}
  toString() { return ''; }
}
const documentSelection = new Selection();
globalThis.Selection = Selection;
globalThis.getSelection = () => documentSelection;

// Source: Blink CustomElementRegistry / ScriptCustomElementDefinitionBuilder.
// DOM owns definitions, element states and reaction queues. This realm retains
// only the JavaScript constructor/callback handles and whenDefined promises.
const ceInvoke = __domInvoke, ceApply = Reflect.apply, ceConstruct = Reflect.construct;
const ceProxy = Proxy, ceString = String, ceArrayIsArray = Array.isArray;
const ceDefinitions = new Map(), ceConstructors = new WeakMap(), ceWaiters = new Map();
const ceRegistryToken = {}, ceRegistryBrands = new WeakSet();
let ceDefinitionRunning = false;
function ceDOMString(value) {
  if (typeof value === 'symbol') throw new TypeError('Cannot convert a Symbol to DOMString');
  return ceString(value);
}
function ceBrand(registry) {
  if (!ceRegistryBrands.has(registry)) throw new TypeError('Illegal invocation');
}
function ceIsConstructor(value) {
  // IsConstructor without invoking author code or reading .prototype. A Proxy
  // only has [[Construct]] when its target does, even with a construct trap.
  try { ceConstruct(new ceProxy(value, {construct(){ return {}; }}), []); return true; }
  catch { return false; }
}
function ceStringSequence(value) {
  if (value === undefined) return [];
  const result = [];
  if (ceArrayIsArray(value) && !ceInvoke(document, '__ceIsProxy', value)) {
    // Blink NativeValueTraits<IDLSequence<IDLString>>'s actual-array branch.
    for (let i=0; i<value.length; ++i) result.push(ceDOMString(value[i]));
    return result;
  }
  if (value === null || (typeof value !== 'object' && typeof value !== 'function'))
    throw new TypeError('Value is not a sequence');
  const iteratorMethod = value[Symbol.iterator];
  if (typeof iteratorMethod !== 'function') throw new TypeError('Value is not iterable');
  const iterator = ceApply(iteratorMethod, value, []);
  if (iterator === null || (typeof iterator !== 'object' && typeof iterator !== 'function'))
    throw new TypeError('Iterator must be an object');
  const next = iterator.next;
  if (typeof next !== 'function') throw new TypeError('Iterator next must be callable');
  while (true) {
    const step = ceApply(next, iterator, []);
    if (step === null || (typeof step !== 'object' && typeof step !== 'function'))
      throw new TypeError('Iterator result must be an object');
    // Blink ScriptIterator::Next retrieves value before done, including the
    // final record; NativeValueTraits does not close on conversion failure.
    const entry = step.value;
    if (step.done) return result;
    result.push(ceDOMString(entry));
  }
}
function ceCallback(prototype, name) {
  const callback = prototype[name];
  if (callback !== undefined && typeof callback !== 'function')
    throw new TypeError(name + ' is not a function');
  return callback;
}
function ceDrainReactions() {
  let reaction;
  while ((reaction = ceInvoke(document, '__ceNext')) !== null) {
    const definition = ceDefinitions.get(reaction.definition);
    if (!definition) throw new Error('Custom element definition has no script handles');
    const element = reaction.element;
    if (reaction.kind === 'upgrade') {
      if (!ceInvoke(element, '__ceBegin', reaction.definition)) continue;
      let success = false;
      try {
        __domUpgradeCustomElement(element, definition.constructor);
        success = true;
      } catch(error) { ceInvoke(document, '__ceReportException', error); }
      finally { ceInvoke(element, '__ceComplete', reaction.definition, success); }
    } else {
      const callback = definition.callbacks[reaction.kind];
      if (callback === undefined) continue;
      const argumentsList = reaction.kind === 'attributeChangedCallback'
        ? [reaction.name, reaction.oldValue, reaction.newValue, reaction.namespace] : [];
      try { ceApply(callback, element, argumentsList); }
      catch(error) { ceInvoke(document, '__ceReportException', error); }
    }
  }
}
__domWrap('__customElementReactions', ceDrainReactions);
__domWrap('__customElementBackupReactions', function() {
  ceInvoke(document, '__ceBackupStart');
  ceDrainReactions();
});
function ceWithReactions(action) {
  ceInvoke(document, '__cePush');
  try { return action(); }
  finally {
    try { ceDrainReactions(); }
    finally { ceInvoke(document, '__cePop'); }
  }
}
class CustomElementRegistry {
  constructor(token) {
    if (token !== ceRegistryToken)
      throw new DOMException('Scoped custom element registries are not supported', 'NotSupportedError');
    ceRegistryBrands.add(this);
  }
  define(name, constructor, options = {}) {
    ceBrand(this);
    if (arguments.length < 2) throw new TypeError('define requires a name and constructor');
    name = ceDOMString(name);
    if (typeof constructor !== 'function') throw new TypeError('Constructor must be callable');
    if (options != null && typeof options !== 'object' && typeof options !== 'function')
      throw new TypeError('ElementDefinitionOptions must be a dictionary');
    let extendsName = options == null ? null : options.extends;
    extendsName = extendsName == null ? null : ceDOMString(extendsName);
    return ceWithReactions(() => {
      if (!ceIsConstructor(constructor)) throw new TypeError('Constructor is not constructible');
      if (!ceInvoke(document, '__ceValidName', name))
        throw new DOMException('Invalid custom element name', 'SyntaxError');
      if (ceInvoke(document, '__ceFind', name))
        throw new DOMException('Custom element already defined', 'NotSupportedError');
      if (ceConstructors.has(constructor))
        throw new DOMException('Constructor already registered', 'NotSupportedError');
      if (extendsName !== null)
        throw new DOMException('Customized built-in elements are not supported', 'NotSupportedError');
      if (ceDefinitionRunning)
        throw new DOMException('An element definition is already being processed', 'NotSupportedError');
      let callbacks, observedAttributes, disabledFeatures, formAssociated;
      ceDefinitionRunning = true;
      try {
        const prototype = constructor.prototype;
        if (prototype === null || (typeof prototype !== 'object' && typeof prototype !== 'function'))
          throw new TypeError('Constructor prototype must be an object');
        callbacks = {};
        for (const name of ['connectedCallback', 'disconnectedCallback', 'connectedMoveCallback',
                            'adoptedCallback', 'attributeChangedCallback'])
          callbacks[name] = ceCallback(prototype, name);
        observedAttributes = callbacks.attributeChangedCallback === undefined
          ? [] : ceStringSequence(constructor.observedAttributes);
        disabledFeatures = ceStringSequence(constructor.disabledFeatures);
        formAssociated = !!constructor.formAssociated;
        if (formAssociated) {
          for (const name of ['formAssociatedCallback', 'formResetCallback', 'formDisabledCallback',
                              'formStateRestoreCallback', 'toolFillCallback'])
            callbacks[name] = ceCallback(prototype, name);
        }
      } finally { ceDefinitionRunning = false; }
      if (formAssociated || disabledFeatures.includes('shadow') || disabledFeatures.includes('internals'))
        throw new DOMException('Form-associated and disabled-feature definitions are not supported', 'NotSupportedError');
      const flags = ['connectedCallback', 'disconnectedCallback', 'connectedMoveCallback',
                     'adoptedCallback', 'attributeChangedCallback']
        .reduce((flags, name, bit) => flags | (callbacks[name] === undefined ? 0 : 1 << bit), 0);
      const id = ceInvoke(document, '__ceRegister', name, flags, ...observedAttributes);
      ceDefinitions.set(id, {constructor, callbacks});
      ceConstructors.set(constructor, id);
      const waiter = ceWaiters.get(name);
      if (waiter) { ceWaiters.delete(name); waiter.resolve(constructor); }
    });
  }
  get(name) {
    ceBrand(this);
    if (!arguments.length) throw new TypeError('get requires a name');
    return ceDefinitions.get(ceInvoke(document, '__ceFind', ceDOMString(name)))?.constructor;
  }
  getName(constructor) {
    ceBrand(this);
    if (typeof constructor !== 'function') throw new TypeError('Constructor must be callable');
    const id = ceConstructors.get(constructor);
    return id === undefined ? null : ceInvoke(document, '__ceDefinitionName', id);
  }
  whenDefined(name) {
    try {
      ceBrand(this);
      if (!arguments.length) throw new TypeError('whenDefined requires a name');
      name = ceDOMString(name);
      if (!ceInvoke(document, '__ceValidName', name))
        throw new DOMException('Invalid custom element name', 'SyntaxError');
      const definition = ceDefinitions.get(ceInvoke(document, '__ceFind', name));
      if (definition) return Promise.resolve(definition.constructor);
      let waiter = ceWaiters.get(name);
      if (!waiter) {
        waiter = {};
        waiter.promise = new Promise(resolve => { waiter.resolve = resolve; });
        ceWaiters.set(name, waiter);
      }
      return waiter.promise;
    } catch(error) { return Promise.reject(error); }
  }
  upgrade(root) {
    ceBrand(this);
    if (!arguments.length) throw new TypeError('upgrade requires a Node');
    ceWithReactions(() => ceInvoke(root, '__ceUpgrade'));
  }
}
for (const member of ['define', 'get', 'getName', 'whenDefined', 'upgrade']) {
  const descriptor = Object.getOwnPropertyDescriptor(CustomElementRegistry.prototype, member);
  Object.defineProperty(CustomElementRegistry.prototype, member, {...descriptor, enumerable:true});
}
Object.defineProperty(CustomElementRegistry.prototype, Symbol.toStringTag,
  {value:'CustomElementRegistry', configurable:true});
globalThis.CustomElementRegistry = CustomElementRegistry;
globalThis.customElements = new CustomElementRegistry(ceRegistryToken);
const nativeCreateElement = Document.prototype.createElement;
const nativeCreateElementNS = Document.prototype.createElementNS;
Document.prototype.createElement = function(name, options) {
  return ceWithReactions(() => {
    const element = nativeCreateElement.call(this, name, options);
    ceInvoke(element, '__ceUpgrade');
    return element;
  });
};
Document.prototype.createElementNS = function(namespace, name, options) {
  return ceWithReactions(() => {
    const element = nativeCreateElementNS.call(this, namespace, name, options);
    ceInvoke(element, '__ceUpgrade');
    return element;
  });
};


function observerRect(rect) {
  return {x:rect.x,y:rect.y,width:rect.width,height:rect.height,
          top:rect.top,right:rect.right,bottom:rect.bottom,left:rect.left};
}
function parseIntersectionMargin(value) {
  const input=value == null ? '0px' : String(value).trim();
  const tokens=input ? input.split(/\s+/) : [];
  if(tokens.length>4)throw new SyntaxError('Extra text found at the end of rootMargin.');
  const values=tokens.map(token=>{
    const match=/^([+-]?(?:\d+(?:\.\d*)?|\.\d+))(px|%)$/i.exec(token);
    if(!match)throw new SyntaxError('rootMargin must be specified in pixels or percent.');
    const unit=match[2].toLowerCase(), parsed=Number(match[1]);
    return {value:unit==='px'?Math.floor(parsed):parsed,unit};
  });
  if(!values.length)values.push({value:0,unit:'px'});
  if(values.length===1)values.push(values[0],values[0],values[0]);
  else if(values.length===2)values.push(values[0],values[1]);
  else if(values.length===3)values.push(values[1]);
  return values;
}
function expandIntersectionRoot(rect, margins) {
  const value=(margin,index)=>margin.unit==='px' ? margin.value :
    margin.value*(index%2 ? rect.width : rect.height)/100;
  const top=rect.top-value(margins[0],0), right=rect.right+value(margins[1],1);
  const bottom=rect.bottom+value(margins[2],2), left=rect.left-value(margins[3],3);
  return {x:left,y:top,left,top,right,bottom,width:right-left,height:bottom-top};
}
class IntersectionObserverEntry {
  constructor(target, rootBounds, bounding, intersection, isIntersecting, time) {
    this._target=target; this._time=time; this._rootBounds=rootBounds;
    this._boundingClientRect=bounding; this._intersectionRect=intersection;
    this._isIntersecting=isIntersecting;
    const area=bounding.width*bounding.height;
    this._intersectionRatio=area ? intersection.width*intersection.height/area : (this._isIntersecting?1:0);
  }
  get time(){return this._time;}
  get target(){return this._target;}
  get rootBounds(){return this._rootBounds;}
  get boundingClientRect(){return this._boundingClientRect;}
  get intersectionRect(){return this._intersectionRect;}
  get isIntersecting(){return this._isIntersecting;}
  get intersectionRatio(){return this._intersectionRatio;}
}
// Blink IntersectionObserverController separates post-layout computation from
// queued notifications. Unchanged observations must never post timer tasks.
const intersectionObservers = new Set(), initialIntersectionObservers = new Set(), pendingIntersectionObservers = new Set();
let intersectionInitialScheduled = false, intersectionDeliveryScheduled = false;
function computeIntersectionObservations(observers) {
  // ComputeIntersectionsContext shares root/target geometry for one lifecycle.
  const rects = new Map(), time = performance.now();
  const rect = target => {
    if (target instanceof Document) target = null;
    if (!rects.has(target)) rects.set(target, target ? observerRect(target.getBoundingClientRect()) :
      {left:0,top:0,right:innerWidth,bottom:innerHeight,width:innerWidth,height:innerHeight,x:0,y:0});
    return rects.get(target);
  };
  for (const observer of observers) if (intersectionObservers.has(observer)) observer._compute(rect, time);
}
function queueIntersectionDelivery(observer) {
  pendingIntersectionObservers.add(observer);
  if (intersectionDeliveryScheduled) return;
  intersectionDeliveryScheduled = true;
  setTimeout(() => {
    intersectionDeliveryScheduled = false;
    const observers = [...pendingIntersectionObservers]; pendingIntersectionObservers.clear();
    for (const observer of observers) {
      const entries = observer.takeRecords();
      if (entries.length) try {observer._callback(entries, observer)} catch(error) {setTimeout(() => {throw error}, 0)}
    }
  }, 0);
}
function scheduleInitialIntersection(observer) {
  initialIntersectionObservers.add(observer);
  if (intersectionInitialScheduled) return;
  intersectionInitialScheduled = true;
  setTimeout(() => {
    intersectionInitialScheduled = false;
    const observers = [...initialIntersectionObservers]; initialIntersectionObservers.clear();
    computeIntersectionObservations(observers);
  }, 0);
}
class IntersectionObserver {
  constructor(callback, options = {}) {
    if (typeof callback !== 'function') throw new TypeError('IntersectionObserver callback must be a function');
    this._callback=callback; this.root=options.root || null;
    this._rootMarginValues=parseIntersectionMargin(options.rootMargin);
    this.rootMargin=this._rootMarginValues.map(margin=>`${margin.value}${margin.unit}`).join(' ');
    const threshold=options.threshold === undefined ? [0] : Array.isArray(options.threshold) ? options.threshold : [options.threshold];
    this.thresholds=[...new Set(threshold.map(Number).sort((a,b)=>a-b))];
    if (this.thresholds.some(value=>!Number.isFinite(value)||value<0||value>1)) throw new RangeError('Invalid threshold');
    this._targets=new Set(); this._states=new Map(); this._records=[];
  }
  observe(target) { if (!(target instanceof Element)) throw new TypeError('Element required'); this._targets.add(target); intersectionObservers.add(this); scheduleInitialIntersection(this); }
  unobserve(target) { this._targets.delete(target); this._states.delete(target); if(!this._targets.size)intersectionObservers.delete(this); }
  disconnect() { this._targets.clear(); this._states.clear(); this._records=[]; intersectionObservers.delete(this); initialIntersectionObservers.delete(this); pendingIntersectionObservers.delete(this); }
  takeRecords() { return this._records.splice(0); }
  _compute(rect, now) {
      const unexpandedRoot=rect(this.root);
      const root=expandIntersectionRoot(observerRect(unexpandedRoot),this._rootMarginValues);
      const rootBounds=observerRect(root);
      for (const target of this._targets) {
        const bounds=rect(target);
        const left=Math.max(bounds.left,root.left), top=Math.max(bounds.top,root.top);
        const right=Math.min(bounds.right,root.right), bottom=Math.min(bounds.bottom,root.bottom);
        // IntersectionObserver uses edge-inclusive intersection. In
        // particular, a zero-area sentinel inside the root is intersecting
        // (with ratio 1), which is commonly used to toggle sticky UI.
        const isIntersecting=right>=left && bottom>=top;
        const width=isIntersecting?Math.max(0,right-left):0;
        const height=isIntersecting?Math.max(0,bottom-top):0;
        const entry=new IntersectionObserverEntry(target,rootBounds,bounds,
          {x:left,y:top,left,top,right:left+width,bottom:top+height,width,height},isIntersecting,now);
        const thresholdIndex=this.thresholds.findIndex(value=>entry.intersectionRatio<value);
        const index=thresholdIndex<0?this.thresholds.length:thresholdIndex;
        const previous=this._states.get(target);
        if(!previous||previous.index!==index||previous.isIntersecting!==entry.isIntersecting)
          this._records.push(entry);
        this._states.set(target,{index,isIntersecting:entry.isIntersecting});
      }
      if (this._records.length) queueIntersectionDelivery(this);
  }
}
globalThis.IntersectionObserverEntry=IntersectionObserverEntry;
globalThis.IntersectionObserver=IntersectionObserver;
Object.defineProperty(globalThis,'__browserUpdateIntersectionObservations',{
  value(){computeIntersectionObservations(intersectionObservers);},
  configurable:true
});
class ResizeObserverEntry {
  constructor(target, rect) {
    this.target=target; this.contentRect=rect;
    const size={inlineSize:rect.width,blockSize:rect.height};
    this.contentBoxSize=[size]; this.borderBoxSize=[size]; this.devicePixelContentBoxSize=[size];
  }
}
class ResizeObserver {
  constructor(callback) { if(typeof callback!=='function')throw new TypeError('ResizeObserver callback must be a function');this._callback=callback;this._targets=new Set();this._scheduled=false; }
  observe(target) { if(!(target instanceof Element))throw new TypeError('Element required');this._targets.add(target);this._schedule(); }
  unobserve(target) { this._targets.delete(target); }
  disconnect() { this._targets.clear(); }
  _schedule() { if(this._scheduled)return;this._scheduled=true;setTimeout(()=>{this._scheduled=false;const entries=[];for(const target of this._targets)entries.push(new ResizeObserverEntry(target,observerRect(target.getBoundingClientRect())));if(entries.length)this._callback(entries,this);},0); }
}
globalThis.ResizeObserverEntry=ResizeObserverEntry;
globalThis.ResizeObserver=ResizeObserver;
})()
