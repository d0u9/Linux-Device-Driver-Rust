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

The `.rela_param` section contains relocation information for kernel parameters.

The `.symtab` section contains symbol table with information about functions and
global variables that are defined and referenced in the program.

To see what sections are contained in an ELF file, use `readelf` command:

```
readelf -S test_module.ko
```

The function `setup_load_info()` defined in [kernel/module.c] reads module file
and fill some fields of `struct load_info *info` according to module file. The
field `hdr` is special for that this field points to the buffer in which module
file residents. This field is assigned in `copy_module_from_user()` function:

```c
info->hdr = __vmalloc(info->len, GFP_KERNEL | __GFP_NOWARN);
if (!info->hdr)
    return -ENOMEM;

if (copy_chunked_from_user(info->hdr, umod, info->len) != 0) {
    err = -EFAULT;
    goto out;
}
```

For X86_64, `info->hdr` has the type of `Elf64_Ehdr`, defined in
[include/uapi/linux/elf.h]. For folks who familiar with ELF file format, it is
easy to find that `Elf64_Ehdr` structure actually describes the [ELF] header.

The parsing process is tedious and rigmarole, and most operations are not
relevant to our topic. But for a better understand of reading module info and
setting up module parameter, I will detail some interested procedures.

### Get module info

Module informations include module name, license, author, parameter type,
parameter description, etc. These informations are packed in `.modinfo` section
in ELF file(module file).

To read `.modinfo` section, first we need to find the index of section. Use
function `find_sec()` to get the index of a specific section with section name:

```
info->index.info = find_sec(info, ".modinfo");
```

Then obtain a specific field in `.modinfo` section by invoking `get_modinfo()`:

```
info->name = get_modinfo(info, "name");
```

For users who curious about the content of `.modinfo` section, use `readelf`
command to inspect:

```
readelf -x .modinfo test_module.ko

Hex dump of section '.modinfo':
  0x00000000 7061726d 74797065 3d75696e 745f7061 parmtype=uint_pa
  0x00000010 72616d3a 75696e74 006c6963 656e7365 ram:uint.license
  0x00000020 3d47504c 00617574 686f723d 446f7567 =GPL.author=Doug
  0x00000030 6c617320 53750064 65736372 69707469 las Su.descripti
  0x00000040 6f6e3d41 6e206578 616d706c 65206d6f on=An example mo
  0x00000050 64756c65 00766572 6d616769 633d352e dule.vermagic=5.
  0x00000060 31322e30 2d726334 2b20534d 50206d6f 12.0-rc4+ SMP mo
  0x00000070 645f756e 6c6f6164 20006e61 6d653d74 d_unload .name=t
  0x00000080 6573745f 6d6f6475 6c650072 6574706f est_module.retpo
  0x00000090 6c696e65 3d590064 6570656e 64733d00 line=Y.depends=.
```

Also, use `-p` option instead of `-x` option to get result in string.

```
readelf -p .modinfo test_module.ko

String dump of section '.modinfo':
  [     0]  parmtype=uint_param:uint
  [    19]  license=GPL
  [    25]  author=Douglas Su
  [    37]  description=An example module
  [    55]  vermagic=5.12.0-rc4+ SMP mod_unload
  [    7a]  name=test_module
  [    8b]  retpoline=Y
  [    97]  depends=
```

`get_modinfo()` function parses these `\0` separated strings, and get target
value string by key.

### Get `__param` section

`struct module` is the data structure that describes the module currently
processing. some fields are points to elf sections. For example, `mod->kp` filed
points to the `__param` section in ELF file.

`__param` section in ELF is an empty table which will be filled during
relocating. So, Reading to this sections from ELF file by `readelf` returns
almost nothing:

```
readelf -x __param test_module.ko

Hex dump of section '__param':
 NOTE: This section has relocations against it, but these have NOT been applied to this dump.
  0x00000000 00000000 00000000 00000000 00000000 ................
  0x00000010 00000000 00000000 2401ff00 00000000 ........$.......
  0x00000020 00000000 00000000                   ........

```

## Parameter parsing and rewriting.

### Define module parameter

To define an `u32` type module parameter:

```c
static u32 uint_param = 1;
module_param(uint_param, uint, S_IRUGO);
```

The first line syas we defined a global variable of type `u32` and initialized
its value to `1`.

The second line is a macro to declare a module paramter. The macro defined as 
below recursively.

