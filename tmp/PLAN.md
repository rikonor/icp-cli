# Automatic WebAssembly Component Linking - Implementation Plan

## Description

This plan outlines a step-by-step approach to transform the current manual WebAssembly component linking code into an automatic system. Each step introduces a small, testable change that builds toward the final solution.

## General Guidelines

Adherence to these guidelines SHOULD BE MAINTAINED AT ALL TIMES.

- Each change should be implemented in a simple minimal fashion.
- After being implemented, a brief description of the change should be given.
- After being implemented, testing instructions should be provided to the user.
- [PLEASE-DONT-FORGET] Once the user approves of the change, it should be committed via git with a descriptive git commit message.
- this PLAN.md file should then be updated to indicate what should be worked on next.
- Each design and coding session should focus on one task at a time. This is to reduce OpenRouter credit costs incurred from the session.
- A JOURNAL.md file should be maintained, describing the work done in each session.
- Assuming you deem it needed, the `Description` section of the PLAN.md file should be updated on an ongoing basis to ensure it provides a clear description of the project.
- You should use `--release` when running code, as it will lead to much quicker component load times.
- [PLEASE-DONT-FORGET] Consider reviewing the JOURNAL.md file for an explanation of previous work and possible learnings.
- [PLEASE-DONT-FORGET] Consider suggesting improvements to your working process.

Current Focus: `All steps completed!`.

## Phase 1: Component Information Extraction

### Step 1: Create Basic Data Structures ✅

- ✅ Add structures to represent component information
- ✅ Implement functions to extract import/export information
- ✅ Test by printing component information

```rust
// Added these structures
struct ComponentInfo {
    name: String,
    component: Component,
    instance: Option<Instance>,
}

struct InterfaceItem {
    interface_name: String,
    item_name: String,
}
```

**Testing**: Run the program and verify it correctly prints all imports and exports for each component.

```bash
cargo run --release
```

**Implementation Notes**:

- Created `auto_linker.rs` module with `AutoLinker` struct to manage components
- Implemented functions to load components and extract import/export information
- Modified `main.rs` to use the new `AutoLinker` implementation
- Successfully tested with `cargo run --release`

### Step 2: Implement Name Parsing ✅

- ✅ Add function to parse interface and item names
- ✅ Extract interface names and function names from imports/exports
- ✅ Store parsed information in data structures

```rust
fn parse_name(name: &str) -> Option<(String, String)> {
    // For names like "local:cmpnt-a/interface-a", this is an interface
    if name.contains(':') && name.contains('/') {
        return Some((name.to_string(), String::new()));
    }

    // For names like "fn-a", this is an item
    Some((String::new(), name.to_string()))
}
```

**Testing**: Verify the program correctly parses and categorizes interface and function names.

```bash
cargo run --release
```

**Implementation Notes**:

- Added `parse_name` function to extract interface names and item names from import/export names
- Updated `load_component` method to use the `parse_name` function for imports and exports
- Enhanced `print_info` method to display both interface names and item names
- Successfully tested with `cargo run --release` and verified correct parsing

## Phase 2: Automatic Function Reference Management

### Step 3: Create Function Reference Registry ✅

- ✅ Implement a registry to track function references
- ✅ Create unique keys for each function reference
- ✅ Initialize function references for all imports

```rust
// Create a function reference key
fn create_ref_key(&self, component: &str, interface: &str, item: &str) -> String {
    format!("{}:{}:{}", component, interface, item)
}
```

**Testing**: Check that function references are correctly created for all imports.

**Implementation Notes**:

- Added `function_refs: HashMap<String, Arc<Mutex<Option<Func>>>>` to the `AutoLinker` struct
- Implemented `create_ref_key` method to generate unique keys for function references
- Created `init_function_refs` method to initialize function references for all imports
- Updated `parse_name` method to handle WebAssembly component model naming conventions
- Added `print_function_refs` method to display function reference information
- Successfully tested with `cargo run --release` and verified function references are created

