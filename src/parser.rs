use crate::element;
use crate::element::Element;
use crate::path::Path;
use crate::path::SubSelector;
use crate::value::Value;
use crate::Error;
use crate::Result;
use std::collections::HashMap;
use std::str;

pub fn bytes_to_vec(mut bytes: &[u8]) -> Result<Vec<Value>> {
    let mut arr = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };
        if b == b'[' {
            break;
        };
        i += 1;
    }

    if i >= bytes.len() - 1 {
        return Ok(arr);
    }

    bytes = &bytes[i + 1..];

    loop {
        let (a, left) = element::read_one(bytes)?;
        bytes = left;
        match a {
            Some(element) => {
                arr.push(element.to_value());
                continue;
            }
            None => break,
        }
    }

    Ok(arr)
}

pub fn bytes_to_map(mut bytes: &[u8]) -> Result<HashMap<&str, Value>> {
    let mut m = HashMap::new();

    let mut i = 0;
    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };
        if b == b'{' {
            break;
        };
        i += 1;
    }

    if i == bytes.len() - 1 {
        return Ok(m);
    }

    if i >= bytes.len() {
        return Err(Error::Object);
    }

    i += 1;

    while i < bytes.len() {
        let &b = unsafe { bytes.get_unchecked(i) };
        if b == b'}' {
            break;
        }

        if b != b'"' {
            i += 1;
            continue;
        }

        let input = unsafe { bytes.get_unchecked(i..) };
        let (key, b, esc) = element::string_u8(input)?;
        bytes = b;

        let (val_element, b) = element::read_one(bytes)?;
        bytes = b;
        i = 0;
        match val_element {
            Some(element) => {
                let s = unsafe { std::str::from_utf8_unchecked(&key[1..key.len() - 1]) };
                let value = element.to_value();
                m.insert(s, value);
            }
            None => break,
        }
    }

    Ok(m)
}

pub fn bytes_get<'a>(bytes: &'a [u8], path: &Path<'a>) -> Result<(Option<Element<'a>>, &'a [u8])> {
    if !path.ok || bytes.is_empty() {
        return Ok((None, "".as_bytes()));
    }

    if path.has_selectors() {
        let element = match path.arrsel {
            true => select_to_array(bytes, path.borrow_selectors())?,
            false => select_to_object(bytes, path.borrow_selectors())?,
        };

        match element {
            Some(element) => {
                if path.more {
                    let next = path.parse_next()?;
                    let element = element_get(element, &next)?;
                    return Ok((element, "".as_bytes()));
                } else {
                    return Ok((Some(element), "".as_bytes()));
                }
            }
            None => return Ok((None, "".as_bytes())),
        }
    }

    let mut i = 0;

    type Getter = for<'a> fn(&'a [u8], &Path<'a>) -> element::MakeResult<'a>;

    const GETTER: [Option<Getter>; 256] = {
        let mut table: [Option<Getter>; 256] = [None; 256];
        table[b'{' as usize] = Some(object_bytes_get);
        table[b'[' as usize] = Some(array_bytes_get);

        table
    };

    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };
        match GETTER[b as usize] {
            None => (),
            Some(getter_fn) => {
                let input = unsafe { bytes.get_unchecked(i..) };
                return getter_fn(input, path);
            }
        }

        i += 1;
    }

    Ok((None, "".as_bytes()))
}

