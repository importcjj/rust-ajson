#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;
extern crate rjson;
extern crate json;
extern crate serde_json;

use json::JsonValue;
use serde_json::Value;

#[allow(dead_code)]
static BENCH_DATA: &'static str = r#"{
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

// fn fibonacci(n: u64) -> u64 {
//     match n {
//         0 => 1,
//         1 => 1,
//         n => fibonacci(n-1) + fibonacci(n-2),
//     }
// }

fn rjson_selector(json: &str) {
    rjson::get(json, "widget.[image.src,text.data]").as_array();
}

fn rjson_multi_query(json: &str) {
    [
        rjson::get(json, "widget.image.src"),
        rjson::get(json, "widget.text.data"),
    ];
}

fn rjson_bench(json: &str) {
    rjson::get(json, "widget.window.name").as_str();
    rjson::get(json, "widget.image.hOffset").as_f64();
    rjson::get(json, "widget.text.onMouseUp").as_str();
    rjson::get(json, "widget.debug").as_str();
    // rjson::get(json, "widget.text").as_map();
    rjson::get(json, "widget.menu.#(sub_item>7)#.title").as_array();
    // rjson::get(json, "widget.menu.[1.title,2.options]").as_array();
}

fn json_rust_bench(data: &str) {
    let a = &json::parse(data).unwrap();
    a["widget"]["window"]["name"].as_str().unwrap();
    let b = &json::parse(data).unwrap();
    b["widget"]["image"]["hOffset"].as_i64().unwrap();
    let c = &json::parse(data).unwrap();
    c["widget"]["text"]["onMouseUp"].as_str().unwrap();
    let d = &json::parse(data).unwrap();
    d["widget"]["debug"].as_str().unwrap();

    // let text = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["text"] ;

    let menu = &json::parse(data).unwrap()["widget"]["menu"];
    let _v: Vec<&JsonValue> = menu
        .members()
        .filter(|x| x["sub_item"].as_i64().unwrap() > 5)
        .map(|x| &x["title"])
        .collect();
}

fn serde_json_bench(json: &str) {
    let a = &serde_json::from_str::<Value>(json).unwrap();
    a["widget"]["window"]["name"].as_str().unwrap();
    let b = &serde_json::from_str::<Value>(json).unwrap();
    b["widget"]["image"]["hOffset"].as_i64().unwrap();
    let c = &serde_json::from_str::<Value>(json).unwrap();
    c["widget"]["text"]["onMouseUp"].as_str().unwrap();
    let d = &serde_json::from_str::<Value>(json).unwrap();
    d["widget"]["debug"].as_str().unwrap();

    // // let text = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["text"] ;

    let menu = &serde_json::from_str::<Value>(BENCH_DATA).unwrap();

    let _v: Vec<&Value> = menu["widget"]["menu"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["sub_item"].as_i64().unwrap() > 5)
        .map(|x| &x["title"])
        .collect();
}

fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
    c.bench_function("rjson benchmark", |b| {
        b.iter(|| rjson_bench(black_box(BENCH_DATA)))
    });
    c.bench_function("serde_json benchmark", |b| {
        b.iter(|| serde_json_bench(black_box(BENCH_DATA)))
    });
    c.bench_function("json-rust benchmark", |b| {
        b.iter(|| json_rust_bench(black_box(BENCH_DATA)))
    });
    c.bench_function("rjson selector", |b| {
        b.iter(|| rjson_selector(black_box(BENCH_DATA)))
    });
    c.bench_function("rjson multi query", |b| {
        b.iter(|| rjson_multi_query(black_box(BENCH_DATA)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
