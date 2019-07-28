extern crate regex;


mod getter;
pub use getter::get;
mod path;
mod read;
mod util;
mod value;
mod wild;
mod reader;


pub use value::Value;
