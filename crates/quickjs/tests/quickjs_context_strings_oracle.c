static void init(JSRuntime*rt,JSContext*ctx,struct host*h,struct boundary_log*log) {
 js_malloc_init(&rt->malloc_ctx);rt->malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt->malloc_ctx.malloc_state.opaque=h;
 init_list_head(&rt->context_list);init_list_head(&rt->gc_obj_list);init_list_head(&rt->gc_zero_ref_count_list);init_list_head(&rt->weakref_list);
 rt->malloc_gc_threshold=SIZE_MAX;assert(!JS_InitAtoms(rt));assert(!init_shape_hash(rt));ctx->rt=rt;ctx->user_opaque=log;
}
static void finish(JSRuntime*rt,struct host*h){assert(!rt->shape_hash_count);assert(list_empty(&rt->gc_obj_list));js_free_rt(rt,rt->shape_hash);cleanup(rt,h);assert(!h->live);}
static void value(JSValue val){num(JS_VALUE_GET_TAG(val),4);if(!JS_IsException(val))dump_string(JS_VALUE_GET_STRING(val));}
static void effects(struct host*h,struct boundary_log*log){num(log->oom,4);num(log->internal,4);num(log->type_error,4);num(h->calls,4);num(h->live,4);num(h->trace,8);}
static void encoded(JSContext*ctx,JSValue v,int cesu8){size_t len=0xa5a5;const char*p=JS_ToCStringLen2(ctx,&len,v,cesu8);num(!p,4);num(len,4);if(p)for(int i=0;i<=len;i++)num((uint8_t)p[i],1);num(js_rc(JS_VALUE_GET_PTR(v))->ref_count,4);JS_FreeCString(ctx,p);}
static uint64_t rng_next(uint64_t*s){*s^=*s<<13;*s^=*s>>7;*s^=*s<<17;return *s;}
static void string_fixtures(void);
static void shape_fixtures(void);
int main(void){string_fixtures();shape_fixtures();return 0;}
static void string_fixtures(void) {
 JSRuntime rt={0};JSContext ctx={0};struct host h={0};h.trace=1469598103934665603ULL;struct boundary_log log={0};init(&rt,&ctx,&h,&log);
 for(uint32_t bytes=0;bytes<65536;bytes++) {uint8_t buf[2]={bytes,bytes>>8};JSValue v=JS_NewStringLen(&ctx,(char*)buf,2);value(v);encoded(&ctx,v,0);encoded(&ctx,v,1);JS_FreeValue(&ctx,v);}
 uint64_t rng=0x123456789abcdef0ULL;
 for(int i=0;i<8192;i++) {
  uint8_t text[128]={0};int len=i%65;for(int j=0;j<len;j++)text[j]=rng_next(&rng);
  JSValue v=JS_NewStringLen(&ctx,(char*)text,len);value(v);encoded(&ctx,v,i%2);JSString*p=JS_VALUE_GET_STRING(v);
  int index=0;while(index<p->len){int c=string_getc(p,&index);num(c,4);num(index,4);}
  int length=p->len;int bounds[3][2]={{0,length},{0,0},{length/3,length*2/3}};
  for(int j=0;j<3;j++){JSValue sub=js_sub_string(&ctx,p,bounds[j][0],bounds[j][1]);value(sub);JS_FreeValue(&ctx,sub);}
  JSAtom atom=JS_NewAtomLen(&ctx,(char*)text,len);num(atom,4);JSValue s=JS_AtomToString(&ctx,atom);value(s);JS_FreeValue(&ctx,s);JS_FreeAtom(&ctx,atom);JS_FreeValue(&ctx,v);
  if(i%1024==0)effects(&h,&log);
 }
 for(int i=0;i<8192;i++) {
  uint16_t units[64]={0};int len=i%65;for(int j=0;j<len;j++)units[j]=rng_next(&rng);
  if(len>=8){uint16_t first[8]={0xd800,0xdc00,0xd800,0x61,0xdc00,0xff,0,0x80};memcpy(units,first,sizeof(first));}
  JSString*p=make_string(&rt,units,len,i%2);JSValue v=JS_MKPTR(JS_TAG_STRING,p);encoded(&ctx,v,0);encoded(&ctx,v,1);
  int index=0;while(index<len){int c=string_getc(p,&index);num(c,4);num(index,4);}
  int bounds[3][2]={{0,len},{0,0},{len/3,len*2/3}};for(int j=0;j<3;j++){JSValue sub=js_sub_string(&ctx,p,bounds[j][0],bounds[j][1]);value(sub);JS_FreeValue(&ctx,sub);}
  StringBuffer b;assert(!string_buffer_init(&ctx,&b,1));string_buffer_concat_value(&b,v);string_buffer_concat_value_free(&b,JS_DupValue(&ctx,v));JSValue result=string_buffer_end(&b);value(result);JS_FreeValue(&ctx,result);
  result=JS_ConcatString3(&ctx,"pre",JS_DupValue(&ctx,v),"post");value(result);JS_FreeValue(&ctx,result);JS_FreeValue(&ctx,v);
  JSValue c=js_new_string_char(&ctx,i);value(c);JS_FreeValue(&ctx,c);
 }
 int64_t ns[]={0,1,47,2147483647,2147483648LL,4294967294LL,4294967295LL,INT64_MAX,INT64_MIN,-1};
 for(int i=0;i<10;i++){int64_t n=ns[i];JSAtom atom=JS_NewAtomInt64(&ctx,n);num(atom,4);JSValue v=JS_AtomToValue(&ctx,atom);value(v);JS_FreeValue(&ctx,v);JS_FreeAtom(&ctx,atom);atom=JS_NewAtomUInt32(&ctx,n);num(atom,4);v=JS_AtomToString(&ctx,atom);value(v);JS_FreeValue(&ctx,v);JS_FreeAtom(&ctx,atom);}
 int types[]={JS_ATOM_TYPE_SYMBOL,JS_ATOM_TYPE_GLOBAL_SYMBOL,JS_ATOM_TYPE_PRIVATE};const char*descrs[]={"","name"};
 for(int i=0;i<3;i++)for(int j=0;j<2;j++){
  JSAtom atom=JS_NewAtom(&ctx,descrs[j]);JSValue symbol=JS_NewSymbolFromAtom(&ctx,atom,types[i]);JS_FreeAtom(&ctx,atom);atom=js_get_atom_index(&rt,JS_VALUE_GET_STRING(symbol));
  for(int force=0;force<2;force++){JSValue v=__JS_AtomToValue(&ctx,atom,force);if(!force){num(JS_VALUE_GET_TAG(v),4);dump_string(JS_VALUE_GET_STRING(v));}else value(v);JS_FreeValue(&ctx,v);}JS_FreeValue(&ctx,symbol);
 }
 JSAtom a=JS_NewAtom(&ctx,"base"),b=js_atom_concat_str(&ctx,a,"-suffix"),c=js_atom_concat_num(&ctx,b,UINT32_MAX);JSAtom atoms[]={a,b,c};for(int i=0;i<3;i++){JSValue v=JS_AtomToString(&ctx,atoms[i]);value(v);JS_FreeValue(&ctx,v);JS_FreeAtom(&ctx,atoms[i]);}
 effects(&h,&log);finish(&rt,&h);
 for(int trial=0;trial<64;trial++) {
  JSRuntime rt={0};JSContext ctx={0};struct host h={0};h.trace=1469598103934665603ULL;struct boundary_log log={0};init(&rt,&ctx,&h,&log);if(trial)h.fail=h.calls+trial;
  StringBuffer b;num(string_buffer_init2(&ctx,&b,1,trial%2),4);
  for(int step=0;step<256;step++){
   int ret;uint32_t c=step%4==0?0x1f642:step%4==1?0x100:65;uint16_t wide[]={0x61,0xff,0x100,0xdfff};
   switch(step%7){case 0:ret=string_buffer_putc(&b,c);break;case 1:ret=string_buffer_putc8(&b,0xff);break;case 2:ret=string_buffer_putc16(&b,0xd800);break;case 3:ret=string_buffer_write8(&b,(const uint8_t*)"abcdefgh",8);break;case 4:ret=string_buffer_write16(&b,wide,4);break;case 5:ret=string_buffer_fill(&b,0x1ff,3);break;default:ret=string_buffer_puts8(&b,"abcd");break;}
   num(ret,4);num(b.len,4);num(b.size,4);num(b.is_wide_char,4);num(b.error_status,4);
  }
  JSValue v=string_buffer_end(&b);value(v);JS_FreeValue(&ctx,v);
  h.fail=0;uint16_t units[1024];for(int i=0;i<1024;i++)units[i]=0xffff;JSString*p=make_string(&rt,units,1024,1);v=JS_MKPTR(JS_TAG_STRING,p);h.fail=h.calls+1;encoded(&ctx,v,0);
  h.fail=0;JS_FreeValue(&ctx,v);JS_FreeValue(&ctx,v);
  string_buffer_init(&ctx,&b,1);num(string_buffer_realloc(&b,JS_STRING_LEN_MAX+1,0),4);value(string_buffer_end(&b));effects(&h,&log);finish(&rt,&h);
 }
}
static void shape_fixtures(void) {
 for(int trial=0;trial<128;trial++) {
  JSRuntime rt={0};JSContext ctx={0};struct host h={0};h.trace=1469598103934665603ULL;struct boundary_log log={0};init(&rt,&ctx,&h,&log);
  JSClass classes[JS_CLASS_INIT_COUNT]={0};rt.class_array=classes;rt.class_count=JS_CLASS_INIT_COUNT;JSValue prototypes[JS_CLASS_INIT_COUNT];for(int i=0;i<JS_CLASS_INIT_COUNT;i++)prototypes[i]=JS_NULL;ctx.class_proto=prototypes;
  JSValue a=JS_NewObject(&ctx),b=JS_NewObject(&ctx);JSObject*pa=JS_VALUE_GET_OBJ(a),*pb=JS_VALUE_GET_OBJ(b);if(trial)h.fail=h.calls+trial;
  for(int i=0;i<256;i++){
   JSAtom atom=__JS_AtomFromUInt32(i);JSObject*p=i%2==0?pa:pb;JSProperty*pr=add_property(&ctx,p,atom,JS_PROP_C_W_E);num(!pr,4);if(pr)pr->u.value=JS_NewInt32(&ctx,i);
   JSShape*sh=p->shape;num(sh->is_hashed,4);num(sh->prop_count,4);num(sh->prop_size,4);num(sh->prop_hash_mask,4);num(js_rc(sh)->ref_count,4);num(sh->hash,4);
  }
  h.fail=0;
  JSValue x=JS_NewObject(&ctx),y=JS_NewObject(&ctx);JSObject*px=JS_VALUE_GET_OBJ(x),*py=JS_VALUE_GET_OBJ(y);
  for(int i=0;i<32;i++){JSObject*ps[]={px,py};for(int j=0;j<2;j++){JSProperty*pr=add_property(&ctx,ps[j],__JS_AtomFromUInt32(i),JS_PROP_C_W_E);assert(pr);pr->u.value=JS_NewInt32(&ctx,i);}num(px->shape==py->shape,4);}
  JSShapeProperty*prs=get_shape_prop(px->shape)+7;num(js_update_property_flags(&ctx,px,&prs,JS_PROP_CONFIGURABLE),4);num(prs-get_shape_prop(px->shape),4);num(prs->flags,4);num(px->shape==py->shape,4);num(get_shape_prop(py->shape)[7].flags,4);
  prs=get_shape_prop(py->shape)+3;num(js_update_property_flags(&ctx,py,&prs,JS_PROP_ENUMERABLE),4);num(py->shape->is_hashed,4);JS_FreeValue(&ctx,x);JS_FreeValue(&ctx,y);
  JSShape*old=pa->shape,*sh=js_clone_shape(&ctx,old);js_free_shape(&rt,old);pa->shape=sh;JSShapeProperty*props=get_shape_prop(sh);
  for(int i=0;i<sh->prop_count;i+=2){JS_FreeAtom(&ctx,props[i].atom);props[i].atom=JS_ATOM_NULL;props[i].flags=0;pa->prop[i].u.value=JS_UNDEFINED;sh->deleted_prop_count++;}
  num(compact_properties(&ctx,pa),4);for(int i=0;i<pa->shape->prop_count;i++){JSShapeProperty*pr=get_shape_prop(pa->shape)+i;num(pr->atom,4);num(pr->flags,4);num(JS_VALUE_GET_INT(pa->prop[i].u.value),4);JSProperty*property;num(find_own_property(&property,pa,pr->atom)!=NULL,4);num(property-pa->prop,4);}
  JS_FreeValue(&ctx,a);JS_FreeValue(&ctx,b);
  int ids[]={JS_CLASS_OBJECT,JS_CLASS_ERROR,JS_CLASS_NUMBER,JS_CLASS_ARRAY,JS_CLASS_GLOBAL_OBJECT,JS_CLASS_RAWJSON};
  for(int i=0;i<6;i++){JSValue v=JS_NewObjectProtoClassAlloc(&ctx,JS_NULL,ids[i],8);num(JS_VALUE_GET_TAG(v),4);JSObject*p=JS_VALUE_GET_OBJ(v);num(p->class_id,4);num(p->extensible,4);num(p->fast_array,4);num(p->is_exotic,4);num(p->shape->prop_count,4);JS_FreeValue(&ctx,v);}
  effects(&h,&log);finish(&rt,&h);
 }
}
