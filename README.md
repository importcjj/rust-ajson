<div align="center">
  <img width="240" height="78" border="0" alt="GJSON" src="logo.png">

  <p>Get JSON values quickly - JSON Parser for Rust</p>

  <a href="https://github.com/importcjj/gjson">
  <img src="https://travis-ci.com/importcjj/gjson2.svg?token=ZZrg3rRkUA8NUGrjEsU9&branch=master"></a>


  <a href="https://importcjj.github.io/rust-gjson-playground/">
  <img src="https://img.shields.io/badge/goto-playground-orange">

  <hr>
</a>


</div>

## Usage

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

#### with query

```
friends.#(last=="Murphy").first   >> "Dale"
friends.#(last=="Murphy")#.first  >> ["Dale","Jane"]
friends.#(age>45)#.last           >> ["Craig","Murphy"]
friends.#(first%"D*").last        >> "Murphy"
friends.#(nets.#(=="fb"))#.first  >> ["Dale","Roger"]
```

#### selectors

```
{name.first,age,"murphys":friends.#(last="Murphy")#.first}
[name.first,age,children.0]
```


## Performance

Benchmarks => [gjson](https://github.com/importcjj/gjson), [serde_json](https://github.com/serde-rs/json), [rust-json](https://github.com/maciejhirsz/json-rust)

```
gjson benchmark         time:   [6.7000 us 6.8023 us 6.9081 us]                             
                        change: [-1.8368% -0.4152% +1.0466%] (p = 0.58 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

serde_json benchmark    time:   [48.196 us 48.543 us 48.947 us]                                  
                        change: [+2.9073% +4.4909% +6.3532%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

json-rust benchmark     time:   [24.540 us 24.773 us 25.061 us]                                 
                        change: [+4.8288% +6.0452% +7.4633%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
```