```c
// file: include/linux/moduleparam.h

#define module_param(name, type, perm)                          \
        module_param_named(name, name, type, perm)

#define module_param_named(name, value, type, perm)                        \
        param_check_##type(name, &(value));                                \
        module_param_cb(name, &param_ops_##type, &value, perm);            \
        __MODULE_PARM_TYPE(name, #type)

#define param_check_uint(name, p) __param_check(name, p, unsigned int)

#define __param_check(name, p, type) \
        static inline type __always_unused *__check_##name(void) { return(p); }

#define module_param_cb(name, ops, arg, perm)                                 \
        __module_param_call(MODULE_PARAM_PREFIX, name, ops, arg, perm, -1, 0)

#define __module_param_call(prefix, name, ops, arg, perm, level, flags) \
        static const char __param_str_##name[] = prefix #name;          \
        static struct kernel_param __moduleparam_const __param_##name   \
        __used __section("__param")                                     \
        __aligned(__alignof__(struct kernel_param))                     \
        = { __param_str_##name, THIS_MODULE, ops,                       \
                VERIFY_OCTAL_PERMISSIONS(perm), level, flags, { arg } }

#define __MODULE_INFO(tag, name, info)                                    \
        static const char __UNIQUE_ID(name)[]                             \
        __used __section(".modinfo") __aligned(1)                         \
        = __MODULE_INFO_PREFIX __stringify(tag) "=" info

#define __MODULE_PARM_TYPE(name, _type)                                   \
        __MODULE_INFO(parmtype, name##type, #name ":" _type)
```


For our example, `module_param(uint_param, uint, S_IRUGO)`, it is finally
expanded as (I have removed some GCC attributes which are unnecessary for us
to understand the mechanisim):

```c
static const char __param_str_uint_param[] = "uint_param";

static struct kernel_param const __param_uint_param __attribute__((__section__("__param"))) = { 
	__param_str_uint_param,
	(&__this_module),
	&param_ops_uint, 
	S_IRUGO,
	-1,
	0,
	{ &uint_param }
};

static const char __UNIQUE_ID_uint_paramtye[] __attribute__((__section__(".modinfo"))) = "parmtype" "=" "uint_param" ":" "uint";
```

This macro is expanded into three variable definitions.

The first variable, `__param_str_uint_param` is an array of chars in which our
parameter name is written in. For C language, this char array is placed in
`rodata` section of ELF file.

```
readelf -s test_module.ko

Symbol table '.symtab' contains 34 entries:
   Num:    Value          Size Type    Bind   Vis      Ndx Name
   3: 0000000000000000    11 OBJECT  LOCAL  DEFAULT    7 __param_str_uint_param
   4: 0000000000000000    40 OBJECT  LOCAL  DEFAULT    8 __param_uint_param
   5: 0000000000000000    25 OBJECT  LOCAL  DEFAULT   10 __UNIQUE_ID_uint_paramtyp
```

```
readelf -S test_module.ko
There are 24 section headers, starting at offset 0xc38:

Section Headers:
  [Nr] Name              Type             Address           Offset
       Size              EntSize          Flags  Link  Info  Align
  [ 7] .rodata           PROGBITS         0000000000000000  0000005c
       000000000000000b  0000000000000000   A       0     0     1
  [ 8] __param           PROGBITS         0000000000000000  00000068
       0000000000000028  0000000000000000   A       0     0     8
  [ 9] .rela__param      RELA             0000000000000000  00000530
       0000000000000060  0000000000000018   I      21     8     8
  [10] .modinfo          PROGBITS         0000000000000000  00000090
       00000000000000a0  0000000000000000   A       0     0     1
```

As you can see from the dump information above, symbol `__param_str_uint_param`
points to section 7 which is `.rodata` section.

The second variable, `__param_uint_param`, has a type of `kernel_param` which
is defined as:

```c
struct kernel_param {
    const char *name;
    struct module *mod;
    const struct kernel_param_ops *ops;
    const u16 perm;
    s8 level;
    u8 flags;
    union {
            void *arg;
            const struct kparam_string *str;
            const struct kparam_array *arr;
    };
};
```

The `name` field is a pointer to `const char` which is initialized to
`__param_str_uint_param`; The `perm` field is assigned as `S_IRUGO`; The `arg`
pointer in union is initialized to the address of our `uint_param` global
variable; The `ops` filed has type of `struct kernel_param_ops` in which three
pointers to functions are included.

```c
struct kernel_param_ops {
        unsigned int flags;
        int (*set)(const char *val, const struct kernel_param *kp);
        int (*get)(char *buffer, const struct kernel_param *kp);
        void (*free)(void *arg);
};
```

The `set` and `get` functions are used to set and get our variable values.
For our module parameter, the `ops` filed points to a pre-defined function
in kernel, `param_ops_uint`.

```c
// file: kernel/params.c
#define STANDARD_PARAM_DEF(name, type, format, strtolfn)                \
        int param_set_##name(const char *val, const struct kernel_param *kp) \
        {                                                               \
                return strtolfn(val, 0, (type *)kp->arg);               \
        }                                                               \
        int param_get_##name(char *buffer, const struct kernel_param *kp) \
        {                                                               \
                return scnprintf(buffer, PAGE_SIZE, format "\n",        \
                         *((type *)kp->arg));                           \
        }                                                               \
        const struct kernel_param_ops param_ops_##name = {              \
                .set = param_set_##name,                                \
                .get = param_get_##name,                                \
        };                                                              \
        EXPORT_SYMBOL(param_set_##name);                                \
        EXPORT_SYMBOL(param_get_##name);                                \
        EXPORT_SYMBOL(param_ops_##name)

STANDARD_PARAM_DEF(uint,	unsigned int,		"%u",		kstrtouint);
```

