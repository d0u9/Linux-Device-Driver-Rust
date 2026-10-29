# How Rust is integrated into Kernel.

There is one question lingered in my head that how the [Kbuild] system builds
rust's core and other stuffs.

This post aims to solve this puzzle.

# Basics of Kernel's Kbuild system.

Before we dive into the chunks of Makefiles, it is worthy to learn some basics
of how kernel's Kbuild system works.

There is one valuable documentation worthy to read from Kernel's source tree:
[Linux Kernel Makefiles]. I highly suggest you to follow this documentation
first before we going into the hall of puzzle.

However, for readers who has no patience to read that, I will try my best to
depict every detail.

## What is Kbuild

Kbuild is the abbreviation of "the Linux Kernel Build System", which manages the
process of build kernel image and in-tree/out-tree modules.

Kbuild is a giant monster. Luckily, for most kernel hackers and driver developers,
it is unnecessary to understand Kbuild thoroughly but a small set of rules and
functions.

Kbuild is made upon makefiles. It adopts make's principles and extends its
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
Makefile                    the top Makefile.
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

For users whose Rust toolchain is missing or unsuitable, `make menuconfig`
menu has no option named `RUST` at all. This is due to that Kconfig first runs
`scripts/rust_is_available.sh` to check the toolchain, and then determines
whether to show this option or not. This logic can be found in `init/Kconfig`

```makefile
config RUST_IS_AVAILABLE
	def_bool $(success,$(srctree)/scripts/rust_is_available.sh)
```

And this:

```makefile
config RUST
	bool "Rust support"
	depends on HAVE_RUST
	depends on RUST_IS_AVAILABLE
	...
```

`HAVE_RUST` is selected by architectures supporting Rust, for example
`select HAVE_RUST if X86_64` in `arch/x86/Kconfig`.

### Target

When building starts, top Makefile recursively descends into subdirectories of
the source tree, and finds targets named as `obj-y`. For rust, in
`rust/Makefile`, it looks like (abridged):

```makefile
obj-$(CONFIG_RUST) += core.o compiler_builtins.o ffi.o
obj-$(CONFIG_RUST) += bindings.o pin_init.o kernel.o
obj-$(CONFIG_RUST) += exports.o
```

Note `CONFIG_RUST` variable here, it will be set according to `.config` file.
When Rust support is enabled, `CONFIG_RUST` variable has value of `y` and
results `obj-$(CONFIG_RUST)` to be `obj-y`. So, when Rust support is enable,
these targets are:

```makefile
obj-y += core.o compiler_builtins.o ffi.o
obj-y += bindings.o pin_init.o kernel.o
obj-y += exports.o
```

`obj-y` is a list of object files which will be finally linked into vmlinux
image.

### Actual building command

`core.o` is an object files and usually it is generated from `core.c` due to
make's implicit rule. However, for `core.o` it is built from Rust source file
instead of C files, so an explicit rule need to be specific.

```makefile
$(obj)/core.o: private skip_clippy = 1
...
$(obj)/core.o: $(RUST_LIB_SRC)/core/src/lib.rs \
    $(wildcard $(objtree)/include/config/RUSTC_VERSION_TEXT) FORCE
	+$(call if_changed_rule,rustc_library)
```

This rule is simple that says `$(obj)/core.o` depends on
`$(RUST_LIB_SRC)/core/src/lib.rs` and `FORCE` to update whenever this rule
is run.

`$(call if_changed_rule,rustc_library)` is a macro. `rustc_library` is run when
files and dependences are changed.

In the same Makefile, we can find:

```makefile
quiet_cmd_rustc_library = $(if $(skip_clippy),RUSTC,$(RUSTC_OR_CLIPPY_QUIET)) L $@
      cmd_rustc_library = \
	$(rustc_target_envs) \
	OBJTREE=$(abspath $(objtree)) \
	$(if $(skip_clippy),$(RUSTC),$(RUSTC_OR_CLIPPY)) \
		$(filter-out $(skip_flags),$(rust_flags)) $(rustc_target_flags) \
		--emit=dep-info=$(depfile) --emit=$(if $(link_helper),llvm-bc=$(patsubst %.o,%.bc,$@),obj=$@) \
		--emit=metadata=$(dir $@)$(patsubst %.o,lib%.rmeta,$(notdir $@)) \
		--crate-type rlib -L$(objtree)/$(obj) \
		--crate-name $(patsubst %.o,%,$(notdir $@)) $< \
		--sysroot=/dev/null \
	...
```

The `quiet_cmd_<command>` and `cmd_<command>` are special variables which is
consumed by Kbuild's "command change detection". Simply, in kernel's Makefile,
we have the following form:

```makefile
quiet_cmd_<command> = ...
      cmd_<command> = ...

<target>: <source(s)> FORCE
        $(call if_changed,<command>)
```

The `if_changed` macro has several variants, and one of them is `if_changed_rule`
what we have met before. All of these macros are defined in
`scripts/Kbuild.include`.

Conclude from above, we can deduce that to build `core.o` the command string
in variable `cmd_rustc_library` should be evaluated.

### Always built targets

Wait, `bindings.o`, on which `kernel.o` depends, needs a file named
`bindings_generated.rs`, and where is it?

The `bindings_generated.rs` is a file which is automatically generated by
`bindgen`.

`bindgen` is tool for automatically generating Rust FFI(Foreign Function
Interface) bindings to C and C++ libraries.

In Kbuild, `always-y` is used for such purpose. `always-y` is a twin sister of
`obj-y`. Targets in `always-y` are always built when Kbuild visits the
Makefile, and are not finally linked into vmlinux.

```makefile
always-$(CONFIG_RUST) += bindings/bindings_generated.rs bindings/bindings_helpers_generated.rs
```

The rule generating it is:

```makefile
$(obj)/bindings/bindings_generated.rs: $(src)/bindings/bindings_helper.h \
    $(src)/bindgen_parameters FORCE
	$(call if_changed_dep,bindgen)
```

`always-y` itself does not order these targets before `obj-y`.
`bindings_generated.rs` is built before `bindings.o` because it is listed as a
prerequisite of `bindings.o`:

```makefile
$(obj)/bindings.o: $(src)/bindings/lib.rs \
    $(obj)/ffi.o \
    $(obj)/pin_init.o \
    $(obj)/bindings/bindings_generated.rs \
    ...
```

Other than the list name and building sequence, the remaining procedures are
identical.

## Evolution

- Kconfig used to decide whether Rust is available with `HAS_RUST`, which
  only checked that `$(RUSTC) --version` succeeds and listed the supported
  architectures directly. Now each architecture selects `HAVE_RUST`, and
  `RUST_IS_AVAILABLE` runs `scripts/rust_is_available.sh` to check the whole
  toolchain.
- `alloc.o` used to be in the `obj-y` list. It is gone together with the
  `alloc` crate.
- Generated files such as `bindings_generated.rs` used to be listed in
  `extra-$(CONFIG_RUST)`. They are in `always-$(CONFIG_RUST)` now.

[Kbuild]: https://www.kernel.org/doc/html/latest/kbuild/kbuild.html
[Linux Kernel Makefiles]: https://www.kernel.org/doc/html/latest/kbuild/makefiles.html
