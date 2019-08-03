#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;
extern crate ajson;
extern crate json;

#[macro_use]
extern crate serde;
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

fn ajson_selector(json: &str) {
    black_box(ajson::get(json, "widget.[image.src,text.data]").as_array());
}

fn ajson_multi_query(json: &str) {
    black_box([
        ajson::get(json, "widget.image.src"),
        ajson::get(json, "widget.text.data"),
    ]);
}

fn ajson_bench(json: &str) {
    black_box(ajson::get(json, "widget.window.name").as_str());
    black_box(ajson::get(json, "widget.image.hOffset").as_f64());
    black_box(ajson::get(json, "widget.text.onMouseUp").as_str());
    black_box(ajson::get(json, "widget.debug").as_str());
    // ajson::get(json, "widget.text").as_map();
    black_box(ajson::get(json, "widget.menu.#(sub_item>7)#.title").as_array());
    // ajson::get(json, "widget.menu.[1.title,2.options]").as_array();
}

fn json_rust_bench(data: &str) {
    let a = &json::parse(data).unwrap();
    black_box(a["widget"]["window"]["name"].as_str().unwrap());
    let b = &json::parse(data).unwrap();
    black_box(b["widget"]["image"]["hOffset"].as_i64().unwrap());
    let c = &json::parse(data).unwrap();
    black_box(c["widget"]["text"]["onMouseUp"].as_str().unwrap());
    let d = &json::parse(data).unwrap();
    black_box(d["widget"]["debug"].as_str().unwrap());

    // let text = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["text"] ;

    let menu = &json::parse(data).unwrap()["widget"]["menu"];
    let _v: Vec<&JsonValue> = black_box(menu
        .members()
        .filter(|x| x["sub_item"].as_i64().unwrap() > 5)
        .map(|x| &x["title"])
        .collect());
}

fn serde_json_bench(json: &str) {
    let a = &serde_json::from_str::<Value>(json).unwrap();
    black_box(a["widget"]["window"]["name"].as_str().unwrap());
    let b = &serde_json::from_str::<Value>(json).unwrap();
    black_box(b["widget"]["image"]["hOffset"].as_i64().unwrap());
    let c = &serde_json::from_str::<Value>(json).unwrap();
    black_box(c["widget"]["text"]["onMouseUp"].as_str().unwrap());
    let d = &serde_json::from_str::<Value>(json).unwrap();
    black_box(d["widget"]["debug"].as_str().unwrap());

    // // let text = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["text"] ;

    let menu = &serde_json::from_str::<Value>(json).unwrap();

    let _v: Vec<&Value> = black_box(menu["widget"]["menu"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["sub_item"].as_i64().unwrap() > 5)
        .map(|x| &x["title"])
        .collect());
}

fn serde_json_derive_bench(json: &str) {
    #![allow(non_snake_case)]

    {
        #[derive(Deserialize)] struct Main { widget: Widget }
        #[derive(Deserialize)] struct Widget { window: Window }
        #[derive(Deserialize)] struct Window { name: String }

        let a = serde_json::from_str::<Main>(json).unwrap();
        black_box(a.widget.window.name);
    }

    {
        #[derive(Deserialize)] struct Main { widget: Widget }
        #[derive(Deserialize)] struct Widget { image: Image }
        #[derive(Deserialize)] struct Image { hOffset: i64 }
        let b = serde_json::from_str::<Main>(json).unwrap();
        black_box(b.widget.image.hOffset);
    }

    {
        #[derive(Deserialize)] struct Main { widget: Widget }
        #[derive(Deserialize)] struct Widget { text: Text }
        #[derive(Deserialize)] struct Text { onMouseUp: String }
        let c = serde_json::from_str::<Main>(json).unwrap();
        black_box(c.widget.text.onMouseUp);
    }

    {
        #[derive(Deserialize)] struct Main { widget: Widget }
        #[derive(Deserialize)] struct Widget { debug: String }
        let d = serde_json::from_str::<Main>(json).unwrap();
        black_box(d.widget.debug);
    }

    {
        #[derive(Deserialize)] struct Main { widget: Widget }
        #[derive(Deserialize)] struct Widget { menu: Vec<Item> }
        #[derive(Deserialize)] struct Item {
            sub_item: i64,
            title: Value,
        }
        let e = serde_json::from_str::<Main>(json).unwrap();
        black_box(e.widget.menu.into_iter()
            .filter(|x| x.sub_item > 5)
            .map(|x| x.title)
            .collect::<Vec<_>>()
        );
    }
}

fn serde_json_derive_multi_query(json: &str) {
    #[derive(Deserialize)] struct Main { widget: Widget }
    #[derive(Deserialize)] struct Widget {
        image: Image,
        text: Text,
    }
    #[derive(Deserialize)] struct Image { src: Value }
    #[derive(Deserialize)] struct Text { data: Value }

    let a = serde_json::from_str::<Main>(json).unwrap();
    black_box([
        a.widget.image.src,
        a.widget.text.data,
    ]);
}

fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
    c.bench_function("ajson benchmark", |b| {
        b.iter(|| ajson_bench(black_box(BENCH_DATA)))
    });
    c.bench_function("serde_json benchmark", |b| {
        b.iter(|| serde_json_bench(black_box(BENCH_DATA)))
    });
    c.bench_function("json-rust benchmark", |b| {
        b.iter(|| json_rust_bench(black_box(BENCH_DATA)))
    });
    c.bench_function("ajson selector", |b| {
        b.iter(|| ajson_selector(black_box(BENCH_DATA)))
    });
    c.bench_function("ajson multi query", |b| {
        b.iter(|| ajson_multi_query(black_box(BENCH_DATA)))
    });
    c.bench_function("serde derive", |b| {
        b.iter(|| serde_json_derive_bench(black_box(BENCH_DATA)))
    });
    c.bench_function("serde derive multi query", |b| {
        b.iter(|| serde_json_derive_multi_query(black_box(BENCH_DATA)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
