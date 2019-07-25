
use crate::util;
use crate::Parser;

use crate::UTF8Reader;
use crate::Value;
use std::str;


use regex::Regex;
use std::fmt;

pub struct Path<'a> {
    part: String,
    pub next: &'a [u8],
    pub more: bool,
    wild: bool,
    arrch: bool,

    pattern: Option<Regex>,

    query: Option<Query<'a>>,
}

impl<'a> fmt::Debug for Path<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Path")?;
        write!(f, " part=`{}` ", self.part)?;
        write!(
            f,
            " next=`{}` ",
            String::from_utf8_lossy(self.next).to_string()
        )?;
        write!(f, " more={} ", self.more)?;
        write!(f, " wild={} ", self.wild)?;
        write!(f, " arrch={}", self.arrch)?;
        if self.query.is_some() {
            write!(f, " query={:?}", self.query.as_ref().unwrap())?;
        }
        write!(f, ">")
    }
}

impl<'a> Path<'a> {
    fn new() -> Path<'a> {
        Path {
            part: String::new(),
            next: &[],
            more: false,
            wild: false,
            arrch: false,
            pattern: None,

            query: None,
        }
    }

    pub fn from_utf8(v: &'a [u8]) -> Path<'a> {
        let mut p = UTF8Reader::new(v);
        let mut path = Path::new();

        while let Some(b) = p.next() {
            match b {
                b'\\' => {
                    p.next();
                }
                b'.' => {
                    let end = p.mark();
                    path.set_part(util::safe_slice(v, 0, end));
                    path.set_next(util::safe_slice(v, end + 1, v.len()));
                    path.set_more(true);
                    return path;
                }
                b'*' | b'?' => path.set_wild(true),
                b'#' => {
                    path.set_arrch(true);
                }
                b'[' | b'(' => {
                    if path.arrch {
                        let q = Query::from_utf8_reader(&mut p, v);
                        path.set_q(q);
                    }
                }
                _ => (),
            };
        }

        path.set_part(v);
        path.set_more(false);

        path
    }


    fn set_part(&mut self, v: &'a [u8]) {
        self.part = String::from_utf8_lossy(v).to_string();
        if self.wild {
            self.part = self.part.replace("?", ".").replace("*", ".+?");
            let re = Regex::new(&self.part).unwrap();
            self.pattern = Some(re);
        }
    }
    fn set_more(&mut self, b: bool) {
        self.more = b;
    }
    fn set_next(&mut self, v: &'a [u8]) {
        self.next = v;
    }
    fn set_wild(&mut self, b: bool) {
        self.wild = b;
    }
    fn set_arrch(&mut self, b: bool) {
        self.arrch = b;
    }
    fn set_q(&mut self, q: Option<Query<'a>>) {
        self.query = q;
    }
    fn is_query_on(&self) -> bool {
        self.query.is_some()
    }

    fn set_pattern(&mut self, p: Option<Regex>) {
        self.pattern = p;
    }

    pub fn match_part(&self, key: &str) -> bool {
        match &self.pattern {
            Some(p) => return p.is_match(key),
            None => (),
        };
        let eq = &self.part == key;
        println!("match {} == {} => {}", self.part, key, eq);
        eq
        
    }
}

pub struct Query<'a> {
    pub ok: bool,
    pub path: &'a [u8],
    pub op: Option<String>,
    pub value: Option<Value>,
    pub all: bool,
}

impl<'a> fmt::Debug for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Query")?;
        write!(f, " ok={}", self.ok)?;
        write!(f, " all={}", self.all)?;
        write!(
            f,
            " key=`{}`",
            String::from_utf8_lossy(self.path).to_string()
        )?;
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

    fn from_utf8(v: &'a [u8]) -> Option<Query<'a>> {
        let mut p = UTF8Reader::new(v);
        Query::from_utf8_reader(&mut p, v)
    }

    #[allow(dead_code)]
    fn from_utf8_reader(p: &mut UTF8Reader, v: &'a [u8]) -> Option<Query<'a>> {
        let mut depth = 1;
        let mut j = 0;


        while let Some(b) = p.next() {
            match b {
                b'!' | b'=' | b'<' | b'>' | b'%' => {
                    if depth == 1 && j == 0 {
                        j = p.mark();
                    }
                }
                b'\\' => {
                    p.next();
                }
                b'"' => {
                    p.read_string();
                }
                b'[' | b'(' => depth += 1,
                b']' | b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => continue,
            }
        }

        if depth > 0 {
            return None;
        }

        let mut value = Value::NotExists;
        let mut all = false;
        let i = p.mark();
        if let Some(b'#') = p.next() {
            all = true;
        }
        if j > 0 {
            let path = util::safe_slice(v, 2, j);
            let mut k = 0;
            let mut new_p = Parser::new(p.tail(j));
            while let Some(b) = new_p.next() {
                value = match b {
                    b'!' | b'>' | b'<' | b'=' | b'%' | b' ' => {
                        k = new_p.mark();
                        continue;
                    }
                    _ => match b {
                        b't' => Value::Boolean(true),
                        b'f' => Value::Boolean(false),
                        b'n' => Value::Null,
                        b'"' => {
                            let raw = new_p.read_string();
                            Value::String(raw)
                        }
                        b'0'...b'9' | b'-' => {
                            let raw = new_p.read_number();
                            Value::Number(raw)
                        }
                        _ => Value::NotExists,
                    },
                };
                break;
            }

            let op = new_p.head_contains_last(k);
            Some(Query {
                ok: true,
                path: util::trim_space_u8(path),
                op: Some(String::from_utf8_lossy(op).to_string()),
                value: Some(value),
                all,
            })
        } else {
            Some(Query {
                ok: true,
                path: util::trim_space_u8(util::safe_slice(v, 2, i)),
                op: None,
                value: None,
                all,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_path() {
        let v = r#"name"#.as_bytes();
        let p = Path::from_utf8(&v);
        println!("{:?}", p);


        let v = r#"#(last=="Murphy")#.first"#.as_bytes();
        let p = Path::from_utf8(&v);
        println!("{:?}", p);

        let v = r#"friends.#(first!%"D*")#.last"#.as_bytes();
        let p = Path::from_utf8(&v);
        println!("{:?}", p);

        let v = r#"c?ildren.0"#.as_bytes();
        let p = Path::from_utf8(&v);
        println!("{:?}", p);
    }

    #[test]
    fn test_parse_query() {
        let v = "#(first)".as_bytes();
        let q = Query::from_utf8(&v).unwrap();
        println!("{:?}", q);

        let v = "#(first)#".as_bytes();
        let q = Query::from_utf8(&v).unwrap();
        println!("{:?}", q);

        let v = r#"#(first="name")"#.as_bytes();
        let q = Query::from_utf8(&v).unwrap();
        println!("{:?}", q);

        let v = r#"#(nets.#(=="ig"))"#.as_bytes();
        let q = Query::from_utf8(&v).unwrap();
        println!("{:?}", q);

        let v = r#"#(nets.#(=="ig"))#"#.as_bytes();
        let q = Query::from_utf8(&v).unwrap();
        println!("{:?}", q);

        let v = r#"#(=="ig")"#.as_bytes();
        let q = Query::from_utf8(&v).unwrap();
        println!("{:?}", q);

        let v = r#"#(first=)"#.as_bytes();
        let q = Query::from_utf8(&v).unwrap();
        println!("{:?}", q);
    }
}