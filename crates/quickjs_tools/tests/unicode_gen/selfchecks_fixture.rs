pub fn selfchecks_fixture_main()->i32 {
 let args:Vec<OsString>=std::env::args_os().collect();if args.len()<4{return 2;}
 let result=run(args[..3].iter().cloned());if result!=0{return result;}
 unsafe{match args[3].to_str(){Some("case")=>check_case_conv(),Some("flags")=>check_flags(),Some("decompose")=>check_decompose_table(),Some("compose")=>check_compose_table(),Some("cc")=>check_cc_table(),Some("normalize")=>{let path=args[1].to_string_lossy();let input=CString::new(format!("{path}/NormalizationTest.txt")).unwrap();normalization_test(input.as_ptr());},_=>return 2,}}
 0
}
