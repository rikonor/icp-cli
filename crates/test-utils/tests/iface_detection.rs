use anyhow::Error;
use dfx_core::interface::{ComponentInterfaces, DetectIfaces, IfaceDetector, Interface};
use test_utils::MockComponentBuilder;
use wasmtime::{Config, Engine};

#[tokio::test]
async fn test_basic_lib_detection() -> Result<(), Error> {
    let mut config = Config::new();
    config.wasm_component_model(true).async_support(true);
    let engine = Engine::new(&config)?;

    // Create component with basic library interfaces
    let component = MockComponentBuilder::new_basic_lib().build(&engine)?;

    // Test interface detection
    let interfaces = IfaceDetector.detect(&engine, &component).await?;

    // Verify imports
    assert_eq!(interfaces.imports.len(), 1);
    assert_eq!(
        interfaces.imports[0],
        Interface {
            name: "test:math/lib".to_string(),
            funcs: vec!["add".to_string()],
        }
    );

    // Verify exports
    assert_eq!(interfaces.exports.len(), 1);
    assert_eq!(
        interfaces.exports[0],
        Interface {
            name: "test:calc/lib".to_string(),
            funcs: vec!["multiply".to_string()],
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_invalid_interface_detection() -> Result<(), Error> {
    let mut config = Config::new();
    config.wasm_component_model(true).async_support(true);
    let engine = Engine::new(&config)?;

    // Create component with non-library interfaces
    let component = MockComponentBuilder::new_invalid_interface().build(&engine)?;

    // Test interface detection
    let interfaces = IfaceDetector.detect(&engine, &component).await?;

    // Verify imports
    assert_eq!(interfaces.imports.len(), 1);
    assert_eq!(
        interfaces.imports[0],
        Interface {
            name: "test:math/helper".to_string(),
            funcs: vec!["log".to_string()],
        }
    );

    // Verify exports
    assert_eq!(interfaces.exports.len(), 1);
    assert_eq!(
        interfaces.exports[0],
        Interface {
            name: "test:calc/utils".to_string(),
            funcs: vec!["square".to_string()],
        }
    );

    Ok(())
}

#[tokio::test]
async fn test_multi_lib_detection() -> Result<(), Error> {
    let mut config = Config::new();
    config.wasm_component_model(true).async_support(true);
    let engine = Engine::new(&config)?;

    // Create component with multiple library interfaces
    let component = MockComponentBuilder::new_multi_lib().build(&engine)?;

    // Test interface detection
    let interfaces = IfaceDetector.detect(&engine, &component).await?;

    // Verify imports
    assert_eq!(interfaces.imports.len(), 2);
    assert!(interfaces.imports.iter().any(|i| i
        == &Interface {
            name: "test:math/lib".to_string(),
            funcs: vec!["add".to_string()],
        }));
    assert!(interfaces.imports.iter().any(|i| i
        == &Interface {
            name: "test:string/lib".to_string(),
            funcs: vec!["concat".to_string()],
        }));

    // Verify exports
    assert_eq!(interfaces.exports.len(), 2);
    assert!(interfaces.exports.iter().any(|i| i
        == &Interface {
            name: "test:calc/lib".to_string(),
            funcs: vec!["multiply".to_string()],
        }));
    assert!(interfaces.exports.iter().any(|i| i
        == &Interface {
            name: "test:format/lib".to_string(),
            funcs: vec!["number_to_string".to_string()],
        }));

    Ok(())
}
