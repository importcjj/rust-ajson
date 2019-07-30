use std::collections::HashMap;
use std::str;
use std::cmp;
use std::fmt;
use getter::Getter;


#[derive(PartialEq, Clone)]
pub enum Value {
    String(String),
    Number(Vec<u8>, f64),
    Object(String),
    Array(String),
    Boolean(bool),
    Null,
    NotExist,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // write!(f, "<G")?;
        // match &self {
        //     Value::String(_) => write!(f, "String")?,
        //     Value::Number(_, _) => write!(f, "Number")?,
        //     Value::Boolean(_) => write!(f, "Bool")?,
        //     Value::Object(_) => write!(f, "Object")?,
        //     Value::Array(_) => write!(f, "Array")?,
        //     Value::NotExist => write!(f, "NotExsit")?,
        //     Value::Null => write!(f, "Null")?
        // };

        // write!(f, " {} >", self.as_str())

        write!(f, "{}", self.as_str())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Value {
    pub fn get(&self, path: &str) -> Value {
        match self {
            Value::Array(s) | Value::Object(s) => Getter::from_str(s).get(path),
            _ => Value::NotExist,
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

    pub fn is_array(&self) -> bool {
        match self {
            Value::Array(_) => true,
            _ => false
        }
    }

    pub fn is_object(&self) -> bool {
        match self {
            Value::Object(_) => true,
            _ => false
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
    pub fn exists(&self) -> bool {
        *self != Value::NotExist
    }

    pub fn as_str(&self) -> &str {
        match &self {
            Value::String(ref s) => s,
            Value::Number(raw, _) => str::from_utf8(raw).unwrap(),
            Value::Boolean(true) => "true",
            Value::Boolean(false) => "false",
            Value::Object(ref s) => s,
            Value::Array(ref s) => s,
            Value::NotExist => "",
            Value::Null => "null",
        }
    }

    pub fn as_f64(&self) -> f64 {
        match *self {
            Value::Number(_, f) => f,
            Value::Boolean(true) => 1.0,
            _ => 0.0
        }
    }

    pub fn as_u64(&self) -> u64 {
        self.as_f64() as u64
    }

    pub fn as_i64(&self) -> i64 {
        self.as_f64() as i64
    }

    pub fn as_bool(&self) -> bool {
        match *self {
            Value::Boolean(b) => b,
            _ => false
        }
    }

    pub fn as_array(&self) -> Vec<Value> {
        match self {
            Value::Array(s) => {
                Getter::from_str(s).as_array()
            }
            Value::NotExist => vec![],
            _ => vec![self.clone()]
        }
    }

    pub fn as_map(&self) -> HashMap<String, Value> {
        match self {
            Value::Object(s) => {
                Getter::from_str(s).as_map()
            }
            _ => HashMap::new()
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