### Step 4: Implement Automatic Linking

- Create linking functions for all imports
- Connect linking functions to function references
- Replace hardcoded linking with dynamic linking

```rust
// Create linking functions for all imports
for (component_name, imports) in &self.imports {
    for import in imports {
        // Skip empty interface names
        if import.interface_name.is_empty() {
            continue;
        }

        // Get the function reference
        let ref_key = self.create_ref_key(
            component_name,
            &import.interface_name,
            &import.item_name,
        );

        let function_ref = self.function_refs.get(&ref_key).cloned()?;

        // Create a linking function
        let mut inst = self.linker.instance(&import.interface_name)?;

        let function_ref_clone = Arc::clone(&function_ref);
        inst.func_new(&import.item_name, move |mut store, params, results| {
            let f = function_ref_clone
                .lock()
                .unwrap()
                .as_ref()
                .ok_or_else(|| anyhow!("Function not set"))?;

            f.call(&mut store, params, results)?;
            f.post_return(&mut store)?;

            Ok(())
        })?;
    }
}
```

**Testing**: Run the program and verify it correctly links all components without errors.

## Phase 3: Component Instantiation and Resolution

### Step 5: Implement Component Instantiation

- Instantiate all components in the correct order
- Store instances in component info
- Handle instantiation errors gracefully

```rust
// Instantiate all components
let mut instances = HashMap::new();

for (name, info) in &self.components {
    println!("  Instantiating: {}", name);
    let instance = self.linker.instantiate(&mut self.store, &info.component)?;
    instances.insert(name.clone(), instance);
}

// Store instances
for (name, instance) in instances {
    if let Some(info) = self.components.get_mut(&name) {
        info.instance = Some(instance);
    }
}
```

**Testing**: Verify all components are instantiated correctly.

### Step 6: Implement Function Reference Resolution

- Resolve function references after instantiation
- Match exports to imports automatically
- Set function references to actual functions

```rust
// For each export that matches an import
if import.interface_name == export.interface_name &&
   import.item_name == inner_export.item_name {
    // Set the function reference
    let ref_key = self.create_ref_key(
        importing_component,
        &import.interface_name,
        &import.item_name,
    );

    if let Some(function_ref) = self.function_refs.get(&ref_key) {
        let mut f = function_ref.lock().unwrap();
        *f = Some(func.clone());
        println!(
            "  Resolved: {}.{} -> {}.{}",
            exporting_component, inner_export.item_name,
            importing_component, import.item_name
        );
    }
}
```

**Note**: This implementation explicitly disallows circular dependencies between components. If a circular dependency is detected, an error will be returned. Handling circular dependencies would require a more complex implementation involving partial instantiation or lazy binding, which is beyond the scope of this project.

**Testing**: Run the program and verify it correctly resolves all function references.

## Phase 4: API and Integration

### Step 7: Create Clean API ✅

- ✅ Implement function to get a function from a component
- ✅ Implement function to call a function with parameters
- ✅ Create a clean API for the automatic linker

```rust
// Get a function from a component
fn get_function(
    &mut self,
    component_name: &str,
    interface_name: &str,
    function_name: &str,
) -> Result<Func> {
    let component_info = self.components
        .get(component_name)
        .ok_or_else(|| anyhow!("Component not found: {}", component_name))?;

    let instance = component_info
        .instance
        .as_ref()
        .ok_or_else(|| anyhow!("Component not instantiated: {}", component_name))?;

    // Get the interface export
    let interface_export = instance
        .get_export(&mut self.store, None, interface_name)
        .ok_or_else(|| anyhow!("Interface not found: {}", interface_name))?;

    // Get the function export
    let function_export = instance
        .get_export(&mut self.store, Some(&interface_export), function_name)
        .ok_or_else(|| anyhow!("Function not found: {}", function_name))?;

    // Get the function
    let func = instance
        .get_func(&mut self.store, function_export)
        .ok_or_else(|| anyhow!("Failed to get function"))?;

    Ok(func)
}

// Call a function
fn call_function(&mut self, func: &Func, params: &[Val]) -> Result<Vec<Val>> {
    let mut results = vec![Val::String("".to_string())];

    func.call(&mut self.store, params, &mut results)?;
    func.post_return(&mut self.store)?;

    Ok(results)
}
```

