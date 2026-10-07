// unicode_gen.c constant/data declarations. Bellard/Gordon MIT.
const RUN_TYPE_U:i32=0;
const RUN_TYPE_L:i32=1;
const RUN_TYPE_UF:i32=2;
const RUN_TYPE_LF:i32=3;
const RUN_TYPE_UL:i32=4;
const RUN_TYPE_LSU:i32=5;
const RUN_TYPE_U2L_399_EXT2:i32=6;
const RUN_TYPE_UF_D20:i32=7;
const RUN_TYPE_UF_D1_EXT:i32=8;
const RUN_TYPE_U_EXT:i32=9;
const RUN_TYPE_LF_EXT:i32=10;
const RUN_TYPE_UF_EXT2:i32=11;
const RUN_TYPE_LF_EXT2:i32=12;
const RUN_TYPE_UF_EXT3:i32=13;
// unicode_gen.c:2693..2730.
type DecompTypeEnum=i32;
const DECOMP_TYPE_C1:i32=0;
const DECOMP_TYPE_L1:i32=1;
const DECOMP_TYPE_L2:i32=2;
const DECOMP_TYPE_L3:i32=3;
const DECOMP_TYPE_L4:i32=4;
const DECOMP_TYPE_L5:i32=5;
const DECOMP_TYPE_L6:i32=6;
const DECOMP_TYPE_L7:i32=7;
const DECOMP_TYPE_LL1:i32=8;
const DECOMP_TYPE_LL2:i32=9;
const DECOMP_TYPE_S1:i32=10;
const DECOMP_TYPE_S2:i32=11;
const DECOMP_TYPE_S3:i32=12;
const DECOMP_TYPE_S4:i32=13;
const DECOMP_TYPE_S5:i32=14;
const DECOMP_TYPE_I1:i32=15;
const DECOMP_TYPE_I2_0:i32=16;
const DECOMP_TYPE_I2_1:i32=17;
const DECOMP_TYPE_I3_1:i32=18;
const DECOMP_TYPE_I3_2:i32=19;
const DECOMP_TYPE_I4_1:i32=20;
const DECOMP_TYPE_I4_2:i32=21;
const DECOMP_TYPE_B1:i32=22;
const DECOMP_TYPE_B2:i32=23;
const DECOMP_TYPE_B3:i32=24;
const DECOMP_TYPE_B4:i32=25;
const DECOMP_TYPE_B5:i32=26;
const DECOMP_TYPE_B6:i32=27;
const DECOMP_TYPE_B7:i32=28;
const DECOMP_TYPE_B8:i32=29;
const DECOMP_TYPE_B18:i32=30;
const DECOMP_TYPE_LS2:i32=31;
const DECOMP_TYPE_PAT3:i32=32;
const DECOMP_TYPE_S2_UL:i32=33;
const DECOMP_TYPE_LS2_UL:i32=34;
static mut run_type_str:[*const c_char;14]=[c"U".as_ptr(),c"L".as_ptr(),c"UF".as_ptr(),c"LF".as_ptr(),c"UL".as_ptr(),c"LSU".as_ptr(),c"U2L_399_EXT2".as_ptr(),c"UF_D20".as_ptr(),c"UF_D1_EXT".as_ptr(),c"U_EXT".as_ptr(),c"LF_EXT".as_ptr(),c"UF_EXT2".as_ptr(),c"LF_EXT2".as_ptr(),c"UF_EXT3".as_ptr()];
static mut decomp_type_str:[*const c_char;35]=[c"C1".as_ptr(),c"L1".as_ptr(),c"L2".as_ptr(),c"L3".as_ptr(),c"L4".as_ptr(),c"L5".as_ptr(),c"L6".as_ptr(),c"L7".as_ptr(),c"LL1".as_ptr(),c"LL2".as_ptr(),c"S1".as_ptr(),c"S2".as_ptr(),c"S3".as_ptr(),c"S4".as_ptr(),c"S5".as_ptr(),c"I1".as_ptr(),c"I2_0".as_ptr(),c"I2_1".as_ptr(),c"I3_1".as_ptr(),c"I3_2".as_ptr(),c"I4_1".as_ptr(),c"I4_2".as_ptr(),c"B1".as_ptr(),c"B2".as_ptr(),c"B3".as_ptr(),c"B4".as_ptr(),c"B5".as_ptr(),c"B6".as_ptr(),c"B7".as_ptr(),c"B8".as_ptr(),c"B18".as_ptr(),c"LS2".as_ptr(),c"PAT3".as_ptr(),c"S2_UL".as_ptr(),c"LS2_UL".as_ptr()];
static mut conv_table:[TableEntry;1000]=[TableEntry{code:0,len:0,v_type:0,data:0,ext_len:0,ext_data:[0;3],data_index:0};1000];
static mut conv_table_len:i32=0;static mut ext_data:[i32;1000]=[0;1000];static mut ext_data_len:i32=0;static mut global_tab:*mut CCInfo=ptr::null_mut();
const decomp_incr_tab:[[i32;4];4]=[[DECOMP_TYPE_I1,0,-1,0],[DECOMP_TYPE_I2_0,0,1,-1],[DECOMP_TYPE_I3_1,1,2,-1],[DECOMP_TYPE_I4_1,1,2,-1]];
const unicode_short_table:[u16;2]=[0x2044,0x2215];
#[repr(C)]#[derive(Clone,Copy)]struct ComposeEntry{c:[u32;2],p:u32}
const COMPOSE_LEN_MAX:usize=10000;

// unicode_gen.c:926 and 2250..2253.
const SEQ_MAX_LEN:usize=16;
const EMOJI_MOD_NONE:i32=0;const EMOJI_MOD_TYPE1:i32=1;const EMOJI_MOD_TYPE2:i32=2;const EMOJI_MOD_TYPE2D:i32=3;
// unicode_gen.c:1844 and 2205.
const PROP_BLOCK_LEN:i32=32;const PROP_TABLE_COUNT:i32=PROP_ASCII;
