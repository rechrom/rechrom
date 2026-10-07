static JSValue fn_generic(JSContext*ctx,JSValueConst self,int argc,JSValueConst*argv){return JS_UNDEFINED;}
static JSValue fn_data(JSContext*ctx,JSValueConst self,int argc,JSValueConst*argv,int magic,JSValue*data){return JS_UNDEFINED;}
static JSValue fn_get(JSContext*ctx,JSValueConst self){return JS_UNDEFINED;}
static JSValue fn_set(JSContext*ctx,JSValueConst self,JSValueConst v){return JS_UNDEFINED;}
static void dump_func(JSValue v){JSObject*p=JS_VALUE_GET_OBJ(v);dump_object(p);num(p->is_constructor,4);num(p->u.cfunc.length,4);num(p->u.cfunc.cproto,4);num(p->u.cfunc.magic,4);num(js_rc(p->u.cfunc.realm)->ref_count,4);}
static JSValue oracle_job(JSContext*ctx,int argc,JSValueConst*argv){int*count=ctx->user_opaque;(*count)++;if(argc>0&&JS_VALUE_GET_INT(argv[0])<0)return JS_Throw(ctx,JS_NewInt32(ctx,-123));return JS_NewInt32(ctx,argc);}
static void jobs_weak_fixtures(JSContext*ctx,JSClass*classes){
 JSRuntime*rt=ctx->rt;init_list_head(&rt->job_list);int count=0;ctx->user_opaque=&count;
 for(int i=-5;i<20;i++){JSValue args[]={JS_NewInt32(ctx,i),JS_NewString(ctx,"job")};num(JS_EnqueueJob(ctx,oracle_job,2,args),4);for(int j=0;j<2;j++)JS_FreeValue(ctx,args[j]);}num(JS_IsJobPending(rt),4);
 for(int i=0;i<26;i++){JSContext*selected=NULL;num(JS_ExecutePendingJob(rt,&selected),4);num(selected!=NULL,4);num(count,4);dump_exception(ctx);}
 classes[JS_CLASS_WEAK_REF].finalizer=js_weakref_finalizer;classes[JS_CLASS_FINALIZATION_REGISTRY].finalizer=js_finrec_finalizer;classes[JS_CLASS_FINALIZATION_REGISTRY].gc_mark=js_finrec_mark;
 for(int i=0;i<20;i++){JSValue target=JS_NewObject(ctx);JSValue args[]={target};JSValue wr=js_weakref_constructor(ctx,JS_TRUE,1,args);assert(!JS_IsException(wr));JSValue v=js_weakref_deref(ctx,wr,0,NULL);dump_val(v);JS_FreeValue(ctx,v);JS_FreeValue(ctx,target);JS_RunGC(rt);v=js_weakref_deref(ctx,wr,0,NULL);dump_val(v);JS_FreeValue(ctx,v);JS_FreeValue(ctx,wr);}
 JSValue cb=JS_NewCFunction2(ctx,fn_generic,"cb",1,JS_CFUNC_generic,0),args[]={cb};JSValue registry=js_finrec_constructor(ctx,JS_TRUE,1,args);assert(!JS_IsException(registry));JSValue token=JS_NewObject(ctx);
 for(int i=0;i<8;i++){JSValue target=JS_NewObject(ctx);JSValue args[]={target,JS_NewInt32(ctx,i),token};JSValue v=js_finrec_register(ctx,registry,3,args);dump_val(v);JS_FreeValue(ctx,v);JS_FreeValue(ctx,target);}JSValue args2[]={token};JSValue v=js_finrec_unregister(ctx,registry,1,args2);dump_val(v);JS_FreeValue(ctx,v);v=js_finrec_unregister(ctx,registry,1,args2);dump_val(v);JS_FreeValue(ctx,v);
 for(int i=0;i<8;i++){JSValue target=JS_NewObject(ctx);JSValue args[]={target,JS_NewInt32(ctx,i),token};JSValue v=js_finrec_register(ctx,registry,3,args);dump_val(v);JS_FreeValue(ctx,v);JS_FreeValue(ctx,target);}JS_FreeValue(ctx,token);JS_RunGC(rt);num(JS_IsJobPending(rt),4);
 struct list_head*el,*el1;list_for_each_safe(el,el1,&rt->job_list){JSJobEntry*e=list_entry(el,JSJobEntry,link);num(e->argc,4);for(int i=0;i<e->argc;i++){dump_val(e->argv[i]);JS_FreeValue(ctx,e->argv[i]);}JS_FreeContext(e->realm);list_del(&e->link);js_free_rt(rt,e);}JS_FreeValue(ctx,registry);JS_FreeValue(ctx,cb);ctx->user_opaque=NULL;
}

