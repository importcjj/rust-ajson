use std::str;

use regex::Regex;

pub fn is_match(k1: &str, pattern: &str) -> bool {
    let pattern = &pattern.replace("?", ".").replace("*", ".+?");
    Regex::new(pattern).unwrap().is_match(k1)
}

pub fn is_match_u8(k1: &[u8], pattern: &[u8]) -> bool {
    let key = str::from_utf8(k1).unwrap();
    let pat = str::from_utf8(pattern).unwrap();
    let pat = &pat.replace("?", ".").replace("*", ".+?");
    Regex::new(pat).unwrap().is_match(key)
}
