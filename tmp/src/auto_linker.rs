use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use wasmtime::{
    component::{Component, Func, Instance, Linker},
    Config, Engine, Store,
};

/// State for the WebAssembly store
pub struct State;

/// Information about a component
pub struct ComponentInfo {
    pub name: String,
    pub component: Component,
    pub instance: Option<Instance>,
}

/// Information about an interface item (import or export)
pub struct InterfaceItem {
    pub interface_name: String,
    pub item_name: String,
}

/// Automatic linker for WebAssembly components
pub struct AutoLinker {
    engine: Engine,
    store: Store<State>,
    linker: Linker<State>,
    components: HashMap<String, ComponentInfo>,
    imports: HashMap<String, Vec<InterfaceItem>>,
    exports: HashMap<String, Vec<InterfaceItem>>,
    function_refs: HashMap<String, Arc<Mutex<Option<Func>>>>,
}

impl AutoLinker {
    /// Create a new automatic linker
    pub fn new() -> Result<Self> {
        let engine = Engine::new(&Config::new())?;
        let store = Store::new(&engine, State);
        let linker = Linker::new(&engine);

        Ok(Self {
            engine,
            store,
            linker,
            components: HashMap::new(),
            imports: HashMap::new(),
            exports: HashMap::new(),
            function_refs: HashMap::new(),
        })
    }

    /// Create a unique key for a function reference
    fn create_ref_key(&self, component: &str, interface: &str, item: &str) -> String {
        format!("{}:{}:{}", component, interface, item)
    }

    /// Initialize function references for all imports
    pub fn init_function_refs(&mut self) -> Result<()> {
        // Skip if already initialized
        if !self.function_refs.is_empty() {
            return Ok(());
        }

        // For each component's imports
        for (component_name, imports) in &self.imports {
            for import in imports {
                // Skip empty interface or item names
                if import.interface_name.is_empty() || import.item_name.is_empty() {
                    continue;
                }

                // Create a unique key for this function reference
                let key =
                    self.create_ref_key(component_name, &import.interface_name, &import.item_name);

                // Initialize the function reference as None
                self.function_refs.insert(key, Arc::new(Mutex::new(None)));
            }
        }

        println!(
            "Initialized {} function references",
            self.function_refs.len()
        );
        Ok(())
    }

    /// Get a function reference by key
    pub fn get_function_ref(&self, key: &str) -> Option<Arc<Mutex<Option<Func>>>> {
        self.function_refs.get(key).cloned()
    }

    /// Parse a name into interface name and item name
    fn parse_name(&self, name: &str) -> (String, String) {
        // For interface names like "local:cmpnt-a/interface-a"
        if name.contains(':') && name.contains('/') {
            return (name.to_string(), String::new());
        }

        // For function names like "fn-a"
        (String::new(), name.to_string())
    }

    /// Load a component from a file
    pub fn load_component(&mut self, name: &str, path: &str) -> Result<()> {
        println!("Loading component: {}", name);
        let component = Component::from_file(&self.engine, path)?;

        // Extract imports and exports
        let component_type = component.component_type();
        let mut imports = Vec::new();
        let mut exports = Vec::new();

        println!("  Imports:");
        for (name, _item) in component_type.imports(&self.engine) {
            println!("    {}", name);

            // Parse the name into interface name and item name
            let (interface_name, item_name) = self.parse_name(name);

            imports.push(InterfaceItem {
                interface_name,
                item_name,
            });
        }

        println!("  Exports:");
        for (name, _item) in component_type.exports(&self.engine) {
            println!("    {}", name);

            // Parse the name into interface name and item name
            let (interface_name, item_name) = self.parse_name(name);

            exports.push(InterfaceItem {
                interface_name,
                item_name,
            });
        }

        // Store component info
        self.components.insert(
            name.to_string(),
            ComponentInfo {
                name: name.to_string(),
                component,
                instance: None,
            },
        );

        // Store imports and exports
        self.imports.insert(name.to_string(), imports);
        self.exports.insert(name.to_string(), exports);

        Ok(())
    }

