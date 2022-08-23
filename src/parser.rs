use crate::element;
use crate::element::Element;
use crate::path::Path;
use crate::sub_selector::SubSelector;
use crate::value::Value;
use crate::Error;
use crate::Result;
use std::collections::HashMap;
use std::str;

pub fn bytes_to_vec(mut input: &str) -> Result<Vec<Value>> {
    let mut arr = Vec::new();
    let mut i = 0;
    let bytes = input.as_bytes();
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

    input = &input[i + 1..];

    loop {
        let (a, left) = element::read_one(input)?;
        input = left;
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

pub fn bytes_to_map(mut input: &str) -> Result<HashMap<&str, Value>> {
    let mut m = HashMap::new();
    let mut key_cache: Option<&str> = None;
    let mut count = 0;

    let mut i = 0;
    let bytes = input.as_bytes();
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

    input = &input[i + 1..];

    loop {
        let (a, b) = element::read_one(input)?;
        input = b;
        match a {
            Some(element) => {
                count += 1;
                if count % 2 == 1 {
                    match element {
                        Element::String(s) => {
                            key_cache = Some(&s[1..s.len() - 1]);
                        }
                        _ => return Err(Error::ObjectKey),
                    };
                } else {
                    m.insert(key_cache.take().unwrap(), element.to_value());
                }
            }
            None => break,
        }
    }

    Ok(m)
}

pub fn bytes_get<'a>(input: &'a str, path: &Path<'a>) -> Result<(Option<Element<'a>>, &'a str)> {
    if !path.ok || input.is_empty() {
        return Ok((None, ""));
    }

    if path.has_selectors() {
        let element = match path.arrsel {
            true => select_to_array(input, path.borrow_selectors())?,
            false => select_to_object(input, path.borrow_selectors())?,
        };

        match element {
            Some(element) => {
                if path.more {
                    let element = element_get(element, path.borrow_next())?;
                    return Ok((element, ""));
                } else {
                    return Ok((Some(element), ""));
                }
            }
            None => return Ok((None, "")),
        }
    }

    let bytes = input.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };
        match b {
            b'{' => {
                let input = unsafe { input.get_unchecked(i..) };
                return object_bytes_get(input, path);
            }
            b'[' => {
                let input = unsafe { input.get_unchecked(i..) };
                return array_bytes_get(input, path);
            }

            _ => {}
        }

        i += 1;
    }

    Ok((None, ""))
}

fn select_to_object<'a>(input: &'a str, sels: &[SubSelector<'a>]) -> Result<Option<Element<'a>>> {
    let mut map = HashMap::new();

    for sel in sels {
        let path = Path::parse(sel.path)?;
        if let (Some(sub_pv), _) = bytes_get(input, &path)? {
            let key = unsafe { str::from_utf8_unchecked(sel.name) };
            map.insert(key, sub_pv);
        }
    }

    Ok(Some(Element::Map(map)))
}

fn select_to_array<'a>(input: &'a str, sels: &[SubSelector<'a>]) -> Result<Option<Element<'a>>> {
    let mut list = Vec::new();

    for sel in sels {
        let path = Path::parse(sel.path)?;
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
            let (a, b) = bytes_get(s, path)?;
            Ok(a)
        }
        _ => Ok(None),
    }
}

fn element_get<'a>(element: Element<'a>, path: &Path<'a>) -> Result<Option<Element<'a>>> {
    if !path.ok {
        return Ok(None);
    }

    match element {
        Element::Array(s) | Element::Object(s) => {
            let (a, b) = bytes_get(s, path)?;
            Ok(a)
        }
        Element::Map(m) => {
            for (key, value) in m.into_iter() {
                if path.is_match(key.as_bytes()) {
                    if path.more {
                        return element_get(value, path.borrow_next());
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
                        return element_get(first, path.borrow_next());
                    }
                    return Ok(Some(first));
                }

                return Ok(None);
            }

            if query_list {
                if path.more {
                    let mut results = vec![];
                    for element in elements.into_iter() {
                        if let Some(sub) = element_get(element, path.borrow_next())? {
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
    mut input: &'a str,
    path: &Path<'a>,
) -> Result<(Option<Element<'a>>, &'a str)> {
    input = &input[1..];
    loop {
        let (element, left) = element::read_one(input)?;
        if element.is_none() {
            return Ok((None, ""));
        }

        input = left;

        // object key
        match element {
            Some(Element::String(s)) => {
                if path.is_match(s[1..s.len() - 1].as_bytes()) {
                    return if path.more {
                        bytes_get(input, path.borrow_next())
                    } else {
                        element::read_one(input)
                    };
                }
            }
            _ => return Err(Error::ObjectKey),
        }

        let (element, left) = element::read_one(input)?;
        if element.is_none() {
            return Ok((None, ""));
        }
        input = left;
    }
}

fn array_bytes_get<'a>(
    mut input: &'a str,
    path: &Path<'a>,
) -> Result<(Option<Element<'a>>, &'a str)> {
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

    let mut elements = vec![];
    let return_list = (query.on && query.all) || (!query.on && path.more);
    let only_first = query.on && !query.all;

    input = &input[1..];

    loop {
        // index matched
        if get_idx && idx == index {
            return if path.more {
                bytes_get(input, path.borrow_next())
            } else {
                element::read_one(input)
            };
        }

        let (readed, left) = element::read_one(input)?;
        input = left;

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
            match element_get(element, path.borrow_next())? {
                Some(el) => element = el,
                None => continue,
            }
        }

        if only_first {
            return Ok((Some(element), input));
        }

        if return_list {
            elements.push(element);
        }
    }

    if return_list {
        Ok((Some(Element::List(elements)), input))
    } else if only_first {
        Ok((None, input))
    } else if path.arrch {
        Ok((Some(Element::Count(index)), input))
    } else {
        Ok((None, input))
    }
}
