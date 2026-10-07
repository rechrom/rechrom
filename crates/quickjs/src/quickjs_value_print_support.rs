// c: quickjs.c:13680..13713. Bellard/Gordon MIT.
// A typed call adapter replaces C variadic promotion; formatting remains byte based.
const JS_PRINT_MAX_DEPTH: u32 = 8;
#[repr(C)]
struct JSPrintValueState {
    rt: *mut JSRuntime,
    ctx: *mut JSContext,
    options: JSPrintValueOptions,
    write_func: Option<JSPrintValueWrite>,
    write_opaque: *mut c_void,
    level: i32,
    print_stack: [*mut JSObject; JS_PRINT_MAX_DEPTH as usize],
}
enum QuickJSPrintArg { Float(f64), Int(u64), Str(*const c_char), Ptr(*const c_void) }
unsafe fn quickjs_format_print(fmt: *const c_char, args: &[QuickJSPrintArg]) -> Vec<u8> {
    let fmt = std::ffi::CStr::from_ptr(fmt).to_bytes();
    let mut out=Vec::new(); let mut i=0; let mut arg=0;
    while i<fmt.len() {
        if fmt[i]!=b'%' {out.push(fmt[i]);i+=1;continue;} i+=1;
        if fmt[i]==b'%' {out.push(b'%');i+=1;continue;}
        let mut zero=false;let mut left=false;
        loop {match fmt[i] {b'0'=>zero=true,b'-'=>left=true,_=>break} i+=1;}
        let width=if fmt[i]==b'*' {i+=1;let QuickJSPrintArg::Int(n)=args[arg] else {unreachable!()};arg+=1;n as i32} else {let mut n=0;while fmt[i].is_ascii_digit(){n=n*10+(fmt[i]-b'0') as i32;i+=1;}n};
        let mut precision=None;
        if fmt[i]==b'.' {i+=1;let n=if fmt[i]==b'*' {i+=1;let QuickJSPrintArg::Int(n)=args[arg] else {unreachable!()};arg+=1;n as i32} else {let mut n=0;while fmt[i].is_ascii_digit(){n=n*10+(fmt[i]-b'0') as i32;i+=1;}n};if n>=0 {precision=Some(n as usize)}}
        let mut longs=0;while fmt[i]==b'l' {longs+=1;i+=1;}
        let spec=fmt[i];i+=1;let a=&args[arg];arg+=1;
        let mut bytes=match (spec,a) {
            (b's',QuickJSPrintArg::Str(p))=>{let b=std::ffi::CStr::from_ptr(*p).to_bytes();b[..precision.unwrap_or(b.len()).min(b.len())].to_vec()},
            (b'c',QuickJSPrintArg::Int(n))=>vec![*n as u8],
            (b'd'|b'i',QuickJSPrintArg::Int(n))=>if longs>0 {(*n as i64).to_string().into_bytes()} else {(*n as i32).to_string().into_bytes()},
            (b'u',QuickJSPrintArg::Int(n))=>if longs>0 {n.to_string().into_bytes()} else {(*n as u32).to_string().into_bytes()},
            (b'x'|b'X',QuickJSPrintArg::Int(n))=>{let n=if longs>0 {*n} else {*n as u32 as u64};if spec==b'x' {format!("{n:x}").into_bytes()} else {format!("{n:X}").into_bytes()}},
            (b'g',QuickJSPrintArg::Float(n))=>quickjs_print_general(*n,precision.unwrap_or(6).max(1)),
            (b'f',QuickJSPrintArg::Float(n))=>format!("{:.*}",precision.unwrap_or(6),n).to_lowercase().into_bytes(),
            (b'p',QuickJSPrintArg::Ptr(p))=>format!("0x{:x}",*p as usize).into_bytes(),
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
// c: quickjs.c:13700..13712. The original bounded 256-byte js_printf buffer.
unsafe fn js_printf(s:*mut JSPrintValueState, bytes:&[u8]) {
    let end=bytes.len().min(255);let end=bytes[..end].iter().position(|c|*c==0).unwrap_or(end);
    ((*s).write_func.expect("original non-null print writer"))((*s).write_opaque,bytes.as_ptr().cast(),end);
}
// c: quickjs.c:14446..14450. Host output adapter, independent of platform FILE layout.
unsafe fn js_dump_value_write(_opaque:*mut c_void,buf:*const c_char,len:usize) {
    use std::io::Write;
    let _=std::io::stdout().lock().write_all(core::slice::from_raw_parts(buf.cast::<u8>(),len));
}
fn quickjs_debug_write(bytes:&[u8])->i32 {
    use std::io::Write;
    match std::io::stdout().lock().write_all(bytes) {Ok(())=>bytes.len() as i32,Err(_)=>-1}
}

// Original DUMP_BYTECODE opcode names; SHORT_OPCODES uses the same original index map.
unsafe fn debug_opcode_name(mut op:usize,short:bool)->*const c_char {
#[cfg(feature="short-opcodes")]
if short && op>=OP_TEMP_START as usize {op+=(OP_TEMP_END-OP_TEMP_START) as usize;}
static NAMES:&[&core::ffi::CStr]=&[
c"invalid",
c"push_i32",
c"push_const",
c"fclosure",
c"push_atom_value",
c"private_symbol",
c"undefined",
c"null",
c"push_this",
c"push_false",
c"push_true",
c"object",
c"special_object",
c"rest",
c"drop",
c"nip",
c"nip1",
c"dup",
c"dup1",
c"dup2",
c"dup3",
c"insert2",
c"insert3",
c"insert4",
c"perm3",
c"perm4",
c"perm5",
c"swap",
c"swap2",
c"rot3l",
c"rot3r",
c"rot4l",
c"rot5l",
c"call_constructor",
c"call",
c"tail_call",
c"call_method",
c"tail_call_method",
c"array_from",
c"apply",
c"return",
c"return_undef",
c"check_ctor_return",
c"check_ctor",
c"init_ctor",
c"check_brand",
c"add_brand",
c"return_async",
c"throw",
c"throw_error",
c"eval",
c"apply_eval",
c"regexp",
c"get_super",
c"import",
c"get_var_undef",
c"get_var",
c"put_var",
c"put_var_init",
c"get_ref_value",
c"put_ref_value",
c"get_field",
c"get_field2",
c"put_field",
c"get_private_field",
c"put_private_field",
c"define_private_field",
c"get_array_el",
c"get_array_el2",
c"get_array_el3",
c"put_array_el",
c"get_super_value",
c"put_super_value",
c"define_field",
c"set_name",
c"set_name_computed",
c"set_proto",
c"set_home_object",
c"define_array_el",
c"append",
c"copy_data_properties",
c"define_method",
c"define_method_computed",
c"define_class",
c"define_class_computed",
c"get_loc",
c"put_loc",
c"set_loc",
c"get_arg",
c"put_arg",
c"set_arg",
c"get_var_ref",
c"put_var_ref",
c"set_var_ref",
c"set_loc_uninitialized",
c"get_loc_check",
c"put_loc_check",
c"set_loc_check",
c"put_loc_check_init",
c"get_loc_checkthis",
c"get_var_ref_check",
c"put_var_ref_check",
c"put_var_ref_check_init",
c"close_loc",
c"if_false",
c"if_true",
c"goto",
c"catch",
c"gosub",
c"ret",
c"nip_catch",
c"to_object",
c"to_propkey",
c"with_get_var",
c"with_put_var",
c"with_delete_var",
c"with_make_ref",
c"with_get_ref",
c"make_loc_ref",
c"make_arg_ref",
c"make_var_ref_ref",
c"make_var_ref",
c"for_in_start",
c"for_of_start",
c"for_await_of_start",
c"for_in_next",
c"for_of_next",
c"for_await_of_next",
c"iterator_check_object",
c"iterator_get_value_done",
c"iterator_close",
c"iterator_next",
c"iterator_call",
c"initial_yield",
c"yield",
c"yield_star",
c"async_yield_star",
c"await",
c"neg",
c"plus",
c"dec",
c"inc",
c"post_dec",
c"post_inc",
c"dec_loc",
c"inc_loc",
c"add_loc",
c"not",
c"lnot",
c"typeof",
c"delete",
c"delete_var",
c"mul",
c"div",
c"mod",
c"add",
c"sub",
c"pow",
c"shl",
c"sar",
c"shr",
c"lt",
c"lte",
c"gt",
c"gte",
c"instanceof",
c"in",
c"eq",
c"neq",
c"strict_eq",
c"strict_neq",
c"and",
c"xor",
c"or",
c"is_undefined_or_null",
c"private_in",
c"push_bigint_i32",
c"nop",
c"enter_scope",
c"leave_scope",
c"label",
c"scope_get_var_undef",
c"scope_get_var",
c"scope_put_var",
c"scope_delete_var",
c"scope_make_ref",
c"scope_get_ref",
c"scope_put_var_init",
c"scope_get_var_checkthis",
c"scope_get_private_field",
c"scope_get_private_field2",
c"scope_put_private_field",
c"scope_in_private_field",
c"get_field_opt_chain",
c"get_array_el_opt_chain",
c"set_class_name",
c"line_num",
c"push_minus1",
c"push_0",
c"push_1",
c"push_2",
c"push_3",
c"push_4",
c"push_5",
c"push_6",
c"push_7",
c"push_i8",
c"push_i16",
c"push_const8",
c"fclosure8",
c"push_empty_string",
c"get_loc8",
c"put_loc8",
c"set_loc8",
c"get_loc0",
c"get_loc1",
c"get_loc2",
c"get_loc3",
c"put_loc0",
c"put_loc1",
c"put_loc2",
c"put_loc3",
c"set_loc0",
c"set_loc1",
c"set_loc2",
c"set_loc3",
c"get_arg0",
c"get_arg1",
c"get_arg2",
c"get_arg3",
c"put_arg0",
c"put_arg1",
c"put_arg2",
c"put_arg3",
c"set_arg0",
c"set_arg1",
c"set_arg2",
c"set_arg3",
c"get_var_ref0",
c"get_var_ref1",
c"get_var_ref2",
c"get_var_ref3",
c"put_var_ref0",
c"put_var_ref1",
c"put_var_ref2",
c"put_var_ref3",
c"set_var_ref0",
c"set_var_ref1",
c"set_var_ref2",
c"set_var_ref3",
c"get_length",
c"if_false8",
c"if_true8",
c"goto8",
c"goto16",
c"call0",
c"call1",
c"call2",
c"call3",
c"is_undefined",
c"is_null",
c"typeof_is_undefined",
c"typeof_is_function",
];
NAMES[op].as_ptr()
}

// Explicit host stream bridge for C FILE-taking diagnostics. Engine algorithms
// receive only this writer contract, without depending on a platform FILE ABI.
#[repr(C)]
pub struct QuickJSPrintStream {
    pub write_func: unsafe fn(*mut c_void,*const c_char,usize)->usize,
    pub write_opaque: *mut c_void,
}
unsafe fn quickjs_stream_write(stream:*mut QuickJSPrintStream,bytes:&[u8])->i32 {
    if stream.is_null() {return quickjs_debug_write(bytes);}
    let written=((*stream).write_func)((*stream).write_opaque,bytes.as_ptr().cast(),bytes.len());
    if written<bytes.len() {-1} else {bytes.len() as i32}
}
unsafe fn quickjs_stream_putc(c:i32,stream:*mut QuickJSPrintStream)->i32 {
    if quickjs_stream_write(stream,&[c as u8])<0 {-1} else {c as u8 as i32}
}
#[repr(C)]
#[derive(Clone,Copy)]
struct QuickJSMemoryObjectType { name:*const c_char,size:usize }
type JSMallocIterFunc=unsafe fn(*mut c_void,*mut c_void);

// C printf %g: round significant digits, choose exponent at -4/precision,
// remove insignificant trailing zeroes, and print a signed 2-digit exponent.
fn quickjs_print_general(n:f64,precision:usize)->Vec<u8> {
 if !n.is_finite(){return if n.is_nan(){b"nan".to_vec()}else if n.is_sign_negative(){b"-inf".to_vec()}else{b"inf".to_vec()};}
 let text=format!("{:.*e}",precision-1,n.abs());let (mant,exp)=text.split_once('e').unwrap();let exp:i32=exp.parse().unwrap();let mut significant=mant.bytes().filter(|b|*b!=b'.').collect::<Vec<_>>();while significant.len()>1&&significant.last()==Some(&b'0'){significant.pop();}
 let mut out=Vec::new();if n.is_sign_negative(){out.push(b'-');}
 if exp < -4 || exp >= precision as i32 {out.push(significant[0]);if significant.len()>1{out.push(b'.');out.extend_from_slice(&significant[1..]);}out.push(b'e');out.push(if exp<0{b'-'}else{b'+'});out.extend_from_slice(format!("{:02}",exp.unsigned_abs()).as_bytes());}
 else {let point=exp+1;if point<=0{out.extend_from_slice(b"0.");out.resize(out.len()+(-point)as usize,b'0');out.extend_from_slice(&significant);}else if point as usize>=significant.len(){out.extend_from_slice(&significant);out.resize(out.len()+point as usize-significant.len(),b'0');}else{out.extend_from_slice(&significant[..point as usize]);out.push(b'.');out.extend_from_slice(&significant[point as usize..]);}}
 out
}
