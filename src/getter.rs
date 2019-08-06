use path::Path;
use reader;
use std::collections::HashMap;
use std::io;

use number::Number;
use std::str;
use sub_selector::SubSelector;
use unescape;
use value::Value;

pub struct Getter<R>
where
    R: reader::ByteReader,
{
    source: R,
}

#[derive(PartialEq, Debug)]
enum ParserValue {
    String(usize, usize),
    Object(usize, usize),
    Array(usize, usize),
    Null,
    Boolean(bool),
    Number(usize, usize),
    NumberUsize(usize),
    NotExist,

    ArrayString(String),
    ObjectString(String),
}

impl ParserValue {
    pub fn exists(&self) -> bool {
        *self != ParserValue::NotExist
    }

    pub fn is_vector(&self) -> bool {
        if let ParserValue::ArrayString(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn vector_to_value(self) -> Option<Value> {
        if let ParserValue::ArrayString(s) = self {
            Some(Value::Array(s))
        } else {
            None
        }
    }
}

impl<R> Getter<reader::LazyReader<R>>
where
    R: io::Read,
{
    pub fn new_from_read(r: R) -> Self {
        let lr = reader::LazyReader::new(r);
        Getter { source: lr }
    }
}

impl<'a> Getter<reader::RefReader<'a>> {
    pub fn new_from_utf8(v: &'a [u8]) -> Self {
        let rr = reader::RefReader::new(v);
        Getter { source: rr }
    }

    pub fn from_str(s: &'a str) -> Self {
        let rr = reader::RefReader::new(s.as_bytes());
        Getter { source: rr }
    }
}

impl<R> Getter<R>
where
    R: reader::ByteReader,
{
    pub fn get(&mut self, path: &str) -> Option<Value> {
        let v = path.as_bytes();
        self.get_by_utf8(&v)
    }


    pub fn get_by_utf8(&mut self, v: &[u8]) -> Option<Value> {
        if v.len() == 0 {
            return None
        }

        // reset offset
        self.seek(0);
        let path = Path::new_from_utf8(v);
        let pv = self.get_by_path(&path);
        if pv.is_vector() {
            pv.vector_to_value()
        } else {
            self.parse_value(pv)
        }
    }

    pub fn to_object(&mut self) -> HashMap<String, Value> {
        let mut m = HashMap::new();
        let mut key_cache: Option<String> = None;
        let mut count = 0;
        'outer: while let Some(b) = self.peek() {
            match b {
                b'{' => {
                    self.next_byte();
                    loop {
                        let v = self.read_next_parse_value();
                        if v.exists() {
                            count += 1;
                            if count % 2 == 1 {
                                match v {
                                    ParserValue::String(start, end) => {
                                        let s = String::from_utf8_lossy(
                                            self.bytes_slice(start + 1, end - 1),
                                        )
                                        .to_string();
                                        key_cache = Some(s);
                                    }
                                    _ => panic!("invalid map key"),
                                };
                            } else {
                                m.insert(key_cache.take().unwrap(), self.parse_value(v).unwrap());
                            }
                            continue;
                        }
                        break 'outer;
                    }
                }
                _ => (),
            };

            self.next_byte();
        }

        m
    }

    pub fn to_vec(&mut self) -> Vec<Value> {
        let mut arr = Vec::new();
        'outer: while let Some(b) = self.peek() {
            match b {
                b'[' => {
                    self.next_byte();
                    loop {
                        let v = self.read_next_parse_value();
                        if v.exists() {
                            arr.push(self.parse_value(v).unwrap());
                            continue;
                        }
                        break 'outer;
                    }
                }
                _ => (),
            };

            self.next_byte();
        }

        arr
    }
}

