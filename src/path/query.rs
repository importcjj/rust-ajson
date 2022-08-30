use std::fmt;

use super::{parser, Path, DEFAULT_NONE_PATH};
#[cfg(feature = "wild")]
use crate::wild;
use crate::{element::Element, Result, Value};

pub const DEFAULT_NONE_QUERY: Query = Query {
    on:    false,
    path:  &[],
    key:   None,
    op:    None,
    value: None,
    all:   false,
};

#[derive(Debug, PartialEq)]
pub enum QueryValue<'a> {
    String(&'a [u8]),
    F64(f64),
    Boolean(bool),
    Null,
    NotExist,
}

impl<'a> QueryValue<'a> {
    pub fn exists(&self) -> bool {
        *self != QueryValue::NotExist
    }
}

pub struct Query<'a> {
    pub on:    bool,
    pub path:  &'a [u8],
    pub key:   Option<Box<Path<'a>>>,
    pub op:    Option<&'a str>,
    pub value: Option<QueryValue<'a>>,
    pub all:   bool,
}

impl<'a> fmt::Debug for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Query")?;
        write!(f, " on={}", self.on)?;
        write!(f, " all={}", self.all)?;
        if !self.path.is_empty() {
            write!(f, " path=`{}`", unsafe {
                std::str::from_utf8_unchecked(self.path)
            })?;
        }
        if self.key.is_some() {
            write!(f, " key=`{:?}`", self.key.as_ref().unwrap())?;
        }
        if self.op.is_some() {
            write!(f, " op=`{}`", self.op.as_ref().unwrap())?;
        }
        if self.value.is_some() {
            write!(f, " value=`{:?}`", self.value.as_ref().unwrap())?;
        }
        write!(f, ">")
    }
}

impl<'a> Query<'a> {
    pub fn empty() -> Query<'a> {
        Query {
            on:    false,
            path:  &[],
            key:   None,
            op:    None,
            value: None,
            all:   false,
        }
    }

    pub fn has_path(&self) -> bool {
        !self.path.is_empty()
    }

    pub fn get_path(&self) -> Result<Path> {
        match self.has_path() {
            true => parser::parse(self.path),
            false => Ok(Path::default()),
        }
    }

    pub fn has_key(&self) -> bool {
        self.key.is_some()
    }

    pub fn get_key(&self) -> &Path {
        match self.key {
            Some(_) => self.key.as_ref().unwrap(),
            None => &DEFAULT_NONE_PATH,
        }
    }

    pub fn set_path(&mut self, v: &'a [u8]) {
        self.path = v;
    }

    pub fn set_op(&mut self, op: &'a str) {
        self.op = Some(op);
    }

    pub fn set_val(&mut self, val: QueryValue<'a>) {
        self.value = Some(val);
    }

    pub fn set_all(&mut self, all: bool) {
        self.all = all;
    }

    pub fn set_key(&mut self, key: Path<'a>) {
        if key.ok {
            self.key = Some(Box::new(key));
        }
    }

    pub fn set_on(&mut self, on: bool) {
        self.on = on;
    }

    pub fn match_element(&self, element: &Element) -> bool {
        if self.value.is_none() {
            return true;
        }

        let op = match &self.op {
            Some(ref s) => s,
            None => return true,
        };

        let target = self.value.as_ref().unwrap();
        let v = element.to_value();

        match *target {
            QueryValue::String(q) => match v {
                Value::String(ref s) => match *op {
                    "==" => s.as_bytes() == q,
                    "=" => s.as_bytes() == q,
                    "!=" => s.as_bytes() != q,
                    ">" => s.as_bytes() > q,
                    ">=" => s.as_bytes() >= q,
                    "<" => s.as_bytes() < q,
                    "<=" => s.as_bytes() <= q,
                    #[cfg(feature = "wild")]
                    "%" => wild::is_match_u8(s.as_bytes(), q),
                    #[cfg(feature = "wild")]
                    "!%" => !wild::is_match_u8(s.as_bytes(), q),
                    _ => false,
                },
                _ => false,
            },

            QueryValue::F64(q) => match v {
                Value::Number(n) => match *op {
                    "=" => (n.to_f64() - q).abs() < f64::EPSILON,
                    "==" => (n.to_f64() - q).abs() < f64::EPSILON,
                    "!=" => (n.to_f64() - q).abs() > f64::EPSILON,
                    "<" => n.to_f64() < q,
                    "<=" => n.to_f64() <= q,
                    ">" => n.to_f64() > q,
                    ">=" => n.to_f64() >= q,
                    _ => false,
                },
                _ => false,
            },

            QueryValue::Boolean(q) => match v {
                Value::Boolean(b) => match *op {
                    "=" => b == q,
                    "==" => b == q,
                    "!=" => b != q,
                    _ => false,
                },
                _ => false,
            },

            QueryValue::Null => match *op {
                "=" => v == Value::Null,
                "==" => v == Value::Null,
                "!=" => v != Value::Null,
                _ => false,
            },
            _ => false,
        }
    }
}