static void dump_array_fixture(JSContext*ctx,JSValue v){dump_val(v);if(JS_IsObject(v)){dump_object(JS_VALUE_GET_OBJ(v));JSValue l=JS_GetProperty(ctx,v,JS_ATOM_length);dump_val(l);int64_t n=0;JS_ToInt64(ctx,&n,l);JS_FreeValue(ctx,l);for(int64_t i=0;i<n;i++){JSValue x=JS_GetPropertyInt64(ctx,v,i);dump_val(x);JS_FreeValue(ctx,x);}}}
static void array_fixtures(JSContext*ctx,JSClass*classes){
 classes[JS_CLASS_ARRAY_ITERATOR].finalizer=js_array_iterator_finalizer;
 for(int kind=0;kind<3;kind++)for(int lengthi=0;lengthi<5;lengthi++)for(int indexi=0;indexi<5;indexi++)for(int method=0;method<24;method++){
 int length=(int[]){0,1,2,5,16}[lengthi],idx=(int[]){-20,-1,0,1,20}[indexi];
 JSValue obj=kind==2?JS_NewObject(ctx):JS_NewArray(ctx);
 for(int i=0;i<length;i++){if(kind==1&&i%3==1)continue;JSValue v;switch(i%6){case 0:v=JS_NewInt32(ctx,3-i);break;case 1:v=JS_NewString(ctx,"雪z");break;case 2:v=JS_UNDEFINED;break;case 3:v=JS_NULL;break;case 4:v=JS_NewFloat64(ctx,-0.0);break;default:v=JS_NewBool(ctx,1);break;}assert(JS_DefinePropertyValueUint32(ctx,obj,i,v,JS_PROP_C_W_E)>=0);}assert(JS_SetProperty(ctx,obj,JS_ATOM_length,JS_NewInt32(ctx,length))>=0);
 JSValue args[]={JS_NewInt32(ctx,idx),JS_NewInt32(ctx,2),JS_NewInt32(ctx,77)},ret=JS_UNDEFINED;
 switch(method){
case 0:{ret=js_array_at(ctx,obj,1,args);break;}
case 1:{ret=js_array_with(ctx,obj,2,args);break;}
case 2:{args[0]=JS_NewInt32(ctx,0);args[1]=JS_NewInt32(ctx,idx);ret=js_array_includes(ctx,obj,2,args);break;}
case 3:{args[0]=JS_NewInt32(ctx,0);args[1]=JS_NewInt32(ctx,idx);ret=js_array_indexOf(ctx,obj,2,args);break;}
case 4:{args[0]=JS_NewInt32(ctx,0);args[1]=JS_NewInt32(ctx,idx);ret=js_array_lastIndexOf(ctx,obj,2,args);break;}
case 5:{ret=js_array_fill(ctx,obj,3,args);break;}
case 6:{ret=js_array_pop(ctx,obj,0,args, 0);break;}
case 7:{ret=js_array_pop(ctx,obj,0,args, 1);break;}
case 8:{ret=js_array_push(ctx,obj,2,args, 0);break;}
case 9:{ret=js_array_push(ctx,obj,2,args, 1);break;}
case 10:{ret=js_array_reverse(ctx,obj,0,args);break;}
case 11:{ret=js_array_toReversed(ctx,obj,0,args);break;}
case 12:{ret=js_array_slice(ctx,obj,2,args, 0);break;}
case 13:{ret=js_array_slice(ctx,obj,3,args, 1);break;}
case 14:{ret=js_array_toSpliced(ctx,obj,3,args);break;}
case 15:{ret=js_array_copyWithin(ctx,obj,3,args);break;}
case 16:{args[0]=JS_UNDEFINED;ret=js_array_sort(ctx,obj,0,args);break;}
case 17:{args[0]=JS_UNDEFINED;ret=js_array_toSorted(ctx,obj,0,args);break;}
case 18:{args[0]=JS_NewString(ctx,"|雪|");ret=js_array_join(ctx,obj,1,args, 0);break;}
case 19:{ret=js_array_concat(ctx,obj,2,args);break;}
case 20:{ret=js_array_of(ctx,JS_UNDEFINED,3,args);break;}
case 21:{args[0]=obj;args[1]=JS_UNDEFINED;ret=js_array_from(ctx,JS_UNDEFINED,1,args);break;}
case 22:{args[0]=JS_NewInt32(ctx,2);ret=js_array_flatten(ctx,obj,1,args, 0);break;}
case 23:{args[0]=JS_UNDEFINED;ret=js_array_toString(ctx,obj,0,args);break;}
}
 num(kind,4);num(length,4);num(idx,4);num(method,4);dump_array_fixture(ctx,ret);dump_array_fixture(ctx,obj);dump_exception(ctx);if(method==18)JS_FreeValue(ctx,args[0]);JS_FreeValue(ctx,ret);JS_FreeValue(ctx,obj);
 }
}
static int promise_tracker_calls,promise_tracker_handled;
static void promise_fixture_tracker(JSContext*ctx,JSValueConst promise,JSValueConst value,BOOL handled,void*opaque){promise_tracker_calls++;promise_tracker_handled+=handled;num(handled,4);dump_val(value);}
static JSValue fixture_pending_promise(JSContext*ctx){JSValue obj=JS_NewObjectProtoClass(ctx,JS_NULL,JS_CLASS_PROMISE);JSPromiseData*s=js_mallocz(ctx,sizeof(*s));assert(s);s->promise_state=JS_PROMISE_PENDING;s->promise_result=JS_UNDEFINED;for(int i=0;i<2;i++)init_list_head(&s->promise_reactions[i]);JS_SetOpaque(obj,s);return obj;}
static void promise_fixtures(JSContext*ctx,JSClass*classes){
 JSRuntime*rt=ctx->rt;classes[JS_CLASS_PROMISE].finalizer=js_promise_finalizer;classes[JS_CLASS_PROMISE].gc_mark=js_promise_mark;for(int i=0;i<2;i++){classes[JS_CLASS_PROMISE_RESOLVE_FUNCTION+i].finalizer=js_promise_resolve_function_finalizer;classes[JS_CLASS_PROMISE_RESOLVE_FUNCTION+i].gc_mark=js_promise_resolve_function_mark;classes[JS_CLASS_PROMISE_RESOLVE_FUNCTION+i].call=js_promise_resolve_function_call;}
 JS_SetHostPromiseRejectionTracker(rt,promise_fixture_tracker,NULL);promise_tracker_calls=promise_tracker_handled=0;
 for(int kind=0;kind<8;kind++)for(int reject=0;reject<2;reject++)for(int before=0;before<3;before++)for(int argc=0;argc<2;argc++){
 JSValue promise=fixture_pending_promise(ctx),funcs[2],handlers[]={JS_UNDEFINED,JS_UNDEFINED},caps[]={JS_UNDEFINED,JS_UNDEFINED};assert(js_create_resolving_functions(ctx,funcs,promise)==0);
 for(int i=0;i<before;i++)num(perform_promise_then(ctx,promise,handlers,caps),4);
 JSValue value;switch(kind){case 0:value=JS_UNDEFINED;break;case 1:value=JS_NULL;break;case 2:value=JS_TRUE;break;case 3:value=JS_NewInt32(ctx,-99);break;case 4:value=JS_NewFloat64(ctx,-0.0);break;case 5:value=JS_NewString(ctx,"promise雪");break;case 6:value=JS_NewObject(ctx);break;default:value=JS_DupValue(ctx,promise);break;}
 for(int attempt=0;attempt<2;attempt++){JSValue ret=js_promise_resolve_function_call(ctx,funcs[(reject+attempt)%2],JS_UNDEFINED,argc,&value,0);dump_val(ret);JS_FreeValue(ctx,ret);num(JS_PromiseState(ctx,promise),4);JSValue result=JS_PromiseResult(ctx,promise);dump_val(result);JS_FreeValue(ctx,result);JSPromiseFunctionData*f=JS_VALUE_GET_OBJ(funcs[0])->u.promise_function_data;num(f->presolved->already_resolved,4);num(f->presolved->ref_count,4);dump_exception(ctx);}
 num(perform_promise_then(ctx,promise,handlers,caps),4);num(JS_IsJobPending(rt),4);while(JS_IsJobPending(rt)){JSContext*chosen=NULL;num(JS_ExecutePendingJob(rt,&chosen),4);num(chosen==ctx,4);dump_exception(ctx);}num(promise_tracker_calls,4);num(promise_tracker_handled,4);JS_FreeValue(ctx,value);JS_FreeValue(ctx,funcs[0]);JS_FreeValue(ctx,funcs[1]);JS_FreeValue(ctx,promise);JS_RunGC(rt);
 }
 JS_SetHostPromiseRejectionTracker(rt,NULL,NULL);
}
static void number_fixtures(JSContext*ctx,JSClass*classes){classes[JS_CLASS_NUMBER].finalizer=js_object_data_finalizer;classes[JS_CLASS_BOOLEAN].finalizer=js_object_data_finalizer;double nums[]={0.0,-0.0,1.0,-99.0,0.1,1e20,1e21,5e-324,1.7976931348623157e308,INFINITY,-INFINITY,NAN};for(int vi=0;vi<12;vi++)for(int box=0;box<3;box++)for(int di=0;di<9;di++)for(int method=0;method<13;method++){JSValue val=JS_NewFloat64(ctx,nums[vi]);if(box){JSValue obj=JS_NewObjectProtoClass(ctx,JS_NULL,box==1?JS_CLASS_NUMBER:JS_CLASS_BOOLEAN);assert(JS_SetObjectData(ctx,obj,box==1?val:JS_NewBool(ctx,JS_ToBool(ctx,val)))==0);val=obj;}JSValue digit=di==8?JS_UNDEFINED:JS_NewInt32(ctx,(int[]){-2,0,1,2,6,20,100,101}[di]),args[]={digit,JS_UNDEFINED,JS_UNDEFINED},ret=JS_UNDEFINED;switch(method){
case 0:{args[0]=val;ret=js_number_constructor(ctx,JS_UNDEFINED,1,args);break;}
case 1:{args[0]=val;ret=js_number_isNaN(ctx,JS_UNDEFINED,1,args);break;}
case 2:{args[0]=val;ret=js_number_isFinite(ctx,JS_UNDEFINED,1,args);break;}
case 3:{args[0]=val;ret=js_number_isInteger(ctx,JS_UNDEFINED,1,args);break;}
case 4:{args[0]=val;ret=js_number_isSafeInteger(ctx,JS_UNDEFINED,1,args);break;}
case 5:{ret=js_number_valueOf(ctx,val,1,args);break;}
case 6:{ret=js_number_toString(ctx,val,1,args, 0);break;}
case 7:{ret=js_number_toFixed(ctx,val,1,args);break;}
case 8:{ret=js_number_toExponential(ctx,val,1,args);break;}
case 9:{ret=js_number_toPrecision(ctx,val,1,args);break;}
case 10:{args[0]=val;ret=js_boolean_constructor(ctx,JS_UNDEFINED,1,args);break;}
case 11:{ret=js_boolean_toString(ctx,val,1,args);break;}
case 12:{ret=js_boolean_valueOf(ctx,val,1,args);break;}
}dump_val(ret);if(JS_IsObject(ret))dump_object(JS_VALUE_GET_OBJ(ret));dump_exception(ctx);JS_FreeValue(ctx,ret);JS_FreeValue(ctx,val);}
 const char*texts[]={""," -42xyz","0xFF","0b11","0o17","1e309"," -Infinity "," 1.25e2tail","雪","9007199254740993"," .5 ","-.0"};for(int ti=0;ti<12;ti++)for(int ri=0;ri<10;ri++){JSValue args[]={JS_NewString(ctx,texts[ti]),ri==9?JS_UNDEFINED:JS_NewInt32(ctx,(int[]){-1,0,1,2,8,10,16,36,37}[ri])};JSValue v=js_parseInt(ctx,JS_UNDEFINED,2,args);dump_val(v);dump_exception(ctx);JS_FreeValue(ctx,v);v=js_parseFloat(ctx,JS_UNDEFINED,1,args);dump_val(v);dump_exception(ctx);JS_FreeValue(ctx,v);JS_FreeValue(ctx,args[0]);}}
