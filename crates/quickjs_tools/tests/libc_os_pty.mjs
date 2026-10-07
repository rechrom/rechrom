import * as os from 'os';
const out=(...args)=>console.log(JSON.stringify(args));
const caught=fn=>{try {return fn();}catch(e){return e.name+':'+e.message;}};
out('tty',os.isatty(0),os.ttyGetWinSize(0),os.ttyGetWinSize(-1),caught(()=>os.ttyGetWinSize(Symbol())));
const names=Object.keys(os).filter(k=>typeof os[k]==='number').sort();
out('constants',names.map(k=>[k,os[k]]));
out('exec failures',os.exec(['/not-a-quickjs-file'],{usePath:false}),os.exec(['/bin/sh','-c','exit 0'],{cwd:'/not-a-quickjs-dir'}),os.exec(['/bin/sh','-c','exit 0'],{stdin:-1}),os.exec(['ignored'],{file:'/not-a-quickjs-file'}));
out('exec env',os.exec(['/bin/sh','-c','test "$EXAMPLE_QJS" = "x y" && test "$EMPTY_QJS" = ""'],{env:{EXAMPLE_QJS:'x y',EMPTY_QJS:''},usePath:false}));
let order=[];
const text=(name)=>({toString(){order.push(name);return name;}});
for(let name of ['block','usePath','file','cwd','stdin','stdout','stderr','env','uid','gid']){
 order=[];let options=new Proxy({}, {get(_o,key){order.push(key);if(key===name)throw Error(name);}});
 out('option',name,caught(()=>os.exec([text('argv0'),text('argv1')],options)),order);
}
order=[];out('env getter',caught(()=>os.exec([text('argv0')],{env:{first:text('first'),get second(){order.push('second');throw Error('second')}}})),order);
order=[];out('argv getter',caught(()=>os.exec({length:3,0:text('first'),get 1(){order.push('throw');throw Error('argv')}})),order);
out('coercions',caught(()=>os.exec([Symbol()])),caught(()=>os.exec(['x'],{file:Symbol()})),caught(()=>os.exec(['x'],{env:{k:Symbol()}})),caught(()=>os.exec(['x'],{stdin:Symbol()})));
let signals=[];
os.signal(os.SIGUSR1,()=>{signals.push('first');os.signal(os.SIGUSR1,()=>{signals.push('second');os.signal(os.SIGUSR1,undefined);});os.kill(os.getpid(),os.SIGUSR1);});
os.kill(os.getpid(),os.SIGUSR1);await os.sleepAsync(0);await os.sleepAsync(0);
os.kill(os.getpid(),os.SIGUSR1);await os.sleepAsync(0);os.signal(os.SIGUSR1,null);
out('signals',signals,caught(()=>os.signal(-1,null)),caught(()=>os.signal(64,null)),caught(()=>os.signal(Symbol(),null)));
let fifo=[];let a=os.setTimeout(()=>{fifo.push('A');os.clearTimeout(b);os.setTimeout(()=>fifo.push('C'),0)},0);let b=os.setTimeout(()=>fifo.push('B'),0);
await os.sleepAsync(0);await os.sleepAsync(0);out('timer mutate',fifo);
out('raw result',os.ttySetRaw(0));
