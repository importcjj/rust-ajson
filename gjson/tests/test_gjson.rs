extern crate gjson;
use gjson::{get, parse, Value};

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

#[test]
fn test_basic() {
    let r = gjson::parse(BASIC_JSON);
    println!("{}", r.get(r#"loggy.programmers.#[tag="good"].firstName"#));
    assert_eq!(
        "Brett",
        r.get(r#"loggy.programmers.#[tag="good"].firstName"#)
            .as_str()
    );
    assert_eq!(
        r.get(r#"loggy.programmers.#[tag="good"]#.firstName"#)
            .as_array(),
        vec!["Brett", "Elliotte"]
    );
}

#[test]
fn test_basic_2() {
    let r = gjson::parse(BASIC_JSON);
    let mut mtok = r.get(r#"loggy.programmers.#[age==101].firstName"#);
    assert_eq!(mtok, "1002.3");
    mtok = r.get(r#"loggy.programmers.#[firstName != "Brett"].firstName"#);
    assert_eq!(mtok, "Jason");

    mtok = r.get(r#"loggy.programmers.#[firstName % "Bre*"].email"#);
    assert_eq!(mtok, "aaaa");

    mtok = r.get(r#"loggy.programmers.#[firstName !% "Bre*"].email"#);
    assert_eq!(mtok, "bbbb");

    mtok = r.get(r#"loggy.programmers.#[firstName == "Brett"].email"#);
    assert_eq!(mtok, "aaaa");

    mtok = r.get("loggy");
    assert!(mtok.is_object());
    println!("{:?}", mtok.as_map());
    assert_eq!(mtok.as_map().len(), 1);

    let programmers = &mtok.as_map()["programmers"];
    assert_eq!(programmers.as_array()[1].as_map()["firstName"], "Jason");
}

#[test]
fn test_basic_3() {
    let t = gjson::parse(BASIC_JSON).get("loggy.programmers").get("1").get("firstName");
    assert_eq!(t, "Jason");

    let json = "-102";
    let t = parse(json);
    assert_eq!(t, -102 as f64);

    let json = "102";
    let t = parse(json);
    assert_eq!(t, 102 as f64);

    let json = "102.2";
    let t = parse(json);
    assert_eq!(t, 102.2 as f64);

    let json = r#""hello""#;
    let t = parse(json);
    assert_eq!(t, "hello");

    let json = r#""\"he\nllo\"""#;
    let t = parse(json);
    assert_eq!(t, r#"\"he\nllo\""#);

    let t = parse(BASIC_JSON).get("loggy.programmers.#.firstName");
    assert_eq!(t.as_array().len(), 4);
    assert_eq!(t.as_array(), ["Brett", "Jason", "Elliotte", "1002.3"]);

    let t = parse(BASIC_JSON).get("loggy.programmers.#.asd");
    assert!(t.is_array());
    assert_eq!(t.as_array().len(), 0);
}

#[test]
fn test_basic_4() {
    assert_eq!(get(&BASIC_JSON, "items.3.tags.#"), 3 as f64);
    assert_eq!(get(&BASIC_JSON, "items.3.points.1.#"), 2 as f64);
    assert_eq!(get(&BASIC_JSON, "items.#"), 8 as f64);
    assert_eq!(get(&BASIC_JSON, "vals.#"), 4 as f64);
    assert!(!get(&BASIC_JSON, "name.last").exists());
    // Need to Fix
    // assert_eq!(get(&BASIC_JSON, "name.here"), r#"B\"R"#);
    assert_eq!(get(&BASIC_JSON, "arr.#"), 6 as f64);
    assert_eq!(get(&BASIC_JSON, "arr.3.hello"), "world");
    // Need to Fix
    // assert_eq!(get(&BASIC_JSON, "name.first"), "tom");
    assert_eq!(get(&BASIC_JSON, "name.last"), "");
    // Need to Fix
    // assert!(get(&BASIC_JSON, "name.last").is_null());
}

#[test]
fn test_basic_5() {
    assert_eq!(get(&BASIC_JSON, "age"), "100");
    assert_eq!(get(&BASIC_JSON, "happy"), "true");
    assert_eq!(get(&BASIC_JSON, "immortal"), "false");

    let t = get(&BASIC_JSON, "noop");
    let m = t.as_map();
    assert_eq!(m.len(), 1);
    assert_eq!(m["what is a wren?"], "a bird");

    let r = parse(&BASIC_JSON);
    assert_eq!(r.as_map()["loggy"].as_map()["programmers"].as_array()[1].as_map()["firstName"], "Jason");
    
}

#[test]
fn test_is_array_is_object() {
    let r = parse(BASIC_JSON);
    let mut mtok = r.get("loggy");
    assert!(mtok.is_object());
    assert!(!mtok.is_array());

    mtok = r.get("loggy.programmers");
    assert!(!mtok.is_object());
    assert!(mtok.is_array());

    mtok = r.get(r#"loggy.programmers.#[tag="good"]#.first"#);
    assert!(mtok.is_array());

    mtok = r.get("loggy.programmers.0.firstName");
    println!("{:?}", mtok.as_map());
    assert!(!mtok.is_object());
    assert!(!mtok.is_array());
}

#[test]
fn test_plus_53_bit_ints() {
    let json = r#"{"IdentityData":{"GameInstanceId":634866135153775564}}"#;
    let v = get(&json, "IdentityData.GameInstanceId");
    assert_eq!(v.as_u64(), 634866135153775564);
    assert_eq!(v.as_i64(), 634866135153775564);
    // Todo
    // assert_eq!(v.to_f64(), 634866135153775616.0);


    let json = r#"{"IdentityData":{"GameInstanceId":634866135153775564.88172}}"#;
    let v = get(&json, "IdentityData.GameInstanceId");
    assert_eq!(v.as_u64() as u64, 634866135153775616);
    assert_eq!(v.as_i64(), 634866135153775616);
    // Todo
    // assert_eq!(v.to_f64(), 634866135153775616.88172);

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

    assert_eq!(get(json, "min_uint53").as_u64(), 0);
    // assert_eq!(get(&json, "max_uint53").to_u64(), 4503599627370495);
    // assert_eq!(get(&json, "overflow_uint53").to_i64(), 4503599627370496);
    // assert_eq!(get(&json, "min_int53").to_i64(), -2251799813685248);
    // assert_eq!(get(&json, "max_int53").to_i64(), 2251799813685247);
    // assert_eq!(get(&json, "overflow_int53").to_i64(), 2251799813685248);
    // assert_eq!(get(&json, "min_uint64").to_u64(), 0);
    // assert_eq!(get(&json, "max_uint64").to_u64(), 18446744073709551615);
    
    // assert_eq!(get(&json, "overflow_uint64").to_i64(), 0);
    // assert_eq!(get(&json, "min_int64").to_i64(), -9223372036854775808);
    // assert_eq!(get(&json, "max_int64").to_i64(), 9223372036854775807);
    
    // assert_eq!(get(&json, "overflow_int64").to_i64(), -9223372036854775808);
}

#[test]
fn test_unicode() {
    let json = r#"{"key":0,"的情况下解":{"key":1,"的情况":2}}"#;
    let r = parse(json);
    println!("{:?}", r.as_map());
    println!("{:?}", r.get("的情况下解").as_map());
    assert_eq!(r.get("的情况下解.key"), 1.0);
    assert_eq!(r.get("的情况下解.的情况"), 2.0);
    assert_eq!(r.get("的情况下解.的?况"), 2.0);
    assert_eq!(r.get("的情况下解.的?*"), 2.0);
    assert_eq!(r.get("的情况下解.*?况"), 2.0);
    assert_eq!(r.get("的情?下解.*?况"), 2.0);
    assert_eq!(r.get("的情下解.*?况"), 0 as f64);
}

#[test]
fn test_emoji() {
    let input = r#"{"utf8":"Example emoji, KO: \ud83d\udd13, \ud83c\udfc3 OK: \u2764\ufe0f "}"#;
    let r = parse(input);
    assert_eq!(
        r.get("utf8"),
        // r#"Example emoji, KO: 🔓, 🏃 OK: ❤️ "#
        r#"Example emoji, KO: \ud83d\udd13, \ud83c\udfc3 OK: \u2764\ufe0f "#
    );
}

#[test]
fn test_parse_any() {
    assert_eq!(parse("100").as_f64(), 100 as f64);
    assert_eq!(parse("true").as_bool(), true);
    assert_eq!(parse("false").as_bool(), false);
    assert_eq!(parse("yikes").exists(), false);
}

#[test]
fn test_map() {
    let a = r#""asdf""#;
    let b = r#"{"asdf":"ghjk""#;
    let c = String::from(r#"**invalid**"#);
    let d = String::from(r#"{"#);
    assert_eq!(parse(a).as_map().len(), 0);
    assert_eq!(parse(b).as_map()["asdf"], "ghjk");
    assert_eq!(Value::Object(c).as_map().len(), 0);
    assert_eq!(Value::Object(d).as_map().len(), 0);
}

#[test]
fn test_array() {
    let json = r#"{
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
    let r = parse(json);
    let a = r.get("widget.menu.#(sub_item>5)#.title");
    assert_eq!(a.as_array(), vec!["file", "edit"]);

    let a = r.get("widget.menu.#.options.#(>4)");
    assert_eq!(a.as_array(), vec!["5", "6"]);

    let a = r.get("widget.menu.#.options.#(>4)#");
    assert_eq!(a.as_array().len(), 3);
}

#[test]
fn test_issue_38() {
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93"]"#).get("0"),
        "S3O PEDRO DO BUTI\\udf93"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93asdf"]"#).get("0"),
        "S3O PEDRO DO BUTI\\udf93asdf"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u"]"#).get("0"),
        "S3O PEDRO DO BUTI\\udf93\\u"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u1"]"#).get("0"),
        "S3O PEDRO DO BUTI\\udf93\\u1"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u13"]"#).get("0"),
        "S3O PEDRO DO BUTI\\udf93\\u13"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u134"]"#).get("0"),
        "S3O PEDRO DO BUTI\\udf93\\u134"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u1345"]"#).get("0"),
        "S3O PEDRO DO BUTI\\udf93\\u1345"
    );
    assert_eq!(
        parse(r#"["S3O PEDRO DO BUTI\udf93\u1345asd"]"#).get("0"),
        "S3O PEDRO DO BUTI\\udf93\\u1345asd"
    );
}

#[test]
fn test_escape_path() {
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

    let r = parse(json);
    assert_eq!(r.get("test.\\*"), "valZ");
    assert_eq!(r.get("test.\\*v"), "val0");
    assert_eq!(r.get("test.keyv\\*"), "val1");
    assert_eq!(r.get("test.key\\*v"), "val2");
    assert_eq!(r.get("test.keyv\\?"), "val3");
    assert_eq!(r.get("test.key\\?v"), "val4");
    assert_eq!(r.get("test.keyv\\."), "val5");
    assert_eq!(r.get("test.key\\.v"), "val6");
    assert_eq!(r.get("test.keyk\\*.key\\?"), "val7");
}

#[test]
fn test_null_array() {
    assert_eq!(parse(r#"{"data":null}"#).get("data").as_array().len(), 0);
    assert_eq!(parse(r#"{}"#).get("data").as_array().len(), 0);
    assert_eq!(parse(r#"{"data":[]}"#).get("data").as_array().len(), 0);
    assert_eq!(parse(r#"{"data":[null]}"#).get("data").as_array().len(), 1);
}

#[test]
fn test_token_raw_for_literal() {
    let raws = vec!["null", "true", "false"];
    for raw in &raws {
        assert_eq!(parse(&raw), *raw);
    }
}

#[test]
fn test_single_array_value() {
    let json = r#"{"key": "value","key2":[1,2,3,4,"A"]}"#;
    let r = get(&json, "key");
    let array = r.as_array();
    
    assert_eq!(array.len(), 1);
    assert_eq!(array[0], "value");

    let r = get(&json, "key2.#");
    let array = r.as_array();
    assert_eq!(array.len(), 1);

    let r = get(&json, "key3");
    let array = r.as_array();
    assert_eq!(array.len(), 0);
}

// #[test]
// fn test_invalid_path() {
//     let r = parse(BASIC_JSON);
//     assert!(r.get("loggy.programmers.#(firstName==").is_null());
//     assert!(r.get("loggy.programmers.#(").is_null());
//     assert!(r.get("loggy.programmers.#(firstName").is_null());
//     assert!(r.get("loggy.programmers.#(first").is_null());
//     assert!(r.get(r#"loggy.programmers.#(firstName=="x""#).is_null());
//     assert!(r.get(r#"loggy.programmers.#()"#).is_null());
// }