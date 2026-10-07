// Include the actual production helper; force the fallback without replacing cfg.
include!("../src/quickjs_libc_clock.rs");
fn main(){
 let seconds=[-9223372035i64,-2147483648,-1000000,-1,0,1,1000000,2147483647,9223372035];
 let micros=[-999999i64,-1001,-1000,-999,-1,0,1,999,1000,1001,999999];
 for sec in seconds {for us in micros {println!("{sec},{us}:{},{}",host_timeval_ms(sec,us),host_timeval_ns(sec,us));}}
 println!("wall:{},{}",host_gettimeofday_ms(),host_gettimeofday_ns());
}
