# How is rust module loaded into Kernel

We have talked the Loading process of Linux module file. For rust, the process
is same because modules compiled from either Rust or C are all ELF dynamic
linked files. The should have same ABI (Application Binary Interface) which
is compatible with Kernel's dynamic linker.

The `module!{ }` macro we have talked in previous section is responsible for
bootstrapping Rust kernel modules. It reads fields defined in it and expands
them to definitions of variables, functions and structures.

For a simple rust module:

```rust
#![no_std]
#![feature(allocator_api, global_asm)]

use kernel::prelude::*;

module! {
    type: ExampleModule,
    name: b"example_module",
    author: b"Douglas Su",
    description: b"An example module",
    license: b"GPL v2",
    params: {
        uint_param: u32 {
            default: 1,
            permissions: 0o644,
            description: b"uint parameter",
        },
    },
}

struct ExampleModule {
    version: u32,
}

impl KernelModule for ExampleModule {
    fn init() -> Result<Self> {
        let lock = THIS_MODULE.kernel_param_lock();
        let module = ExampleModule { version: *uint_param.read(&lock) };
        pr_info!("Hello World! uint_prarm={}\n", module.version);
        pr_info!("The address of module parameter uint_param: {:p}\n", uint_param.read(&lock));
        Ok(module)
    }
}

impl Drop for ExampleModule {
    fn drop(&mut self) {
        pr_info!("Bye World\n");
    }
}
```

These piece of code will be expanded to code below during compiling time by
Rust's macro processor.

```rust
const __LOG_PREFIX: &[u8] = b"example_module\0";
static mut __MOD: Option<ExampleModule> = None;

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
    match <ExampleModule as kernel::KernelModule>::init() {
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
pub static __example_module_author: [u8; 18] = *b"author=Douglas Su\0";

#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __example_module_description: [u8; 30] = *b"description=An example module\0";

#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __example_module_license: [u8; 15] = *b"license=GPL v2\0";

#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __example_module_parmtype_uint_param: [u8; 24] = *b"parmtype=uint_param:u32\0";

#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __example_module_parm_uint_param: [u8; 31] = *b"parm=uint_param:uint parameter\0";
static mut __example_module_uint_param_value: u32 = 1;
struct __example_module_uint_param;
impl __example_module_uint_param {
    fn read<'lck>(
        &self,
        lock: &'lck kernel::KParamGuard,
    ) -> &'lck <u32 as kernel::module_param::ModuleParam>::Value {
        unsafe {
            <u32 as kernel::module_param::ModuleParam>::value(&__example_module_uint_param_value)
        }
    }
}
const uint_param: __example_module_uint_param = __example_module_uint_param;

#[repr(transparent)]
struct __example_module_uint_param_RacyKernelParam(kernel::bindings::kernel_param);
unsafe impl Sync for __example_module_uint_param_RacyKernelParam {}

#[cfg(MODULE)]
const __example_module_uint_param_name: *const kernel::c_types::c_char =
    b"uint_param\0" as *const _ as *const kernel::c_types::c_char;

#[link_section = "__param"]
#[used]
static __example_module_uint_param_struct: __example_module_uint_param_RacyKernelParam =
    __example_module_uint_param_RacyKernelParam(kernel::bindings::kernel_param {
        name: __example_module_uint_param_name,

    #[cfg(MODULE)]
    mod_: unsafe { &kernel::bindings::__this_module as *const _ as *mut _ },
    ops: unsafe { &kernel::module_param::PARAM_OPS_U32 }
        as *const kernel::bindings::kernel_param_ops,
    perm: 0o644,
    level: -1,
    flags: 0,
    __bindgen_anon_1: kernel::bindings::kernel_param__bindgen_ty_1 {
        arg: unsafe { &__example_module_uint_param_value } as *const _
            as *mut kernel::c_types::c_void,
    },
});

struct ExampleModule {
    version: u32,
}

impl KernelModule for ExampleModule {
    fn init() -> Result<Self> {
        Ok(module)
    }
}

impl Drop for ExampleModule {
    fn drop(&mut self) { };
    }
}

```

