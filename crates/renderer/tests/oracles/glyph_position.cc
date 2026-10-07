// Reference: SkGlyphRunPainter.cpp direct CPU mask positioning, using the
// project's existing Skia archive. Output is little-endian f32 pairs.
#include <array>
#include <cstdio>
#include <cmath>
#include "include/core/SkMatrix.h"
#include "src/core/SkGlyph.h"
int main() {
 const std::array<std::array<float,6>,6> transforms{{
  {{1,0,0,1,0,0}},{{1,0,0,1,10.125f,-7.625f}},{{2,0,0,0.75f,4,5}},
  {{0,1,-1,0,3,5}},{{1,0,0.25f,1,0,0}},{{1,0.25f,0.125f,1,0,0}}
 }};
 for(const auto& t:transforms) {
  SkMatrix m=SkMatrix::MakeAll(t[0],t[2],t[4],t[1],t[3],t[5],0,0,1);
  auto axis=t[1]==0?SkAxisAlignment::kX:t[0]==0?SkAxisAlignment::kY:SkAxisAlignment::kNone;
  SkGlyphPositionRoundingSpec rounding(true,axis);
  auto half=rounding.halfAxisSampleFreq;
  auto mask=rounding.ignorePositionFieldMask;
  m.postTranslate(half.x(),half.y());
  for(int i=0;i<1024;i++){
   float x=(i-512)/64.f,y=((i*37)%1024-512)/64.f;
   auto mapped=m.mapPoint({x,y});SkPackedGlyphID id(1,mapped,mask);
   float result[2]={floorf(mapped.x())+SkFixedToFloat(id.getSubXFixed()),floorf(mapped.y())+SkFixedToFloat(id.getSubYFixed())};
   if(fwrite(result,sizeof(result),1,stdout)!=1)return 1;
  }
 }
}
