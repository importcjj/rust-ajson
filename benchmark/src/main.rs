#![feature(test)]
extern crate test;

#[allow(unused_imports)]
extern crate gjson;
extern crate json;
extern crate serde_json;

#[allow(unused_imports)]
use json::JsonValue;
#[allow(unused_imports)]
use serde_json::Value;
#[allow(unused_imports)]
use test::Bencher;

use std::collections::HashMap;

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

#[test]
fn json_rs() {
    let parsed = json::parse(BENCH_DATA).unwrap();
    println!("{:?}", parsed["widget"]["window"]["name"]);

    let menu = &json::parse(BENCH_DATA).unwrap()["widget"]["menu"];
    let v: Vec<&JsonValue> = menu
        .members()
        .filter(|x| x["sub_item"].as_i64().unwrap() > 5)
        .map(|x| &x["title"])
        .collect();
    println!("{:?}", v);
    let parsed = json::parse(BENCH_DATA).unwrap();
    println!("{:?}", parsed["overflow"].as_i64().unwrap());
}

#[test]
fn serde_json() {
    let v: Value = serde_json::from_str(BENCH_DATA).unwrap();
    println!("{:?}", v["widget"]["window"]["name"]);

    let menu = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["menu"];
    let v: Vec<&Value> = menu
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["sub_item"].as_i64().unwrap() > 5)
        .map(|x| &x["title"])
        .collect();
    println!("{:?}", v);
}

#[test]
fn gjson_ex() {
    let example = r#"
{"age":100, "name":{"here":"B\\\"R"},
	"noop":{"what is a wren?":"a bird"},
	"happy":true,"immortal":false,
	"items":[1,2,3,{"tags":[1,2,3],"points":[[1,2],[3,4]]},4,5,6,7],
	"arr":["1",2,"3",{"hello":"world"},"4",5],
	"vals":[1,2,3,{"sadf":sdx"asdf"}],"name":{"first":"tom","last":null},
	"created":"2014-05-16T08:28:06.989Z",
	"loggy":{
		"programmers": [
    	    {
    	        "firstName": "Brett",
    	        "lastName": "McLaughlin",
    	        "email": "aaaa",
				"tag": "good"
    	    },
    	    {
    	        "firstName": "Jason",
    	        "lastName": "Hunter",
    	        "email": "bbbb",
				"tag": "bad"
    	    },
    	    {
    	        "firstName": "Elliotte",
    	        "lastName": "Harold",
    	        "email": "cccc",
				"tag":, "good"
    	    },
			{
				"firstName": 1002.3,
				"age": 101
			}
    	]
	},
	"lastly":{"yay":"final"}
}
"#;

        // println!("=>{:?}<=", gjson::get(example, r#"friends.#(nets.#(=="ig"))"#));
        // println!("=>{:?}<=", gjson::get(example, r#"friends.#(nets."#));
        // println!("=>{:?}<=", gjson::get(example, r#"friends.#()#"#));
//     let json: Vec<char> = example.chars().collect();
//     let r = gjson::get(&json, r#"#.{field1,field2}"#);
//     println!("===> {:?}", r.array());

        // println!("result {:?}", gjson::get(example, r#"friends.#(nets.#(=="ig"))#"#).as_array());

        // println!("result {}", gjson::get(BENCH_DATA, "widget.window.name"));
        // println!("result {}", gjson::get(BENCH_DATA, "widget.image.hOffset"));
        // println!("result {}", gjson::get(BENCH_DATA, "widget.text.onMouseUp"));
        // println!("result {}", gjson::get(BENCH_DATA, "widget.debug"));
        // println!("result {:?}", gjson::get(BENCH_DATA, "widget.menu.#(sub_item>=7)#.title").as_array());
        println!("result {:?}", gjson::get(example, r#"loggy.programmers.#[tag="good"]#.firstName"#));
}

#[bench]
fn bench_json_rs(b: &mut Bencher) {
    b.iter(|| {
        // let d = json::parse(bench_data).unwrap();
        let _a = &json::parse(BENCH_DATA).unwrap()["widget"]["window"]["name"]
            .as_str()
            .unwrap();
        let _b = &json::parse(BENCH_DATA).unwrap()["widget"]["image"]["hOffset"]
            .as_i64()
            .unwrap();
        let _c = &json::parse(BENCH_DATA).unwrap()["widget"]["text"]["onMouseUp"]
            .as_str()
            .unwrap();
        let _d = &json::parse(BENCH_DATA).unwrap()["widget"]["debug"]
            .as_str()
            .unwrap();

        let text = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["text"] ;


        let menu = &json::parse(BENCH_DATA).unwrap()["widget"]["menu"];
        let _v: Vec<&JsonValue> = menu
            .members()
            .filter(|x| x["sub_item"].as_i64().unwrap() > 5)
            .map(|x| &x["title"])
            .collect();
    })
}

#[bench]
fn bench_serde_json(b: &mut Bencher) {
    b.iter(|| {
        // let d = json::parse(bench_data).unwrap();
        let _a = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["window"]["name"]
            .as_str()
            .unwrap();
        let _b = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["image"]["hOffset"]
            .as_i64()
            .unwrap();
        let _c = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["text"]["onMouseUp"]
            .as_str()
            .unwrap();
        let _d = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["debug"]
            .as_str()
            .unwrap();

        let text = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["text"] ;

        let menu = &serde_json::from_str::<Value>(BENCH_DATA).unwrap()["widget"]["menu"];
        let _v: Vec<&Value> = menu
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| x["sub_item"].as_i64().unwrap() > 5)
            .map(|x| &x["title"])
            .collect();
    })
}

#[bench]
fn bench_gjson(b: &mut Bencher) {
    b.iter(|| {
        gjson::get(BENCH_DATA, "widget.window.name").as_str();
        gjson::get(BENCH_DATA, "widget.image.hOffset").as_f64();
        gjson::get(BENCH_DATA, "widget.text.onMouseUp").as_str();
        gjson::get(BENCH_DATA, "widget.debug").as_str();
        gjson::get(BENCH_DATA, "widget.text").as_map();
        gjson::get(BENCH_DATA, "widget.menu.#(sub_item>7)#.title").as_array();
    })
}

fn main() {}
