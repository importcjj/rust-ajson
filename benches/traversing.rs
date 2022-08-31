#[macro_use]
extern crate criterion;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const BENCH_DATA: &str = r#"{
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

pub fn string(bytes: &[u8]) -> ajson::Result<(&[u8], &[u8])> {
    // skip check the first byte

    let mut i = 1;
    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };

        match b {
            b'"' => {
                i += 1;
                break;
            }
            b'\\' => {
                i += 1;
            }
            _ => {}
        }

        i += 1;
    }

    Ok(split_at(bytes, i))
}

#[inline(always)]
pub fn split_at(s: &[u8], mid: usize) -> (&[u8], &[u8]) {
    unsafe { (s.get_unchecked(..mid), s.get_unchecked(mid..s.len())) }
}

pub fn string_chunk(bytes: &[u8]) -> ajson::Result<(&[u8], &[u8])> {
    // skip check the first byte

    let mut i = 1;
    const CHUNK: usize = 4;

    'outer: while i + CHUNK < bytes.len() {
        for _ in 0..CHUNK {
            let &b = unsafe { bytes.get_unchecked(i) };
            i += 1;
            match b {
                b'"' => return Ok(split_at(bytes, i)),
                b'\\' => {
                    i += 1;
                    continue 'outer;
                }
                _ => {}
            }
        }
    }

    while i < bytes.len() {
        let b = unsafe { *bytes.get_unchecked(i) };

        match b {
            b'"' => {
                i += 1;
                break;
            }
            b'\\' => {
                i += 1;
            }
            _ => {}
        }

        i += 1;
    }

    Ok(split_at(bytes, i))
}

fn traversing(bytes: &[u8]) -> ajson::Result<(&[u8], &[u8])> {
    let mut i = 0;
    let mut depth = 1;

    while i < bytes.len() {
        let &b = unsafe { bytes.get_unchecked(i) };
        match b {
            b'\\' => {
                i += 1;
            }
            b'"' => {
                let input = unsafe { bytes.get_unchecked(i..) };
                let (s, _) = string(input).unwrap();
                i += s.len();
                continue;
            }
            b'[' | b'{' => depth += 1,
            b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
            }
            _ => (),
        }
        i += 1;
    }

    return Ok(split_at(bytes, i));

    // println!("{}", &json[..i]);
}

fn chunk_traversing(bytes: &[u8]) -> ajson::Result<(&[u8], &[u8])> {
    let mut i = 1;
    let mut depth = 1;

    const CHUNK_SIZE: usize = 32;

    'outer: while i + CHUNK_SIZE < bytes.len() {
        for _ in 0..CHUNK_SIZE {
            let &b = unsafe { bytes.get_unchecked(i) };

            match b {
                b'\\' => {
                    i += 2;
                    continue 'outer;
                }
                b'"' => {
                    let input = unsafe { bytes.get_unchecked(i..) };
                    let (s, _) = string(input).unwrap();

                    i += s.len();
                    continue 'outer;
                }
                b'[' | b'{' => depth += 1,
                b']' | b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        return Ok(split_at(bytes, i));
                    }
                }
                _ => (),
            }
            i += 1;
        }
    }

    while i < bytes.len() {
        let &b = unsafe { bytes.get_unchecked(i) };
        match b {
            b'\\' => {
                i += 1;
            }
            b'"' => {
                let input = unsafe { bytes.get_unchecked(i..) };
                let (s, _) = string(input).unwrap();
                i += s.len();
                continue;
            }
            b'[' | b'{' => depth += 1,
            b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
            }
            _ => (),
        }
        i += 1;
    }

    return Ok(split_at(bytes, i));
}

pub fn traverse_benchmark(c: &mut Criterion) {
    c.bench_function("traversing", |b| {
        b.iter(|| traversing(black_box(BENCH_DATA.as_bytes())))
    });

    c.bench_function("chunk traversing u8", |b| {
        b.iter(|| ajson::compound_u8(black_box(BENCH_DATA.as_bytes())))
    });
}

criterion_group!(benches, traverse_benchmark);
criterion_main!(benches);
