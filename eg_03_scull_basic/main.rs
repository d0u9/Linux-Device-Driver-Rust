#![no_std]
#![feature(allocator_api, global_asm)]

use kernel::prelude::*;

module! {
    type: ScullBasic,
    name: b"scull_basic",
    author: b"d0u9",
    description: b"A simple memory based storage device",
    license: b"GPL v2",
}

struct ScullBasic;

impl KernelModule for ScullBasic {
    fn init() -> Result<Self> {
        pr_info!("Hello world from rust!\n");

        Ok(ScullBasic)
    }
}

impl Drop for ScullBasic {
    fn drop(&mut self) {
        pr_info!("Bye world from rust!\n");
    }
}
