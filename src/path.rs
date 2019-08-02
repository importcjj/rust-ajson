use path_parser;
use std::fmt;
use sub_selector::SubSelector;

use util;
use value::Value;
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

    pub fn new_from_utf8(v: &'a [u8]) -> Path<'a> {
        path_parser::new_path_from_utf8(v)
    }

    pub fn is_match(&self, key: &[u8]) -> bool {
        let eq = if self.wild {
            wild::is_match_u8(key, self.part)
        } else {
            util::equal_escape_u8(key, self.part)
        };

        eq
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

    pub fn borrow_next(&self) -> &Path {
        match self.next {
            Some(_) => self.next.as_ref().unwrap(),
            None => &DEFAULT_NONE_PATH,
        }
    }

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

    pub fn borrow_query(&self) -> &Query {
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

    pub fn borrow_selectors(&self) -> &[SubSelector] {
        match self.selectors {
            Some(_) => self.selectors.as_ref().unwrap(),
            None => &[],
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum QueryValue {
    String(String),
    F64(f64),
    Boolean(bool),
    Null,
    NotExsit,
}

impl QueryValue {
    pub fn exists(&self) -> bool {
        *self != QueryValue::NotExsit
    }
}

pub struct Query<'a> {
    pub on: bool,
    pub path: &'a [u8],
    pub key: Option<Box<Path<'a>>>,
    pub op: Option<String>,
    pub value: Option<QueryValue>,
    pub all: bool,
}

impl<'a> fmt::Debug for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Query")?;
        write!(f, " on={}", self.on)?;
        write!(f, " all={}", self.all)?;
        if self.path.len() > 0 {
            write!(
                f,
                " path=`{}`",
                String::from_utf8_lossy(self.path).to_string()
            )?;
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
        self.path.len() > 0
    }

    pub fn get_path(&self) -> Path {
        match self.has_path() {
            true => path_parser::new_path_from_utf8(self.path),
            false => Path::empty(),
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

    pub fn set_val(&mut self, val: QueryValue) {
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

    pub fn is_match(&self, v: &Value) -> bool {
        // println!("match value {:?} {:?}",self, v);
        if !v.exists() {
            return false;
        }

        if self.value.is_none() {
            return true;
        }

        let op = match &self.op {
            Some(ref s) => s,
            None => return true,
        };

        let target = self.value.as_ref().unwrap();

        match target {
            QueryValue::String(q) => match v {
                Value::String(s) => match op.as_str() {
                    "==" => s == q,
                    "=" => s == q,
                    "!=" => s != q,
                    ">" => s > q,
                    ">=" => s >= q,
                    "<" => s < q,
                    "<=" => s <= q,
                    "%" => wild::is_match(s, q),
                    "!%" => !wild::is_match(s, q),
                    _ => false,
                },
                _ => false,
            },

            QueryValue::F64(q) => match v {
                Value::Number(n) => match op.as_str() {
                    "=" => n.as_f64() == *q,
                    "==" => n.as_f64() == *q,
                    "!=" => n.as_f64() != *q,
                    "<" => n.as_f64() < *q,
                    "<=" => n.as_f64() <= *q,
                    ">" => n.as_f64() > *q,
                    ">=" => n.as_f64() >= *q,
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
                "=" => *v == Value::Null,
                "==" => *v == Value::Null,
                "!=" => *v != Value::Null,
                _ => false,
            },
            _ => false,
        }
    }
}
