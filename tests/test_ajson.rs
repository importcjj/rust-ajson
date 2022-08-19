extern crate ajson;
extern crate json;
extern crate serde_json;

use ajson::{get, parse, Result, Value};

// #[test]
// fn test_json_rs_unicode() {

//     use serde_json::Value;

//     let data = r#"{"IdentityData":{"GameInstanceId":634866135153775564}}"#;
//     // let a = &json::parse(data).unwrap();
//     let a = &serde_json::from_str::<Value>(data).unwrap();
//     println!("{}", a["IdentityData"]["GameInstanceId"].to_u64().unwrap());
//     println!("{}", a["IdentityData"]["GameInstanceId"].to_i64().unwrap());
//     println!("{}", a["IdentityData"]["GameInstanceId"].to_f64().unwrap());

//     let data = r#"{"IdentityData":{"GameInstanceId":634866135153775564.88172}}"#;
//     let a = &serde_json::from_str::<Value>(data).unwrap();
//     // println!("{}", a["IdentityData"]["GameInstanceId"].to_u64().unwrap());
//     // println!("{}", a["IdentityData"]["GameInstanceId"].to_i64().unwrap());
//     println!("{}", a["IdentityData"]["GameInstanceId"].to_f64().unwrap());

//     let data = r#"
//     {
// 		"min_uint64": 0,
// 		"max_uint64": 18446744073709551615,
// 		"overflow_uint64": 18446744073709551616,
// 		"min_int64": -9223372036854775808,
// 		"max_int64": 9223372036854775807,
// 		"overflow_int64": 9223372036854775808,
// 		"min_uint53":  0,
// 		"max_uint53":  4503599627370495,
// 		"overflow_uint53": 4503599627370496,
// 		"min_int53": -2251799813685248,
// 		"max_int53": 2251799813685247,
// 		"overflow_int53": 2251799813685248
// 	}
//     "#;

//     // let b = &json::parse(data).unwrap();
//     let b = &serde_json::from_str::<Value>(data).unwrap();
//     assert_eq!(b["min_uint53"].to_u64().unwrap(), 0);
//     assert_eq!(b["max_uint53"].to_u64().unwrap(), 4503599627370495);
//     assert_eq!(b["overflow_uint53"].to_i64().unwrap(), 4503599627370496);
//     assert_eq!(b["min_int53"].to_i64().unwrap(), -2251799813685248);
//     assert_eq!(b["max_int53"].to_i64().unwrap(), 2251799813685247);
//     assert_eq!(b["overflow_int53"].to_i64().unwrap(), 2251799813685248);
//     assert_eq!(b["min_uint64"].to_u64().unwrap(), 0);
//     assert_eq!(b["max_uint64"].to_u64().unwrap(), 18446744073709551615);

//     assert_eq!(b["overflow_uint64"].to_i64().unwrap(), 0);
//     assert_eq!(b["min_int64"].to_i64().unwrap(), -9223372036854775808);
//     assert_eq!(b["max_int64"].to_i64().unwrap(), 9223372036854775807);

//     assert_eq!(b["overflow_int64"].to_i64().unwrap(), -9223372036854775808);
// }

// fn get(json: &str, path: &str) -> Option<Value> {
//     match env::var("GETTER_FROM_READ") {
//         Ok(open) => {
//             if open.len() > 0 {
//                 println!("get from read");
//                 let mut g = Getter::new_from_read(json.as_bytes());
//                 g.get(path)
//             } else {
//                 println!("get from str");
//                 ajson_get(json, path)
//             }
//         }
//         _ => {
//             println!("get from str");
//             ajson_get(json, path)
//         }
//     }
// }

static BASIC_JSON: &'static str = r#"
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

static BASIC_JSON2: &'static str = r#"
{
    "name": {"first": "Tom", "last": "Anderson"},
    "age":37,
    "children": ["Sara","Alex","Jack"],
    "fav.movie": "Deer Hunter",
    "friends": [
        {"first": "Dale", "last": "Murphy", "age": 44, "nets": ["ig", "fb", "tw"]},
        {"first": "Roger", "last": "Craig", "age": 68, "nets": ["fb", "tw"]},
        {"first": "Jane", "last": "Murphy", "age": 47, "nets": ["ig", "tw"]}
    ]
}
"#;

