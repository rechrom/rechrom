//! DevTools `Page.startScreencast` / `Page.stopScreencast` for the displayless
//! host, corresponding to Chromium's `DevToolsVideoConsumer` /
//! `PageHandler::StartScreencast` (content/browser/devtools/protocol/
//! page_handler.cc). Each completed Display frame is sampled once, after it
//! is published, with its publication time.
//!
//! Rechrom's DevTools transport is request/response HTTP and carries no
//! `Page.screencastFrame` events, so frames are streamed to a file chosen by
//! the client instead: `<path>` receives packed top-down BGR24 frames and
//! `<path>.idx` one line per frame (`index timestamp_us present_sequence
//! width height`). File I/O runs on a writer thread; the Display thread only
//! takes a point-sampled copy.
use std::{
    io::Write,
    sync::{mpsc, Mutex, OnceLock},
    thread::JoinHandle,
    time::Instant,
};

struct Frame {
    timestamp: Instant,
    sequence: u64,
    width: u32,
    height: u32,
    bgr: Vec<u8>,
}

struct Session {
    max_width: u32,
    max_height: u32,
    every_nth_frame: u32,
    seen: u32,
    started: Instant,
    frames: mpsc::Sender<Frame>,
    writer: JoinHandle<Result<u64, String>>,
    path: String,
}

fn session() -> &'static Mutex<Option<Session>> {
    static SESSION: OnceLock<Mutex<Option<Session>>> = OnceLock::new();
    SESSION.get_or_init(|| Mutex::new(None))
}

pub(crate) fn attach() {
    devtools::set_screencast_handler(std::sync::Arc::new(|request| match request {
        Some(options) => start(options).map(|()| serde_json::json!({})),
        None => stop(),
    }));
}

fn start(options: devtools::ScreencastOptions) -> Result<(), String> {
    let mut active = session().lock().unwrap();
    if active.is_some() {
        return Err("screencast already started".into());
    }
    let mut file = std::io::BufWriter::new(
        std::fs::File::create(&options.path).map_err(|error| error.to_string())?,
    );
    let mut index = std::io::BufWriter::new(
        std::fs::File::create(format!("{}.idx", options.path))
            .map_err(|error| error.to_string())?,
    );
    let started = Instant::now();
    let (frames, received) = mpsc::channel::<Frame>();
    let writer = std::thread::Builder::new()
        .name("screencast-writer".into())
        .spawn(move || {
            let mut count = 0u64;
            for frame in received {
                file.write_all(&frame.bgr).map_err(|e| e.to_string())?;
                writeln!(
                    index,
                    "{count} {} {} {} {}",
                    frame
                        .timestamp
                        .saturating_duration_since(started)
                        .as_micros(),
                    frame.sequence,
                    frame.width,
                    frame.height
                )
                .map_err(|e| e.to_string())?;
                count += 1;
            }
            file.flush().map_err(|e| e.to_string())?;
            index.flush().map_err(|e| e.to_string())?;
            Ok(count)
        })
        .map_err(|error| error.to_string())?;
    *active = Some(Session {
        max_width: options.max_width.unwrap_or(u32::MAX).max(1),
        max_height: options.max_height.unwrap_or(u32::MAX).max(1),
        every_nth_frame: options.every_nth_frame.unwrap_or(1).max(1),
        seen: 0,
        started,
        frames,
        writer,
        path: options.path,
    });
    Ok(())
}

fn stop() -> Result<serde_json::Value, String> {
    let Some(active) = session().lock().unwrap().take() else {
        return Err("screencast not started".into());
    };
    let duration = active.started.elapsed();
    drop(active.frames);
    let frames = active
        .writer
        .join()
        .map_err(|_| "screencast writer panicked".to_string())??;
    Ok(serde_json::json!({
        "path": active.path,
        "frames": frames,
        "durationMs": duration.as_secs_f64() * 1000.0,
    }))
}

/// Called by Display after a frame becomes the completed output. Sample the
/// page region of a BGRA target into packed BGR24 within the session's
/// maximum size and queue it for the writer.
pub(crate) fn capture(
    sequence: u64,
    pixels: &[u32],
    row_stride: usize,
    width: u32,
    start_y: u32,
    height: u32,
) {
    let timestamp = Instant::now();
    let mut active = session().lock().unwrap();
    let Some(session) = active.as_mut() else {
        return;
    };
    session.seen += 1;
    if (session.seen - 1) % session.every_nth_frame != 0 || width == 0 || height == 0 {
        return;
    }
    let factor = width
        .div_ceil(session.max_width)
        .max(height.div_ceil(session.max_height))
        .max(1);
    let out_width = width / factor;
    let out_height = height / factor;
    // Point-sample one pixel per output cell. This runs on the Display thread
    // inside the frame cycle, so it must stay far below the frame budget: a
    // full box filter cost ~10 ms per 1280x800 frame on a 2-core host and
    // distorted the very frame timing the recording is meant to measure.
    let mut bgr = vec![0u8; out_width as usize * out_height as usize * 3];
    let step = factor as usize;
    for (y, out_row) in bgr.chunks_exact_mut(out_width as usize * 3).enumerate() {
        let start = (start_y as usize + y * step) * row_stride;
        let row = &pixels[start..start + width as usize];
        for (out, &p) in out_row.chunks_exact_mut(3).zip(row.iter().step_by(step)) {
            out[0] = p as u8;
            out[1] = (p >> 8) as u8;
            out[2] = (p >> 16) as u8;
        }
    }
    let _ = session.frames.send(Frame {
        timestamp,
        sequence,
        width: out_width,
        height: out_height,
        bgr,
    });
}
