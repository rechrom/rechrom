import * as std from 'std';
globalThis.std=std;
function emit(name,value){std.out.puts(name,'=',JSON.stringify(value),'\n');}
function caught(fn){try{return fn()}catch(e){return [e.name,e.message]}}
emit('exports',Object.keys(std));
emit('errno',Object.keys(std.Error).map(n=>[n,std.Error[n]]));
let formats=[
 [''],['literal %% done'],['%d %i %o %u %x %X',-1,23,511,-1,255,255],
 ['%ld %lu %lx',123456789012345678901234n,-1n,-1n],['%+08d %#o %#x % d',23,8,15,23],
 ['%*.*s',-9,3,'abcdef'],['%.2f %.4g %a %E',1.235,12345,1.5,0.01],
 ['%f %g %F %f',NaN,Infinity,-Infinity,-0],['%c %c %c %c','😀',0x10ffff,-1,0x110000],
 ['%5.2c','é'],['%s', 'a\0b'],['%0250d',42],['%s',{toString(){return 'coerced'}}],
 ['%ld',1.9],['%q',1],['%d'],['%lld',1n],['%s',Symbol('x')]
];
emit('formats',formats.map(f=>caught(()=>std.sprintf(...f))));
let effects=[];emit('printf-order',std.sprintf('%*.*s %d',{valueOf(){effects.push('width');return 8}},{valueOf(){effects.push('precision');return 2}},{toString(){effects.push('string');return 'hello'}},{valueOf(){effects.push('number');return 4}}));emit('printf-effects',effects);
let f=std.tmpfile(),s='A\0B\nCé😀\n';f.puts(s);emit('tell',[f.tell(),f.tello().toString()]);emit('seek',f.seek(0,std.SEEK_SET));emit('lines',[f.getline(),f.getline(),f.getline(),f.eof(),f.error()]);f.clearerr();emit('cleared',[f.eof(),f.error()]);f.seek(0,0);emit('limited',[f.readAsString(2),f.readAsString(0),f.readAsString()]);
f.seek(0,0);let b=new ArrayBuffer(32);emit('read',f.read(b,1,7));emit('read-bytes',Array.from(new Uint8Array(b)).slice(0,10));emit('overflow',caught(()=>f.read(b,31,2)));emit('negative',caught(()=>f.read(b,-1,1)));emit('type',caught(()=>f.read({},0,1)));emit('fd',[typeof f.fileno(),f.fileno()>=0]);emit('close',f.close());emit('closed',[caught(()=>f.close()),caught(()=>f.getByte()),caught(()=>f.puts('fail'))]);
f=std.tmpfile();emit('putBytes',[f.putByte(65),f.putByte(-1),f.putByte(256)]);f.seek(0,0);emit('getBytes',[f.getByte(),f.getByte(),f.getByte(),f.getByte(),f.eof()]);f.close();
let err={};let missing=std.open('/tmp/quickjs-stdio-definitely-no-such-file','r',err);emit('missing',[missing,err.errno]);emit('modes',['rx','rbb','rw','w+',''].map(mode=>caught(()=>{let file=std.open('/dev/null',mode,err);let status=err.errno;if(file)file.close();return [!!file,status]})));
let p=std.popen("printf 'popen\\n'; exit 3",'r');emit('popen',[p.getline(),p.getline(),p.eof(),p.close()]);emit('popen-mode',caught(()=>std.popen('true','rx')));
emit('json',[std.parseExtJSON('{foo:1,/*c*/bar:[2,],}'),caught(()=>std.parseExtJSON('{foo:}'))]);
emit('eval',[std.evalScript('1+2'),std.evalScript('var nestedResult=std.evalScript("4+5");nestedResult'),caught(()=>std.evalScript('throw new TypeError("eval failure")'))]);
let asynchronous=std.evalScript('await Promise.resolve(17)',{async:true});emit('async',await asynchronous);
let url=std.getenv('QUICKJS_STDIO_TEST_URL');if(url){
 emit('url',std.urlGet(url+'/ok'));
 emit('url-full',std.urlGet(url+'/ok',{full:true}));
 let binary=std.urlGet(url+'/ok',{binary:true});emit('url-binary',Array.from(new Uint8Array(binary)));
 emit('url-missing',std.urlGet(url+'/missing'));
 emit('url-missing-full',std.urlGet(url+'/missing',{full:true}));
 effects=[];let options={get binary(){effects.push('binary');return false},get full(){effects.push('full');return true}};emit('url-options',std.urlGet(url+'/ok',options));emit('url-effects',effects);
 emit('url-escaped',std.urlGet(url+"/quote'[]{}\\",{full:true}));
}
std.out.flush();
