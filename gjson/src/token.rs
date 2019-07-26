use std::collections::HashMap;
use value::Value;
use get_from_reader;
use path::Path;
use parser::Parser;
use std::str;

#[derive(PartialEq, Debug)]
pub enum Token<'a> {
    String(Vec<u8>),
    Object(&'a [u8], Option<HashMap<String, Value>>),
    Array(&'a [u8], Option<Vec<Value>>),
    Boolean(bool),
    Number(&'a [u8], Option<f64>),
    Null,
    NotExists,
}


impl<'a> Token<'a> {
    pub fn get_path<'b>(&'a self, path: &Path) -> Token<'b> {
        match self {
            Token::Object(ref raw, _) | Token::Array(ref raw, _) => {
                let mut reader = Parser::new(*raw);
                get_from_reader(&mut reader, path)
            }
            _ => Token::NotExists
        }
    }

    pub fn to_value(self) -> Value {
        match self {
            Token::String(raw) => {
                Value::String(String::from_utf8_lossy(&raw).to_string())
            },
            Token::Number(raw, None) => {
                let f = str::from_utf8(raw).unwrap().parse::<f64>().unwrap();
                Value::Number(f)
            }
            Token::Number(_, Some(f)) => {
                Value::Number(f)
            }
            Token::Object(raw, None) => {
                Value::Object(String::from_utf8_lossy(raw).to_string(), None)
            }
            Token::Object(_, a) => {
                Value::Object("".to_owned(), a)
            }
            Token::Array(raw, None) => {
                Value::Array(String::from_utf8_lossy(raw).to_string(), None)
            }
            Token::Array(_, a) => {
                Value::Array("".to_owned(), a)
            }
            Token::Null => Value::Null,
            _ => Value::NotExists,
        }
    }

    pub fn exists(&self) -> bool {
        *self != Token::NotExists
    }
}