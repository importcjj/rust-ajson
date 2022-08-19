use path_parser;
use std::fmt;
use sub_selector::SubSelector;

use crate::element::Element;
use crate::Result;
use unescape::unescape;
use util;
use value::Value;

#[cfg(feature = "wild")]
use wild;

static DEFAULT_NONE_QUERY: Query = Query {
    on: false,
    path: &[],
    key: None,
    op: None,
    value: None,
    all: false,
};

static DEFAULT_NONE_PATH: Path = Path {
    ok: false,
    part: &[],
    next: None,
    more: false,
    wild: false,
    arrch: false,

    query: None,
    selectors: None,
    arrsel: false,
};

pub struct Path<'a> {
    pub ok: bool,
    pub part: &'a [u8],
    pub next: Option<Box<Path<'a>>>,
    pub query: Option<Query<'a>>,

    pub selectors: Option<Vec<SubSelector<'a>>>,
    pub arrsel: bool,
    pub more: bool,
    pub wild: bool,
    pub arrch: bool,
}

impl<'a> fmt::Debug for Path<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Path")?;
        write!(f, " ok={}", self.ok)?;
        write!(
            f,
            " part=`{:?}`",
            String::from_utf8_lossy(self.part).to_string()
        )?;

        write!(f, " more={}", self.more)?;
        write!(f, " wild={}", self.wild)?;
        write!(f, " arrch={}", self.arrch)?;
        if self.next.is_some() {
            write!(f, " next=`{:?}`", self.next.as_ref().unwrap())?;
        }
        if self.selectors.is_some() {
            for sel in self.borrow_selectors() {
                write!(f, "\n\tselector {:?}", sel)?;
            }
        }
        if self.has_query() {
            write!(f, " query={:?}", self.query)?;
        }
        write!(f, ">")
    }
}

impl<'a> Path<'a> {
    pub fn empty() -> Path<'a> {
        Path {
            ok: false,
            part: &[],
            next: None,
            more: false,
            wild: false,
            arrch: false,
            query: None,
            selectors: None,
            arrsel: false,
        }
    }

    pub fn parse(v: &'a [u8]) -> Result<Path<'a>> {
        path_parser::parse_path(v)
    }

    pub fn is_match(&self, key: &[u8]) -> bool {
        let optional_key = if key.contains(&b'\\') {
            Some(unescape(key))
        } else {
            None
        };
        let key = optional_key.as_ref().map_or(key, |v| v.as_bytes());
        if self.wild {
            #[cfg(feature = "wild")]
            return wild::is_match_u8(key, self.part);
            false
        } else {
            util::equal_escape_u8(key, self.part)
        }
    }

    pub fn set_part(&mut self, v: &'a [u8]) {
        self.part = v;
    }

    pub fn set_more(&mut self, b: bool) {
        self.more = b;
    }

    pub fn set_next(&mut self, next: Path<'a>) {
        self.next = Some(Box::new(next));
    }

    pub fn borrow_next(&self) -> &Path<'a> {
        match self.next {
            Some(_) => self.next.as_ref().unwrap(),
            None => &DEFAULT_NONE_PATH,
        }
    }

    #[cfg(feature = "wild")]
    pub fn set_wild(&mut self, b: bool) {
        self.wild = b;
    }

    pub fn set_ok(&mut self, b: bool) {
        self.ok = b;
    }

    pub fn set_arrch(&mut self, b: bool) {
        self.arrch = b;
    }

    pub fn set_q(&mut self, q: Query<'a>) {
        self.query = Some(q);
    }

    pub fn has_query(&self) -> bool {
        self.query.is_some()
    }

    pub fn borrow_query(&self) -> &Query<'a> {
        match self.query {
            Some(_) => self.query.as_ref().unwrap(),
            None => &DEFAULT_NONE_QUERY,
        }
    }

    pub fn set_selectors(&mut self, selectors: Vec<SubSelector<'a>>) {
        self.selectors = Some(selectors);
    }

    pub fn set_arrsel(&mut self, b: bool) {
        self.arrsel = b;
    }

    pub fn has_selectors(&self) -> bool {
        self.selectors.is_some()
    }

    pub fn borrow_selectors(&self) -> &[SubSelector<'a>] {
        match self.selectors {
            Some(_) => self.selectors.as_ref().unwrap(),
            None => &[],
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum QueryValue<'a> {
    String(&'a str),
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
    pub on: bool,
    pub path: &'a [u8],
    pub key: Option<Box<Path<'a>>>,
    pub op: Option<String>,
    pub value: Option<QueryValue<'a>>,
    pub all: bool,
}

impl<'a> fmt::Debug for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Query")?;
        write!(f, " on={}", self.on)?;
        write!(f, " all={}", self.all)?;
        if !self.path.is_empty() {
            write!(f, " path=`{}`", String::from_utf8_lossy(self.path))?;
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
            on: false,
            path: &[],
            key: None,
            op: None,
            value: None,
            all: false,
        }
    }

    pub fn has_path(&self) -> bool {
        !self.path.is_empty()
    }

    pub fn get_path(&self) -> Result<Path> {
        match self.has_path() {
            true => path_parser::parse_path(self.path),
            false => Ok(Path::empty()),
        }
    }

    #[allow(dead_code)]
    pub fn has_key(&self) -> bool {
        self.key.is_some()
    }

    #[allow(dead_code)]
    pub fn get_key(&self) -> &Path {
        match self.key {
            Some(_) => self.key.as_ref().unwrap(),
            None => &DEFAULT_NONE_PATH,
        }
    }

    pub fn set_path(&mut self, v: &'a [u8]) {
        self.path = v;
    }

    pub fn set_op(&mut self, op: String) {
        self.op = Some(op);
    }

    pub fn set_val(&mut self, val: QueryValue<'a>) {
        self.value = Some(val);
    }

    pub fn set_all(&mut self, all: bool) {
        self.all = all;
    }

    #[allow(dead_code)]
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
                Value::String(ref s) => match op.as_str() {
                    "==" => s.as_ref() == q,
                    "=" => s == q,
                    "!=" => s != q,
                    ">" => s.as_ref() > q,
                    ">=" => s.as_ref() >= q,
                    "<" => s.as_ref() < q,
                    "<=" => s.as_ref() <= q,
                    #[cfg(feature = "wild")]
                    "%" => wild::is_match(s, q),
                    #[cfg(feature = "wild")]
                    "!%" => !wild::is_match(s, q),
                    _ => false,
                },
                _ => false,
            },

            QueryValue::F64(q) => match v {
                Value::Number(n) => match op.as_str() {
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
                Value::Boolean(b) => match op.as_str() {
                    "=" => b == q,
                    "==" => b == q,
                    "!=" => b != q,
                    _ => false,
                },
                _ => false,
            },

            QueryValue::Null => match op.as_str() {
                "=" => v == Value::Null,
                "==" => v == Value::Null,
                "!=" => v != Value::Null,
                _ => false,
            },
            _ => false,
        }
    }
}