fn select_to_object<'a>(input: &'a [u8], sels: &[SubSelector<'a>]) -> Result<Option<Element<'a>>> {
    let mut map = HashMap::new();

    for sel in sels {
        let path = Path::from_slice(sel.path)?;
        if let (Some(sub_pv), _) = bytes_get(input, &path)? {
            map.insert((sel.name, false), sub_pv);
        }
    }

    Ok(Some(Element::Map(map)))
}

fn select_to_array<'a>(input: &'a [u8], sels: &[SubSelector<'a>]) -> Result<Option<Element<'a>>> {
    let mut list = Vec::new();

    for sel in sels {
        let path = Path::from_slice(sel.path)?;
        if let (Some(sub_pv), _) = bytes_get(input, &path)? {
            list.push(sub_pv)
        }
    }

    Ok(Some(Element::List(list)))
}

fn element_ref_get<'a>(element: &Element<'a>, path: &Path<'a>) -> Result<Option<Element<'a>>> {
    if !path.ok {
        return Ok(None);
    }

    match element {
        Element::Array(s) | Element::Object(s) => {
            let (a, _b) = bytes_get(s, path)?;
            Ok(a)
        }
        _ => Ok(None),
    }
}

fn element_get<'a>(element: Element<'a>, path: &Path<'a>) -> Result<Option<Element<'a>>> {
    if !path.ok {
        return Ok(None);
    }

    let next_path = path.parse_next()?;
    match element {
        Element::Array(s) | Element::Object(s) => {
            let (a, _b) = bytes_get(s, path)?;
            Ok(a)
        }
        Element::Map(m) => {
            for (key, value) in m.into_iter() {
                if path.is_match(key.0, key.1) {
                    if path.more {
                        return element_get(value, &next_path);
                    }
                    return Ok(Some(value));
                }
            }

            Ok(None)
        }
        Element::List(elements) => {
            let query = path.borrow_query();
            let query_list = (query.on && query.all) || (!query.on && path.more);
            let query_first = query.on && !query.all;

            if query_first {
                if !elements.is_empty() {
                    let first = elements.into_iter().next().unwrap();
                    if path.more {
                        return element_get(first, &next_path);
                    }
                    return Ok(Some(first));
                }

                return Ok(None);
            }

            if query_list {
                if path.more {
                    let mut results = vec![];
                    for element in elements.into_iter() {
                        if let Some(sub) = element_get(element, &next_path)? {
                            results.push(sub);
                        }
                    }
                    return Ok(Some(Element::List(results)));
                }
                return Ok(Some(Element::List(elements)));
            }

            Ok(Some(Element::Count(elements.len())))
        }
        _ => Ok(None),
    }
}

#[inline]
fn object_bytes_get<'a>(
    mut input: &'a [u8],
    path: &Path<'a>,
) -> Result<(Option<Element<'a>>, &'a [u8])> {
    let mut i = 1;

    while i < input.len() {
        let &b = unsafe { input.get_unchecked(i) };
        if b == b'}' {
            i += 1;
            return Ok((None, &input[i..]));
        }

        if b != b'"' {
            i += 1;
            continue;
        }

        input = unsafe { input.get_unchecked(i..) };
        let (s, left, esc) = element::string_u8(input)?;
        input = left;

        // object key
        if path.is_match(&s[1..s.len() - 1], esc) {
            return if path.more {
                let next_path = path.parse_next()?;
                bytes_get(input, &next_path)
            } else {
                element::read_one(input)
            };
        }

        let (element, left) = element::read_one(input)?;
        if element.is_none() {
            return Ok((None, "".as_bytes()));
        }
        input = left;
        i = 0;
    }

    Ok((None, "".as_bytes()))
}

fn array_bytes_get<'a>(
    mut bytes: &'a [u8],
    path: &Path<'a>,
) -> Result<(Option<Element<'a>>, &'a [u8])> {
    let mut index = 0;
    let (idx, get_idx) = match str::from_utf8(path.part)
        .map_err(|_| Error::Path)?
        .parse::<usize>()
    {
        Ok(i) => (i, true),
        Err(_) => (0, false),
    };

    let query = path.borrow_query();
    let query_key = query.get_path()?;

    let mut elements = Vec::new();
    let return_list = (query.on && query.all) || (!query.on && path.more);
    let only_first = query.on && !query.all;

    bytes = &bytes[1..];

    let next_path = path.parse_next()?;

    loop {
        // index matched
        if get_idx && idx == index {
            return if path.more {
                bytes_get(bytes, &next_path)
            } else {
                element::read_one(bytes)
            };
        }

        let (readed, left) = element::read_one(bytes)?;
        bytes = left;

        let mut element = match readed {
            None => break,
            Some(el) => el,
        };

        // do query filter
        if query.on {
            if query.has_path() {
                match element_ref_get(&element, &query_key)? {
                    None => continue,
                    Some(v) => {
                        if !query.match_element(&v) {
                            continue;
                        }
                    }
                }
            } else if !query.match_element(&element) {
                continue;
            }
        }

        index += 1;

        if path.more {
            match element_get(element, &next_path)? {
                Some(el) => element = el,
                None => continue,
            }
        }

        if only_first {
            return Ok((Some(element), bytes));
        }

        if return_list {
            elements.push(element);
        }
    }

    if return_list {
        Ok((Some(Element::List(elements)), bytes))
    } else if only_first {
        Ok((None, bytes))
    } else if path.arrch {
        Ok((Some(Element::Count(index)), bytes))
    } else {
        Ok((None, bytes))
    }
}
