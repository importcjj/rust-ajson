use std::collections::HashMap;
use parser::Parser;
use get_from_reader;
use std::fmt::Debug;
use path::Path;

#[derive(PartialEq, Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Object(String, Option<HashMap<String, Value>>),
    Array(String, Option<Vec<Value>>),
    Boolean(bool),
    Null,
    NotExists,
}


impl Value {
    pub fn get_utf8(&self, path: &[u8]) -> Value {
        let p = Path::new_from_utf8(path);
        self.get_path(&p)
    }

    pub fn get_path(&self, path: &Path) -> Value {
        match &self {
            Value::Object(raw, _) | Value::Array(raw, _) => {
                let mut r = Parser::new(raw.as_bytes());
                get_from_reader(&mut r, path).to_value()
            }
            _ => Value::NotExists
        }
    }

    pub fn get(self, path: &str) -> Value {
        self.get_utf8(path.as_bytes())
    }

    pub fn exists(&self) -> bool {
        *self != Value::NotExists
    }

    pub fn as_str(&self) -> &str {
        match &self {
            Value::String(s) => s,
            _ => "as_str not implementaion!!",
        }
    }

    pub fn number(&self) -> f64 {
        match &self {
            Value::Number(ref f) => *f,
            _ => 0.0
        }
    }
}