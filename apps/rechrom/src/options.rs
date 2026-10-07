use std::{io, path::Path};

pub struct Options {
    pub address: String,
    pub width: u32,
    pub height: u32,
    pub exit_after: Option<std::time::Duration>,
    pub devtools_port: u16,
}

impl Options {
    pub fn parse() -> io::Result<Option<Self>> {
        let mut options = Self {
            address: "about:home".into(),
            width: 1280,
            height: 800,
            exit_after: None,
            devtools_port: 9223,
        };
        let mut address_set = false;
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--version" | "-V" => {
                    println!("Rechrom {}", env!("CARGO_PKG_VERSION"));
                    return Ok(None);
                }
                "--help" | "-h" => {
                    println!("rechrom [URL or file] [--url URL | --input FILE] [--width N] [--height N]\n\
                        Shortcuts: Ctrl/Cmd+L address, Ctrl/Cmd+R reload, Alt+Left/Right history, F12 Performance DevTools.\n\
                        --devtools-port PORT selects the localhost Performance server (default 9223, 0 automatic).\n\
                        --exit-after SECONDS closes the window after a timed smoke run.");
                    return Ok(None);
                }
                "--url" | "--input" => {
                    options.address = args.next().ok_or_else(|| invalid("address required"))?;
                    address_set = true;
                }
                "--width" => options.width = parse_number(args.next(), "width")?,
                "--height" => options.height = parse_number(args.next(), "height")?,
                "--devtools-port" => {
                    options.devtools_port = parse_number(args.next(), "devtools-port")?
                }
                "--exit-after" => {
                    let seconds: f64 = parse_number(args.next(), "exit-after")?;
                    if !seconds.is_finite() || seconds <= 0.0 || seconds > 86400.0 {
                        return Err(invalid("invalid exit-after"));
                    }
                    options.exit_after = Some(std::time::Duration::from_secs_f64(seconds));
                }
                _ if !arg.starts_with('-') && !address_set => {
                    options.address = arg;
                    address_set = true;
                }
                _ => return Err(invalid(format!("unsupported argument: {arg}"))),
            }
        }
        if !(320..=4096).contains(&options.width) || !(200..=4096).contains(&options.height) {
            return Err(invalid("window dimensions must be 320..4096 by 200..4096"));
        }
        options.address = normalize_address(&options.address)?;
        Ok(Some(options))
    }
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn parse_number<T: std::str::FromStr>(value: Option<String>, name: &str) -> io::Result<T> {
    value
        .ok_or_else(|| invalid(format!("{name} needs a value")))?
        .parse()
        .map_err(|_| invalid(format!("invalid {name}")))
}

pub fn normalize_address(input: &str) -> io::Result<String> {
    let input = input.trim();
    if input.is_empty() {
        return Err(invalid("address is empty"));
    }
    if input == "about:home" || input == "about:blank" {
        return Ok(input.into());
    }
    if Path::new(input).exists() {
        let path = std::fs::canonicalize(input)?;
        return url::Url::from_file_path(path)
            .map(String::from)
            .map_err(|_| invalid("invalid file path"));
    }
    let address = if input.contains("://") || input.starts_with("data:") {
        input.to_owned()
    } else {
        format!("https://{input}")
    };
    let url = url::Url::parse(&address).map_err(|error| invalid(error.to_string()))?;
    if !matches!(url.scheme(), "http" | "https" | "file" | "data") {
        return Err(invalid("unsupported URL scheme"));
    }
    Ok(url.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn addresses_keep_queries_and_default_to_https() {
        assert_eq!(
            normalize_address(" baidu.com/s?wd=rust ").unwrap(),
            "https://baidu.com/s?wd=rust"
        );
        assert_eq!(normalize_address("about:blank").unwrap(), "about:blank");
        assert!(normalize_address("javascript://anything").is_err());
    }
}
