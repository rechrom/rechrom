// Rust host implementation of quickjs.c:47227 getTimezoneOffset's localtime
// boundary. TZif v1/v2/v3/v4 transition data and POSIX tail rules are interpreted
// directly; neither an embedded JavaScript engine nor C time functions are used.
mod qjs_date_timezone {
    #[derive(Clone, Copy)]
    enum Rule { Julian(i32), Day(i32), Month(i32,i32,i32) }
    #[derive(Clone, Copy)]
    struct Change { rule: Rule, seconds:i32, clock:u8 }
    struct Posix { standard:i32, daylight:i32, start:Change, end:Change, has_dst:bool, implicit:bool }
    fn integer(s:&[u8], p:&mut usize)->Option<i32>{let start=*p;let mut v=0i32;while *p<s.len()&&s[*p].is_ascii_digit(){v=v.checked_mul(10)?.checked_add((s[*p]-b'0') as i32)?;*p+=1;}if *p==start{None}else{Some(v)}}
    fn signed_time(s:&[u8],p:&mut usize)->Option<i32>{let sign=if s.get(*p)==Some(&b'-'){*p+=1;-1}else{if s.get(*p)==Some(&b'+'){*p+=1;}1};let h=integer(s,p)?;let mut m=0;let mut sec=0;if s.get(*p)==Some(&b':'){*p+=1;m=integer(s,p)?;if s.get(*p)==Some(&b':'){*p+=1;sec=integer(s,p)?;}}if m>59||sec>59{return None;}h.checked_mul(3600)?.checked_add(m*60+sec)?.checked_mul(sign)}
    fn name(s:&[u8],p:&mut usize)->Option<()>{let start=*p;if s.get(*p)==Some(&b'<'){*p+=1;while *p<s.len()&&s[*p]!=b'>'{*p+=1;}if *p==s.len(){return None;}*p+=1;}else{while *p<s.len()&&s[*p].is_ascii_alphabetic(){*p+=1;}if *p-start<3{return None;}}Some(())}
    fn change(s:&[u8],p:&mut usize)->Option<Change>{
        let rule=match s.get(*p)?{
            b'J'=>{*p+=1;let n=integer(s,p)?;if !(1..=365).contains(&n){return None;}Rule::Julian(n)}
            b'M'=>{*p+=1;let m=integer(s,p)?;if s.get(*p)!=Some(&b'.'){return None;}*p+=1;let w=integer(s,p)?;if s.get(*p)!=Some(&b'.'){return None;}*p+=1;let d=integer(s,p)?;if !(1..=12).contains(&m)||!(1..=5).contains(&w)||!(0..=6).contains(&d){return None;}Rule::Month(m,w,d)}
            _=>{let n=integer(s,p)?;if !(0..=365).contains(&n){return None;}Rule::Day(n)}
        };
        let mut seconds=7200;let mut clock=b'w';if s.get(*p)==Some(&b'/'){*p+=1;seconds=signed_time(s,p)?;if matches!(s.get(*p),Some(b'w'|b's'|b'u'|b'g'|b'z')){clock=s[*p];*p+=1;}}
        Some(Change{rule,seconds,clock})
    }
    fn posix(s:&[u8])->Option<Posix>{
        let mut p=0;name(s,&mut p)?;let standard=-signed_time(s,&mut p)?;
        let default=Change{rule:Rule::Day(0),seconds:0,clock:b'w'};
        if p==s.len(){return Some(Posix{standard,daylight:standard,start:default,end:default,has_dst:false,implicit:false});}
        name(s,&mut p)?;let daylight=if s.get(p)==Some(&b',')||p==s.len(){standard+3600}else{-signed_time(s,&mut p)?};
        // POSIX permits omitted rules; tzcode uses the current posixrules file.
        let implicit=p==s.len();
        let (start,end)=if implicit{(Change{rule:Rule::Month(4,1,0),seconds:7200,clock:b'w'},Change{rule:Rule::Month(10,5,0),seconds:7200,clock:b'w'})}else{
            if s.get(p)!=Some(&b','){return None;}p+=1;let start=change(s,&mut p)?;if s.get(p)!=Some(&b','){return None;}p+=1;let end=change(s,&mut p)?;if p!=s.len(){return None;}(start,end)
        };
        Some(Posix{standard,daylight,start,end,has_dst:true,implicit})
    }
    fn leap(y:i64)->bool{y%4==0&&(y%100!=0||y%400==0)}
    fn days(y:i64)->i64{365*(y-1970)+(y-1969).div_euclid(4)-(y-1901).div_euclid(100)+(y-1601).div_euclid(400)}
    fn year(t:i64)->i64{let d=t.div_euclid(86400);let mut y=(d*10000).div_euclid(3652425)+1970;while d<days(y){y-=1;}while d>=days(y+1){y+=1;}y}
    fn when(c:Change,y:i64,wall:i32,standard:i32)->i64{
        let doy=match c.rule{
            Rule::Day(n)=>n,
            Rule::Julian(n)=>n-1+((leap(y)&&n>=60) as i32),
            Rule::Month(m,w,d)=>{let lens=[31,28+leap(y) as i32,31,30,31,30,31,31,30,31,30,31];let before:i32=lens[..(m-1) as usize].iter().sum();let first=(days(y)+before as i64+4).rem_euclid(7) as i32;let mut date=(d-first).rem_euclid(7)+1+(w-1)*7;if date>lens[(m-1) as usize]{date-=7;}before+date-1}
        };
        let offset=match c.clock{b'u'|b'g'|b'z'=>0,b's'=>standard,_=>wall};(days(y)+doy as i64)*86400+c.seconds as i64-offset as i64
    }
    fn posix_offset(z:&Posix,t:i64)->i32{
        if !z.has_dst{return z.standard;}
        let y=year(t);let a=when(z.start,y,z.standard,z.standard);let b=when(z.end,y,z.daylight,z.standard);
        let dst=if a<b{t>=a&&t<b}else{t>=a||t<b};if dst{z.daylight}else{z.standard}
    }
    fn u32_at(d:&[u8],p:usize)->Option<u32>{Some(u32::from_be_bytes(d.get(p..p+4)?.try_into().ok()?))}
    fn block(d:&[u8],h:usize,width:usize,t:i64)->Option<(i32,usize,i64)>{
        if d.get(h..h+4)?!=b"TZif"{return None;}
        let utc=u32_at(d,h+20)? as usize;let std=u32_at(d,h+24)? as usize;let leaps=u32_at(d,h+28)? as usize;let n=u32_at(d,h+32)? as usize;let types=u32_at(d,h+36)? as usize;let chars=u32_at(d,h+40)? as usize;
        if types==0||types>256{return None;}
        let times=h.checked_add(44)?;let indices=times.checked_add(n.checked_mul(width)?)?;let records=indices.checked_add(n)?;let end=records.checked_add(types.checked_mul(6)?)?.checked_add(chars)?.checked_add(leaps.checked_mul(width+4)?)?.checked_add(std)?.checked_add(utc)?;d.get(..end)?;
        let transition=|i:usize|->Option<i64>{let p=times+i*width;Some(if width==8{i64::from_be_bytes(d.get(p..p+8)?.try_into().ok()?)}else{i32::from_be_bytes(d.get(p..p+4)?.try_into().ok()?) as i64})};
        let mut lo=0;let mut hi=n;while lo<hi{let mid=lo+(hi-lo)/2;if transition(mid)?<=t{lo=mid+1}else{hi=mid}}
        // localsub chooses the first standard type before the first transition.
        let k=if lo==0{(0..types).find(|&i|d[records+i*6+4]==0).unwrap_or(0)}else{*d.get(indices+lo-1)? as usize};if k>=types{return None;}
        let offset=u32_at(d,records+k*6)? as i32;let last=if n==0{i64::MIN}else{transition(n-1)?};Some((offset,end,last))
    }
    fn tzif(d:&[u8],t:i64)->Option<i32>{
        let (offset,end,_)=block(d,0,4,t)?;
        if matches!(d.get(4),Some(b'2'|b'3'|b'4')){
            let (offset,end,last)=block(d,end,8,t)?;
            if t>last&&d.get(end)==Some(&b'\n'){
                let tail=d.get(end+1..)?;let n=tail.iter().position(|&c|c==b'\n')?;if n>0{if let Some(z)=posix(&tail[..n]){return Some(posix_offset(&z,t));}}
            }
            Some(offset)
        }else{Some(offset)}
    }
    fn zone_dir()->std::path::PathBuf{
        #[cfg(target_vendor="apple")]
        {std::path::PathBuf::from("/var/db/timezone/zoneinfo")}
        #[cfg(not(target_vendor="apple"))]
        {std::path::PathBuf::from("/usr/share/zoneinfo")}
    }
    // Apple tzparse's omitted-rule path borrows the installed posixrules
    // transition table without tzload tail extension, then converts its UTC
    // offsets to the requested standard/daylight offsets.
    fn implicit_offset(z:&Posix,t:i64)->Option<i32>{
        let d=std::fs::read(zone_dir().join("posixrules")).ok()?;
        let (_,first_end,_)=block(&d,0,4,t)?;
        let (h,width)=if matches!(d.get(4),Some(b'2'|b'3'|b'4')){(first_end,8)}else{(0,4)};
        let utc=u32_at(&d,h+20)? as usize;let std=u32_at(&d,h+24)? as usize;
        let leaps=u32_at(&d,h+28)? as usize;let n=u32_at(&d,h+32)? as usize;let types=u32_at(&d,h+36)? as usize;let chars=u32_at(&d,h+40)? as usize;
        let times=h+44;let indices=times+n*width;let records=indices+n;let flags=records+types*6+chars+leaps*(width+4);
        let mut their_standard=0;let mut their_daylight=0;let mut found_std=false;let mut found_dst=false;
        for i in 0..n{let k=*d.get(indices+i)? as usize;if k>=types{return None;}let daylight=*d.get(records+k*6+4)?!=0;let offset=u32_at(&d,records+k*6)? as i32;
            if daylight&&!found_dst{their_daylight=offset;found_dst=true;}
            if !daylight&&!found_std{their_standard=offset;found_std=true;}
            if found_std&&found_dst{break;}
        }
        let mut result=z.standard;
        // The source initializes isdst=FALSE and preserves it throughout this
        // adjustment loop. Keep its behavior, including historical transitions.
        let isdst=false;
        for i in 0..n{let k=*d.get(indices+i)? as usize;if k>=types{return None;}
            let p=times+i*width;let mut at=if width==8{i64::from_be_bytes(d.get(p..p+8)?.try_into().ok()?)}else{i32::from_be_bytes(d.get(p..p+4)?.try_into().ok()?) as i64};
            let is_gmt=utc!=0&&*d.get(flags+std+k)?!=0;let is_std=std!=0&&*d.get(flags+k)?!=0;
            if !is_gmt{at+=(if isdst&&!is_std{their_daylight-z.daylight}else{their_standard-z.standard}) as i64;}
            let daylight=*d.get(records+k*6+4)?!=0;let offset=u32_at(&d,records+k*6)? as i32;
            if daylight{their_daylight=offset;}else{their_standard=offset;}
            if at<=t{result=if daylight{z.daylight}else{z.standard};}
        }
        Some(result)
    }
    pub fn offset(t:i64)->i32{
        let tz=std::env::var_os("TZ");
        let path=if let Some(ref tz)=tz {
            let raw=tz.to_string_lossy();let colon=raw.starts_with(':');let raw=raw.strip_prefix(':').unwrap_or(&raw);
            if raw.is_empty(){return 0;}
            let zone_path=if raw.starts_with('/') {std::path::PathBuf::from(raw)}else{zone_dir().join(raw)};
            if let Ok(data)=std::fs::read(&zone_path){if let Some(offset)=tzif(&data,t){return offset;}}
            // A leading colon requests a file, never POSIX TZ parsing.
            if colon{return 0;}
            if let Some(z)=posix(raw.as_bytes()) {
                if z.implicit {
                    if let Some(offset)=implicit_offset(&z,t){return offset;}
                }
                // Apple tzparse materializes TZ_MAX_TIMES=1200 transitions
                // (1970..2569) for an explicit environment rule. Its localsub
                // chooses the first standard type before that table, and the
                // last transition type afterwards; TZif tail extrapolation is
                // a separate tzload path and deliberately remains unlimited.
                // Reference: Apple Libc stdtime/FreeBSD/localtime.c (public
                // domain Arthur David Olson), tzparse/localsub; tzfile.h.
                #[cfg(target_vendor="apple")]
                if z.has_dst {
                    let first=when(z.start,1970,z.standard,z.standard).min(when(z.end,1970,z.daylight,z.standard));
                    let last_start=when(z.start,2569,z.standard,z.standard);
                    let last_end=when(z.end,2569,z.daylight,z.standard);
                    if t<first{return z.standard;}
                    if t>=last_start.max(last_end){return if last_start>last_end{z.daylight}else{z.standard};}
                }
                return posix_offset(&z,t);
            }
            if raw.starts_with('/') {std::path::PathBuf::from(raw)}else{zone_dir().join(raw)}
        }else{std::path::PathBuf::from("/etc/localtime")};
        // Apple tzset_basic/gmtload explicitly uses UTC after both a failed
        // TZif lookup and a failed POSIX parse. This is that host fallback,
        // rather than a replacement for valid or unknown offset algorithms.
        std::fs::read(path).ok().and_then(|d|tzif(&d,t)).unwrap_or(0)
    }
}
// c: quickjs.c:47227. The source divides milliseconds before localtime_r and
// truncates sub-minute historical zone offsets toward zero.
unsafe fn getTimezoneOffset(time:i64)->i32 { -qjs_date_timezone::offset(time/1000)/60 }
