
static void lvalue_flow_fixtures(JSContext*ctx,struct host*h){
 const uint8_t input[]="lvalue input";GetLineColCache cache={0};cache.ptr=input;cache.buf_start=input;JSAtom dynamic=JS_NewAtom(ctx,"owned_lvalue");int ops[]={OP_scope_get_var,OP_get_field,OP_scope_get_private_field,OP_get_array_el,OP_get_super_value,OP_push_i32};JSAtom names[]={JS_ATOM_name,JS_ATOM_arguments,JS_ATOM_eval,JS_ATOM_this,JS_ATOM_new_target,dynamic};
 for(int mask=0;mask<4;mask++)for(int oi=0;oi<6;oi++)for(int ni=0;ni<6;ni++)for(int keep=0;keep<2;keep++)for(int special=0;special<5;special++){
 int op=ops[oi];JSAtom name=names[ni];JSFunctionDef*fd=js_new_function_def(ctx,NULL,0,0,"lvalue.js",input,&cache);assert(fd);fd->js_mode=mask&1?JS_MODE_STRICT:0;JSParseState s={0};s.ctx=ctx;s.cur_func=fd;s.buf_start=input;s.token.ptr=input;s.filename="lvalue.js";push_scope(&s);if(mask&2)assert(add_scope_var(ctx,fd,JS_ATOM__with_,JS_VAR_NORMAL)>=0);
 emit_op(&s,op);if(op==OP_scope_get_var||op==OP_get_field||op==OP_scope_get_private_field){emit_atom(&s,name);if(op!=OP_get_field)emit_u16(&s,fd->scope_level);}else if(op==OP_push_i32)emit_u32(&s,7);
 int opcode=0,scope=0,label=0,depth=0;JSAtom result_name=0;int ret=get_lvalue(&s,&opcode,&scope,&result_name,&label,&depth,keep,TOK_INC);num(ret,4);num(opcode,4);num(scope,4);num(result_name,4);num(label,4);num(depth,4);dump_exception(ctx);if(!ret){emit_op(&s,OP_inc);put_lvalue(&s,opcode,scope,result_name,label,special,0);}num(fd->byte_code.size,4);fwrite(fd->byte_code.buf,1,fd->byte_code.size,stdout);num(fd->last_opcode_pos,4);num(fd->label_count,4);js_free_function_def(ctx,fd);num(h->calls,4);num(h->live,4);num(h->trace,8);
 }
 JS_FreeAtom(ctx,dynamic);
 for(int kind=0;kind<4;kind++)for(int derived=0;derived<2;derived++)for(int hasval=0;hasval<2;hasval++)for(int iter=0;iter<2;iter++){
 JSFunctionDef*fd=js_new_function_def(ctx,NULL,0,0,"flow.js",input,&cache);assert(fd);fd->func_kind=kind;fd->is_derived_class_constructor=derived;JSParseState s={0};s.ctx=ctx;s.cur_func=fd;s.buf_start=input;s.token.ptr=input;s.filename="flow.js";push_scope(&s);BlockEnv envs[3]={0};
 for(int i=0;i<3;i++){int lb=new_label(&s),lc=new_label(&s);push_break_entry(fd,envs+i,JS_ATOM_name,lb,lc,4);envs[i].has_iterator=iter;if(i==1)envs[i].label_finally=new_label(&s);envs[i].is_regular_stmt=i==2;}
 JSAtom labels[]={0,JS_ATOM_name,JS_ATOM_length};for(int i=0;i<3;i++)for(int is_cont=0;is_cont<2;is_cont++){num(emit_break(&s,labels[i],is_cont),4);dump_exception(ctx);int fresh=new_label(&s);emit_label(&s,fresh);}
 emit_return(&s,hasval);for(int i=0;i<3;i++)pop_break_entry(fd);num(fd->byte_code.size,4);fwrite(fd->byte_code.buf,1,fd->byte_code.size,stdout);num(fd->last_opcode_pos,4);num(fd->label_count,4);js_free_function_def(ctx,fd);num(h->calls,4);num(h->live,4);num(h->trace,8);
 }
}