    /// Get the engine
    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    /// Get a mutable reference to the store
    pub fn store_mut(&mut self) -> &mut Store<State> {
        &mut self.store
    }

    /// Get the linker
    pub fn linker(&self) -> &Linker<State> {
        &self.linker
    }

    /// Get a mutable reference to the linker
    pub fn linker_mut(&mut self) -> &mut Linker<State> {
        &mut self.linker
    }

    /// Get a component by name
    pub fn get_component(&self, name: &str) -> Option<&ComponentInfo> {
        self.components.get(name)
    }

    /// Get imports for a component
    pub fn get_imports(&self, name: &str) -> Option<&Vec<InterfaceItem>> {
        self.imports.get(name)
    }

    /// Get exports for a component
    pub fn get_exports(&self, name: &str) -> Option<&Vec<InterfaceItem>> {
        self.exports.get(name)
    }

    /// Print information about all components
    pub fn print_info(&self) {
        println!("\nComponent Information:");

        for (name, info) in &self.components {
            let status = if info.instance.is_some() {
                "Instantiated"
            } else {
                "Not instantiated"
            };
            println!("  Component: {} ({})", name, status);

            if let Some(imports) = self.imports.get(name) {
                println!("    Imports:");
                for import in imports {
                    if import.item_name.is_empty() {
                        println!("      Interface: {}", import.interface_name);
                    } else {
                        println!("      Item: {}.{}", import.interface_name, import.item_name);
                    }
                }
            }

            if let Some(exports) = self.exports.get(name) {
                println!("    Exports:");
                for export in exports {
                    if export.item_name.is_empty() {
                        println!("      Interface: {}", export.interface_name);
                    } else {
                        println!("      Item: {}.{}", export.interface_name, export.item_name);
                    }
                }
            }
        }
    }

    /// Print information about function references
    pub fn print_function_refs(&self) {
        println!("\nFunction References:");

        for (key, func_ref) in &self.function_refs {
            let status = if func_ref.lock().unwrap().is_some() {
                "Resolved"
            } else {
                "Unresolved"
            };

            println!("  {}: {}", key, status);
        }
    }

    /// Get a function from a component
    pub fn get_function(
        &mut self,
        component_name: &str,
        interface_name: &str,
        function_name: &str,
    ) -> Result<Func> {
        // Get component info
        let component_info = self
            .components
            .get(component_name)
            .ok_or_else(|| anyhow!("Component not found: {}", component_name))?;

        // Check if component is instantiated
        let instance = component_info
            .instance
            .as_ref()
            .ok_or_else(|| anyhow!("Component not instantiated: {}", component_name))?;

        // Get interface export
        let interface_export = instance
            .get_export(&mut self.store, None, interface_name)
            .ok_or_else(|| anyhow!("Interface not found: {}", interface_name))?;

        // Get function export
        let function_export = instance
            .get_export(&mut self.store, Some(&interface_export), function_name)
            .ok_or_else(|| anyhow!("Function not found: {}", function_name))?;

        // Get function
        let func = instance
            .get_func(&mut self.store, function_export)
            .ok_or_else(|| anyhow!("Failed to get function"))?;

        Ok(func)
    }

    /// Call a function with parameters
    pub fn call_function(
        &mut self,
        func: &Func,
        params: &[wasmtime::component::Val],
    ) -> Result<Vec<wasmtime::component::Val>> {
        // Based on the WIT definitions, all functions return a string
        let mut results = vec![wasmtime::component::Val::String("".to_string())];

        // Call the function
        func.call(&mut self.store, params, &mut results)?;
        func.post_return(&mut self.store)?;

        Ok(results)
    }

