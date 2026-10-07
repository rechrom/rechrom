/* Driver only; all arithmetic above is extracted unchanged from quickjs.c. */
static uint64_t rng=0x123456789abcdef0ULL;
static uint64_t next(void){rng^=rng<<13;rng^=rng>>7;rng^=rng<<17;return rng;}
static void num(uint64_t n,int size){for(int i=0;i<size;i++)putchar((n>>(8*i))&255);}
static void dump(js_limb_t r,const js_limb_t *p,int n){num(r,sizeof(r));for(int i=0;i<n;i++)num(p[i],sizeof(*p));}
int main(void){
 for(int step=0;step<16384;step++){
  js_limb_t a[34],b[34],r[68],v=next();int n=step%33,shift=1+step%(JS_LIMB_BITS-1);
  for(int i=0;i<34;i++){a[i]=next();b[i]=next();if(step%16==0)a[i]=0;if(step%16==1)a[i]=-1;if(step%16==2)b[i]=-1;if(step%16==3)b[i]=0;if(step%16==4)a[i]=(js_limb_t)1<<(i%JS_LIMB_BITS);}
  num(js_limb_safe_clz(v),4);if(v)num(js_limb_clz(v),4);
  for(int op=0;op<11;op++)for(int alias=0;alias<3;alias++){
   js_limb_t x[34],y[34],ret=0;memcpy(x,a,sizeof(x));memcpy(y,b,sizeof(y));memset(r,0xa5,sizeof(r));js_limb_t*p=alias==0?r:alias==1?x:y;js_limb_t divisor=v|1,rem=b[0]%divisor;
   switch(op){
   case 0:ret=mp_add(p,x,y,n,step&1);break;
   case 1:ret=mp_sub(p,x,y,n,step&1);break;
   case 2:ret=mp_neg(p,x,n);break;
   case 3:ret=mp_mul1(p,x,n,v,b[0]);break;
   case 4:ret=mp_div1(p,x,n,divisor,rem);break;
   case 5:ret=mp_add_mul1(p,x,n,v);break;
   case 6:ret=mp_sub_mul1(p,x,n,v);break;
   case 7:ret=mp_shl(p,x,n,shift);break;
   case 8:ret=mp_shr(p,x,n,shift,v);break;
   case 9:divisor=v|((js_limb_t)1<<(JS_LIMB_BITS-1));ret=mp_div1norm(p,x,n,divisor,b[0]%divisor);break;
   case 10:divisor=v|((js_limb_t)1<<(JS_LIMB_BITS-1));rem=a[0]%divisor;ret=udiv1norm(p,rem,b[0],divisor,udiv1norm_init(divisor));break;
   }
   dump(ret,p,n+2);
  }
  int na=1+step%32,nb=1+(step/32)%32;memset(r,0xa5,sizeof(r));mp_mul_basecase(r,a,na,b,nb);dump(0,r,na+nb+2);
  nb=1+step%na;b[nb-1]|=(js_limb_t)1<<(JS_LIMB_BITS-1);
  if(step%16==5)memcpy(a,b,nb*sizeof(*a));
  if(step%16==6)for(int i=0;i<na;i++)a[i]=-1;
  memset(r,0xa5,sizeof(r));mp_divnorm(r,a,na,b,nb);dump(0,r,na-nb+2);dump(0,a,na+2);
 }
 for(int k=0;k<4096;k++){
  int64_t v=next();JSBigIntBuf buf={0};JSBigInt*p=js_bigint_set_si64(&buf,v);num(p->len,4);for(int i=0;i<p->len;i++)num(p->tab[i],sizeof(js_limb_t));num(js_bigint_sign(p),4);num(js_bigint_get_si_sat(p),sizeof(js_limb_t));
  p=js_bigint_set_short(&buf,JS_MKVAL(JS_TAG_SHORT_BIG_INT,v));num(p->len,4);num(p->tab[0],sizeof(js_limb_t));
  struct{int len;js_limb_t tab[4];} big={4,{next(),next(),next(),next()}};num(js_bigint_sign((JSBigInt*)&big),4);num(js_bigint_get_si_sat((JSBigInt*)&big),sizeof(js_limb_t));
 }
 return 0;
}
