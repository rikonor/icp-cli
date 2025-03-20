//! Interface detection and management for WebAssembly components.
//!
//! This module provides functionality for detecting and managing interfaces in WebAssembly components.
//! It allows for inspecting components to determine what interfaces they import and export, which is
//! essential for dependency resolution and extension management.
//!
//! # Interface Detection
//!
//! The primary functionality is provided through the [`DetectIfaces`] trait, which defines methods
//! for detecting interfaces in WebAssembly components. The default implementation is [`IfaceDetector`].
//!
//! ## Example
//!
//! ```rust,no_run
//! use icp_core::interface::{DetectIfaces, IfaceDetector};
//! use wasmtime::{Config, Engine, component::Component};
//! use anyhow::Result;
//!
//! async fn detect_interfaces(path: &str) -> Result<()> {
//!     // Create a WebAssembly engine with component model support
//!     let mut config = Config::new();
//!     config.wasm_component_model(true).async_support(true);
//!     let engine = Engine::new(&config)?;
//!
//!     // Load a WebAssembly component
//!     let component = Component::from_file(&engine, path)?;
//!
//!     // Create an interface detector
//!     let detector = IfaceDetector;
//!
//!     // Detect interfaces in the component
//!     let interfaces = detector.detect(&engine, &component).await?;
//!
//!     // Print the detected interfaces
//!     println!("Imports:");
//!     for interface in &interfaces.imports {
//!         println!("  {}", interface.name);
//!         for func in &interface.funcs {
//!             println!("    - {}", func);
//!         }
//!     }
//!
//!     println!("Exports:");
//!     for interface in &interfaces.exports {
//!         println!("  {}", interface.name);
//!         for func in &interface.funcs {
//!             println!("    - {}", func);
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Library Interfaces
//!
//! Library interfaces are identified by the suffix [`LIBRARY_SUFFIX`], which is currently defined as "/lib".
//! These interfaces are used for cross-extension communication and dependency resolution.
//!
//! # Error Handling
//!
//! Interface detection can fail for various reasons, such as:
//!
//! - The component model is not enabled in the engine
//! - The component has an invalid format
//! - Interface parsing fails
//! - Required elements are missing
//!
//! These errors are represented by the [`InterfaceError`](crate::error::InterfaceError) enum in the
//! [`error`](crate::error) module.

mod detector;

pub use detector::{ComponentInterfaces, DetectIfaces, IfaceDetector, Interface};

/// Suffix used for library interfaces
///
/// This constant defines the suffix that identifies library interfaces, which are used for
/// cross-extension communication and dependency resolution. Currently, this is set to "/lib".
///
/// # Example
///
/// ```
/// use icp_core::interface::LIBRARY_SUFFIX;
///
/// let interface_name = "math/lib";
/// assert!(interface_name.ends_with(LIBRARY_SUFFIX));
/// ```
pub const LIBRARY_SUFFIX: &str = "/lib";
