//! Component management functionality.
//!
//! This module provides core functionality for managing WebAssembly components,
//! including function registration, dynamic linking, and component lifecycle management.

mod function_registry;
mod linker;

pub use function_registry::{FunctionRegistry, FunctionRegistryError};
pub use linker::{DynamicLinker, DynamicLinkingError};