## How Rust module source is compiled into object

It is intuitive that object is compiled from C source file, and GNU make command
takes this as an implicit rule. However, for Rust code, Kbuild doesn't know
how to handle it and doesn't know that rust source can be compiled into object.

To make Kbuild understand Rust, below lines are added in `scripts/Makefile.build`:


```makefile
rustc_cross_flags := --target=$(realpath $(KBUILD_RUSTC_TARGET))

quiet_cmd_rustc_o_rs = $(RUSTC_OR_CLIPPY_QUIET) $(quiet_modtag) $@
      cmd_rustc_o_rs = \
	RUST_MODFILE=$(modfile) \
	$(RUSTC_OR_CLIPPY) $(rustc_flags) $(rustc_cross_flags) \
		--extern alloc --extern kernel \
		--crate-type rlib --out-dir $(obj) -L $(objtree)/rust/ \
		--crate-name $(patsubst %.o,%,$(notdir $@)) $<; \
	mv $(obj)/$(subst .o,,$(notdir $@)).d $(depfile); \
	sed -i '/^\#/d' $(depfile)

$(obj)/%.o: $(src)/%.rs FORCE
	$(call if_changed_dep,rustc_o_rs)
```

## `init_module` and `cleanup_module` symbols

From the discussions in previous chapter, two important symbols `init_module`
and `cleanup_module` must be defined, and they will be referenced my
`struct module` which later be used to do initialization and cleanup works of
our module.

Other than than the symbol name, the ABI also plays an important role. Functions
export `init_module` and `cleanup_module` symbols must obey the ABI of standard
C. For Rust code, this is implemented by decorating functions with `extern "C"`.

Functions marked as `extern "C"` will be compile by using standard C ABI and
in turn be invoked directly by C code. Usually, `#[no_mangle]` annotation is
needed to tell the Rust compiler not to mangle the name of this function.
Mangling is when a compiler changes the name we’ve given a function to a
different name that contains more information for other parts of the compilation
process to consume but is less human readable.

For example, we a function in Rust:

```rust
#[no_mangle]
fn my_add(a: i32, b: i32) -> i32 { a + b }
```

If it is decorated with `#[no_mangle]` annotation, function symbol generated
will be something like:

```
0000000100001680 T _my_add
```

However, by default, Rust will mangle symbols it generated, and produce
something like this:

```
0000000100001680 t __ZN2z16my_add17hb64f7e7eb99a7564E
```

Back to our Rust code. After macro expansion, two functions are generated, and
be annotated with `#[no_mangle]`.

```rust
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
```

After compiling this Rust into object, `init_module` and `cleanup_module`
symbols is defined as strong symbols which will be used in `postmod`. When the
module file is loaded, kernel's module loader will find these two functions and
use them to init/cleanup module.

Two functions, `__init()` and `__exit()`, are invoked in `init_module()`
and `cleanup_module()` respectively. `__init()` function in turn calls
our modulel structure's member fucntion `ExampleModule::init()`, in which
`struct ExampleModule` is initialized.

The `__exit()` function desn't nothing. That is because the `Drop` trait has
been implemented for our `struct ExampleModule`, and the `drop()` function will
be automaticall invoked when instace of `struct ExampleModule` goes out of its
scope.

## Build `modinfo` section

We have talked that in module's ELF file, a special section, `.modinfo`, is used
to store information about this module. It includes the name, paramter type,
parameter name, license, author, etc.

For rust module, we have to obey this rule. The expanded rust code has self
expained:

```rust
#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __example_module_author: [u8; 18] = *b"author=Douglas Su\0";

#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __example_module_description: [u8; 30] = *b"description=An example module\0";

#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __example_module_license: [u8; 15] = *b"license=GPL v2\0";

#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __example_module_parmtype_uint_param: [u8; 24] = *b"parmtype=uint_param:u32\0";

#[cfg(MODULE)]
#[link_section = ".modinfo"]
#[used]
pub static __example_module_parm_uint_param: [u8; 31] = *b"parm=uint_param:uint parameter\0";
```