static void json_lexer_fixtures(JSContext*ctx,struct host*h){
 for(int ext=0;ext<2;ext++)for(int case_idx=0;case_idx<countof(parser_cases);case_idx++){size_t len=parser_case_lengths[case_idx];uint8_t*input=calloc(1,len+8);memcpy(input,parser_cases[case_idx],len);JSParseState s;js_parse_init(ctx,&s,(const char*)input,len,"json.js");s.ext_json=ext;num(ext,4);num(case_idx,4);
 for(int j=0;j<len+2;j++){int ret=json_next_token(&s);num(ret,4);num(s.token.val,4);num(s.token.ptr-s.buf_start,4);num(s.buf_ptr-s.buf_start,4);num(s.last_ptr-s.buf_start,4);if(!ret){switch(s.token.val){case TOK_NUMBER:dump_val(s.token.u.num.val);break;case TOK_STRING:dump_val(s.token.u.str.str);num(s.token.u.str.sep,4);break;case TOK_IDENT:num(s.token.u.ident.atom,4);break;default:break;}}dump_exception(ctx);if(ret<0||s.token.val==TOK_EOF)break;}
 free_token(&s,&s.token);free(input);num(h->calls,4);num(h->live,4);num(h->trace,8);
 }
}

static void compiler_fixtures(JSContext*ctx,struct host*h){
 const uint8_t input[]="compiler source filename bytes";GetLineColCache cache={0};cache.ptr=input;cache.buf_start=input;
 for(int trial=0;trial<64;trial++){num(trial,4);JSFunctionDef*fd=js_new_function_def(ctx,NULL,trial&1,0,"compiler.js",input,&cache);assert(fd);fd->is_global_var=(trial>>1)&1;fd->eval_type=trial&4?JS_EVAL_TYPE_MODULE:JS_EVAL_TYPE_GLOBAL;fd->js_mode=trial&8?JS_MODE_STRICT:0;
 JSFunctionDef*child=js_new_function_def(ctx,fd,0,1,"child.js",input+5,&cache);assert(child);num(child->parent_scope_level,4);num(child->js_mode,4);num(child->source_pos,4);
 JSParseState s={0};s.ctx=ctx;s.filename="compiler.js";s.buf_start=input;s.token.ptr=input;s.cur_func=fd;
 num(push_scope(&s),4);fd->body_scope=fd->scope_level;JSAtom atoms[]={JS_ATOM_arguments,JS_ATOM_name,JS_ATOM_length,JS_ATOM_this};for(int i=0;i<4;i++)num(add_arg(ctx,fd,atoms[i]),4);num(add_arguments_arg(ctx,fd),4);num(add_func_var(ctx,fd,atoms[1]),4);num(add_arguments_var(ctx,fd),4);
 h->fail=trial?h->calls+trial:0;
 for(int step=0;step<128;step++){JSAtom atom=atoms[step%4];int r;switch(step%8){
 case 0:r=push_scope(&s);break;case 1:r=define_var(&s,fd,atom,step%7);break;case 2:r=add_private_class_field(&s,fd,atom,JS_VAR_PRIVATE_FIELD,step&1);break;
 case 3:{int label=new_label(&s);if(label>=0){emit_goto(&s,OP_goto,label);emit_label(&s,label);}r=label;break;}
 case 4:emit_source_pos(&s,input+step%20);r=emit_push_const(&s,JS_NewInt32(ctx,step),0);break;
 case 5:if(fd->scope_level>fd->body_scope)pop_scope(&s);r=0;break;case 6:r=find_lexical_decl(ctx,fd,atom,fd->scope_first,1);break;default:r=find_var(ctx,fd,atom);break;}
 num(r,4);dump_exception(ctx);if(r<0&&dbuf_error(&fd->byte_code))break;
 }
 h->fail=0;num(fd->byte_code.size,4);fwrite(fd->byte_code.buf,1,fd->byte_code.size,stdout);int ns[]={fd->last_opcode_pos,fd->scope_level,fd->scope_first,fd->scope_count,fd->scope_size,fd->var_count,fd->var_size,fd->arg_count,fd->arg_size,fd->global_var_count,fd->global_var_size,fd->cpool_count,fd->cpool_size,fd->label_count,fd->label_size};for(int i=0;i<countof(ns);i++)num(ns[i],4);
 for(int i=0;i<fd->scope_count;i++){num(fd->scopes[i].parent,4);num(fd->scopes[i].first,4);}
 for(int i=0;i<fd->var_count;i++){JSVarDef*v=fd->vars+i;num(v->var_name,4);num(v->scope_level,4);num(v->scope_next,4);num(v->is_const|(v->is_lexical<<1)|(v->is_captured<<2)|(v->is_static_private<<3)|(v->var_kind<<4),4);num(v->var_ref_idx,4);num(v->func_pool_idx,4);}
 for(int i=0;i<fd->global_var_count;i++){JSGlobalVar*v=fd->global_vars+i;num(v->cpool_idx,4);num(v->force_init,4);num(v->is_lexical,4);num(v->is_const,4);num(v->scope_level,4);num(v->var_name,4);}
 js_free_function_def(ctx,fd);num(h->calls,4);num(h->live,4);num(h->trace,8);
 }
}
int main(void){
 JSRuntime rt={0};JSContext ctx={0};struct host h={0};JSClass classes[JS_CLASS_INIT_COUNT]={0};JSValue prototypes[JS_CLASS_INIT_COUNT];for(int i=0;i<JS_CLASS_INIT_COUNT;i++)prototypes[i]=JS_NULL;setup(&rt,&ctx,&h,classes,prototypes);
 num(sizeof(BlockEnv),4);num(sizeof(JSGlobalVar),4);num(sizeof(JSVarDef),4);num(sizeof(JSFunctionDef),4);num(sizeof(JSToken),4);num(sizeof(JSParseState),4);num(offsetof(JSFunctionDef,func_name),4);num(offsetof(JSFunctionDef,filename)-4,4);num(offsetof(JSFunctionDef,filename),4);
 for(int mode=0;mode<16;mode++)for(int case_idx=0;case_idx<countof(parser_cases);case_idx++){
  size_t len=parser_case_lengths[case_idx];uint8_t*input=calloc(1,len+8);memcpy(input,parser_cases[case_idx],len);JSFunctionDef fd={0},parent={0};fd.js_mode=mode&1?JS_MODE_STRICT:0;fd.func_kind=(mode>>1)&3;fd.func_type=mode&8?JS_PARSE_FUNC_ARROW:JS_PARSE_FUNC_EXPR;parent.func_kind=JS_FUNC_ASYNC_GENERATOR;fd.parent=&parent;
  JSParseState s={0};s.ctx=&ctx;s.filename="lexer.js";s.cur_func=&fd;s.buf_start=input;s.buf_ptr=input;s.buf_end=input+len;s.token.val=TOK_EOF;s.token.ptr=input;s.is_module=mode==15;s.allow_html_comments=!s.is_module;
 num(JS_DetectModule((const char*)input,len),4);for(int no_lf=0;no_lf<2;no_lf++){const uint8_t*p=input;num(simple_next_token(&p,no_lf),4);num(p-input,4);}
 GetLineColCache cache={0};cache.ptr=input;cache.buf_start=input;for(int i=0;i<32;i++){size_t index=(i*193)%(len+1);int col=0;num(get_line_col_cached(&cache,&col,input+index),4);num(col,4);}
 if(len&&input[0]=='/'&&input[1]!='/'&&input[1]!='*'){int ret=js_parse_regexp(&s);num(ret,4);num(s.buf_ptr-s.buf_start,4);if(!ret){dump_val(s.token.u.regexp.body);dump_val(s.token.u.regexp.flags);}dump_exception(&ctx);free_token(&s,&s.token);s.token.val=TOK_EOF;s.buf_ptr=s.buf_start;}
 if(len&&(input[0]=='('||input[0]=='['||input[0]=='{')){int ret=next_token(&s);assert(!ret);int bits=0;num(js_parse_skip_parens_token(&s,&bits,mode&1),4);num(bits,4);num(s.token.val,4);num(s.buf_ptr-s.buf_start,4);dump_exception(&ctx);free_token(&s,&s.token);s.token.val=TOK_EOF;s.buf_ptr=s.buf_start;}
num(mode,4);num(case_idx,4);
  for(int j=0;j<len+2;j++){int ret=next_token(&s);num(ret,4);num(s.token.val,4);num(s.token.ptr-s.buf_start,4);num(s.buf_ptr-s.buf_start,4);num(s.last_ptr-s.buf_start,4);num(s.got_lf,4);
   if(!ret){switch(s.token.val){case TOK_NUMBER:dump_val(s.token.u.num.val);break;case TOK_STRING:case TOK_TEMPLATE:dump_val(s.token.u.str.str);num(s.token.u.str.sep,4);break;default:if(s.token.val==TOK_IDENT||s.token.val==TOK_PRIVATE_NAME||(s.token.val>=TOK_FIRST_KEYWORD&&s.token.val<=TOK_LAST_KEYWORD)){num(s.token.u.ident.atom,4);num(s.token.u.ident.has_escape,4);num(s.token.u.ident.is_reserved,4);}break;}}
   dump_exception(&ctx);if(ret<0||s.token.val==TOK_EOF)break;
  }
  free_token(&s,&s.token);free(input);num(h.calls,4);num(h.live,4);num(h.trace,8);
 }
 json_lexer_fixtures(&ctx,&h);compiler_fixtures(&ctx,&h);lvalue_flow_fixtures(&ctx,&h);JS_FreeValue(&ctx,rt.current_exception);rt.current_exception=JS_UNINITIALIZED;js_free_shape(&rt,ctx.array_shape);assert(!rt.shape_hash_count);assert(list_empty(&rt.gc_obj_list));js_free_rt(&rt,rt.shape_hash);cleanup(&rt,&h);assert(!h.live);return 0;
}
