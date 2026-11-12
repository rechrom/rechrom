use std::{env, fs, io};

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let input = args.next().expect("input path");
    let output = args.next().expect("output path");
    if let (Some(width), Some(height)) = (args.next(), args.next()) {
        let encoded = fs::read(input)?;
        let pixels = image_decoder::jpeg_scaled_decode::decode_to_n32(
            &encoded,
            width.parse().expect("width"),
            height.parse().expect("height"),
        )?
        .expect("supported JPEG scale");
        return fs::write(output, pixels);
    }
    let decoded = image::open(input).map_err(io::Error::other)?.into_rgba8();
    fs::write(output, decoded.as_raw())
}