// name.last        >> "Anderson"
// age              >> 37
// children         >> ["Sara","Alex","Jack"]
// children.#       >> 3
// children.1       >> "Alex"
// child*.2         >> "Jack"
// c?ildren.0       >> "Sara"
// fav\.movie       >> "Deer Hunter"
// friends.#.first  >> ["Dale","Roger","Jane"]
// friends.1.last   >> "Craig"
#[test]
fn test_example() -> Result<()> {
    let r = parse(BASIC_JSON2)?.unwrap();
    assert_eq!(r.get("name.last")?.unwrap(), "Anderson");
    assert_eq!(r.get("age")?.unwrap(), 37);
    assert_eq!(
        r.get("children")?.unwrap().as_vec().unwrap(),
        vec!["Sara", "Alex", "Jack"]
    );
    assert_eq!(r.get("children.#")?.unwrap(), 3 as u64);
    assert_eq!(r.get("children.1")?.unwrap(), "Alex");
    assert_eq!(r.get("child*.2")?.unwrap(), "Jack");
    assert_eq!(r.get("c?ildren.0")?.unwrap(), "Sara");
    assert_eq!(r.get("fav\\.movie")?.unwrap(), "Deer Hunter");
    assert_eq!(r.get("friends.1.last")?.unwrap(), "Craig");
    assert_eq!(
        r.get("friends.#.first")?.unwrap().as_vec().unwrap(),
        vec!["Dale", "Roger", "Jane"]
    );

    Ok(())
}

