extern crate regex;

mod getter;
mod number;
mod path;
pub mod path_v2;
mod path_parser;
mod reader;
mod sub_selector;
mod unescape;
mod util;
mod value;
mod wild;
pub mod group;

pub use getter::Getter;
pub use number::Number;
pub use path::Path;
use std::io;
pub use value::Value;


pub fn get(json: &str, path: &str) -> Option<Value> {
    Getter::from_str(json).get(path)
}

pub fn parse(json: &str) -> Option<Value> {
    let mut getter = Getter::from_utf8(json.as_bytes());
    getter.next_value()
}

pub fn get_from_read<R>(r: R, path: &str) -> Option<Value>
where
    R: io::Read,
{
    let mut getter = Getter::from_read(r);
    getter.get_by_utf8(path.as_bytes())
}

pub fn parse_from_read<R>(r: R) -> Option<Value>
where
    R: io::Read,
{
    let mut getter = Getter::from_read(r);
    getter.next_value()
}
