#![allow(non_snake_case)]

use std::sync::{Arc, OnceLock};

use font_engine::SharedFontBytes;
use layoutng_assembly::internal::layout_input::{FontFace, FontUnicodeRange, FontVariation};

// Match the host faces used by the C++ demo/browser input boundary. The
// current acceptance page requests system-ui, then -apple-system.
pub fn DemoFonts() -> Vec<FontFace> {
    static FONTS: OnceLock<Vec<FontFace>> = OnceLock::new();
    let profile = std::env::var_os("BROWSER_PROFILE_INPUT").is_some();
    let started = profile.then(std::time::Instant::now);
    let hit = FONTS.get().is_some();
    let fonts = FONTS.get_or_init(LoadDemoFonts).clone();
    if let Some(started) = started {
        eprintln!(
            "browser-font-catalog-profile hit={hit} faces={} ms={:.3}",
            fonts.len(),
            started.elapsed().as_secs_f64() * 1000.0
        );
    }
    fonts
}

fn ReadFont(path: &str) -> std::io::Result<SharedFontBytes> {
    let started = std::env::var_os("BROWSER_PROFILE_INPUT")
        .is_some()
        .then(std::time::Instant::now);
    let result = ReadFontBytes(path);
    if let Some(started) = started {
        eprintln!(
            "browser-font-read-profile path={path:?} bytes={} succeeded={} mapped={} ms={:.3}",
            result.as_ref().map_or(0, |bytes| bytes.len()),
            result.is_ok(),
            result.as_ref().is_ok_and(SharedFontBytes::is_mapped),
            started.elapsed().as_secs_f64() * 1000.0
        );
    }
    result
}

#[cfg(not(unix))]
fn ReadFontBytes(path: &str) -> std::io::Result<SharedFontBytes> {
    std::fs::read(path).map(SharedFontBytes::from)
}

#[cfg(unix)]
fn RetryInterrupted<T>(mut operation: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    loop {
        match operation() {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => return result,
        }
    }
}

#[cfg(unix)]
unsafe fn ReadRaw(
    fd: std::os::fd::BorrowedFd<'_>,
    buffer: *mut u8,
    length: usize,
) -> std::io::Result<usize> {
    use std::os::fd::AsRawFd;
    unsafe extern "C" {
        #[link_name = "read"]
        fn posix_read(fd: std::ffi::c_int, buffer: *mut std::ffi::c_void, count: usize) -> isize;
    }
    RetryInterrupted(|| {
        // SAFETY: The caller supplies writable memory for length bytes and a
        // live borrowed descriptor. Cap count at POSIX's signed return limit.
        let count = unsafe {
            posix_read(
                fd.as_raw_fd(),
                buffer.cast(),
                length.min(isize::MAX as usize),
            )
        };
        if count < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(count as usize)
        }
    })
}

#[cfg(unix)]
fn ReadExactArc(fd: std::os::fd::BorrowedFd<'_>, length: usize) -> std::io::Result<Arc<[u8]>> {
    if length > isize::MAX as usize {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "font size exceeds allocation limit",
        ));
    }
    let mut bytes = Arc::<[u8]>::new_uninit_slice(length);
    // The allocation is uniquely owned until every byte has been written.
    // MaybeUninit remains its element type on every partial/error path, so
    // dropping it never forms a safe byte slice over unwritten memory.
    let memory = Arc::get_mut(&mut bytes).expect("new font buffer has one owner");
    let mut written = 0;
    while written < length {
        // SAFETY: memory is the unique allocation of length uninitialized bytes.
        // POSIX read accepts raw writable memory and initializes precisely its
        // nonnegative returned count. No &[u8]/&mut [u8] is formed here.
        let count = unsafe {
            ReadRaw(
                fd,
                memory.as_mut_ptr().cast::<u8>().add(written),
                length - written,
            )
        }?;
        if count == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "font ended before its recorded size",
            ));
        }
        written += count;
    }
    // SAFETY: Each successful read initialized the next count bytes, and the
    // loop only finishes after exactly length bytes were initialized.
    Ok(unsafe { bytes.assume_init() })
}

