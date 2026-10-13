//! Profile the production stylesheet parser using a captured, fixed input.
//! Usage: cargo run --release -p style --example css_parse_profile -- INPUT [RUNS]
use std::{
    hash::{Hash, Hasher},
    time::Instant,
};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("stylesheet input path");
    let runs: usize = args
        .next()
        .map(|n| n.parse().expect("run count"))
        .unwrap_or(30);
    assert!(runs > 0);
    let css = std::fs::read_to_string(path).expect("UTF-8 stylesheet");
    let mut times = Vec::with_capacity(runs);
    let mut checksum = None;
    let mut rules = 0;
    for _ in 0..runs + 1 {
        let start = Instant::now();
        let sheet = style::ParseCSSWithBase(&css, "https://www.baidu.com/");
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        // Check the complete public projection, not just the accepted rule count.
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        format!(
            "{:?}",
            (
                &sheet.rules,
                &sheet.font_faces,
                &sheet.keyframes,
                &sheet.scopes,
                &sheet.property_rules,
                &sheet.layer_order
            )
        )
        .hash(&mut hash);
        let value = hash.finish();
        if let Some(expected) = checksum {
            assert_eq!(
                expected, value,
                "parser result changed across repeated input"
            );
            times.push(elapsed);
        } else {
            checksum = Some(value);
        }
        rules = sheet.rules.len();
        std::hint::black_box(sheet);
    }
    times.sort_by(f64::total_cmp);
    println!("{{\"bytes\":{},\"runs\":{},\"rules\":{},\"projection_hash\":\"{:016x}\",\"median_ms\":{},\"min_ms\":{},\"max_ms\":{}}}", css.len(), runs, rules, checksum.unwrap(), times[times.len()/2], times[0], times[times.len()-1]);
}
