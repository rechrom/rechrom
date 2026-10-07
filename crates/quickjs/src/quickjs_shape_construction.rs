// quickjs.c:5214-5292,5334-5509,9179-9246. MIT.
// Awaiting production context allocation and exception integration.
#[inline]
unsafe fn js_new_shape_nohash(
    ctx: *mut JSContext,
    proto: *mut JSObject,
    hash_size: i32,
    prop_size: i32,
) -> *mut JSShape {
    let rt = (*ctx).rt;
    let sh =
        js_malloc(ctx, get_shape_size(hash_size as usize, prop_size as usize)).cast::<JSShape>();
    if sh.is_null() {
        return ptr::null_mut();
    }
    (*js_rc(sh.cast())).ref_count = 1;
    add_gc_object(rt, &mut (*sh).header, JS_GC_OBJ_TYPE_SHAPE);
    if !proto.is_null() {
        JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, proto.cast()));
    }
    (*sh).proto = proto;
    ptr::write_bytes(
        ptr::addr_of_mut!((*sh).hash_table).cast::<u32>(),
        0,
        hash_size as usize,
    );
    (*sh).prop_hash_mask = hash_size.wrapping_sub(1) as u32;
    (*sh).prop_size = prop_size;
    (*sh).prop_count = 0;
    (*sh).deleted_prop_count = 0;
    (*sh).is_hashed = 0;
    sh
}
#[inline(never)]
unsafe fn js_new_shape2(
    ctx: *mut JSContext,
    proto: *mut JSObject,
    hash_size: i32,
    prop_size: i32,
) -> *mut JSShape {
    let rt = (*ctx).rt;
    if 2i32.wrapping_mul((*rt).shape_hash_count.wrapping_add(1)) > (*rt).shape_hash_size {
        resize_shape_hash(rt, (*rt).shape_hash_bits + 1);
    }
    let sh = js_new_shape_nohash(ctx, proto, hash_size, prop_size);
    if sh.is_null() {
        return ptr::null_mut();
    }
    (*sh).hash = shape_initial_hash(proto);
    (*sh).is_hashed = 1;
    js_shape_hash_link((*ctx).rt, sh);
    sh
}
unsafe fn js_new_shape(ctx: *mut JSContext, proto: *mut JSObject) -> *mut JSShape {
    js_new_shape2(ctx, proto, JS_PROP_INITIAL_HASH_SIZE, JS_PROP_INITIAL_SIZE)
}
unsafe fn js_clone_shape(ctx: *mut JSContext, sh1: *mut JSShape) -> *mut JSShape {
    let hash_size = (*sh1).prop_hash_mask.wrapping_add(1);
    let size = get_shape_size(hash_size as usize, (*sh1).prop_size as usize);
    let sh = js_malloc(ctx, size).cast::<JSShape>();
    if sh.is_null() {
        return ptr::null_mut();
    }
    ptr::copy_nonoverlapping(
        sh1.cast::<u8>().add(size_of::<JSGCObjectHeader>()),
        sh.cast::<u8>().add(size_of::<JSGCObjectHeader>()),
        size - size_of::<JSGCObjectHeader>(),
    );
    (*js_rc(sh.cast())).ref_count = 1;
    add_gc_object((*ctx).rt, &mut (*sh).header, JS_GC_OBJ_TYPE_SHAPE);
    (*sh).is_hashed = 0;
    if !(*sh).proto.is_null() {
        JS_DupValue(ctx, JS_MKPTR(JS_TAG_OBJECT, (*sh).proto.cast()));
    }
    let mut i = 0;
    let mut pr = get_shape_prop(sh);
    while i < (*sh).prop_count as u32 {
        JS_DupAtom(ctx, (*pr).atom);
        pr = pr.add(1);
        i += 1;
    }
    sh
}
#[inline(never)]
unsafe fn resize_properties(
    ctx: *mut JSContext,
    psh: *mut *mut JSShape,
    p: *mut JSObject,
    count: u32,
) -> i32 {
    let old_sh = *psh;
    let new_size = (count as i32).max((*old_sh).prop_size.wrapping_mul(3) / 2) as u32;
    if !p.is_null() {
        let new_prop = js_realloc(
            ctx,
            (*p).prop.cast(),
            size_of::<JSProperty>() * new_size as usize,
        )
        .cast::<JSProperty>();
        if new_prop.is_null() {
            return -1;
        }
        (*p).prop = new_prop;
    }
    let mut new_hash_size = (*old_sh).prop_hash_mask.wrapping_add(1);
    while new_hash_size < new_size {
        new_hash_size = new_hash_size.wrapping_mul(2);
    }
    let sh = js_malloc(
        ctx,
        get_shape_size(new_hash_size as usize, new_size as usize),
    )
    .cast::<JSShape>();
    if sh.is_null() {
        return -1;
    }
    remove_gc_object(&mut (*old_sh).header);
    (*js_rc(sh.cast())).ref_count = 1;
    add_gc_object((*ctx).rt, &mut (*sh).header, JS_GC_OBJ_TYPE_SHAPE);
    ptr::copy_nonoverlapping(
        old_sh.cast::<u8>().add(size_of::<JSGCObjectHeader>()),
        sh.cast::<u8>().add(size_of::<JSGCObjectHeader>()),
        size_of::<JSShape>() - size_of::<JSGCObjectHeader>(),
    );
    if new_hash_size != (*sh).prop_hash_mask.wrapping_add(1) {
        let new_hash_mask = new_hash_size - 1;
        (*sh).prop_hash_mask = new_hash_mask;
        let hash = ptr::addr_of_mut!((*sh).hash_table).cast::<u32>();
        ptr::write_bytes(hash, 0, new_hash_size as usize);
        ptr::copy_nonoverlapping(
            get_shape_prop(old_sh),
            get_shape_prop(sh),
            (*old_sh).prop_count as usize,
        );
        let mut i = 0;
        let mut pr = get_shape_prop(sh);
        while i < (*sh).prop_count as u32 {
            if (*pr).atom != JS_ATOM_NULL as u32 {
                let h = ((*pr).atom & new_hash_mask) as usize;
                (*pr).set_hash_next(*hash.add(h));
                *hash.add(h) = i + 1;
            }
            pr = pr.add(1);
            i += 1;
        }
    } else {
        ptr::copy_nonoverlapping(
            ptr::addr_of!((*old_sh).hash_table).cast::<u32>(),
            ptr::addr_of_mut!((*sh).hash_table).cast::<u32>(),
            new_hash_size as usize,
        );
        ptr::copy_nonoverlapping(
            get_shape_prop(old_sh),
            get_shape_prop(sh),
            (*old_sh).prop_count as usize,
        );
    }
    js_free(ctx, old_sh.cast());
    *psh = sh;
    (*sh).prop_size = new_size as i32;
    0
}
unsafe fn compact_properties(ctx: *mut JSContext, p: *mut JSObject) -> i32 {
    let old_sh = (*p).shape;
    assert!((*old_sh).is_hashed == 0);
    let new_size =
        JS_PROP_INITIAL_SIZE.max((*old_sh).prop_count - (*old_sh).deleted_prop_count) as u32;
    assert!(new_size <= (*old_sh).prop_size as u32);
    let mut new_hash_size = (*old_sh).prop_hash_mask + 1;
    while new_hash_size / 2 >= new_size {
        new_hash_size /= 2;
    }
    let new_hash_mask = new_hash_size - 1;
    let sh = js_malloc(
        ctx,
        get_shape_size(new_hash_size as usize, new_size as usize),
    )
    .cast::<JSShape>();
    if sh.is_null() {
        return -1;
    }
    remove_gc_object(&mut (*old_sh).header);
    (*js_rc(sh.cast())).ref_count = 1;
    add_gc_object((*ctx).rt, &mut (*sh).header, JS_GC_OBJ_TYPE_SHAPE);
    ptr::copy_nonoverlapping(
        old_sh.cast::<u8>().add(size_of::<JSGCObjectHeader>()),
        sh.cast::<u8>().add(size_of::<JSGCObjectHeader>()),
        size_of::<JSShape>() - size_of::<JSGCObjectHeader>(),
    );
    let hash = ptr::addr_of_mut!((*sh).hash_table).cast::<u32>();
    ptr::write_bytes(hash, 0, new_hash_size as usize);
    (*sh).prop_hash_mask = new_hash_mask;
    let mut j = 0u32;
    let mut i = 0u32;
    let mut old_pr = get_shape_prop(old_sh);
    let mut pr = get_shape_prop(sh);
    let prop = (*p).prop;
    while i < (*sh).prop_count as u32 {
        if (*old_pr).atom != JS_ATOM_NULL as u32 {
            (*pr).atom = (*old_pr).atom;
            (*pr).set_flags((*old_pr).flags());
            let h = ((*old_pr).atom & new_hash_mask) as usize;
            (*pr).set_hash_next(*hash.add(h));
            *hash.add(h) = j + 1;
            ptr::copy(prop.add(i as usize), prop.add(j as usize), 1);
            j += 1;
            pr = pr.add(1);
        }
        old_pr = old_pr.add(1);
        i += 1;
    }
    assert!(j == ((*sh).prop_count - (*sh).deleted_prop_count) as u32);
    (*sh).prop_size = new_size as i32;
    (*sh).deleted_prop_count = 0;
    (*sh).prop_count = j as i32;
    (*p).shape = sh;
    js_free(ctx, old_sh.cast());
    let new_prop = js_realloc(
        ctx,
        (*p).prop.cast(),
        size_of::<JSProperty>() * new_size as usize,
    )
    .cast::<JSProperty>();
    if !new_prop.is_null() {
        (*p).prop = new_prop;
    }
    0
}
unsafe fn add_shape_property(
    ctx: *mut JSContext,
    psh: *mut *mut JSShape,
    p: *mut JSObject,
    atom: JSAtom,
    prop_flags: i32,
) -> i32 {
    let rt = (*ctx).rt;
    let mut sh = *psh;
    let mut new_shape_hash = 0;
    if (*sh).is_hashed != 0 {
        js_shape_hash_unlink(rt, sh);
        new_shape_hash = shape_hash(shape_hash((*sh).hash, atom), prop_flags as u32);
    }
    if (*sh).prop_count >= (*sh).prop_size {
        if resize_properties(ctx, psh, p, ((*sh).prop_count as u32).wrapping_add(1)) != 0 {
            if (*sh).is_hashed != 0 {
                js_shape_hash_link(rt, sh);
            }
            return -1;
        }
        sh = *psh;
    }
    if (*sh).is_hashed != 0 {
        (*sh).hash = new_shape_hash;
        js_shape_hash_link(rt, sh);
    }
    let pr = get_shape_prop(sh).add((*sh).prop_count as usize);
    (*sh).prop_count += 1;
    (*pr).atom = JS_DupAtom(ctx, atom);
    (*pr).set_flags(prop_flags as u32);
    let h = (atom & (*sh).prop_hash_mask) as usize;
    let hash = ptr::addr_of_mut!((*sh).hash_table).cast::<u32>();
    (*pr).set_hash_next(*hash.add(h));
    *hash.add(h) = (*sh).prop_count as u32;
    0
}
unsafe fn add_property(
    ctx: *mut JSContext,
    p: *mut JSObject,
    prop: JSAtom,
    prop_flags: i32,
) -> *mut JSProperty {
    if __JS_AtomIsTaggedInt(prop) != 0 {
        if (*p).is_std_array_prototype() != 0 {
            (*p).set_is_std_array_prototype(0);
        } else if (*p).has_immutable_prototype() != 0 {
            let rt = (*ctx).rt;
            let mut el = (*rt).context_list.next;
            while el != ptr::addr_of_mut!((*rt).context_list) {
                let ctx1 = el
                    .cast::<u8>()
                    .sub(offset_of!(JSContext, link))
                    .cast::<JSContext>();
                let object_proto = *(*ctx1).class_proto.add(JS_CLASS_OBJECT as usize);
                if JS_IsObject(object_proto) != 0
                    && JS_VALUE_GET_PTR(object_proto).cast::<JSObject>() == p
                {
                    let array_proto = *(*ctx1).class_proto.add(JS_CLASS_ARRAY as usize);
                    if JS_IsObject(array_proto) != 0 {
                        (*JS_VALUE_GET_PTR(array_proto).cast::<JSObject>())
                            .set_is_std_array_prototype(0);
                    }
                    break;
                }
                el = (*el).next;
            }
        }
    }
    let sh = (*p).shape;
    if (*sh).is_hashed != 0 {
        let new_sh = find_hashed_shape_prop((*ctx).rt, sh, prop, prop_flags);
        if !new_sh.is_null() {
            if (*new_sh).prop_size != (*sh).prop_size {
                let new_prop = js_realloc(
                    ctx,
                    (*p).prop.cast(),
                    size_of::<JSProperty>() * (*new_sh).prop_size as usize,
                )
                .cast::<JSProperty>();
                if new_prop.is_null() {
                    return ptr::null_mut();
                }
                (*p).prop = new_prop;
            }
            (*p).shape = js_dup_shape(new_sh);
            js_free_shape((*ctx).rt, sh);
            return (*p).prop.add((*new_sh).prop_count as usize - 1);
        } else if (*js_rc(sh.cast())).ref_count != 1 {
            let new_sh = js_clone_shape(ctx, sh);
            if new_sh.is_null() {
                return ptr::null_mut();
            }
            (*new_sh).is_hashed = 1;
            js_shape_hash_link((*ctx).rt, new_sh);
            js_free_shape((*ctx).rt, (*p).shape);
            (*p).shape = new_sh;
        }
    }
    assert!((*js_rc((*p).shape.cast())).ref_count == 1);
    if add_shape_property(ctx, ptr::addr_of_mut!((*p).shape), p, prop, prop_flags) != 0 {
        return ptr::null_mut();
    }
    (*p).prop.add((*(*p).shape).prop_count as usize - 1)
}
// quickjs.c:10302-10340, shared shapes must be detached before mutation.
unsafe fn js_shape_prepare_update(
    ctx: *mut JSContext,
    p: *mut JSObject,
    pprs: *mut *mut JSShapeProperty,
) -> i32 {
    let mut sh = (*p).shape;
    if (*sh).is_hashed != 0 {
        if (*js_rc(sh.cast())).ref_count != 1 {
            let mut idx = 0u32;
            if !pprs.is_null() {
                idx = (*pprs).offset_from(get_shape_prop(sh)) as u32;
            }
            sh = js_clone_shape(ctx, sh);
            if sh.is_null() {
                return -1;
            }
            js_free_shape((*ctx).rt, (*p).shape);
            (*p).shape = sh;
            if !pprs.is_null() {
                *pprs = get_shape_prop(sh).add(idx as usize);
            }
        } else {
            js_shape_hash_unlink((*ctx).rt, sh);
            (*sh).is_hashed = 0;
        }
    }
    0
}
unsafe fn js_update_property_flags(
    ctx: *mut JSContext,
    p: *mut JSObject,
    pprs: *mut *mut JSShapeProperty,
    flags: i32,
) -> i32 {
    if flags != (**pprs).flags() as i32 {
        if js_shape_prepare_update(ctx, p, pprs) != 0 {
            return -1;
        }
        (**pprs).set_flags(flags as u32);
    }
    0
}
