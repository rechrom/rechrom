pub unsafe fn oracle_basics(){
 println!("layout {} {} {} {} {}",size_of::<CCInfo>(),size_of::<REString>(),core::mem::offset_of!(REString,buf),size_of::<TableEntry>(),size_of::<DecompEntry>());
 let rows=[c"",c"a;b;c",c";;",c" 0041  0062; 0043;",c" 0x10ffff 0000 1F600;"];
 for (row,text) in rows.iter().enumerate(){for field in 0..5{let p=get_field(text.as_ptr(),field);println!("field {row} {field} {}",if p.is_null(){"<null>"}else{CStr::from_ptr(p).to_str().unwrap()});let mut n=0;let buf=get_field_str(ptr::addr_of_mut!(n),text.as_ptr(),field);print!("ints {n}");for i in 0..n{print!(" {:08x}",*buf.add(i as usize));}println!();free(buf.cast());if !p.is_null(){for cap in 1..12{let mut b=[0xaa_u8 as c_char;12];get_field_buf(b.as_mut_ptr(),cap,text.as_ptr(),field);println!("buf {cap} {}",CStr::from_ptr(b.as_ptr()).to_str().unwrap());}}}}
 unicode_db=mallocz(size_of::<CCInfo>()*4096).cast();let mut h=1u32;
 for code in 0..4096{let ci=&mut*unicode_db.add(code as usize);ci.set_is_compat((code&1)as u8);ci.set_is_excluded(((code>>1)&1)as u8);for prop in 0..PROP_COUNT{let v=(((code*37+prop*17)&15)==0)as i32;set_prop(code as u32,prop,v);h=h.wrapping_mul(263).wrapping_add(get_prop(code as u32,prop)as u32);}h=h.wrapping_mul(263).wrapping_add(ci.is_compat()as u32+ci.is_excluded()as u32*3);}
 println!("bitmap {h}");free(unicode_db.cast());unicode_db=ptr::null_mut();
 let mut list:REStringList=core::mem::zeroed();re_string_list_init(&mut list);let mut buf=[0u32;8];h=1;
 for j in 0..4096{let n=(j%8)+1;for i in 0..n{buf[i as usize]=(j%1024)*113+i*37;}re_string_add(&mut list,n as i32,buf.as_ptr());let p=re_string_find(&mut list,n as i32,buf.as_ptr(),FALSE);assert!(!p.is_null());h=h.wrapping_mul(263).wrapping_add((*p).hash);h=h.wrapping_mul(263).wrapping_add((*p).len);let p2=re_string_find(&mut list,n as i32,buf.as_ptr(),FALSE);h=h.wrapping_mul(263).wrapping_add((p==p2)as u32);}
 println!("strings {} {} {} {h}",list.n_strings,list.hash_size,list.hash_bits);re_string_list_free(&mut list);
 let tmpname=c"/tmp/quickjs-unicode-gen-staging/line.txt";std::fs::write(CStr::from_ptr(tmpname.as_ptr()).to_str().unwrap(),b"a\nbbbbbbbb\r\nlast").unwrap();let f=fopen(tmpname.as_ptr(),c"rb".as_ptr());let mut b=[0 as c_char;5];while !get_line(b.as_mut_ptr(),5,f).is_null(){print!("line");for x in CStr::from_ptr(b.as_ptr()).to_bytes(){print!(" {x:02x}");}println!();}fclose(f);
}
