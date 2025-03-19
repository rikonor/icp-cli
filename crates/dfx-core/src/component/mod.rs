//! Component management functionality.
//!
//! This module provides core functionality for managing WebAssembly components,
//! including function registration, dynamic linking, and component lifecycle management.

pub mod function_registry;
pub mod linker;

pub use function_registry::{FunctionRegistry, FunctionRegistryError};
pub use linker::{DynamicLinker, DynamicLinkingError};
