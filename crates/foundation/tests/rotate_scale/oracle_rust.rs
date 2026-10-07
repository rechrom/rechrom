use foundation::{gfx,RotateTransformOperation,ScaleTransformOperation,TransformOperationType as Kind};
use std::io::{self,Write};
fn bits(v:f64)->u64{if v.is_nan(){0x7ff8000000000000}else{v.to_bits()}}
fn emit(t:&gfx::Transform,out:&mut Vec<u8>){for value in t.GetColMajor(){out.extend_from_slice(&bits(value).to_ne_bytes());}}
fn setup(mode:usize)->gfx::Transform{let mut t=gfx::Transform::default();if mode==1{t.Scale3d(1.1,2.3,1.0);t.Translate3d(4.0,5.0,0.0);}if mode==2{t=gfx::Transform::ColMajor(&[1.25,-2.5,3.1,0.001,4.2,5.3,-6.4,0.002,-7.5,8.6,9.7,0.003,10.8,-11.9,12.1,0.999]);}t}
fn main(){
 let mut out=Vec::new();let axes=[[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0],[0.0,0.0,-1.0],[-1.0,0.0,0.0],[0.0,-1.0,0.0],[1.0,2.0,3.0],[1e-20,1e-20,1e-20],[1e20,-1e20,1e20],[1.0,1e-8,0.0]];
 let angles=[0.0,-0.0,1e-8,-1e-8,44.9,45.0,45.1,90.0,180.0,270.0,360.0,-45.0,-360.0,123.456,90000000.0,-90000000.0,1e20,f64::INFINITY,f64::NAN];
 let kinds=[Kind::kRotate,Kind::kRotateX,Kind::kRotateY,Kind::kRotateZ,Kind::kRotate3D];let size=gfx::SizeF::new(150.0,250.0);
 for mode in 0..3{for axis in axes{for angle in angles{for kind in kinds{let mut t=setup(mode);let op=RotateTransformOperation::new_3d(axis[0],axis[1],axis[2],angle,kind);op.Apply(&mut t,&size);emit(&t,&mut out);for v in[op.X(),op.Y(),op.Z(),op.Angle()]{out.extend_from_slice(&bits(v).to_ne_bytes());}}}}}
 let factors=[[1.0,1.0,1.0],[0.0,0.0,0.0],[-0.0,1.0,-1.0],[1.0000000596046448,1.2345678912345,1.0],[-2.0,3.0,-4.0],[1e30,1e-30,1e10],[f64::INFINITY,1.0,1.0],[f64::NAN,1.0,1.0]];let scale_kinds=[Kind::kScale,Kind::kScaleX,Kind::kScaleY,Kind::kScaleZ,Kind::kScale3D];
 for mode in 0..3{for factor in factors{for kind in scale_kinds{let mut t=setup(mode);let op=ScaleTransformOperation::new(factor[0],factor[1],factor[2],kind);op.Apply(&mut t,&size);emit(&t,&mut out);}}}
 for a in kinds{for b in kinds{out.push((RotateTransformOperation::new_3d(1.0,2.0,3.0,4.0,a)==RotateTransformOperation::new_3d(1.0,2.0,3.0,4.0,b))as u8);}}
 for a in scale_kinds{for b in scale_kinds{out.push((ScaleTransformOperation::new(1.0,2.0,3.0,a)==ScaleTransformOperation::new(1.0,2.0,3.0,b))as u8);}}
 let rn=RotateTransformOperation::new(f64::NAN,Kind::kRotate);let sn=ScaleTransformOperation::new(f64::NAN,1.0,1.0,Kind::kScale);out.extend_from_slice(&[(rn==rn)as u8,(sn==sn)as u8]);io::stdout().lock().write_all(&out).unwrap();
}
