#include <stdio.h>
#include <stdint.h>
#include "libregexp.c"
static void num(uint32_t n){for(int i=0;i<4;i++)putchar((n>>(8*i))&255);}
struct host{int calls,fail,polls,timeout;};
int lre_check_stack_overflow(void*opaque,size_t n){return 0;}
int lre_check_timeout(void*opaque){struct host*h=opaque;h->polls++;return h->timeout;}
void*lre_realloc(void*opaque,void*p,size_t n){struct host*h=opaque;h->calls++;if(n&&h->calls==h->fail)return NULL;return realloc(p,n);}
static int hex(const char*s,uint8_t*b){int n=0;while(*s&&*s!='\n'&&*s!='\t'){int a=from_hex(*s++),c=from_hex(*s++);b[n++]=(a<<4)|c;}b[n]=0;return n;}
int main(int argc,char**argv){
 FILE*f=fopen(argv[1],"r");char line[8192];int record=0;
 while(fgets(line,sizeof(line),f)){
  char*t1=strchr(line,'\t'),*t2=strchr(t1+1,'\t');int flags=atoi(line);uint8_t pattern[1024],subject[1024];hex(t1+1,pattern);int len=hex(t2+1,subject);
  int bc_len;char error[128];struct host h={0,0,0,0};uint8_t*bc=lre_compile(&bc_len,error,sizeof(error),(char*)pattern,strlen((char*)pattern),flags,&h);num(record++);num(bc_len);
  if(!bc){num(strlen(error));fwrite(error,1,strlen(error),stdout);continue;}
  fwrite(bc,1,bc_len,stdout);uint16_t utf16[1024];const uint8_t*p=subject,*end=subject+len;int utf16len=0;
  while(p<end){int c=unicode_from_utf8(p,end-p,&p);if(c<0)return 2;if(c<0x10000)utf16[utf16len++]=c;else{utf16[utf16len++]=get_hi_surrogate(c);utf16[utf16len++]=get_lo_surrogate(c);}}
  for(int mode=0;mode<2;mode++){
   uint8_t*data=mode?(uint8_t*)utf16:subject;int length=mode?utf16len:len;num(mode);num(length);fwrite(data,1,length<<mode,stdout);
   for(int index=0;index<=length;index++)for(int fail=0;fail<3;fail++){
    uint8_t**captures=calloc(lre_get_alloc_count(bc),sizeof(captures[0]));h.calls=0;h.polls=0;h.fail=fail;h.timeout=1;
    int ret=lre_exec(captures,bc,data,index,length,mode,&h);num(ret);num(h.calls);num(h.polls);
    for(int i=0;i<2*lre_get_capture_count(bc);i++)num(captures[i]?(uint32_t)(captures[i]-data):UINT32_MAX);
    free(captures);
   }
  }
  free(bc);
 }
 fclose(f);
 const char*escapes[]={"b","f","n","r","t","v","x00","xFE","xG1","x1","u0041","uD83D\\uDE00","uD83D\\uXXXX","u{10ffff}","u{110000}","u{}","u{00000000000000001}","0","00","07","377","400","777","09","q","","cA"};
 for(int mode=0;mode<3;mode++)for(size_t i=0;i<countof(escapes);i++){const uint8_t*p=(const uint8_t*)escapes[i];num(lre_parse_escape(&p,mode));num(p-(const uint8_t*)escapes[i]);}
 return 0;
}
