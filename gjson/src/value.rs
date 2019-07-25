use std::collections::HashMap;
use parser::Parser;
use get_from_reader;

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
        match &self {
            Value::Object(raw, _) | Value::Array(raw, _) => {
                let mut r = Parser::new(raw.as_bytes());
                get_from_reader(&mut r, path)
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
            _ => ""
        }
    }

    pub fn number(&self) -> f64 {
        match &self {
            Value::Number(ref f) => *f,
            _ => 0.0
        }
    }
}