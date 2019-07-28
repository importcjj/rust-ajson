use std::collections::HashMap;


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