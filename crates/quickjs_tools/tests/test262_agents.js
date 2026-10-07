function fail(message) { throw new Error(message); }
const reports = [];
const agent = $262.agent;
const before = agent.monotonicNow(); agent.sleep(2);
if (agent.monotonicNow() < before) fail('clock');
for (let id = 0; id < 4; id++) {
  agent.start(`
    const id = ${id};
    try { $262.agent.start(''); } catch(e) { $262.agent.report('guard:'+id+':'+e.message); }
    try { $262.agent.broadcast(new SharedArrayBuffer(4),0); } catch(e) { $262.agent.report('broadcast:'+id+':'+e.message); }
    try { $262.agent.receiveBroadcast(1); } catch(e) { $262.agent.report('callable:'+id+':'+e.message); }
    $262.agent.receiveBroadcast((sab,value)=> {
      const view=new Int32Array(sab);
      Atomics.store(view,id,value+id);
      Atomics.add(view,4,1); Atomics.notify(view,4);
      if(id===0) {
        while(Atomics.load(view,4)<4) Atomics.wait(view,4,Atomics.load(view,4),1000);
        $262.agent.report('shared:'+Array.from(view).join(','));
      }
      Promise.resolve().then(()=>{$262.agent.report('job:'+id); $262.agent.leaving();});
    });
    $262.agent.report('ready:'+id);
  `);
}
function collect(n) {
  const end = agent.monotonicNow()+10000;
  while(reports.length<n) {
    const r=agent.getReport(); if(r!==null) reports.push(r);
    else if(agent.monotonicNow()>end) fail('reports timed out '+reports.length); else agent.sleep(1);
  }
}
collect(16);
const sab=new SharedArrayBuffer(20); agent.broadcast(sab,37);
collect(21); reports.sort(); print(JSON.stringify(reports));
if(agent.getReport()!==null) fail('queue not empty');
print(JSON.stringify(Array.from(new Int32Array(sab))));
try { agent.leaving(); } catch(e) { print(e.message); }
try { agent.receiveBroadcast(()=>{}); } catch(e) { print(e.message); }
const r=$262.createRealm(); print(r.evalScript('Object.keys($262).join(",")'));
print(typeof r.evalScript('$262.IsHTMLDDA')+','+r.evalScript('$262.IsHTMLDDA == null')+','+r.evalScript('$262.IsHTMLDDA()'));
const ab=new ArrayBuffer(4); $262.detachArrayBuffer(ab); print(ab.byteLength);
print($262.codePointRange(0x1f600,0x1f602));
print('Test262:AsyncTestComplete');
