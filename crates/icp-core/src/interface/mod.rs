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
//! # Inter-Extension and Host Interfaces
//!
//! Extensions can import and export interfaces for several purposes:
//! - **Inter-extension communication:** Extensions can export interfaces (e.g., `my-package:my-iface/api`)
//!   that other extensions can import and use. The system facilitates dynamic linking for these.
//!   Previously, a "/lib" suffix was a convention for identifying such interfaces,
//!   but the system now considers all non-host interfaces for potential inter-extension linking.
//! - **Host-provided interfaces:** Extensions can import interfaces provided by the host CLI application
//!   itself (e.g., "icp:cli/misc", "icp:cli/filesystem"). These are identified by the
//!   [`HOST_INTERFACE_PREFIX`] and are linked directly by the host.
//! - **CLI command definition:** Extensions export a specific interface (typically `icp:cli/cli`)
//!   that the host uses to integrate the extension's commands.
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

/// Prefix used to identify interfaces provided by the host CLI application.
///
/// These interfaces (e.g., "icp:cli/misc", "icp:cli/filesystem") are implemented
/// by the icp-cli host and imported by extensions. They are handled differently
/// during dependency validation and linking compared to inter-extension interfaces.
pub const HOST_INTERFACE_PREFIX: &str = "icp:cli/";

/// Parse an interface name into its base name and version components
///
/// This function extracts the base name and version from an interface name.
/// For versioned interfaces (e.g., "math/lib@1.0.0"), it returns the base name
/// ("math/lib") and the version ("1.0.0").
/// For non-versioned interfaces (e.g., "math/lib"), it returns the name and None.
///
/// # Examples
///
/// ```
/// use icp_core::interface::parse_interface_name;
///
/// let (base, version) = parse_interface_name("math/lib@1.0.0");
/// assert_eq!(base, "math/lib");
/// assert_eq!(version, Some("1.0.0".to_string()));
///
/// let (base, version) = parse_interface_name("math/lib");
/// assert_eq!(base, "math/lib");
/// assert_eq!(version, None);
/// ```
pub fn parse_interface_name(name: &str) -> (String, Option<String>) {
    if let Some(idx) = name.rfind('@') {
        let base_name = name[..idx].to_string();
        let version = name[idx + 1..].to_string();
        (base_name, Some(version))
    } else {
        (name.to_string(), None)
    }
}
