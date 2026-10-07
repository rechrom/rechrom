// Official quickjs.c Date constants, formatter and host time boundary. MIT.
const month_days: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const month_names: [u8; 37] = *b"JanFebMarAprMayJunJulAugSepOctNovDec\0";
const day_names: [u8; 22] = *b"SunMonTueWedThuFriSat\0";
unsafe fn date_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as i64,
        // gettimeofday supplies floor seconds and nonnegative microseconds.
        Err(e) => -(((e.duration().as_nanos() + 999_999) / 1_000_000) as i64),
    }
}
// c: quickjs.c:55063. Exact format/part selection; Rust formatting replaces
// snprintf only, with the source's explicit field widths and sign behavior.
unsafe fn get_date_string(ctx:*mut JSContext, this_val:JSValueConst, _argc:i32, _argv:*mut JSValueConst, magic:i32)->JSValue {
    use std::fmt::Write;
    let fmt = (magic >> 4) & 15;
    let part = magic & 15;
    let mut fields = [0.0;9];
    let res = get_date_fields(ctx,this_val,fields.as_mut_ptr(),fmt&1,0);
    if res < 0 { return JS_EXCEPTION; }
    if res == 0 { return if fmt == 2 { JS_ThrowRangeError(ctx,c"Date value is NaN".as_ptr()) } else { js_new_string8(ctx,c"Invalid Date".as_ptr()) }; }
    let (y,mon,d,h,m,s,ms,wd,mut tz)=(fields[0] as i32,fields[1] as usize,fields[2] as i32,fields[3] as i32,fields[4] as i32,fields[5] as i32,fields[6] as i32,fields[7] as usize,fields[8] as i32);
    let months=b"JanFebMarAprMayJunJulAugSepOctNovDec";
    let month=std::str::from_utf8_unchecked(&months[mon*3..mon*3+3]);
    let day=std::str::from_utf8_unchecked(&day_names[wd*3..wd*3+3]);
    let mut buf=String::with_capacity(64);
    if part & 1 != 0 {
        match fmt {
            0 => {let _=write!(buf,"{}, {:02} {} {:0width$} ",day,d,month,y,width=4+(y<0) as usize);}
            1 => {let _=write!(buf,"{} {} {:02} {:0width$}",day,month,d,y,width=4+(y<0) as usize);if part==3{buf.push(' ');}}
            2 => {if (0..=9999).contains(&y){let _=write!(buf,"{:04}",y);}else{let _=write!(buf,"{:+07}",y);}let _=write!(buf,"-{:02}-{:02}T",mon+1,d);}
            3 => {let _=write!(buf,"{:02}/{:02}/{:0width$}",mon+1,d,y,width=4+(y<0) as usize);if part==3{buf.push_str(", ");}}
            _=>{}
        }
    }
    if part & 2 != 0 {
        match fmt {
            0 => {let _=write!(buf,"{:02}:{:02}:{:02} GMT",h,m,s);}
            1 => {let _=write!(buf,"{:02}:{:02}:{:02} GMT",h,m,s);if tz<0{buf.push('-');tz=-tz;}else{buf.push('+');}let _=write!(buf,"{:02}{:02}",tz/60,tz%60);}
            2 => {let _=write!(buf,"{:02}:{:02}:{:02}.{:03}Z",h,m,s,ms);}
            3 => {let _=write!(buf,"{:02}:{:02}:{:02} {}M",(h+11)%12+1,m,s,if h<12{'A'}else{'P'});}
            _=>{}
        }
    }
    JS_NewStringLen(ctx,buf.as_ptr().cast(),buf.len())
}

#[repr(C)]
struct DateTzAbbreviation { name: [c_char;6], offset:i16 }
const fn date_abbreviation(s:&[u8],offset:i16)->DateTzAbbreviation{let mut name=[0;6];let mut i=0;while i<s.len(){name[i]=s[i] as c_char;i+=1;}DateTzAbbreviation{name,offset}}
const js_tzabbr:[DateTzAbbreviation;18]=[
 date_abbreviation(b"GMT",0),date_abbreviation(b"UTC",0),date_abbreviation(b"UT",0),date_abbreviation(b"Z",0),
 date_abbreviation(b"EDT",-240),date_abbreviation(b"EST",-300),date_abbreviation(b"CDT",-300),date_abbreviation(b"CST",-360),
 date_abbreviation(b"MDT",-360),date_abbreviation(b"MST",-420),date_abbreviation(b"PDT",-420),date_abbreviation(b"PST",-480),
 date_abbreviation(b"WET",0),date_abbreviation(b"WEST",60),date_abbreviation(b"CET",60),date_abbreviation(b"CEST",120),
 date_abbreviation(b"EET",120),date_abbreviation(b"EEST",180),
];
unsafe fn js_date_strchr(s:*const c_char,c:i32)->*mut c_char{let mut p=s;loop{if *p as u8==c as u8{return p.cast_mut();}if *p==0{return ptr::null_mut();}p=p.add(1);}}
