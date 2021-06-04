# The module macro

Every (almost?) Rust driver starts by a macro `module! { }`. This is a
[Function-like macro] that are invoked like a function call but suffixed with
the macro invocation operator `!`.

It is noticeable that macros in Rust is not identical to macros in C. For macros
in C language, it works almost in a string substitution style. C's macros
expands strings recursively. However, for Rust's macros, they are categorised
into two types: declarative macros with `macro_rules!` and three kinds of
procedural macros. For declarative macros, they [hygienic macros]. For
procedural macros, they are [not hygiene]. All in all, Rust's macros are
string substitution. For our `module! {}` macro, a function-like macro, it is
a procedural macro.

## A simple example of Rust function-like macro

Due to the differences to traditional macros, it is good to illustrate the basic
process of implementing Rust's function-like macro by an example. And, rust for
kernel uses bare `rustc` compiler instead of `cargo`, it is a little bit complex
to build and run such an example without `cargo`'s help. I will first build the
example by `cargo` and then build it directly by `rustc`.

### With the help of Cargo

Create a cargo package:

```bash
cargo init my_macro
```

Append lines below to your `Cargo.toml` file:

```toml
[[bin]]
name = "my_bin"
path = "src/main.rs"

[lib]
name = "my_macros"
path = "src/lib/lib.rs"
proc-macro = true
```

The line `proc-macro = true` is very important to our example. It says that
our library has `proc-macro` macros.

Crate `src/main.rs` source file:

```rust
extern crate my_macros;
use my_macros::answer_fn;

answer_fn!();

fn main() {
    print!("This is my answer: {}\n", answer());
}
```

Create `src/lib/lib.rs` file:

```rust
extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn answer_fn(_item: TokenStream) -> TokenStream {
    println!("This line is printed in macro");
    "
        fn answer() -> u32 {
            let k = 1234;
            print!(\"The answer is: {}\n\", k);
            k
        }
    ".parse().unwrap()
}
```

Run our example:

```bash
Cargo run
```

The output is:

```
   Compiling my_macro v0.1.0 (/tmp/my_macro)
This line is printed in macro
    Finished dev [unoptimized + debuginfo] target(s) in 0.19s
     Running `target/debug/my_bin`
The answer is: 1234
This is my answer: 1234
```

Are you confused by the line `This line is printed in macro`? Would it be
printed after the line `Running xxxx`? That is the key of macro.




[Function-like macro]: https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
[hygienic macros]: https://en.wikipedia.org/wiki/Hygienic_macro
[not hygiene]: https://doc.rust-lang.org/reference/procedural-macros.html#procedural-macro-hygiene
