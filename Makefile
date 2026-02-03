.PHONY: stage0 stage1 stage2 verify diff-stages clean all release install

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

# Output directory for binaries
BIN_DIR := bin

# Default target
all: verify

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

# Release builds (optimized)
release: stage1
	@echo "Building Release Stage 1..."
	QUICHE_STAGE=stage1 QUICHE_COMPILER_BIN=$(abspath $(HOST_BIN)) CARGO_TARGET_DIR=$(STAGE1_TARGET_DIR) cargo build -p metaquiche-native --release
	@echo "Building Release Stage 2..."
	QUICHE_STAGE=stage2 QUICHE_COMPILER_BIN=$(abspath $(STAGE1_RELEASE_BIN)) CARGO_TARGET_DIR=$(STAGE2_TARGET_DIR) cargo build -p metaquiche-native --release
	@mkdir -p $(BIN_DIR)
	@ln -sf ../$(STAGE2_RELEASE_BIN) $(BIN_DIR)/mq
	@echo "Release binary: $(BIN_DIR)/mq"

# Install to /usr/local/bin (requires sudo)
install: release
	@echo "Installing mq to /usr/local/bin..."
	sudo cp $(STAGE2_RELEASE_BIN) /usr/local/bin/mq
	@echo "Installed! Run 'mq --help' to verify."

verify: stage2
	@echo "Verifying Stage 1 output matches Stage 2 output..."
	python3 verify.py diff $(STAGE1_TARGET_DIR)/debug/build/metaquiche-native-*/out $(STAGE2_TARGET_DIR)/debug/build/metaquiche-native-*/out

diff-stages: stage2
	@echo "Showing differences between Stage 1 and Stage 2..."
	python3 verify.py show-diff $(STAGE1_TARGET_DIR)/debug/build/metaquiche-native-*/out $(STAGE2_TARGET_DIR)/debug/build/metaquiche-native-*/out

clean:
	rm -rf target bin stage0 stage1 stage2

test: stage2
	@echo "Running regression tests..."
	./target/stage2/debug/mq tests/test_private_visibility.qrs
	@echo "Regression tests passed!"
