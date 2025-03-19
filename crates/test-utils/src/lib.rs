//! Test utilities for working with WebAssembly components
//!
//! This crate provides tools for creating and working with mock components
//! in tests, particularly focusing on testing interface detection and
//! library interface validation.

mod mock;
mod templates;

pub use mock::MockComponentBuilder;
pub use templates::{BASIC_LIB_TEMPLATE, MULTI_LIB_TEMPLATE};

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Error;
    use wasmtime::{Config, Engine};

    #[test]
    fn test_builders() -> Result<(), Error> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;

        // Test that all builders create valid components
        MockComponentBuilder::new_basic_lib().build(&engine)?;
        MockComponentBuilder::new_multi_lib().build(&engine)?;

        // Test custom WAT
        let wat = r#"
        (component
          (core module $impl
            (memory (export "mem") 1)
            (func $noop (export "noop"))
          )
          (core instance $instance (instantiate $impl))
        )"#;
        MockComponentBuilder::new_custom(wat).build(&engine)?;

        Ok(())
    }
}
