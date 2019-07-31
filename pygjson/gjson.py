from cffi import FFI
import os

ffi = FFI()
ffi.cdef("""
    char* gjson_get(char *json, char *path);
""")

C = ffi.dlopen("../target/debug/libpygjson.dylib")


def get(json, path):
    cjson = ffi.new("char []", json)
    cpath = ffi.new("char []", path)
    return ffi.string(C.gjson_get(cjson, cpath))


if __name__ == "__main__":
    json = """
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
    """

    v = get(json, '{name.first,age,"murphys":friends.#(last="Murphy")#.first}')
    print(v)
