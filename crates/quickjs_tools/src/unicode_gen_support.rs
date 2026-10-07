// Rust host adaptations for unicode_gen.c standard allocation/stdio/ctype calls.
// C QuickJS is never invoked. Reallocation shares the already translated cutils
// default allocator so normalization/DynBuf buffers retain one ownership domain.
use std::{ffi::{CStr,CString,OsString},fs::{File,OpenOptions},io::{BufRead,BufReader,BufWriter,Read,Write},sync::Mutex};
enum ToolPrintArg { Float(f64), Int(u64), Str(*const c_char), Ptr(*const c_void) }
unsafe fn tool_format_print(fmt: *const c_char, args: &[ToolPrintArg]) -> Vec<u8> {
    let fmt = std::ffi::CStr::from_ptr(fmt).to_bytes();
    let mut out=Vec::new(); let mut i=0; let mut arg=0;
    while i<fmt.len() {
        if fmt[i]!=b'%' {out.push(fmt[i]);i+=1;continue;} i+=1;
        if fmt[i]==b'%' {out.push(b'%');i+=1;continue;}
        let mut zero=false;let mut left=false;
        loop {match fmt[i] {b'0'=>zero=true,b'-'=>left=true,_=>break} i+=1;}
        let width=if fmt[i]==b'*' {i+=1;let ToolPrintArg::Int(n)=args[arg] else {unreachable!()};arg+=1;n as i32} else {let mut n=0;while fmt[i].is_ascii_digit(){n=n*10+(fmt[i]-b'0') as i32;i+=1;}n};
        let mut precision=None;
        if fmt[i]==b'.' {i+=1;let n=if fmt[i]==b'*' {i+=1;let ToolPrintArg::Int(n)=args[arg] else {unreachable!()};arg+=1;n as i32} else {let mut n=0;while fmt[i].is_ascii_digit(){n=n*10+(fmt[i]-b'0') as i32;i+=1;}n};if n>=0 {precision=Some(n as usize)}}
        let mut longs=0;while fmt[i]==b'l' {longs+=1;i+=1;}
        let spec=fmt[i];i+=1;let a=&args[arg];arg+=1;
        let mut bytes=match (spec,a) {
            (b's',ToolPrintArg::Str(p))=>{let b=std::ffi::CStr::from_ptr(*p).to_bytes();b[..precision.unwrap_or(b.len()).min(b.len())].to_vec()},
            (b'c',ToolPrintArg::Int(n))=>vec![*n as u8],
            (b'd'|b'i',ToolPrintArg::Int(n))=>if longs>0 {(*n as i64).to_string().into_bytes()} else {(*n as i32).to_string().into_bytes()},
            (b'u',ToolPrintArg::Int(n))=>if longs>0 {n.to_string().into_bytes()} else {(*n as u32).to_string().into_bytes()},
            (b'x'|b'X',ToolPrintArg::Int(n))=>{let n=if longs>0 {*n} else {*n as u32 as u64};if spec==b'x' {format!("{n:x}").into_bytes()} else {format!("{n:X}").into_bytes()}},
            (b'f',ToolPrintArg::Float(n))=>format!("{:.*}",precision.unwrap_or(6),n).to_lowercase().into_bytes(),
            (b'p',ToolPrintArg::Ptr(p))=>format!("0x{:x}",*p as usize).into_bytes(),
            _=>unreachable!("unsupported official print directive"),
        };
        if matches!(spec,b'd'|b'i'|b'u'|b'x'|b'X') { if let Some(precision)=precision {
            zero=false;let negative=bytes.first()==Some(&b'-');let digit_count=bytes.len()-negative as usize;
            if precision==0 && bytes==b"0" {bytes.clear();} else if precision>digit_count {let mut padded=Vec::new();if negative {padded.push(b'-');}padded.resize(padded.len()+precision-digit_count,b'0');padded.extend_from_slice(&bytes[negative as usize..]);bytes=padded;}
        }}
        if width<0 {left=true;}let width=width.unsigned_abs() as usize;let padding=width.saturating_sub(bytes.len());
        if left||width==0 {out.append(&mut bytes);out.resize(out.len()+padding,b' ');} else if zero&&matches!(spec,b'd'|b'i'|b'u'|b'x'|b'X') {if bytes.first()==Some(&b'-') {out.push(b'-');bytes.remove(0);}out.resize(out.len()+padding,b'0');out.append(&mut bytes);} else {out.resize(out.len()+padding,b' ');out.append(&mut bytes);}
    }
    out
}

