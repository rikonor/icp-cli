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

wkg:
	@wkg get \
		icp:cli@0.0.1 \
		--overwrite \
		-o crates/icp-cli/wit/world.wit
