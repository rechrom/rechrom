import {Worker} from 'os';
let worker = new Worker('./libc_worker_child.mjs');
let received=[];
worker.onmessage=(event)=>{
 const v=event.data;
 received.push([v.index,v.self===v,v.a.buffer===v.b.buffer,v.a[0],v.big.toString(),v.nested]);
 if(v.index===2){worker.onmessage=null;print(JSON.stringify(received));}
};
let s=new SharedArrayBuffer(16),a=new Int32Array(s);a[0]=37;
for(let index=0;index<3;index++){let v={index,a,b:new Int32Array(s),big:123456789012345678901234567890n};v.self=v;worker.postMessage(v);}
