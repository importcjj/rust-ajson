#[macro_use]
extern crate criterion;

use criterion::{black_box, Criterion};
extern crate ajson;
extern crate json;

#[macro_use]
extern crate serde;
extern crate nom;
extern crate serde_json;
mod nom_json;
use json::JsonValue;
use nom::error::ErrorKind;
use serde_json::Value;

#[allow(dead_code)]
static BENCH_DATA: &str = r#"{
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

fn ajson_selector(json: &str) {
    black_box(
        ajson::get(json, "widget.[image.src,text.data]")
            .unwrap()
            .unwrap()
            .as_vec(),
    );
}

fn ajson_multi_query(json: &str) {
    black_box([
        ajson::get(json, "widget.image.src"),
        ajson::get(json, "widget.text.data"),
    ]);
}

fn ajson_path_bench() {
    black_box(ajson::Path::from_slice("widget.window.name".as_bytes()));
    black_box(ajson::Path::from_slice("widget.image.hOffset".as_bytes()));
    black_box(ajson::Path::from_slice("widget.text.onMouseUp".as_bytes()));
    black_box(ajson::Path::from_slice("widget.debug".as_bytes()));
    black_box(ajson::Path::from_slice(
        "widget.menu.#(sub_item>7)#.title".as_bytes(),
    ));
}

fn ajson_bench(json: &str) {
    black_box(
        ajson::get(json, "widget.window.name")
            .unwrap()
            .unwrap()
            .as_str(),
    );
    black_box(
        ajson::get(json, "widget.image.hOffset")
            .unwrap()
            .unwrap()
            .as_f64(),
    );
    black_box(
        ajson::get(json, "widget.text.onMouseUp")
            .unwrap()
            .unwrap()
            .as_str(),
    );
    black_box(ajson::get(json, "widget.debug").unwrap().unwrap().as_str());
    // black_box(ajson::get(json, "widget.text").unwrap().unwrap().as_object());

    black_box(
        ajson::get(json, "widget.menu.#(sub_item>7)#.title")
            .unwrap()
            .unwrap()
            .as_vec(),
    );
    // ajson::get(json, "widget.menu.[1.title,2.options]").as_array();
}

fn gjson_selector(json: &str) {
    black_box(gjson::get(json, "widget.[image.src,text.data]").array());
}

fn gjson_multi_query(json: &str) {
    black_box([
        gjson::get(json, "widget.image.src"),
        gjson::get(json, "widget.text.data"),
    ]);
}

fn gjson_bench(json: &str) {
    black_box(gjson::get(json, "widget.window.name").str());
    black_box(gjson::get(json, "widget.image.hOffset").f64());
    black_box(gjson::get(json, "widget.text.onMouseUp").str());
    black_box(gjson::get(json, "widget.debug").str());

    black_box(gjson::get(json, "widget.menu.#(sub_item>7)#.title").array());
    // gjson::get(json, "widget.menu.[1.title,2.options]").as_array();
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
    let _v: Vec<&JsonValue> = black_box(
        menu.members()
            .filter(|x| x["sub_item"].as_i64().unwrap() > 5)
            .map(|x| &x["title"])
            .collect(),
    );
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

    let _v: Vec<&Value> = black_box(
        menu["widget"]["menu"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| x["sub_item"].as_i64().unwrap() > 5)
            .map(|x| &x["title"])
            .collect(),
    );
}

