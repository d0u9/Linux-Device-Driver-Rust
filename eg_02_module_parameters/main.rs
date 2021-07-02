#![no_std]
#![feature(allocator_api, global_asm)]

use kernel::prelude::*;

module! {
    type: ModuleParameter,
    name: b"hello_world",
    author: b"d0u9",
    description: b"A simple hello world example",
    license: b"GPL v2",
    params: {
        howmany: i32 {
            default: 3,
            permissions: 0o644,
            description: b"How many times string will be printed",
        },
        whom: str {
            default: b"Mom",
            permissions: 0o644,
            description: b"What string to be printed",
        },
    },
}

struct ModuleParameter;

impl KernelModule for ModuleParameter {
    fn init() -> Result<Self> {
        pr_info!("Hello world from rust!\n");

        let lock = THIS_MODULE.kernel_param_lock();
        for i in 0..*howmany.read(&lock) {
            pr_info!("{} Hello, {}\n", i, core::str::from_utf8(whom.read(&lock))?);
        }

        Ok(ModuleParameter)
    }
}

impl Drop for ModuleParameter {
    fn drop(&mut self) {
        pr_info!("Bye world from rust!\n");
    }
}
