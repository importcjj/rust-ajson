extern crate regex;

mod getter;
mod path;
mod path_parser;
mod util;
mod value;
mod wild;
mod reader;
mod sub_selector;
mod unescape;

use std::io;
pub use value::Value;
pub use getter::Getter;


pub fn get(json: &str, path: &str) -> Value {
    Getter::new_from_utf8(json.as_bytes()).get(path)
}

pub fn parse(json: &str) -> Value {
    let mut getter = Getter::new_from_utf8(json.as_bytes());
    getter.next_value()
}

pub fn get_from_read<R>(r: R, path: &str) -> Value
where
    R: io::Read,
{
    Getter::new_from_read(r).get(path)
}

pub fn parse_from_read<R>(r: R) -> Value 
where
    R: io::Read,
{
    let mut getter = Getter::new_from_read(r);
    getter.next_value()
}
