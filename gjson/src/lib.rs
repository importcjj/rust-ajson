extern crate regex;


mod getter;
pub use getter::{get, parse};

mod path;
mod path_parser;
mod util;
mod value;
mod wild;
mod reader;


pub use value::Value;
