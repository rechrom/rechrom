// qjs.c argv is a byte array. POSIX filenames, source expressions and script
// arguments retain all bytes; no Unicode decoder intervenes at the CLI boundary.
#[derive(Debug)]
pub struct ByteCliError {
    pub message: Vec<u8>,
    pub help: bool,
    pub exit: i32,
}
impl ByteCliError {
    fn new(message: impl Into<Vec<u8>>, exit: i32) -> Self {
        Self {
            message: message.into(),
            help: false,
            exit,
        }
    }
}
impl From<CliError> for ByteCliError {
    fn from(e: CliError) -> Self {
        Self {
            message: e.message.into_bytes(),
            help: e.help,
            exit: e.exit,
        }
    }
}
#[derive(Debug)]
pub struct RawOptions {
    pub scalar: Options,
    pub expr: Option<Vec<u8>>,
    pub include_list: Vec<Vec<u8>>,
}
pub fn get_suffixed_size_bytes(value: &[u8]) -> Result<usize, ByteCliError> {
    // Original strtod accepts only ASCII numeric syntax in the tool's default
    // C locale. Preserve byte offsets even in an invalid suffix.
    let ascii = value
        .iter()
        .map(|&b| if b.is_ascii() { b } else { 127 })
        .collect::<Vec<_>>();
    let text = std::str::from_utf8(&ascii).unwrap();
    match get_suffixed_size(text) {
        Ok(n) => Ok(n),
        Err(e) => {
            let prefix = b"qjs: invalid suffix: ";
            if let Some(suffix) = e
                .message
                .as_bytes()
                .strip_prefix(prefix)
                .and_then(|b| b.strip_suffix(b"\n"))
            {
                let mut message = prefix.to_vec();
                message.extend_from_slice(&value[value.len() - suffix.len()..]);
                message.push(b'\n');
                Err(ByteCliError::new(message, 1))
            } else {
                Err(e.into())
            }
        }
    }
}
pub fn parse_raw_options(argv: &[Vec<u8>]) -> Result<RawOptions, ByteCliError> {
    let mut o = RawOptions {
        scalar: Options::default(),
        expr: None,
        include_list: Vec::new(),
    };
    while o.scalar.optind < argv.len() && argv[o.scalar.optind].starts_with(b"-") {
        let argument = &argv[o.scalar.optind];
        let mut arg = &argument[1..];
        if arg.is_empty() {
            break;
        }
        o.scalar.optind += 1;
        let mut longopt = &b""[..];
        if arg.starts_with(b"-") {
            longopt = &arg[1..];
            arg = b"";
            if longopt.is_empty() {
                break;
            }
        }
        while !arg.is_empty() || !longopt.is_empty() {
            let opt = arg.first().copied().unwrap_or(0);
            if opt != 0 {
                arg = &arg[1..];
            }
            if opt == b'h' || opt == b'?' || longopt == b"help" {
                return Err(ByteCliError {
                    message: Vec::new(),
                    help: true,
                    exit: 1,
                });
            }
            if opt == b'e' || longopt == b"eval" {
                if !arg.is_empty() {
                    o.expr = Some(arg.to_vec());
                    break;
                }
                if o.scalar.optind < argv.len() {
                    o.expr = Some(argv[o.scalar.optind].clone());
                    o.scalar.optind += 1;
                    break;
                }
                return Err(ByteCliError::new(
                    b"qjs: missing expression for -e\n".to_vec(),
                    2,
                ));
            } else if opt == b'I' || longopt == b"include" {
                if o.scalar.optind >= argv.len() {
                    return Err(ByteCliError::new(b"expecting filename".to_vec(), 1));
                }
                if o.include_list.len() >= 32 {
                    return Err(ByteCliError::new(b"too many included files".to_vec(), 1));
                }
                o.include_list.push(argv[o.scalar.optind].clone());
                o.scalar.optind += 1;
            } else if opt == b'i' || longopt == b"interactive" {
                o.scalar.interactive += 1;
            } else if opt == b'm' || longopt == b"module" {
                o.scalar.module = 1;
            } else if longopt == b"script" {
                o.scalar.module = 0;
            } else if longopt == b"strict" {
                o.scalar.strict = 1;
            } else if opt == b'd' || longopt == b"dump" {
                o.scalar.dump_memory += 1;
            } else if opt == b'T' || longopt == b"trace" {
                o.scalar.trace_memory += 1;
            } else if longopt == b"std" {
                o.scalar.load_std = 1;
            } else if longopt == b"no-unhandled-rejection" {
                o.scalar.dump_unhandled_promise_rejection = 0;
            } else if opt == b'q' || longopt == b"quit" {
                o.scalar.empty_run += 1;
            } else if longopt == b"memory-limit" || longopt == b"stack-size" {
                if o.scalar.optind >= argv.len() {
                    return Err(ByteCliError::new(
                        if longopt == b"memory-limit" {
                            b"expecting memory limit".to_vec()
                        } else {
                            b"expecting stack size".to_vec()
                        },
                        1,
                    ));
                }
                let value = get_suffixed_size_bytes(&argv[o.scalar.optind])?;
                o.scalar.optind += 1;
                if longopt == b"memory-limit" {
                    o.scalar.memory_limit = value;
                } else {
                    o.scalar.stack_size = value;
                }
            } else if opt == b's' {
                o.scalar.strip_flags = JS_STRIP_DEBUG;
            } else if longopt == b"strip-source" {
                o.scalar.strip_flags = JS_STRIP_SOURCE;
            } else {
                let mut message = b"qjs: unknown option '".to_vec();
                if opt != 0 {
                    message.extend_from_slice(&[b'-', opt]);
                } else {
                    message.extend_from_slice(b"--");
                    message.extend_from_slice(longopt);
                }
                message.extend_from_slice(b"'\n");
                return Err(ByteCliError {
                    message,
                    help: true,
                    exit: 1,
                });
            }
            longopt = b"";
        }
    }
    Ok(o)
}