enum FileKind {Reader(BufReader<File>),Writer(BufWriter<File>)}
struct FILE{kind:FileKind}
unsafe fn malloc(size:usize)->*mut c_void{quickjs::cutils::dbuf_default_realloc(ptr::null_mut(),ptr::null_mut(),size)}
unsafe fn realloc(p:*mut c_void,size:usize)->*mut c_void{quickjs::cutils::dbuf_default_realloc(ptr::null_mut(),p,size)}
unsafe fn free(p:*mut c_void){let _=quickjs::cutils::dbuf_default_realloc(ptr::null_mut(),p,0);}
unsafe fn strlen(mut p:*const c_char)->usize {let start=p;while *p!=0{p=p.add(1);}p.offset_from(start)as usize}
unsafe fn strcmp(mut a:*const c_char,mut b:*const c_char)->i32{loop{let x=*a as u8;let y=*b as u8;if x!=y||x==0{return x as i32-y as i32;}a=a.add(1);b=b.add(1);}}
unsafe fn memcmp(a:*const c_void,b:*const c_void,n:usize)->i32 {let a=a.cast::<u8>();let b=b.cast::<u8>();for i in 0..n{let x=*a.add(i);let y=*b.add(i);if x!=y{return x as i32-y as i32;}}0}
unsafe fn strchr(mut p:*const c_char,c:i32)->*mut c_char{loop{if *p as u8==c as u8{return p as *mut c_char;}if *p==0{return ptr::null_mut();}p=p.add(1);}}
unsafe fn strspn(mut p:*const c_char,set:*const c_char)->usize{let start=p;while *p!=0&&!strchr(set,*p as u8 as i32).is_null(){p=p.add(1);}p.offset_from(start)as usize}
unsafe fn strstr(mut p:*const c_char,needle:*const c_char)->*mut c_char{let n=strlen(needle);if n==0{return p as*mut c_char;}let len=strlen(p);if n>len{return ptr::null_mut();}for _ in 0..=len-n{if memcmp(p.cast(),needle.cast(),n)==0{return p as*mut c_char;}p=p.add(1);}ptr::null_mut()}
fn isspace(c:i32)->i32{matches!(c,9..=13|32)as i32}
fn isxdigit(c:i32)->i32{((c>=48&&c<=57)||(c>=65&&c<=70)||(c>=97&&c<=102))as i32}
unsafe fn strtoul(mut s:*const c_char,end:*mut*mut c_char,base:i32)->usize{
 let original=s;while isspace(*s as u8 as i32)!=0{s=s.add(1);}let mut neg=false;if *s==45{neg=true;s=s.add(1);}else if *s==43{s=s.add(1);}
 let mut base=base as usize;if (base==0||base==16)&&*s==48&&matches!(*s.add(1)as u8,b'x'|b'X')&&isxdigit(*s.add(2)as u8 as i32)!=0{s=s.add(2);base=16;}if base==0{base=if *s==48{8}else{10};}
 let first=s;let mut n=0usize;let mut overflow=false;loop{let c=*s as u8;let d=match c{b'0'..=b'9'=>c-b'0',b'a'..=b'z'=>c-b'a'+10,b'A'..=b'Z'=>c-b'A'+10,_=>255}as usize;if d>=base{break;}match n.checked_mul(base).and_then(|n|n.checked_add(d)){Some(v)=>n=v,None=>overflow=true};s=s.add(1);}
 if !end.is_null(){*end=if s==first{original}else{s}as*mut c_char;}if overflow{usize::MAX}else if neg{n.wrapping_neg()}else{n}
}
const tool_stderr:*mut FILE=2usize as *mut FILE;
fn io_write(f:*mut FILE,bytes:&[u8])->i32{let result=unsafe{if f==tool_stderr{std::io::stderr().lock().write_all(bytes)}else if f.is_null(){std::io::stdout().lock().write_all(bytes)}else{match &mut(*f).kind{FileKind::Writer(out)=>out.write_all(bytes),FileKind::Reader(_)=>Err(std::io::Error::from(std::io::ErrorKind::Unsupported))}}};if result.is_ok(){bytes.len()as i32}else{-1}}
unsafe fn tool_printf(fmt:*const c_char,args:&[ToolPrintArg])->i32{io_write(ptr::null_mut(),&tool_format_print(fmt,args))}
unsafe fn tool_fprintf(f:*mut FILE,fmt:*const c_char,args:&[ToolPrintArg])->i32{io_write(f,&tool_format_print(fmt,args))}
unsafe fn tool_snprintf(dst:*mut c_char,size:usize,fmt:*const c_char,args:&[ToolPrintArg])->i32{let bytes=tool_format_print(fmt,args);if size>0{let n=bytes.len().min(size-1);ptr::copy_nonoverlapping(bytes.as_ptr(),dst.cast(),n);*dst.add(n)=0;}bytes.len()as i32}
unsafe fn fopen(path:*const c_char,mode:*const c_char)->*mut FILE{
 #[cfg(unix)] let path={use std::os::unix::ffi::OsStrExt;std::path::PathBuf::from(std::ffi::OsStr::from_bytes(CStr::from_ptr(path).to_bytes()))};#[cfg(not(unix))] let path=std::path::PathBuf::from(CStr::from_ptr(path).to_string_lossy().as_ref());let mode=CStr::from_ptr(mode).to_bytes();let mut opts=OpenOptions::new();let read=mode.first()==Some(&b'r');if read{opts.read(true);}else{opts.write(true).create(true).truncate(true);}match opts.open(path){Ok(f)=>Box::into_raw(Box::new(FILE{kind:if read{FileKind::Reader(BufReader::new(f))}else{FileKind::Writer(BufWriter::new(f))}})),Err(error)=>{LAST_IO_ERROR.with(|e|*e.borrow_mut()=Some(error));ptr::null_mut()}}
}
thread_local!{static LAST_IO_ERROR:std::cell::RefCell<Option<std::io::Error>>=const{std::cell::RefCell::new(None)};}
unsafe fn perror(path:*const c_char){let name=CStr::from_ptr(path).to_string_lossy();LAST_IO_ERROR.with(|e|{let error=e.borrow();let message=error.as_ref().map(|e|{let text=e.to_string();text.split(" (os error ").next().unwrap_or(&text).to_owned()}).unwrap_or_else(||"Success".into());let _=std::io::stderr().lock().write_all(format!("{name}: {message}\n").as_bytes());});}
unsafe fn fclose(f:*mut FILE)->i32{let mut f=Box::from_raw(f);match &mut f.kind{FileKind::Writer(out)=>if out.flush().is_ok(){0}else{-1},FileKind::Reader(_)=>0}}
unsafe fn fgets(dst:*mut c_char,n:i32,f:*mut FILE)->*mut c_char{if n<=0{return ptr::null_mut();}let FileKind::Reader(input)=&mut(*f).kind else{return ptr::null_mut();};let mut i=0;while i<(n-1)as usize{let mut b=[0];match input.read(&mut b){Ok(1)=>{*dst.add(i)=b[0]as c_char;i+=1;if b[0]==b'\n'{break;}},Ok(_)=>break,Err(error)=>{LAST_IO_ERROR.with(|e|*e.borrow_mut()=Some(error));return ptr::null_mut();}}}if i==0{return ptr::null_mut();}*dst.add(i)=0;dst}
unsafe fn tool_qsort(base:*mut c_void,n:usize,size:usize,compare:unsafe fn(*const c_void,*const c_void)->i32){unsafe fn cmp(a:*const c_void,b:*const c_void,ctx:*mut c_void)->i32{let f=*(ctx as*mut unsafe fn(*const c_void,*const c_void)->i32);f(a,b)}let mut compare=compare;quickjs::cutils::rqsort(base,n,size,cmp,ptr::addr_of_mut!(compare).cast());}
fn tool_exit(code:i32)->!{std::process::exit(code)}

