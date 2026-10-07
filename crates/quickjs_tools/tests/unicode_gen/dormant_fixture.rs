pub fn dormant_fixture_main()->i32 {
 let args:Vec<OsString>=std::env::args_os().collect();if args.len()<4{return 2;}let status=run(args[..3].iter().cloned());if status!=0{return status;}
 unsafe{match args[3].to_str(){
 Some("parse")=>{ptr::write_bytes(unicode_db,0,(CHARCODE_MAX+1)as usize);let name=CString::new(format!("{}/UnicodeData.txt",args[1].to_string_lossy())).unwrap();parse_unicode_data_dormant(name.as_ptr());},
 Some("conv")=>build_conv_table_dormant(unicode_db,-1),
 Some("props")=>{compute_internal_props_dormant();let mut h=0u32;for i in 0..=CHARCODE_MAX{h=h.wrapping_mul(31).wrapping_add(get_prop(i as u32,PROP_Cased1)as u32);}tool_printf(c"%08x\n".as_ptr(),&[ToolPrintArg::Int(h as u64)]);},
 Some("mark")=>{let mut sl:REStringList=core::mem::zeroed();let mut buf=[0x1f469,0x200d,0x1f4bb];re_string_list_init(ptr::addr_of_mut!(sl));re_string_add(ptr::addr_of_mut!(sl),3,buf.as_ptr());let result=mark_zwj_string_dormant(ptr::addr_of_mut!(sl),buf.as_mut_ptr(),3,EMOJI_MOD_NONE,ptr::null_mut(),-1,1);tool_printf(c"%d\n".as_ptr(),&[ToolPrintArg::Int(result as u64)]);re_string_list_free(ptr::addr_of_mut!(sl));},
 Some("zwj")=>{re_string_list_init(ptr::addr_of_mut!(rgi_emoji_zwj_sequence));let name=CString::new(format!("{}/emoji-zwj-sequences.txt",args[1].to_string_lossy())).unwrap();parse_sequence_prop_list(name.as_ptr());let file=fopen(c"/dev/null".as_ptr(),c"wb".as_ptr());build_rgi_emoji_zwj_sequence_dormant(file,ptr::addr_of_mut!(rgi_emoji_zwj_sequence));fclose(file);},
 Some("compose")=>{let file=fopen(c"/dev/null".as_ptr(),c"wb".as_ptr());build_decompose_table_with_dormant_composition(file);fclose(file);},
 Some("large")=>dump_large_char(),_=>return 2,
 }}0
}
