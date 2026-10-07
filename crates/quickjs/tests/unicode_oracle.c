/* Test oracle only: official C algorithm and data, with no source edits. */
#include <stdio.h>
#include <stdint.h>
#include "libunicode.c"
static void num(uint32_t n){for(int i=0;i<4;i++)putchar((n>>(8*i))&255);}
static void range(CharRange*cr,int ret){num(ret);num(cr->len);num(cr->size);for(int i=0;i<cr->len;i++)num(cr->points[i]);}
static uint32_t state=0x12345678;
static uint32_t rng(void){return state=state*1664525u+1013904223u;}
static void callback(void*opaque,const uint32_t*buf,int len){num(len);for(int i=0;i<len;i++)num(buf[i]);}
static void names(const char*table,int kind){
 const char*p=table;while(*p){const char*q=p;while(*q&&*q!=',')q++;size_t n=q-p;char name[128];memcpy(name,p,n);name[n]=0;
  CharRange cr;cr_init(&cr,NULL,NULL);int ret;
  switch(kind){case 0:ret=unicode_script(&cr,name,0);break;case 1:ret=unicode_script(&cr,name,1);break;case 2:ret=unicode_general_category(&cr,name);break;case 3:ret=unicode_prop(&cr,name);break;default:ret=unicode_sequence_prop(name,callback,NULL,&cr);}
  range(&cr,ret);cr_free(&cr);p=q+1;
 }
}
struct allocator{int calls,fail;void*last;};
static void*failrealloc(void*o,void*p,size_t n){struct allocator*a=o;a->calls++;if(n&&a->calls==a->fail)return NULL;void*r=realloc(p,n);if(n)a->last=r;else if(p==a->last)a->last=NULL;return r;}
int main(int argc,char**argv){
 if(argc>1){CharRange cr;struct allocator a={0,1,NULL};cr_init(&cr,&a,failrealloc);return unicode_script(&cr,"Latin",0);}
 for(uint32_t c=0;c<=0x10ffff;c++){
  for(int type=0;type<3;type++){uint32_t res[3]={0xa5a5a5a5,0xa5a5a5a5,0xa5a5a5a5};num(lre_case_conv(res,c,type));for(int i=0;i<3;i++)num(res[i]);}
  num(lre_canonicalize(c,0));num(lre_canonicalize(c,1));num(lre_is_cased(c));num(lre_is_case_ignorable(c));num(lre_is_id_start(c));num(lre_is_id_continue(c));num(lre_is_space_non_ascii(c));num(lre_is_space(c));num(lre_js_is_ident_first(c));num(lre_js_is_ident_next(c));num(unicode_get_cc(c));
  for(int compat=0;compat<2;compat++){uint32_t res[18];int n=unicode_decomp_char(res,c,compat);num(n);for(int i=0;i<n;i++)num(res[i]);}
 }
 for(uint32_t c=0;c<=0x10ffff;c++)for(int type=0;type<4;type++){uint32_t*out=NULL;int n=unicode_normalize(&out,&c,1,type,NULL,NULL);num(n);for(int i=0;i<n;i++)num(out[i]);free(out);}
 for(size_t i=0;i<countof(unicode_comp_table);i++){uint32_t idx=unicode_comp_table[i]>>6,offset=unicode_comp_table[i]&63,v=unicode_decomp_table1[idx],code=v>>14,len=(v>>7)&127,type=(v>>1)&63,pair[2];unicode_decomp_entry(pair,code+offset,idx,code,len,type);num(compose_pair(pair[0],pair[1]));}
 for(int run=0;run<10000;run++){
  uint32_t src[16];int len=rng()%17;for(int i=0;i<len;i++){uint32_t a=rng();src[i]=a%4==0?0x300+(a%112):a%4==1?0x1100+(a%256):a%0x110000;}
  for(int type=0;type<4;type++){uint32_t*out=NULL;int n=unicode_normalize(&out,src,len,type,NULL,NULL);num(n);for(int i=0;i<n;i++)num(out[i]);free(out);}
 }
 names(unicode_script_name_table,0);names(unicode_script_name_table,1);names(unicode_gc_name_table,2);names(unicode_prop_name_table,3);names(unicode_sequence_prop_name_table,4);
 for(int i=0;i<256;i++){num(lre_ctype_bits[i]);num(lre_is_space_byte(i));num(lre_is_id_start_byte(i));num(lre_is_id_continue_byte(i));num(lre_is_word_byte(i));}
 for(int run=0;run<1000;run++){
  uint32_t a[16],b[16],v=0;for(int i=0;i<16;i++){v+=rng()%100;a[i]=v;}v=0;for(int i=0;i<16;i++){v+=rng()%100;b[i]=v;}
  for(int op=0;op<4;op++){CharRange cr;cr_init(&cr,NULL,NULL);int ret=cr_op(&cr,a,16,b,16,op);range(&cr,ret);ret=cr_invert(&cr);range(&cr,ret);ret=cr_op1(&cr,a,16,op);range(&cr,ret);cr_free(&cr);}
  for(int unicode=0;unicode<2;unicode++){CharRange cr;cr_init(&cr,NULL,NULL);for(int i=0;i<16;i+=2)cr_add_interval(&cr,a[i],a[i+1]);int ret=cr_regexp_canonicalize(&cr,unicode);range(&cr,ret);cr_free(&cr);}
 }
 for(int unicode=0;unicode<2;unicode++){CharRange cr;cr_init(&cr,NULL,NULL);cr_add_interval(&cr,0,0x110000);int ret=cr_regexp_canonicalize(&cr,unicode);range(&cr,ret);cr_free(&cr);}
 for(int fail=1;fail<40;fail++){
  CharRange cr;struct allocator a={0,fail,NULL};cr_init(&cr,&a,failrealloc);int ret=unicode_prop(&cr,"Alphabetic");range(&cr,ret);cr_free(&cr);num(a.calls);
 }
 for(int fail=1;fail<5;fail++){
  uint32_t src[]={0xfb03,0xac01,0x1f82,0x301},*out=NULL;struct allocator a={0,fail,NULL};int n=unicode_normalize(&out,src,4,UNICODE_NFKC,&a,failrealloc);num(n);num(out!=NULL);if(n>=0)for(int i=0;i<n;i++)num(out[i]);if(a.last)free(a.last);num(a.calls);
 }
 return 0;
}
