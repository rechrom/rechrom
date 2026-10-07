static uint64_t rng_debug(uint64_t*s){*s^=*s<<13;*s^=*s>>7;*s^=*s<<17;return *s;}
int main(void){
 uint64_t rng=0x987654321abcdef0ULL;
 for(int i=0;i<32768;i++){
  uint64_t bits=rng_debug(&rng);DynBuf db;dbuf_init(&db);dbuf_put_leb128(&db,bits);dbuf_put_sleb128(&db,bits>>32);num(db.size,4);for(int j=0;j<db.size;j++)num(db.buf[j],1);
  for(int size=0;size<=db.size;size++){uint32_t u=0xa5a5a5a5;int32_t v=0xa5a5a5a5;int ret=get_leb128(&u,db.buf,db.buf+size);num(ret,4);num(u,4);ret=get_sleb128(&v,db.buf,db.buf+size);num(ret,4);num(v,4);}dbuf_free(&db);
  uint8_t bytes[128]={0};int size=i%129;for(int j=0;j<size;j++)bytes[j]=rng_debug(&rng);
  for(int offset=0;offset<=size;offset++){uint32_t u=0;int ret=get_leb128(&u,bytes+offset,bytes+size);num(ret,4);num(u,4);}
  JSFunctionBytecode b={0};b.has_debug=i%2;b.debug.pc2line_buf=bytes;b.debug.pc2line_len=size;uint32_t pcs[]={0,1,63,255,65535,UINT32_MAX,bits};
  for(int j=0;j<7;j++){int col=0xa5a5a5a5;int line=find_line_num(NULL,&b,pcs[j],&col);num(line,4);num(col,4);}
 }
 for(int prop_flags=0;prop_flags<64;prop_flags++)for(int flags=0;flags<16384;flags++){num(check_define_prop_flags(prop_flags,flags),4);num(get_prop_flags(flags,prop_flags),4);}
 return 0;
}
