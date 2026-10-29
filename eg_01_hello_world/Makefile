ifneq ($(KERNELRELEASE),)

# In kbuild context
hello_world-objs := main.o
obj-m := hello_world.o

else
# In normal make context
KDIR ?= /lib/modules/$(shell uname -r)/build
PWD := $(shell pwd)

.PHONY: modules
modules:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

.PHONY: clean
clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

endif