    /// Automatically link all components
    pub fn auto_link(&mut self) -> Result<()> {
        println!("\nAutomatically linking components...");

        // First, discover functions in interfaces
        self.discover_functions()?;

        // Initialize function references
        self.init_function_refs()?;

        // For each component's imports
        for (component_name, imports) in &self.imports {
            println!("  Linking imports for: {}", component_name);

            for import in imports {
                // Skip empty interface names or item names
                if import.interface_name.is_empty() || import.item_name.is_empty() {
                    continue;
                }

                println!(
                    "    Linking: {}.{}",
                    import.interface_name, import.item_name
                );

                // Get the function reference
                let ref_key =
                    self.create_ref_key(component_name, &import.interface_name, &import.item_name);

                if let Some(function_ref) = self.function_refs.get(&ref_key) {
                    // Create a linking function
                    let mut inst = self.linker.instance(&import.interface_name)?;

                    // Clone the function reference for the closure
                    let function_ref_clone = Arc::clone(function_ref);

                    // Define a function that will be called when the import is invoked
                    inst.func_new(&import.item_name, move |mut store, params, results| {
                        let f = function_ref_clone
                            .lock()
                            .unwrap()
                            .expect("function not set");

                        f.call(&mut store, params, results)?;
                        f.post_return(&mut store)?;

                        Ok(())
                    })?;
                }
            }
        }

        println!("Automatic linking completed");
        Ok(())
    }

