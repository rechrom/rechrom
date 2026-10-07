//! Extract unchanged C sections for independent test executables. No runtime FFI.
pub fn function<'a>(source: &'a str, name: &str) -> &'a str {
    let needle = format!("{name}(");
    let mut begin = 0;
    while let Some(rel) = source[begin..].find(&needle) {
        let name_at = begin + rel;
        if name_at != 0
            && (source.as_bytes()[name_at - 1].is_ascii_alphanumeric()
                || source.as_bytes()[name_at - 1] == b'_')
        {
            begin = name_at + needle.len();
            continue;
        }
        let line = source[..name_at].rfind('\n').map_or(0, |n| n + 1);
        if source[line..name_at].contains(';')
            || source[line..name_at].contains("return")
            || source[line..name_at].contains('=')
        {
            begin = name_at + needle.len();
            continue;
        }
        let Some(after) = source[name_at..].find(')') else {
            panic!("missing {name} arguments");
        };
        let args_end = name_at + after + 1;
        let mut body = args_end;
        while source.as_bytes()[body].is_ascii_whitespace() {
            body += 1;
        }
        if source.as_bytes()[body] != b'{' {
            begin = args_end;
            continue;
        }
        let bytes = source.as_bytes();
        let mut p = body;
        let mut depth = 0;
        let mut state = 0u8;
        loop {
            let c = bytes[p];
            let next = bytes.get(p + 1).copied().unwrap_or(0);
            match state {
                0 => {
                    if c == b'/' && next == b'/' {
                        state = 1;
                        p += 1;
                    } else if c == b'/' && next == b'*' {
                        state = 2;
                        p += 1;
                    } else if c == b'"' {
                        state = 3;
                    } else if c == b'\'' {
                        state = 4;
                    } else if c == b'{' {
                        depth += 1;
                    } else if c == b'}' {
                        depth -= 1;
                        if depth == 0 {
                            return &source[line..=p];
                        }
                    }
                }
                1 => {
                    if c == b'\n' {
                        state = 0;
                    }
                }
                2 => {
                    if c == b'*' && next == b'/' {
                        state = 0;
                        p += 1;
                    }
                }
                3 | 4 => {
                    if c == b'\\' {
                        p += 1;
                    } else if (state == 3 && c == b'"') || (state == 4 && c == b'\'') {
                        state = 0;
                    }
                }
                _ => unreachable!(),
            }
            p += 1;
        }
    }
    panic!("missing C function definition {name}");
}
pub fn source_prefix(source: &str) -> String {
    let start = source.find("enum {\n    /* classid").unwrap();
    let end = source[start..].find("/* JS malloc */").unwrap() + start;
    let alloc_start = source.find("#define JS_MALLOC_ALIGN").unwrap();
    let alloc_end = source[alloc_start..].find("/* end JS Malloc */").unwrap() + alloc_start;
    let rt_start = source.find("struct JSRuntime {").unwrap();
    let rt_end = source[rt_start..]
        .find("typedef enum {\n    JS_CLOSURE_LOCAL")
        .unwrap()
        + rt_start;
    let atoms_start = source.find("enum {\n    __JS_ATOM_NULL").unwrap();
    let atoms_end = source[atoms_start..]
        .find("typedef enum OPCodeFormat")
        .unwrap()
        + atoms_start;
    let atom_flags = source.find("#define JS_ATOM_TAG_INT").unwrap();
    let atom_flags_end = source[atom_flags..]
        .find("static inline BOOL __JS_AtomIsConst")
        .unwrap()
        + atom_flags;
    let malloc_start = source
        .find("static const uint16_t js_malloc_block_sizes")
        .unwrap();
    let malloc_end = source[malloc_start..]
        .find("static __maybe_unused void js_malloc_dump_arenas")
        .unwrap()
        + malloc_start;
    format!("#include <stdlib.h>\n#include <stdio.h>\n#include <math.h>\n#include <stddef.h>\n#include <assert.h>\n#include \"cutils.h\"\n#include \"list.h\"\n#include \"quickjs.h\"\n{}\n{}\n{}\n{}\n{}\n{}\n",&source[start..end],&source[alloc_start..alloc_end],&source[rt_start..rt_end],&source[atoms_start..atoms_end],&source[atom_flags..atom_flags_end],&source[malloc_start..malloc_end])
}
pub fn append_functions(out: &mut String, source: &str, names: &[&str]) {
    let bodies: Vec<_> = names.iter().map(|n| function(source, n)).collect();
    for body in &bodies {
        out.push_str(body.split_once('{').unwrap().0);
        out.push_str(";\n");
    }
    for body in bodies {
        out.push_str(body);
        out.push('\n');
    }
}
