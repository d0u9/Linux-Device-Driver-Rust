# Compile Rust Compatible Kernel
## Prerequisites

### Install nightly Rust

For now, rust for Linux is still in development that means the project continuously takes advantages from latest rust compiler. So, a proper Rust compiler version is necessary to prevent from potential BUGs.

```
rustup override set $(scripts/min-tool-version.sh rustc)
```

### Install bindgen

`bindgen` is used to generate rust code from C side.

```
cargo install --locked --version $(scripts/min-tool-version.sh bindgen) bindgen
```

### Install standard library source

The Rust standard library source is required because the build system will cross-compile `core` and `alloc`.

```
rustup component add rust-src
```

### Install libclang

`libclang` (part of LLVM) is used by `bindgen` to understand the C code in the kernel, which means you will need a recent LLVM installed

Download pre-build binaries from here. Beware to choose appropriate architecture.

[https://github.com/llvm/llvm-project/releases](https://github.com/llvm/llvm-project/releases)

## Building Kernel

### Configure kernel.

A new option is added to enable/disable Rust abilities in Kernel in `General setup`.

```
General setup  --->
  Rust support
```

It is notable that is option is only shown if a `rustc` build system id detected.

### Building kernel

`GCC` is good for building Kernel. However, due to current experimental state, building Kernel with `Clang` or a complete `LLVM` toolchain is much better.


If `libclang` is not installed in a standard location, e.g. `/lib`, you have to locate it manually via `LIBCLANG_PATH` environment:

```
LIBCLANG_PATH=/path/to/libclang make -j LLVM=1 bzImage
```


