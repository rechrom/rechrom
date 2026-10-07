#include "quickjs.c"
static void vm_dump(JSRuntime *rt, JSValue v) {
    uint32_t tag=JS_VALUE_GET_NORM_TAG(v), frame=rt->current_stack_frame==NULL;
    uint64_t bits;
    if ((int32_t)tag==JS_TAG_FLOAT64) { double d=JS_VALUE_GET_FLOAT64(v); memcpy(&bits,&d,8); }
    else bits=(uint32_t)JS_VALUE_GET_INT(v);
    fwrite(&tag,4,1,stdout); fwrite(&bits,8,1,stdout); fwrite(&frame,4,1,stdout);
}
static JSValue vm_native_generic(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
    JSStackFrame *sf=ctx->rt->current_stack_frame;
    int n=argc*100000+sf->arg_count*1000+JS_VALUE_GET_INT(this_val)*10;
    for(int i=0;i<3;i++) n+=JS_VALUE_GET_INT(argv[i]);
    return JS_NewInt32(ctx,n);
}
static JSValue vm_native_magic(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv,int magic) { return JS_NewInt32(ctx,JS_VALUE_GET_INT(vm_native_generic(ctx,this_val,argc,argv))+magic); }
static JSValue vm_native_getter(JSContext *ctx,JSValueConst v){return JS_NewInt32(ctx,JS_VALUE_GET_INT(v)+31);}
static JSValue vm_native_setter(JSContext *ctx,JSValueConst v,JSValueConst arg){return JS_NewInt32(ctx,JS_VALUE_GET_INT(v)+JS_VALUE_GET_INT(arg)+17);}
static JSValue vm_native_getter_magic(JSContext *ctx,JSValueConst v,int magic){return JS_NewInt32(ctx,JS_VALUE_GET_INT(v)+magic);}
static JSValue vm_native_setter_magic(JSContext *ctx,JSValueConst v,JSValueConst arg,int magic){return JS_NewInt32(ctx,JS_VALUE_GET_INT(v)+JS_VALUE_GET_INT(arg)+magic);}
static double vm_native_f_f(double x){double y=x*x-1.0;return isnan(y)?NAN:y;}
static double vm_native_f_f_f(double x,double y){double z=x*x-y;return isnan(z)?NAN:z;}
static JSValue vm_native_iterator(JSContext *ctx,JSValueConst v,int argc,JSValueConst *argv,int *done,int magic){*done=2;return vm_native_magic(ctx,v,argc,argv,magic);}
static JSValue vm_native_bound(JSContext *ctx, JSValueConst this_val, int argc, JSValueConst *argv) {
 int marker=JS_IsObject(this_val)?JS_VALUE_GET_OBJ(this_val)->class_id:JS_VALUE_GET_INT(this_val);
 JSStackFrame *sf=ctx->rt->current_stack_frame;
 int n=argc*100000+sf->arg_count*1000+marker*10;
 for(int i=0;i<argc;i++)n+=JS_VALUE_GET_INT(argv[i])*(i+1);
 return JS_NewInt32(ctx,n);
}
static JSValue vm_native_data(JSContext *ctx,JSValueConst this_val,int argc,JSValueConst *argv,int magic,JSValue *data) {
 int n=argc*100000+magic+JS_VALUE_GET_INT(this_val)*10+(ctx->rt->current_stack_frame==NULL)*1000000
       +JS_VALUE_GET_INT(data[0])*3+JS_VALUE_GET_INT(data[1])*5;
 for(int i=0;i<3;i++)n+=JS_VALUE_GET_INT(argv[i])*(i+1);
 return JS_NewInt32(ctx,n);
}
int main(void) {
 JSRuntime rt={0}; JSContext ctx={0}; JSFunctionBytecode b={0}; JSObject p={0}; JSClass classes[JS_CLASS_INIT_COUNT]={0};
 ctx.rt=&rt; rt.current_exception=JS_UNINITIALIZED;
 classes[JS_CLASS_C_FUNCTION].call=js_call_c_function;
 rt.class_array=classes;rt.class_count=JS_CLASS_INIT_COUNT;
 p.class_id=JS_CLASS_BYTECODE_FUNCTION;p.u.func.function_bytecode=&b;b.realm=&ctx;b.stack_size=8;b.var_count=2;
 JSValue function=JS_MKPTR(JS_TAG_OBJECT,&p);
 uint32_t seed=0x84739823;
 uint8_t bin_ops[]={OP_add,OP_sub,OP_mul,OP_shl,OP_sar,OP_and,OP_xor,OP_or,OP_div,OP_mod,OP_shr,OP_lt,OP_lte,OP_gt,OP_gte,OP_eq,OP_neq,OP_strict_eq,OP_strict_neq};
 uint8_t unary_ops[]={OP_neg,OP_not,OP_lnot};
 for(uint32_t i=0;i<32768;i++) {
  seed^=seed<<13;seed^=seed>>17;seed^=seed<<5;
  int32_t x=i%16==0?(int32_t)i-100:(int32_t)seed;
  seed=seed*1664525u+1013904223u;int32_t y=seed;
  for(int j=0;j<(int)sizeof(bin_ops);j++) {
   uint8_t code[12]={OP_push_i32};memcpy(code+1,&x,4);code[5]=OP_push_i32;memcpy(code+6,&y,4);code[10]=bin_ops[j];code[11]=OP_return;
   b.byte_code_buf=code;b.byte_code_len=sizeof(code);ctx.interrupt_counter=10000;
   JSValue v=JS_Call(&ctx,function,JS_UNDEFINED,0,NULL);vm_dump(&rt,v);JS_FreeValue(&ctx,v);
  }
  for(int j=0;j<3;j++) {
   uint8_t code[7]={OP_push_i32};memcpy(code+1,&x,4);code[5]=unary_ops[j];code[6]=OP_return;
   b.byte_code_buf=code;b.byte_code_len=sizeof(code);ctx.interrupt_counter=10000;
   JSValue v=JS_Call(&ctx,function,JS_UNDEFINED,0,NULL);vm_dump(&rt,v);JS_FreeValue(&ctx,v);
  }
  uint8_t code[16]={OP_push_i32};memcpy(code+1,&x,4);
  uint8_t rest[]={OP_put_loc,0,0,OP_inc_loc,0,OP_dec_loc,0,OP_get_loc,0,0,OP_return};memcpy(code+5,rest,sizeof(rest));
  b.byte_code_buf=code;b.byte_code_len=sizeof(code);ctx.interrupt_counter=10000;
  JSValue v=JS_Call(&ctx,function,JS_UNDEFINED,0,NULL);vm_dump(&rt,v);JS_FreeValue(&ctx,v);
 }
#if SHORT_OPCODES
 for(int op=OP_push_minus1;op<=OP_push_7;op++) {
  uint8_t code[]={op,OP_return};b.byte_code_buf=code;b.byte_code_len=sizeof(code);ctx.interrupt_counter=10000;
  JSValue v=JS_Call(&ctx,function,JS_UNDEFINED,0,NULL);vm_dump(&rt,v);JS_FreeValue(&ctx,v);
 }
 uint8_t short_codes[][7]={
  {OP_push_i8,128,OP_return}, {OP_push_i16,0,128,OP_return},
  {OP_push_7,OP_put_loc0,OP_get_loc0,OP_return},
  {OP_push_i8,238,OP_put_loc8,0,OP_get_loc8,0,OP_return},
  {OP_push_0,OP_if_false8,3,OP_push_1,OP_return,OP_push_7,OP_return},
  {OP_push_1,OP_if_false8,3,OP_push_1,OP_return,OP_push_7,OP_return}
 };
 const int short_lengths[]={3,4,4,7,7,7};
 for(int k=0;k<6;k++) {
  b.byte_code_buf=short_codes[k];b.byte_code_len=short_lengths[k];ctx.interrupt_counter=10000;
  JSValue v=JS_Call(&ctx,function,JS_UNDEFINED,0,NULL);vm_dump(&rt,v);JS_FreeValue(&ctx,v);
 }
#endif
 p.class_id=JS_CLASS_C_FUNCTION;p.u.cfunc.realm=&ctx;p.u.cfunc.length=3;p.u.cfunc.magic=-173;
 JSValue args[]={JS_NewInt32(&ctx,11),JS_NewInt32(&ctx,31),JS_NewInt32(&ctx,-17),JS_NewInt32(&ctx,9),JS_NewInt32(&ctx,27)};
 for(int cproto=0;cproto<=JS_CFUNC_iterator_next;cproto++) {
  p.u.cfunc.cproto=cproto;
  switch(cproto){
  case JS_CFUNC_generic:case JS_CFUNC_constructor:case JS_CFUNC_constructor_or_func:p.u.cfunc.c_function.generic=vm_native_generic;break;
  case JS_CFUNC_generic_magic:case JS_CFUNC_constructor_magic:case JS_CFUNC_constructor_or_func_magic:p.u.cfunc.c_function.generic_magic=vm_native_magic;break;
  case JS_CFUNC_f_f:p.u.cfunc.c_function.f_f=vm_native_f_f;break;
  case JS_CFUNC_f_f_f:p.u.cfunc.c_function.f_f_f=vm_native_f_f_f;break;
  case JS_CFUNC_getter:p.u.cfunc.c_function.getter=vm_native_getter;break;
  case JS_CFUNC_setter:p.u.cfunc.c_function.setter=vm_native_setter;break;
  case JS_CFUNC_getter_magic:p.u.cfunc.c_function.getter_magic=vm_native_getter_magic;break;
  case JS_CFUNC_setter_magic:p.u.cfunc.c_function.setter_magic=vm_native_setter_magic;break;
  case JS_CFUNC_iterator_next:p.u.cfunc.c_function.iterator_next=vm_native_iterator;break;
  }
  for(int argc=0;argc<=5;argc++) {
   ctx.interrupt_counter=10000;
   JSValue v=js_call_c_function(&ctx,function,JS_NewInt32(&ctx,4),argc,args,JS_CALL_FLAG_CONSTRUCTOR);
   vm_dump(&rt,v);JS_FreeValue(&ctx,v);
  }
 }
 p.is_constructor=1;p.u.cfunc.cproto=JS_CFUNC_constructor_or_func;p.u.cfunc.c_function.generic=vm_native_bound;
 union { uint64_t align; uint8_t bytes[sizeof(JSBoundFunction)+2*sizeof(JSValue)]; } storage={0};
 JSBoundFunction *bf=(JSBoundFunction *)storage.bytes;
 bf->func_obj=function;bf->this_val=JS_NewInt32(&ctx,19);bf->argc=2;bf->argv[0]=JS_NewInt32(&ctx,-7);bf->argv[1]=JS_NewInt32(&ctx,18);
 struct {JSMallocBlockHeader header;JSObject object;} bound_store={0},other_store={0};
 JSObject *bound=&bound_store.object,*other=&other_store.object;
 js_rc(bound)->ref_count=1;js_rc(other)->ref_count=1;
 bound->class_id=JS_CLASS_BOUND_FUNCTION;bound->u.bound_function=bf;other->class_id=JS_CLASS_OBJECT;
 JSValue bound_val=JS_MKPTR(JS_TAG_OBJECT,bound),other_val=JS_MKPTR(JS_TAG_OBJECT,other);
 for(int argc=0;argc<=5;argc++) {
  JSValue targets[]={JS_UNDEFINED,bound_val,other_val};
  for(int j=0;j<3;j++) {
   ctx.interrupt_counter=10000;int flags=JS_IsUndefined(targets[j])?0:JS_CALL_FLAG_CONSTRUCTOR;
   JSValue v=js_call_bound_function(&ctx,bound_val,targets[j],argc,args,flags);vm_dump(&rt,v);JS_FreeValue(&ctx,v);
  }
 }
 union {uint64_t align;uint8_t bytes[sizeof(JSCFunctionDataRecord)+2*sizeof(JSValue)];} data_storage={0};
 JSCFunctionDataRecord *data=(JSCFunctionDataRecord *)data_storage.bytes;
 data->func=vm_native_data;data->length=3;data->magic=-173;data->data_len=2;
 data->data[0]=JS_NewInt32(&ctx,72);data->data[1]=JS_NewInt32(&ctx,-11);
 classes[JS_CLASS_C_FUNCTION_DATA].call=js_c_function_data_call;p.class_id=JS_CLASS_C_FUNCTION_DATA;p.u.opaque=data;
 for(int argc=0;argc<=5;argc++) {
  ctx.interrupt_counter=10000;JSValue v=JS_Call(&ctx,function,JS_NewInt32(&ctx,4),argc,args);vm_dump(&rt,v);JS_FreeValue(&ctx,v);
 }
 return 0;
}
