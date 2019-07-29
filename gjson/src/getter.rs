use path::Path;
use reader;
use std::str;
use value;
use std::io;

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
    Vector(String),
    Null,
    Boolean(bool),
    Number(usize, usize),
    NumberUsize(usize),
    NotExist,
}

impl ParserValue {
    pub fn exists(&self) -> bool {
        *self != ParserValue::NotExist
    }

    pub fn is_vector(&self) -> bool {
        if let ParserValue::Vector(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn vector_to_value(self) -> value::Value {
        if let ParserValue::Vector(s) = self {
            value::Value::Array(s, None)
        } else {
            value::Value::NotExists
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
}

impl<R> Getter<R>
where
    R: reader::ByteReader,
{
    pub fn new(r: R) -> Getter<R> {
        Getter { source: r }
    }

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

    pub fn get(&mut self, path: &str) -> value::Value {
        // reset offset
        self.seek(0);
        // let path = Path::new_from_utf8(path.as_bytes());
        let path = Path::new_from_utf8(path.as_bytes());
        // println!("{:?}", path);

        let v = self.get_by_path(&path);
        if v.is_vector() {
            v.vector_to_value()
        } else {
            self.parse_value(&v)
        }
    }

    fn value_to_raw_str<'b>(&'b mut self, v: &'b ParserValue) -> &'b str {
        match *v {
            ParserValue::String(start, end)
            | ParserValue::Object(start, end)
            | ParserValue::Array(start, end)
            | ParserValue::Number(start, end) => {
                str::from_utf8(self.bytes_slice(start, end)).unwrap()
            }
            ParserValue::Vector(ref s) => s,
            ParserValue::Boolean(true) => "true",
            ParserValue::Boolean(false) => "false",
            // ParserValue::NumberUsize(u) => &u.to_string(),
            ParserValue::Null => "null",
            _ => "",
        }
    }

    fn parse_value(&mut self, v: &ParserValue) -> value::Value {
        match *v {
            ParserValue::String(start, end) => {
                let s = String::from_utf8_lossy(self.bytes_slice(start + 1, end - 1)).to_string();
                value::Value::String(s)
            }
            ParserValue::Object(start, end) => {
                let s = String::from_utf8_lossy(self.bytes_slice(start, end)).to_string();
                value::Value::Object(s, None)
            }
            ParserValue::Array(start, end) => {
                let s = String::from_utf8_lossy(self.bytes_slice(start, end)).to_string();
                value::Value::Array(s, None)
            }
            // ParserValue::Vector(ref string) => value::Value::Array(string.clone(), None),
            ParserValue::Number(start, end) => {
                let f: f64 = str::from_utf8(self.bytes_slice(start, end))
                    .unwrap()
                    .parse()
                    .unwrap();
                value::Value::Number(f)
            }
            ParserValue::NumberUsize(u) => value::Value::Number(u as f64),
            ParserValue::Boolean(bool) => value::Value::Boolean(bool),
            ParserValue::Null => value::Value::Null,
            _ => value::Value::NotExists,
        }
    }

    fn get_by_path(&mut self, path: &Path) -> ParserValue {
        if !path.ok {
            return self.read_next_value();
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
        match value {
            ParserValue::Array(start, _) | ParserValue::Object(start, _) => {
                let old = self.position();
                self.seek(*start);
                let v = self.get_by_path(path);
                self.seek(old);
                v
            }
            _ => ParserValue::NotExist,
        }
    }

    fn read_next_value(&mut self) -> ParserValue {
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

            // println!("read value {:?}", self.parse_value(&v));
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
        // println!("get object by path {:?}", path);
        // println!("{:?}",str::from_utf8(path.part) );
        let mut count = 0;
        loop {
            let v = self.read_next_value();
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
                        self.read_next_value()
                    };
                }
            } else {
                panic!("key must by str");
            }
        }
    }

    fn get_from_array(&mut self, path: &Path) -> ParserValue {
        // println!("get array by path {:?}", path);
        // println!("{:?}",str::from_utf8(path.part) );
        let mut count = 0;
        let (idx, idx_get) = match str::from_utf8(path.part).unwrap().parse::<usize>() {
            Ok(i) => (i, true),
            Err(_) => (0, false),
        };

        let query = path.borrow_query();
        let query_key = query.get_key();
        let mut vector_str = String::new();
        let return_vector = (query.on && query.all) || (!query.on && path.more);
        if return_vector {
            vector_str = String::with_capacity(100);
            vector_str.push('[');
        }

        loop {
            if idx_get && idx == count {
                return self.get_by_path(path.borrow_next());
            }

            let mut v = self.read_next_value();
            if !v.exists() {
                break;
            }

            // do query match
            if query.on {
                let value_to_match = match query.has_key() {
                    true => {
                        let v = self.get_from_value(&v, &query_key);
                        self.parse_value(&v)
                    }
                    false => self.parse_value(&v),
                };
                if !query.is_match(&value_to_match) {
                    continue;
                }
            }

            count += 1;

            if path.more {
                v = self.get_from_value(&v, path.borrow_next());

            }

            if query.on && !query.all {
                return v;
            }

            if return_vector {
                vector_str.push_str(self.value_to_raw_str(&v));
                vector_str.push(',');
            }
        }

        if return_vector {
            if vector_str.len() > 1 {
                // remove last comma
                vector_str.pop();
            }
            vector_str.push(']');
            ParserValue::Vector(vector_str)
        } else if idx_get {
            ParserValue::NotExist
        } else {
            ParserValue::NumberUsize(count)
        }
    }
}

pub fn get(json: &str, path: &str) -> value::Value {
    Getter::new_from_utf8(json.as_bytes()).get(path)
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

        println!("result {:?}", get(json, "widget.window.name"));
        println!("result {:?}", get(json, "widget.image.hOffset"));
        println!("result {:?}", get(json, "widget.text.onMouseUp"));
        println!("result {:?}", get(json, "widget.debug"));

        println!("result {:?}", g.get("widget.menu.0"));
        println!("result {:?}", g.get("widget.menu.#(sub_item>=7)#.title"));
        println!("result {:?}", g.get("widget.menu"));
        println!("result {:?}", g.get("widget.menu.#"));
    }
}
