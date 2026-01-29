#.PHONY: stage0 stage1 stage2 verify clean

# Host compiler binary (quiche-host)
HOST_BIN_DIR := target/debug
HOST_BIN := $(HOST_BIN_DIR)/quiche-host

# Stage 1 (built with host)
STAGE1_TARGET_DIR := target/stage1
STAGE1_BIN := $(STAGE1_TARGET_DIR)/debug/quiche_self

# Stage 2 (built with stage 1)
STAGE2_TARGET_DIR := target/stage2
STAGE2_BIN := $(STAGE2_TARGET_DIR)/debug/quiche_self

# Default target
all: verify

stage0:
	cargo build -p quiche-host

stage1: stage0
	@echo "Building Stage 1 (Host -> Self)..."
	QUICHE_BOOTSTRAP_BIN=$(abspath $(HOST_BIN)) CARGO_TARGET_DIR=$(STAGE1_TARGET_DIR) cargo build -p quiche_self

stage2: stage1
	@echo "Building Stage 2 (Stage 1 -> Self)..."
	QUICHE_BOOTSTRAP_BIN=$(abspath $(STAGE1_BIN)) CARGO_TARGET_DIR=$(STAGE2_TARGET_DIR) cargo build -p quiche_self

verify: stage2
	@echo "Verifying Stage 1 output matches Stage 2 output..."
	python3 verify.py diff $(STAGE1_TARGET_DIR)/debug/build/quiche_self-*/out $(STAGE2_TARGET_DIR)/debug/build/quiche_self-*/out

clean:
	rm -rf target