impl<R> Getter<R>
where
    R: reader::ByteReader,
{
    fn next_byte(&mut self) -> Option<u8> {
        self.source.next()
    }

    fn peek(&mut self) -> Option<u8> {
        self.source.peek()
    }

    fn position(&self) -> usize {
        self.source.position()
    }

    fn seek(&mut self, new: usize) {
        self.source.seek(new);
    }

    fn bytes_slice(&self, start: usize, end: usize) -> &[u8] {
        self.source.slice(start, end)
    }

    #[allow(dead_code)]
    fn value_to_raw_str<'b>(&'b mut self, v: &'b ParserValue) -> &'b str {
        match *v {
            ParserValue::String(start, end)
            | ParserValue::Object(start, end)
            | ParserValue::Array(start, end)
            | ParserValue::Number(start, end) => {
                str::from_utf8(self.bytes_slice(start, end)).unwrap()
            }
            ParserValue::ArrayString(ref s) => s,
            ParserValue::Boolean(true) => "true",
            ParserValue::Boolean(false) => "false",
            // ParserValue::NumberUsize(u) => &u.to_string(),
            ParserValue::Null => "null",
            _ => "",
        }
    }

    fn write_to_bytes_buffer(&self, v: &mut Vec<u8>, pv: &ParserValue) {
        match *pv {
            ParserValue::String(start, end)
            | ParserValue::Object(start, end)
            | ParserValue::Array(start, end)
            | ParserValue::Number(start, end) => v.extend(self.bytes_slice(start, end)),

            ParserValue::ArrayString(ref s) | ParserValue::ObjectString(ref s) => {
                v.extend(s.as_bytes())
            }

            ParserValue::Boolean(true) => v.extend("true".as_bytes()),
            ParserValue::Boolean(false) => v.extend("false".as_bytes()),
            ParserValue::NumberUsize(ref u) => v.extend(u.to_string().as_bytes()),
            ParserValue::Null => v.extend("null".as_bytes()),
            _ => (),
        };
    }

    fn write_to_string_buffer<'b>(&'b mut self, buffer: &mut String, v: &'b ParserValue) {
        match *v {
            ParserValue::String(start, end)
            | ParserValue::Object(start, end)
            | ParserValue::Array(start, end)
            | ParserValue::Number(start, end) => {
                let s = str::from_utf8(self.bytes_slice(start, end)).unwrap();
                buffer.push_str(s)
            }

            ParserValue::ArrayString(ref s) | ParserValue::ObjectString(ref s) => {
                buffer.push_str(s)
            }

            ParserValue::Boolean(true) => buffer.push_str("true"),
            ParserValue::Boolean(false) => buffer.push_str("false"),
            ParserValue::NumberUsize(ref u) => buffer.push_str(&u.to_string()),
            ParserValue::Null => buffer.push_str("null"),
            _ => buffer.push_str(""),
        };
    }

    fn parse_value(&self, v: ParserValue) -> Option<Value> {
        let val = match v {
            ParserValue::ArrayString(s) => Value::Array(s),
            ParserValue::ObjectString(s) => Value::Object(s),
            _ => return self.parse_value_borrow(&v),
        };

        Some(val)
    }

    fn parse_value_borrow(&self, v: &ParserValue) -> Option<Value> {
        let val = match *v {
            ParserValue::String(start, end) => {
                let s = unescape::unescape(self.bytes_slice(start + 1, end - 1));
                // let s = String::from_utf8_lossy(self.bytes_slice(start + 1, end - 1)).to_string();
                Value::String(s)
            }
            ParserValue::Object(start, end) => {
                let s = String::from_utf8_lossy(self.bytes_slice(start, end)).to_string();
                Value::Object(s)
            }
            ParserValue::Array(start, end) => {
                let s = String::from_utf8_lossy(self.bytes_slice(start, end)).to_string();
                Value::Array(s)
            }

            ParserValue::ArrayString(_) | ParserValue::ObjectString(_) => {
                panic!("should not borrow string value")
            }

            ParserValue::Number(start, end) => {
                let raw = self.bytes_slice(start, end);
                let n = Number::from(raw);

                Value::Number(n)
            }
            ParserValue::NumberUsize(u) => Value::Number(Number::U64(u.to_string())),
            ParserValue::Boolean(bool) => Value::Boolean(bool),
            ParserValue::Null => Value::Null,
            _ => return None,
        };

        Some(val)
    }

    fn select_to_object(&mut self, sels: &[SubSelector]) -> ParserValue {
        let mut raw = Vec::with_capacity(100);
        let start = self.position();
        raw.push(b'{');

        for sel in sels {
            let path = Path::new_from_utf8(sel.path);
            self.seek(start);
            let sub_pv = self.get_by_path(&path);
            if sub_pv.exists() {
                raw.push(b'"');
                raw.extend(sel.name);
                raw.push(b'"');
                raw.push(b':');
                self.write_to_bytes_buffer(&mut raw, &sub_pv);
                raw.push(b',');
            }
        }

        if raw.len() > 1 {
            raw.pop();
        }
        raw.push(b'}');

        let s = String::from_utf8_lossy(&raw).to_string();
        ParserValue::ObjectString(s)
    }

    fn select_to_array(&mut self, sels: &[SubSelector]) -> ParserValue {
        let mut raw = Vec::with_capacity(100);
        let start = self.position();
        raw.push(b'[');

        for sel in sels {
            let path = Path::new_from_utf8(sel.path);
            self.seek(start);
            let sub_pv = self.get_by_path(&path);
            if sub_pv.exists() {
                self.write_to_bytes_buffer(&mut raw, &sub_pv);
                raw.push(b',');
            }
        }

        if raw.len() > 1 {
            raw.pop();
        }
        raw.push(b']');

        let s = String::from_utf8_lossy(&raw).to_string();
        ParserValue::ArrayString(s)
    }

    fn get_by_path(&mut self, path: &Path) -> ParserValue {
        if !path.ok {
            return ParserValue::NotExist;
        }

        if path.has_selectors() {
            let v = match path.arrsel {
                true => self.select_to_array(path.borrow_selectors()),
                false => self.select_to_object(path.borrow_selectors()),
            };

            return if path.more {
                self.get_from_value(&v, path.borrow_next())
            } else {
                v
            };
        }

        while let Some(b) = self.peek() {
            return match b {
                b'{' => {
                    self.next_byte();
                    self.get_from_object(path)
                }
                b'[' => {
                    self.next_byte();
                    self.get_from_array(path)
                }
                _ => {
                    self.next_byte();
                    continue;
                }
            };
        }

        ParserValue::NotExist
    }

    fn get_from_value(&mut self, value: &ParserValue, path: &Path) -> ParserValue {
        if !path.ok {
            return ParserValue::NotExist;
        }

        match value {
            ParserValue::Array(start, _) | ParserValue::Object(start, _) => {
                let old = self.position();
                self.seek(*start);
                let v = self.get_by_path(path);
                self.seek(old);
                v
            }
            ParserValue::ArrayString(ref s) | ParserValue::ObjectString(ref s) => {
                let mut getter = Getter::from_str(s);
                getter.get_by_path(path)
            }
            _ => ParserValue::NotExist,
        }
    }

    pub fn next_value(&mut self) -> Option<Value> {
        let pv = self.read_next_parse_value();
        self.parse_value(pv)
    }

    fn read_next_parse_value(&mut self) -> ParserValue {
        while let Some(b) = self.peek() {
            let v = match b {
                b'"' => self.read_str_value(),
                b't' | b'f' => self.read_boolean_value(),
                b'n' => self.read_null_value(),
                b'{' | b'[' => self.read_json_value(),
                b'0'...b'9' | b'-' | b'.' => self.read_number_value(),
                b'}' | b']' => ParserValue::NotExist,
                _ => {
                    self.next_byte();
                    continue;
                }
            };

            // println!("read value {:?}", self.parse_value(v));
            return v;
        }

        ParserValue::NotExist
    }

    fn read_boolean_value(&mut self) -> ParserValue {
        let is_true = match self.peek() {
            Some(b't') => true,
            Some(b'f') => false,
            _ => panic!("invalid boolean"),
        };

        self.source.read_boolean_value();

        if is_true {
            ParserValue::Boolean(true)
        } else {
            ParserValue::Boolean(false)
        }
    }

    fn read_null_value(&mut self) -> ParserValue {
        self.source.read_null_value();
        ParserValue::Null
    }

    fn read_json_value(&mut self) -> ParserValue {
        let is_object = match self.peek() {
            Some(b'{') => true,
            Some(b'[') => false,
            _ => panic!("Not JSON"),
        };

        let (start, end) = self.source.read_json_value();

        if is_object {
            ParserValue::Object(start, end)
        } else {
            ParserValue::Array(start, end)
        }
    }

    fn read_str_value(&mut self) -> ParserValue {
        let (start, end) = self.source.read_str_value();
        ParserValue::String(start, end)
    }

    fn read_number_value(&mut self) -> ParserValue {
        let (start, end) = self.source.read_number_value();
        ParserValue::Number(start, end)
    }

    fn get_from_object(&mut self, path: &Path) -> ParserValue {
        // println!(
        //     "get object by path {:?}",
        //     str::from_utf8(path.part).unwrap()
        // );

        let mut count = 0;
        loop {
            let v = self.read_next_parse_value();
            if v == ParserValue::NotExist {
                return v;
            }

            count += 1;
            if count % 2 == 0 {
                continue;
            }

            // check the object key
            if let ParserValue::String(start, end) = v {
                if path.is_match(self.bytes_slice(start + 1, end - 1)) {
                    return if path.more {
                        self.get_by_path(path.borrow_next())
                    } else {
                        self.read_next_parse_value()
                    };
                }
            } else {
                panic!("key must by str");
            }
        }
    }

    fn get_from_array(&mut self, path: &Path) -> ParserValue {
        // println!("get array by path {:?}", path);

        let mut count = 0;
        let (idx, idx_get) = match str::from_utf8(path.part).unwrap().parse::<usize>() {
            Ok(i) => (i, true),
            Err(_) => (0, false),
        };

        let query = path.borrow_query();
        let query_key = query.get_path();

        // println!("path =====> {:?}", path);
        // println!("=> query {:?} {}", query, query.has_path());
        let mut vector_str = String::new();
        let return_vector = (query.on && query.all) || (!query.on && path.more);
        let return_first = query.on && !query.all;
        if return_vector {
            vector_str = String::with_capacity(100);
            vector_str.push('[');
        }

        loop {
            if idx_get && idx == count {
                if path.more {
                    return self.get_by_path(path.borrow_next());
                }
                return self.read_next_parse_value();
            }

            let mut v = self.read_next_parse_value();
            if !v.exists() {
                break;
            }

            // do query match
            // println!("{:?}", self.value_to_raw_str(&v));
            // println!("more {}", path.more);
            if query.on {
                let value_to_match = match query.has_path() {
                    true => {
                        let v = self.get_from_value(&v, &query_key);
                        self.parse_value_borrow(&v)
                    }
                    false => self.parse_value_borrow(&v),
                };

                if value_to_match.is_none() {
                    continue;
                }

                if !query.is_match(value_to_match.as_ref().unwrap()) {
                    continue;
                }
            }

            count += 1;

            if path.more {
                v = self.get_from_value(&v, path.borrow_next());
                if !v.exists() {
                    continue;
                }
            }

            if return_first {
                return v;
            }

            if return_vector {
                self.write_to_string_buffer(&mut vector_str, &v);
                vector_str.push(',');
            }
        }

        if return_vector {
            if vector_str.len() > 1 {
                // remove last comma
                vector_str.pop();
            }
            vector_str.push(']');
            ParserValue::ArrayString(vector_str)
        } else if return_first {
            ParserValue::NotExist
        } else if path.arrch {
            ParserValue::NumberUsize(count)
        } else {
            ParserValue::NotExist
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_read_value() {
        let json = r#"{
    "overflow": 9223372036854775808,
    "widget": {
        "debug": "on",
        "window": {
            "title": "Sample Konfabulator Widget",
            "name": "main_window",
            "width": 500,
            "height": 500
        },
        "image": {
            "src": "Images/Sun.png",
            "hOffset": 250,
            "vOffset": 250,
            "alignment": "center"
        },
        "text": {
            "data": "Click Here",
            "size": 36,
            "style": "bold",
            "vOffset": 100,
            "alignment": "center",
            "onMouseUp": "sun1.opacity = (sun1.opacity / 100) * 90;"
        },
        "menu": [
            {
                "title": "file",
                "sub_item": 7,
                "options": [1, 2, 3]
            },
            {
                "title": "edit",
                "sub_item": 14,
                "options": [4, 5]
            },
            {
                "title": "help",
                "sub_item": 4,
                "options": [6, 7, 8]
            }
        ]
    }
}"#;

        let mut g = Getter::new_from_utf8(json.as_bytes());
        println!("_________");
        println!("result {:?}", g.get("widget.window.name"));
        println!("result {:?}", g.get("widget.image.hOffset"));
        println!("result {:?}", g.get("widget.text.onMouseUp"));
        println!("result {:?}", g.get("widget.debug"));

        println!("result {:?}", g.get("widget.window.name"));
        println!("result {:?}", g.get("widget.image.hOffset"));
        println!("result {:?}", g.get("widget.text.onMouseUp"));
        println!("result {:?}", g.get("widget.debug"));

        println!("result {:?}", g.get("widget.menu.0"));
        println!("result {:?}", g.get("widget.menu.#(sub_item>=7)#.title"));
        println!("result {:?}", g.get("widget.menu"));
        println!("result {:?}", g.get("widget.menu.#"));
    }
}
