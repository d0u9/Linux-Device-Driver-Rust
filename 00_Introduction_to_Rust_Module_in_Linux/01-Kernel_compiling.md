# Compile Rust Compatible Kernel
## Prerequisites

The commands below are run in the top directory of the Linux kernel source
tree, not in this repository. Paths such as `scripts/min-tool-version.sh` are
relative to the kernel source tree:

```
cd /path/to/kernel/source/
```

### Install Rust

A stable Rust toolchain is enough; nightly is not required. The kernel needs
a `rustc` no older than the version printed by:

```
scripts/min-tool-version.sh rustc
```

Install the stable toolchain with `rustup`:

```
rustup default stable
```

Out-of-tree Rust modules must be built with exactly the same `rustc` as the
kernel they are built against. After `rustup update` brings a new stable
`rustc`, rebuild the kernel before building modules, or pin the version with a
`rust-toolchain.toml` file, e.g.:

```toml
[toolchain]
channel = "1.99.1"
components = ["rust-src"]
```

### Install standard library source

The Rust standard library source is required because the build system will
cross-compile `core`.

```
rustup component add rust-src
```

### Install bindgen

`bindgen` is used to generate Rust bindings to the C side of the kernel. Its
minimum version is printed by `scripts/min-tool-version.sh bindgen`.

```
cargo install --locked bindgen-cli
```

### Install libclang

`libclang` (part of LLVM) is used by `bindgen` to understand the C code in the
kernel. It is required even if the kernel itself is compiled with GCC.

To get the latest LLVM, download the prebuilt toolchain from
[kernel.org](https://kernel.org/pub/tools/llvm/), which contains `clang`,
`lld` and the other LLVM tools, and extract it (not tested yet; the name of the
extracted directory and whether `bindgen` finds its `libclang` by itself are
not verified):

```
curl -LO https://kernel.org/pub/tools/llvm/files/llvm-23.1.4-x86_64.tar.xz
tar -xf llvm-23.1.4-x86_64.tar.xz -C /path/to/toolchains/
export PATH=/path/to/toolchains/llvm-23.1.4-x86_64/bin:$PATH
```

Alternatively, install the LLVM packaged by the distribution, which may be a
few versions behind. On Fedora:

```
sudo dnf install clang llvm lld
```

If `libclang` is not installed in a standard location, point `bindgen` to it
with the `LIBCLANG_PATH` environment variable.

### Check the toolchain

```
make rustavailable
```

This runs the same check Kconfig uses to decide whether Rust support can be
enabled, and explains what is missing if it cannot.

## Building Kernel

### Configure kernel

If the kernel is already configured following
[Linux-Device-Driver](https://github.com/d0u9/Linux-Device-Driver), only Rust
support needs to be turned on:

```bash
scripts/config --enable RUST
make olddefconfig
```

Otherwise, start from scratch as below.

Start from the default configuration, then adjust it with `scripts/config`.

```bash
make defconfig

scripts/config \
    --disable SOUND \
    --disable WLAN \
    --disable USB_NET_DRIVERS \
    --disable WIRELESS \
    --disable IPV6 \
    --disable IA32_EMULATION \
    --disable E100 \
    --enable E1000 \
    --disable E1000E \
    --enable RUST \
    --enable VIRTIO_PCI \
    --enable NET_9P \
    --enable NET_9P_VIRTIO \
    --enable 9P_FS

make olddefconfig
```

`make olddefconfig` resolves Kconfig dependencies after the edits, and
silently turns off options whose dependencies are not met. Check that Rust
support is still enabled:

```bash
grep CONFIG_RUST= .config
```

If it does not print `CONFIG_RUST=y`, run `make rustavailable` to see
why. `RUST` also depends on other options; for example, it is turned off while
`MODVERSIONS` is enabled without `GENDWARFKSYMS`, or while `RANDSTRUCT` is
enabled.

The option is in `General setup` in `make menuconfig`, which can be used
to review the configuration:

```
General setup  --->
  [*] Rust support
```

### Building kernel

Build the kernel as usual:

```
make -j"$(nproc)" bzImage
make -j"$(nproc)" modules
```

### GCC or LLVM

When Rust support was first developed, the kernel had to be built with Clang
or a complete LLVM toolchain (`LLVM=1`), because `rustc` generates code with
LLVM, and `bindgen` parses the kernel's C headers with `libclang`. The C flags
of a GCC build had to be translated for `libclang`, and the structure layouts
and calling conventions seen by Rust were not guaranteed to match what GCC
produced.

Since then, Kbuild has learned to translate and filter the GCC flags for
`bindgen`, and a kernel built with GCC can enable Rust support. `libclang` is
still required by `bindgen`.

LLVM is now needed only for options that support Clang alone, such as `KASAN`
together with Rust. To build with LLVM, pass `LLVM=1` to every `make`
invocation on the tree, including `defconfig`, `rustavailable` and module
builds.

## Evolution

- Rust support used to require a nightly toolchain, pinned to the exact
  version printed by `scripts/min-tool-version.sh rustc` with
  `rustup override set`. Now any stable `rustc` no older than that version
  works; Kbuild enables the unstable features the kernel needs by itself.
- The command line tool of `bindgen` used to be installed with
  `cargo install bindgen`. It has moved to the `bindgen-cli` crate.
