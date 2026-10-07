// C library calls occurring in the official parser/compiler. Pure Rust.
unsafe fn parser_strlen(mut p: *const c_char) -> usize {
    let start = p;
    while *p != 0 { p = p.add(1); }
    p.offset_from(start) as usize
}
unsafe fn parser_strcmp(mut a: *const c_char, mut b: *const c_char) -> i32 {
    loop {
        let x = *a as u8; let y = *b as u8;
        if x != y || x == 0 { return i32::from(x) - i32::from(y); }
        a = a.add(1); b = b.add(1);
    }
}
unsafe fn parser_strrchr(mut s: *const c_char, c: i32) -> *mut c_char {
    let mut result = core::ptr::null_mut();
    loop { if *s as u8 == c as u8 { result = s as *mut c_char; }
        if *s == 0 { return result; } s = s.add(1); }
}
// snprintf(dst, cap, "%.*s", len, src) for raw source directives.
unsafe fn parser_copy_directive(dst: *mut c_char, cap: usize, src: *const c_char, len: i32) -> i32 {
    let mut n = 0usize;
    while (len < 0 || n < len as usize) && *src.add(n) != 0 { n += 1; }
    if cap != 0 { let copied = n.min(cap - 1); core::ptr::copy_nonoverlapping(src, dst, copied); *dst.add(copied) = 0; }
    n as i32
}
