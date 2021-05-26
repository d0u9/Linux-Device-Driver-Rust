# How Rust is integrated into Kernel.

There is one question lingered in my head that how the [Kbuild] system builds
rust's core and other stuffs.

This post aims to solve this puzzle.

# Basics of Kernel's Kbuild system.

Before we dive into the chunks of Makefils, it is worthy to learn some basics
of how kernel's Kbuild system works.

There is one valuable documentation worthy to read from Kernel's source tree:
[Linux Kernel Makefiles]. I highly suggest you to follow this documentation
first before we going into the hall of puzzle.

However, for readers who has no patience to read that, I will try my best to
depict every detail.

## What is Kbuild

Kbuild is the abbreviation of "the Linux Kernel Build System", which manages the
process of build kernel image and in-tree/out-tree modules.

Kbuild is a giant monster. Luckily, for most kernel hackers and driver deveopers,
it is unnecessary to understand Kbuild thoroughly but a small set of rules and
functions.

Kbuild is made upon makefiles. It adopts make's principals and extents its
abilities to a intact "system". For Linux Kernel, most codes are written in C
and assembly. But there still are complex rules to compile and link these pieces
together. What makes things worse is that other than C and assembly, some
foreign programming languages exists, such as Python, Perl and Rust.

Kbuild gathers information from every corner for source tree, and then makes
decision on that to build kernel image.

## Prerequisites of Kbuild's makefile

### Config

Kbuild's Makefiles have five parts:

```
makefile                    the top Makefile.
.config                     the kernel configuration file.
arch/$(SRCARCH)/Makefile    the arch Makefile.
scripts/Makefile.*          common rules etc. for all kbuild Makefiles.
kbuild Makefiles            exist in every subdirectory
```

The `.config` file contains configuration switches which tell Kbuild what to
build and what to be built as modules. Kernel users usually generate `.config`
file by `make menuconfig` command which pops a interactive menu for making
choice. As you wish, switch to enable or disable Rust support in kernel is
contained in this file as well.

```makefile
CONFIG_RUST=y
```

For users who has not installed `rustc` yet, `make menuconfig` menu has no
option named `RUST` at all. This is due to that `make menuconfig` will first
check the existence of `rustc` compile and then determine whether to show this
option or not. This logical can be found in `init/Kconfig`

```makefile
config HAS_RUST
    depends on ARM64 || CPU_32v6 || CPU_32v6K || (PPC64 && CPU_LITTLE_ENDIAN) || X86_64 || RISCV
    def_bool $(success,$(RUSTC) --version)
```

And this:

```makefile
config RUST
    bool "Rust support"
    depends on HAS_RUST
    depends on !COMPILE_TEST
    default n
    help
      Enables Rust support in the kernel.
```

### Target

When building starts, top Makefile recursively descends into subdirectories of
the source tree, and finds targets named as `obj-y`. For rust, it looks like:

```makefile
obj-$(CONFIG_RUST) += core.o compiler_builtins.o helpers.o
obj-$(CONFIG_RUST) += alloc.o kernel.o
obj-$(CONFIG_RUST) += exports.o
```

Note `CONFIG_RUST` variable here, it will be set according to `.config` file.
When Rust support is enabled, `CONFIG_RUST` variable has value of `y` and
results `obj-$(CONFIG_RUST)` to be `obj-y`. So, when Rust support is enable,
these targets are:

```makefile
obj-y += core.o compiler_builtins.o helpers.o
obj-y += alloc.o kernel.o
obj-y += exports.o
```

`obj-y` is a list of object files which will be finally linked into vmlinux
image.

### Actual building command

`core.o` is an object files and usually it is generated from `core.c` due to
make's implicit rule. However, for `core.o` it is built from Rust source file
instead of C files, so an explicit rule need to be specific.

```makefile
.SECONDEXPANSION:
$(objtree)/rust/core.o: private skip_clippy = 1
$(objtree)/rust/core.o: $$(RUST_LIB_SRC)/core/src/lib.rs FORCE
	$(call if_changed_dep,rustc_library)
```

This rule is simple that says `$(objtree)/rust/core.o` depends on
`$$(RUST_LIB_SRC)/core/src/lib.rs` and `FORCE` to update whenever this rule
is run.

`$(call if_changed_dep,rustc_library)` is a macro. `rustc_library` is run when
files and dependences are changed.

In the same Makefile, we can find:

```makefile
quiet_cmd_rustc_library = $(if $(skip_clippy),RUSTC,$(RUSTC_OR_CLIPPY_QUIET)) L $@
      cmd_rustc_library = \
	RUST_BINDINGS_FILE=$(abspath $(objtree)/rust/bindings_generated.rs) \
	$(if $(skip_clippy),$(RUSTC),$(RUSTC_OR_CLIPPY)) \
		$(rustc_flags) $(rustc_cross_flags) $(rustc_target_flags) \
		--crate-type rlib --out-dir $(objtree)/rust/ -L $(objtree)/rust/ \
		--crate-name $(patsubst %.o,%,$(notdir $@)) $<; \
	mv $(objtree)/rust/$(patsubst %.o,%,$(notdir $@)).d $(depfile); \
	sed -i '/^\#/d' $(depfile) \
	$(if $(rustc_objcopy),;$(OBJCOPY) $(rustc_objcopy) $@)
```

The `quiet_cmd_<command>` and `cmd_<command>` are special variables which is
consumed by Kbuild's "command change detection". Simply, in kernel's Makfile,
we have the following form:

```makefile
quiet_cmd_<command> = ...
      cmd_<command> = ...

<target>: <source(s)> FORCE
        $(call if_changed,<command>)
```

The `if_changed` macro has several variants, and one of them is `if_changed_dep`
what we have met before. All of these macros are defined in
`scripts/Kbuild.include`.

Conclude from above, we can deduce that to build `core.o` the command string
in variable `cmd_rustc_library` should be evaluated.

### Non-builtin targets

Wait, you say that a file named `bindings_generated.rs` is need and where is it?

The `bindings_generated.rs` is a file which is automatically generated by
`bindgen`.

`bindgen` is tool for automatically generating Rust FFI(Foreign Function
Interface) bindings to C and C++ libraries. For our examples here,
`bindings_generated.rs` is generated before any `obj-y` object is built.

In Kbuild, `extra-y` is used for such purpose. `extra-y` is a twin sister of
`obj-y`. Targets in `extra-y` are built prior to targets in `obj-y`, and are not
finally linked into vmlinux.

```makefile
$(objtree)/rust/bindings_generated.rs: $(srctree)/rust/kernel/bindings_helper.h FORCE
	$(call if_changed_dep,bindgen)
```

Other than the list name and building sequence, the remaining procedures are
identical.


[Kbuild]: https://www.kernel.org/doc/html/latest/kbuild/kbuild.html
[Linux Kernel Makefiles]: https://www.kernel.org/doc/html/latest/kbuild/makefiles.html
