use crate::templates::{BASIC_LIB_TEMPLATE, INVALID_INTERFACE_TEMPLATE, MULTI_LIB_TEMPLATE};
use anyhow::Error;
use wasmtime::{component::Component, Engine};

/// Builder for creating mock components
pub struct MockComponentBuilder {
    wat: String,
}

impl MockComponentBuilder {
    /// Create a new builder with the basic library interface template
    pub fn new_basic_lib() -> Self {
        Self {
            wat: BASIC_LIB_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with the invalid interface template
    pub fn new_invalid_interface() -> Self {
        Self {
            wat: INVALID_INTERFACE_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with the multi-library interface template
    pub fn new_multi_lib() -> Self {
        Self {
            wat: MULTI_LIB_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with custom WAT content
    pub fn new_custom(wat: &str) -> Self {
        Self {
            wat: wat.to_string(),
        }
    }

    /// Build the component using the provided engine
    pub fn build(&self, engine: &Engine) -> Result<Component, Error> {
        Component::new(engine, &self.wat)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmtime::Config;

    #[test]
    fn test_build_templates() -> Result<(), Error> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;

        // Test that all templates can be compiled into components
        MockComponentBuilder::new_basic_lib().build(&engine)?;
        MockComponentBuilder::new_invalid_interface().build(&engine)?;
        MockComponentBuilder::new_multi_lib().build(&engine)?;

        Ok(())
    }
}
