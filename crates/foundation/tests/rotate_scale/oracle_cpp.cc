#include "blink_geometry/transforms/rotate_transform_operation.h"
#include "blink_geometry/transforms/scale_transform_operation.h"
#include "gfx_geometry/transform.h"
#include <cstdio>
#include <cstdint>
#include <cmath>
#include <bit>
using blink::TransformOperation;
static void emit(const gfx::Transform& t){double values[16];t.GetColMajor(values);for(double value:values){uint64_t bits=std::bit_cast<uint64_t>(value);if(std::isnan(value))bits=0x7ff8000000000000ULL;std::fwrite(&bits,8,1,stdout);}}
int main(){
 const double axes[][3]={{0,0,0},{1,0,0},{0,1,0},{0,0,1},{0,0,-1},{-1,0,0},{0,-1,0},{1,2,3},{1e-20,1e-20,1e-20},{1e20,-1e20,1e20},{1,1e-8,0}};
 const double angles[]={0,-0.0,1e-8,-1e-8,44.9,45,45.1,90,180,270,360,-45,-360,123.456,90000000,-90000000,1e20,INFINITY,NAN};
 const int kinds[]={TransformOperation::kRotate,TransformOperation::kRotateX,TransformOperation::kRotateY,TransformOperation::kRotateZ,TransformOperation::kRotate3D};
 gfx::SizeF size(150,250);
 for(int setup=0;setup<3;setup++)for(auto& axis:axes)for(double angle:angles)for(int kind:kinds){
  gfx::Transform t;
  if(setup==1){t.Scale3d(1.1f,2.3f,1);t.Translate3d(4,5,0);}
  if(setup==2){const double m[]={1.25,-2.5,3.1,0.001,4.2,5.3,-6.4,0.002,-7.5,8.6,9.7,0.003,10.8,-11.9,12.1,0.999};t=gfx::Transform::ColMajor(m);}
  blink::RotateTransformOperation op(axis[0],axis[1],axis[2],angle,static_cast<TransformOperation::OperationType>(kind));op.Apply(t,size);emit(t);
  double payload[]={op.X(),op.Y(),op.Z(),op.Angle()};for(double v:payload){uint64_t bits=std::isnan(v)?0x7ff8000000000000ULL:std::bit_cast<uint64_t>(v);std::fwrite(&bits,8,1,stdout);}
 }
 const double factors[][3]={{1,1,1},{0,0,0},{-0.0,1,-1},{1.0000000596046448,1.2345678912345,1},{-2,3,-4},{1e30,1e-30,1e10},{INFINITY,1,1},{NAN,1,1}};
 const int scale_kinds[]={TransformOperation::kScale,TransformOperation::kScaleX,TransformOperation::kScaleY,TransformOperation::kScaleZ,TransformOperation::kScale3D};
 for(int setup=0;setup<3;setup++)for(auto& factor:factors)for(int kind:scale_kinds){gfx::Transform t;if(setup==1){t.Scale3d(1.1f,2.3f,1);t.Translate3d(4,5,0);}if(setup==2){double m[]={1.25,-2.5,3.1,0.001,4.2,5.3,-6.4,0.002,-7.5,8.6,9.7,0.003,10.8,-11.9,12.1,0.999};t=gfx::Transform::ColMajor(m);}blink::ScaleTransformOperation op(factor[0],factor[1],factor[2],static_cast<TransformOperation::OperationType>(kind));op.Apply(t,size);emit(t);}
 // Source equality includes operation type and does not make NaN reflexive.
 for(int a:kinds)for(int b:kinds){blink::RotateTransformOperation left(1,2,3,4,static_cast<TransformOperation::OperationType>(a)),right(1,2,3,4,static_cast<TransformOperation::OperationType>(b));unsigned char v=left==right;std::fwrite(&v,1,1,stdout);}
 for(int a:scale_kinds)for(int b:scale_kinds){blink::ScaleTransformOperation left(1,2,3,static_cast<TransformOperation::OperationType>(a)),right(1,2,3,static_cast<TransformOperation::OperationType>(b));unsigned char v=left==right;std::fwrite(&v,1,1,stdout);}
 blink::RotateTransformOperation rn(NAN,TransformOperation::kRotate);blink::ScaleTransformOperation sn(NAN,1,1,TransformOperation::kScale);unsigned char nan_equal[]={static_cast<unsigned char>(rn==rn),static_cast<unsigned char>(sn==sn)};std::fwrite(nan_equal,1,2,stdout);
 return 0;
}
