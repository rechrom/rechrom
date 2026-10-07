// quickjs.c shape/property layouts and hash table algorithms. MIT.
#[repr(C)]
#[derive(Clone, Copy)]
struct JSPropertyGetSet {
    getter: *mut JSObject,
    setter: *mut JSObject,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct JSPropertyInit {
    realm_and_id: usize,
    opaque: *mut c_void,
}
#[repr(C)]
union JSPropertyUnion {
    value: JSValue,
    getset: JSPropertyGetSet,
    var_ref: *mut JSVarRef,
    init: JSPropertyInit,
}
#[repr(C)]
struct JSProperty {
    u: JSPropertyUnion,
}
const JS_PROP_INITIAL_SIZE: i32 = 2;
const JS_PROP_INITIAL_HASH_SIZE: i32 = 4;
#[repr(C)]
struct JSShapeProperty {
    hash_next_and_flags: u32,
    atom: JSAtom,
}
impl JSShapeProperty {
    fn hash_next(&self) -> u32 {
        self.hash_next_and_flags & 0x3ffffff
    }
    fn set_hash_next(&mut self, n: u32) {
        self.hash_next_and_flags = (self.hash_next_and_flags & 0xfc000000) | (n & 0x3ffffff);
    }
    fn flags(&self) -> u32 {
        self.hash_next_and_flags >> 26
    }
    fn set_flags(&mut self, n: u32) {
        self.hash_next_and_flags = (self.hash_next_and_flags & 0x3ffffff) | (n << 26);
    }
}
#[repr(C)]
struct JSShape {
    header: JSGCObjectHeader,
    is_hashed: u8,
    hash: u32,
    prop_hash_mask: u32,
    prop_size: i32,
    prop_count: i32,
    deleted_prop_count: i32,
    shape_hash_next: *mut JSShape,
    proto: *mut JSObject,
    hash_table: [u32; 0],
}
fn get_shape_size(hash_size: usize, prop_size: usize) -> usize {
    size_of::<JSShape>()
        .wrapping_add(hash_size.wrapping_mul(size_of::<u32>()))
        .wrapping_add(prop_size.wrapping_mul(size_of::<JSShapeProperty>()))
}
unsafe fn get_shape_prop(sh: *mut JSShape) -> *mut JSShapeProperty {
    sh.add(1)
        .cast::<u32>()
        .add((*sh).prop_hash_mask as usize + 1)
        .cast()
}
unsafe fn init_shape_hash(rt: *mut JSRuntime) -> i32 {
    (*rt).shape_hash_bits = 4;
    (*rt).shape_hash_size = 1 << (*rt).shape_hash_bits;
    (*rt).shape_hash_count = 0;
    (*rt).shape_hash = js_mallocz_rt(
        rt,
        size_of::<*mut JSShape>() * (*rt).shape_hash_size as usize,
    )
    .cast();
    if (*rt).shape_hash.is_null() {
        -1
    } else {
        0
    }
}
fn shape_hash(h: u32, val: u32) -> u32 {
    h.wrapping_add(val).wrapping_mul(0x9e370001)
}
fn get_shape_hash(h: u32, hash_bits: i32) -> u32 {
    h >> (32 - hash_bits)
}
fn shape_initial_hash(proto: *mut JSObject) -> u32 {
    let mut h = shape_hash(1, proto as usize as u32);
    if size_of::<*mut JSObject>() > 4 {
        h = shape_hash(h, ((proto as usize as u64) >> 32) as u32);
    }
    h
}
unsafe fn resize_shape_hash(rt: *mut JSRuntime, new_shape_hash_bits: i32) -> i32 {
    let new_shape_hash_size = 1 << new_shape_hash_bits;
    let new_shape_hash =
        js_mallocz_rt(rt, size_of::<*mut JSShape>() * new_shape_hash_size as usize)
            .cast::<*mut JSShape>();
    if new_shape_hash.is_null() {
        return -1;
    }
    for i in 0..(*rt).shape_hash_size {
        let mut sh = *(*rt).shape_hash.add(i as usize);
        while !sh.is_null() {
            let sh_next = (*sh).shape_hash_next;
            let h = get_shape_hash((*sh).hash, new_shape_hash_bits);
            (*sh).shape_hash_next = *new_shape_hash.add(h as usize);
            *new_shape_hash.add(h as usize) = sh;
            sh = sh_next;
        }
    }
    js_free_rt(rt, (*rt).shape_hash.cast());
    (*rt).shape_hash_bits = new_shape_hash_bits;
    (*rt).shape_hash_size = new_shape_hash_size;
    (*rt).shape_hash = new_shape_hash;
    0
}
unsafe fn js_shape_hash_link(rt: *mut JSRuntime, sh: *mut JSShape) {
    let h = get_shape_hash((*sh).hash, (*rt).shape_hash_bits);
    (*sh).shape_hash_next = *(*rt).shape_hash.add(h as usize);
    *(*rt).shape_hash.add(h as usize) = sh;
    (*rt).shape_hash_count += 1;
}
unsafe fn js_shape_hash_unlink(rt: *mut JSRuntime, sh: *mut JSShape) {
    let h = get_shape_hash((*sh).hash, (*rt).shape_hash_bits);
    let mut psh = (*rt).shape_hash.add(h as usize);
    while *psh != sh {
        psh = ptr::addr_of_mut!((**psh).shape_hash_next);
    }
    *psh = (*sh).shape_hash_next;
    (*rt).shape_hash_count -= 1;
}
unsafe fn js_dup_shape(sh: *mut JSShape) -> *mut JSShape {
    (*js_rc(sh.cast())).ref_count += 1;
    sh
}
unsafe fn find_hashed_shape_proto(rt: *mut JSRuntime, proto: *mut JSObject) -> *mut JSShape {
    let h = shape_initial_hash(proto);
    let h1 = get_shape_hash(h, (*rt).shape_hash_bits);
    let mut sh1 = *(*rt).shape_hash.add(h1 as usize);
    while !sh1.is_null() {
        if (*sh1).hash == h && (*sh1).proto == proto && (*sh1).prop_count == 0 {
            return sh1;
        }
        sh1 = (*sh1).shape_hash_next;
    }
    ptr::null_mut()
}
unsafe fn find_hashed_shape_prop(
    rt: *mut JSRuntime,
    sh: *mut JSShape,
    atom: JSAtom,
    prop_flags: i32,
) -> *mut JSShape {
    let h = shape_hash(shape_hash((*sh).hash, atom), prop_flags as u32);
    let h1 = get_shape_hash(h, (*rt).shape_hash_bits);
    let mut sh1 = *(*rt).shape_hash.add(h1 as usize);
    while !sh1.is_null() {
        let n = (*sh).prop_count;
        if (*sh1).hash == h && (*sh1).proto == (*sh).proto && (*sh1).prop_count == n + 1 {
            let prop = get_shape_prop(sh);
            let prop1 = get_shape_prop(sh1);
            let mut matched = true;
            for i in 0..n as usize {
                if (*prop1.add(i)).atom != (*prop.add(i)).atom
                    || (*prop1.add(i)).flags() != (*prop.add(i)).flags()
                {
                    matched = false;
                    break;
                }
            }
            if matched
                && (*prop1.add(n as usize)).atom == atom
                && (*prop1.add(n as usize)).flags() as i32 == prop_flags
            {
                return sh1;
            }
        }
        sh1 = (*sh1).shape_hash_next;
    }
    ptr::null_mut()
}
