# Hello World Module

We have learned enough for understanding the mechanism of how Rust works in
the Linux Kernel. It is time to write our first hello world module which is
powered by Rust.

## Code snippet

The code is not very long, and is very concise. I paste the holistic Rust part
in below:

```rust
#![no_std]
#![feature(allocator_api, global_asm)]

use kernel::prelude::*;

module! {
    type: HelloWorld,
    name: b"hello_world",
    author: b"d0u9",
    description: b"A simple hello world example",
    license: b"GPL v2",
}

struct HelloWorld;

impl KernelModule for HelloWorld {
    fn init() -> Result<Self> {
        pr_info!("Hello world from rust!\n");

        Ok(HelloWorld)
    }
}

impl Drop for HelloWorld {
    fn drop(&mut self) {
        pr_info!("Bye world from rust!\n");
    }
}
```

You can find the entire project in [eg_01_hello_world] directory in this repo.
I omit the Makefile for building this module out of source tree. The Makefile
is identical to its C counterpart. The Kbuild system takes care of every thing.

To build this project, run this command:

```
LIBCLANG_PATH=/path/to/libclang make KERNELDIR=/rust/linux/kernel LLVM=1 modules
```

Again, the Linux Kernel must be compiled by clang and llvm beforehand instead of
GCC for our Rust example. GCC maybe works, but not tested in my circumstance.

`LIBCLANG_PATH` is an environment which points the `libclang` library. We have
talked `libclang` in previous chapter. Its duty is making code generation for
our target host.

## Notes on code

If you are a rustacean, you will be very familiar with this code. However, for
guys who has been a rustacean not very long, like me, I will pin some key points
out to help people understanding this code.

The first one worthy to mention is `#![no_std]` attribute which prevents Rust
from using standard library. Standard library is of course not accessible from
Kernel space. We have spent a lot words before to talk about this topic.
`libcore` is a replacement (It seems word replacement is not very accurate) of
`libstd` for Rust in situations that no standard library is available.
`#![no_std]` attribute tells Rust compiler that this source file contains
nothing from standard library and doesn't need it at all. It is very suitable
for situations as development rust in the Kernel.

`#![feature(allocator_api, global_asm)]` is a Rust's attribute used to enable
unstable or experimental compiler features. A list of unstable features can
be found here: https://doc.rust-lang.org/unstable-book/index.html

`use kernel::prelude::*;` tells that use our own kernel crate.

`module! {}` is a Rust's [Function-like macro]. We will expand this topic later.

`struct HelloWorld;` is the body of our kernel module.

`impl KernelModule for HelloWorld`: implement `KernelModule` trait on our own
`HelloWorld` structure. This trait is the entrypoint of our kernel module.
`KernelModule` trait contains an `init()` method which will be invoked during
the loading process of our module. It behaves like C's `module_init()` function.

`impl Drop for HelloWorld`: implement `Drop` trait on our own `HelloWorld`
structure. `Drop` trait is a Rust's standard trait. Types with `Drop` trait
implemented will be deconstructed when its lifetime ended via calling `drop()`
method of `Drop` trait. `Drop` trait here almost an identical to C's
`module_exit()` function.

[eg_01_hello_world]: ../eg_01_hello_world
[Function-like macro]: https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
