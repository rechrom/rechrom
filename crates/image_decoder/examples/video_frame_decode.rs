use std::hash::{DefaultHasher, Hash, Hasher};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: video_frame_decode <mp4-or-mov>")?;
    let bytes = std::fs::read(&path)?;
    let mime_type = match std::path::Path::new(&path)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some(extension) if extension.eq_ignore_ascii_case("mov") => "video/quicktime",
        _ => "video/mp4",
    };
    let frame = image_decoder::video_frame_decoder::DecodeVideoFirstFrame(&bytes, mime_type)?;
    let software = &frame.software_image;
    if let Some(output) = std::env::args_os().nth(2) {
        image::save_buffer(
            output,
            &software.rgba8,
            software.width,
            software.height,
            image::ColorType::Rgba8,
        )?;
    }
    let mut hasher = DefaultHasher::new();
    software.rgba8.hash(&mut hasher);
    println!(
        "{}x{} orientation={} rgba={} nv12={}+{} hash={:016x}",
        software.width,
        software.height,
        software.orientation.exif(),
        software.rgba8.len(),
        frame.resource.y_plane.len(),
        frame.resource.uv_plane.len(),
        hasher.finish()
    );
    Ok(())
}