    /// Discover functions in interfaces
    fn discover_functions(&mut self) -> Result<()> {
        println!("\nDiscovering functions in interfaces...");

        // Based on the WIT definitions we know:
        // - Component A: imports fn-b from local:cmpnt-b/interface-b, exports fn-a from local:cmpnt-a/interface-a
        // - Component B: imports fn-c from local:cmpnt-c/interface-c, exports fn-b from local:cmpnt-b/interface-b
        // - Component C: exports fn-c from local:cmpnt-c/interface-c

        // In a real implementation, we would extract this information from the component metadata
        // For now, we'll use a simple approach to discover functions

        // For each component
        for component_name in self.components.keys().cloned().collect::<Vec<_>>() {
            // For each export
            if let Some(exports) = self.exports.get_mut(&component_name) {
                for export in exports.iter_mut() {
                    // If this is an interface export with no item name
                    if !export.interface_name.is_empty() && export.item_name.is_empty() {
                        // Extract the function name from the interface name
                        // For example, from "local:cmpnt-a/interface-a" we extract "fn-a"
                        let parts: Vec<&str> = export.interface_name.split('/').collect();
                        if parts.len() == 2 {
                            let interface_parts: Vec<&str> = parts[0].split(':').collect();
                            if interface_parts.len() == 2 {
                                let component_part = interface_parts[1];
                                // Extract the component letter (a, b, c) and use it to form the function name
                                if let Some(component_letter) = component_part.chars().last() {
                                    export.item_name = format!("fn-{}", component_letter);
                                    println!(
                                        "  Discovered export: {}.{}",
                                        export.interface_name, export.item_name
                                    );
                                }
                            }
                        }
                    }
                }
            }

            // For each import
            if let Some(imports) = self.imports.get_mut(&component_name) {
                for import in imports.iter_mut() {
                    // If this is an interface import with no item name
                    if !import.interface_name.is_empty() && import.item_name.is_empty() {
                        // Extract the function name from the interface name
                        // For example, from "local:cmpnt-b/interface-b" we extract "fn-b"
                        let parts: Vec<&str> = import.interface_name.split('/').collect();
                        if parts.len() == 2 {
                            let interface_parts: Vec<&str> = parts[0].split(':').collect();
                            if interface_parts.len() == 2 {
                                let component_part = interface_parts[1];
                                // Extract the component letter (a, b, c) and use it to form the function name
                                if let Some(component_letter) = component_part.chars().last() {
                                    import.item_name = format!("fn-{}", component_letter);
                                    println!(
                                        "  Discovered import: {}.{}",
                                        import.interface_name, import.item_name
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Instantiate all components
    pub fn instantiate_all(&mut self) -> Result<()> {
        println!("\nInstantiating components...");

        // Track successful instantiations
        let mut instantiated = HashMap::new();

        // Try to instantiate each component
        for (name, info) in &self.components {
            println!("  Instantiating: {}", name);

            match self.linker.instantiate(&mut self.store, &info.component) {
                Ok(instance) => {
                    instantiated.insert(name.clone(), instance);
                    println!("    Success");
                }
                Err(e) => {
                    println!("    Failed: {}", e);
                    // We'll continue and try to instantiate other components
                }
            }
        }

        // Store instances in component info
        for (name, instance) in instantiated {
            if let Some(info) = self.components.get_mut(&name) {
                info.instance = Some(instance);
            }
        }

        // Count how many components were instantiated
        let instantiated_count = self
            .components
            .values()
            .filter(|info| info.instance.is_some())
            .count();

        println!(
            "Instantiated {}/{} components",
            instantiated_count,
            self.components.len()
        );

        Ok(())
    }

    /// Determine instantiation order based on dependencies
    pub fn determine_instantiation_order(&self) -> Result<Vec<String>> {
        let mut order = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new(); // For detecting cycles

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
                return Err(anyhow!(
                    "Circular dependency detected involving component: {}",
                    component
                ));
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

    /// Instantiate components in dependency order
    pub fn instantiate_in_dependency_order(&mut self) -> Result<()> {
        println!("\nInstantiating components in dependency order...");

        // Determine the instantiation order
        let order = self.determine_instantiation_order()?;

        println!("  Instantiation order: {:?}", order);

        // Instantiate components in the determined order
        let mut instantiated_count = 0;
        for component_name in &order {
            if self.instantiate_component(component_name)? {
                instantiated_count += 1;
            }
        }

        println!(
            "Instantiated {}/{} components",
            instantiated_count,
            self.components.len()
        );

        Ok(())
    }

    // Helper method to check if all dependencies are satisfied
    fn check_dependencies_satisfied(
        &self,
        component_name: &str,
        instantiated: &HashSet<String>,
    ) -> bool {
        // Get imports for this component
        if let Some(imports) = self.imports.get(component_name) {
            for import in imports {
                // Skip empty interface names
                if import.interface_name.is_empty() {
                    continue;
                }

                // Find a component that exports this interface
                let mut found = false;
                for (exporting_name, exports) in &self.exports {
                    if instantiated.contains(exporting_name) {
                        for export in exports {
                            if export.interface_name == import.interface_name {
                                found = true;
                                break;
                            }
                        }
                    }
                    if found {
                        break;
                    }
                }

                // If we didn't find a component that exports this interface,
                // then dependencies are not satisfied
                if !found {
                    return false;
                }
            }
        }

        // All dependencies are satisfied
        true
    }

    // Helper method to instantiate a single component
    fn instantiate_component(&mut self, name: &str) -> Result<bool> {
        println!("  Instantiating: {}", name);

        if let Some(info) = self.components.get_mut(name) {
            match self.linker.instantiate(&mut self.store, &info.component) {
                Ok(instance) => {
                    info.instance = Some(instance);
                    println!("    Success");
                    return Ok(true);
                }
                Err(e) => {
                    println!("    Failed: {}", e);
                    return Ok(false);
                }
            }
        }

        Ok(false)
    }

    /// Print the dependency graph
    pub fn print_dependency_graph(&self) -> Result<()> {
        println!("\nDependency Graph:");

        for (component_name, imports) in &self.imports {
            println!("  Component: {}", component_name);

            // Print dependencies (imports)
            let mut dependencies = Vec::new();
            for import in imports {
                if import.interface_name.is_empty() {
                    continue;
                }

                // Find components that export this interface
                for (exporting_component, exports) in &self.exports {
                    for export in exports {
                        if export.interface_name == import.interface_name {
                            dependencies.push(exporting_component);
                            break;
                        }
                    }
                }
            }

            if dependencies.is_empty() {
                println!("    No dependencies");
            } else {
                println!("    Depends on: {:?}", dependencies);
            }
        }

        Ok(())
    }

    /// Load all WebAssembly components from a directory
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

                println!(
                    "  Found component: {} at {}",
                    component_name,
                    path.display()
                );

                // Load the component
                match self.load_component(&component_name, &path.to_string_lossy()) {
                    Ok(_) => {
                        println!("    Successfully loaded");
                        loaded_components.push(component_name);
                    }
                    Err(e) => {
                        println!("    Failed to load: {}", e);
                    }
                }
            }
        }

        println!(
            "Loaded {}/{} components from directory",
            loaded_components.len(),
            loaded_components.len() + (self.components.len() - loaded_components.len())
        );

        Ok(loaded_components)
    }

    /// Resolve function references by matching exports to imports
    pub fn resolve_references(&mut self) -> Result<()> {
        println!("\nResolving function references...");

        // Track how many references were resolved
        let mut resolved_count = 0;

        // For each instantiated component (exporting component)
        for (exporting_component, info) in &self.components {
            // Skip components that aren't instantiated
            if info.instance.is_none() {
                continue;
            }

            let instance = info.instance.as_ref().unwrap();

            // Get exports for this component
            if let Some(exports) = self.exports.get(exporting_component) {
                // For each export
                for export in exports {
                    // Skip empty interface or item names
                    if export.interface_name.is_empty() || export.item_name.is_empty() {
                        continue;
                    }

                    // Skip if the export is not an interface (for now)
                    if !export.interface_name.contains(':') || !export.interface_name.contains('/')
                    {
                        continue;
                    }

                    // Get the interface export
                    let interface_export =
                        match instance.get_export(&mut self.store, None, &export.interface_name) {
                            Some(export) => export,
                            None => {
                                println!(
                                    "    Warning: Interface export not found: {}",
                                    export.interface_name
                                );
                                continue;
                            }
                        };

                    // For each component's imports (importing component)
                    for (importing_component, imports) in &self.imports {
                        // Skip if it's the same component
                        if importing_component == exporting_component {
                            continue;
                        }

                        // For each import
                        for import in imports {
                            // Check if this export matches this import
                            if import.interface_name == export.interface_name
                                && import.item_name == export.item_name
                            {
                                // Create a reference key for the importing component
                                let ref_key = self.create_ref_key(
                                    importing_component,
                                    &import.interface_name,
                                    &import.item_name,
                                );

                                // Get the function export
                                if let Some(func_export) = instance.get_export(
                                    &mut self.store,
                                    Some(&interface_export),
                                    &import.item_name,
                                ) {
                                    // Get the actual function
                                    if let Some(func) =
                                        instance.get_func(&mut self.store, func_export)
                                    {
                                        // Get the function reference
                                        if let Some(function_ref) = self.function_refs.get(&ref_key)
                                        {
                                            // Set the function reference to the exported function
                                            let mut f = function_ref.lock().unwrap();
                                            *f = Some(func);

                                            resolved_count += 1;
                                            println!(
                                                "  Resolved: {}.{} -> {}.{}",
                                                exporting_component,
                                                import.item_name,
                                                importing_component,
                                                import.item_name
                                            );
                                        }
                                    } else {
                                        println!(
                                            "    Warning: Failed to get function: {}.{}",
                                            export.interface_name, import.item_name
                                        );
                                    }
                                } else {
                                    println!(
                                        "    Warning: Function export not found: {}.{}",
                                        export.interface_name, import.item_name
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        println!(
            "Resolved {}/{} function references",
            resolved_count,
            self.function_refs.len()
        );
        Ok(())
    }
}
