/* Independent official implementation, compiled only for differential tests. */
#include <stdio.h>
#include <stdint.h>
#include <math.h>
#include "cutils.c"
static void num(uint32_t n){for(int i=0;i<4;i++)putchar((n>>(8*i))&255);}
static void num64(uint64_t n){for(int i=0;i<8;i++)putchar((n>>(8*i))&255);}
static uint32_t state=0x12345678;
static uint32_t rng(void){return state=state*1664525u+1013904223u;}
static void utf(uint32_t c){
 uint8_t b[8];memset(b,0xa5,sizeof(b));int n=unicode_to_utf8(b,c);num(n);fwrite(b,1,6,stdout);
 if(n){const uint8_t*p=b+7;int r=unicode_from_utf8(b,n,&p);num(r);num(p-b);}
}
static void decode(const uint8_t*b,int len){const uint8_t*p=b+7;num(unicode_from_utf8(b,len,&p));num(p-b);}
static void bufstate(DynBuf*s,int r){num(r);num64(s->size);num64(s->allocated_size);num(s->error);if(s->size)fwrite(s->buf,1,s->size,stdout);}
struct allocator{int calls,fail;};
static void* failrealloc(void*o,void*p,size_t n){struct allocator*a=o;a->calls++;if(n&&a->calls==a->fail)return NULL;return realloc(p,n);}
struct cmpctx{uint64_t hash;uint32_t calls;int direction;};
static int compare(const void*a,const void*b,void*o){struct cmpctx*s=o;uint8_t x=*(uint8_t*)a,y=*(uint8_t*)b;s->calls++;s->hash=s->hash*1099511628211ull+x;s->hash=s->hash*1099511628211ull+y;return ((x>y)-(x<y))*s->direction;}
static void sortcase(int n,int size,int offset,int distribution,int direction,int heap){
 _Alignas(16) uint8_t b[8192];memset(b,0x77,sizeof(b));uint8_t*p=b+offset;
 for(int i=0;i<n;i++){uint8_t v=distribution==0?rng():distribution==1?i:distribution==2?n-i:distribution==3?5:i%7;for(int j=0;j<size;j++)p[i*size+j]=v+j;}
 struct cmpctx ctx={1469598103934665603ull,0,direction};
 if(heap)heapsortx(p,n,size,compare,&ctx);else rqsort(p,n,size,compare,&ctx);
 num(ctx.calls);num64(ctx.hash);fwrite(b,1,n*size+offset+16,stdout);
}
int main(void){
 for(uint32_t i=0;i<65536;i++){double d=fromfp16(i);num64(float64_as_uint64(d));num(tofp16(d));num(isfp16nan(i));num(isfp16zero(i));
  if(i<0x7bff){double next=fromfp16(i+1),mid=(d+next)*0.5;num(tofp16(mid));num(tofp16(nextafter(mid,-INFINITY)));num(tofp16(nextafter(mid,INFINITY)));}
 }
 for(int i=0;i<100000;i++){uint64_t b=((uint64_t)rng()<<32)|rng();num(tofp16(uint64_as_float64(b)));}
 for(uint32_t i=0;i<=0x10ffff;i++)utf(i);
 uint32_t bounds[]={0x1fffff,0x200000,0x3ffffff,0x4000000,0x7fffffff,0x80000000,0xffffffff};
 for(size_t i=0;i<countof(bounds);i++)utf(bounds[i]);
 for(int i=0;i<10000;i++)utf(rng());
 for(int i=0;i<65536;i++){uint8_t b[8]={i>>8,i,0x80,0x80,0x80,0x80,0x80,0x80};for(int len=1;len<=6;len++)decode(b,len);}
 for(int fail=0;fail<=6;fail++){
  DynBuf s;struct allocator a={0,fail};dbuf_init2(&s,&a,failrealloc);bufstate(&s,dbuf_put(&s,NULL,0));
  bufstate(&s,dbuf_putc(&s,0xff));bufstate(&s,dbuf_put_u16(&s,0x82fe));bufstate(&s,dbuf_put_u32(&s,0x89abcdef));bufstate(&s,dbuf_put_u64(&s,0x0123456789abcdefull));
  if(s.size>=7)bufstate(&s,dbuf_put_self(&s,0,7));
  bufstate(&s,dbuf_claim(&s,40));bufstate(&s,dbuf_putstr(&s,"abc"));
  dbuf_set_error(&s);bufstate(&s,dbuf_claim(&s,0));dbuf_free(&s);num(a.calls);num(s.buf==NULL&&s.realloc_func==NULL&&s.opaque==NULL);dbuf_free(&s);
 }
 DynBuf s;dbuf_init(&s);bufstate(&s,dbuf_printf(&s,"value:%08x/%s",0x12,"héllo"));
 char text[401];memset(text,'x',400);text[400]=0;bufstate(&s,dbuf_printf(&s,"[%s]",text));dbuf_free(&s);
 for(int kind=0;kind<3;kind++){dbuf_init(&s);s.size=kind==0?SIZE_MAX-1:0;s.allocated_size=kind==1?SIZE_MAX-1:0;s.error=kind==2;num(dbuf_claim(&s,kind==0?5:SIZE_MAX));num(s.error);s.buf=NULL;dbuf_free(&s);}
 for(int size=1;size<=32;size++)for(int offset=0;offset<16;offset++)for(int dist=0;dist<5;dist++)for(int heap=0;heap<2;heap++)sortcase(127,size,offset,dist,dist%2?-1:1,heap);
 for(int n=0;n<16;n++)sortcase(n,16,0,4,1,0);
 for(int k=-10;k<=300;k++)num(from_hex(k));
 for(uint32_t c=0;c<=0x10ffff;c++){num(is_surrogate(c));num(is_hi_surrogate(c));num(is_lo_surrogate(c));if(c>=0x10000)num(from_surrogate(get_hi_surrogate(c),get_lo_surrogate(c)));}
 for(int k=0;k<10000;k++){uint32_t a=rng()|1,b=rng();uint64_t x=((uint64_t)a<<32)|b;num(clz32(a));num(ctz32(a));num(clz64(x));num(ctz64(x));num(max_int(a,b));num(min_int(a,b));num(max_uint32(a,b));num(min_uint32(a,b));num64(max_int64(x,~x));num64(min_int64(x,~x));num(bswap16(a));num(bswap32(a));num64(bswap64(x));
  uint8_t buf[24];memset(buf,0xa5,sizeof(buf));put_u64(buf+1,x);put_u32(buf+3,b);put_u16(buf+7,a);put_u8(buf+9,a);fwrite(buf,1,sizeof(buf),stdout);num64(get_u64(buf+1));num64(get_i64(buf+1));num(get_u32(buf+3));num(get_i32(buf+3));num(get_u16(buf+7));num(get_i16(buf+7));num(get_u8(buf+9));num(get_i8(buf+9));
 }
 const char*str="abcdef";const char*prefixes[]={"","a","abc","abcdef","abcdefg","abX"};
 for(int size=-1;size<12;size++){char b[16];memset(b,0x7f,sizeof(b));pstrcpy(b,size,str);fwrite(b,1,sizeof(b),stdout);}
 for(size_t i=0;i<countof(prefixes);i++){const char*p=str+6;num(strstart(str,prefixes[i],&p));num(p-str);num(has_suffix(str,prefixes[i]));}
 char b[16]="abc";num(pstrcat(b,5,"defgh")==b);fwrite(b,1,sizeof(b),stdout);memcpy_no_ub(NULL,NULL,0);
 return 0;
}
