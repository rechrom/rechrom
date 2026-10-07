/* Independent oracle: compile the unchanged official libregexp.c. */
#include <stdio.h>
#include <stdint.h>
#include "libregexp.c"
static void num(uint32_t n){for(int i=0;i<4;i++)putchar((n>>(8*i))&255);}
struct host { int calls, fail, stack_calls, stack_fail; void *allocs[16384]; int count; };
int lre_check_stack_overflow(void *opaque,size_t n){struct host *h=opaque;return ++h->stack_calls==h->stack_fail;}
int lre_check_timeout(void *opaque){return 0;}
void *lre_realloc(void *opaque,void *p,size_t n){
 struct host *h=opaque;h->calls++;
 if(n&&h->calls==h->fail)return NULL;
 int index=-1;for(int i=0;i<h->count;i++)if(h->allocs[i]==p){index=i;break;}
 if(!n){free(p);if(index>=0)h->allocs[index]=h->allocs[--h->count];return NULL;}
 void *q=realloc(p,n);if(q){if(index>=0)h->allocs[index]=q;else h->allocs[h->count++]=q;}return q;
}
static int hex(const char *s,uint8_t *b){int n=0;while(*s&&*s!='\n'&&*s!='\t'){int a=from_hex(*s++),c=from_hex(*s++);b[n++]=(a<<4)|c;}return n;}
int main(int argc,char **argv){
 FILE *f=fopen(argv[1],"r");char line[32768];int record=0;
 while(fgets(line,sizeof(line),f)){
  char *t=strchr(line,'\t');int flags,fail,stack_fail,error_size;
  sscanf(line,"%d,%d,%d,%d",&flags,&fail,&stack_fail,&error_size);
  uint8_t pattern[16384]={0};int len=hex(t+1,pattern);struct host h={0};h.fail=fail;h.stack_fail=stack_fail;
  if(getenv("QUICKJS_ORACLE_TRACE"))fprintf(stderr,"record=%d flags=%d fail=%d stack_fail=%d pattern=%s\n",record,flags,fail,stack_fail,pattern);
  int bc_len=0;char error[128]={0};uint8_t *bc=lre_compile(&bc_len,error,error_size,(char*)pattern,len,flags,&h);
  num(record++);num(bc_len);num(h.calls);num(h.stack_calls);num(h.count);num(strlen(error));fwrite(error,1,strlen(error),stdout);
  if(bc_len)fwrite(bc,1,bc_len,stdout);
  /* Some upstream error paths retain allocations; reclaim oracle-owned blocks. */
  for(int i=0;i<h.count;i++)free(h.allocs[i]);
 }
 fclose(f);return 0;
}