// friends.#(last=="Murphy").first   >> "Dale"
// friends.#(last=="Murphy")#.first  >> ["Dale","Jane"]
// friends.#(age>45)#.last           >> ["Craig","Murphy"]
// friends.#(first%"D*").last        >> "Murphy"
// friends.#(nets.#(=="fb"))#.first  >> ["Dale","Roger"]
#[test]
fn test_query_example() -> Result<()> {
    let r = parse(BASIC_JSON2)?.unwrap();
    assert_eq!(
        r.get(r#"friends.#(last=="Murphy").first"#)?.unwrap(),
        "Dale"
    );
    assert_eq!(
        r.get(r#"friends.#(last=="Murphy")#.first"#)?
            .unwrap()
            .as_vec()
            .unwrap(),
        vec!["Dale", "Jane"]
    );
    assert_eq!(
        r.get(r#"friends.#(age>45)#.last"#)?
            .unwrap()
            .as_vec()
            .unwrap(),
        vec!["Craig", "Murphy"]
    );
    assert_eq!(r.get(r#"friends.#(first%"D*").last"#)?.unwrap(), "Murphy");
    assert_eq!(
        r.get(r#"friends.#(nets.#(=="fb"))#.first"#)?
            .unwrap()
            .as_vec()
            .unwrap(),
        vec!["Dale", "Roger"]
    );
    Ok(())
}

#[test]
fn test_basic() -> Result<()> {
    let r = ajson::parse(BASIC_JSON).unwrap().unwrap();
    println!(
        "{:?}",
        r.get(r#"loggy.programmers.#[tag="good"].firstName"#)
            .unwrap()
    );
    assert_eq!(
        "Brett",
        r.get(r#"loggy.programmers.#[tag="good"].firstName"#)
            .unwrap()
            .unwrap()
            .as_str()
            .unwrap()
    );
    assert_eq!(
        r.get(r#"loggy.programmers.#[tag="good"]#.firstName"#)
            .unwrap()
            .unwrap()
            .as_vec()
            .unwrap(),
        vec!["Brett", "Elliotte"]
    );

    Ok(())
}

#[test]
fn test_basic_2() -> Result<()> {
    let r = ajson::parse(BASIC_JSON)?.unwrap();
    let mut mtok = r
        .get(r#"loggy.programmers.#[age==101].firstName"#)?
        .unwrap();
    assert_eq!(mtok, 1002.3);
    mtok = r
        .get(r#"loggy.programmers.#[firstName != "Brett"].firstName"#)?
        .unwrap();
    assert_eq!(mtok, "Jason");

    mtok = r
        .get(r#"loggy.programmers.#[firstName % "Bre*"].email"#)?
        .unwrap();
    assert_eq!(mtok, "aaaa");

    mtok = r
        .get(r#"loggy.programmers.#[firstName !% "Bre*"].email"#)?
        .unwrap();
    assert_eq!(mtok, "bbbb");

    mtok = r
        .get(r#"loggy.programmers.#[firstName == "Brett"].email"#)?
        .unwrap();
    assert_eq!(mtok, "aaaa");

    mtok = r.get("loggy")?.unwrap();
    assert!(mtok.is_object());
    println!("{:?}", mtok.as_object());
    assert_eq!(mtok.as_object().unwrap().len(), 1);

    let programmers = &mtok.as_object().unwrap()["programmers"];
    assert_eq!(
        programmers.as_vec().unwrap()[1].as_object().unwrap()["firstName"],
        "Jason"
    );

    Ok(())
}

#[test]
fn test_basic_3() -> Result<()> {
    let t = ajson::parse(BASIC_JSON)?.unwrap();
    let json = "-102";
    let t = parse(json)?.unwrap();
    assert_eq!(t, -102);

    let json = "102";
    let t = parse(json)?.unwrap();
    assert_eq!(t, 102);

    let json = "102.2";
    let t = parse(json)?.unwrap();
    assert_eq!(t, 102.2);

    let json = r#""hello""#;
    let t = parse(json)?.unwrap();
    assert_eq!(t, "hello");

    let json = r#""\"he\nllo\"""#;
    let t = parse(json)?.unwrap();
    assert_eq!(t, "\"he\nllo\"");

    let t = parse(BASIC_JSON)?.unwrap();
    let t = t.get("loggy.programmers.#.firstName")?.unwrap();
    assert_eq!(t.as_vec().unwrap().len(), 4);
    assert_eq!(t.as_vec().unwrap()[..3], ["Brett", "Jason", "Elliotte"]);

    let t = parse(BASIC_JSON)?.unwrap();
    let t = t.get("loggy.programmers.#.asd")?.unwrap();
    assert!(t.is_array());
    assert_eq!(t.as_vec().unwrap().len(), 0);
    Ok(())
}

#[test]
fn test_basic_4() -> Result<()> {
    assert_eq!(get(&BASIC_JSON, "items.3.tags.#")?.unwrap(), 3 as f64);
    assert_eq!(get(&BASIC_JSON, "items.3.points.1.#")?.unwrap(), 2 as f64);
    assert_eq!(get(&BASIC_JSON, "items.#")?.unwrap(), 8 as f64);
    assert_eq!(get(&BASIC_JSON, "vals.#")?.unwrap(), 4 as f64);
    assert!(!get(&BASIC_JSON, "name.last")?.is_some());
    assert_eq!(get(&BASIC_JSON, "name.here")?.unwrap(), "B\\\"R");

    assert_eq!(get(&BASIC_JSON, "arr.#")?.unwrap(), 6 as f64);
    assert_eq!(get(&BASIC_JSON, "arr.3.hello")?.unwrap(), "world");
    // Need to Fix
    // assert_eq!(get(&BASIC_JSON, "name.first"), "tom");
    // assert_eq!(get(&BASIC_JSON, "name.last").unwrap(), "");
    // Need to Fix
    // assert!(get(&BASIC_JSON, "name.last").is_null());
    Ok(())
}

#[test]
fn test_basic_5() -> Result<()> {
    assert_eq!(get(&BASIC_JSON, "age")?.unwrap(), 100);
    assert_eq!(get(&BASIC_JSON, "happy")?.unwrap(), true);
    assert_eq!(get(&BASIC_JSON, "immortal")?.unwrap(), false);

    let t = get(&BASIC_JSON, "noop")?.unwrap();
    let m = t.as_object().unwrap();
    assert_eq!(m.len(), 1);
    assert_eq!(m["what is a wren?"], "a bird");

    let r = parse(&BASIC_JSON)?.unwrap();
    assert_eq!(
        r.as_object().unwrap()["loggy"].as_object().unwrap()["programmers"]
            .as_vec()
            .unwrap()[1]
            .as_object()
            .unwrap()["firstName"],
        "Jason"
    );

    Ok(())
}

#[test]
fn test_is_array_is_object() -> Result<()> {
    let r = parse(BASIC_JSON)?.unwrap();
    let mut mtok = r.get("loggy")?.unwrap();
    assert!(mtok.is_object());
    assert!(!mtok.is_array());

    mtok = r.get("loggy.programmers")?.unwrap();
    assert!(!mtok.is_object());
    assert!(mtok.is_array());

    mtok = r.get(r#"loggy.programmers.#[tag="good"]#.first"#)?.unwrap();
    assert!(mtok.is_array());

    mtok = r.get("loggy.programmers.0.firstName")?.unwrap();
    println!("{:?}", mtok.as_object());
    assert!(!mtok.is_object());
    assert!(!mtok.is_array());

    Ok(())
}

#[test]
fn test_plus_53_bit_ints() -> Result<()> {
    let json = r#"{"IdentityData":{"GameInstanceId":634866135153775564}}"#;
    let v = get(&json, "IdentityData.GameInstanceId")?.unwrap();
    assert_eq!(v, 634866135153775564 as u64);
    assert_eq!(v, 634866135153775564 as i64);
    assert_eq!(v, 634866135153775616.0 as f64);

    let json = r#"{"IdentityData":{"GameInstanceId":634866135153775564.88172}}"#;
    let v = get(&json, "IdentityData.GameInstanceId")?.unwrap();
    assert_eq!(v, 634866135153775564 as u64);
    assert_eq!(v, 634866135153775564 as i64);
    assert_eq!(v, 634866135153775616.88172 as f64);

    let json = r#"
    {
		"min_uint64": 0,
		"max_uint64": 18446744073709551615,
		"overflow_uint64": 18446744073709551616,
		"min_int64": -9223372036854775808,
		"max_int64": 9223372036854775807,
		"overflow_int64": 9223372036854775808,
		"min_uint53":  0,
		"max_uint53":  4503599627370495,
		"overflow_uint53": 4503599627370496,
		"min_int53": -2251799813685248,
		"max_int53": 2251799813685247,
		"overflow_int53": 2251799813685248
	}
    "#;

    assert_eq!(get(json, "min_uint53")?.unwrap(), 0);
    assert_eq!(get(&json, "max_uint53")?.unwrap(), 4503599627370495 as u64);
    assert_eq!(
        get(&json, "overflow_uint53")?.unwrap(),
        4503599627370496 as i64
    );
    assert_eq!(get(&json, "min_int53")?.unwrap(), -2251799813685248 as i64);
    assert_eq!(get(&json, "max_int53")?.unwrap(), 2251799813685247 as i64);
    assert_eq!(
        get(&json, "overflow_int53")?.unwrap(),
        2251799813685248 as i64
    );
    assert_eq!(get(&json, "min_uint64")?.unwrap(), 0);
    assert_eq!(
        get(&json, "max_uint64")?.unwrap(),
        18446744073709551615 as u64
    );

    assert_eq!(get(&json, "overflow_uint64")?.unwrap(), 0);
    assert_eq!(
        get(&json, "min_int64")?.unwrap(),
        -9223372036854775808 as i64
    );
    assert_eq!(
        get(&json, "max_int64")?.unwrap(),
        9223372036854775807 as i64
    );

    assert_eq!(get(&json, "overflow_int64")?.unwrap(), 0);

    Ok(())
}

#[test]
fn test_unicode() -> Result<()> {
    let json = r#"{"key":0,"的情况下解":{"key":1,"的情况":2}}"#;
    let r = parse(json)?.unwrap();
    println!("{:?}", r.as_object());
    println!("{:?}", r.get("的情况下解")?.unwrap().as_object());
    assert_eq!(r.get("的情况下解.key")?.unwrap(), 1.0);
    assert_eq!(r.get("的情况下解.的情况")?.unwrap(), 2.0);
    assert_eq!(r.get("的情况下解.的?况")?.unwrap(), 2.0);
    assert_eq!(r.get("的情况下解.的?*")?.unwrap(), 2.0);
    assert_eq!(r.get("的情况下解.*?况")?.unwrap(), 2.0);
    assert_eq!(r.get("的情?下解.*?况")?.unwrap(), 2.0);
    assert!(r.get("的情下解.*?况")?.is_none());

    Ok(())
}

#[test]
fn test_emoji() -> Result<()> {
    let input = r#"{"utf8":"Example emoji, KO: \ud83d\udd13, \ud83c\udfc3 OK: \u2764\ufe0f "}"#;
    let r = parse(input)?.unwrap();
    assert_eq!(r.get("utf8")?.unwrap(), "Example emoji, KO: 🔓, 🏃 OK: ❤️ ");
    Ok(())
}

#[test]
fn test_parse_any() -> Result<()> {
    assert_eq!(parse("100")?.unwrap(), 100 as f64);
    assert_eq!(parse("true")?.unwrap(), true);
    assert_eq!(parse("false")?.unwrap(), false);
    assert_eq!(parse("yikes")?.is_some(), false);
    Ok(())
}

#[test]
fn test_map() -> Result<()> {
    let a = r#""asdf""#;
    let b = r#"{"asdf":"ghjk""#;
    let c = String::from(r#"**invalid**"#);
    let d = String::from(r#"{"#);
    assert!(parse(a)?.unwrap().as_object().is_none());
    assert_eq!(parse(b)?.unwrap().as_object().unwrap()["asdf"], "ghjk");
    assert!(Value::Object(std::borrow::Cow::Owned(c))
        .as_object()
        .is_none());
    assert!(Value::Object(std::borrow::Cow::Owned(d))
        .as_object()
        .is_some());
    Ok(())
}

#[test]
fn test_array() -> Result<()> {
    let json = r#"
    {
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
    let r = parse(json)?.unwrap();
    let a = r.get("widget.menu.#(sub_item>5)#.title")?.unwrap();
    assert_eq!(a.as_vec().unwrap(), vec!["file", "edit"]);

    let a = r.get("widget.menu.#.options.#(>4)")?.unwrap();
    assert_eq!(a.as_vec().unwrap(), vec![5, 6]);

    let a = r.get("widget.menu.#.options.#(>4)#")?.unwrap();
    assert_eq!(a.as_vec().unwrap().len(), 3);

    Ok(())
}

#[test]
fn test_issue_38() -> Result<()> {
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93"]"#)?
            .unwrap()
            .get("0")?
            .unwrap(),
        r#"S3O PEDRO DO BUTI\udf93"#
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93asdf"]"#)?
            .unwrap()
            .get("0")?
            .unwrap(),
        "S3O PEDRO DO BUTI\\udf93asdf"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u"]"#)?
            .unwrap()
            .get("0")?
            .unwrap(),
        "S3O PEDRO DO BUTI\\udf93\\u"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u1"]"#)?
            .unwrap()
            .get("0")?
            .unwrap(),
        "S3O PEDRO DO BUTI\\udf93\\u1"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u13"]"#)?
            .unwrap()
            .get("0")?
            .unwrap(),
        "S3O PEDRO DO BUTI\\udf93\\u13"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u134"]"#)?
            .unwrap()
            .get("0")?
            .unwrap(),
        "S3O PEDRO DO BUTI\\udf93\\u134"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u1345"]"#)?
            .unwrap()
            .get("0")?
            .unwrap(),
        "S3O PEDRO DO BUTI\\udf93ፅ"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u1345asd"]"#)?
            .unwrap()
            .get("0")?
            .unwrap(),
        "S3O PEDRO DO BUTI\\udf93ፅasd"
    );

    Ok(())
}

#[test]
fn test_escape_path() -> Result<()> {
    let json = r#"{
		"test":{
			"*":"valZ",
			"*v":"val0",
			"keyv*":"val1",
			"key*v":"val2",
			"keyv?":"val3",
			"key?v":"val4",
			"keyv.":"val5",
			"key.v":"val6",
			"keyk*":{"key?":"val7"}
		}
	}"#;

    let r = parse(json)?.unwrap();
    assert_eq!(r.get("test.\\*")?.unwrap(), "valZ");
    assert_eq!(r.get("test.\\*v")?.unwrap(), "val0");
    assert_eq!(r.get("test.keyv\\*")?.unwrap(), "val1");
    assert_eq!(r.get("test.key\\*v")?.unwrap(), "val2");
    assert_eq!(r.get("test.keyv\\?")?.unwrap(), "val3");
    assert_eq!(r.get("test.key\\?v")?.unwrap(), "val4");
    assert_eq!(r.get("test.keyv\\.")?.unwrap(), "val5");
    assert_eq!(r.get("test.key\\.v")?.unwrap(), "val6");
    assert_eq!(r.get("test.keyk\\*.key\\?")?.unwrap(), "val7");

    Ok(())
}

#[test]
fn test_null_array() -> Result<()> {
    assert!(get(r#"{"data":null}"#, "data")?.unwrap().as_vec().is_none());
    assert!(get(r#"{}"#, "data")?.is_none());
    assert_eq!(
        parse(r#"{"data":[]}"#)?
            .unwrap()
            .get("data")?
            .unwrap()
            .as_vec()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        parse(r#"{"data":[null]}"#)?
            .unwrap()
            .get("data")?
            .unwrap()
            .as_vec()
            .unwrap()
            .len(),
        1
    );

    Ok(())
}

#[test]
fn test_token_raw_for_literal() -> Result<()> {
    assert_eq!(parse("null")?.unwrap(), Value::Null);
    assert_eq!(parse("true")?.unwrap(), Value::Boolean(true));
    assert_eq!(parse("false")?.unwrap(), Value::Boolean(false));

    Ok(())
}

#[test]
fn test_single_array_value() -> Result<()> {
    let json = r#"{"key": "value","key2":[1,2,3,4,"A"]}"#;
    let array: Vec<Value> = get(&json, "key")?.into_iter().collect();

    assert_eq!(array.len(), 1);
    assert_eq!(array[0], "value");

    let array: Vec<Value> = get(&json, "key2.#")?.into_iter().collect();
    assert_eq!(array.len(), 1);

    let r = get(&json, "key3")?;
    assert!(r.is_none());

    Ok(())
}

// #[test]
// fn test_invalid_path() -> Result<()> {
//     let r = parse(BASIC_JSON)?.unwrap();
//     assert!(r.get("loggy.programmers.#(firstName==").is_err());
//     assert!(r.get("loggy.programmers.#(").is_err());
//     assert!(r.get("loggy.programmers.#(firstName").is_err());
//     assert!(r.get("loggy.programmers.#(first").is_err());
//     assert!(r.get(r#"loggy.programmers.#(firstName=="x""#).is_err());
//     assert!(r.get(r#"loggy.programmers.#()"#).is_err());

//     Ok(())
// }

#[test]
fn test_bracket_in_array() -> Result<()> {
    let json = r##"{
    "children": ["Sara","Alex]","Jack"],
    "##;
    let r = parse(json)?.unwrap();
    assert_eq!(r.get("children.#")?.unwrap(), 3);
    assert_eq!(
        r.get("children")?.unwrap().as_vec().unwrap(),
        vec!["Sara", "Alex]", "Jack"]
    );
    assert_eq!(r.get("children.1")?.unwrap(), "Alex]");
    assert_eq!(r.get("child*.2")?.unwrap(), "Jack");
    assert_eq!(r.get("c?ildren.0")?.unwrap(), "Sara");

    Ok(())
}

#[test]
fn test_key_unicode() -> Result<()> {
    {
        let json = r#"{"sample_unicode\u0041\u0042": "HelloWorld"}"#;
        assert_eq!(get(json, "sample_unicodeAB")?.unwrap(), "HelloWorld");
    }

    {
        let json = r#"{"的情况下解\u0030\u0031\u0032":{"key":1,"的情况":2}}"#;
        assert_eq!(get(json, "的情况下解012.key")?.unwrap(), 1);
        assert_eq!(get(json, "的情况下解012.的情况")?.unwrap(), 2);
    }

    Ok(())
}
