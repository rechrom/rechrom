// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/html/anchor_element_utils.h:30-39; .cc:197-230.
// UseCounter annotations do not change the retained navigation relation bits.
pub const RELATION_NO_REFERRER: u32 = 0x00001000;
// Source: anchor_element_utils.cc:193-195.
pub fn HasRel(link_relations: u32, relation: u32) -> bool {
    link_relations & relation != 0
}
pub fn ParseRelAttribute(value: &str) -> u32 {
    value
        .split(|c| matches!(c, '\t' | '\n' | '\u{000c}' | '\r' | ' '))
        .fold(0, |bits, token| {
            bits | match token.to_ascii_lowercase().as_str() {
                "noreferrer" => 0x00001000,
                "noopener" => 0x00040000,
                "opener" => 0x00080000,
                "privacy-policy" => 0x00100000,
                "terms-of-service" => 0x00200000,
                _ => 0,
            }
        })
}
