use crate::element;
use crate::element::Element;
use crate::path::Path;
use crate::reader::Bytes;
use crate::sub_selector::SubSelector;
use crate::value::Value;
use crate::Error;
use crate::Result;
use std::collections::HashMap;
use std::str;

pub fn bytes_to_vec<'a>(bytes: &mut Bytes<'a>) -> Result<Vec<Value<'a>>> {
    let mut arr = Vec::new();
    'outer: while let Some(b) = bytes.peek() {
        if let b'[' = b {
            bytes.next();
            loop {
                match element::read_one(bytes)? {
                    Some(element) => {
                        arr.push(element.to_value());
                        continue;
                    }
                    None => break 'outer,
                }
            }
        };

        bytes.next();
    }

    Ok(arr)
}

pub fn bytes_to_map<'a>(bytes: &mut Bytes<'a>) -> Result<HashMap<&'a str, Value<'a>>> {
    let mut m = HashMap::new();
    let mut key_cache: Option<&str> = None;
    let mut count = 0;
    #[allow(unused_assignments)]
    let mut is_map = false;
    'outer: while let Some(b) = bytes.peek() {
        if let b'{' = b {
            is_map = true;
            bytes.next();
            loop {
                match element::read_one(bytes)? {
                    Some(element) => {
                        count += 1;
                        if count % 2 == 1 {
                            match element {
                                Element::String(buf) => {
                                    let key =
                                        unsafe { str::from_utf8_unchecked(&buf[1..buf.len() - 1]) };
                                    key_cache = Some(key);
                                }
                                _ => return Err(Error::ObjectKey),
                            };
                        } else {
                            m.insert(key_cache.take().unwrap(), element.to_value());
                        }
                    }
                    None => break 'outer,
                }
            }
        };

        bytes.next();
    }

    if is_map {
        Ok(m)
    } else {
        Err(Error::Object)
    }
}

pub fn bytes_get<'a>(bytes: &mut Bytes<'a>, path: &Path<'a>) -> Result<Option<Element<'a>>> {
    if !path.ok {
        return Ok(None);
    }

    if path.has_selectors() {
        let element = match path.arrsel {
            true => select_to_array(bytes, path.borrow_selectors())?,
            false => select_to_object(bytes, path.borrow_selectors())?,
        };

        match element {
            Some(element) => {
                if path.more {
                    return element_get(element, path.borrow_next());
                } else {
                    return Ok(Some(element));
                }
            }
            None => return Ok(None),
        }
    }

    while let Some(b) = bytes.peek() {
        match b {
            b'{' => {
                bytes.next();
                return object_bytes_get(bytes, path);
            }
            b'[' => {
                bytes.next();
                return array_bytes_get(bytes, path);
            }

            _ => {
                bytes.next();
                continue;
            }
        }
    }

    Ok(None)
}

fn select_to_object<'a>(
    bytes: &mut Bytes<'a>,
    sels: &[SubSelector<'a>],
) -> Result<Option<Element<'a>>> {
    let mut map = HashMap::new();
    let start = bytes.position();

    for sel in sels {
        let path = Path::parse(sel.path)?;
        bytes.seek(start);
        if let Some(sub_pv) = bytes_get(bytes, &path)? {
            let key = unsafe { str::from_utf8_unchecked(sel.name) };
            map.insert(key, sub_pv);
        }
    }

    Ok(Some(Element::Map(map)))
}

fn select_to_array<'a>(
    bytes: &mut Bytes<'a>,
    sels: &[SubSelector<'a>],
) -> Result<Option<Element<'a>>> {
    let mut list = Vec::new();
    let start = bytes.position();

    for sel in sels {
        let path = Path::parse(sel.path)?;
        bytes.seek(start);
        if let Some(sub_pv) = bytes_get(bytes, &path)? {
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
        Element::Array(buf) | Element::Object(buf) => {
            let mut bytes = Bytes::new(buf);
            return bytes_get(&mut bytes, path);
        }
        _ => return Ok(None),
    }
}

fn element_get<'a>(element: Element<'a>, path: &Path<'a>) -> Result<Option<Element<'a>>> {
    if !path.ok {
        return Ok(None);
    }

    match element {
        Element::Array(buf) | Element::Object(buf) => {
            let mut bytes = Bytes::new(buf);
            return bytes_get(&mut bytes, path);
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

            return Ok(None);
        }
        Element::List(elements) => {
            let query = path.borrow_query();
            let query_list = (query.on && query.all) || (!query.on && path.more);
            let query_first = query.on && !query.all;

            if query_first {
                if elements.len() > 0 {
                    let first = elements.into_iter().nth(0).unwrap();
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

            return Ok(Some(Element::Count(elements.len())));
        }
        _ => return Ok(None),
    }
}

fn object_bytes_get<'a>(bytes: &mut Bytes<'a>, path: &Path<'a>) -> Result<Option<Element<'a>>> {
    let mut num = 0;
    loop {
        let element = element::read_one(bytes)?;
        if element.is_none() {
            return Ok(None);
        }

        num += 1;
        // object value
        if num % 2 == 0 {
            continue;
        }

        // object key
        match element {
            Some(Element::String(buf)) => {
                if path.is_match(&buf[1..buf.len() - 1]) {
                    return if path.more {
                        bytes_get(bytes, path.borrow_next())
                    } else {
                        element::read_one(bytes)
                    };
                }
            }
            _ => return Err(Error::ObjectKey),
        }
    }
}

fn array_bytes_get<'a>(bytes: &mut Bytes<'a>, path: &Path<'a>) -> Result<Option<Element<'a>>> {
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

    loop {
        // index matched
        if get_idx && idx == index {
            return if path.more {
                bytes_get(bytes, path.borrow_next())
            } else {
                element::read_one(bytes)
            };
        }

        let mut element = match element::read_one(bytes)? {
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
            } else {
                if !query.match_element(&element) {
                    continue;
                }
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
            return Ok(Some(element));
        }

        if return_list {
            elements.push(element);
        }
    }

    if return_list {
        Ok(Some(Element::List(elements)))
    } else if only_first {
        Ok(None)
    } else if path.arrch {
        Ok(Some(Element::Count(index)))
    } else {
        Ok(None)
    }
}
