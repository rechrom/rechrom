static uint64_t rng=0x123456789abcdef0ULL;
static uint64_t next(void){rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;return rng;}
static void num(uint64_t n,int size){for(int i=0;i<size;i++)putchar((n>>(8*i))&255);}
struct big{int len;js_limb_t tab[128];};
static void normalize(struct big*a){while(a->len>1){js_limb_t v=a->tab[a->len-1];if((v!=0&&v!=(js_limb_t)-1)||(v&1)!=(a->tab[a->len-2]>>(JS_LIMB_BITS-1)))break;a->len--;}}
int main(void){
 uint64_t patterns[]={0,0x8000000000000000ULL,0x7ff0000000000000ULL,0xfff0000000000000ULL,0x7ff8000000000000ULL,0x7ff0000000000001ULL,1,0x8000000000000001ULL,0x0010000000000000ULL,0x8010000000000000ULL,0x3ff0000000000000ULL,0xbff0000000000000ULL,0x7fefffffffffffffULL,0xffefffffffffffffULL};
 for(int step=0;step<32768;step++){
  struct big a={1+step%128,{0}},b={1+(step/128)%128,{0}};for(int i=0;i<a.len;i++)a.tab[i]=next();for(int i=0;i<b.len;i++)b.tab[i]=next();
  if(step%8==0){memset(a.tab,0,sizeof(a.tab));a.tab[(step/8)%a.len]=(js_limb_t)1<<((step/1024)%JS_LIMB_BITS);}
  if(step%8==1){for(int i=0;i<a.len;i++)a.tab[i]=-1;a.tab[0]-=(step%2048);}
  if(step%8==2){a.len=1;a.tab[0]=(step/8)&1?0:1;}
  if(step%8==3){a.len=2;a.tab[0]=(1ULL<<53)+(step%2048);a.tab[1]=0;}
  normalize(&a);normalize(&b);num(a.len,4);num(b.len,4);
  int e=0;if(a.len!=1||a.tab[0]!=0){num(js_bigint_get_mant_exp(NULL,&e,(JSBigInt*)&a),8);num(e,4);}uint64_t f=float64_as_uint64(js_bigint_to_float64(NULL,(JSBigInt*)&a));num(f,8);num(js_bigint_cmp(NULL,(JSBigInt*)&a,(JSBigInt*)&b),4);num(js_bigint_cmp(NULL,(JSBigInt*)&a,(JSBigInt*)&a),4);
  for(int k=0;k<18;k++){uint64_t bits=k<14?patterns[k]:k==14?f:k==15?f+1:k==16?f-1:next();num(js_bigint_float64_cmp(NULL,(JSBigInt*)&a,uint64_as_float64(bits)),4);}
  uint64_t v=next();for(int n=1;n<=30;n++)num(shr_rndn(v,n),8);
 }
 return 0;
}
