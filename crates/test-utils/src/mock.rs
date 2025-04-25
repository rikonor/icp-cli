use crate::templates::{
    BASIC_LIB_TEMPLATE, EMPTY_COMPONENT_TEMPLATE, EXTENSION_MINIMAL_TEMPLATE,
    MANY_INTERFACES_TEMPLATE, MIXED_VERSIONED_LIB_TEMPLATE, MULTI_LIB_TEMPLATE,
    VERSIONED_LIB_TEMPLATE,
};
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

    /// Create a new builder with the multi-library interface template
    pub fn new_multi_lib() -> Self {
        Self {
            wat: MULTI_LIB_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with an empty component template
    pub fn new_empty_component() -> Self {
        Self {
            wat: EMPTY_COMPONENT_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with many interfaces template
    pub fn new_many_interfaces() -> Self {
        Self {
            wat: MANY_INTERFACES_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with the minimal extension template
    pub fn new_extension_minimal() -> Self {
        Self {
            wat: EXTENSION_MINIMAL_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with the versioned library interface template
    pub fn new_versioned_lib() -> Self {
        Self {
            wat: VERSIONED_LIB_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with the mixed versioned and non-versioned library interface template
    pub fn new_mixed_versioned_lib() -> Self {
        Self {
            wat: MIXED_VERSIONED_LIB_TEMPLATE.to_string(),
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
        MockComponentBuilder::new_multi_lib().build(&engine)?;
        MockComponentBuilder::new_empty_component().build(&engine)?;
        MockComponentBuilder::new_many_interfaces().build(&engine)?;
        MockComponentBuilder::new_extension_minimal().build(&engine)?;
        MockComponentBuilder::new_versioned_lib().build(&engine)?;
        MockComponentBuilder::new_mixed_versioned_lib().build(&engine)?;

        Ok(())
    }
}
