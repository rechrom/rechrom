/* Header-only independent oracle; never linked to the Rust runtime. */
#include <stdio.h>
#include <math.h>
#include <stddef.h>
#include "quickjs.h"
static void num(uint64_t n,int size){for(int i=0;i<size;i++)putchar((n>>(8*i))&255);}
static void value(JSValue v){
#ifdef JS_PTR64
 num(v.u.uint64,8);num(v.tag,8);
#else
 num(v,8);
#endif
 num(JS_VALUE_GET_NORM_TAG(v),4);num(JS_VALUE_IS_NAN(v),4);num(JS_VALUE_HAS_REF_COUNT(v),4);
 num(JS_IsNumber(v),4);num(JS_IsBigInt(NULL,v),4);num(JS_IsBool(v),4);num(JS_IsNull(v),4);num(JS_IsUndefined(v),4);num(JS_IsException(v),4);num(JS_IsUninitialized(v),4);num(JS_IsString(v),4);num(JS_IsSymbol(v),4);num(JS_IsObject(v),4);
 JSValue other=JS_FALSE;num(JS_VALUE_IS_BOTH_INT(v,other),4);num(JS_VALUE_IS_BOTH_INT(v,JS_NewInt32(NULL,1)),4);num(JS_VALUE_IS_BOTH_FLOAT(v,__JS_NewFloat64(NULL,1.5)),4);
}
static void floats(uint64_t bits){union{uint64_t bits;double d;}u={.bits=bits};value(__JS_NewFloat64(NULL,u.d));value(JS_NewFloat64(NULL,u.d));}
int main(void){
#define SIZE(T) num(sizeof(T),4);num(_Alignof(T),4)
#define OFF(T,F) num(offsetof(T,F),4)
 SIZE(JSValue);SIZE(JSRefCountHeader);SIZE(JSMallocState);SIZE(JSMallocFunctions);SIZE(JSPropertyEnum);SIZE(JSPropertyDescriptor);SIZE(JSSharedArrayBufferFunctions);
 OFF(JSValue,u);OFF(JSValue,tag);OFF(JSMallocState,opaque);OFF(JSPropertyDescriptor,value);OFF(JSPropertyDescriptor,getter);OFF(JSPropertyDescriptor,setter);OFF(JSSharedArrayBufferFunctions,sab_opaque);
 num(JS_LIMB_BITS,4);num(JS_SHORT_BIG_INT_BITS,4);
#include "quickjs_header_constants.inc"
 SIZE(JSMemoryUsage);SIZE(JSClassExoticMethods);SIZE(JSClassDef);SIZE(JSCFunctionType);SIZE(JSCFunctionListEntry);SIZE(JSPrintValueOptions);
 OFF(JSMemoryUsage,binary_object_size);OFF(JSClassDef,exotic);OFF(JSCFunctionListEntry,u);OFF(JSCFunctionListEntry,magic);OFF(JSPrintValueOptions,max_depth);
 const int enums[]={JS_CFUNC_generic,JS_CFUNC_generic_magic,JS_CFUNC_constructor,JS_CFUNC_constructor_magic,JS_CFUNC_constructor_or_func,JS_CFUNC_constructor_or_func_magic,JS_CFUNC_f_f,JS_CFUNC_f_f_f,JS_CFUNC_getter,JS_CFUNC_setter,JS_CFUNC_getter_magic,JS_CFUNC_setter_magic,JS_CFUNC_iterator_next,JS_PROMISE_PENDING,JS_PROMISE_FULFILLED,JS_PROMISE_REJECTED};for(int i=0;i<16;i++)num(enums[i],4);
 const int flags[]={-129,-128,-1,0,1,127,128,255,256};for(int i=0;i<9;i++)for(int j=0;j<9;j++){JSPrintValueOptions p={0};p.show_hidden=flags[i];p.raw_dump=flags[j];num(p.show_hidden,4);num(p.raw_dump,4);uint32_t raw;memcpy(&raw,&p,4);num(raw&65535,4);}
 const JSCFunctionListEntry entries[]={
  JS_CFUNC_DEF("entry",2,NULL),JS_CFUNC_MAGIC_DEF("entry",3,NULL,-42),JS_CFUNC_SPECIAL_DEF("entry",4,f_f,NULL),JS_ITERATOR_NEXT_DEF("entry",5,NULL,17),JS_CGETSET_DEF("entry",NULL,NULL),JS_CGETSET_MAGIC_DEF("entry",NULL,NULL,-7),JS_PROP_STRING_DEF("entry","text",7),JS_PROP_INT32_DEF("entry",INT32_MIN,3),JS_PROP_INT64_DEF("entry",INT64_MIN,5),JS_PROP_DOUBLE_DEF("entry",-0.0,6),JS_PROP_UNDEFINED_DEF("entry",2),JS_PROP_ATOM_DEF("entry",123,1),JS_PROP_BOOL_DEF("entry",1,7),JS_OBJECT_DEF("entry",NULL,3,6),JS_ALIAS_DEF("entry","target"),JS_ALIAS_BASE_DEF("entry","target",2),JS_PROP_DOUBLE_DEF("entry",NAN,0)
 };
 for(size_t i=0;i<sizeof(entries)/sizeof(entries[0]);i++){
  const JSCFunctionListEntry*e=&entries[i];num(strlen(e->name),4);fwrite(e->name,1,strlen(e->name),stdout);num(e->prop_flags,4);num(e->def_type,4);num(e->magic,4);
  switch(e->def_type){
   case JS_DEF_CFUNC:num(e->u.func.length,4);num(e->u.func.cproto,4);num(e->u.func.cfunc.generic!=NULL,4);break;
   case JS_DEF_CGETSET:case JS_DEF_CGETSET_MAGIC:num(e->u.getset.get.generic!=NULL,4);num(e->u.getset.set.generic!=NULL,4);break;
   case JS_DEF_PROP_STRING:num(strlen(e->u.str),4);fwrite(e->u.str,1,strlen(e->u.str),stdout);break;
   case JS_DEF_PROP_INT64:num(e->u.i64,8);break;
   case JS_DEF_PROP_DOUBLE:{uint64_t bits;memcpy(&bits,&e->u.f64,8);num(bits,8);break;}
   case JS_DEF_OBJECT:num(e->u.prop_list.tab!=NULL,4);num(e->u.prop_list.len,4);break;
   case JS_DEF_ALIAS:num(strlen(e->u.alias.name),4);fwrite(e->u.alias.name,1,strlen(e->u.alias.name),stdout);num(e->u.alias.base,4);break;
   default:num(e->u.i32,4);break;
  }
 }
 value(JS_NULL);value(JS_UNDEFINED);value(JS_FALSE);value(JS_TRUE);value(JS_EXCEPTION);value(JS_UNINITIALIZED);value(JS_NAN);
 const int32_t ints[]={INT32_MIN,-1,0,1,INT32_MAX};
 for(int i=0;i<5;i++){value(JS_NewBool(NULL,ints[i]));value(JS_NewInt32(NULL,ints[i]));value(JS_NewCatchOffset(NULL,ints[i]));value(JS_NewUint32(NULL,(uint32_t)ints[i]));}
 const int64_t bigs[]={INT64_MIN,INT64_MIN+1,-9007199254740993,-2147483649,INT32_MIN,-1,0,INT32_MAX,2147483648,9007199254740991,9007199254740993,INT64_MAX};
 for(size_t i=0;i<sizeof(bigs)/sizeof(bigs[0]);i++){value(JS_NewInt64(NULL,bigs[i]));value(__JS_NewShortBigInt(NULL,bigs[i]));}
 for(int tag=-16;tag<=16;tag++)for(int i=0;i<5;i++)value(JS_MKVAL(tag,ints[i]));
 for(int tag=-9;tag<=8;tag++)value(JS_MKPTR(tag,(void*)(uintptr_t)0x123456789abc));
 const uint64_t boundaries[]={0,0x8000000000000000,1,0x8000000000000001,0x000fffffffffffff,0x0010000000000000,0x3ff0000000000000,0xbff0000000000000,0x41dfffffffc00000,0x41e0000000000000,0xc1e0000000000000,0xc1e0000000200000,0x7fefffffffffffff,0x7ff0000000000000,0xfff0000000000000,0x7ff0000000000001,0xfff0000000000001,0x7ff8000000000000,0x7fffffffffffffff,0xfff8000000000042};
 for(size_t i=0;i<sizeof(boundaries)/8;i++)for(int offset=-1;offset<=1;offset++)floats(boundaries[i]+offset);
 uint64_t x=0x123456789abcdef0;for(int i=0;i<100000;i++){x^=x<<13;x^=x>>7;x^=x<<17;floats(x);}
 for(uint64_t x=0;x<=65535;x++)floats(x<<48);
 struct{int32_t rc;uint32_t payload;}cell={1,0};
 for(int tag=-9;tag<=-1;tag++){cell.rc=1;JSValue v=JS_MKPTR(tag,&cell.payload);JSValue dup=JS_DupValue(NULL,v);num(cell.rc,4);num(JS_VALUE_GET_PTR(dup)==&cell.payload,4);dup=JS_DupValueRT(NULL,v);num(cell.rc,4);num(JS_VALUE_GET_PTR(dup)==&cell.payload,4);}
 return 0;
}
