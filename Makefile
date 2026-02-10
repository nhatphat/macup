# Binary name và paths
BINARY_NAME=macup
BUILD_DIR=target/release
OUTPUT_DIR=.

# Default target
.PHONY: all
all: release

# Build release binary
.PHONY: release
release:
	@echo "Building release binary..."
	cargo build --release
	@echo "Copying binary to root..."
	cp $(BUILD_DIR)/$(BINARY_NAME) $(OUTPUT_DIR)/
	@echo "✓ Release binary ready: ./$(BINARY_NAME)"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -f ./$(BINARY_NAME)
	@echo "✓ Clean complete"

# Help
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make release  - Build release binary and copy to root"
	@echo "  make clean    - Remove build artifacts"
	@echo "  make help     - Show this help message"
