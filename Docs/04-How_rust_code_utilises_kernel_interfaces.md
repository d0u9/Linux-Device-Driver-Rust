# How rust code utilises Kernel's interfaces instead of standard library

## What is the difference between and kernel interfaces

For normal userspace applications, Rust uses its standard library `libstd` which
in turn utilises standard system calls or system interfaces provided by a
specific OS. However, for circumstances that has no operation system backup,
these standard interfaces is not accessible. Programming in Kernel is one of
such situations in which Rust must be feed with a set of low leveled API for its
functionality. Rust's `libcore` is a tiny, dependency-free library, on which
`libstd` is built, which provides primitive blocks of all Rust code. `libcore`
library links to no upstream libraries, no system libraries, and no libc at all.

Currently, in Kernel, `libcore` for Rust is compiled from official's libcore
source code. That is why we have to run `rustup component add rust-src` before
any compilation.

Other than `libcore`, `alloc` crate provides supports of basic data structures
and memory allocation management. Rust benefits itself from this library for
allocating memory directly from Kernel's `kalloc` interfaces. The familiar
types, such as `rc`, `slice`, are exported from this `alloc` library. Rust's
`libstd` re-exports these types in it to make `libstd` a concrete library.

## Build `libcore` and `alloc` in Kernel.

For `core.o` and `alloc.o` targets:

```
.SECONDEXPANSION:
$(objtree)/rust/core.o: private skip_clippy = 1
$(objtree)/rust/core.o: $$(RUST_LIB_SRC)/core/src/lib.rs FORCE
	$(call if_changed_dep,rustc_library)
```

```
$(objtree)/rust/alloc.o: private skip_clippy = 1
$(objtree)/rust/alloc.o: $$(RUST_LIB_SRC)/alloc/src/lib.rs \
    $(objtree)/rust/compiler_builtins.o FORCE
	$(call if_changed_dep,rustc_library)
```

The `$$(RUST_LIB_SRC)` is the location where Rust's lib source files residents.
It was detected via these two commands:

```
rustc_sysroot = $(shell $(RUSTC) $(rustc_flags) --print sysroot)
RUST_LIB_SRC ?= $(rustc_sysroot)/lib/rustlib/src/rust/library
```

## compiler_builtins for Kernel

Before we talk about compiler builtins, some basic knowledges of compiling Rust
into object file are requisite.

Rust is a front end of `llvm` and it depends on `llvm` for code generation. The
front end is responsible for lexical analysis and parsing. It generates IR
(Intermediate representation) and pass IR to `llvm` which converts IR to binary
files for target architecture. So, basically and currently, architectures
supported by Rust in kernel is largely constrained by the architectures
supported by `llvm`.

There is another term, `intrinsic function`, in `llvm` context. An intrinsic
function is a function built in to the compiler. The compiler knows how to
best implement the functionality in the most optimized way for these functions
and replaces with a set of machine instruction for a particular backend.
Simply, intrinsics are basic elementary functions for code generation in llvm
for a specific architecture.

`compiler-rt` is library that provides a set of intrinsic functions required
by llvm's code generation and other runtime components. Rust provides
`compiler_builtins` as a port of `compiler-rt`, and it provides necessary
intrinsics for llvm as well.

However, Rust's core library provides some floating-point functionalities which
are useless for Linux kernel. But, due to Rust in kernel utilises the whole
`libcore` and doesn't strip off these functions. Some floating-point relevant
intrinsics are still need for code generation, even they are meaningless.

That is why `compiler_builtins.rs` comes in. Rust in kernel uses this source
file as a replacement of `compiler_builtins` to Rust's standard library.
Useless intrinsics to kernel are defined int this file and implemented as
`panic` which means any invocation of such a intrinsics will cause a kernel
panic.

Obviously, this implementation is crude, but is the fastest way.

## How Rust uses our libcore instead of libstd.

It is all about `--extern` flag and symbols.

`--extern` flag of `rustc` command specifies the location of an exteranl
library. Indirect dependencies are located using the `-L` flag. `--extern`
flag just makes rust compiler to link libraries that complement the missing
symbols during linkage phase. During the runtime, symbol findings are
automatically done by kernel itself.

This flag also applies to buding out-tree modules. However, different from
building Rust support in kernel itself, building an out-tree module needs to
link `kernel` only. `kernel` in trun depends on `alloc` and `core` libraries
we have discuessed before.


