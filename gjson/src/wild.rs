use regex::Regex;

pub fn is_match(k1: &str, pattern: &str) -> bool {
    let pattern = &pattern.replace("?", ".").replace("*", ".+?");
    Regex::new(pattern).unwrap().is_match(k1)
}