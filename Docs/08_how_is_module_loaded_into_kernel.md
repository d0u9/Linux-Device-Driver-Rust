# How is module loaded into kernel

## Procedures to load and execute a .ko file 

Module file for Linux kernel is an `ELF` file. `ELF` is an abbreviation of
Executable Linkable Format. It is the standard format which is used by
executable binary file and dynamic library file as well as kernel module file.
Actually, loading an kernel module file into kernel almost the same as load an
dynamic library file in user space.

The whole process can be divided into several steps:

1. Copy file from user space to kernel space;
2. Verification.
3. Parsing file;
4. Relocating symbols;
5. Set module parameters;
6. Execution.

## Copy file from user space to kernel space

Loading from user space is implemented via a specific system call [init_module].
This system call takes three paramters: 1. The buffer in which ko file fills; 2.
The size of buffer; 3, a parameter string.

This system call is defined in file [kernel/module.c], and it is defined as:

```c
SYSCALL_DEFINE3(init_module, void __user *, umod,
		unsigned long, len, const char __user *, uargs)
```

This function does nothing but checks invoker's capability and copies content
from user space to kernel space. User who invokes this system call must have the
capability of `CAP_SYS_MODULE`.

Then, calls `load_module()` function which does the heavy duties.

## Module verification

`load_module()` function is defined in [kernel/module.c] as well. It is a long
function that does the actual work to load and run a kernel module. At the
very beginning of this function, some verifications are taken to make sure that
the module will be loaded is valid and matches our kernel version.

```c
/*
	 * Do the signature check (if any) first. All that
	 * the signature check needs is info->len, it does
	 * not need any of the section info. That can be
	 * set up later. This will minimize the chances
	 * of a corrupt module causing problems before
	 * we even get to the signature check.
	 *
	 * The check will also adjust info->len by stripping
	 * off the sig length at the end of the module, making
	 * checks against info->len more correct.
	 */
	err = module_sig_check(info, flags);
	if (err)
		goto free_copy;

	/*
	 * Do basic sanity checks against the ELF header and
	 * sections.
	 */
	err = elf_validity_check(info);
	if (err) {
		pr_err("Module has invalid ELF structures\n");
		goto free_copy;
	}
```

## Parsing module file

The parsing procedure breaks module elf file into data structures. For module
elf file, there are four sections worthy to mention.

1. `.modinfo` section;
2. `__param` section;
3. `.rela__param` section;
4. `.symtab` section.

The `.modinfo` sections contains basic information about this module. For
example, module description, module parameters, module author, etc.

The `__param` section contains module parameter structures which will be
initialized after relocating.


[kernel/module.c]: https://github.com/torvalds/linux/blob/master/kernel/module.c
[init_module]: https://man7.org/linux/man-pages/man2/init_module.2.html]
