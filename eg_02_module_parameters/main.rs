#![allow(missing_docs)]

use kernel::prelude::*;

module! {
    type: ModuleParameter,
    name: "module_parameters",
    authors: ["d0u9"],
    description: "Module parameters example",
    license: "GPL v2",
    params: {
        howmany: i32 {
            default: 3,
            description: "How many times string will be printed",
        },
    },
}

struct ModuleParameter;

impl kernel::Module for ModuleParameter {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("Hello world from rust!\n");

        for i in 0..*module_parameters::howmany.value() {
            pr_info!("{} Hello, Mom\n", i);
        }

        Ok(ModuleParameter)
    }
}

impl Drop for ModuleParameter {
    fn drop(&mut self) {
        pr_info!("Bye world from rust!\n");
    }
}
