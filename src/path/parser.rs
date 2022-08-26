use crate::element;
use crate::number::Number;
use crate::util;
use crate::Result;

use super::path::{Path, Query, QueryValue};
use super::sub_selector;

pub(super) fn parse(v: &[u8]) -> Result<Path> {
    if v.is_empty() {
        return Ok(Default::default());
    }

    let bytes = v;
    let mut current_path = Path::default();
    let mut depth = 0;
    let mut i = 0;

    while i < bytes.len() {
        let &b = unsafe { bytes.get_unchecked(i) };

        match b {
            b'\\' => {
                i += 2;
                current_path.set_esc(true);
                continue;
            }
            b']' | b')' | b'}' => {
                if depth > 0 {
                    depth -= 0;
                }
            }
            b'.' => {
                if depth == 0 && i > 0 {
                    current_path.set_part(&v[..i]);
                    current_path.set_ok(true);
                    current_path.set_more(true);
                    i += 1;

                    current_path.set_next(&v[i..]);

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
                        i += 1;
                        let (query, offset) = parse_query(&v[i..])?;
                        if query.on {
                            i += offset - 1;
                        }
                        current_path.set_q(query);

                        depth = 0;
                        continue;
                    } else {
                        let (selectors, offset, ok) = sub_selector::parse_selectors(&v[i..]);
                        if ok {
                            if b != b'{' {
                                current_path.set_arrsel(true);
                            }
                            current_path.set_selectors(selectors);
                            i += offset - 1;
                            depth = 0;
                        }
                    }
                }
            }
            _ => (),
        };
        i += 1;
    }

    current_path.set_part(v);
    current_path.set_more(false);
    current_path.set_ok(true);
    Ok(current_path)
}

fn parse_query(v: &[u8]) -> Result<(Query, usize)> {
    if v.is_empty() {
        return Ok((Query::empty(), 0));
    }

    let bytes = v;
    let mut q = Query::empty();
    let mut depth = 1;
    let mut end = 0;

    let mut op_exist = false;
    let mut op_start = 0;
    let mut op_end = 0;

    let mut i = 0;

    while i < bytes.len() {
        let &b = unsafe { bytes.get_unchecked(i) };

        match b {
            b'!' | b'=' | b'<' | b'>' | b'%' => {
                if depth == 1 {
                    if !op_exist {
                        op_exist = true;
                        op_start = i;
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
                    match bytes.get(i + 1) {
                        Some(b'#') => {
                            q.set_all(true);
                            end = i;
                            i += 1;
                        }
                        Some(_) => end = i - 1,
                        None => end = i,
                    }
                    break;
                }
            }
            b' ' => (),
            _ => {
                if op_exist {
                    let (val, offset) = parser_query_value(&v[i..])?;
                    if val.exists() {
                        q.set_val(val);
                    }
                    if offset > 1 {
                        i += offset - 1;
                    }
                }
            }
        };

        i += 1;
    }

    q.set_on(true);

    if op_exist {
        q.set_path(util::trim_space_u8(&v[..op_start]));
        q.set_op(unsafe { std::str::from_utf8_unchecked(v.get_unchecked(op_start..op_end + 1)) });
    } else if end > 0 {
        q.set_path(util::trim_space_u8(&v[..end + 1]));
    } else {
        q.set_path(util::trim_space_u8(v));
    }

    Ok((q, i))
}

fn parser_query_value(bytes: &[u8]) -> Result<(QueryValue, usize)> {
    if let Some(b) = bytes.first() {
        let val = match b {
            b't' => {
                element::true_u8(bytes)?;
                (QueryValue::Boolean(true), 4)
            }
            b'f' => {
                element::false_u8(bytes)?;
                (QueryValue::Boolean(false), 5)
            }
            b'n' => {
                element::null_u8(bytes)?;
                (QueryValue::Null, 4)
            }
            b'"' => {
                let (s, _, esc) = element::string_u8(bytes)?;
                if s.len() < 2 {
                    (QueryValue::NotExist, s.len())
                } else {
                    (QueryValue::String(&s[1..s.len() - 1]), s.len())
                }
            }
            b'0'..=b'9' | b'-' => {
                let (n, _) = element::number_u8(bytes)?;
                (QueryValue::F64(Number::from(n).to_f64()), n.len())
            }
            _ => (QueryValue::NotExist, 0),
        };

        return Ok(val);
    }

    Ok((QueryValue::NotExist, 0))
}

#[cfg(test)]
mod tests {
    #![allow(unused_variables)]
    use super::*;

    #[test]
    fn test_invalid_path() {
        parse("friends.{}first]".as_bytes()).unwrap();
    }

    #[test]
    fn test_fn_parse_from_utf8() {
        let v = r#"name"#.as_bytes();
        let p = parse(v);

        let v = r#"#(last=="Murphy")#.first"#.as_bytes();
        let p = parse(v);

        let v = r#"friends.#(first!%"D*")#.last"#.as_bytes();
        let p = parse(v);

        let v = r#"c?ildren.0"#.as_bytes();
        let p = parse(v);

        let v = r#"#(sub_item>7)#.title"#.as_bytes();
        let p = parse(v);

        let v = r#"friends.#(nets."#.as_bytes();
        let p = parse(v);

        let v = r#"friends.#()#"#.as_bytes();
        let p = parse(v);

        let v = "widget.[window,name].#.name".as_bytes();
        let p = parse(v);

        let v = r#"widget.menu.#(title="help")#.title"#.as_bytes();
        let p = parse(v);
    }

    #[test]
    fn test_fn_parse() {
        let v = r#"name"#.as_bytes();
        let p = parse(v);

        let v = r#"#(last=="Murphy")#.first"#.as_bytes();
        let p = parse(v);

        let v = r#"friends.#(first!%"D*")#.last"#.as_bytes();
        let p = parse(v);

        let v = r#"c?ildren.0"#.as_bytes();
        let p = parse(v);

        let v = r#"#(sub_item>7)#.title"#.as_bytes();
        let p = parse(v);
    }

    #[test]
    fn test_fn_parse_query() {
        let v = "first)".as_bytes();
        let q = parse_query(v);

        let v = "first)#".as_bytes();
        let q = parse_query(v);

        let v = r#"first="name")"#.as_bytes();
        let q = parse_query(v);

        let v = r#"nets.#(=="ig"))"#.as_bytes();
        let q = parse_query(v);

        let v = r#"nets.#(=="ig"))#"#.as_bytes();
        let q = parse_query(v);

        let v = r#"=="ig")"#.as_bytes();
        let q = parse_query(v);

        let v = r#"first=)"#.as_bytes();
        let q = parse_query(v);

        let v = r#"sub_item>7)#.title"#.as_bytes();
        let q = parse_query(v);
    }
}
