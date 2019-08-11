/*! 
# A-JSON
Ajson is a lightweight json parser that allows users to dynamically retrieve values in json using a simple syntax.

## Examples
```
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

let name = ajson::get(data, "project.name").unwrap();
assert_eq!(name, "ajson");
```
## Syntax
JSON example
```text
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

A path is a series of keys separated by a dot. A key may contain special wildcard characters '*' and '?'. To access an array value use the index as the key. To get the number of elements in an array or to access a child path, use the '#' character. The dot and wildcard characters can be escaped with ''.

```text 
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
Special purpose characters, such as ., *, and ? can be escaped with .

```text
fav\.movie             "Deer Hunter"
```

#### Arrays
The # character allows for digging into JSON Arrays.To get the length of an array you'll just use the # all by itself.
```text
friends.#              3
friends.#.age         [44,68,47]
```

#### queries
You can also query an array for the first match by using #(...), or find all matches with #(...)#. Queries support the ==, !=, <, <=, >, >= comparison operators and the simple pattern matching % (like) and !% (not like) operators.

```text
friends.#(last=="Murphy").first   >> "Dale"
friends.#(last=="Murphy")#.first  >> ["Dale","Jane"]
friends.#(age>45)#.last           >> ["Craig","Murphy"]
friends.#(first%"D*").last        >> "Murphy"
friends.#(nets.#(=="fb"))#.first  >> ["Dale","Roger"]
```

#### construct
Basically, you can use selectors to assemble whatever you want, and of course, the result is still a json ;)
```text
{name.first,age,"murphys":friends.#(last="Murphy")#.first}
[name.first,age,children.0]
```
*/

extern crate regex;

mod getter;
mod number;
mod path;
mod path_parser;
mod reader;
mod sub_selector;
mod unescape;
mod util;
mod value;
mod wild;

pub use getter::Getter;
pub use number::Number;
use std::io;
pub use value::Value;

/// `get` value from JSON string with the specified path, it is relatively loose and 
/// can tolerate some illegal JSON.
/// ```
/// let data = r#"{"name": "ajson"}"#;
/// let v = ajson::get(data, "name").unwrap();
/// assert_eq!(v.as_str(), "ajson");
/// ```
/// If the given JSON is not a valid JSON, then ajson 
/// will try to get the value from the first JSON array 
/// or map it finds.
/// ```
/// let data = r#"someinvalidstring{"name": "ajson"}"#;
/// let v = ajson::get(data, "name").unwrap();
/// assert_eq!(v.as_str(), "ajson");
/// ```
/// If there is no valid JSON array or map in the given JSON, 
/// `get` returns None.
/// ```should_panic
/// let data = r#"someinvalidstring"#;
/// let v = ajson::get(data, "name").unwrap();
/// ```
pub fn get(json: &str, path: &str) -> Option<Value> {
    Getter::new_from_utf8(json.as_bytes()).get(path)
}


/// Returns the first JSON value parsed, and it may be having 
/// problems because it does not actively panic on incomplete 
/// JSON values. For example, array or map are not closed properly.
/// ```
/// let v = ajson::parse(r#"{"name": "ajson"}"#).unwrap();
/// assert!(v.is_object());
/// let v = ajson::parse(r#"{"name": "ajson""#).unwrap();
/// assert!(v.is_object());
/// let v = ajson::parse(r#"null,"string", 2"#).unwrap();
/// assert!(v.is_null());
/// ```
pub fn parse(json: &str) -> Option<Value> {
    let mut getter = Getter::new_from_utf8(json.as_bytes());
    getter.next_value()
}

/// Same to [`get`](fn.get.html), but for `io::read`.
pub fn get_from_read<R>(r: R, path: &str) -> Option<Value>
where
    R: io::Read,
{
    Getter::new_from_read(r).get(path)
}

/// Same to [`parse`](fn.parse.html), but for `io::read`.
pub fn parse_from_read<R>(r: R) -> Option<Value>
where
    R: io::Read,
{
    let mut getter = Getter::new_from_read(r);
    getter.next_value()
}
