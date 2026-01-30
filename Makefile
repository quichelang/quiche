#.PHONY: stage0 stage1 stage2 verify clean

# Host compiler binary (quiche-host)
HOST_BIN_DIR := target/debug
HOST_BIN := $(HOST_BIN_DIR)/metaquiche-host

# Stage 1 (built with host)
STAGE1_TARGET_DIR := target/stage1
STAGE1_BIN := $(STAGE1_TARGET_DIR)/debug/metaquiche-native
STAGE2_TARGET_DIR := target/stage2
STAGE2_BIN := $(STAGE2_TARGET_DIR)/debug/metaquiche-native

# Default target
all: verify

stage0:
	cargo build -p metaquiche-host
	@ln -sf $(HOST_BIN) stage0

stage1: stage0
	@echo "Building Stage 1 (Host -> Self)..."
	QUICHE_STAGE=stage1 QUICHE_COMPILER_BIN=$(abspath $(HOST_BIN)) CARGO_TARGET_DIR=$(STAGE1_TARGET_DIR) cargo build -p metaquiche-native
	@ln -sf $(STAGE1_BIN) stage1

stage2: stage1
	@echo "Building Stage 2 (Stage 1 -> Self)..."
	QUICHE_STAGE=stage2 QUICHE_COMPILER_BIN=$(abspath $(STAGE1_BIN)) CARGO_TARGET_DIR=$(STAGE2_TARGET_DIR) cargo build -p metaquiche-native
	@ln -sf $(STAGE2_BIN) stage2

verify: stage2
	@echo "Verifying Stage 1 output matches Stage 2 output..."
	python3 verify.py diff $(STAGE1_TARGET_DIR)/debug/build/metaquiche-native-*/out $(STAGE2_TARGET_DIR)/debug/build/metaquiche-native-*/out

clean:
	rm -rf target
