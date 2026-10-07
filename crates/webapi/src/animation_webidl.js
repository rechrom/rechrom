;(function() {
'use strict';
// Web Animations host adapter. Effects contribute at animation cascade origin;
// sampling never rewrites the element's inline style or its DOM attributes.
const all = new Set(), running = new Set(); let scheduled=false, nextId=1;
const cssName = name => name.startsWith('--') ? name : name.replace(/[A-Z]/g,c=>'-'+c.toLowerCase());
const metadata = new Set(['offset','computedOffset','easing','composite']);
function ease(value, text='linear') {
  const presets={ease:'.25,.1,.25,1','ease-in':'.42,0,1,1','ease-out':'0,0,.58,1','ease-in-out':'.42,0,.58,1'};
  text=String(text).trim(); if(presets[text])text='cubic-bezier('+presets[text]+')';
  if(text==='step-start')return 1;if(text==='step-end')return value<1?0:1;
  const steps=text.match(/^steps\(\s*(\d+)\s*(?:,\s*(start|end|jump-start|jump-end|jump-none|jump-both))?\s*\)$/);
  if(steps){const n=Number(steps[1]),mode=steps[2]||'end';let step=Math.floor(value*n),count=n;
    if(mode==='start'||mode==='jump-start'||mode==='jump-both')step++;
    if(mode==='jump-both')count++;if(mode==='jump-none')count--;
    return Math.max(0,Math.min(1,step/count));}
  const match=text.match(/^cubic-bezier\(([^)]+)\)$/);if(!match)return value;
  const [x1,y1,x2,y2]=match[1].split(',').map(Number);
  const curve=(t,a,b)=>3*(1-t)*(1-t)*t*a+3*(1-t)*t*t*b+t*t*t;
  let lo=0,hi=1;for(let i=0;i<30;i++){const mid=(lo+hi)/2;if(curve(mid,x1,x2)<value)lo=mid;else hi=mid;}
  return curve((lo+hi)/2,y1,y2);
}
function interpolate(a,b,t,property) {
  if(a===b)return a;
  // Unregistered custom properties have the CSS discrete animation type.
  if(property.startsWith('--'))return t<.5?a:b;
  const pattern=/-?(?:\d*\.\d+|\d+\.?\d*)(?:e[+-]?\d+)?/gi;
  const aa=String(a).match(pattern),bb=String(b).match(pattern);
  if(!aa||!bb||aa.length!==bb.length||String(a).replace(pattern,'#')!==String(b).replace(pattern,'#'))return t<.5?a:b;
  let i=0;return String(a).replace(pattern,()=>String(Number(aa[i])+(Number(bb[i])-Number(aa[i++]))*t));
}
function normalize(input) {
  let frames;
  if(Array.isArray(input))frames=input.map(frame=>({...frame}));
  else {input=input||{};const properties=Object.keys(input).filter(k=>!metadata.has(k));
    const count=Math.max(0,...properties.map(k=>Array.isArray(input[k])?input[k].length:1));frames=Array.from({length:count},()=>({}));
    for(const property of properties){const values=Array.isArray(input[property])?input[property]:[input[property]];
      for(let i=0;i<values.length;i++){const index=values.length===1?count-1:Math.round(i*(count-1)/(values.length-1));frames[index][property]=values[i];}}
    for(let i=0;i<count;i++)for(const key of ['offset','easing','composite'])if(input[key]!==undefined)frames[i][key]=Array.isArray(input[key])?input[key][i]:input[key];
  }
  let previous=-1;
  for(const frame of frames){frame.offset=frame.offset==null?null:Number(frame.offset);frame.easing=frame.easing||'linear';frame.composite=frame.composite||'auto';
    if(frame.offset!==null){if(!Number.isFinite(frame.offset)||frame.offset<0||frame.offset>1||frame.offset<previous)throw new TypeError('Invalid keyframe offsets');previous=frame.offset;}
    for(const key of Object.keys(frame))if(!metadata.has(key))frame[key]=String(frame[key]);}
  if(frames.length===1 && frames[0].offset===null)frames[0].offset=1;
  if(frames.length>1){if(frames[0].offset===null)frames[0].offset=0;if(frames.at(-1).offset===null)frames.at(-1).offset=1;}
  for(let i=0;i<frames.length;){let j=i+1;while(j<frames.length&&frames[j].offset===null)j++;
    const start=frames[i].offset??0,end=frames[j]?.offset??1;
    for(let k=i;k<j;k++)frames[k].computedOffset=start+(end-start)*(k-i)/(j-i);i=j;}
  return frames;
}
class KeyframeEffect {
  constructor(target,keyframes,options={}){this.target=target;this.pseudoElement=null;this._animation=null;this.setKeyframes(keyframes);this._timing={delay:0,endDelay:0,fill:'auto',iterationStart:0,iterations:1,duration:'auto',direction:'normal',easing:'linear'};this.updateTiming(typeof options==='number'?{duration:options}:options);}
  setKeyframes(keyframes){this._frames=normalize(keyframes);this._animation?._sample();}
  getKeyframes(){return this._frames.map(frame=>({...frame}));}
  getTiming(){return {...this._timing};}
  updateTiming(options={}){const timing={...this._timing,...options};
    for(const key of ['delay','endDelay','iterationStart','iterations'])timing[key]=Number(timing[key]);
    if(timing.duration!=='auto')timing.duration=Number(timing.duration);
    if(timing.iterations<0||timing.iterationStart<0||timing.duration<0)throw new TypeError('Invalid animation timing');
    this._timing=timing;this._animation?._sample();}
  getComputedTiming(){const t=this._timing,duration=t.duration==='auto'?0:t.duration,activeDuration=duration*t.iterations;
    const endTime=Math.max(0,t.delay+activeDuration+t.endDelay),localTime=this._animation?.currentTime??null;
    let progress=null,currentIteration=null;
    if(localTime!==null){let active=localTime-t.delay;const before=active<0,after=active>=activeDuration;
      if((!before&&!after)||(before&&['backwards','both'].includes(t.fill))||(after&&['forwards','both'].includes(t.fill))){
        active=Math.max(0,Math.min(activeDuration,active));const overall=(duration?active/duration:t.iterations)+t.iterationStart;
        currentIteration=Math.floor(overall);progress=overall%1;
        if(active===activeDuration&&progress===0&&t.iterations!==0){progress=1;currentIteration=Math.max(0,currentIteration-1);}
        const reverse=t.direction==='reverse'||(t.direction==='alternate'&&currentIteration%2===1)||(t.direction==='alternate-reverse'&&currentIteration%2===0);
        if(reverse)progress=1-progress;progress=ease(progress,t.easing);
      }
    }
    return {...t,duration,activeDuration,endTime,localTime,progress,currentIteration};}
}
function schedule(){if(scheduled||!running.size)return;scheduled=true;requestAnimationFrame(()=>{scheduled=false;for(const a of [...running])a._sample();schedule();});}
class Animation extends EventTarget {
  constructor(effect=null,timeline=document.timeline){super();this.id='';this.effect=effect;this.timeline=timeline;this._effectId=nextId++;this._rate=1;this._start=null;this._hold=null;this._state='idle';this._finishedPromise=null;this._lastCSS=null;if(effect)effect._animation=this;all.add(this);}
  get playbackRate(){return this._rate;}set playbackRate(rate){const current=this.currentTime;this._rate=Number(rate);if(current!==null){this._hold=current;this._start=performance.now();}this._sample();}
  get startTime(){return this._start;}set startTime(time){this._start=time==null?null:Number(time);this._hold=0;this._sample();}
  get currentTime(){if(this._hold===null)return null;return this._state==='running'&&this._start!==null?this._hold+(performance.now()-this._start)*this._rate:this._hold;}
  set currentTime(time){this._hold=time==null?null:Number(time);this._start=this._state==='running'?performance.now():null;this._sample();}
  get playState(){return this._state;}get pending(){return false;}get replaceState(){return 'active';}
  get ready(){return Promise.resolve(this);}
  get finished(){if(!this._finishedPromise)this._finishedPromise=new Promise((resolve,reject)=>{this._resolve=resolve;this._reject=reject;if(this._state==='finished')resolve(this);});return this._finishedPromise;}
  play(){if(this._state==='running')return;const timing=this.effect?.getComputedTiming();
    if(this._hold===null||this._state==='finished')this._hold=this._rate<0?(timing?.endTime||0):0;
    this._state='running';this._start=performance.now();this._finishedPromise=null;running.add(this);this._sample();schedule();}
  pause(){this._hold=this.currentTime??0;this._start=null;this._state='paused';running.delete(this);this._sample();}
  cancel(){this._state='idle';this._hold=this._start=null;running.delete(this);this._clear();if(this._reject)this._reject(new DOMException('Animation canceled','AbortError'));this._finishedPromise=null;this._resolve=this._reject=null;this.dispatchEvent(new Event('cancel'));}
  finish(){const end=this.effect?.getComputedTiming().endTime||0;if(!Number.isFinite(end)||this._rate===0)throw new DOMException('Cannot finish animation','InvalidStateError');this._hold=this._rate<0?0:end;this._start=null;this._complete();this._sample();}
  reverse(){this.playbackRate=-this.playbackRate;this.play();}
  updatePlaybackRate(rate){this.playbackRate=rate;}
  persist(){}
  commitStyles(){const css=this._css();for(const [name,value]of Object.entries(css))this.effect.target.style.setProperty(name,value);}
  _complete(){if(this._state==='finished')return;this._hold=this._rate<0?0:this.effect.getComputedTiming().endTime;this._start=null;this._state='finished';running.delete(this);this._resolve?.(this);setTimeout(()=>this.dispatchEvent(new Event('finish')),0);}
  _clear(){if(this.effect?.target && this._lastCSS!==''){__domInvoke(this.effect.target,'animationSample',this._effectId,performance.now(),'');this._lastCSS='';}}
  _css(){if(this._state==='idle'||!this.effect?.target)return {};const progress=this.effect.getComputedTiming().progress;if(progress===null)return {};
    const result={},properties=new Set(this.effect._frames.flatMap(frame=>Object.keys(frame).filter(k=>!metadata.has(k))));
    for(const property of properties){const frames=this.effect._frames.filter(frame=>property in frame).map(frame=>({offset:frame.computedOffset,value:frame[property],easing:frame.easing}));
      const name=cssName(property),underlying=()=>getComputedStyle(this.effect.target).getPropertyValue(name);
      if(frames[0].offset>0)frames.unshift({offset:0,value:underlying(),easing:'linear'});
      if(frames.at(-1).offset<1)frames.push({offset:1,value:underlying(),easing:'linear'});
      let index=0;while(index<frames.length-2&&progress>=frames[index+1].offset)index++;
      const a=frames[index],b=frames[index+1]||a,t=b.offset===a.offset?1:(progress-a.offset)/(b.offset-a.offset);
      result[name]=interpolate(a.value,b.value,ease(Math.max(0,Math.min(1,t)),a.easing),name);
    }return result;}
  _sample(){if(!this.effect?.target)return;
    if(this._state==='running'){const time=this.currentTime,end=this.effect.getComputedTiming().endTime;if((this._rate>=0&&time>=end)||(this._rate<0&&time<=0))this._complete();}
    const css=Object.entries(this._css()).map(([name,value])=>name+':'+value+';').join('');
    if(css!==this._lastCSS){__domInvoke(this.effect.target,'animationSample',this._effectId,performance.now(),css);this._lastCSS=css;}
  }
}
Object.assign(globalThis,{Animation,KeyframeEffect});
const timelines=new WeakMap();Object.defineProperty(Document.prototype,'timeline',{get(){if(!timelines.has(this))timelines.set(this,{get currentTime(){return performance.now();}});return timelines.get(this);}});
Element.prototype.animate=function(frames,options={}){const animation=new Animation(new KeyframeEffect(this,frames,options),this.ownerDocument.timeline);if(options.id!==undefined)animation.id=String(options.id);animation.play();return animation;};
Element.prototype.getAnimations=function(options={}){return [...all].filter(a=>a.playState!=='idle'&&(a.effect.target===this||(options.subtree&&this.contains(a.effect.target))));};
Document.prototype.getAnimations=function(){return [...all].filter(a=>a.playState!=='idle'&&a.effect.target?.ownerDocument===this&&a.effect.target.isConnected);};
})()