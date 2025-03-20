use anyhow::Error;
use icp_core::interface::{DetectIfaces, IfaceDetector, Interface};
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
async fn test_empty_component_detection() -> Result<(), Error> {
    let mut config = Config::new();
    config.wasm_component_model(true).async_support(true);
    let engine = Engine::new(&config)?;

    // Create empty component
    let component = MockComponentBuilder::new_empty_component().build(&engine)?;

    // Test interface detection
    let interfaces = IfaceDetector.detect(&engine, &component).await?;

    // Verify no imports or exports
    assert!(interfaces.imports.is_empty());
    assert!(interfaces.exports.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_many_interfaces_detection() -> Result<(), Error> {
    let mut config = Config::new();
    config.wasm_component_model(true).async_support(true);
    let engine = Engine::new(&config)?;

    // Create component with many interfaces
    let component = MockComponentBuilder::new_many_interfaces().build(&engine)?;

    // Test interface detection
    let interfaces = IfaceDetector.detect(&engine, &component).await?;

    // Verify imports
    assert_eq!(interfaces.imports.len(), 3);

    // Check for math library interface
    let math_lib = interfaces
        .imports
        .iter()
        .find(|i| i.name == "test:math/lib")
        .expect("Math lib not found");
    assert_eq!(math_lib.funcs.len(), 4);
    assert!(math_lib.funcs.contains(&"add".to_string()));
    assert!(math_lib.funcs.contains(&"subtract".to_string()));
    assert!(math_lib.funcs.contains(&"multiply".to_string()));
    assert!(math_lib.funcs.contains(&"divide".to_string()));

    // Check for string library interface
    let string_lib = interfaces
        .imports
        .iter()
        .find(|i| i.name == "test:string/lib")
        .expect("String lib not found");
    assert_eq!(string_lib.funcs.len(), 4);
    assert!(string_lib.funcs.contains(&"concat".to_string()));
    assert!(string_lib.funcs.contains(&"length".to_string()));
    assert!(string_lib.funcs.contains(&"to-upper".to_string()));
    assert!(string_lib.funcs.contains(&"to-lower".to_string()));

    // Check for logger library interface
    let logger_lib = interfaces
        .imports
        .iter()
        .find(|i| i.name == "test:logger/lib")
        .expect("Logger lib not found");
    assert_eq!(logger_lib.funcs.len(), 1);
    assert!(logger_lib.funcs.contains(&"log".to_string()));

    // Verify exports
    assert_eq!(interfaces.exports.len(), 2);

    // Check for math-utils library interface
    let math_utils_lib = interfaces
        .exports
        .iter()
        .find(|i| i.name == "test:math-utils/lib")
        .expect("Math utils lib not found");
    assert_eq!(math_utils_lib.funcs.len(), 2);
    assert!(math_utils_lib.funcs.contains(&"square".to_string()));
    assert!(math_utils_lib.funcs.contains(&"cube".to_string()));

    // Check for number-utils library interface
    let number_utils_lib = interfaces
        .exports
        .iter()
        .find(|i| i.name == "test:number-utils/lib")
        .expect("Number utils lib not found");
    assert_eq!(number_utils_lib.funcs.len(), 2);
    assert!(number_utils_lib.funcs.contains(&"is-even".to_string()));
    assert!(number_utils_lib.funcs.contains(&"is-positive".to_string()));

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
            funcs: vec!["number-to-double".to_string()],
        }));

    Ok(())
}
