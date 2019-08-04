extern crate regex;

mod getter;
mod number;
mod path;
mod path_parser;
mod reader;
mod sub_selector;
mod unescape;
mod util;
mod value;
mod wild;

pub use getter::Getter;
pub use number::Number;
use std::io;
pub use value::Value;

pub fn get(json: &str, path: &str) -> Option<Value> {
    Getter::new_from_utf8(json.as_bytes()).get(path)
}

pub fn parse(json: &str) -> Option<Value> {
    let mut getter = Getter::new_from_utf8(json.as_bytes());
    getter.next_value()
}

pub fn get_from_read<R>(r: R, path: &str) -> Option<Value>
where
    R: io::Read,
{
    Getter::new_from_read(r).get(path)
}

pub fn parse_from_read<R>(r: R) -> Option<Value>
where
    R: io::Read,
{
    let mut getter = Getter::new_from_read(r);
    getter.next_value()
}