int main(void){
 JSRuntime storage={0},*rt=&storage;JSContext initial={0};struct host h={0};JSClass classes[JS_CLASS_INIT_COUNT]={0};JSValue protos[JS_CLASS_INIT_COUNT];for(int i=0;i<JS_CLASS_INIT_COUNT;i++)protos[i]=JS_NULL;
 setup(rt,&initial,&h,classes,protos);JSContext*ctx=js_mallocz_rt(rt,sizeof(*ctx));num(h.trace,8);memcpy(ctx,&initial,sizeof(*ctx));init_list_head(&ctx->loaded_modules);js_rc(ctx)->ref_count=1000;ctx->function_proto=JS_NULL;
 classes[JS_CLASS_C_FUNCTION].finalizer=js_c_function_finalizer;classes[JS_CLASS_C_FUNCTION_DATA].finalizer=js_c_function_data_finalizer;
 const char*names[]={"","f","abc","雪","ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789abcdefghi"};int lengths[]={0,1,255,256},magics[]={-32769,-1,0,65535},nfields[]={0,8};
 for(int n=0;n<5;n++)for(int cp=0;cp<13;cp++)for(int l=0;l<4;l++)for(int m=0;m<4;m++)for(int nf=0;nf<2;nf++){JSValue v=JS_NewCFunction3(ctx,fn_generic,names[n],lengths[l],cp,magics[m],JS_NULL,nfields[nf]);assert(!JS_IsException(v));dump_func(v);JS_FreeValue(ctx,v);}
 JSValue data[]={JS_NewInt32(ctx,123),JS_NewString(ctx,"data"),JS_NULL};for(int n=0;n<4;n++){JSValue v=JS_NewCFunctionData(ctx,fn_data,257,-1,n,data);dump_object(JS_VALUE_GET_OBJ(v));JSCFunctionDataRecord*s=JS_GetOpaque(v,JS_CLASS_C_FUNCTION_DATA);num(s->length,4);num(s->data_len,4);num(s->magic,4);for(int i=0;i<n;i++)dump_val(s->data[i]);JS_FreeValue(ctx,v);}for(int i=0;i<3;i++)JS_FreeValue(ctx,data[i]);
 JSCFunctionListEntry sub[]={JS_PROP_INT32_DEF("answer",42,JS_PROP_C_W_E)};
 JSCFunctionListEntry fields[]={JS_CFUNC_DEF("run",2,fn_generic),JS_PROP_STRING_DEF("str","value",JS_PROP_C_W_E),JS_PROP_INT32_DEF("int",-17,JS_PROP_C_W_E),JS_PROP_INT64_DEF("large",9007199254740991LL,JS_PROP_C_W_E),JS_PROP_DOUBLE_DEF("dbl",-0.0,JS_PROP_C_W_E),JS_PROP_UNDEFINED_DEF("undef",JS_PROP_C_W_E),JS_PROP_BOOL_DEF("bool",1,JS_PROP_C_W_E),JS_PROP_ATOM_DEF("atom",JS_ATOM_name,JS_PROP_C_W_E),JS_OBJECT_DEF("nested",sub,1,JS_PROP_C_W_E),JS_CGETSET_DEF("access",fn_get,fn_set),JS_ALIAS_DEF("alias","str")};
 JSValue obj=JS_NewObject(ctx);num(JS_SetPropertyFunctionList(ctx,obj,fields,countof(fields)),4);for(int i=0;i<countof(fields);i++){JSCFunctionListEntry*e=fields+i;JSAtom atom=find_atom(ctx,e->name);if(e->def_type!=JS_DEF_CGETSET){JSValue v=JS_GetProperty(ctx,obj,atom);dump_val(v);if(JS_IsObject(v))dump_object(JS_VALUE_GET_OBJ(v));JS_FreeValue(ctx,v);}JS_FreeAtom(ctx,atom);}dump_object(JS_VALUE_GET_OBJ(obj));JS_FreeValue(ctx,obj);
 ctx->global_obj=JS_NewObject(ctx);for(int flags=0;flags<16;flags++){if(flags&JS_NEW_CTOR_PROTO_EXIST)protos[JS_CLASS_OBJECT]=JS_NewObjectProtoClass(ctx,JS_NULL,JS_CLASS_OBJECT);JSValue ctor=JS_NewCConstructor(ctx,JS_CLASS_OBJECT,"Custom",fn_generic,2,JS_CFUNC_constructor_or_func,7,JS_UNDEFINED,NULL,0,NULL,0,flags);assert(!JS_IsException(ctor));dump_func(ctor);JSValue proto=JS_GetProperty(ctx,ctor,JS_ATOM_prototype);dump_object(JS_VALUE_GET_OBJ(proto));num(delete_property(ctx,JS_VALUE_GET_OBJ(proto),JS_ATOM_constructor),4);if(!(flags&JS_NEW_CTOR_NO_GLOBAL)){JSAtom a=JS_NewAtom(ctx,"Custom");delete_property(ctx,JS_VALUE_GET_OBJ(ctx->global_obj),a);JS_FreeAtom(ctx,a);}JS_FreeValue(ctx,proto);JS_FreeValue(ctx,ctor);JS_FreeValue(ctx,protos[JS_CLASS_OBJECT]);protos[JS_CLASS_OBJECT]=JS_NULL;}
 JS_FreeValue(ctx,ctx->global_obj);jobs_weak_fixtures(ctx,classes);array_fixtures(ctx,classes);promise_fixtures(ctx,classes);number_fixtures(ctx,classes);num(js_rc(ctx)->ref_count,4);assert(js_rc(ctx)->ref_count==1000);num(h.calls,4);num(h.live,4);num(h.trace,8);js_free_shape(rt,ctx->array_shape);js_free_rt(rt,ctx);assert(rt->shape_hash_count==0);assert(list_empty(&rt->gc_obj_list));js_free_rt(rt,rt->shape_hash);cleanup(rt,&h);assert(h.live==0);return 0;
}