A special directive `#[link_section()]` tells Rust compiler which section the
data or code it annotated will be placed in. For our example, the section
is `.modinfo`.

## Kernel module

Module parameter is defined in `module!{ }` macro:

```rust
module! {
    ...
    params: {
        uint_param: u32 {
            default: 1,
            permissions: 0o644,
            description: b"uint parameter",
        },
    },
}
```

After macro expansion:

```rust
static mut __example_module_uint_param_value: u32 = 1;

struct __example_module_uint_param;

impl __example_module_uint_param {
    fn read<'lck>(
        &self,
        lock: &'lck kernel::KParamGuard,
    ) -> &'lck <u32 as kernel::module_param::ModuleParam>::Value {
        unsafe {
            <u32 as kernel::module_param::ModuleParam>::value(&__example_module_uint_param_value)
        }
    }
}

const uint_param: __example_module_uint_param = __example_module_uint_param;
```

The actual variable in which our value is stored is mangled to
`__example_module_uint_param_value`. Then, a new structure
`struct __example_module_uint_param;` is defined and implemented a member
function, say `read()`, on it. The function takes the parameter lock as its
variable to prevent from concurrent acesses to our variable.

We re-defined a new variable which has the same name as our parameter variable,
i.e. `uint_param`, but has the type of `__example_module_uint_param`. Later,
module programmer can use this variable as:

```rust
let lock = THIS_MODULE.kernel_param_lock();
pr_info!("uint_param = {:p}\n", uint_param.read(&lock));
```

## Build `__param` section

Previous chapter introduces `__param` section in ELF file and how module loader
process this section during loading time.

For Rust code, we have to build this section consistent with its C counterpart.
The `module!{ }` macro takes care of this, and you can find codes generated
from this macro like this:

```rust
#[repr(transparent)]
struct __example_module_uint_param_RacyKernelParam(kernel::bindings::kernel_param);
unsafe impl Sync for __example_module_uint_param_RacyKernelParam {}

#[cfg(MODULE)]
const __example_module_uint_param_name: *const kernel::c_types::c_char =
    b"uint_param\0" as *const _ as *const kernel::c_types::c_char;

#[link_section = "__param"]
#[used]
static __example_module_uint_param_struct: __example_module_uint_param_RacyKernelParam =
    __example_module_uint_param_RacyKernelParam(kernel::bindings::kernel_param {
        name: __example_module_uint_param_name,

    #[cfg(MODULE)]
    mod_: unsafe { &kernel::bindings::__this_module as *const _ as *mut _ },
    ops: unsafe { &kernel::module_param::PARAM_OPS_U32 }
        as *const kernel::bindings::kernel_param_ops,
    perm: 0o644,
    level: -1,
    flags: 0,
    __bindgen_anon_1: kernel::bindings::kernel_param__bindgen_ty_1 {
        arg: unsafe { &__example_module_uint_param_value } as *const _
            as *mut kernel::c_types::c_void,
    },
});
```

`struct __example_module_uint_param_RacyKernelParam` is a wrapper of
`kernel::bindings::kernel_param`. `kernel_param` in kernel describes single
parameter in module. We have detailed this in previous chapter, and for
convenience I paste it here again:

```c
struct kernel_param {
    const char *name;
    struct module *mod;
    const struct kernel_param_ops *ops;
    const u16 perm;
    s8 level;
    u8 flags;
    union {
            void *arg;
            const struct kparam_string *str;
            const struct kparam_array *arr;
    };
};
```

The Rust part is a mimic of C structure which initialized each field with
Rust code, either in FFI or rust variable. The structure then is compiled into
`__param` section due the the `#[link_section = "__param"]` annotation.

However, the `ops` field is a little bit special. It is assigned
`&kernel::module_param::PARAM_OPS_U32`.

This variable is defined via a macro `make_param_ops!()`, and expanded during
kernel compiling time. For `PARAM_OPS_U32`, it is define as:

```rust
make_param_ops!(
    /// Rust implementation of [`kernel_param_ops`](../../../include/linux/moduleparam.h)
    /// for [`u32`].
    PARAM_OPS_U32,
    u32
);
```


