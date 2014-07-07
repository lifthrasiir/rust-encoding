CC ?= gcc
CXX ?= g++
CXXFLAGS ?=
AR ?= ar
RUSTC ?= rustc
RUSTDOC ?= rustdoc
RUSTFLAGS ?= -O
EXT_DEPS ?=

LIB_RS = src/encoding/lib.rs
LIB = ./libencoding.rlib
TEST_BIN = ./rustencoding-test
RUST_SRC = $(shell find src/encoding/. -type f -name '*.rs')

.PHONY: all
all:	$(LIB)

$(LIB): $(LIB_RS) $(RUST_SRC) $(EXT_DEPS)
	$(RUSTC) $(RUSTFLAGS) $< --out-dir $(dir $@)

$(TEST_BIN): $(LIB_RS) $(RUST_SRC)
	$(RUSTC) $(RUSTFLAGS) $< -o $@ --test

.PHONY: doctest
doctest: $(LIB_RS) $(LIB)
	$(RUSTDOC) $< -L . --test

.PHONY: check
check: doctest $(TEST_BIN)
	$(TEST_BIN)

.PHONY: doc
doc: $(LIB_RS) $(RUST_SRC)
	$(RUSTDOC) $<

.PHONY: clean
clean:
	rm -f *.o *.a *.so *.dylib *.rlib *.dll *.exe *-test

