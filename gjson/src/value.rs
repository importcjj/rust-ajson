use std::collections::HashMap;
use std::str;

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

    pub fn to_string(&self) -> String {
        match &self {
            Value::String(ref s) => s.clone(),
            Value::Number(f) => f.to_string(),
            Value::Boolean(true) => "true".to_owned(),
            Value::Boolean(false) => "false".to_owned(),
            Value::Object(ref s, _) => s.clone(),
            Value::Array(ref s, _) => s.clone(),
            Value::NotExists => "".to_owned(),
            Value::Null => "null".to_owned(),
        }
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