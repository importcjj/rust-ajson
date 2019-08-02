use getter::Getter;
use number::Number;
use std::cmp;
use std::collections::HashMap;
use std::fmt;
use std::str;

#[derive(PartialEq, Clone)]
pub enum Value {
    String(String),
    Number(Number),
    Object(String),
    Array(String),
    Boolean(bool),
    Null,
    NotExist,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::NotExist => write!(f, "<NOT Exist>"),
            Value::String(_) => write!(f, r#""{}""#, self.as_str()),
            _ => write!(f, "{}", self.as_str()),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Value {
    pub fn get(&self, path: &str) -> Value {
        self.get_by_utf8(&path.as_bytes())
    }

    pub fn get_by_utf8(&self, v: &[u8]) -> Value {
        match self {
            Value::Array(s) | Value::Object(s) => Getter::from_str(s).get_by_utf8(v),
            _ => Value::NotExist,
        }
    }
}

impl Value {
    pub fn exists(&self) -> bool {
        *self != Value::NotExist
    }

    pub fn is_string(&self) -> bool {
        match self {
            Value::String(_) => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            Value::Array(_) => true,
            _ => false,
        }
    }

    pub fn is_object(&self) -> bool {
        match self {
            Value::Object(_) => true,
            _ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Value::Boolean(_) => true,
            _ => false,
        }
    }

    pub fn is_null(&self) -> bool {
        match self {
            Value::Null => true,
            _ => false,
        }
    }
}

impl Value {
    pub fn as_str(&self) -> &str {
        match &self {
            Value::String(ref s) => s,
            Value::Number(number) => number.as_str(),
            Value::Boolean(true) => "true",
            Value::Boolean(false) => "false",
            Value::Object(ref s) => s,
            Value::Array(ref s) => s,
            Value::NotExist => "",
            Value::Null => "null",
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Value::Number(number) => number.as_f64(),
            Value::Boolean(true) => 1.0,
            Value::String(s) => Number::from(s.as_bytes()).as_f64(),
            _ => 0.0,
        }
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            Value::Number(number) => number.as_u64(),
            Value::Boolean(true) => 1,
            Value::String(s) => Number::from(s.as_bytes()).as_u64(),
            _ => 0,
        }
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            Value::Number(number) => number.as_i64(),
            Value::Boolean(true) => 1,
            Value::String(ref s) => Number::from(s.as_bytes()).as_i64(),
            _ => 0,
        }
    }

    pub fn as_bool(&self) -> bool {
        match *self {
            Value::Boolean(b) => b,
            _ => false,
        }
    }

    pub fn as_array(&self) -> Vec<Value> {
        match self {
            Value::Array(s) => Getter::from_str(s).as_array(),
            Value::Null | Value::NotExist => vec![],
            _ => vec![self.clone()],
        }
    }

    pub fn as_map(&self) -> HashMap<String, Value> {
        match self {
            Value::Object(s) => Getter::from_str(s).as_map(),
            _ => HashMap::new(),
        }
    }
}

impl cmp::PartialEq<&str> for Value {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl cmp::PartialEq<f64> for Value {
    fn eq(&self, other: &f64) -> bool {
        self.as_f64() == *other
    }
}
