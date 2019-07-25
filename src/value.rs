use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub enum Value {
    String(String),
    Number(f64),
    Object(String, Option<HashMap<String, Value>>),
    Array(String, Option<Vec<Value>>),
    Boolean(bool),
    Null,
    NotExists,
}

