BUILD_DIR ?= $(shell mkdir -p build; echo build)
BIN := my_bin
BIN_SRC := src/main.rs
MACRO_LIB := my_macros
MACRO_LIB_FILE := $(addprefix lib,$(addsuffix .so,$(MACRO_LIB)))
MACRO_SRC := src/lib/lib.rs
MACRO_FLAGS := --emit=obj,link \
			   --extern proc_macro \
			   --crate-type proc-macro \
			   --crate-name $(MACRO_LIB) \
			   --out-dir $(BUILD_DIR)

$(BUILD_DIR)/$(BIN): $(BIN_SRC) $(MACRO_LIB_FILE)
	rustc -o $@ --extern $(MACRO_LIB) -L $(BUILD_DIR) $<

$(MACRO_LIB_FILE): $(MACRO_SRC)
	rustc $(MACRO_FLAGS) $^

.PHONY: clean
clean:
	rm -fr $(BUILD_DIR)
