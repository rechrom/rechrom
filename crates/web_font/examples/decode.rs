use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    let Some(input) = args.next() else {
        return ExitCode::FAILURE;
    };
    let Some(output) = args.next() else {
        return ExitCode::FAILURE;
    };
    let bytes = fs::read(input).expect("read webfont");
    if let Some(raw_output) = args.next() {
        let raw = if bytes.starts_with(b"wOF2") {
            wuff::decompress_woff2(&bytes).expect("decompress webfont")
        } else if bytes.starts_with(b"wOFF") {
            wuff::decompress_woff1(&bytes).expect("decompress webfont")
        } else {
            bytes.clone()
        };
        fs::write(raw_output, raw).expect("write raw sfnt");
    }
    let decoded = web_font::DecodeWebFont(bytes).expect("decode webfont");
    fs::write(output, decoded).expect("write sfnt");
    ExitCode::SUCCESS
}
