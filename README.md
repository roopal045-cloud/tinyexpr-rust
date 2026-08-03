# tinyexpr-rs

Tiny recursive descent expression parser and evaluation engine for math
expressions, written in Rust. Port of [TinyExpr](https://github.com/codeplea/tinyexpr),
a small C library by Lewis Van Winkle.

Handy when you want to evaluate math expressions at runtime (user input,
config files, formulas, calculator apps, etc.) without pulling in a full
expression-language crate.

## Features

- Single dependency-free crate.
- Standard operators with normal precedence: `+ - * / ^ %`.
- Standard math functions: `sin`, `cos`, `sqrt`, `pow`, `log`, `atan2`, etc.
- Bind variables and re-evaluate cheaply without reparsing.
- Add your own functions/closures.
- Constant folding at compile time.

## Example

```rust
use tinyexpr::interp;

fn main() {
    let r = interp("sqrt(3^2 + 4^2)").unwrap();
    println!("{r}"); // 5
}
```

## Binding variables

```rust
use tinyexpr::Context;

let mut ctx = Context::new();
let x = ctx.var("x");
let y = ctx.var("y");

let expr = tinyexpr::compile("sqrt(x^2+y^2)", &ctx).unwrap();

x.set(3.0);
y.set(4.0);
println!("{}", expr.eval()); // 5

// change the variables and re-evaluate, no reparsing needed
x.set(6.0);
y.set(8.0);
println!("{}", expr.eval()); // 10
```

## Custom functions

```rust
use tinyexpr::Context;

let mut ctx = Context::new();
ctx.func1("square", |a| a * a);
ctx.closure("clamp", 3, true, |args| args[0].max(args[1]).min(args[2]));

let r = tinyexpr::compile("clamp(square(x), 0, 100)", &ctx);
```

## Grammar

```
list   = expr {"," expr}
expr   = term {("+" | "-") term}
term   = factor {("*" | "/" | "%") factor}
factor = power {"^" power}
power  = {("-" | "+")} base
base   = number | variable | function0 {"(" ")"}
       | function1 power
       | functionN "(" expr {"," expr} ")"
       | "(" list ")"
```

`^` is evaluated left-to-right by default (`2^2^3 == (2^2)^3 == 64`), same
as most spreadsheets. Unary minus binds tighter than `^`, so
`-2^2 == (-2)^2 == 4`.

Single-argument functions don't need parens (`sin x` works same as
`sin(x)`), and zero-argument functions can be called with or without
empty parens (`pi` and `pi()` both work).

## Built-in functions

`abs acos asin atan atan2 ceil cos cosh e exp fac floor ln log log10
ncr npr pi pow sin sinh sqrt tan tanh`

`log` is base-10 by default (matching upstream tinyexpr). Use
`Context::with_natural_log()` for a natural-log `log`.

## Project layout

```
src/lib.rs         the library: tokenizer, parser, evaluator, built-ins
tests/parity.rs    test suite
examples/          example.rs, example2.rs, repl.rs
```

## Running

```sh
cargo test
cargo run --example example
cargo run --example example2 -- "x+y"
cargo run --example repl
```
