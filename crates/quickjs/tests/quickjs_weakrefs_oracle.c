static uint64_t rng=0x123456789abcdef0ULL;
static uint64_t next(void){rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;return rng;}
static void weak_snapshot(JSRuntime*rt,struct host*h,JSMapState*s){
 num(s->record_count,4);struct list_head*el;int count=0;list_for_each(el,&s->records)count++;num(count,4);
 list_for_each(el,&s->records){JSMapRecord*r=list_entry(el,JSMapRecord,link);num(r->ref_count,4);num(r->empty,4);num(JS_VALUE_GET_TAG(r->key),4);num(JS_VALUE_GET_INT(r->value),4);}
 int buckets=0;for(int j=0;j<(int)s->hash_size;j++)for(JSMapRecord*r=s->hash_table[j];r;r=r->hash_next){assert(!r->empty);assert(js_weakref_is_live(r->key));buckets++;}num(buckets,4);num(h->calls,4);num(h->live,4);num(h->trace,8);
}
int main(void){
 JSRuntime rt={0};struct host h={0};h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;init_list_head(&rt.gc_obj_list);init_list_head(&rt.gc_zero_ref_count_list);init_list_head(&rt.tmp_obj_list);init_list_head(&rt.context_list);init_list_head(&rt.weakref_list);JS_InitAtoms(&rt);rt.current_exception=JS_UNINITIALIZED;JSContext ctx={0};ctx.rt=&rt;
 num(sizeof(JSWeakRefData),4);num(sizeof(JSFinRecEntry),4);num(sizeof(JSFinalizationRegistryData),4);
 for(int i=0;i<16384;i++){
  uint64_t a=next();int bits=1+i%31;num(map_hash32(a,bits),4);num(map_hash64(a,bits),4);num(map_hash_pointer(a,bits),4);
  JSValue v=JS_NewInt32(&ctx,a);num(map_hash_key(v,bits),4);v=__JS_NewFloat64(&ctx,(double)(int64_t)a/17);num(map_hash_key(v,bits),4);
  v=__JS_NewFloat64(&ctx,i%2?-0.0:NAN);num(map_hash_key(v,bits),4);num(JS_VALUE_GET_TAG(map_normalize_key_const(&ctx,v)),4);num(JS_VALUE_GET_INT(map_normalize_key(&ctx,v)),4);
  v=JS_MKPTR(JS_TAG_OBJECT,(void*)(uintptr_t)a);num(map_hash_key(v,bits),4);v=JS_MKPTR(JS_TAG_SYMBOL,(void*)(uintptr_t)a);num(map_hash_key(v,bits),4);
  v=__JS_NewShortBigInt(&ctx,a);num(map_hash_key(v,bits),4);
  js_limb_t buf[9];JSBigInt*b=(JSBigInt*)buf;b->len=1+i%8;for(int j=0;j<b->len;j++)b->tab[j]=next();num(map_hash_key(JS_MKPTR(JS_TAG_BIG_INT,b),bits),4);
  uint16_t str[5]={(uint16_t)a,(uint16_t)(a>>16),(uint16_t)(a>>32),1,0};JSString*p=make_string(&rt,str,5,i%2);v=JS_MKPTR(JS_TAG_STRING,p);num(map_hash_key(v,bits),4);JS_FreeValueRT(&rt,v);
  JSStringRope*rope=js_mallocz_rt(&rt,sizeof(*rope));js_rc(rope)->ref_count=1;rope->left=JS_MKPTR(JS_TAG_STRING,make_string(&rt,str,3,i%2));rope->right=JS_MKPTR(JS_TAG_STRING,make_string(&rt,str+3,2,1));rope->len=5;rope->is_wide_char=1;num(map_hash_key(JS_MKPTR(JS_TAG_STRING_ROPE,rope),bits),4);JS_FreeValueRT(&rt,JS_MKPTR(JS_TAG_STRING_ROPE,rope));
 }
 for(int trial=0;trial<512;trial++){
  JSMapState*s=js_mallocz_rt(&rt,sizeof(*s));init_list_head(&s->records);s->is_weak=TRUE;s->hash_bits=4;s->hash_size=16;s->hash_table=js_mallocz_rt(&rt,16*sizeof(JSMapRecord*));s->weakref_header.weakref_type=JS_WEAKREF_TYPE_MAP;list_add_tail(&s->weakref_header.link,&rt.weakref_list);
  JSValue keys[64];JSMapRecord*records[64];JSWeakRefData*weakrefs[64];int dead[64];
  for(int i=0;i<64;i++){
   JSValue key;if(i%2){JSAtom atom=__JS_NewAtomInit(&rt,"weak-key",8,JS_ATOM_TYPE_SYMBOL);key=JS_MKPTR(JS_TAG_SYMBOL,rt.atom_array[atom]);}
   else {JSObject*p=js_mallocz_rt(&rt,sizeof(*p));js_rc(p)->ref_count=1;key=JS_MKPTR(JS_TAG_OBJECT,p);}
   keys[i]=key;num(js_weakref_is_target(key),4);num(js_weakref_is_live(key),4);
   JSMapRecord*r=js_mallocz_rt(&rt,sizeof(*r));records[i]=r;r->ref_count=i%3==0?2:1;r->key=js_weakref_new(&ctx,key);r->value=JS_NewInt32(&ctx,i);uint32_t bucket=map_hash_key(key,4);r->hash_next=s->hash_table[bucket];s->hash_table[bucket]=r;list_add_tail(&r->link,&s->records);s->record_count++;
   JSWeakRefData*w=js_mallocz_rt(&rt,sizeof(*w));weakrefs[i]=w;w->target=js_weakref_new(&ctx,key);w->weakref_header.weakref_type=JS_WEAKREF_TYPE_WEAKREF;list_add_tail(&w->weakref_header.link,&rt.weakref_list);dead[i]=(trial+i)%3!=0;
   if(dead[i]){js_rc(JS_VALUE_GET_PTR(key))->ref_count=0;if(i%2)JS_FreeAtomStruct(&rt,JS_VALUE_GET_STRING(key));}
  }
  // Emulate a previous hash resize omitting dead keys, testing the missing-chain branch.
  if(trial%2){memset(s->hash_table,0,16*sizeof(JSMapRecord*));for(int i=0;i<64;i++)if(!dead[i]){JSMapRecord*r=records[i];uint32_t bucket=map_hash_key(keys[i],4);r->hash_next=s->hash_table[bucket];s->hash_table[bucket]=r;}}
  map_delete_weakrefs(&rt,&s->weakref_header);weak_snapshot(&rt,&h,s);
  for(int i=0;i<64;i++){weakref_delete_weakref(&rt,&weakrefs[i]->weakref_header);num(JS_IsUndefined(weakrefs[i]->target),4);if(dead[i]&&i%3==0)map_decref_record(&rt,records[i]);}
  weak_snapshot(&rt,&h,s);
  for(int i=0;i<64;i++){JSObject holder={0};holder.class_id=JS_CLASS_WEAK_REF;holder.u.opaque=weakrefs[i];js_weakref_finalizer(&rt,JS_MKPTR(JS_TAG_OBJECT,&holder));}
  JSObject holder={0};holder.u.map_state=s;js_map_finalizer(&rt,JS_MKPTR(JS_TAG_OBJECT,&holder));
  for(int i=0;i<64;i++)if(!dead[i]){if(i%2)JS_FreeValueRT(&rt,keys[i]);else js_free_rt(&rt,JS_VALUE_GET_PTR(keys[i]));}
  num(list_empty(&rt.weakref_list),4);snapshot(&rt,&h);
 }
 num(js_weakref_is_target(JS_UNDEFINED),4);num(js_weakref_is_live(JS_UNDEFINED),4);num(JS_IsUndefined(js_weakref_new(&ctx,JS_UNDEFINED)),4);js_weakref_free(&rt,JS_UNDEFINED);
 cleanup(&rt,&h);return 0;
}
