use std::{borrow::Cow, collections::HashMap, fmt, fmt::Formatter, str};

use crate::{number::Number, parser, path::Path, Result};

/// Represents JSON valuue.
#[derive(PartialEq, Eq, Clone)]
pub enum Value<'a> {
    /// Represents a JSON String.
    String(Cow<'a, str>),
    /// Respesents a JSON number.
    Number(Number<'a>),
    /// Respesents a JSON number.
    Usize(usize),
    /// Respesents a JSON object.
    Object(Cow<'a, str>),
    /// Respesents a JSON array.
    Array(Cow<'a, str>),
    /// Respesents a JSON boolean.
    Boolean(bool),
    /// Respesents a JSON null value.
    Null,
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, r#""{}""#, s),
            Value::Number(n) => write!(f, "{}", n.as_str()),
            Value::Usize(n) => write!(f, "{}", n),
            Value::Object(s) | Value::Array(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

impl<'a> Value<'a> {
    /// Get sub value from a JSON array or map.
    /// About path syntax, see [here](index.html#syntax).
    /// For more detail, see [`get`](fn.get.html).
    /// ```
    /// use ajson::{Result, Value};
    /// fn main() -> Result<()> {
    ///     let v = Value::Array("[1,2,3]".into());
    ///     let first_num = v.get("0")?.unwrap();
    ///     assert_eq!(first_num, 1_i64);
    ///     Ok(())
    /// }
    /// ```
    pub fn get(&self, path: &'a str) -> Result<Option<Value>> {
        match self {
            Value::Array(s) | Value::Object(s) => {
                let p = Path::from_slice(path.as_ref())?;
                let (a, _left) = parser::bytes_get(s.as_bytes(), &p)?;
                Ok(a.map(|el| el.to_value()))
            }
            _ => Ok(None),
        }
    }
}

impl<'a> Value<'a> {
    /// Returns true if the `Value` is a JSON string.
    /// ```
    /// let v = ajson::get(r#"{"name":"ajson"}"#, "name").unwrap().unwrap();
    /// assert!(v.is_string());
    /// ```
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Boolean(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

impl<'a> std::fmt::Display for Value<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::String(ref s) => write!(f, "{}", s),
            Value::Number(number) => write!(f, "{}", number.as_str()),
            Value::Boolean(true) => write!(f, "true"),
            Value::Boolean(false) => write!(f, "false"),
            Value::Object(ref s) => write!(f, "{}", s),
            Value::Array(ref s) => write!(f, "{}", s),
            Value::Usize(u) => write!(f, "{}", u),
            Value::Null => write!(f, "null"),
        }
    }
}

impl<'a> Value<'a> {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(number) => Some(number.to_f64()),
            Value::Usize(n) => Some(*n as f64),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Value::Number(number) => Some(number.to_u64()),
            Value::Usize(n) => Some(*n as u64),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Number(number) => Some(number.to_i64()),
            Value::Usize(n) => Some(*n as i64),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Boolean(b) => Some(b),
            _ => None,
        }
    }

    pub fn as_vec(&self) -> Option<Vec<Value>> {
        match self {
            Value::Array(s) => parser::bytes_to_vec(s.as_bytes()).ok(),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<HashMap<&str, Value>> {
        match self {
            Value::Object(s) => parser::bytes_to_map(s.as_bytes()).ok(),
            _ => None,
        }
    }
}

fn eq_f64(value: &Value, other: f64) -> bool {
    value.as_f64().map_or(false, |i| i == other)
}

fn eq_i64(value: &Value, other: i64) -> bool {
    value.as_i64().map_or(false, |i| i == other)
}

fn eq_u64(value: &Value, other: u64) -> bool {
    value.as_u64().map_or(false, |i| i == other)
}

fn eq_bool(value: &Value, other: bool) -> bool {
    value.as_bool().map_or(false, |i| i == other)
}

fn eq_str(value: &Value, other: &str) -> bool {
    value.as_str().map_or(false, |i| i == other)
}

impl<'a> PartialEq<str> for Value<'a> {
    fn eq(&self, other: &str) -> bool {
        eq_str(self, other)
    }
}

impl<'a> PartialEq<&'a str> for Value<'a> {
    fn eq(&self, other: &&str) -> bool {
        eq_str(self, *other)
    }
}

impl<'a> PartialEq<Value<'a>> for str {
    fn eq(&self, other: &Value) -> bool {
        eq_str(other, self)
    }
}

impl<'a> PartialEq<Value<'a>> for &'a str {
    fn eq(&self, other: &Value) -> bool {
        eq_str(other, *self)
    }
}

impl<'a> PartialEq<String> for Value<'a> {
    fn eq(&self, other: &String) -> bool {
        eq_str(self, other.as_str())
    }
}

impl<'a> PartialEq<Value<'a>> for String {
    fn eq(&self, other: &Value) -> bool {
        eq_str(other, self.as_str())
    }
}

macro_rules! partialeq_numeric {
    ($($eq:ident [$($ty:ty)*])*) => {
        $($(
            impl<'a> PartialEq<$ty> for Value<'a> {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(self, *other as _)
                }
            }

            impl<'a> PartialEq<Value<'a>> for $ty {
                fn eq(&self, other: &Value) -> bool {
                    $eq(other, *self as _)
                }
            }

            impl<'a> PartialEq<$ty> for &'a Value<'a> {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(*self, *other as _)
                }
            }

            impl<'a> PartialEq<$ty> for &'a mut Value<'a> {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(*self, *other as _)
                }
            }
        )*)*
    }
}

partialeq_numeric! {
    eq_i64[i8 i16 i32 i64 isize]
    eq_u64[u8 u16 u32 u64 usize]
    eq_f64[f32 f64]
    eq_bool[bool]
}
