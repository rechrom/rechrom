/* Test process only: includes official headers, never linked into Rust. */
#include <stdio.h>
#include <stdint.h>
#include "cutils.h"
#include "list.h"
enum {
 JS_ATOM_NULL=0,
#define DEF(name,str) JS_ATOM_##name,
#include "quickjs-atom.h"
#undef DEF
 JS_ATOM_END
};
static const char atoms[] =
#define DEF(name,str) str "\0"
#include "quickjs-atom.h"
#undef DEF
;
enum {
#define FMT(f) OP_FMT_##f,
#define DEF(id,size,pop,push,f)
#include "quickjs-opcode.h"
#undef DEF
#undef FMT
};
enum {
#define FMT(f)
#define DEF(id,size,pop,push,f) OP_##id,
#define def(id,size,pop,push,f)
#include "quickjs-opcode.h"
#undef def
#undef DEF
#undef FMT
 OP_COUNT, OP_TEMP_START=OP_nop+1, OP___dummy=OP_TEMP_START-1,
#define FMT(f)
#define DEF(id,size,pop,push,f)
#define def(id,size,pop,push,f) OP_##id,
#include "quickjs-opcode.h"
#undef def
#undef DEF
#undef FMT
 OP_TEMP_END
};
static const uint8_t opinfo[][4] = {
#define FMT(f)
#define DEF(id,size,pop,push,f) {size,pop,push,OP_FMT_##f},
#include "quickjs-opcode.h"
#undef DEF
#undef FMT
};
enum {
#define DEF(id,size) REOP_##id,
#include "libregexp-opcode.h"
#undef DEF
 REOP_COUNT
};
static void num(uint32_t n){for(int j=0;j<4;j++)putchar((n>>(8*j))&255);}
struct node{int value;struct list_head link;};
int main(void){
 num(JS_ATOM_END);fwrite(atoms,1,sizeof(atoms),stdout);
#define DEF(name,str) num(JS_ATOM_##name);
#include "quickjs-atom.h"
#undef DEF
#define FMT(f) num(OP_FMT_##f);
#define DEF(id,size,pop,push,f)
#include "quickjs-opcode.h"
#undef DEF
#undef FMT
 num(OP_COUNT);num(OP_TEMP_START);num(OP_TEMP_END);
#define FMT(f)
#define DEF(id,size,pop,push,f) num(OP_##id);
#include "quickjs-opcode.h"
#undef DEF
#undef FMT
 fwrite(opinfo,1,sizeof(opinfo),stdout);
 for(int i=0;i<OP_COUNT;i++){
#if SHORT_OPCODES
 int idx=i>=OP_TEMP_START ? i+OP_TEMP_END-OP_TEMP_START:i;
#else
 int idx=i;
#endif
 fwrite(opinfo[idx],1,4,stdout);
 }
 num(REOP_COUNT);
#define DEF(id,size) num(REOP_##id);putchar(size);
#include "libregexp-opcode.h"
#undef DEF
 struct list_head head=LIST_HEAD_INIT(head);struct node nodes[4];
 struct list_head *el,*next;
 num(list_empty(&head));
 for(int i=0;i<4;i++){nodes[i].value=i;list_add_tail(&nodes[i].link,&head);}
 list_del(&nodes[3].link);list_add(&nodes[3].link,&head);
 list_for_each(el,&head){num(list_entry(el,struct node,link)->value);}
 list_for_each_prev(el,&head){num(list_entry(el,struct node,link)->value);}
 list_for_each_safe(el,next,&head){num(list_entry(el,struct node,link)->value);list_del(el);num(el->next==NULL&&el->prev==NULL);continue;}
 num(list_empty(&head));
 for(int i=0;i<4;i++)list_add_tail(&nodes[i].link,&head);
 list_for_each_prev_safe(el,next,&head){num(list_entry(el,struct node,link)->value);list_del(el);}
 num(list_empty(&head));return 0;
}
