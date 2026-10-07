//! Standalone unicode_gen.c tool, separate from the JavaScript engine.
fn main() {
    std::process::exit(quickjs_tools::unicode_gen::run(std::env::args_os()));
}
