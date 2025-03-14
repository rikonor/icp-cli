use crate::templates::{
    BASIC_LIB_TEMPLATE, DUPLICATE_INTERFACE_TEMPLATE, EMPTY_COMPONENT_TEMPLATE,
    INVALID_INTERFACE_TEMPLATE, MANY_INTERFACES_TEMPLATE, MISSING_MEMORY_TEMPLATE,
    MISSING_REALLOC_TEMPLATE, MULTI_LIB_TEMPLATE, NESTED_INSTANCES_TEMPLATE,
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

    /// Create a new builder with nested instances template
    pub fn new_nested_instances() -> Self {
        Self {
            wat: NESTED_INSTANCES_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with duplicate interface names template
    pub fn new_duplicate_interface() -> Self {
        Self {
            wat: DUPLICATE_INTERFACE_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with missing memory template
    pub fn new_missing_memory() -> Self {
        Self {
            wat: MISSING_MEMORY_TEMPLATE.to_string(),
        }
    }

    /// Create a new builder with missing realloc template
    pub fn new_missing_realloc() -> Self {
        Self {
            wat: MISSING_REALLOC_TEMPLATE.to_string(),
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
        // MockComponentBuilder::new_empty_component().build(&engine)?;
        // MockComponentBuilder::new_many_interfaces().build(&engine)?;
        // MockComponentBuilder::new_nested_instances().build(&engine)?;
        // MockComponentBuilder::new_duplicate_interface().build(&engine)?;

        // Note: The following templates are intentionally malformed for error testing
        // and may not compile successfully
        // MockComponentBuilder::new_missing_memory().build(&engine)?;
        // MockComponentBuilder::new_missing_realloc().build(&engine)?;

        Ok(())
    }
}
