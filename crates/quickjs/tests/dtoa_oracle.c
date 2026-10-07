#include <stdio.h>
#include <stdint.h>
#include "dtoa.c"
static void num(uint32_t n){for(int i=0;i<4;i++)putchar((n>>(8*i))&255);}
static void num64(uint64_t n){for(int i=0;i<8;i++)putchar((n>>(8*i))&255);}
static uint32_t state=0x12345678;static uint32_t rng(void){return state=state*1664525u+1013904223u;}
static void printcase(uint64_t bits,int radix,int digits,int flags){
 double d=uint64_as_float64(bits);char buf[2048];memset(buf,0x7f,sizeof(buf));JSDTOATempMem temp;
 int n=js_dtoa(buf,d,radix,digits,flags,&temp);num(js_dtoa_max_len(d,radix,digits,flags));num(n);fwrite(buf,1,n+1,stdout);num(buf[n+1]);
 JSATODTempMem parse;const char*next=buf;double value=js_atod(buf,&next,radix,0,&parse);num64(float64_as_uint64(value));num(next-buf);
}
static void parsecase(const char*text,int radix,int flags){JSATODTempMem temp;const char*next=text;double value=js_atod(text,&next,radix,flags,&temp);num64(float64_as_uint64(value));num(next-text);}
int main(int argc,char**argv){
 uint64_t bounds[]={0,0x8000000000000000ull,1,2,0x8000000000000001ull,0xfffffffffffffull,0x10000000000000ull,0x10000000000001ull,0x3ff0000000000000ull,0x3fefffffffffffffull,0x3ff0000000000001ull,0x7fefffffffffffffull,0x7ff0000000000000ull,0xfff0000000000000ull,0x7ff0000000000001ull,0xfff8000000000000ull};
 for(int radix=2;radix<=36;radix++){
  for(int i=0;i<16;i++)for(int exp=0;exp<=8;exp+=4)for(int sign=0;sign<=16;sign+=16)printcase(bounds[i],radix,0,exp|sign);
  for(int i=0;i<1500;i++){uint64_t bits=((uint64_t)rng()<<32)|rng();for(int exp=0;exp<=8;exp+=4)printcase(bits,radix,0,exp);}
 }
 int digits[]={0,1,2,6,17,50,100,101};
 for(int i=0;i<500;i++){
  uint64_t bits=((uint64_t)rng()<<32)|rng();for(size_t k=0;k<countof(digits);k++)for(int exp=0;exp<=8;exp+=4)for(int fmt=1;fmt<=2;fmt++)if(digits[k]||fmt==2)printcase(bits,10,digits[k],exp|fmt);
 }
 for(int i=0;i<65536;i++){uint64_t bits=float64_as_uint64(fromfp16(i));printcase(bits,10,0,0);printcase(bits,10,2,2);printcase(bits,10,10,1);}
 char buf[128];for(int i=0;i<10000;i++){
  uint32_t a=rng();uint64_t b=((uint64_t)rng()<<32)|rng();size_t n=u32toa(buf,a);num(n);fwrite(buf,1,n,stdout);n=i32toa(buf,a);num(n);fwrite(buf,1,n,stdout);n=u64toa(buf,b);num(n);fwrite(buf,1,n,stdout);n=i64toa(buf,b);num(n);fwrite(buf,1,n,stdout);
  for(int radix=2;radix<=36;radix++){n=u64toa_radix(buf,b,radix);num(n);fwrite(buf,1,n,stdout);n=i64toa_radix(buf,b,radix);num(n);fwrite(buf,1,n,stdout);}
 }
 FILE*f=fopen(argv[1],"r");char line[4096];while(fgets(line,sizeof(line),f)){char*tab1=strchr(line,'\t');char*tab2=strchr(tab1+1,'\t');int radix=atoi(line),flags=atoi(tab1+1);char*p=tab2+1;p[strcspn(p,"\n")]=0;parsecase(p,radix,flags);}fclose(f);
 for(int radix=2;radix<=36;radix++)for(int a=-2048;a<=2047;a++)num(mul_log2_radix(a,radix));
 return 0;
}
