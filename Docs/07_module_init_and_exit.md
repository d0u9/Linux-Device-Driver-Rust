# Module Init and Exit

## An exmaple of C module

For C module, it has a skeleton like:

```C
#include <linux/types.h>
#include <linux/module.h>

static u32 uint_param = 1;
module_param(uint_param, uint, S_IRUGO);

struct example_module {
    u32 version;
};
static struct example_module module_struct;

static int __init m_init(void)
{
    module_struct.version = uint_param;
    pr_info("Hello World! uint_prarm=%u\n", module_struct.version);
    return 0;
}

static void __exit m_exit(void)
{
    pr_info("Bye World!\n");
}

module_init(m_init);
module_exit(m_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Douglas Su");
MODULE_DESCRIPTION("An example module");
```

## Rewrite C module in Rust

Our C module has a structure named `exmaple_module`, which describes our module.
Single field, `version`, in `struct exmaple_module` has a type of `int` and
records the current version of our module. To rewrite this part in Rust, we
declare a Rust structure with an int `version` member which is identical to
C's counterpart:

```rust
struct ExampleModule {
    version: u32,
}
```

Next, we need to mimic the ability of `module_init()` function. This is
implemented via a member function `init()`:

```rust
impl KernelModule for ExampleModule {
    fn init() -> Result<Self> {
        let lock = THIS_MODULE.kernel_param_lock();
        let module = ExampleModule { version: *uint_param.read(&lock) };

        pr_info!("Hello World! uint_prarm={}\n", module.version);
        Ok(module)
    }
}
```

`module_exit()` is replaced by `Drop` trait:

```rust
impl Drop for ExampleModule {
    fn drop(&mut self) {
        pr_info!("Bye World\n");
    }
}
```

## Author, Description, License and etc.

All of these macros listed below are now members of `module!{ }` macro:

```c
MODULE_LICENSE("GPL");
MODULE_AUTHOR("Douglas Su");
MODULE_DESCRIPTION("An example module");
```

In Rust:

```rust
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
```

## Make our `struct ExampleModule` be the module entry point

Just assign structure name to type field in `module!{ }` macro.

```rust
module! {
    type: ExampleModule,
    ...
}
```

## The whole rust code

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
        Ok(module)
    }
}

impl Drop for ExampleModule {
    fn drop(&mut self) {
        pr_info!("Bye World\n");
    }
}
```
