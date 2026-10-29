# Linux-Device-Driver-Rust

This is a twain Repo of [Linux-Device-Driver].

[Linux-Device-Driver] is a long time work in which I rewrite all examples in
the book LDD3 (Linux Device Driver, the 3rd edition).

# Before start

Current test baseline:

- Kernel: Linux 7.2.11, built with GCC and `CONFIG_RUST=y`.
- GCC: 16.2.1.
- Rust: rustc 1.99.1 (stable, installed with `rustup`).
- bindgen: 0.73.2.
- libclang: 23.1.4.
- QEMU: 11.1.2, with the devices from
  [Linux-Device-Driver](https://github.com/d0u9/Linux-Device-Driver).
- Host system: Fedora release 44 (Forty Four).

Other versions are not tested. Out-of-tree Rust modules must be built with
exactly the same `rustc` as the kernel they are built against.

The examples target 64-bit kernels (x86_64).

[Linux-Device-Driver]: https://github.com/d0u9/Linux-Device-Driver