Expanded to:

```c
int param_set_uint(const char *val, const struct kernel_param *kp)
{
        return strtolfn(val, 0, (type *)kp->arg);
}

int param_get_uint(char *buffer, const struct kernel_param *kp)
{
        return scnprintf(buffer, PAGE_SIZE, format "\n",
                 *((type *)kp->arg));
 }
const struct kernel_param_ops param_ops_uint = {
        .set = param_set_uint,
        .get = param_get_uint,
};
EXPORT_SYMBOL(param_set_uint);
EXPORT_SYMBOL(param_get_uint);
EXPORT_SYMBOL(param_ops_uint);
```

Also, the `__param_uint_param` variable is decorated with
`__attribute__((__section__("__param")))` attribute which tells GCC or clang to
put this variable in `__param` section. However, due to the relocating, the
final `__param` section in ELF file doesn't contain these structures. They
are placed in `.rela__param` section, and are copied to `__param` section during
relocating process.

For people who is not familiar with relocating, I suggest to read the chapter 7
of book "Computer Systems: A programmer's Perspective".

The third variable `__UNIQUE_ID_uint_paramtye` is also a array of chars in
which a special string "parmtype=uint_param:uint" is written in. This variable
is linked into `modiinfo` section and be used by tools such as `modinfo` to
fetch information about this module.

### Apply relocation

`apply_relocate()` function is responsible for applying relocation to our
kernel module. We primarily focus on relocation of `__param` section in ELF
file. This sections contains data structures which describes the parameters
a kernel have.

Hex data dumped below are what content in `__param` section before and after
relocation is taken in action. These data is directly printed in kernel log
by hacking kernel.

```
before: (____ptrval____): 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  ................
before: (____ptrval____): 00 00 00 00 00 00 00 00 24 01 ff 00 00 00 00 00  ........$.......
before: (____ptrval____): 00 00 00 00 00 00 00 00                          ........
```

```
after: (____ptrval____): 00 50 33 c0 ff ff ff ff 40 60 33 c0 ff ff ff ff  .P3.....@`3.....
after: (____ptrval____): 60 c2 80 85 ff ff ff ff 24 01 ff 00 00 00 00 00  `.......$.......
after: (____ptrval____): 00 60 33 c0 ff ff ff ff                          .`3.....
```

For our example, `__param` section only contains single `struct kernel_param`
data structure describes `static u32 uint_param` parameter. The last field of
`struct kernel_param` an union type. Our `static u32 uint_param` parameter
fits in the void `*arg` type option.
```c
union {
        void *arg;
        const struct kparam_string *str;
        const struct kparam_array *arr;
};
```

It has the value `0xffffffc0336000`, i.e. the address of kernel parameter. This
is verified by looking the kernel log that we print this address by `pr_info()`
function in our code.

```
Hello World! uint_param=1
Address of module parameter uint_param: ffffffffc0336000
```

Also, it is not hard to find the address of `param_set_uint()` by decoding this
binary according to `struct kernel_param_ops`. The address `0xffffffff8200c260`
points the `struct kernel_param_ops` structure. Grep this address in kernel's
symbol table file (System.map file in the root of your kernel source, it will
be generated after successfully compiling kernel module).

```
grep param_ops_uint System.map

ffffffff8200c260 D param_ops_uint
```

Note: You may find that the address printed in kernel log and obtained from
System.map file are mismatched. This is due to `KASLR` (Kernel address space
layout randomization), and you can disable `KASLR` by appending `nokaslr`
kernel parameter. For QEMU users, add this in your `-append` option.

### Parsing parameters passed in via `init_module()` system call

First copy parameter string from user space to kernel:

```c
mod->args = strndup_user(uargs, ~0UL >> 1);
if (IS_ERR(mod->args)) {
    err = PTR_ERR(mod->args);
    goto free_arch_cleanup;
}
```

Then parsing it in function `parse_args()`:

```c
after_dashes = parse_args(mod->name, mod->args, mod->kp, mod->num_kp,
              -32768, 32767, mod,
              unknown_module_param_cb);
```

In turn, it calls `parse_one()` function for parsing single parameter:

```c
ret = parse_one(param, val, doing, params, num,
				min_level, max_level, arg, unknown);
```

For each parameter passed in, first compare the parameter name against paramters
in `__param` (Note: params[] is an array of type `struct kernel_param` which
is read directly from `__param` section after relocation). Then, after some
checks, set the variable value by calling `set()` function on it.

For our `uint` type parameter, this is done via `strtolfn()` function actually.

```c
if (parameq(param, params[i].name)) {
    ...
    if (param_check_unsafe(&params[i]))
				err = params[i].ops->set(val, &params[i]);
    ...
}
```

[init_module]: https://man7.org/linux/man-pages/man2/init_module.2.html]
[kernel/module.c]: https://github.com/torvalds/linux/blob/master/kernel/module.c
[include/uapi/linux/elf.h]: https://github.com/torvalds/linux/blob/master/include/uapi/linux/elf.h
[ELF]: https://en.wikipedia.org/wiki/Executable_and_Linkable_Format
