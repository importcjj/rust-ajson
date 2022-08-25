/*!
# A-JSON
Ajson is a lightweight json parser that allows users to dynamically retrieve values in json using a simple syntax.

## Examples
```
use ajson::Result;
fn main() -> Result<()> {
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

    let name = ajson::get(data, "project.name")?.unwrap();
    assert_eq!(name, "ajson");
    Ok(())
}
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

#[cfg(feature = "wild")]
extern crate regex;
#[cfg(feature = "wild")]
mod wild;

mod element;
mod number;
mod parser;
mod path;
mod unescape;
mod util;
mod value;

pub use number::Number;
pub use path::Path;
pub use unescape::unescape;
pub use value::Value;

#[doc(hidden)]
pub use element::compound;
#[doc(hidden)]
pub use element::compound_u8;

use std::result;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Path,
    Eof,
    ObjectKey,
    Object,
    Array,
}

pub type Result<T> = result::Result<T, Error>;

/// `get` value from JSON string with the specified path, it is relatively loose and
/// can tolerate some illegal JSON.
/// ```
/// use ajson::Result;
/// fn main() -> Result<()> {
///     let data = r#"{"name": "ajson"}"#;
///     let v = ajson::get(data, "name")?.unwrap();
///     assert_eq!(v, "ajson");
///     Ok(())
/// }
/// ```
/// If the given JSON is not a valid JSON, then ajson
/// will try to get the value from the first JSON array
/// or map it finds.
/// ```
/// use ajson::Result;
/// fn main() -> Result<()> {
///     let data = r#"someinvalidstring{"name": "ajson"}"#;
///     let v = ajson::get(data, "name")?.unwrap();
///     assert_eq!(v, "ajson");
///     Ok(())
/// }
/// ```
/// If there is no valid JSON array or map in the given JSON,
/// `get` returns None.
/// ```should_panic
/// let data = r#"someinvalidstring"#;
/// let v = ajson::get(data, "name").unwrap().unwrap();
/// ```
pub fn get<'a>(json: &'a str, path: &'a str) -> Result<Option<Value<'a>>> {
    let path = path::Path::from_slice(path.as_bytes())?;
    let (a, _left) = parser::bytes_get(json.as_bytes(), &path)?;
    Ok(a.map(|el| el.to_value()))
}

/// Returns the first JSON value parsed, and it may be having
/// problems because it does not actively panic on incomplete
/// JSON values. For example, array or map are not closed properly.
/// ```
/// use ajson::Result;
/// fn main() -> Result<()> {
///     let v = ajson::parse(r#"{"name": "ajson"}"#)?.unwrap();
///     assert!(v.is_object());
///     let v = ajson::parse(r#"{"name": "ajson""#)?.unwrap();
///     assert!(v.is_object());
///     let v = ajson::parse(r#"null,"string", 2"#)?.unwrap();
///     assert!(v.is_null());
///     Ok(())
/// }
/// ```
pub fn parse(json: &str) -> Result<Option<Value>> {
    let (parsed, _left) = element::read_one(json.as_bytes())?;

    Ok(parsed.map(|el| el.to_value()))
}
