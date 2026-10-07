static uint64_t rng=0xabcdef0123456789ULL;
static uint64_t next(void){rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;return rng;}
int main(void){
 JSRuntime rt={0};struct host h={0};h.trace=1469598103934665603ULL;js_malloc_init(&rt.malloc_ctx);rt.malloc_ctx.mf=(JSMallocFunctions){host_malloc,host_free,host_realloc,NULL};rt.malloc_ctx.malloc_state.opaque=&h;JS_InitAtoms(&rt);JSContext ctx={0};ctx.rt=&rt;
 for(int i=0;i<32768;i++){
  uint64_t bits=next();JSValue vs[]={JS_NewInt32(&ctx,bits),JS_MKVAL(JS_TAG_BOOL,bits),JS_NULL,JS_UNDEFINED,JS_EXCEPTION,__JS_NewShortBigInt(&ctx,bits),__JS_NewFloat64(&ctx,uint64_as_float64(bits)),JS_MKVAL(JS_TAG_UNINITIALIZED,0)};
  for(int j=0;j<8;j++){num(JS_ToBool(&ctx,vs[j]),4);num(JS_ToBoolFree(&ctx,vs[j]),4);}num(is_safe_integer(uint64_as_float64(bits)),4);num(is_digit((int)bits),4);num(to_digit((int)bits),4);
  uint16_t units[5]={(uint16_t)bits,1,2,0,4};JSString*p=make_string(&rt,units,i%6,i%2);JSValue v=JS_MKPTR(JS_TAG_STRING,p);num(JS_ToBool(&ctx,v),4);num(js_rc(p)->ref_count,4);num(JS_ToBoolFree(&ctx,v),4);
  JSStringRope*rope=js_mallocz_rt(&rt,sizeof(*rope));js_rc(rope)->ref_count=1;rope->left=JS_MKPTR(JS_TAG_STRING,make_string(&rt,units,i%3,i%2));rope->right=JS_MKPTR(JS_TAG_STRING,make_string(&rt,units,i%3,i%2));rope->len=(i%3)*2;v=JS_MKPTR(JS_TAG_STRING_ROPE,rope);num(JS_ToBool(&ctx,v),4);num(js_rc(rope)->ref_count,4);num(JS_ToBoolFree(&ctx,v),4);
  JSBigInt*b=js_mallocz_rt(&rt,sizeof(*b)+8*sizeof(js_limb_t));js_rc(b)->ref_count=1;b->len=8;for(int j=0;j<8;j++)b->tab[j]=i%3==0?0:next();v=JS_MKPTR(JS_TAG_BIG_INT,b);num(JS_ToBool(&ctx,v),4);num(js_rc(b)->ref_count,4);num(JS_ToBoolFree(&ctx,v),4);
  JSObject*o=js_mallocz_rt(&rt,sizeof(*o));js_rc(o)->ref_count=100;o->is_HTMLDDA=i%2;v=JS_MKPTR(JS_TAG_OBJECT,o);num(JS_ToBool(&ctx,v),4);num(js_rc(o)->ref_count,4);num(JS_ToBoolFree(&ctx,v),4);num(js_rc(o)->ref_count,4);js_free_rt(&rt,o);
  JSAtom atom=__JS_NewAtomInit(&rt,"symbol",6,JS_ATOM_TYPE_SYMBOL);p=rt.atom_array[atom];v=JS_MKPTR(JS_TAG_SYMBOL,p);num(JS_ToBool(&ctx,v),4);num(js_rc(p)->ref_count,4);num(JS_ToBoolFree(&ctx,v),4);
 }
 for(int cp=0;cp<0x110000;cp++){uint8_t text[32]={0};text[0]=' ';int n=unicode_to_utf8(text+1,cp);text[n+1]='\t';text[n+2]='x';num(skip_spaces((const char*)text),4);}
 for(int c=-1024;c<=1024;c++){num(to_digit(c),4);num(is_digit(c),4);}
 for(int trial=0;trial<64;trial++){
  DynBuf db;js_dbuf_bytecode_init(&ctx,&db);num(db.opaque==&rt,4);num(db.realloc_func==js_realloc_bytecode_rt,4);num(db.size,4);num(db.allocated_size,4);num(db.error,4);num(db.buf==NULL,4);
  void*p=js_realloc_bytecode_rt(&rt,NULL,32);num(p!=NULL,4);if(p){memset(p,trial,32);num(js_realloc_bytecode_rt(&rt,p,(size_t)INT32_MAX/2+1)==NULL,4);num(((uint8_t*)p)[0],4);p=js_realloc_bytecode_rt(&rt,p,1024);num(p!=NULL,4);num(((uint8_t*)p)[0],4);js_realloc_bytecode_rt(&rt,p,0);}js_dbuf_init(&ctx,&db);num(db.opaque==&rt,4);num(db.realloc_func==(DynBufReallocFunc*)js_realloc_rt,4);
 }
 cleanup(&rt,&h);return 0;
}
