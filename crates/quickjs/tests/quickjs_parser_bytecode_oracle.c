#include "quickjs_parser_compiler_cases.h"
#include "quickjs_parser_module_cases.h"
static void dump_compiled(JSValue val){
 if(JS_VALUE_GET_TAG(val)==JS_TAG_MODULE){
 num(JS_TAG_MODULE,4);JSModuleDef*m=JS_VALUE_GET_PTR(val);int ns[]={m->module_name,m->has_tla,m->resolved,m->func_created,m->status,m->req_module_entries_count,m->export_entries_count,m->star_export_entries_count,m->import_entries_count};for(int j=0;j<countof(ns);j++)num(ns[j],4);
 for(int i=0;i<m->req_module_entries_count;i++){JSReqModuleEntry*r=m->req_module_entries+i;num(r->module_name,4);dump_compiled(r->attributes);}
 for(int i=0;i<m->export_entries_count;i++){JSExportEntry*e=m->export_entries+i;num(e->export_type,4);num(e->local_name,4);num(e->export_name,4);num(e->export_type==JS_EXPORT_TYPE_LOCAL?e->u.local.var_idx:e->u.req_module_idx,4);}
 for(int i=0;i<m->star_export_entries_count;i++)num(m->star_export_entries[i].req_module_idx,4);
 for(int i=0;i<m->import_entries_count;i++){JSImportEntry*e=m->import_entries+i;num(e->var_idx,4);num(e->is_star,4);num(e->import_name,4);num(e->req_module_idx,4);}
 dump_compiled(m->func_obj);return;}
 if(JS_VALUE_GET_TAG(val)==JS_TAG_OBJECT){
 num(JS_TAG_OBJECT,4);JSObject*p=JS_VALUE_GET_OBJ(val);num(p->class_id,4);num(p->extensible,4);num(p->fast_array,4);JSShape*sh=p->shape;num(sh->prop_count,4);
 for(int i=0;i<sh->prop_count;i++){JSShapeProperty*prs=get_shape_prop(sh)+i;num(prs->atom,4);num(prs->flags,4);if(prs->atom){assert((prs->flags&JS_PROP_TMASK)==0);dump_compiled(p->prop[i].u.value);}}
 if(p->fast_array){num(p->u.array.count,4);for(int i=0;i<p->u.array.count;i++)dump_compiled(p->u.array.u.values[i]);}return;}
 if(JS_VALUE_GET_TAG(val)!=JS_TAG_FUNCTION_BYTECODE){dump_val(val);return;}
 num(JS_TAG_FUNCTION_BYTECODE,4);JSFunctionBytecode*b=JS_VALUE_GET_PTR(val);
 int ns[]={b->js_mode,b->has_prototype,b->has_simple_parameter_list,b->is_derived_class_constructor,b->need_home_object,b->func_kind,b->new_target_allowed,b->super_call_allowed,b->super_allowed,b->arguments_allowed,b->has_debug,b->read_only_bytecode,b->is_direct_or_indirect_eval,b->byte_code_len,b->func_name,b->arg_count,b->var_count,b->defined_arg_count,b->stack_size,b->var_ref_count,b->cpool_count,b->closure_var_count};for(int i=0;i<countof(ns);i++)num(ns[i],4);
 fwrite(b->byte_code_buf,1,b->byte_code_len,stdout);num(b->vardefs!=NULL,4);
 if(b->vardefs)for(int i=0;i<b->arg_count+b->var_count;i++){JSBytecodeVarDef*v=b->vardefs+i;int ns[]={v->var_name,v->scope_next,v->is_const,v->is_lexical,v->is_captured,v->has_scope,v->var_kind,v->var_ref_idx};for(int j=0;j<countof(ns);j++)num(ns[j],4);}
 for(int i=0;i<b->closure_var_count;i++){JSClosureVar*v=b->closure_var+i;int ns[]={v->closure_type,v->is_lexical,v->is_const,v->var_kind,v->var_idx,v->var_name};for(int j=0;j<countof(ns);j++)num(ns[j],4);}
 if(b->has_debug){num(b->debug.filename,4);num(b->debug.source_len,4);num(b->debug.pc2line_len,4);num(b->debug.source!=NULL,4);if(b->debug.pc2line_len)fwrite(b->debug.pc2line_buf,1,b->debug.pc2line_len,stdout);if(b->debug.source)fwrite(b->debug.source,1,b->debug.source_len,stdout);}
 for(int i=0;i<b->cpool_count;i++)dump_compiled(b->cpool[i]);
}
static void compiler_bytecode_fixtures(JSContext*base,struct host*h){
 JSContext*ctx=js_mallocz_rt(base->rt,sizeof(JSContext));assert(ctx);memcpy(ctx,base,sizeof(JSContext));init_list_head(&ctx->loaded_modules);js_rc(ctx)->ref_count=1;ctx->eval_internal=__JS_EvalInternal;
 for(int strip=0;strip<4;strip++)for(int strict=0;strict<2;strict++)for(int i=0;i<countof(compiler_cases);i++){
  ctx->rt->strip_flags=strip;const char*input=compiler_cases[i];JSValue val=JS_Eval(ctx,input,strlen(input),"compiler.js",JS_EVAL_FLAG_COMPILE_ONLY|(strict?JS_EVAL_FLAG_STRICT:0));dump_compiled(val);dump_exception(ctx);JS_FreeValue(ctx,val);num(h->calls,4);num(h->live,4);num(h->trace,8);
 }
 for(int strip=0;strip<4;strip++)for(int i=0;i<countof(module_cases);i++){ctx->rt->strip_flags=strip;const char*input=module_cases[i];JSValue val=JS_Eval(ctx,input,strlen(input),"module.js",JS_EVAL_TYPE_MODULE|JS_EVAL_FLAG_COMPILE_ONLY);dump_compiled(val);dump_exception(ctx);JS_FreeValue(ctx,val);js_free_modules(ctx,JS_FREE_MODULE_ALL);num(h->calls,4);num(h->live,4);num(h->trace,8);}
 ctx->rt->strip_flags=0;assert(js_rc(ctx)->ref_count==1);js_free_rt(ctx->rt,ctx);
}
