// quickjs.c:33764..33886. Complete original bytecode pattern matcher.
// The C variadic int sequence is represented as a typed slice.
unsafe fn code_match(s: *mut CodeContext, mut pos: i32, pattern: &[i32]) -> i32 {
    let tab = (*s).bc_buf;
    let mut line_num = -1;
    let mut args = pattern.iter();
    loop {
        let op1 = *args.next().expect("terminated bytecode pattern");
        if op1 == -1 { (*s).pos = pos; (*s).line_num = line_num; return 1; }
        let (op, pos_next) = loop {
            if pos >= (*s).bc_len { return 0; }
            let op = i32::from(*tab.offset(pos as isize));
            let len = i32::from(opcode_info[op as usize].size);
            let pos_next = pos + len;
            if pos_next > (*s).bc_len { return 0; }
            if op == OP_line_num as i32 {
                line_num = get_u32(tab.offset((pos + 1) as isize)) as i32;
                pos = pos_next;
            } else { break (op, pos_next); }
        };
        if op != op1 {
            if op1 == i32::from(op1 as u8) || op == 0 { return 0; }
            if ![op1 as u8, (op1 >> 8) as u8, (op1 >> 16) as u8, (op1 >> 24) as u8].contains(&(op as u8)) { return 0; }
            (*s).op = op;
        }
        pos += 1;
        let p = tab.offset(pos as isize);
        match opcode_info[op as usize].fmt as i32 {
            OP_FMT_loc8 | OP_FMT_u8 => {
                let idx = i32::from(*p); let arg = *args.next().expect("bytecode operand pattern");
                if arg == -1 { (*s).idx = idx; } else if arg != idx { return 0; }
            },
            OP_FMT_u16 | OP_FMT_npop | OP_FMT_loc | OP_FMT_arg | OP_FMT_var_ref => {
                let idx = get_u16(p) as i32; let arg = *args.next().expect("bytecode operand pattern");
                if arg == -1 { (*s).idx = idx; } else if arg != idx { return 0; }
            },
            OP_FMT_i32 | OP_FMT_u32 | OP_FMT_label | OP_FMT_const => { (*s).label = get_u32(p) as i32; },
            OP_FMT_label_u16 => { (*s).label = get_u32(p) as i32; (*s).val = get_u16(p.add(4)) as i32; },
            OP_FMT_atom => { (*s).atom = get_u32(p); },
            OP_FMT_atom_u8 => { (*s).atom = get_u32(p); (*s).val = i32::from(*p.add(4)); },
            OP_FMT_atom_u16 => { (*s).atom = get_u32(p); (*s).val = get_u16(p.add(4)) as i32; },
            OP_FMT_atom_label_u8 => { (*s).atom = get_u32(p); (*s).label = get_u32(p.add(4)) as i32; (*s).val = i32::from(*p.add(8)); },
            _ => {},
        }
        pos = pos_next;
    }
}