**Testing**: Verified the API works correctly by calling functions from components. Successfully called functions with different parameters and received correct results.

**Implementation Notes**:

- Added `get_function` method to retrieve a function from a component
- Added `call_function` method to call a function with parameters
- Added `discover_functions` method to automatically extract function names from interfaces
- Improved the `auto_link` method to discover functions and initialize function references
- Updated `main.rs` to use the new API
- Successfully tested with `cargo run --release`

### Step 8: Refactor Main Function ✅

- ✅ Replace manual linking with automatic linking
- ✅ Clean up the main function
- ✅ Ensure the same functionality is preserved

```rust
// Create auto linker
let mut auto_linker = AutoLinker::new()?;

// Load components
auto_linker.load_component(
    "cmpnt-a",
    "/Users/orricon/workspace/playgrounds/component-model-poc/inter-cmpnt/src/cmpnt-a/out/cmpnt-a.component.wasm",
)?;

auto_linker.load_component(
    "cmpnt-b",
    "/Users/orricon/workspace/playgrounds/component-model-poc/inter-cmpnt/src/cmpnt-b/out/cmpnt-b.component.wasm",
)?;

auto_linker.load_component(
    "cmpnt-c",
    "/Users/orricon/workspace/playgrounds/component-model-poc/inter-cmpnt/src/cmpnt-c/out/cmpnt-c.component.wasm",
)?;

// Automatically link components
auto_linker.auto_link()?;

// Instantiate components
auto_linker.instantiate_all()?;

// Resolve function references
auto_linker.resolve_references()?;

// Get function from component A
let fn_a = auto_linker.get_function("cmpnt-a", "local:cmpnt-a/interface-a", "fn-a")?;

// Call function (1)
let result1 = auto_linker.call_function(&fn_a, &[Val::String("host-1".to_string())])?;
println!("\nResult 1: {:?}", result1);

// Call function (2)
let result2 = auto_linker.call_function(&fn_a, &[Val::String("host-2".to_string())])?;
println!("Result 2: {:?}", result2);
```

**Testing**: Run the program and verify it produces the same results as the original code.

**Implementation Notes**:

- Simplified the `main.rs` file by removing redundant debugging steps
- Removed calls to `print_info()` and `print_function_refs()`
- Removed the second call to `instantiate_all()`
- Focused on the essential operations for component linking and instantiation
- Successfully tested with `cargo run --release` and verified the same results are produced

## Phase 5: Extensions and Improvements

### Step 9: Add Directory Loading ✅

- ✅ Implement function to load components from a directory
- ✅ Automatically discover and load .wasm files
- ✅ Extract component names from filenames

```rust
// Load components from a directory
pub fn load_components_from_directory(&mut self, dir_path: &str) -> Result<Vec<String>> {
    println!("\nLoading components from directory: {}", dir_path);

    let path = Path::new(dir_path);
    if !path.is_dir() {
        return Err(anyhow!("Not a directory: {}", dir_path));
    }

    let mut loaded_components = Vec::new();

    // Read directory entries
    let entries = std::fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip if not a file or doesn't have .wasm extension
        if !path.is_file() || path.extension().map_or(false, |ext| ext != "wasm") {
            continue;
        }

        // Extract component name from filename (without extension)
        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
            let component_name = file_stem.to_string();

            // If the filename ends with .component.wasm, remove the .component part too
            let component_name = if component_name.ends_with(".component") {
                component_name.trim_end_matches(".component").to_string()
            } else {
                component_name
            };

            // Load the component
            match self.load_component(&component_name, &path.to_string_lossy()) {
                Ok(_) => {
                    loaded_components.push(component_name);
                }
                Err(e) => {
                    println!("    Failed to load: {}", e);
                }
            }
        }
    }

    Ok(loaded_components)
}
```