#[cfg(unix)]
fn ReadFontBytes(path: &str) -> std::io::Result<SharedFontBytes> {
    let file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        // Preserve read-to-end semantics for descriptors with no regular-file
        // size (system font paths normally all use the direct allocation).
        return std::fs::read(path).map(SharedFontBytes::from);
    }
    let length = usize::try_from(metadata.len()).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "font size exceeds address space",
        )
    })?;
    // macOS system font files live on the immutable system volume. Mapping
    // arbitrary mutable font paths would violate SharedFontBytes' contract.
    #[cfg(target_os = "macos")]
    if path.starts_with("/System/Library/") {
        return MapOrReadFont(&file, length, |file| {
            // SAFETY: These trusted system fonts remain immutable for the
            // lifetime of every catalog, layout snapshot and native font handle.
            unsafe { SharedFontBytes::map_readonly(file) }
        });
    }
    ReadRegularFileArc(&file, length).map(SharedFontBytes::from)
}

#[cfg(unix)]
fn MapOrReadFont(
    file: &std::fs::File,
    length: usize,
    map: impl FnOnce(&std::fs::File) -> std::io::Result<SharedFontBytes>,
) -> std::io::Result<SharedFontBytes> {
    match map(file) {
        Ok(bytes) => Ok(bytes),
        // mmap does not change the file offset; read the full file on failure.
        Err(_) => ReadRegularFileArc(file, length).map(SharedFontBytes::from),
    }
}

#[cfg(unix)]
fn ReadRegularFileArc(file: &std::fs::File, length: usize) -> std::io::Result<Arc<[u8]>> {
    use std::os::fd::AsFd;
    let bytes = ReadExactArc(file.as_fd(), length)?;
    let mut trailing = [0u8; 1];
    // SAFETY: trailing is initialized, writable memory for one byte.
    if unsafe { ReadRaw(file.as_fd(), trailing.as_mut_ptr(), trailing.len()) }? != 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "font grew while it was read",
        ));
    }
    Ok(bytes)
}

fn LoadDemoFonts() -> Vec<FontFace> {
    LoadDemoFontsWithReader(ReadFont)
}

