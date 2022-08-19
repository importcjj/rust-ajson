use crate::element;
use crate::Result;
use path::{Path, Query, QueryValue};
use reader::Bytes;
use sub_selector;

use number::Number;
use util;

pub(super) fn parse_path(v: &[u8]) -> Result<Path> {
    if v.is_empty() {
        return Ok(Path::empty());
    }

    let mut bytes = Bytes::new(v);
    let mut current_path = Path::empty();
    let mut depth = 0;

    while let Some(b) = bytes.peek() {
        match b {
            b'\\' => {
                bytes.next();
            }
            b']' | b')' | b'}' => {
                if depth > 0 {
                    depth -= 0;
                }
            }
            b'.' => {
                if depth == 0 && bytes.position() > 0 {
                    let end = bytes.position() - 1;
                    current_path.set_part(bytes.head(v, end));
                    current_path.set_ok(true);
                    current_path.set_more(true);
                    bytes.next();
                    let next = parse_path(bytes.tail(v))?;
                    if next.ok {
                        current_path.set_next(next);
                    }
                    return Ok(current_path);
                }
            }
            #[cfg(feature = "wild")]
            b'*' | b'?' => current_path.set_wild(true),
            b'#' => {
                if depth == 0 {
                    current_path.set_arrch(true)
                }
            }
            b @ b'[' | b @ b'(' | b @ b'{' => {
                depth += 1;
                if depth == 1 {
                    if current_path.arrch {
                        bytes.next();
                        let (query, offset) = parse_query(bytes.tail(v))?;
                        if query.on {
                            bytes.forward(offset - 1);
                        }
                        current_path.set_q(query);

                        depth = 0;
                        continue;
                    } else {
                        let (selectors, offset, ok) = sub_selector::parse_selectors(bytes.tail(v));
                        if ok {
                            if b != b'{' {
                                current_path.set_arrsel(true);
                            }
                            current_path.set_selectors(selectors);
                            bytes.forward(offset - 1);
                            depth = 0;
                        }
                    }
                }
            }
            _ => (),
        };
        bytes.next();
    }

    current_path.set_part(v);
    current_path.set_more(false);
    current_path.set_ok(true);
    // println!("path => {:?}", current_path);
    Ok(current_path)
}

fn parse_query(v: &[u8]) -> Result<(Query, usize)> {
    if v.is_empty() {
        return Ok((Query::empty(), 0));
    }

    // println!("parse query {:?}", String::from_utf8_lossy(v));
    let mut bytes = Bytes::new(v);
    let mut q = Query::empty();
    let mut depth = 1;
    let mut end = 0;

    let mut op_exist = false;
    let mut op_start = 0;
    let mut op_end = 0;

    while let Some(b) = bytes.peek() {
        match b {
            b'!' | b'=' | b'<' | b'>' | b'%' => {
                if depth == 1 {
                    if !op_exist {
                        op_exist = true;
                        op_start = bytes.position();
                        op_end = op_start;
                    } else {
                        op_end += 1;
                    }
                }
            }
            b'[' | b'(' => {
                depth += 1;
            }
            b']' | b')' => {
                depth -= 1;
                if depth == 0 {
                    match bytes.next() {
                        Some(b'#') => {
                            q.set_all(true);
                            end = bytes.position();
                            bytes.next();
                        }
                        Some(_) => end = bytes.position() - 1,
                        None => end = bytes.position(),
                    }
                    break;
                }
            }
            b' ' => (),
            _ => {
                if op_exist {
                    let (val, offset) = parser_query_value(bytes.tail(v))?;
                    if val.exists() {
                        q.set_val(val);
                    }
                    if offset > 1 {
                        bytes.forward(offset - 1);
                    }
                }
            }
        };

        bytes.next();
    }

    q.set_on(true);

    if op_exist {
        q.set_path(util::trim_space_u8(&v[..op_start]));
        q.set_op(unsafe { std::str::from_utf8_unchecked(bytes.slice(op_start, op_end)) });
    } else if end > 0 {
        q.set_path(util::trim_space_u8(&v[..end + 1]));
    } else {
        q.set_path(util::trim_space_u8(v));
    }

    Ok((q, bytes.offset()))
}

fn parser_query_value(v: &[u8]) -> Result<(QueryValue, usize)> {
    // println!("parse query value {:?}", String::from_utf8_lossy(v));
    let mut bytes = Bytes::new(v);
    if let Some(b) = bytes.peek() {
        let value = match b {
            b't' => {
                element::read_true(&mut bytes).unwrap();
                QueryValue::Boolean(true)
            }
            b'f' => {
                element::read_false(&mut bytes).unwrap();
                QueryValue::Boolean(false)
            }
            b'n' => {
                element::read_null(&mut bytes).unwrap();
                QueryValue::Null
            }
            b'"' => {
                let (start, end) = element::read_str_range(&mut bytes)?;
                if end - start < 2 {
                    QueryValue::NotExist
                } else {
                    let raw = bytes.slice(start + 1, end - 1);
                    let s = unsafe { std::str::from_utf8_unchecked(raw) };
                    QueryValue::String(s)
                }
                // Value::Null
            }
            b'0'..=b'9' | b'-' => {
                let n = Number::from(&mut bytes);
                QueryValue::F64(n.to_f64())
            }
            _ => QueryValue::NotExist,
        };

        return Ok((value, bytes.offset() - 1));
    }

    Ok((QueryValue::NotExist, 0))
}

#[allow(dead_code)]
fn new_query_from_utf8(v: &[u8]) -> Result<Query> {
    let (q, _) = parse_query(v)?;
    Ok(q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_path() {
        parse_path("friends.{}first]".as_bytes()).unwrap();
    }

    #[test]
    fn test_fn_parse_path_from_utf8() {
        let v = r#"name"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"#(last=="Murphy")#.first"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"friends.#(first!%"D*")#.last"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"c?ildren.0"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"#(sub_item>7)#.title"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");
        let v = r#"friends.#(nets."#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"friends.#()#"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = "widget.[window,name].#.name".as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"widget.menu.#(title="help")#.title"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");
    }

    #[test]
    fn test_fn_parse_path() {
        let v = r#"name"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"#(last=="Murphy")#.first"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"friends.#(first!%"D*")#.last"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"c?ildren.0"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");

        let v = r#"#(sub_item>7)#.title"#.as_bytes();
        let p = parse_path(v);
        println!("{:?}", p);
        println!("======================");
    }

    #[test]
    fn test_fn_parse_query() {
        let v = "first)".as_bytes();
        let q = new_query_from_utf8(v);
        println!("{:?}", q);
        println!("======================");

        let v = "first)#".as_bytes();
        let q = new_query_from_utf8(v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"first="name")"#.as_bytes();
        let q = new_query_from_utf8(v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"nets.#(=="ig"))"#.as_bytes();
        let q = new_query_from_utf8(v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"nets.#(=="ig"))#"#.as_bytes();
        let q = new_query_from_utf8(v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"=="ig")"#.as_bytes();
        let q = new_query_from_utf8(v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"first=)"#.as_bytes();
        let q = new_query_from_utf8(v);
        println!("{:?}", q);
        println!("======================");

        let v = r#"sub_item>7)#.title"#.as_bytes();
        let q = new_query_from_utf8(v);
        println!("{:?}", q);
        println!("======================");
    }
}