fn tool_isctype(c:i32,mask:u32)->i32{let matched=(mask&0x4000!=0&&isspace(c)!=0)||(mask&0x10000!=0&&isxdigit(c)!=0);matched as i32}
#[cfg(feature="unicode-use-test")]
use quickjs::libunicode_header::{UnicodeNormalizationEnum,UNICODE_NFC,UNICODE_NFD,UNICODE_NFKC,UNICODE_NFKD};
#[cfg(feature="unicode-use-test")]
unsafe fn tool_lre_case_conv(res:*mut u32,c:u32,conv_type:i32)->i32 {quickjs::libunicode::lre_case_conv(core::slice::from_raw_parts_mut(res,3),c,conv_type)}
#[cfg(feature="unicode-use-test")]
unsafe fn tool_unicode_decomp_char(res:*mut u32,c:u32,is_compat:i32)->i32 {quickjs::libunicode::unicode_decomp_char(core::slice::from_raw_parts_mut(res,18),c,is_compat)}
#[cfg(feature="unicode-use-test")]
unsafe fn tool_unicode_compose_pair(c0:u32,c1:u32)->i32 {quickjs::libunicode::unicode_compose_pair(c0,c1) as i32}
#[cfg(feature="unicode-use-test")]
unsafe fn tool_unicode_get_cc(c:u32)->i32 {quickjs::libunicode::unicode_get_cc(c)}
#[cfg(feature="unicode-use-test")]
unsafe fn tool_unicode_normalize(dst:*mut *mut u32,src:*const u32,len:i32,kind:UnicodeNormalizationEnum,opaque:*mut c_void,realloc:Option<quickjs::cutils::DynBufReallocFunc>)->i32 {quickjs::libunicode::unicode_normalize(dst,src,len,kind,opaque,realloc)}
// c: unicode_gen.c:2511..2517. Rust Instant is the tool-layer monotonic clock.
#[cfg(feature="unicode-profile")]
fn get_time_ns()->i64 {static EPOCH:std::sync::OnceLock<std::time::Instant>=std::sync::OnceLock::new();EPOCH.get_or_init(std::time::Instant::now).elapsed().as_nanos()as i64}

#[cfg(feature="unicode-use-test")]
const UNICODE_DECOMP_LEN_MAX:usize=18;

// Host assertion rendering matches the official C generator on macOS.
unsafe fn tool_assert_rtn(function:*const c_char,file:*const c_char,line:i32,expression:*const c_char)->! {
 tool_fprintf(tool_stderr,c"Assertion failed: (%s), function %s, file %s, line %d.\n".as_ptr(),&[ToolPrintArg::Str(expression),ToolPrintArg::Str(function),ToolPrintArg::Str(file),ToolPrintArg::Int(line as u64)]);std::process::abort()
}