fn LoadDemoFontsWithReader(
    ReadFont: impl Fn(&str) -> std::io::Result<SharedFontBytes>,
) -> Vec<FontFace> {
    let sfns: SharedFontBytes = ReadFont("/System/Library/Fonts/SFNS.ttf")
        .unwrap_or_default()
        .into();
    let ping_fang: SharedFontBytes = ReadFont(
        "/System/Library/PrivateFrameworks/FontServices.framework/Versions/A/Resources/Reserved/PingFangUI.ttc",
    ).unwrap_or_default().into();
    const WGHT: u32 = u32::from_be_bytes(*b"wght");
    let cjk_ranges = || {
        vec![
            FontUnicodeRange {
                start: 0x3000,
                end: 0x303f,
            },
            FontUnicodeRange {
                start: 0x3400,
                end: 0x9fff,
            },
            FontUnicodeRange {
                start: 0xf900,
                end: 0xfaff,
            },
            FontUnicodeRange {
                start: 0xff00,
                end: 0xffef,
            },
        ]
    };
    let mut faces = Vec::new();
    if let Ok(arial) = ReadFont("/System/Library/Fonts/Supplemental/Arial.ttf") {
        let arial: SharedFontBytes = arial.into();
        for (family, metrics_family) in [("demo", ""), ("Arial", "Arial")] {
            faces.push(FontFace {
                family: family.into(),
                native_family: "Arial".into(),
                metrics_family: metrics_family.into(),
                bytes: arial.clone(),
                ..FontFace::default()
            });
        }
    }
    if let Ok(bytes) = ReadFont("/System/Library/Fonts/Supplemental/Tamil Sangam MN.ttc") {
        faces.push(FontFace {
            family: "tamil-fallback".into(),
            native_family: "Tamil Sangam MN".into(),
            metrics_family: "Tamil Sangam MN".into(),
            bytes: bytes.into(),
            unicode_ranges: vec![FontUnicodeRange {
                start: 0x0b80,
                end: 0x0bff,
            }],
            ..FontFace::default()
        });
    }
    if let Ok(emoji) = ReadFont("/System/Library/Fonts/Apple Color Emoji.ttc") {
        faces.push(FontFace {
            family: "emoji-fallback".into(),
            native_family: "Apple Color Emoji".into(),
            metrics_family: "Apple Color Emoji".into(),
            bytes: emoji.into(),
            unicode_ranges: vec![
                FontUnicodeRange {
                    start: 0x2300,
                    end: 0x27ff,
                },
                FontUnicodeRange {
                    start: 0x1f000,
                    end: 0x1faff,
                },
            ],
            ..FontFace::default()
        });
    }
    if let Ok(bytes) = ReadFont("/System/Library/Fonts/Supplemental/STIXTwoMath.otf") {
        faces.push(FontFace {
            family: "math".into(),
            bytes: bytes.into(),
            ..FontFace::default()
        });
    }
    if !ping_fang.is_empty() {
        for (weight, axis_weight) in [(400.0, 400.0), (600.0, 600.0), (700.0, 600.0)] {
            faces.push(FontFace {
                family: "cjk-fallback".into(),
                native_family: "PingFang SC".into(),
                metrics_family: "PingFang SC".into(),
                weight,
                face_index: 16,
                bytes: ping_fang.clone(),
                variations: vec![FontVariation {
                    tag: WGHT,
                    value: axis_weight,
                }],
                unicode_ranges: cjk_ranges(),
                ..FontFace::default()
            });
        }
        for (weight, axis_weight) in [
            (400.0, 400.0),
            (500.0, 500.0),
            (600.0, 600.0),
            (700.0, 600.0),
        ] {
            faces.push(FontFace {
                family: "PingFang SC".into(),
                native_family: "PingFang SC".into(),
                metrics_family: "PingFang SC".into(),
                weight,
                face_index: 16,
                bytes: ping_fang.clone(),
                variations: vec![FontVariation {
                    tag: WGHT,
                    value: axis_weight,
                }],
                ..FontFace::default()
            });
        }
        if !sfns.is_empty() {
            for weight in [400.0, 500.0, 600.0, 700.0] {
                faces.push(FontFace {
                    family: "PingFang SC".into(),
                    native_family: ".SF NS".into(),
                    metrics_family: ".SF NS".into(),
                    weight,
                    bytes: sfns.clone(),
                    variations: vec![FontVariation {
                        tag: WGHT,
                        value: weight as f32,
                    }],
                    unicode_ranges: vec![FontUnicodeRange {
                        start: 0,
                        end: 0x2fff,
                    }],
                    ..FontFace::default()
                });
            }
        }
    }
    for (path, weight, range) in [
        (
            "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
            400.0,
            FontUnicodeRange {
                start: 0x3040,
                end: 0x30ff,
            },
        ),
        (
            "/System/Library/Fonts/ヒラギノ角ゴシック W6.ttc",
            700.0,
            FontUnicodeRange {
                start: 0x3040,
                end: 0x30ff,
            },
        ),
    ] {
        if let Ok(bytes) = ReadFont(path) {
            faces.push(FontFace {
                family: "cjk-fallback".into(),
                native_family: "Hiragino Sans".into(),
                weight,
                bytes: bytes.into(),
                unicode_ranges: vec![range],
                ..FontFace::default()
            });
        }
    }
    if let Ok(bytes) = ReadFont("/System/Library/Fonts/Helvetica.ttc") {
        faces.push(FontFace {
            family: "sans-serif".into(),
            native_family: "Helvetica".into(),
            metrics_family: "Helvetica".into(),
            bytes: bytes.into(),
            unicode_ranges: vec![FontUnicodeRange {
                start: 0,
                end: 0x2fff,
            }],
            ..FontFace::default()
        });
    }
    if !sfns.is_empty() {
        for family in ["-apple-system", "system-ui"] {
            for weight in [400.0, 500.0, 600.0, 700.0] {
                faces.push(FontFace {
                    family: family.into(),
                    native_family: ".SF NS".into(),
                    metrics_family: ".SF NS".into(),
                    weight,
                    bytes: sfns.clone(),
                    variations: vec![FontVariation {
                        tag: WGHT,
                        value: weight as f32,
                    }],
                    ..FontFace::default()
                });
            }
        }
    }
    if let Ok(bytes) = ReadFont("/System/Library/Fonts/Times.ttc") {
        faces.push(FontFace {
            family: "serif".into(),
            native_family: "Times".into(),
            metrics_family: "Times".into(),
            bytes: bytes.into(),
            ..FontFace::default()
        });
    }
    if faces.is_empty() {
        faces.push(FontFace {
            family: "sans-serif".into(),
            bytes: include_bytes!(
                "../../../../src/third_party/skia/resources/fonts/Roboto-Regular.ttf"
            )
            .as_slice()
            .into(),
            ..FontFace::default()
        });
    }
    faces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapped_catalog_preserves_every_face_and_all_owned_reference_bytes() {
        let actual = LoadDemoFonts();
        let reference =
            LoadDemoFontsWithReader(|path| std::fs::read(path).map(SharedFontBytes::from));
        assert_eq!(actual.len(), reference.len());
        #[cfg(target_os = "macos")]
        assert_eq!(
            actual.len(),
            28,
            "all host faces must remain in the catalog"
        );
        for (a, b) in actual.iter().zip(&reference) {
            // FontFace equality includes every descriptor field and the entire
            // file contents. Separate owners force the content comparison.
            assert!(!SharedFontBytes::ptr_eq(&a.bytes, &b.bytes));
            assert!(a == b, "font descriptor/content differs: {}", a.family);
            assert_eq!(a.bytes.len(), b.bytes.len());
        }
        println!(
            "font descriptors and complete bytes verified: faces={}",
            actual.len()
        );
    }

    #[cfg(unix)]
    #[test]
    fn failed_mapping_reads_the_complete_file_and_preserves_read_errors() {
        use std::io::{Seek, Write};
        let path = std::env::temp_dir().join(format!(
            "browser-font-map-fallback-{}.bin",
            std::process::id()
        ));
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        std::fs::remove_file(path).unwrap();
        file.write_all(b"entire font file").unwrap();
        file.rewind().unwrap();
        let actual = MapOrReadFont(&file, 16, |_| {
            Err(std::io::ErrorKind::PermissionDenied.into())
        })
        .unwrap();
        assert_eq!(actual.as_ref(), b"entire font file");
        assert!(!actual.is_mapped());
        file.rewind().unwrap();
        assert_eq!(
            MapOrReadFont(&file, 17, |_| Err(std::io::ErrorKind::Other.into()))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::UnexpectedEof
        );
        file.set_len(0).unwrap();
        file.rewind().unwrap();
        let empty =
            MapOrReadFont(&file, 0, |_| Err(std::io::ErrorKind::Unsupported.into())).unwrap();
        assert!(empty.is_empty());
        assert!(!empty.is_mapped());
    }

    #[test]
    fn font_aliases_and_constraint_snapshots_share_font_files() {
        let space = crate::CreateBrowserConstraints(1280, 900);
        let snapshot = space.clone();
        let another_page = crate::CreateBrowserConstraints(1024, 768);
        for ((face, copy), other) in space
            .fonts
            .iter()
            .zip(&snapshot.fonts)
            .zip(&another_page.fonts)
        {
            assert!(SharedFontBytes::ptr_eq(&face.bytes, &copy.bytes));
            assert!(SharedFontBytes::ptr_eq(&face.bytes, &other.bytes));
        }
        for (i, a) in space.fonts.iter().enumerate() {
            for b in &space.fonts[i + 1..] {
                if a.bytes == b.bytes {
                    assert!(
                        SharedFontBytes::ptr_eq(&a.bytes, &b.bytes),
                        "font aliases copied {}",
                        a.family
                    );
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn direct_font_reader_handles_partial_reads_and_short_eof() {
        use std::{
            io::Write,
            os::{fd::AsFd, unix::net::UnixStream},
        };
        let (reader, mut writer) = UnixStream::pair().unwrap();
        let expected = b"partial font bytes";
        let producer = std::thread::spawn(move || {
            for chunk in expected.chunks(3) {
                writer.write_all(chunk).unwrap();
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });
        let actual = ReadExactArc(reader.as_fd(), expected.len()).unwrap();
        producer.join().unwrap();
        assert_eq!(&*actual, expected);
        assert_eq!(Arc::strong_count(&actual), 1);
        let error = ReadExactArc(reader.as_fd(), 1).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
        assert!(ReadExactArc(reader.as_fd(), 0).unwrap().is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn font_read_retries_interrupted_operations_and_preserves_other_errors() {
        let mut calls = 0;
        let result = RetryInterrupted(|| {
            calls += 1;
            if calls < 3 {
                Err(std::io::Error::from(std::io::ErrorKind::Interrupted))
            } else {
                Ok(7)
            }
        });
        assert_eq!(result.unwrap(), 7);
        assert_eq!(calls, 3);
        let error = RetryInterrupted::<usize>(|| {
            Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied))
        })
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    }

    #[cfg(unix)]
    #[test]
    fn direct_font_reader_rejects_short_grown_and_unreadable_files() {
        use std::{
            io::{Seek, Write},
            os::fd::AsFd,
        };
        let path =
            std::env::temp_dir().join(format!("browser-font-reader-{}.bin", std::process::id()));
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        // Unlink while keeping the descriptor live; every error/panic path
        // closes the file, and no test data remains on disk.
        std::fs::remove_file(&path).unwrap();
        file.write_all(b"font").unwrap();
        file.rewind().unwrap();
        assert_eq!(&*ReadRegularFileArc(&file, 4).unwrap(), b"font");
        file.rewind().unwrap();
        assert_eq!(
            ReadRegularFileArc(&file, 5).unwrap_err().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
        file.rewind().unwrap();
        assert_eq!(
            ReadRegularFileArc(&file, 3).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
        let write_only = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/null")
            .unwrap();
        assert!(ReadExactArc(write_only.as_fd(), 1).is_err());
        assert_eq!(
            ReadExactArc(write_only.as_fd(), usize::MAX)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn direct_font_reader_matches_all_available_host_file_bytes() {
        use std::io::Read;
        let paths = [
            "/System/Library/Fonts/SFNS.ttf",
            "/System/Library/PrivateFrameworks/FontServices.framework/Versions/A/Resources/Reserved/PingFangUI.ttc",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
            "/System/Library/Fonts/Supplemental/Tamil Sangam MN.ttc",
            "/System/Library/Fonts/Apple Color Emoji.ttc",
            "/System/Library/Fonts/Supplemental/STIXTwoMath.otf",
            "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc",
            "/System/Library/Fonts/ヒラギノ角ゴシック W6.ttc",
            "/System/Library/Fonts/Helvetica.ttc",
            "/System/Library/Fonts/Times.ttc",
        ];
        let mut checked = 0;
        let mut total_bytes = 0u64;
        for path in paths {
            let Ok(mut reference) = std::fs::File::open(path) else {
                continue;
            };
            let actual = ReadFont(path).unwrap();
            let mut buffer = [0u8; 65536];
            let mut offset = 0;
            loop {
                let count = reference.read(&mut buffer).unwrap();
                if count == 0 {
                    break;
                }
                assert_eq!(
                    &actual[offset..offset + count],
                    &buffer[..count],
                    "{path} at {offset}"
                );
                offset += count;
            }
            assert_eq!(actual.len(), offset, "{path}");
            checked += 1;
            total_bytes += offset as u64;
        }
        println!("font bytes verified: files={checked} bytes={total_bytes}");
        #[cfg(target_os = "macos")]
        assert!(checked > 0, "the host font corpus must be read");
        #[cfg(not(target_os = "macos"))]
        let _ = checked;
    }
}
