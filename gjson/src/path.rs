use read::UTF8Reader;
use std::fmt;
use util;
use value::Value;
use wild;


static DEFAULT_NONE_PATH: Path = Path {
    ok: false,
    part: &[],
    next: None,
    more: false,
    wild: false,
    arrch: false,

    query: None,
};

pub struct Path<'a> {
    pub ok: bool,
    pub part: &'a [u8],
    pub next: Option<Box<Path<'a>>>,
    pub query: Option<Query<'a>>,
    pub more: bool,
    pub wild: bool,
    pub arrch: bool,

    // pub query: Query<'a>,
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
        if self.more {
            write!(f, "\n=> next=`{:?}`", self.next.as_ref().unwrap())?;
        }
        if self.has_query() {
            write!(f, "\n=> query={:?}", self.query)?;
        }
        write!(f, ">")
    }
}

impl<'a> Path<'a> {
    pub fn new() -> Path<'a> {
        Path {
            ok: false,
            part: &[],
            next: None,
            more: false,
            wild: false,
            arrch: false,

            query: None,
        }
    }

    pub fn new_from_utf8(v: &'a [u8]) -> Path<'a> {
        // println!("parse path {}", String::from_utf8_lossy(&v).to_string());
        if v.len() == 0 {
            return Path::new();
        }

        let mut reader = UTF8Reader::new(v);
        let mut path = Path::new();
        path.set_ok(true);

        while let Some(b) = reader.next() {
            match b {
                b'\\' => {
                    reader.next();
                }
                b'.' => {
                    let end = reader.mark();
                    path.set_part(util::safe_slice(v, 0, end));
                    let next = Path::new_from_utf8(util::safe_slice(v, end + 1, v.len()));
                    path.set_next(next);
                    path.set_more(true);
                    return path;
                }
                b'*' | b'?' => path.set_wild(true),
                b'#' => {
                    path.set_arrch(true);
                }
                b'[' | b'(' => {
                    if path.arrch {
                        reader.back(2);
                        let q = Query::from_utf8_reader(&mut reader).unwrap();
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
                    // path.set_next(util::safe_slice(v, end + 1, v.len()));
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
                        let q = Query::from_utf8_reader(&mut p).unwrap();
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

    pub fn set_part(&mut self, v: &'a [u8]) {
        self.part = v;
    }

    pub fn set_more(&mut self, b: bool) {
        self.more = b;
    }

    pub fn set_next(&mut self, next: Path<'a>) {
        self.next = Some(Box::new(next));
        self.more = true;
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
            None => panic!("path has none query!"),
        }
    }

    pub fn is_match(&self, key: &[u8]) -> bool {
        let eq = if self.wild {
            wild::is_match_u8(key, self.part)
        } else {
            util::equal_escape_u8(key, self.part)
        };

        // println!("match key {:?} == {:?} ? {}", self.part, key, eq);
        eq
    }
}

pub struct Query<'a> {
    pub on: bool,
    pub path: &'a [u8],
    pub key: Option<Box<Path<'a>>>,
    pub op: Option<String>,
    pub value: Option<Value>,
    pub all: bool,
}

impl<'a> fmt::Debug for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Query")?;
        write!(f, " ok={}", self.on)?;
        write!(f, " all={}", self.all)?;
        if self.path.len()> 0 {
            write!(f, "\n=> path=`{}`", String::from_utf8_lossy(self.path).to_string())?;
        }
        if self.key.is_some() {
            write!(f, "\n=> key=`{:?}`", self.key.as_ref().unwrap())?;
        }
        if self.op.is_some() {
            write!(f, "\n=> op=`{}`", self.op.as_ref().unwrap())?;
        }
        if self.value.is_some() {
            write!(f, "\n=> value=`{:?}`", self.value.as_ref().unwrap())?;
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

    pub fn has_key(&self) -> bool {
        self.key.is_some()
    }

    pub fn get_key(&self) -> &Path {
        match self.key {
            Some(_) => self.key.as_ref().unwrap(),
            None => &DEFAULT_NONE_PATH,
        }
    }

    // pub fn has_key(&self) -> bool {
    //     self.path.len() > 0
    // }

    // pub fn get_key(&self) -> Path {
    //     match self.has_key() {
    //         true => Path::new_from_utf8(self.path),
    //         false => Path::new(),
    //     }
    // }

    fn from_utf8(v: &'a [u8]) -> Option<Query<'a>> {
        let mut p = UTF8Reader::new(v);
        Query::from_utf8_reader(&mut p)
    }

    #[allow(dead_code)]
    fn from_utf8_reader<'b, 'c>(p: &'c mut UTF8Reader) -> Option<Query<'b>> {
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
                    p.skip_string();
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
            let path = p.slice(2, j);
            let mut k = 0;
            let mut new_p = UTF8Reader::new(p.tail(j));
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
                            let f = new_p.read_number();
                            // println!("get raw {:?}", raw);
                            Value::Number(f)
                        }
                        _ => Value::NotExists,
                    },
                };
                break;
            }

            let op = new_p.head_contains_last(k);
            Some(Query {
                on: true,
                key: None,
                path: util::trim_space_u8(path),
                op: Some(String::from_utf8_lossy(op).to_string()),
                value: Some(value),
                all,
            })
        } else {
            Some(Query {
                on: true,
                key: None,
                path: util::trim_space_u8(p.slice(2, i)),
                op: None,
                value: None,
                all,
            })
        }
    }

    pub fn set_op(&mut self, op: String) {
        self.op = Some(op);
    }

    pub fn set_val(&mut self, val: Value) {
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
        self.on = true;
    }

    pub fn is_match(&self, v: &Value) -> bool {
        // println!("match value {:?} {:?}",self, v);
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
        let p = Path::new_from_utf8(&v);
        println!("{:?}", p);

        let v = r#"#(last=="Murphy")#.first"#.as_bytes();
        let p = Path::new_from_utf8(&v);
        println!("{:?}", p);

        let v = r#"friends.#(first!%"D*")#.last"#.as_bytes();
        let p = Path::new_from_utf8(&v);
        println!("{:?}", p);

        let v = r#"c?ildren.0"#.as_bytes();
        let p = Path::new_from_utf8(&v);
        println!("{:?}", p);

        let v = r#"#(sub_item>7)#.title"#.as_bytes();
        let p = Path::new_from_utf8(&v);
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
