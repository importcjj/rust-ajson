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
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
    pub fn get(&self, path: &str) -> Option<Value> {
        self.get_by_utf8(&path.as_bytes())
    }

    pub fn get_by_utf8(&self, v: &[u8]) -> Option<Value> {
        match self {
            Value::Array(s) | Value::Object(s) => Getter::from_str(s).get_by_utf8(v),
            _ => None,
        }
    }
}

impl Value {

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
            Value::Null => "null",
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            Value::Number(number) => number.to_f64(),
            Value::Boolean(true) => 1.0,
            Value::String(s) => Number::from(s.as_bytes()).to_f64(),
            _ => 0.0,
        }
    }

    pub fn to_u64(&self) -> u64 {
        match self {
            Value::Number(number) => number.to_u64(),
            Value::Boolean(true) => 1,
            Value::String(s) => Number::from(s.as_bytes()).to_u64(),
            _ => 0,
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            Value::Number(number) => number.to_i64(),
            Value::Boolean(true) => 1,
            Value::String(ref s) => Number::from(s.as_bytes()).to_i64(),
            _ => 0,
        }
    }

    pub fn to_bool(&self) -> bool {
        match *self {
            Value::Boolean(b) => b,
            _ => false,
        }
    }

    pub fn to_vec(&self) -> Vec<Value> {
        match self {
            Value::Array(s) => Getter::from_str(s).to_vec(),
            Value::Null => vec![],
            _ => vec![self.clone()],
        }
    }

    pub fn to_object(&self) -> HashMap<String, Value> {
        match self {
            Value::Object(s) => Getter::from_str(s).to_object(),
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
        self.to_f64() == *other
    }
}
