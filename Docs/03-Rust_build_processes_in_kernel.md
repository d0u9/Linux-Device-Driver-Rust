# Rust Build Processes in Kernel

This documentation try to steps over phases of building Rust support in Kernel
and linking rust files into Kernel image.

## Dependency Graph

![Dependency Graph](./dependency_graph.png)


## FFI (Foreign Function Interface)

FFI bridges Rust and C (Some C++) libraries. It provides a neat interface for
rust users to invoke c functions in a library.

## Bindgen

However, directly write FFI interface is time consuming and not long lasting. It
is better to use tools to automatically generate these bindings. For this
purpose, [bindgen] comes in.

bindgen automatically generates FFI bindings to C libraries, and give Ruster the
abilities to invoke C functions directly in Rust code. It is super convenient
and usable. For readers who have not been familiar with [bindgen], it is good
to take a quick survey.

One thing need to be noted in precedence is that [bindgen] can be both used as a
library and command line tool. For Rust in Kernel, command line tool is used.

Basically, [bindgen] takes a series of C header files in which signatures of
functions are given, then [bindgen] analysis these function signatures and
generates wrapper Rust functions on that.

[bindgen] relays on [libclang] for parsing C codes. That is why we have to
install [libclang] before compiling Rust subsystem in Kernel.

Makefile rule for generating Rust binding files is listed as below:

```
quiet_cmd_bindgen = BINDGEN $@
      cmd_bindgen = \
	$(BINDGEN) $< $(addprefix --opaque-type , $(bindgen_opaque_types)) \
		--use-core --with-derive-default --ctypes-prefix c_types \
		--size_t-is-usize -o $@ -- $(bindgen_c_flags_final) -DMODULE

$(objtree)/rust/bindings_generated.rs: $(srctree)/rust/kernel/bindings_helper.h FORCE
	$(call if_changed_dep,bindgen)
```

Options after two dashed `--` will be passed to clang for header searching and
etc.

## Objects generated from C

Some object files are generated directly from C source files in Rust directory.
These files are generated implicitly by Makefile's implicit rule in that if
`foo.o` file cannot be found in any target, make will find and compile source
file named `foo.c` by default.

In Rust in Kernel, `exports.o` and `helpers.o` are such type of generation.
They are compiled from `exports.c` and `helpers.c` respectively via make's
implicit rule.



[bindgen]: https://github.com/rust-lang/rust-bindgen
[libclang]: https://clang.llvm.org/docs/Tooling.html#libclang

