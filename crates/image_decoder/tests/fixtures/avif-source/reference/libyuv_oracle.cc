#include "libyuv.h"
#include <cstdio>
#include <cstdlib>
#include <cstdint>
#include <vector>
#include <cstring>
using namespace libyuv;
static std::vector<uint8_t> rgba(uint32_t* m, std::vector<uint8_t>& planes, int cpu) {
  MaskCpuFlags(cpu);
  const int w=m[0], h=m[1], f=m[2], mc=m[3], full=m[4], alpha=m[5];
  const int cw=f==444?w:(w+1)/2, ch=f==420?(h+1)/2:h;
  const uint8_t *y=planes.data();
  const uint8_t *u=f==400?nullptr:y+w*h;
  const uint8_t *v=f==400?nullptr:u+cw*ch;
  const uint8_t *a=f==400?y+w*h:v+cw*ch;
  std::vector<uint8_t> out(w*h*4);
  const YuvConstants* c=mc==1?(full?&kYvuF709Constants:&kYvuH709Constants):(full?&kYvuJPEGConstants:&kYvuI601Constants);
  int r;
  if(f==400) r=I400ToARGBMatrix(y,w,out.data(),w*4,c,w,h);
  else if(f==444) r=alpha?I444AlphaToARGBMatrix(y,w,v,cw,u,cw,a,w,out.data(),w*4,c,w,h,0):I444ToARGBMatrix(y,w,v,cw,u,cw,out.data(),w*4,c,w,h);
  else if(f==422) r=alpha?I422AlphaToARGBMatrixFilter(y,w,v,cw,u,cw,a,w,out.data(),w*4,c,w,h,0,kFilterBilinear):I422ToARGBMatrixFilter(y,w,v,cw,u,cw,out.data(),w*4,c,w,h,kFilterBilinear);
  else r=alpha?I420AlphaToARGBMatrixFilter(y,w,v,cw,u,cw,a,w,out.data(),w*4,c,w,h,0,kFilterBilinear):I420ToARGBMatrixFilter(y,w,v,cw,u,cw,out.data(),w*4,c,w,h,kFilterBilinear);
  if(r) { fprintf(stderr,"conversion failure %d\n",r); exit(3); }
  if(f==400 && alpha) for(int i=0;i<w*h;i++) out[i*4+3]=a[i];
  return out;
}
int main(int argc,char**argv) {
 if(argc!=3)return 2;
 FILE*in=fopen(argv[1],"rb"),*out=fopen(argv[2],"wb"); if(!in||!out)return 2;
 uint32_t m[6]; size_t cases=0,pixels=0;
 while(fread(m,sizeof(m),1,in)==1) {
  size_t cw=m[2]==444?m[0]:(m[0]+1)/2, ch=m[2]==420?(m[1]+1)/2:m[1];
  size_t n=m[0]*m[1]*(1+m[5])+(m[2]==400?0:2*cw*ch);
  std::vector<uint8_t> p(n);if(fread(p.data(),n,1,in)!=1)return 4;
  auto ref=rgba(m,p,1),simd=rgba(m,p,-1);
  if(ref!=simd){fprintf(stderr,"C/SIMD mismatch case %zu\n",cases);return 5;}
  fwrite(m,sizeof(m),1,out);fwrite(p.data(),p.size(),1,out);fwrite(ref.data(),ref.size(),1,out);
  cases++;pixels+=m[0]*m[1];
 }
 fclose(in);fclose(out);printf("%zu cases; %zu pixels; scalar/SIMD exact\n",cases,pixels);return 0;
}
