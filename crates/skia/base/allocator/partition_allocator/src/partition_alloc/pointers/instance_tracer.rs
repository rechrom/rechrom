use super::raw_ptr::InstanceTracer;
// Source: partition_alloc/pointers/instance_tracer.h:95-107
pub struct DisabledInstanceTracer;
unsafe impl InstanceTracer for DisabledInstanceTracer{
 fn new()->Self{Self}fn owner_id(&self)->u64{0}
}
// Source: partition_alloc/pointers/instance_tracer.h:28-40,77-90
#[cfg(target_has_atomic="64")]
pub struct ActiveInstanceTracer{owner_id_:u64}
// Source: partition_alloc/pointers/instance_tracer.cc:53
#[cfg(target_has_atomic="64")]
static COUNTER:core::sync::atomic::AtomicU64=core::sync::atomic::AtomicU64::new(0);
#[cfg(target_has_atomic="64")]
unsafe impl InstanceTracer for ActiveInstanceTracer{
 fn new()->Self{Self{owner_id_:COUNTER.fetch_add(1,core::sync::atomic::Ordering::SeqCst).wrapping_add(1)}}
 fn owner_id(&self)->u64{self.owner_id_}
}
#[cfg(target_has_atomic="64")]
impl ActiveInstanceTracer{
 // Native copies/moves construct a new ID; assignments preserve destination ID.
 pub fn copy_from(_other:&Self)->Self{<Self as InstanceTracer>::new()}
 pub fn move_from(_other:&mut Self)->Self{<Self as InstanceTracer>::new()}
 pub fn assign(&mut self,_other:&Self){}
 // Source: partition_alloc/pointers/instance_tracer.h:42-73
 // TraceImpl/UntraceImpl and stack queries require actual PA slot metadata,
 // CollectStackTrace, NoDestructor<std::map> and mutex producers in .cc.
 // They are pending, not empty active tracer methods or guessed native symbols.
}
