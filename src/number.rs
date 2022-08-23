use reader::Bytes;
use std::convert::From;
const MIN_UINT_53: u64 = 0;
const MAX_UINT_53: u64 = 4503599627370495;
const MIN_INT_53: i64 = -2251799813685248;
const MAX_INT_53: i64 = 2251799813685247;
const ZERO_UINT: u64 = 0;
const ZERO_INT: i64 = 0;
const ZERO_FLOAT: f64 = 0.0;
const ZERO_FLOAT_F32: f32 = 0.0;

#[allow(dead_code)]
const ZERO_INT_I32: i32 = 0;
#[allow(dead_code)]
const ZERO_INT_U32: u32 = 0;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Number<'a> {
    F64(&'a str),
    U64(&'a str),
    I64(&'a str),
}

impl<'a> From<&'a [u8]> for Number<'a> {
    fn from(v: &[u8]) -> Number {
        let mut reader = Bytes::new(v);
        Number::from(&mut reader)
    }
}

impl<'a> From<&'a str> for Number<'a> {
    fn from(s: &str) -> Number {
        Number::from(s.as_bytes())
    }
}

impl<'a> From<&mut Bytes<'a>> for Number<'a> {
    fn from(r: &mut Bytes<'a>) -> Number<'a> {
        let start = r.position();
        let sign = match r.peek() {
            Some(b'-') => true,
            None => panic!("invalid number"),
            _ => false,
        };

        let mut float = false;

        while let Some(b) = r.next() {
            match b {
                b'0'..=b'9' => (),
                b'.' => float = true,
                _ => {
                    break;
                }
            };
        }
        let end = r.position() - 1;

        let s = unsafe { std::str::from_utf8_unchecked(r.slice(start, end)) };
        if float {
            Number::F64(s)
        } else if sign {
            Number::I64(s)
        } else {
            Number::U64(s)
        }
    }
}

impl<'a> Number<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            Number::F64(s) => s,
            Number::U64(s) => s,
            Number::I64(s) => s,
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            Number::F64(s) => s.parse().unwrap_or(ZERO_FLOAT),
            Number::U64(s) => s.parse().unwrap_or(ZERO_FLOAT),
            Number::I64(s) => s.parse().unwrap_or(ZERO_FLOAT),
        }
    }

    pub fn to_f32(&self) -> f32 {
        match self {
            Number::F64(s) => s.parse().unwrap_or(ZERO_FLOAT_F32),
            Number::U64(s) => s.parse().unwrap_or(ZERO_FLOAT_F32),
            Number::I64(s) => s.parse().unwrap_or(ZERO_FLOAT_F32),
        }
    }

    pub fn to_u64(&self) -> u64 {
        match self {
            Number::F64(s) => {
                f64_to_u64(self.to_f64()).unwrap_or_else(|| parse_uint_lossy(s.as_bytes()))
            }
            Number::I64(s) => s.parse().unwrap_or(ZERO_UINT),
            Number::U64(s) => s.parse().unwrap_or(ZERO_UINT),
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            Number::F64(s) => {
                f64_to_i64(self.to_f64()).unwrap_or_else(|| parse_int_lossy(s.as_bytes()))
            }
            Number::I64(s) => s.parse().unwrap_or(ZERO_INT),
            Number::U64(s) => s.parse().unwrap_or(ZERO_INT),
        }
    }
}

fn f64_to_u64(f: f64) -> Option<u64> {
    let u = f as u64;
    match u {
        MIN_UINT_53..=MAX_UINT_53 => Some(u),
        _ => None,
    }
}

fn f64_to_i64(f: f64) -> Option<i64> {
    let i = f as i64;
    match i {
        MIN_INT_53..=MAX_INT_53 => Some(i),
        _ => None,
    }
}

pub fn parse_uint_lossy(v: &[u8]) -> u64 {
    let mut acc: u64 = 0;
    for b in v {
        match b {
            b'0'..=b'9' => acc = acc * 10 + (*b - 48) as u64,
            _ => return acc,
        }
    }

    acc
}

pub fn parse_int_lossy(v: &[u8]) -> i64 {
    if v.is_empty() {
        return ZERO_INT;
    }

    let sign = v[0] == b'-';
    let mut acc: i64 = 0;

    for b in v {
        match b {
            b'0'..=b'9' => match sign {
                true => acc = acc * 10 - (*b - 48) as i64,
                false => acc = acc * 10 + (*b - 48) as i64,
            },
            _ => return acc,
        }
    }

    acc
}
