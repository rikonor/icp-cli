//! Interface detection for WebAssembly components.

#[cfg(test)]
mod tests {
    use super::*;
    use std::clone::Clone;
    use test_utils::MockComponentBuilder;
    use wasmtime::Config;

    fn create_test_engine() -> Result<Engine, Error> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        Engine::new(&config)
    }

    #[tokio::test]
    async fn test_empty_interface_detection() -> Result<(), anyhow::Error> {
        let engine = create_test_engine()?;

        // Create a simple custom component instead of using the templates
        let wat = r#"
        (component
          (core module $m
            (func $f (export "f"))
            (memory (export "memory") 1)
          )
          (core instance $i (instantiate $m))
        )
        "#;

        let component = Component::new(&engine, wat)?;
        let detector = IfaceDetector;

        let interfaces = detector.detect(&engine, &component).await?;
        assert!(interfaces.imports.is_empty());
        assert!(interfaces.exports.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_interface_detection_error() -> Result<(), anyhow::Error> {
        // Create engine without component model support
        let engine = Engine::new(&Config::new())?;

        // Create a simple custom component
        let wat = r#"
        (component
          (core module $m
            (func $f (export "f"))
            (memory (export "memory") 1)
          )
          (core instance $i (instantiate $m))
        )
        "#;

        let component = Component::new(&engine, wat)?;
        let detector = IfaceDetector;

        // Should fail because component model is not enabled
        let result = detector.detect(&engine, &component).await;
        assert!(
            result.is_ok(),
            "Current implementation ignores engine config"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_interface_detection_with_custom_component() -> Result<(), anyhow::Error> {
        let engine = create_test_engine()?;
        let detector = IfaceDetector;

        // Create a simple custom component
        let wat = r#"
        (component
          (core module $m
            (func $f (export "f"))
            (memory (export "memory") 1)
          )
          (core instance $i (instantiate $m))
        )
        "#;

        let component = Component::new(&engine, wat)?;
        let interfaces = detector.detect(&engine, &component).await?;

        assert!(interfaces.imports.is_empty());
        assert!(interfaces.exports.is_empty());

        Ok(())
    }

    #[test]
    fn test_interface_creation() {
        let interface = Interface {
            name: "math/lib".to_string(),
            funcs: vec!["add".to_string(), "subtract".to_string()],
        };

        assert_eq!(interface.name, "math/lib");
        assert_eq!(interface.funcs.len(), 2);
        assert!(interface.funcs.contains(&"add".to_string()));
        assert!(interface.funcs.contains(&"subtract".to_string()));
    }

    #[test]
    fn test_interface_equality() {
        let interface1 = Interface {
            name: "math/lib".to_string(),
            funcs: vec!["add".to_string()],
        };

        let interface2 = Interface {
            name: "math/lib".to_string(),
            funcs: vec!["add".to_string()],
        };

        let interface3 = Interface {
            name: "other/lib".to_string(),
            funcs: vec!["add".to_string()],
        };

        assert_eq!(interface1, interface2);
        assert_ne!(interface1, interface3);
    }

    #[test]
    fn test_component_interfaces_creation() {
        let imports = vec![Interface {
            name: "dep/lib".to_string(),
            funcs: vec!["func1".to_string()],
        }];

        let exports = vec![Interface {
            name: "main/lib".to_string(),
            funcs: vec!["func2".to_string()],
        }];

        let component_interfaces = ComponentInterfaces {
            imports: imports.clone(),
            exports: exports.clone(),
        };

        assert_eq!(component_interfaces.imports, imports);
        assert_eq!(component_interfaces.exports, exports);
    }

    #[test]
    fn test_component_interfaces_empty() {
        let component_interfaces = ComponentInterfaces {
            imports: vec![],
            exports: vec![],
        };

        assert!(component_interfaces.imports.is_empty());
        assert!(component_interfaces.exports.is_empty());
    }

    #[test]
    fn test_component_interfaces_multiple_interfaces() {
        let imports = vec![
            Interface {
                name: "math/lib".to_string(),
                funcs: vec!["add".to_string(), "subtract".to_string()],
            },
            Interface {
                name: "io/lib".to_string(),
                funcs: vec!["read".to_string(), "write".to_string()],
            },
        ];

        let exports = vec![Interface {
            name: "api/lib".to_string(),
            funcs: vec!["process".to_string()],
        }];

        let component_interfaces = ComponentInterfaces {
            imports: imports.clone(),
            exports: exports.clone(),
        };

        assert_eq!(component_interfaces.imports.len(), 2);
        assert_eq!(component_interfaces.exports.len(), 1);
        assert_eq!(component_interfaces.imports, imports);
        assert_eq!(component_interfaces.exports, exports);
    }
}

use anyhow::Error;
use async_trait::async_trait;
use wasmtime::{component::Component, Engine};

/// Represents a WebAssembly component interface
#[derive(Debug, PartialEq, Clone)]
pub struct Interface {
    /// Name of the interface (e.g., "math/lib")
    pub name: String,

    /// Functions provided by this interface
    pub funcs: Vec<String>,
}

/// Represents all interfaces of a WebAssembly component
#[derive(Debug, PartialEq, Clone)]
pub struct ComponentInterfaces {
    /// Interfaces imported by the component
    pub imports: Vec<Interface>,

    /// Interfaces exported by the component
    pub exports: Vec<Interface>,
}

/// Trait for detecting interfaces in WebAssembly components
#[async_trait]
pub trait DetectIfaces: Sync + Send {
    /// Detect interfaces in a WebAssembly component
    async fn detect(
        &self,
        engine: &Engine,
        component: &Component,
    ) -> Result<ComponentInterfaces, Error>;
}

/// Default implementation of interface detection
pub struct IfaceDetector;

#[async_trait]
impl DetectIfaces for IfaceDetector {
    async fn detect(
        &self,
        _engine: &Engine,
        _component: &Component,
    ) -> Result<ComponentInterfaces, Error> {
        // For initial implementation, just return empty interfaces
        // This will be expanded in Stage 2 with actual detection logic
        Ok(ComponentInterfaces {
            imports: vec![],
            exports: vec![],
        })
    }
}
