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

pub fn string(input: &str) -> ajson::Result<(&str, &str)> {
    // skip check the first byte

    let mut i = 1;
    let bytes = input.as_bytes();
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

    Ok(input.split_at(i))
}

pub fn string_chunk(input: &str) -> ajson::Result<(&str, &str)> {
    // skip check the first byte

    let mut i = 1;
    let bytes = input.as_bytes();
    const CHUNK: usize = 4;

    'outer: while i + CHUNK < bytes.len() {
        for _ in 0..CHUNK {
            let &b = unsafe { bytes.get_unchecked(i) };
            i += 1;
            match b {
                b'"' => {
                    return Ok(input.split_at(i));
                }
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

    Ok(input.split_at(i))
}

fn traversing(json: &str) {
    let bytes = json.as_bytes();
    let mut i = 0;
    let mut depth = 1;

    while i < bytes.len() {
        let &b = unsafe { bytes.get_unchecked(i) };
        match b {
            b'\\' => {
                i += 1;
            }
            b'"' => {
                let input = unsafe { json.get_unchecked(i..) };
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

    // println!("{}", &json[..i]);
}

fn chunk_traversing(json: &str) {
    let bytes = json.as_bytes();
    let mut i = 0;
    let mut depth = 1;

    const CHUNK_SIZE: usize = 8;

    'outer: while i + CHUNK_SIZE < bytes.len() {
        for _ in 0..CHUNK_SIZE {
            let &b = unsafe { bytes.get_unchecked(i) };
            match b {
                b'\\' => {
                    i += 2;
                    continue 'outer;
                }
                b'"' => {
                    let input = unsafe { json.get_unchecked(i..) };
                    let (s, _) = string(input).unwrap();

                    i += s.len();
                    continue 'outer;
                }
                b'[' | b'{' => depth += 1,
                b']' | b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        return;
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
                let input = unsafe { json.get_unchecked(i..) };
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

}

pub fn traverse_benchmark(c: &mut Criterion) {
    c.bench_function("traversing", |b| {
        b.iter(|| traversing(black_box(BENCH_DATA)))
    });
    c.bench_function("chunk traversing", |b| {
        b.iter(|| chunk_traversing(black_box(BENCH_DATA)))
    });
}

criterion_group!(benches, traverse_benchmark);
criterion_main!(benches);
