```
make KERNELDIR=/path/to/kernel/source modules
```

This example used to have a string parameter, `whom`, to choose whom to greet.
Rust modules in mainline Linux only support integer parameters (`i8` to `u64`,
`isize` and `usize`), so it has been removed.
