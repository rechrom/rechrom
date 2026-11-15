use std::{env, fs, io};

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let input = args.next().expect("input path");
    let output = args.next().expect("output path");
    let decoded = image::open(input).map_err(io::Error::other)?.into_rgba8();
    fs::write(output, decoded.as_raw())
}
