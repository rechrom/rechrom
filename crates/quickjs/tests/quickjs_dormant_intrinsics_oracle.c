/* Official default source untouched; six exact dormant functions appended. */
#include "quickjs.c"
#include "quickjs_dormant_intrinsics_original.c"
static void u32(uint32_t x){for(int i=0;i<4;i++)putchar((x>>(i*8))&255);}
static void dormant_dump(JSContext*ctx,JSValue value){u32(JS_VALUE_GET_NORM_TAG(value));JSValue val=JS_IsException(value)?JS_GetException(ctx):JS_DupValue(ctx,value);size_t len=0;const char*encoded=JS_ToCStringLen2(ctx,&len,val,0);assert(encoded);u32(len);fwrite(encoded,1,len,stdout);JS_FreeCString(ctx,encoded);JS_FreeValue(ctx,val);}
int main(void){JSRuntime*rt=JS_NewRuntime();assert(rt);JSContext*ctx=JS_NewContext(rt);assert(ctx);
 double numbers[]={0.0,-0.0,0.1,-0.1,1.9,-1.9,5e-324,1e20,9007199254740991.0,9007199254740992.0,INFINITY,-INFINITY,NAN};
 JSCFunction*helpers[]={js_number___toInteger,js_number___toLength,js_string___isSpace,js___date_getTimezoneOffset};
 for(int n=0;n<13;n++){JSValue arg=JS_NewFloat64(ctx,numbers[n]);for(int h=0;h<4;h++){JSValue val=helpers[h](ctx,JS_UNDEFINED,1,&arg);dormant_dump(ctx,val);JS_FreeValue(ctx,val);}}
 for(int c=-1;c<=0x10000;c++){JSValue arg=JS_NewInt32(ctx,c);JSValue val=js_string___isSpace(ctx,JS_UNDEFINED,1,&arg);putchar(JS_VALUE_GET_INT(val));JS_FreeValue(ctx,val);}
 JSValue def_proto=JS_NewObject(ctx);const char*texts[]={"({})","({prototype:1})","({prototype:null})","({prototype:{x:42}})","({get prototype(){return {x:43}}})","({get prototype(){throw new Error('proto')}})"};
 for(int i=0;i<6;i++){JSValue ctor=JS_Eval(ctx,texts[i],strlen(texts[i]),"dormant.js",0);assert(!JS_IsException(ctor));JSValue proto=js_get_prototype_from_ctor(ctx,ctor,def_proto);dormant_dump(ctx,proto);putchar(JS_VALUE_GET_PTR(proto)==JS_VALUE_GET_PTR(def_proto));JS_FreeValue(ctx,proto);JSValue data=JS_NewString(ctx,"retained-date-data");JSValue args[]={ctor,def_proto,data};JSValue obj=js___date_create(ctx,JS_UNDEFINED,3,args);u32(JS_VALUE_GET_NORM_TAG(obj));if(JS_IsException(obj))dormant_dump(ctx,obj);if(JS_IsObject(obj)){u32(JS_VALUE_GET_OBJ(obj)->class_id);dormant_dump(ctx,JS_VALUE_GET_OBJ(obj)->u.object_data);}JS_FreeValue(ctx,obj);JS_FreeValue(ctx,data);JS_FreeValue(ctx,ctor);}
 JS_FreeValue(ctx,def_proto);JS_FreeContext(ctx);JS_FreeRuntime(rt);return 0;}
