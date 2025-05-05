EXTENSION_NAME =

CARGO_RELEASE     ?=
CARGO_TARGET      ?= wasm32-unknown-unknown

CARGO_TARGET_DIR   ?= target$(if $(CARGO_TARGET),/$(CARGO_TARGET))
CARGO_ARTIFACT_DIR ?= $(CARGO_TARGET_DIR)/$(if $(CARGO_RELEASE),release,debug)

COMPONENT_OUT_DIR ?= $(CARGO_ARTIFACT_DIR)

# --- Extension Building (Requires EXTENSION_NAME to be set) ---

# Default target: Build the extension component and print its path
all: component output-path

# Build the extension crate (Wasm binary)
build:
	@cargo build \
		--package $(EXTENSION_NAME) \
		$(if $(CARGO_TARGET),--target $(CARGO_TARGET)) \
		$(if $(CARGO_RELEASE),--release)

component: build
	@wasm-tools component new \
		$(CARGO_ARTIFACT_DIR)/$(EXTENSION_NAME).wasm \
		> $(COMPONENT_OUT_DIR)/$(EXTENSION_NAME).component.wasm \

# Print the real path to the built extension component
output-path:
	@realpath $(COMPONENT_OUT_DIR)/$(EXTENSION_NAME).component.wasm

# --- End Extension Building ---

# --- Local CLI Development ---

# Build and install the icp-cli binary locally
# Usage:
#   make install-cli          (Debug build)
#   make install-cli CARGO_RELEASE=1 (Release build)
.PHONY: install-cli
install-cli:
	@echo "--- Building and installing icp-cli locally ---"
	@cargo install --path crates/icp-cli $(if $(CARGO_RELEASE),--release) --force
	@echo "--- icp-cli installed successfully ---"

# --- End Local CLI Development ---

# --- WIT Package Management ---

WIT_BUILD_DIR := target/wit-build
ORDER_WIT_SCRIPT := python3 scripts/order_wit_packages.py

# Build a single WIT package
# Usage: make wit-build DIR=wit/cli
.PHONY: wit-build
wit-build:
ifndef DIR
	$(error DIR is not set. Usage: make wit-build DIR=<path>)
endif
	@echo "--- Building WIT package in $(DIR) ---"
	@mkdir -p $(WIT_BUILD_DIR)
	@pkg_name=$$(basename $(DIR)); \
	wasm_file="$(WIT_BUILD_DIR)/$${pkg_name}.wasm"; \
	wkg wit build -d $(DIR) -o $$wasm_file

# Publish a single pre-built WIT package
# Usage: make wit-publish DIR=wit/cli
.PHONY: wit-publish
wit-publish:
ifndef DIR
	$(error DIR is not set. Usage: make wit-publish DIR=<path>)
endif
	@echo "--- Publishing WIT package from $(DIR) ---"
	@pkg_name=$$(basename $(DIR)); \
	wasm_file="$(WIT_BUILD_DIR)/$${pkg_name}.wasm"; \
	if [ ! -f $$wasm_file ]; then \
		echo "Error: WASM file $$wasm_file not found. Build first with 'make wit-build DIR=$(DIR)'?" >&2; \
		exit 1; \
	fi; \
	wkg publish $$wasm_file

# Build and publish all WIT packages in dependency order
.PHONY: wit-build-publish-all
wit-build-publish-all:
	@echo "Determining WIT package build order..."
	@ORDERED_DIRS=$$($(ORDER_WIT_SCRIPT)); \
	if [ -z "$$ORDERED_DIRS" ]; then \
		echo "Error: Failed to get WIT package order from script." >&2; \
		exit 1; \
	fi; \
	echo "Building and publishing WIT packages in order:"; \
	for dir in $$ORDERED_DIRS; do \
		$(MAKE) wit-build DIR=$$dir || exit 1; \
		$(MAKE) wit-publish DIR=$$dir || exit 1; \
	done
	@echo "All WIT packages built and published successfully."

# --- End WIT Package Management ---

# --- Release Management ---

# Release the core CLI (updates workspace version, commits, tags vX.Y.Z, pushes)
# Usage: make release-cli VERSION=X.Y.Z
.PHONY: release-cli
release-cli:
ifndef VERSION
	$(error VERSION is not set. Usage: make release-cli VERSION=<X.Y.Z>)
endif
	@echo "--- Running Core CLI Release Script for v$(VERSION) ---"
	@scripts/release-cli.sh $(VERSION)
	@echo "--- Core CLI Release Script Finished ---"

# Release an individual extension (updates extension version, commits, tags <NAME>-vX.Y.Z, pushes)
# Usage: make release-extension NAME=project VERSION=X.Y.Z
.PHONY: release-extension
release-extension:
ifndef NAME
	$(error NAME is not set. Usage: make release-extension NAME=<name> VERSION=<X.Y.Z>)
endif
ifndef VERSION
	$(error VERSION is not set. Usage: make release-extension NAME=<name> VERSION=<X.Y.Z>)
endif
	@echo "--- Running Extension Release Script for $(NAME) v$(VERSION) ---"
	@scripts/release-extension.sh $(NAME) $(VERSION)
	@echo "--- Extension Release Script Finished ---"

# --- End Release Management ---
