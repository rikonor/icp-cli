EXTENSION_NAME =

CARGO_RELEASE     ?=
CARGO_TARGET      ?= wasm32-unknown-unknown

CARGO_TARGET_DIR   ?= target$(if $(CARGO_TARGET),/$(CARGO_TARGET))
CARGO_ARTIFACT_DIR ?= $(CARGO_TARGET_DIR)/$(if $(CARGO_RELEASE),release,debug)

COMPONENT_OUT_DIR ?= $(CARGO_ARTIFACT_DIR)

all: component output-path

build:
	@cargo build \
		--package $(EXTENSION_NAME) \
		$(if $(CARGO_TARGET),--target $(CARGO_TARGET)) \
		$(if $(CARGO_RELEASE),--release)

component: build
	@wasm-tools component new \
		$(CARGO_ARTIFACT_DIR)/$(EXTENSION_NAME).wasm \
		> $(COMPONENT_OUT_DIR)/$(EXTENSION_NAME).component.wasm \

output-path:
	@realpath $(COMPONENT_OUT_DIR)/$(EXTENSION_NAME).component.wasm

test-quick-install:
	@mkdir -p dist/binaries/icp dist/binaries/extensions
	@echo "test content" > dist/binaries/icp/icp-x86_64-apple-darwin-standard
	@echo "test content" > dist/binaries/extensions/multiply.component.wasm
	@echo "test content" > dist/binaries/extensions/power.component.wasm
	@echo "test content" | sha256sum | cut -d' ' -f1 > dist/checksums.txt
	@echo "$$(cat dist/checksums.txt) icp-x86_64-apple-darwin-standard" > dist/checksums.txt
	@echo "$$(cat dist/checksums.txt | head -n1) multiply.component.wasm" >> dist/checksums.txt
	@echo "$$(cat dist/checksums.txt | head -n1) power.component.wasm" >> dist/checksums.txt
	@cargo run --bin generate_scripts -- \
		--binary-path dist/binaries/icp \
		--extensions-path dist/binaries/extensions \
		--checksums-path dist/checksums.txt \
		--output-dir dist \
		--domain example.com
	@echo "\nGeneration complete! View the result with:\n  open dist/index.html\n"
