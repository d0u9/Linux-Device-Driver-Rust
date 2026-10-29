# The module macro

Every (almost?) Rust driver starts by a macro `module! { }`. This is a
[Function-like macro] that are invoked like a function call but suffixed with
the macro invocation operator `!`.

It is noticeable that macros in Rust is not identical to macros in C. For macros
in C language, it works almost in a string substitution style. C's macros
expands strings recursively. However, for Rust's macros, they are categorised
into two types: declarative macros with `macro_rules!` and three kinds of
procedural macros. For declarative macros, they are [hygienic macros]. For
procedural macros, they are [not hygiene]. All in all, Rust's macros are not
string substitution, they work on tokens. For our `module! {}` macro, a function-like macro, it is
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
use my_macros::answer_fn;

answer_fn!();

fn main() {
    print!("This is my answer: {}\n", answer());
}
```

Create `src/lib/lib.rs` file:

```rust
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
cargo run
```

The output is:

```
   Compiling my_macro v0.1.0 (/tmp/my_macro)
This line is printed in macro
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.19s
     Running `target/debug/my_bin`
The answer is: 1234
This is my answer: 1234
```

Are you confused by the line `This line is printed in macro`? Would it be
printed after the line `Running xxxx`? That is the key of macro.

### Use `rustc` directly

For Rust in Kernel, we use bare `rustc` compiler instead of redundant `cargo`
tool.

For our example before, to compile it in an executable, compose a `Makefile`:

```makefile
BUILD_DIR ?= $(shell mkdir -p build; echo build)
BIN := my_bin
BIN_SRC := src/main.rs
MACRO_LIB := my_macros
MACRO_LIB_FILE := $(addprefix lib,$(addsuffix .so,$(MACRO_LIB)))
MACRO_SRC := src/lib/lib.rs
MACRO_FLAGS := --edition=2021 \
			   --emit=link \
			   --extern proc_macro \
			   --crate-type proc-macro \
			   --crate-name $(MACRO_LIB) \
			   --out-dir $(BUILD_DIR)

$(BUILD_DIR)/$(BIN): $(BIN_SRC) $(MACRO_LIB_FILE)
	rustc --edition=2021 -o $@ --extern $(MACRO_LIB) -L $(BUILD_DIR) $<

$(MACRO_LIB_FILE): $(MACRO_SRC)
	rustc $(MACRO_FLAGS) $^

.PHONY: clean
clean:
	rm -fr $(BUILD_DIR)
