use std::collections::HashMap;
use std::str;
use std::cmp;

#[derive(PartialEq, Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Object(String, Option<HashMap<String, Value>>),
    Array(String, Option<Vec<Value>>),
    Boolean(bool),
    Null,
    NotExist,
}


impl Value {
    pub fn exists(&self) -> bool {
        *self != Value::NotExist
    }

    pub fn to_string(&self) -> String {
        match &self {
            Value::String(ref s) => s.clone(),
            Value::Number(f) => f.to_string(),
            Value::Boolean(true) => "true".to_owned(),
            Value::Boolean(false) => "false".to_owned(),
            Value::Object(ref s, _) => s.clone(),
            Value::Array(ref s, _) => s.clone(),
            Value::NotExist => "".to_owned(),
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

impl cmp::PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        match &self {
            Value::String(ref s) => s == *other,
            Value::Number(f) => &f.to_string() == *other,
            Value::Boolean(true) => "true" == *other,
            Value::Boolean(false) => "false" == *other,
            Value::Object(ref s, _) => s == *other,
            Value::Array(ref s, _) => s == *other,
            Value::NotExist => "" == *other,
            Value::Null => "null" == *other,
        }
    }
}

impl cmp::PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        match *self {
            Value::Number(f) => f == *other,
            Value::Boolean(true) => 1.0 == *other,
            _ => 0.0 == *other,
        }
    }
}