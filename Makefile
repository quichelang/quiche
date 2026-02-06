.PHONY: all bootstrap stage0 stage1 stage2 bootstrap-verify verify diff-stages bootstrap-release quiche quiche-release release install test test-bootstrap test-quiche clean

# Host compiler binary (mq0)
HOST_BIN_DIR := target/debug
HOST_BIN := $(HOST_BIN_DIR)/mq0

# Stage 1 (built with host)
STAGE1_TARGET_DIR := target/stage1
STAGE1_BIN := $(STAGE1_TARGET_DIR)/debug/mq
STAGE2_TARGET_DIR := target/stage2
STAGE2_BIN := $(STAGE2_TARGET_DIR)/debug/mq

# Release builds
STAGE1_RELEASE_BIN := $(STAGE1_TARGET_DIR)/release/mq
STAGE2_RELEASE_BIN := $(STAGE2_TARGET_DIR)/release/mq

# Quiche compiler (user-facing .q compiler)
QUICHE_TARGET_DIR := target/quiche
QUICHE_BIN := $(QUICHE_TARGET_DIR)/debug/mq
QUICHE_RELEASE_BIN := $(QUICHE_TARGET_DIR)/release/mq

# Output directory for binaries
BIN_DIR := bin

# Default target
all: bootstrap-verify quiche

# Create bin directory and symlinks
$(BIN_DIR):
	@mkdir -p $(BIN_DIR)

stage0: $(BIN_DIR)
	cargo build -p metaquiche-host
	@ln -sf ../$(HOST_BIN) $(BIN_DIR)/mq0
	@ln -sf ../$(HOST_BIN) $(BIN_DIR)/stage0

stage1: stage0 $(BIN_DIR)
	@echo "Building Stage 1 (Host -> Self)..."
	QUICHE_STAGE=stage1 QUICHE_COMPILER_BIN=$(abspath $(HOST_BIN)) CARGO_TARGET_DIR=$(STAGE1_TARGET_DIR) cargo build -p metaquiche-native
	@ln -sf ../$(STAGE1_BIN) $(BIN_DIR)/mq1
	@ln -sf ../$(STAGE1_BIN) $(BIN_DIR)/stage1

stage2: stage1 $(BIN_DIR)
	@echo "Building Stage 2 (Stage 1 -> Self)..."
	QUICHE_STAGE=stage2 QUICHE_COMPILER_BIN=$(abspath $(STAGE1_BIN)) CARGO_TARGET_DIR=$(STAGE2_TARGET_DIR) cargo build -p metaquiche-native
	@ln -sf ../$(STAGE2_BIN) $(BIN_DIR)/mq
	@ln -sf ../$(STAGE2_BIN) $(BIN_DIR)/stage2

bootstrap: stage2

bootstrap-verify: stage2
	@echo "Verifying Stage 1 output matches Stage 2 output..."
	python3 verify.py diff '$(STAGE1_TARGET_DIR)/debug/build/metaquiche-native-*/out' '$(STAGE2_TARGET_DIR)/debug/build/metaquiche-native-*/out'

verify: bootstrap-verify

diff-stages: stage2
	@echo "Showing differences between Stage 1 and Stage 2..."
	python3 verify.py show-diff '$(STAGE1_TARGET_DIR)/debug/build/metaquiche-native-*/out' '$(STAGE2_TARGET_DIR)/debug/build/metaquiche-native-*/out'

# Release builds (optimized)
bootstrap-release: stage1
	@echo "Building Release Stage 1..."
	QUICHE_STAGE=stage1 QUICHE_COMPILER_BIN=$(abspath $(HOST_BIN)) CARGO_TARGET_DIR=$(STAGE1_TARGET_DIR) cargo build -p metaquiche-native --release
	@echo "Building Release Stage 2..."
	QUICHE_STAGE=stage2 QUICHE_COMPILER_BIN=$(abspath $(STAGE1_RELEASE_BIN)) CARGO_TARGET_DIR=$(STAGE2_TARGET_DIR) cargo build -p metaquiche-native --release
	@mkdir -p $(BIN_DIR)
	@ln -sf ../$(STAGE2_RELEASE_BIN) $(BIN_DIR)/stage2-release
	@echo "Bootstrap release binary: $(BIN_DIR)/stage2-release"

quiche: stage2 $(BIN_DIR)
	@echo "Building Quiche compiler (MetaQuiche -> Quiche)..."
	QUICHE_STAGE=quiche QUICHE_COMPILER_BIN=$(abspath $(STAGE2_BIN)) CARGO_TARGET_DIR=$(QUICHE_TARGET_DIR) cargo build -p quiche-compiler
	@ln -sf ../$(QUICHE_BIN) $(BIN_DIR)/quiche
	@ln -sf ../$(QUICHE_BIN) $(BIN_DIR)/mq-quiche
	@echo "Quiche compiler binary: $(BIN_DIR)/quiche"

quiche-release: stage2 $(BIN_DIR)
	@echo "Building Quiche compiler release..."
	QUICHE_STAGE=quiche QUICHE_COMPILER_BIN=$(abspath $(STAGE2_BIN)) CARGO_TARGET_DIR=$(QUICHE_TARGET_DIR) cargo build -p quiche-compiler --release
	@ln -sf ../$(QUICHE_RELEASE_BIN) $(BIN_DIR)/quiche
	@ln -sf ../$(QUICHE_RELEASE_BIN) $(BIN_DIR)/mq-quiche
	@echo "Quiche release binary: $(BIN_DIR)/quiche"

release: quiche-release

# Install binaries to per-user cargo path
install: quiche-release
	@echo "Installing Quiche compiler to $$HOME/.cargo/bin..."
	@mkdir -p $$HOME/.cargo/bin
	cp $(QUICHE_RELEASE_BIN) $$HOME/.cargo/bin/quiche
	cp $(QUICHE_RELEASE_BIN) $$HOME/.cargo/bin/mq
	@echo "Installed! Run 'quiche --help' (or 'mq --help') to verify."

test: test-bootstrap test-quiche

test-bootstrap: stage2
	@echo "Running bootstrap regression test..."
	./$(STAGE2_BIN) tests/test_private_visibility.qrs
	@echo "Bootstrap regression test passed!"

test-quiche: quiche
	@echo "Running Quiche (.q) smoke tests..."
	./$(QUICHE_BIN) tests/test_codegen_scope_regression.q
	./$(QUICHE_BIN) tests/test_comprehensions.q
	./$(QUICHE_BIN) tests/test_fstring.q
	@echo "Quiche smoke tests passed!"

clean:
	rm -rf target bin stage0 stage1 stage2