**Testing**: Created a test directory with multiple .wasm files and verified they are loaded correctly.

**Implementation Notes**:

- Added `load_components_from_directory` method to the `AutoLinker` struct
- Implemented directory scanning to find .wasm files
- Added special handling for filenames ending with .component.wasm
- Added error handling for invalid directories and file loading failures
- Updated `main.rs` to use the new method with a test directory
- Added helper function to copy component files to the test directory
- Successfully tested with `cargo run --release`

### Step 10: Enhance Dependency Resolution ✅

- ✅ Extract dependency analysis into a separate method
- ✅ Return the instantiation order without performing instantiation
- ✅ Improve detection and handling of circular dependencies
- ✅ Add better error reporting for dependency issues

```rust
// Determine instantiation order based on dependencies
pub fn determine_instantiation_order(&self) -> Result<Vec<String>> {
    let mut order = Vec::new();
    let mut visited = HashSet::new();
    let mut visiting = HashSet::new();  // For detecting cycles

    // Helper function for depth-first traversal with cycle detection
    fn visit(
        component: &str,
        order: &mut Vec<String>,
        visited: &mut HashSet<String>,
        visiting: &mut HashSet<String>,
        imports: &HashMap<String, Vec<InterfaceItem>>,
        exports: &HashMap<String, Vec<InterfaceItem>>,
    ) -> Result<()> {
        if visited.contains(component) {
            return Ok(());
        }

        if visiting.contains(component) {
            return Err(anyhow!("Circular dependency detected involving component: {}", component));
        }

        visiting.insert(component.to_string());

        // Visit dependencies first
        if let Some(component_imports) = imports.get(component) {
            for import in component_imports {
                // Skip empty interface names
                if import.interface_name.is_empty() {
                    continue;
                }

                // Find components that export this interface
                let mut found = false;
                for (exporting_component, component_exports) in exports {
                    for export in component_exports {
                        if export.interface_name == import.interface_name {
                            visit(
                                exporting_component,
                                order,
                                visited,
                                visiting,
                                imports,
                                exports,
                            )?;
                            found = true;
                            break;
                        }
                    }
                    if found {
                        break;
                    }
                }

                if !found {
                    return Err(anyhow!(
                        "Unsatisfied dependency: component {} requires interface {}, but no component exports it",
                        component,
                        import.interface_name
                    ));
                }
            }
        }

        // Add this component to the order
        order.push(component.to_string());
        visited.insert(component.to_string());
        visiting.remove(component);

        Ok(())
    }

    // Visit all components
    for component in self.components.keys() {
        if !visited.contains(component) {
            visit(
                component,
                &mut order,
                &mut visited,
                &mut visiting,
                &self.imports,
                &self.exports,
            )?;
        }
    }

    Ok(order)
}
```

**Testing**: Ran the program with `cargo run --release` and verified that the method correctly determines the instantiation order. The program successfully prints the dependency graph and instantiation order before instantiating the components.

**Implementation Notes**:

- Added `determine_instantiation_order` method to analyze dependencies and determine instantiation order
- Implemented depth-first traversal with cycle detection to handle dependencies correctly
- Added explicit error reporting for circular dependencies and missing dependencies
- Added `print_dependency_graph` method to visualize the dependency relationships
- Updated `instantiate_in_dependency_order` to use the new method
- Updated `main.rs` to print the dependency graph and instantiation order
- Successfully tested with `cargo run --release`

## Final Testing

For each step, run the program with:

```bash
cargo run
```

Verify the output matches expectations and all components are linked and function correctly.

The final program should:

1. Automatically discover and load components
2. Analyze imports and exports
3. Create function references for all imports
4. Link components automatically
5. Instantiate components in the correct order
6. Resolve function references
7. Allow calling functions from any component

This approach ensures each change is small, testable, and builds toward the final solution.
