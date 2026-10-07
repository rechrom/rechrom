// quickjs.c:10119-10125,10272-10300. MIT.
const JS_PROP_C_W_E: i32 = JS_PROP_CONFIGURABLE | JS_PROP_WRITABLE | JS_PROP_ENUMERABLE;
fn get_prop_flags(flags: i32, def_flags: i32) -> i32 {
    let mask = (flags >> JS_PROP_HAS_SHIFT) & JS_PROP_C_W_E;
    (flags & mask) | (def_flags & !mask)
}
fn check_define_prop_flags(prop_flags: i32, flags: i32) -> JS_BOOL {
    if prop_flags & JS_PROP_CONFIGURABLE == 0 {
        if flags & (JS_PROP_HAS_CONFIGURABLE | JS_PROP_CONFIGURABLE)
            == (JS_PROP_HAS_CONFIGURABLE | JS_PROP_CONFIGURABLE)
        {
            return 0;
        }
        if flags & JS_PROP_HAS_ENUMERABLE != 0
            && flags & JS_PROP_ENUMERABLE != prop_flags & JS_PROP_ENUMERABLE
        {
            return 0;
        }
        if flags & (JS_PROP_HAS_VALUE | JS_PROP_HAS_WRITABLE | JS_PROP_HAS_GET | JS_PROP_HAS_SET)
            != 0
        {
            let has_accessor = flags & (JS_PROP_HAS_GET | JS_PROP_HAS_SET) != 0;
            let is_getset = prop_flags & JS_PROP_TMASK == JS_PROP_GETSET;
            if has_accessor != is_getset {
                return 0;
            }
            if !is_getset
                && prop_flags & JS_PROP_WRITABLE == 0
                && flags & (JS_PROP_HAS_WRITABLE | JS_PROP_WRITABLE)
                    == (JS_PROP_HAS_WRITABLE | JS_PROP_WRITABLE)
            {
                return 0;
            }
        }
    }
    1
}
