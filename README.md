<div align="center">
  <!-- <img alt="AJSON" src="logo.png"> -->
  <h1>A-JSON</h1>
  <p>Read JSON values quickly - Rust JSON Parser</p>

  <a href="https://github.com/importcjj/rust-ajson">
  <img src="https://github.com/importcjj/rust-ajson/actions/workflows/rust.yml/badge.svg"></a>

  <a href="https://crates.io/crates/ajson">
  <img src="https://img.shields.io/badge/crates.io-0.2.1-blue"></a>

  <a href="https://importcjj.github.io/rust-ajson-playground/">
  <img src="https://img.shields.io/badge/goto-playground-orange">

</a>


</div>


#### change name to AJSON, see [issue](https://github.com/importcjj/a-json/issues/2)
Inspiration comes from [gjson](https://github.com/tidwall/gjson) in golang

## Installation
Add it to your `Cargo.toml` file:
```
[dependencies]
ajson = "0.3"
```

## Todo

* Add documentation
* Follow api-guidelines
* Update benchmark
* Optimize

## A simple example

AJSON get json value with specified path, such as `project.name` or `project.version`. When the path matches, it returns immediately!

```rust
let data = r#"
{
  "project": {
    "name": "ajson",
    "maintainer": "importcjj",
    "version": 0.1,
    "rusts": ["stable", "nightly"]
  }
}
"#;

let name = ajson::get(data, "project.name").unwrap().unwrap();
println!("{}", name.as_str()); // ajson
```

## Path Syntax

JSON example

```json
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
```

#### basic
Below is a quick overview of the path syntax, for more complete information please check out GJSON Syntax.

A path is a series of keys separated by a dot. A key may contain special wildcard characters '*' and '?'. To access an array value use the index as the key. To get the number of elements in an array or to access a child path, use the '#' character. The dot and wildcard characters can be escaped with '\'.

```
name.last        >> "Anderson"
age              >> 37
children         >> ["Sara","Alex","Jack"]
children.#       >> 3
children.1       >> "Alex"
child*.2         >> "Jack"
c?ildren.0       >> "Sara"
fav\.movie       >> "Deer Hunter"
friends.#.first  >> ["Dale","Roger","Jane"]
friends.1.last   >> "Craig"
```

#### Escape character
Special purpose characters, such as ., *, and ? can be escaped with \.

```
fav\.movie             "Deer Hunter"
```

#### Arrays
The # character allows for digging into JSON Arrays.To get the length of an array you'll just use the # all by itself.

```
friends.#              3
friends.#.age         [44,68,47]
```

#### queries
You can also query an array for the first match by using #(...), or find all matches with #(...)#. Queries support the ==, !=, <, <=, >, >= comparison operators and the simple pattern matching % (like) and !% (not like) operators.

```
friends.#(last=="Murphy").first   >> "Dale"
friends.#(last=="Murphy")#.first  >> ["Dale","Jane"]
friends.#(age>45)#.last           >> ["Craig","Murphy"]
friends.#(first%"D*").last        >> "Murphy"
friends.#(nets.#(=="fb"))#.first  >> ["Dale","Roger"]
```

#### construct
Basically, you can use selectors to assemble whatever you want, and of course, the result is still a json ;)


```
{name.first,age,"murphys":friends.#(last="Murphy")#.first}
[name.first,age,children.0]
```

```rust
ajson::get(json, "name.[first,last]").unwrap().unwrap().to_vec();
ajson::get(json, "name.first").unwrap().unwrap(); 
ajson::get(json, "name.last").unwrap().unwrap();
```

## Value

Value types.
```rust
enum Value {
    String(String),
    Number(Number),
    Object(String),
    Array(String),
    Boolean(bool),
    Null,
}
```

Value has a number of methods that meet your different needs.

```rust
value.get(&str) -> Option<Value>
value.as_str() -> &str
value.as_u64() -> u64
value.as_i64() -> i64
value.as_f64() -> f64
value.as_bool() -> bool
value.as_vec() -> Vec<Value>
value.as_object() -> HashMap<String, Value>
```


```rust
value.is_number() -> bool
value.is_string() -> bool
value.is_bool() -> bool
value.is_object() -> bool
value.is_array() -> bool
value.is_null() -> bool
```

## Performance

`$ cargo bench`

* [ajson](https://github.com/importcjj/ajson)
* [serde_json](https://github.com/serde-rs/json)
* [rust-json](https://github.com/maciejhirsz/json-rust)

```
ajson benchmark         time:   [2.0816 us 2.0865 us 2.0917 us]                             
                        change: [+0.6172% +0.9272% +1.2430%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) high mild
  4 (4.00%) high severe

serde_json benchmark    time:   [23.033 us 23.076 us 23.119 us]                                  
                        change: [-0.7185% -0.3455% +0.0230%] (p = 0.07 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe

json-rust benchmark     time:   [12.225 us 12.289 us 12.381 us]                                 
                        change: [-2.6200% -1.1789% +0.8442%] (p = 0.19 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe

ajson selector          time:   [1.1523 us 1.1561 us 1.1604 us]                            
                        change: [+0.1567% +0.7278% +1.2945%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

ajson multi query       time:   [559.19 ns 559.96 ns 560.77 ns]                               
                        change: [-1.4268% -1.0380% -0.6698%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

serde derive            time:   [4.5301 us 4.5403 us 4.5507 us]                          
                        change: [-2.3423% -1.9438% -1.5697%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

serde derive multi query                        
                        time:   [956.86 ns 962.64 ns 970.05 ns]
                        change: [-1.7069% -1.0299% -0.2924%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe

nom json bench          time:   [2.9468 us 2.9515 us 2.9566 us]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
```

* MacBook Pro (14-inch, 2021)
* Apple M1 Pro
* 16 GB

## License
 MIT License.
