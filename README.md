# brod

A small scripting language and tree-walking interpreter, written in Rust.

## Build

```
cargo build --release
```

Produces two binaries: `brod` (run scripts / REPL) and `formatter` (pretty-print an AST).

## Usage

```
brod script.brod              # run a script
brod script.brod -i           # run a script, then drop into a REPL sharing its state
brod                          # REPL only
```

## Language

```
var x = 10;
var name = "brod";
print(x, name);

fn add(a, b) {
    return a + b;
}

if (x > 5) {
    print("big");
} elif (x > 0) {
    print("small");
} else {
    print("non-positive");
}

var i = 0;
while (i < 3) {
    print(i);
    i = i + 1;
}

class Point {
    fn Point() {}
    fn describe() {
        return "a point";
    }
}
var p = Point();
print(p.describe());
```

## TODO

- **Arrays**: no list/array type.
- **References**: no way to passe variables arround like with references or pointer, all value are copy only
- **Variable resolver**: locals are still looked up by name in a `HashMap` per scope at runtime; a compile-time resolver pass (slot-indexed scopes) would remove that lookup cost.
