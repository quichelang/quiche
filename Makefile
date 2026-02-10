.PHONY: all build release install test clean

# Default target
all: build

# Debug build
build:
	cargo build -p quiche

# Release build (optimized)
release:
	cargo build -p quiche --release

# Install to ~/.cargo/bin
install: release
	@echo "Installing Quiche compiler to $$HOME/.cargo/bin..."
	@mkdir -p $$HOME/.cargo/bin
	cp target/release/quiche $$HOME/.cargo/bin/quiche
	@echo "Installed! Run 'quiche --help' to verify."

# Run tests
test:
	cargo test -p quiche

clean:
	cargo clean
