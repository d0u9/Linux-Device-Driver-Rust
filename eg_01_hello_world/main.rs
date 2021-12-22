#![no_std]
#![feature(allocator_api, global_asm)]
#![allow(missing_docs)]

use kernel::prelude::*;

module! {
    type: HelloWorld,
    name: b"hello_world",
    author: b"Douglas Su",
    description: b"A simple hello world example",
    license: b"GPL v2",
}

struct HelloWorld;

impl KernelModule for HelloWorld {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello world from rust!\n");

        Ok(HelloWorld)
    }
}

impl Drop for HelloWorld {
    fn drop(&mut self) {
        pr_info!("Bye world from rust!\n");
    }
}
