mod app;
mod begin_frame_source;
mod chrome;
mod compositor;
mod devtools_window;
mod display;
mod engine;
mod input;
mod navigation;
mod options;
mod presentation_runtime;
mod tabs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(options) = options::Options::parse()? else {
        return Ok(());
    };
    app::run(options)
}

mod window_surface;
