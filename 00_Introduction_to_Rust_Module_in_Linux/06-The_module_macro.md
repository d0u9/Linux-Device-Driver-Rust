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
MACRO_FLAGS := --emit=obj,link \
			   --extern proc_macro \
			   --crate-type proc-macro \
			   --crate-name $(MACRO_LIB) \
			   --out-dir $(BUILD_DIR)

$(BUILD_DIR)/$(BIN): $(BIN_SRC) $(MACRO_LIB_FILE)
	rustc -o $@ --extern $(MACRO_LIB) -L $(BUILD_DIR) $<

$(MACRO_LIB_FILE): $(MACRO_SRC)
	rustc $(MACRO_FLAGS) $^

.PHONY: clean
clean:
	rm -fr $(BUILD_DIR)
```

The Makefile makes compiling procedure more clearly than cargo which has an
simple one-shot operation. The compiling process contains two phases: 1) build
macro_proc library; 2) build our executable binary.

The proc_macro is special, that all proc_macros are compiled into a dynamic
libraries and then is feed to rustc compiler. The string "This line is printed
in macro" is printed in our macro_proc function during the compiling time. This
means macro_proc functions are invoked by compiler other than our code.

## Real module macro

We have understanded what is a Rust's procedural macro and how it is expaned
by an example. It is time to play around with real module macro.

For writing kernel modules in C, it always starts with two functions(they are
actually C macros), `module_init()` and `module_exit()`, and some auxiliary
macros, e.g. `MODULE_LICENSE()`, `MODULE_AUTHOR()`, `MODULE_DESCRIPTION()`.

For kernel modules in Rust, a macro named `module!{ }` is used to describe
the behavior what our module will have.

```rust
module! {
    type: RustModuleParameters,
    name: b"rust_module_parameters",
    author: b"Rust for Linux Contributors",
    description: b"Rust module parameters sample",
    license: b"GPL v2",
    params: {
        my_bool: bool {
            default: true,
            permissions: 0,
            description: b"Example of bool",
        },
        my_i32: i32 {
            default: 42,
            permissions: 0o644,
            description: b"Example of i32",
        },
    },
}
```

This macro will be expanded during compiling time. For curious readers who
wonder what is the results of output, it is convenient to use rustc's
`--pretty=expanded` option to inspect. `--pretty=expanded` is an option that is
only available when `-Zunstable-options` is also enabled.

The generated content after macro expansion looks like:

```rust
/// The module name.
///
/// Used by the printing macros, e.g. [`info!`].
const __LOG_PREFIX: &[u8] = b"test_module\0";
static mut __MOD: Option<RustModuleParameters> = None;
#[cfg(MODULE)]
static THIS_MODULE: kernel::ThisModule =
    unsafe { kernel::ThisModule::from_ptr(&kernel::bindings::__this_module as *const _ as *mut _) };
#[cfg(MODULE)]
#[no_mangle]
pub extern "C" fn init_module() -> kernel::c_types::c_int {
    __init()
}
#[cfg(MODULE)]
#[no_mangle]
pub extern "C" fn cleanup_module() {
    __exit()
}
fn __init() -> kernel::c_types::c_int {
    match <RustModuleParameters as kernel::KernelModule>::init() {
        Ok(m) => {
            unsafe {
                __MOD = Some(m);
            }
            return 0;
        }
        Err(e) => {
            return e.to_kernel_errno();
        }
    }
}
fn __exit() {
    unsafe {
        __MOD = None;
    }
}
#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __test_module_author: [u8; 35] = *b"author=Rust for Linux Contributors\0";
#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __test_module_description: [u8; 42] = *b"description=Rust module parameters sample\0";
#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __test_module_license: [u8; 15] = *b"license=GPL v2\0";
#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __test_module_parmtype_my_bool: [u8; 22] = *b"parmtype=my_bool:bool\0";
#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __test_module_parm_my_bool: [u8; 29] = *b"parm=my_bool:Example of bool\0";
static mut __test_module_my_bool_value: bool = true;
struct __test_module_my_bool;
impl __test_module_my_bool {
    fn read(&self) -> &<bool as kernel::module_param::ModuleParam>::Value {
        unsafe { <bool as kernel::module_param::ModuleParam>::value(&__test_module_my_bool_value) }
    }
}
const my_bool: __test_module_my_bool = __test_module_my_bool;
#[repr(transparent)]
struct __test_module_my_bool_RacyKernelParam(kernel::bindings::kernel_param);
unsafe impl Sync for __test_module_my_bool_RacyKernelParam {}
#[cfg(MODULE)]
const __test_module_my_bool_name: *const kernel::c_types::c_char =
    b"my_bool\0" as *const _ as *const kernel::c_types::c_char;
#[link_section = "__param"]
#[used]
static __test_module_my_bool_struct: __test_module_my_bool_RacyKernelParam =
    __test_module_my_bool_RacyKernelParam(kernel::bindings::kernel_param {
        name: __test_module_my_bool_name,

        #[cfg(MODULE)]
        mod_: unsafe { &kernel::bindings::__this_module as *const _ as *mut _ },
        ops: unsafe { &kernel::module_param::PARAM_OPS_BOOL }
            as *const kernel::bindings::kernel_param_ops,
        perm: 0,
        level: -1,
        flags: 0,
        __bindgen_anon_1: kernel::bindings::kernel_param__bindgen_ty_1 {
            arg: unsafe { &__test_module_my_bool_value } as *const _
                as *mut kernel::c_types::c_void,
        },
    });
#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __test_module_parmtype_my_i32: [u8; 20] = *b"parmtype=my_i32:i32\0";
#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __test_module_parm_my_i32: [u8; 27] = *b"parm=my_i32:Example of i32\0";
static mut __test_module_my_i32_value: i32 = 267390960;
struct __test_module_my_i32;
impl __test_module_my_i32 {
    fn read<'lck>(
        &self,
        lock: &'lck kernel::KParamGuard,
    ) -> &'lck <i32 as kernel::module_param::ModuleParam>::Value {
        unsafe { <i32 as kernel::module_param::ModuleParam>::value(&__test_module_my_i32_value) }
    }
}
const my_i32: __test_module_my_i32 = __test_module_my_i32;
#[repr(transparent)]
struct __test_module_my_i32_RacyKernelParam(kernel::bindings::kernel_param);
unsafe impl Sync for __test_module_my_i32_RacyKernelParam {}
#[cfg(MODULE)]
const __test_module_my_i32_name: *const kernel::c_types::c_char =
    b"my_i32\0" as *const _ as *const kernel::c_types::c_char;
#[link_section = "__param"]
#[used]
static __test_module_my_i32_struct: __test_module_my_i32_RacyKernelParam =
    __test_module_my_i32_RacyKernelParam(kernel::bindings::kernel_param {
        name: __test_module_my_i32_name,

        #[cfg(MODULE)]
        mod_: unsafe { &kernel::bindings::__this_module as *const _ as *mut _ },
        ops: unsafe { &kernel::module_param::PARAM_OPS_I32 }
            as *const kernel::bindings::kernel_param_ops,
        perm: 0o644,
        level: -1,
        flags: 0,
        __bindgen_anon_1: kernel::bindings::kernel_param__bindgen_ty_1 {
            arg: unsafe { &__test_module_my_i32_value } as *const _ as *mut kernel::c_types::c_void,
        },
    });
```

This is a very long code. I will detail this in next article.

[Function-like macro]: https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
[hygienic macros]: https://en.wikipedia.org/wiki/Hygienic_macro
[not hygiene]: https://doc.rust-lang.org/reference/procedural-macros.html#procedural-macro-hygiene
