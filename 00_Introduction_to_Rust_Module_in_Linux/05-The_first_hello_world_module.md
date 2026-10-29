# Hello World Module

We have learned enough for understanding the mechanism of how Rust works in
the Linux Kernel. It is time to write our first hello world module which is
powered by Rust.

## Code snippet

The code is not very long, and is very concise. I paste the holistic Rust part
in below:

```rust
#![allow(missing_docs)]

use kernel::prelude::*;

module! {
    type: HelloWorld,
    name: "hello_world",
    authors: ["Douglas Su"],
    description: "A simple hello world example",
    license: "GPL v2",
}

struct HelloWorld;

impl kernel::Module for HelloWorld {
    fn init(_module: &'static ThisModule) -> Result<Self> {
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
is identical to its C counterpart. The Kbuild system takes care of everything.

To build this project, run this command:

```
make KDIR="$LDD_ROOT/src/linux-stable" modules
```

`KDIR` points to a kernel source tree built with Rust support enabled.
The `rustc` used here must be exactly the same version as the one used to build
that kernel, otherwise `rustc` refuses to load the kernel's crates with an
error like `found crate core compiled by an incompatible version of rustc`.

## Notes on code

If you are a rustacean, you will be very familiar with this code. However, for
guys who have been a rustacean not very long, like me, I will pin some key points
out to help people understanding this code.

The first one worthy to mention is `#![no_std]` attribute which prevents Rust
from using standard library. Standard library is of course not accessible from
Kernel space. We have spent a lot of words before to talk about this topic.
`libcore` is a replacement (It seems word replacement is not very accurate) of
`libstd` for Rust in situations that no standard library is available.
`#![no_std]` attribute tells Rust compiler that this source file contains
nothing from standard library and doesn't need it at all. It is very suitable
for situations as development rust in the Kernel. This attribute is not written
in our code, because Kbuild adds it to every Rust module with
`-Zcrate-attr=no_std`.

In the same way, the unstable compiler features used by modules are enabled by
Kbuild, so no `#![feature(...)]` attribute is needed either.

`#![allow(missing_docs)]` silences the warning Kernel emits for items without
documentation comments.

`use kernel::prelude::*;` tells that use our own kernel crate.

`module! {}` is a Rust's [Function-like macro]. We will expand this topic later.

`struct HelloWorld;` is the body of our kernel module.

`impl kernel::Module for HelloWorld`: implement `kernel::Module` trait on our
own `HelloWorld` structure. This trait is the entrypoint of our kernel module.
`kernel::Module` trait contains an `init()` method which will be invoked during
the loading process of our module. It takes a `&'static ThisModule` referring to
our module. It behaves like C's `module_init()` function.

`impl Drop for HelloWorld`: implement `Drop` trait on our own `HelloWorld`
structure. `Drop` trait is a Rust's standard trait. Types with `Drop` trait
implemented will be deconstructed when its lifetime ended via calling `drop()`
method of `Drop` trait. `Drop` trait here is almost identical to C's
`module_exit()` function.

## Evolution

- Every Rust module used to start with `#![no_std]` and
  `#![feature(allocator_api, global_asm)]`. Kbuild adds both now.
- The module trait used to be `KernelModule` with `fn init() -> Result<Self>`.
  It was renamed to `kernel::Module`, and `init()` took the module name and
  `&'static ThisModule`. The name argument has since been dropped.
- The fields of `module! { }` used to be byte strings, e.g. `b"hello_world"`,
  and the author was given by `author`. See the next article for details.

[eg_01_hello_world]: ../eg_01_hello_world
[Function-like macro]: https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
