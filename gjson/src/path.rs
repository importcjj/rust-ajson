
use parser::Parser;

use read::UTF8Reader;

use std::fmt;
use util;


use value::Value;
use wild;

pub struct Path<'a> {
    pub part: String,
    pub next: &'a [u8],
    pub more: bool,
    wild: bool,
    pub arrch: bool,

    pub query: Query<'a>,
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
        if self.query.on {
            write!(f, " query={:?}", self.query)?;
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

            query: Query::empty(),
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
                    // path.set_part(p.slice(0, end));
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
                        p.back(2);
                        let q = Query::from_utf8_reader(&mut p, v).unwrap();
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

    fn set_q(&mut self, q: Query<'a>) {
        self.query = q;
    }

    fn set_all(&mut self, b: bool) {
        self.query.all = b;
    }


    pub fn is_query_on(&self) -> bool {
        self.query.on
    }

    pub fn is_match(&self, key: &str) -> bool {

        let eq = if self.wild {
            wild::is_match(key, &self.part)
        } else {
            &self.part == key
        };

        // println!("match key {:?} == {:?} ? {}", self.part, key, eq);
        eq
    }
}

pub struct Query<'a> {
    pub on: bool,
    pub path: &'a [u8],
    pub op: Option<String>,
    pub value: Option<Value>,
    pub all: bool,
}

impl<'a> fmt::Debug for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Query")?;
        write!(f, " ok={}", self.on)?;
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
    pub fn empty() -> Query<'a> {
        Query {
            on: false,
            path: &[],
            op: None,
            value: None,
            all: false,
        }
    }

    fn from_utf8(v: &'a [u8]) -> Option<Query<'a>> {
        let mut p = UTF8Reader::new(v);
        Query::from_utf8_reader(&mut p, v)
    }

    #[allow(dead_code)]
    fn from_utf8_reader(p: &mut UTF8Reader, v: &'a [u8]) -> Option<Query<'a>> {
        let mut depth = 1;
        let mut j = 0;

        p.next();
        p.next();

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
        let i = p.mark();

        let all = if let Some(b'#') = p.next() {
            true
        } else {
            p.back(1);
            false
        };



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
                on: true,
                path: util::trim_space_u8(path),
                op: Some(String::from_utf8_lossy(op).to_string()),
                value: Some(value),
                all,
            })
        } else {
            Some(Query {
                on: true,
                path: util::trim_space_u8(util::safe_slice(v, 2, i)),
                op: None,
                value: None,
                all,
            })
        }
    }

    pub fn is_match(&self, v: &Value) -> bool {
        if !v.exists() {
            return false;
        }

        let op = match &self.op {
            Some(ref s) => s,
            None => return true,
        };

        let target = self.value.as_ref().unwrap();

        match &v {
            Value::String(ref s1) => match target {
                Value::String(ref s2) => match op.as_str() {
                    "==" => s1 == s2,
                    "=" => s1 == s2,
                    "!=" => s1 != s2,
                    ">" => s1 > s2,
                    ">=" => s1 >= s2,
                    "<" => s1 < s2,
                    "<=" => s1 <= s2,
                    "%" => wild::is_match(s1, s2),
                    "!%" => !wild::is_match(s1, s2),
                    _ => false,
                },
                _ => false,
            },

            Value::Number(f1) => match target {
                Value::Number(f2) => match op.as_str() {
                    "=" => f1 == f2,
                    "==" => f1 == f2,
                    "!=" => f1 != f2,
                    "<" => f1 < f2,
                    "<=" => f1 <= f2,
                    ">" => f1 > f2,
                    ">=" => f1 >= f2,
                    _ => false,
                },
                _ => false,
            },

            Value::Boolean(b1) => match target {
                Value::Boolean(b2) => match op.as_str() {
                    "=" => b1 == b2,
                    "==" => b1 == b2,
                    "!=" => b1 != b2,
                    "<" => b1 < b2,
                    "<=" => b1 <= b2,
                    ">" => b1 > b2,
                    ">=" => b1 >= b2,
                    _ => false,
                },
                _ => false,
            },

            Value::Null => match op.as_str() {
                "=" => *v == Value::Null,
                "==" => *v == Value::Null,
                "!=" => *v != Value::Null,
                _ => false,
            },
            _ => false,
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
        //  println!("{:?}", p);

        let v = r#"#(last=="Murphy")#.first"#.as_bytes();
        let p = Path::from_utf8(&v);
        //  println!("{:?}", p);

        let v = r#"friends.#(first!%"D*")#.last"#.as_bytes();
        let p = Path::from_utf8(&v);
        //  println!("{:?}", p);

        let v = r#"c?ildren.0"#.as_bytes();
        let p = Path::from_utf8(&v);
        //  println!("{:?}", p);

        let v = r#"#(sub_item>7)#.title"#.as_bytes();
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


        let v = r#"#(sub_item>7)#.title"#.as_bytes();
        let q = Query::from_utf8(&v).unwrap();
        println!("{:?}", q);
    }
}
