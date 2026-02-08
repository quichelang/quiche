.PHONY: all stage0 stage1 stage2 bootstrap verify diff-stages release install test clean

# Stage 0: Host compiler (Rust) — quiche-host
HOST_BIN_DIR := target/debug
HOST_BIN := $(HOST_BIN_DIR)/quiche-host

# Stage 1: quiche-compiler built by Stage 0
STAGE1_TARGET_DIR := target/stage1
STAGE1_BIN := $(STAGE1_TARGET_DIR)/debug/quiche

# Stage 2: quiche-compiler built by Stage 1 (self-hosting verification)
STAGE2_TARGET_DIR := target/stage2
STAGE2_BIN := $(STAGE2_TARGET_DIR)/debug/quiche

# Release builds
STAGE1_RELEASE_BIN := $(STAGE1_TARGET_DIR)/release/quiche
STAGE2_RELEASE_BIN := $(STAGE2_TARGET_DIR)/release/quiche

# Output directory for binaries
BIN_DIR := bin

# Default target
all: stage2

# Create bin directory
$(BIN_DIR):
	@mkdir -p $(BIN_DIR)

stage0: $(BIN_DIR)
	cargo build -p quiche-host
	@ln -sf ../$(HOST_BIN) $(BIN_DIR)/quiche-host
	@ln -sf ../$(HOST_BIN) $(BIN_DIR)/stage0

stage1: stage0 $(BIN_DIR)
	@echo "Building Stage 1 (quiche-host → quiche-compiler)..."
	QUICHE_COMPILER_BIN=$(abspath $(HOST_BIN)) CARGO_TARGET_DIR=$(STAGE1_TARGET_DIR) cargo build -p quiche-compiler
	@ln -sf ../$(STAGE1_BIN) $(BIN_DIR)/stage1

stage2: stage1 $(BIN_DIR)
	@echo "Building Stage 2 (Stage 1 → quiche-compiler)..."
	QUICHE_COMPILER_BIN=$(abspath $(STAGE1_BIN)) CARGO_TARGET_DIR=$(STAGE2_TARGET_DIR) cargo build -p quiche-compiler
	@ln -sf ../$(STAGE2_BIN) $(BIN_DIR)/quiche
	@ln -sf ../$(STAGE2_BIN) $(BIN_DIR)/stage2

bootstrap: stage2

verify: stage2
	@echo "Verifying Stage 1 output matches Stage 2 output..."
	python3 verify.py diff '$(STAGE1_TARGET_DIR)/debug/build/quiche-compiler-*/out' '$(STAGE2_TARGET_DIR)/debug/build/quiche-compiler-*/out'

diff-stages: stage2
	@echo "Showing differences between Stage 1 and Stage 2..."
	python3 verify.py show-diff '$(STAGE1_TARGET_DIR)/debug/build/quiche-compiler-*/out' '$(STAGE2_TARGET_DIR)/debug/build/quiche-compiler-*/out'

# Release builds (optimized)
release: stage1
	@echo "Building Release Stage 1..."
	QUICHE_COMPILER_BIN=$(abspath $(HOST_BIN)) CARGO_TARGET_DIR=$(STAGE1_TARGET_DIR) cargo build -p quiche-compiler --release
	@echo "Building Release Stage 2..."
	QUICHE_COMPILER_BIN=$(abspath $(STAGE1_RELEASE_BIN)) CARGO_TARGET_DIR=$(STAGE2_TARGET_DIR) cargo build -p quiche-compiler --release
	@mkdir -p $(BIN_DIR)
	@ln -sf ../$(STAGE2_RELEASE_BIN) $(BIN_DIR)/quiche
	@echo "Release binary: $(BIN_DIR)/quiche"

# Install to user cargo path
install: release
	@echo "Installing Quiche compiler to $$HOME/.cargo/bin..."
	@mkdir -p $$HOME/.cargo/bin
	cp $(STAGE2_RELEASE_BIN) $$HOME/.cargo/bin/quiche
	@echo "Installed! Run 'quiche --help' to verify."

test: stage2
	@echo "Running smoke tests..."
	./$(STAGE2_BIN) tests/test_codegen_scope_regression.q
	./$(STAGE2_BIN) tests/test_comprehensions.q
	./$(STAGE2_BIN) tests/test_fstring.q
	@echo "Smoke tests passed!"

clean:
	rm -rf target bin
