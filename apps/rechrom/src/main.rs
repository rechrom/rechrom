// Diagnostic builds alone replace Rust's global allocation entry point. The
// ordinary Release build retains the standard allocator without header overhead.
#[cfg(feature = "allocation_diagnostics")]
#[global_allocator]
static ALLOCATOR: browser_tracing::allocation::PhaseAllocator =
    browser_tracing::allocation::PhaseAllocator;

mod app;
mod begin_frame_source;
mod browser_host;
mod chrome;
mod crash_report;
mod devtools_host;
mod devtools_window;
mod display;
mod engine;
mod headless;
mod input;
mod memory_diagnostics;
mod navigation;
mod options;
mod page_dom;
mod page_error;
mod page_presentation;
mod page_runtime;
mod presentation_runtime;
mod tabs;
mod window_runtime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    memory_diagnostics::start_sampler();
    let Some(options) = options::Options::parse()? else {
        return Ok(());
    };
    app::run(options)
}

mod window_surface;
