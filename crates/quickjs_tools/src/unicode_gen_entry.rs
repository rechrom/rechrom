// c: unicode_gen.c:3663..3779. Bellard/Gordon MIT.
// This executable has the upstream generator's process-lifetime global state.
unsafe fn unicode_gen_main(argc:i32,argv:*mut *mut c_char)->i32 {
 let mut filename=[0 as c_char;1024];let mut arg=1;
 if arg>=argc || strcmp(*argv.add(arg as usize),c"-h".as_ptr())==0 || strcmp(*argv.add(arg as usize),c"--help".as_ptr())==0 {
  tool_printf(c"usage: %s PATH [OUTPUT]\n  PATH    path to the Unicode database directory\n  OUTPUT  name of the output file.  If omitted, a self test is performed\n          using the files from the Unicode library\n".as_ptr(),&[ToolPrintArg::Str(*argv)]);return 1;
 }
 let unicode_db_path=*argv.add(arg as usize);arg+=1;
 let outfilename=if arg<argc{*argv.add(arg as usize)}else{ptr::null_mut()};
 unicode_db=mallocz(size_of::<CCInfo>()*((CHARCODE_MAX+1)as usize)).cast();
 re_string_list_init(ptr::addr_of_mut!(rgi_emoji_zwj_sequence));
 dbuf_init(ptr::addr_of_mut!(rgi_emoji_tag_sequence));
 macro_rules! input {($name:literal)=>{{tool_snprintf(filename.as_mut_ptr(),filename.len(),concat!("%s/",$name,"\0").as_ptr().cast(),&[ToolPrintArg::Str(unicode_db_path)]);filename.as_ptr()}};}
 parse_unicode_data(input!("UnicodeData.txt"));
 parse_special_casing(unicode_db,input!("SpecialCasing.txt"));
 parse_case_folding(unicode_db,input!("CaseFolding.txt"));
 parse_composition_exclusions(input!("CompositionExclusions.txt"));
 parse_derived_core_properties(input!("DerivedCoreProperties.txt"));
 parse_derived_norm_properties(input!("DerivedNormalizationProps.txt"));
 parse_prop_list(input!("PropList.txt"));
 parse_scripts(input!("Scripts.txt"));
 parse_script_extensions(input!("ScriptExtensions.txt"));
 parse_prop_list(input!("emoji-data.txt"));
 parse_sequence_prop_list(input!("emoji-sequences.txt"));
 parse_sequence_prop_list(input!("emoji-zwj-sequences.txt"));
 build_conv_table(unicode_db);
 #[cfg(feature="unicode-dump-case-folding-special-cases")]dump_case_folding_special_cases(unicode_db);
 if outfilename.is_null(){
  #[cfg(feature="unicode-use-test")]{check_case_conv();check_flags();check_decompose_table();check_compose_table();check_cc_table();normalization_test(input!("NormalizationTest.txt"));}
  #[cfg(not(feature="unicode-use-test"))]{tool_fprintf(tool_stderr,c"Tests are not compiled\n".as_ptr(),&[]);tool_exit(1);}
 }else{
  let fo=fopen(outfilename,c"wb".as_ptr());if fo.is_null(){perror(outfilename);tool_exit(1);}
  tool_fprintf(fo,c"/* Compressed unicode tables */\n/* Automatically generated file - do not edit */\n\n#include <stdint.h>\n\n".as_ptr(),&[]);
  dump_case_conv_table(fo);compute_internal_props();build_flags_tables(fo);
  tool_fprintf(fo,c"#ifdef CONFIG_ALL_UNICODE\n\n".as_ptr(),&[]);
  build_cc_table(fo);build_decompose_table(fo);build_general_category_table(fo);build_script_table(fo);build_script_ext_table(fo);build_prop_list_table(fo);build_sequence_prop_list_table(fo);
  tool_fprintf(fo,c"#endif /* CONFIG_ALL_UNICODE */\n".as_ptr(),&[]);
  tool_fprintf(fo,c"/* %u tables / %u bytes, %u index / %u bytes */\n".as_ptr(),&[ToolPrintArg::Int(total_tables as u64),ToolPrintArg::Int(total_table_bytes as u64),ToolPrintArg::Int(total_index as u64),ToolPrintArg::Int(total_index_bytes as u64)]);
  fclose(fo);
 }
 re_string_list_free(ptr::addr_of_mut!(rgi_emoji_zwj_sequence));0
}
/// Execute the standalone Unicode generator with the complete upstream CLI.
/// Algorithms keep upstream process-global state, so this tool is invoked once.
pub fn run(args:impl IntoIterator<Item=OsString>)->i32 {
 let mut args:Vec<CString>=args.into_iter().map(|arg|{
  #[cfg(unix)]let bytes={use std::os::unix::ffi::OsStrExt;arg.as_os_str().as_bytes().to_vec()};
  #[cfg(not(unix))]let bytes=arg.to_string_lossy().as_bytes().to_vec();
  CString::new(bytes).expect("operating-system arguments cannot contain NUL")
 }).collect();
 if args.is_empty(){args.push(CString::new("unicode_gen").unwrap());}
 let mut argv:Vec<*mut c_char>=args.iter().map(|arg|arg.as_ptr()as*mut c_char).collect();let argc=argv.len()as i32;argv.push(ptr::null_mut());
 // Safe public entry admits a single execution, matching the C executable.
 static USED:std::sync::atomic::AtomicBool=std::sync::atomic::AtomicBool::new(false);
 if USED.swap(true,std::sync::atomic::Ordering::AcqRel){eprintln!("unicode_gen can execute only once per process");return 1;}
 unsafe{unicode_gen_main(argc,argv.as_mut_ptr())}
}