fn serde_json_derive_bench(json: &str) {
    #![allow(non_snake_case)]

    {
        #[derive(Deserialize)]
        struct Main {
            widget: Widget,
        }
        #[derive(Deserialize)]
        struct Widget {
            window: Window,
        }
        #[derive(Deserialize)]
        struct Window {
            name: String,
        }

        let a = serde_json::from_str::<Main>(json).unwrap();
        black_box(a.widget.window.name);
    }

    {
        #[derive(Deserialize)]
        struct Main {
            widget: Widget,
        }
        #[derive(Deserialize)]
        struct Widget {
            image: Image,
        }
        #[derive(Deserialize)]
        struct Image {
            hOffset: i64,
        }
        let b = serde_json::from_str::<Main>(json).unwrap();
        black_box(b.widget.image.hOffset);
    }

    {
        #[derive(Deserialize)]
        struct Main {
            widget: Widget,
        }
        #[derive(Deserialize)]
        struct Widget {
            text: Text,
        }
        #[derive(Deserialize)]
        struct Text {
            onMouseUp: String,
        }
        let c = serde_json::from_str::<Main>(json).unwrap();
        black_box(c.widget.text.onMouseUp);
    }

    {
        #[derive(Deserialize)]
        struct Main {
            widget: Widget,
        }
        #[derive(Deserialize)]
        struct Widget {
            debug: String,
        }
        let d = serde_json::from_str::<Main>(json).unwrap();
        black_box(d.widget.debug);
    }

    {
        #[derive(Deserialize)]
        struct Main {
            widget: Widget,
        }
        #[derive(Deserialize)]
        struct Widget {
            menu: Vec<Item>,
        }
        #[derive(Deserialize)]
        struct Item {
            sub_item: i64,
            title:    Value,
        }
        let e = serde_json::from_str::<Main>(json).unwrap();
        black_box(
            e.widget
                .menu
                .into_iter()
                .filter(|x| x.sub_item > 5)
                .map(|x| x.title)
                .collect::<Vec<_>>(),
        );
    }
}

fn nom_json_bench(json: &str) {
    black_box({
        nom_json::root::<(&str, ErrorKind)>(json).map(|(_, value)| {
            value["widget"]["window"]["name"].as_str();
        });
    });

    black_box({
        nom_json::root::<(&str, ErrorKind)>(json).map(|(_, value)| {
            &value["widget"]["image"]["hOffset"];
        });
    });

    black_box({
        nom_json::root::<(&str, ErrorKind)>(json).map(|(_, value)| {
            value["widget"]["text"]["onMouseUp"].as_str();
        });
    });

    black_box({
        nom_json::root::<(&str, ErrorKind)>(json).map(|(_, value)| {
            value["widget"]["debug"].as_str();
        });
    });

    black_box({
        nom_json::root::<(&str, ErrorKind)>(json).map(|(_, value)| {
            let menu = &value["widget"]["menu"];
            let v: Vec<&nom_json::JsonValue> = black_box(
                menu.members()
                    .filter(|x| x["sub_item"].to_f64() > 5.0)
                    .map(|x| &x["title"])
                    .collect(),
            );
            1
        })
    });
}

fn serde_json_derive_multi_query(json: &str) {
    #[derive(Deserialize)]
    struct Main {
        widget: Widget,
    }
    #[derive(Deserialize)]
    struct Widget {
        image: Image,
        text:  Text,
    }
    #[derive(Deserialize)]
    struct Image {
        src: Value,
    }
    #[derive(Deserialize)]
    struct Text {
        data: Value,
    }

    let a = serde_json::from_str::<Main>(json).unwrap();
    black_box([a.widget.image.src, a.widget.text.data]);
}

fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));

    // c.bench_function("ajson path benchmark", |b| b.iter(ajson_path_bench));

    c.bench_function("ajson benchmark", |b| {
        b.iter(|| ajson_bench(black_box(BENCH_DATA)))
    });

    c.bench_function("gjson benchmark", |b| {
        b.iter(|| gjson_bench(black_box(BENCH_DATA)))
    });

    // c.bench_function("serde_json benchmark", |b| {
    //     b.iter(|| serde_json_bench(black_box(BENCH_DATA)))
    // });

    // c.bench_function("json-rust benchmark", |b| {
    //     b.iter(|| json_rust_bench(black_box(BENCH_DATA)))
    // });

    // c.bench_function("ajson selector", |b| {
    //     b.iter(|| ajson_selector(black_box(BENCH_DATA)))
    // });

    // c.bench_function("gjson selector", |b| {
    //     b.iter(|| gjson_selector(black_box(BENCH_DATA)))
    // });

    // c.bench_function("ajson multi query", |b| {
    //     b.iter(|| ajson_multi_query(black_box(BENCH_DATA)))
    // });

    // c.bench_function("gjson multi query", |b| {
    //     b.iter(|| gjson_multi_query(black_box(BENCH_DATA)))
    // });

    // c.bench_function("serde derive", |b| {
    //     b.iter(|| serde_json_derive_bench(black_box(BENCH_DATA)))
    // });
    // c.bench_function("serde derive multi query", |b| {
    //     b.iter(|| serde_json_derive_multi_query(black_box(BENCH_DATA)))
    // });

    // c.bench_function("nom json bench", |b| {
    //     b.iter(|| nom_json_bench(black_box(BENCH_DATA)))
    // });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