```

The Makefile makes compiling procedure more clearly than cargo which has a
simple one-shot operation. The compiling process contains two phases: 1) build
proc_macro library; 2) build our executable binary.

Bare `rustc` defaults to the 2015 edition, so `--edition=2021` is passed, as
Kernel does. Cargo passes the edition from `Cargo.toml` instead.

The proc_macro is special, that all proc_macros are compiled into dynamic
libraries and then are fed to rustc compiler. The string "This line is printed
in macro" is printed in our proc_macro function during the compiling time. This
means proc_macro functions are invoked by compiler other than our code.

## Real module macro

We have understood what is a Rust's procedural macro and how it is expanded
by an example. It is time to play around with real module macro.

For writing kernel modules in C, it always starts with two functions(they are
actually C macros), `module_init()` and `module_exit()`, and some auxiliary
macros, e.g. `MODULE_LICENSE()`, `MODULE_AUTHOR()`, `MODULE_DESCRIPTION()`.

For kernel modules in Rust, a macro named `module!{ }` is used to describe
the behavior what our module will have.

Take `samples/rust/rust_minimal.rs` in Kernel source tree as an example:

```rust
module! {
    type: RustMinimal,
    name: "rust_minimal",
    authors: ["Rust for Linux Contributors"],
    description: "Rust minimal sample",
    license: "GPL",
    params: {
        test_parameter: i64 {
            default: 1,
            description: "This parameter has a default of 1",
        },
    },
}
```

This macro will be expanded during compiling time. For curious readers who
wonder what is the results of output, Kbuild has a `.rsi` target which prints
the source after macro expansion. Run it in Kernel source tree:

```bash
make samples/rust/rust_minimal.rsi
```

The generated content of `module!` after macro expansion looks like:

```rust
#[doc = r" The module name."]
#[doc = r""]
#[doc = r" Used by the printing macros, e.g. [`info!`]."]
const __LOG_PREFIX: &[u8] = c"rust_minimal".to_bytes_with_nul();
static THIS_MODULE: ::kernel::ThisModule = unsafe {
    extern "C" {
        static __this_module: ::kernel::types::Opaque<::kernel::bindings::module>;
    };
    ::kernel::ThisModule::from_ptr(__this_module.get())
};
#[doc = r" The `LocalModule` type is the type of the module created by `module!`,"]
#[doc = r" `module_pci_driver!`, `module_platform_driver!`, etc."]
type LocalModule = RustMinimal;
impl ::kernel::ModuleMetadata for RustMinimal {
    const NAME: &'static ::kernel::str::CStr = c"rust_minimal";
}
#[doc(hidden)]
mod __module_init {
    mod __module_init {
        use pin_init::PinInit;
        #[doc = r#" The "Rust loadable module" mark."#]
        #[used(compiler)]
        static __IS_RUST_MODULE: () = ();
        static mut __MOD: ::core::mem::MaybeUninit<super::super::LocalModule> =
            ::core::mem::MaybeUninit::uninit();
        #[doc = r" # Safety"]
        #[doc = r""]
        #[doc = r" This function must not be called after module initialization, because it may be"]
        #[doc = r" freed after that completes."]
        #[no_mangle]
        #[link_section = ".init.text"]
        pub unsafe extern "C" fn init_module() -> ::kernel::ffi::c_int {
            unsafe { __init() }
        }
        #[used(compiler)]
        #[link_section = ".init.data"]
        static __UNIQUE_ID___addressable_init_module: unsafe extern "C" fn() -> i32 = init_module;
        #[no_mangle]
        #[link_section = ".exit.text"]
        pub extern "C" fn cleanup_module() {
            unsafe { __exit() }
        }
        #[used(compiler)]
        #[link_section = ".exit.data"]
        static __UNIQUE_ID___addressable_cleanup_module: extern "C" fn() = cleanup_module;
        #[doc = r" # Safety"]
        #[doc = r""]
        #[doc = r" This function must only be called once."]
        unsafe fn __init() -> ::kernel::ffi::c_int {
            let initer = <super::super::LocalModule as ::kernel::InPlaceModule>::init(
                &super::super::THIS_MODULE,
            );
            match unsafe { initer.__pinned_init(__MOD.as_mut_ptr()) } {
                Ok(m) => 0,
                Err(e) => e.to_errno(),
            }
        }
        #[doc = r" # Safety"]
        #[doc = r""]
        #[doc = r" This function must"]
        #[doc = r" - only be called once,"]
        #[doc = r" - be called after `__init` has been called and returned `0`."]
        unsafe fn __exit() {
            unsafe {
                __MOD.assume_init_drop();
            }
        }
        #[link_section = ".modinfo"]
        #[used(compiler)]
        pub static __RUST_MINIMAL_1: [u8; 35usize] = *b"author=Rust for Linux Contributors\0";
        #[link_section = ".modinfo"]
        #[used(compiler)]
        pub static __RUST_MINIMAL_3: [u8; 32usize] = *b"description=Rust minimal sample\0";
        #[link_section = ".modinfo"]
        #[used(compiler)]
        pub static __RUST_MINIMAL_5: [u8; 12usize] = *b"license=GPL\0";
    }
}
mod module_parameters {
    #[link_section = ".modinfo"]
    #[used(compiler)]
    pub static __RUST_MINIMAL_8: [u8; 28usize] = *b"parmtype=test_parameter:i64\0";
    #[link_section = ".modinfo"]
    #[used(compiler)]
    pub static __RUST_MINIMAL_10: [u8; 54usize] =
        *b"parm=test_parameter:This parameter has a default of 1\0";
    #[allow(non_upper_case_globals)]
    pub(crate) static test_parameter: ::kernel::module_param::ModuleParamAccess<i64> =
        ::kernel::module_param::ModuleParamAccess::new(1);
    const _: () = {
        #[allow(non_upper_case_globals)]
        #[link_section = "__param"]
        #[used(compiler)]
        static __rust_minimal_test_parameter_struct: ::kernel::module_param::KernelParam =
            ::kernel::module_param::KernelParam::new(::kernel::bindings::kernel_param {
                name: kernel::str::as_char_ptr_in_const_context(if true {
                    c"test_parameter"
                } else {
                    c"rust_minimal.test_parameter"
                }),
                mod_: unsafe { core::ptr::from_ref(&::kernel::bindings::__this_module).cast_mut() },
                ops: core::ptr::from_ref(&::kernel::module_param::PARAM_OPS_I64),
                perm: 0,
                level: -1,
                flags: 0,
                __bindgen_anon_1: ::kernel::bindings::kernel_param__bindgen_ty_1 {
                    arg: test_parameter.as_void_ptr(),
                },
            });
    };
}
```

This is a very long code. I will detail this in next article.

## Evolution

- `module! { }` used to be parsed by a hand-written token parser, and its
  fields were byte strings, e.g. `name: b"rust_minimal"`. The macro is parsed
  with the `syn` crate now, and its fields are plain string literals,
  converted to C strings by the macro itself. Fields such as `name`,
  `license` and `authors` are also checked to be ASCII.
- `author` took a single author. `authors` takes a list, and each element
  becomes an `author=` entry in `.modinfo`, like multiple `MODULE_AUTHOR()` in
  C.
- Parameters used to have a `permissions` field. It is gone, and the
  permission of a parameter is always 0.
- The module instance used to be kept in `static mut __MOD: Option<T>`, and
  C types came from `kernel::c_types`. Now it is kept in
  `MaybeUninit<T>` and initialized in place through `InPlaceModule`, and C
  types come from `kernel::ffi`.
- The expanded code used to be inspected with rustc's
  `-Zunstable-options --pretty=expanded`. Kbuild provides the `.rsi` target
  for it now.

[Function-like macro]: https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
[hygienic macros]: https://en.wikipedia.org/wiki/Hygienic_macro
[not hygiene]: https://doc.rust-lang.org/reference/procedural-macros.html#procedural-macro-hygiene
