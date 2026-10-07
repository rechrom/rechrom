/* Paired fixture uses the complete, unchanged upstream engine. */
static JSValue callable(JSContext*ctx,JSValueConst func,JSValueConst self,int argc,JSValueConst*argv,int flags){abort();}
static void setup(JSRuntime*rt,JSContext*ctx,struct host*h,JSClass*classes,JSValue*prototypes){
 h->trace=1469598103934665603ULL;js_malloc_init(&rt->malloc_ctx);rt->malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt->malloc_ctx.malloc_state.opaque=h;
 init_list_head(&rt->context_list);init_list_head(&rt->gc_obj_list);init_list_head(&rt->gc_zero_ref_count_list);init_list_head(&rt->weakref_list);rt->malloc_gc_threshold=SIZE_MAX;rt->current_exception=JS_UNINITIALIZED;
 assert(!JS_InitAtoms(rt));assert(!init_shape_hash(rt));rt->class_array=classes;rt->class_count=JS_CLASS_INIT_COUNT;
 classes[JS_CLASS_ARRAY].finalizer=js_array_finalizer;int box_classes[]={JS_CLASS_NUMBER,JS_CLASS_BOOLEAN,JS_CLASS_BIG_INT,JS_CLASS_SYMBOL,JS_CLASS_STRING};for(int i=0;i<5;i++)classes[box_classes[i]].finalizer=js_object_data_finalizer;classes[JS_CLASS_GLOBAL_OBJECT].finalizer=js_global_object_finalizer;classes[JS_CLASS_C_FUNCTION].call=callable;
 ctx->rt=rt;ctx->class_proto=prototypes;for(int i=0;i<JS_NATIVE_ERROR_COUNT;i++)ctx->native_error_proto[i]=JS_NULL;init_list_head(&ctx->loaded_modules);
 JSValue arr=JS_NewObjectProtoClass(ctx,JS_NULL,JS_CLASS_ARRAY);assert(!JS_IsException(arr));ctx->array_shape=js_dup_shape(JS_VALUE_GET_OBJ(arr)->shape);JS_FreeValue(ctx,arr);
}
static void dump_val(JSValue v){
 int tag=JS_VALUE_GET_NORM_TAG(v);num(tag,4);
 switch(tag){case JS_TAG_STRING:{JSString*s=JS_VALUE_GET_STRING(v);num(s->len,4);for(int i=0;i<s->len;i++)num(string_get(s,i),4);break;}
 case JS_TAG_FLOAT64:{JSFloat64Union u={.d=JS_VALUE_GET_FLOAT64(v)};num(u.u64,8);break;}
 case JS_TAG_SHORT_BIG_INT:num(JS_VALUE_GET_SHORT_BIG_INT(v),8);break;
 case JS_TAG_BIG_INT:{JSBigInt*b=JS_VALUE_GET_PTR(v);num(b->len,4);for(int i=0;i<b->len;i++)num(b->tab[i],8);break;}
 case JS_TAG_OBJECT:num(JS_VALUE_GET_OBJ(v)->class_id,4);break;
 default:num(JS_VALUE_GET_INT(v),4);break;}
}
static void dump_exception(JSContext*ctx){
 int has=JS_HasException(ctx);num(has,4);if(!has)return;JSValue ex=JS_GetException(ctx);dump_val(ex);
 if(JS_IsObject(ex)){JSObject*p=JS_VALUE_GET_OBJ(ex);JSAtom atoms[]={JS_ATOM_message,JS_ATOM_stack};for(int i=0;i<2;i++){JSProperty*pr;JSShapeProperty*prs=find_own_property(&pr,p,atoms[i]);num(prs!=NULL,4);if(prs){num(prs->flags,4);dump_val(pr->u.value);}}}JS_FreeValue(ctx,ex);
}
static void dump_object(JSObject*p){
 JSShape*sh=p->shape;num(p->class_id,4);num(p->extensible,4);num(p->fast_array,4);num(sh->is_hashed,4);num(sh->prop_count,4);num(sh->deleted_prop_count,4);num(sh->prop_size,4);num(sh->prop_hash_mask,4);
 for(int i=0;i<sh->prop_count;i++){JSShapeProperty*prs=get_shape_prop(sh)+i;JSProperty*pr=p->prop+i;num(prs->atom,4);num(prs->flags,4);switch(prs->flags&JS_PROP_TMASK){
 case JS_PROP_GETSET:num(pr->u.getset.getter!=NULL,4);num(pr->u.getset.setter!=NULL,4);break;
 case JS_PROP_VARREF:{JSVarRef*vr=pr->u.var_ref;num(js_rc(vr)->ref_count,4);num(vr->is_const,4);dump_val(*vr->pvalue);break;}
 default:dump_val(pr->u.value);break;}}
 if(p->fast_array){num(p->u.array.count,4);num(p->u.array.u1.size,4);for(int i=0;i<p->u.array.count;i++)dump_val(p->u.array.u.values[i]);}
}
static JSValue test_rope(JSContext*ctx,JSValue left,JSValue right){JSStringRope*r=js_malloc(ctx,sizeof(*r));assert(r);js_rc(r)->ref_count=1;r->left=left;r->right=right;r->len=string_rope_get_len(left)+string_rope_get_len(right);r->is_wide_char=1;r->depth=1;return JS_MKPTR(JS_TAG_STRING_ROPE,r);}
static void supplemental(JSContext*ctx){
 JSValue base=JS_NewObject(ctx),child=JS_NewObject(ctx);JSAtom key=__JS_AtomFromUInt32(3);num(JS_DefinePropertyValue(ctx,base,key,JS_NewInt32(ctx,23),JS_PROP_C_W_E),4);num(JS_SetPrototype(ctx,child,base),4);
 JSValue v=JS_GetProperty(ctx,child,key);dump_val(v);JS_FreeValue(ctx,v);num(JS_HasProperty(ctx,child,key),4);
 JSValue objects[]={base,child};for(int i=0;i<2;i++){JSPropertyDescriptor d={0};int ret=JS_GetOwnProperty(ctx,&d,objects[i],key);num(ret,4);if(ret>0){num(d.flags,4);dump_val(d.value);dump_val(d.getter);dump_val(d.setter);JS_FreeValue(ctx,d.value);JS_FreeValue(ctx,d.getter);JS_FreeValue(ctx,d.setter);}}
 struct {JSValue obj,proto;int flags;} pairs[]={{child,child,1},{base,child,0},{base,child,1},{child,JS_TRUE,0},{JS_NewInt32(ctx,3),JS_NULL,1},{JS_NULL,JS_NULL,1}};
 for(int i=0;i<6;i++){num(JS_SetPrototypeInternal(ctx,pairs[i].obj,pairs[i].proto,pairs[i].flags),4);dump_exception(ctx);}
 v=JS_GetPrototype(ctx,child);dump_val(v);JS_FreeValue(ctx,v);JS_SetImmutablePrototype(ctx,child);num(JS_SetPrototype(ctx,child,JS_NULL),4);dump_exception(ctx);
 num(JS_PreventExtensions(ctx,base),4);num(JS_IsExtensible(ctx,base),4);num(JS_SetPrototype(ctx,base,child),4);dump_exception(ctx);
 JSValue read_objs[]={child,JS_NULL,JS_UNDEFINED,JS_EXCEPTION,JS_TRUE};for(int i=0;i<5;i++){v=JS_GetPropertyInternal(ctx,read_objs[i],__JS_AtomFromUInt32(77),read_objs[i],1);dump_val(v);JS_FreeValue(ctx,v);dump_exception(ctx);}JS_FreeValue(ctx,child);JS_FreeValue(ctx,base);
 JSValue arr=JS_NewArray(ctx);for(int i=0;i<12;i++)JS_DefinePropertyValueUint32(ctx,arr,i,JS_NewInt32(ctx,i*9),JS_PROP_C_W_E);
 int64_t indices[]={-1,0,7,11,12,2147483648LL,4294967295LL,4294967296LL};for(int i=0;i<8;i++){v=JS_UNDEFINED;num(JS_TryGetPropertyInt64(ctx,arr,indices[i],&v),4);dump_val(v);JS_FreeValue(ctx,v);v=JS_GetPropertyInt64(ctx,arr,indices[i]);dump_val(v);JS_FreeValue(ctx,v);}JS_FreeValue(ctx,arr);
 JSValue global=JS_NewObjectProtoClass(ctx,JS_NULL,JS_CLASS_GLOBAL_OBJECT);JSObject*gp=JS_VALUE_GET_OBJ(global);gp->u.global_object.uninitialized_vars=JS_NewObject(ctx);JS_DefinePropertyValue(ctx,global,key,JS_NewInt32(ctx,11),JS_PROP_C_W_E);JSProperty*pr;find_own_property(&pr,gp,key);JSVarRef*vr=pr->u.var_ref;js_rc(vr)->ref_count++;
 num(delete_property(ctx,gp,key),4);dump_object(gp);dump_val(*vr->pvalue);dump_object(JS_VALUE_GET_OBJ(gp->u.global_object.uninitialized_vars));num(JS_DefinePropertyValue(ctx,global,key,JS_NewInt32(ctx,12),JS_PROP_C_W_E),4);dump_val(*vr->pvalue);dump_object(gp);free_var_ref(ctx->rt,vr);JS_FreeValue(ctx,global);
 JSValue empty=JS_NewStringLen(ctx,"",0),a=JS_NewStringLen(ctx,"a",1),b=JS_NewStringLen(ctx,"b",1);uint16_t units[]={97,98};JSValue wide=js_new_string16_len(ctx,units,2);JSValue rope=test_rope(ctx,JS_DupValue(ctx,a),JS_DupValue(ctx,b)),nested=test_rope(ctx,JS_DupValue(ctx,empty),JS_DupValue(ctx,rope));
 JSValue sym=JS_NewSymbolFromAtom(ctx,JS_ATOM_name,JS_ATOM_TYPE_SYMBOL),sym2=JS_NewSymbolFromAtom(ctx,JS_ATOM_name,JS_ATOM_TYPE_SYMBOL),obj=JS_NewObject(ctx),obj2=JS_NewObject(ctx);JSFloat64Union nan={.u64=0x7ff8000000001234ULL};
 JSValue vals[]={JS_NULL,JS_UNDEFINED,JS_FALSE,JS_TRUE,JS_NewInt32(ctx,0),JS_NewInt32(ctx,-1),__JS_NewFloat64(ctx,-0.0),__JS_NewFloat64(ctx,nan.d),JS_NAN,__JS_NewFloat64(ctx,0.5),__JS_NewShortBigInt(ctx,-1),__JS_NewShortBigInt(ctx,0),empty,a,b,wide,rope,nested,sym,JS_DupValue(ctx,sym),sym2,obj,JS_DupValue(ctx,obj),obj2};
 for(int i=0;i<countof(vals);i++)for(int j=0;j<countof(vals);j++){num(JS_StrictEq(ctx,vals[i],vals[j]),4);num(JS_SameValue(ctx,vals[i],vals[j]),4);num(JS_SameValueZero(ctx,vals[i],vals[j]),4);if(tag_is_string(JS_VALUE_GET_TAG(vals[i]))&&tag_is_string(JS_VALUE_GET_TAG(vals[j]))){num(js_string_rope_compare(ctx,vals[i],vals[j],0),4);num(js_string_rope_compare(ctx,vals[i],vals[j],1),4);}}
 JSValue strings[]={wide,rope,nested};for(int i=0;i<3;i++){for(int idx=0;idx<4;idx++){v=JS_GetPropertyUint32(ctx,strings[i],idx);dump_val(v);JS_FreeValue(ctx,v);}v=JS_GetProperty(ctx,strings[i],JS_ATOM_length);dump_val(v);JS_FreeValue(ctx,v);}for(int i=0;i<countof(vals);i++)JS_FreeValue(ctx,vals[i]);
 JSValue f=JS_NewObjectProtoClass(ctx,JS_NULL,JS_CLASS_C_FUNCTION);JS_DefinePropertyValue(ctx,f,JS_ATOM_name,JS_NewStringLen(ctx,"native_fn",9),JS_PROP_C_W_E);JSStackFrame sf={0};sf.cur_func=f;ctx->rt->current_stack_frame=&sf;
 JSValue error=JS_NewError(ctx);build_backtrace(ctx,error,"filename.js",12,3,0);dump_object(JS_VALUE_GET_OBJ(error));JS_FreeValue(ctx,error);error=JS_NewError(ctx);build_backtrace(ctx,error,NULL,0,0,JS_BACKTRACE_FLAG_SKIP_FIRST_LEVEL);dump_object(JS_VALUE_GET_OBJ(error));JS_FreeValue(ctx,error);ctx->rt->current_stack_frame=NULL;JS_FreeValue(ctx,f);
}
static void numeric_fixtures(JSContext*ctx,struct host*h){
 uint64_t rng=0x123456789abcdef0ULL;for(int i=0;i<16384;i++){rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;JSFloat64Union u={.u64=rng};JSValue v=__JS_NewFloat64(ctx,u.d);int32_t n=0;num(JS_ToInt32(ctx,&n,v),4);num(n,4);}
 const char*texts[]={"+Infinity","-0x1","0_1","00_1","1_2_3",".1","1.e2","1e+","077","078","0b11","0o77","0xG","--1"," 1","1__2","_1","1_","1e1_2","1e_2"};int radices[]={0,2,8,10,16,36};
 for(int t=0;t<20;t++)for(int r=0;r<6;r++)for(int mask=0;mask<32;mask++){int flags=((mask&1)?ATOD_INT_ONLY:0)|((mask&2)?ATOD_ACCEPT_BIN_OCT:0)|((mask&4)?ATOD_ACCEPT_LEGACY_OCTAL:0)|((mask&8)?ATOD_ACCEPT_UNDERSCORES:0)|((mask&16)?ATOD_ACCEPT_PREFIX_AFTER_SIGN:0);const char*end=NULL;JSValue v=js_atof(ctx,texts[t],&end,radices[r],flags);num(end-texts[t],4);dump_val(v);JS_FreeValue(ctx,v);}
 const char*bigtexts[]={"0n","1234567890123456789012345678901234567890n","-0x123456789abcdef0123456789n","0b10101010101010101010101010101010101010101010101010101010101010101n","077n","1.5n","1e2n"};for(int t=0;t<7;t++)for(int mask=0;mask<16;mask++){int flags=ATOD_ACCEPT_SUFFIX|((mask&1)?ATOD_TYPE_BIG_INT:0)|((mask&2)?ATOD_ACCEPT_BIN_OCT:0)|((mask&4)?ATOD_ACCEPT_LEGACY_OCTAL:0)|((mask&8)?ATOD_ACCEPT_PREFIX_AFTER_SIGN:0);const char*end=NULL;JSValue v=js_atof(ctx,bigtexts[t],&end,0,flags);num(end-bigtexts[t],4);dump_val(v);JS_FreeValue(ctx,v);dump_exception(ctx);}
 num(h->calls,4);int big_radices[]={2,8,10,16},lengths[]={1,19,20,65,128};for(int r=0;r<4;r++)for(int l=0;l<5;l++)for(int negative=0;negative<2;negative++){char text[256];int j=0;if(negative)text[j++]='-';for(int i=0;i<lengths[l];i++)text[j++]=digits[(i*7+1)%big_radices[r]];text[j]=0;num(h->calls,4);JSBigInt*b=js_bigint_from_string(ctx,text,big_radices[r]);assert(b);JSValue v=JS_CompactBigInt(ctx,b);dump_val(v);for(int base=2;base<=36;base++){num(big_radices[r],4);num(lengths[l],4);num(negative,4);num(base,4);num(h->calls,4);JSValue s=js_bigint_to_string1(ctx,v,base);dump_val(s);JS_FreeValue(ctx,s);}JSValue s=JS_ToString(ctx,v);dump_val(s);JS_FreeValue(ctx,s);JS_FreeValue(ctx,v);}
 uint64_t nums[]={0,1,2147483648ULL,9223372036854775807ULL,9223372036854775808ULL,18446744073709551615ULL};for(int i=0;i<6;i++){JSValue v=JS_NewBigUint64(ctx,nums[i]);dump_val(v);JSValue s=JS_ToString(ctx,v);dump_val(s);JS_FreeValue(ctx,s);JS_FreeValue(ctx,v);}
 for(int fail=0;fail<64;fail++){num(fail,4);num(h->calls,4);h->fail=fail?h->calls+fail:0;const char*text="-123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890";JSBigInt*b=js_bigint_from_string(ctx,text,10);JSValue v=b?JS_CompactBigInt(ctx,b):JS_EXCEPTION;dump_val(v);dump_exception(ctx);if(!JS_IsException(v))for(int radix=2;radix<=36;radix++){JSValue s=js_bigint_to_string1(ctx,v,radix);dump_val(s);JS_FreeValue(ctx,s);dump_exception(ctx);}JS_FreeValue(ctx,v);h->fail=0;}
}
int main(void){
 for(int trial=0;trial<128;trial++){
  JSRuntime rt={0};JSContext ctx={0};struct host h={0};JSClass classes[JS_CLASS_INIT_COUNT]={0};JSValue prototypes[JS_CLASS_INIT_COUNT];for(int i=0;i<JS_CLASS_INIT_COUNT;i++)prototypes[i]=JS_NULL;setup(&rt,&ctx,&h,classes,prototypes);
  JSValue obj=JS_NewObject(&ctx);JSObject*p=JS_VALUE_GET_OBJ(obj);JSValue getter=JS_NewObjectProtoClass(&ctx,JS_NULL,JS_CLASS_C_FUNCTION);if(trial)h.fail=h.calls+trial;
  for(int step=0;step<512;step++){
   JSAtom atom=__JS_AtomFromUInt32(step%32);JSValue val=JS_NewInt32(&ctx,step);int flags=step%5==0?JS_PROP_THROW:0;int ret;
   switch(step%8){
   case 0:ret=JS_DefinePropertyValue(&ctx,obj,atom,val,JS_PROP_C_W_E|flags);break;
   case 1:ret=JS_DefineProperty(&ctx,obj,atom,val,JS_UNDEFINED,JS_UNDEFINED,JS_PROP_HAS_VALUE|flags);break;
   case 2:ret=JS_DefineProperty(&ctx,obj,atom,JS_UNDEFINED,getter,JS_UNDEFINED,JS_PROP_HAS_GET|JS_PROP_CONFIGURABLE|JS_PROP_HAS_CONFIGURABLE|flags);break;
   case 3:ret=JS_DefinePropertyValue(&ctx,obj,atom,val,JS_PROP_CONFIGURABLE|flags);break;
   case 4:ret=delete_property(&ctx,p,atom);break;
   case 5:ret=JS_DefineProperty(&ctx,obj,atom,JS_UNDEFINED,JS_UNDEFINED,JS_UNDEFINED,JS_PROP_HAS_WRITABLE|flags);break;
   case 6:ret=JS_DefineProperty(&ctx,obj,atom,val,JS_UNDEFINED,JS_UNDEFINED,JS_PROP_HAS_VALUE|JS_PROP_HAS_ENUMERABLE|JS_PROP_ENUMERABLE|flags);break;
   default:ret=JS_DefinePropertyValue(&ctx,obj,atom,val,flags);break;}
   num(ret,4);dump_exception(&ctx);if(step%64==0)dump_object(p);
  }
  h.fail=0;p->extensible=0;num(JS_DefinePropertyValueStr(&ctx,obj,"new",JS_NewInt32(&ctx,1),JS_PROP_THROW),4);dump_exception(&ctx);
  num(JS_DefinePropertyValue(&ctx,JS_NULL,__JS_AtomFromUInt32(0),JS_NewInt32(&ctx,1),0),4);dump_exception(&ctx);JS_FreeValue(&ctx,getter);JS_FreeValue(&ctx,obj);
  JSValue arr=JS_NewArray(&ctx);p=JS_VALUE_GET_OBJ(arr);for(int i=0;i<64;i++)num(JS_DefinePropertyValueUint32(&ctx,arr,i,JS_NewInt32(&ctx,i),JS_PROP_C_W_E),4);
  num(JS_DefinePropertyValueUint32(&ctx,arr,7,JS_NewInt32(&ctx,70),JS_PROP_CONFIGURABLE|JS_PROP_WRITABLE),4);dump_object(p);
  double lengths[]={50,8,7,0,-1,1.5,4294967296.0};for(int i=0;i<7;i++){num(JS_DefinePropertyValue(&ctx,arr,JS_ATOM_length,JS_NewFloat64(&ctx,lengths[i]),JS_PROP_WRITABLE|JS_PROP_THROW),4);dump_exception(&ctx);dump_object(p);}JS_FreeValue(&ctx,arr);
  const char*texts[]={"","  ","0x10","-0x10","0b101","Infinity","-0","1.25e2","1e+","123abc"};
  for(int i=0;i<10;i++){JSValue v=JS_NewStringLen(&ctx,texts[i],strlen(texts[i]));JSValue n=JS_ToNumber(&ctx,v);dump_val(n);JSValue s=JS_ToString(&ctx,n);dump_val(s);JS_FreeValue(&ctx,s);JS_FreeValue(&ctx,n);JS_FreeValue(&ctx,v);}
  double nums[]={-INFINITY,-0.0,0,1.5,4294967295.0,4294967296.0,9007199254740991.0,INFINITY};for(int i=0;i<8;i++){JSValue v=JS_NewFloat64(&ctx,nums[i]);uint32_t u=0;num(JS_ToUint32(&ctx,&u,v),4);num(u,4);JSValue s=JS_ToString(&ctx,v);dump_val(s);JS_FreeValue(&ctx,s);}
  supplemental(&ctx);if(!trial){numeric_fixtures(&ctx,&h);writes_conversions_fixtures(&ctx,&h);names_private_global_fixtures(&ctx,&h);objects_ropes_fixtures(&ctx,&h);}num(h.calls,4);num(h.live,4);num(h.trace,8);JS_FreeValue(&ctx,rt.current_exception);rt.current_exception=JS_UNINITIALIZED;js_free_shape(&rt,ctx.array_shape);assert(!rt.shape_hash_count);assert(list_empty(&rt.gc_obj_list));js_free_rt(&rt,rt.shape_hash);cleanup(&rt,&h);assert(!h.live);
 }
 return 0;
}
