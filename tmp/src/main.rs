use anyhow::{Error, Result};
use std::fs;
use tempfile::TempDir;

mod auto_linker;
use auto_linker::AutoLinker;
use wasmtime::component::Val;

// Helper function to copy a component file to the test directory
fn copy_component_to_test_dir(
    src_path: &str,
    test_dir: &str,
    component_name: &str,
) -> Result<String> {
    let dest_path = format!("{}/{}.component.wasm", test_dir, component_name);
    fs::copy(src_path, &dest_path)?;
    Ok(dest_path)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create auto linker
    let mut auto_linker = AutoLinker::new()?;

    // Create a temporary directory for components
    // This will be automatically cleaned up when it goes out of scope
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path().to_str().unwrap();

    // Copy component files to the test directory
    println!("Copying component files to test directory: {}", test_dir);
    copy_component_to_test_dir(
        "/Users/orricon/workspace/playgrounds/component-model-poc/inter-cmpnt/src/cmpnt-a/out/cmpnt-a.component.wasm",
        test_dir,
        "cmpnt-a",
    )?;

    copy_component_to_test_dir(
        "/Users/orricon/workspace/playgrounds/component-model-poc/inter-cmpnt/src/cmpnt-b/out/cmpnt-b.component.wasm",
        test_dir,
        "cmpnt-b",
    )?;

    copy_component_to_test_dir(
        "/Users/orricon/workspace/playgrounds/component-model-poc/inter-cmpnt/src/cmpnt-c/out/cmpnt-c.component.wasm",
        test_dir,
        "cmpnt-c",
    )?;

    // Load components from the test directory
    let loaded_components = auto_linker.load_components_from_directory(test_dir)?;
    println!("Loaded components: {:?}", loaded_components);

    // Automatically link components
    auto_linker.auto_link()?;

    // Print the dependency graph
    auto_linker.print_dependency_graph()?;

    // Determine instantiation order
    let order = auto_linker.determine_instantiation_order()?;
    println!("Instantiation order: {:?}", order);

    // Instantiate components in dependency order
    auto_linker.instantiate_in_dependency_order()?;

    // Resolve function references
    auto_linker.resolve_references()?;

    // Get function from component A
    let fn_a = auto_linker.get_function("cmpnt-a", "local:cmpnt-a/interface-a", "fn-a")?;

    // Call function with different parameters
    let result1 = auto_linker.call_function(&fn_a, &[Val::String("host-1".to_string())])?;
    println!("Result 1: {:?}", result1);

    let result2 = auto_linker.call_function(&fn_a, &[Val::String("host-2".to_string())])?;
    println!("Result 2: {:?}", result2);

    println!("\nPhase 5 / Step 10 completed: Enhanced Dependency Resolution");
    println!("All steps completed!");

    Ok(())
}
