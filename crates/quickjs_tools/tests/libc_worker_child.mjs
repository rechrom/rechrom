import {Worker} from 'os';
let parent=Worker.parent;
let nested;
try {new Worker('./never-created.mjs')} catch(e) {nested=e.message;}
parent.onmessage=event=>{
 let v=event.data;if(v.self!==v||v.a.buffer!==v.b.buffer)throw Error('serialization identity');
 // Write once: all three messages share the same memory and view its final value.
 if(v.index===0)v.a[0]+=5;
 v.nested=nested;parent.postMessage(v);
 if(v.index===2)parent.onmessage=null;
};
