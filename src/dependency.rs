use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::Context;
use thiserror::Error;

use crate::manifest::{Extension, Manifest};

/// Errors that can occur during dependency resolution
#[derive(Debug, Error)]
pub enum DependencyError {
    /// A circular dependency was detected
    #[error("circular dependency detected: {0}")]
    CircularDependency(String),

    /// A required interface is missing
    #[error("extension '{importer}' imports interface '{interface}' which is not exported by any installed extension")]
    MissingInterface {
        /// Name of the extension that imports the interface
        importer: String,
        /// Name of the interface that is missing
        interface: String,
    },

    /// A function is missing from an interface
    #[error("extension '{importer}' imports function '{function}' from interface '{interface}', but it is not exported by '{exporter}'")]
    MissingFunction {
        /// Name of the extension that imports the function
        importer: String,
        /// Name of the interface that should contain the function
        interface: String,
        /// Name of the function that is missing
        function: String,
        /// Name of the extension that exports the interface
        exporter: String,
    },

    /// An unexpected error occurred
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

/// A graph representing dependencies between extensions
pub struct DependencyGraph {
    /// Map from extension name to the names of extensions it depends on
    dependencies: HashMap<String, Vec<String>>,
    /// Map from extension name to the names of extensions that depend on it
    dependents: HashMap<String, Vec<String>>,
    /// Map from interface name to the extension that exports it
    interface_providers: HashMap<String, String>,
    /// Map from extension name to the interfaces it exports
    exported_interfaces: HashMap<String, Vec<String>>,
    /// Map from extension name to the interfaces it imports
    imported_interfaces: HashMap<String, Vec<String>>,
    /// Map from interface name to the functions it provides
    interface_functions: HashMap<String, HashSet<String>>,
    /// All extension names in the graph
    extension_names: Vec<String>,
    /// Detected cycles in the dependency graph
    cycles: Vec<Vec<String>>,
}

impl DependencyGraph {
    /// Creates a new dependency graph from a manifest
    pub fn new(manifest: &Manifest) -> Result<Self, DependencyError> {
        let mut graph = Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            interface_providers: HashMap::new(),
            exported_interfaces: HashMap::new(),
            imported_interfaces: HashMap::new(),
            interface_functions: HashMap::new(),
            extension_names: Vec::new(),
            cycles: Vec::new(),
        };

        graph.build_graph(manifest)?;
        graph.detect_cycles();

        Ok(graph)
    }

    /// Builds the dependency graph from a manifest
    fn build_graph(&mut self, manifest: &Manifest) -> Result<(), DependencyError> {
        // Initialize collections
        for extension in &manifest.xs {
            self.extension_names.push(extension.name.clone());
            self.dependencies.insert(extension.name.clone(), Vec::new());
            self.dependents.insert(extension.name.clone(), Vec::new());

            // Track exported interfaces
            let mut exported = Vec::new();
            for interface in &extension.exported_interfaces {
                exported.push(interface.name.clone());
                self.interface_providers
                    .insert(interface.name.clone(), extension.name.clone());

                // Track functions provided by this interface
                let functions: HashSet<String> = interface.functions.iter().cloned().collect();
                self.interface_functions
                    .insert(interface.name.clone(), functions);
            }
            self.exported_interfaces
                .insert(extension.name.clone(), exported);

            // Track imported interfaces
            let mut imported = Vec::new();
            for interface in &extension.imported_interfaces {
                imported.push(interface.name.clone());
            }
            self.imported_interfaces
                .insert(extension.name.clone(), imported);
        }

        // Build dependency edges
        for extension in &manifest.xs {
            for imported in &extension.imported_interfaces {
                if let Some(provider) = self.interface_providers.get(&imported.name) {
                    // Add dependency edge
                    if let Some(deps) = self.dependencies.get_mut(&extension.name) {
                        if !deps.contains(provider) {
                            deps.push(provider.clone());
                        }
                    }

                    // Add dependent edge
                    if let Some(deps) = self.dependents.get_mut(provider) {
                        if !deps.contains(&extension.name) {
                            deps.push(extension.name.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Detects cycles in the dependency graph using depth-first search
    fn detect_cycles(&mut self) {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut cycles = Vec::new();

        for extension in &self.extension_names {
            if !visited.contains(extension) {
                self.dfs_detect_cycles(
                    extension,
                    &mut visited,
                    &mut path,
                    &mut HashSet::new(),
                    &mut cycles,
                );
            }
        }

        self.cycles = cycles;
    }

    /// Depth-first search helper for cycle detection
    fn dfs_detect_cycles(
        &self,
        current: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
        path_set: &mut HashSet<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(current.to_string());
        path.push(current.to_string());
        path_set.insert(current.to_string());

        if let Some(deps) = self.dependencies.get(current) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.dfs_detect_cycles(dep, visited, path, path_set, cycles);
                } else if path_set.contains(dep) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|x| x == dep).unwrap();
                    let cycle = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        path_set.remove(current);
    }

    /// Checks if the dependency graph has cycles
    pub fn has_cycles(&self) -> bool {
        !self.cycles.is_empty()
    }

    /// Gets the cycles in the dependency graph
    pub fn get_cycles(&self) -> &[Vec<String>] {
        &self.cycles
    }

    /// Gets a formatted string representation of the cycles
    pub fn format_cycles(&self) -> String {
        if self.cycles.is_empty() {
            return "No cycles detected".to_string();
        }

        let mut result = String::new();
        for (i, cycle) in self.cycles.iter().enumerate() {
            result.push_str(&format!("Cycle {}: ", i + 1));
            for (j, ext) in cycle.iter().enumerate() {
                result.push_str(ext);
                if j < cycle.len() - 1 {
                    result.push_str(" → ");
                }
            }
            result.push_str(" → ");
            result.push_str(&cycle[0]);
            result.push('\n');
        }
        result
    }

    /// Resolves the loading order of extensions using topological sorting
    pub fn resolve_loading_order(&self) -> Result<Vec<String>, DependencyError> {
        if self.has_cycles() {
            return Err(DependencyError::CircularDependency(self.format_cycles()));
        }

        // Kahn's algorithm for topological sorting
        let mut result = Vec::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut queue = VecDeque::new();

        // Calculate in-degree for each extension
        for ext in &self.extension_names {
            let empty_vec = Vec::new();
            let deps = self.dependencies.get(ext).unwrap_or(&empty_vec);
            in_degree.insert(ext.clone(), deps.len());
            if deps.is_empty() {
                queue.push_back(ext.clone());
            }
        }

        // Process extensions with no dependencies
        while let Some(ext) = queue.pop_front() {
            result.push(ext.clone());

            // Reduce in-degree of dependents
            if let Some(dependents) = self.dependents.get(&ext) {
                for dependent in dependents {
                    let in_deg = in_degree.get_mut(dependent).unwrap();
                    *in_deg -= 1;
                    if *in_deg == 0 {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }

        // Check if we processed all extensions
        if result.len() != self.extension_names.len() {
            return Err(DependencyError::CircularDependency(
                "Unexpected cycle detected during topological sort".to_string(),
            ));
        }

        Ok(result)
    }

    /// Validates that all dependencies are satisfied
    pub fn validate_dependencies(&self, manifest: &Manifest) -> Result<(), DependencyError> {
        for extension in &manifest.xs {
            for imported in &extension.imported_interfaces {
                // Check if the interface is exported by any extension
                if let Some(provider) = self.interface_providers.get(&imported.name) {
                    // Check if all required functions are provided
                    if let Some(provided_functions) = self.interface_functions.get(&imported.name) {
                        for function in &imported.functions {
                            if !provided_functions.contains(function) {
                                return Err(DependencyError::MissingFunction {
                                    importer: extension.name.clone(),
                                    interface: imported.name.clone(),
                                    function: function.clone(),
                                    exporter: provider.clone(),
                                });
                            }
                        }
                    }
                } else {
                    return Err(DependencyError::MissingInterface {
                        importer: extension.name.clone(),
                        interface: imported.name.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Validates that a new extension's dependencies are satisfied
    pub fn validate_extension_dependencies(
        &self,
        extension: &Extension,
        manifest: &Manifest,
    ) -> Result<(), DependencyError> {
        for imported in &extension.imported_interfaces {
            // Check if the interface is exported by any extension
            let mut provider_found = false;

            for existing in &manifest.xs {
                for exported in &existing.exported_interfaces {
                    if exported.name == imported.name {
                        provider_found = true;

                        // Check if all required functions are provided
                        for function in &imported.functions {
                            if !exported.functions.contains(function) {
                                return Err(DependencyError::MissingFunction {
                                    importer: extension.name.clone(),
                                    interface: imported.name.clone(),
                                    function: function.clone(),
                                    exporter: existing.name.clone(),
                                });
                            }
                        }
                    }
                }
            }

            if !provider_found {
                return Err(DependencyError::MissingInterface {
                    importer: extension.name.clone(),
                    interface: imported.name.clone(),
                });
            }
        }

        // Check for potential cycles
        let mut temp_manifest = manifest.clone();
        temp_manifest.xs.push(extension.clone());

        let temp_graph = DependencyGraph::new(&temp_manifest)
            .context("failed to create temporary dependency graph")?;

        if temp_graph.has_cycles() {
            return Err(DependencyError::CircularDependency(
                temp_graph.format_cycles(),
            ));
        }

        Ok(())
    }

    /// Formats a text representation of the dependency graph
    pub fn format_text_representation(&self) -> String {
        let mut result = String::new();

        for ext in &self.extension_names {
            result.push_str(&format!("Extension: {}\n", ext));

            // Exports
            if let Some(exports) = self.exported_interfaces.get(ext) {
                if exports.is_empty() {
                    result.push_str("├── Exports: none\n");
                } else {
                    result.push_str("├── Exports:\n");
                    for (i, interface) in exports.iter().enumerate() {
                        let prefix = if i == exports.len() - 1 {
                            "    └── "
                        } else {
                            "    ├── "
                        };
                        result.push_str(&format!("{}{}\n", prefix, interface));

                        // Functions
                        if let Some(functions) = self.interface_functions.get(interface) {
                            let functions: Vec<_> = functions.iter().collect();
                            for (j, function) in functions.iter().enumerate() {
                                let func_prefix = if j == functions.len() - 1 {
                                    "        └── "
                                } else {
                                    "        ├── "
                                };
                                result.push_str(&format!("{}{}\n", func_prefix, function));
                            }
                        }
                    }
                }
            }

            // Imports
            if let Some(imports) = self.imported_interfaces.get(ext) {
                if imports.is_empty() {
                    result.push_str("└── Imports: none\n");
                } else {
                    result.push_str("└── Imports:\n");
                    for (i, interface) in imports.iter().enumerate() {
                        let prefix = if i == imports.len() - 1 {
                            "    └── "
                        } else {
                            "    ├── "
                        };

                        // Find provider
                        let provider = self
                            .interface_providers
                            .get(interface)
                            .map(|p| format!(" (from {})", p))
                            .unwrap_or_else(|| " (provider not found)".to_string());

                        result.push_str(&format!("{}{}{}\n", prefix, interface, provider));
                    }
                }
            }

            result.push('\n');
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ExportedInterface, ImportedInterface};

    fn create_test_manifest() -> Manifest {
        let mut manifest = Manifest::default();

        // Extension A exports math/lib
        let mut ext_a =
            Extension::new("ext-a".to_string(), "ext-a.wasm".into(), "ext-a.bin".into());
        ext_a.add_exported_interface(ExportedInterface::new(
            "math/lib".to_string(),
            vec!["add".to_string(), "subtract".to_string()],
        ));
        manifest.xs.push(ext_a);

        // Extension B imports math/lib and exports calc/lib
        let mut ext_b =
            Extension::new("ext-b".to_string(), "ext-b.wasm".into(), "ext-b.bin".into());
        ext_b.add_imported_interface(ImportedInterface::new(
            "math/lib".to_string(),
            "ext-a".to_string(),
            vec!["add".to_string()],
        ));
        ext_b.add_exported_interface(ExportedInterface::new(
            "calc/lib".to_string(),
            vec!["calculate".to_string()],
        ));
        manifest.xs.push(ext_b);

        // Extension C imports calc/lib
        let mut ext_c =
            Extension::new("ext-c".to_string(), "ext-c.wasm".into(), "ext-c.bin".into());
        ext_c.add_imported_interface(ImportedInterface::new(
            "calc/lib".to_string(),
            "ext-b".to_string(),
            vec!["calculate".to_string()],
        ));
        manifest.xs.push(ext_c);

        manifest
    }

    fn create_cyclic_manifest() -> Manifest {
        let mut manifest = Manifest::default();

        // Extension A exports a/lib
        let mut ext_a =
            Extension::new("ext-a".to_string(), "ext-a.wasm".into(), "ext-a.bin".into());
        ext_a.add_exported_interface(ExportedInterface::new(
            "a/lib".to_string(),
            vec!["func_a".to_string()],
        ));
        ext_a.add_imported_interface(ImportedInterface::new(
            "c/lib".to_string(),
            "ext-c".to_string(),
            vec!["func_c".to_string()],
        ));
        manifest.xs.push(ext_a);

        // Extension B imports a/lib and exports b/lib
        let mut ext_b =
            Extension::new("ext-b".to_string(), "ext-b.wasm".into(), "ext-b.bin".into());
        ext_b.add_imported_interface(ImportedInterface::new(
            "a/lib".to_string(),
            "ext-a".to_string(),
            vec!["func_a".to_string()],
        ));
        ext_b.add_exported_interface(ExportedInterface::new(
            "b/lib".to_string(),
            vec!["func_b".to_string()],
        ));
        manifest.xs.push(ext_b);

        // Extension C imports b/lib and exports c/lib
        let mut ext_c =
            Extension::new("ext-c".to_string(), "ext-c.wasm".into(), "ext-c.bin".into());
        ext_c.add_imported_interface(ImportedInterface::new(
            "b/lib".to_string(),
            "ext-b".to_string(),
            vec!["func_b".to_string()],
        ));
        ext_c.add_exported_interface(ExportedInterface::new(
            "c/lib".to_string(),
            vec!["func_c".to_string()],
        ));
        manifest.xs.push(ext_c);

        manifest
    }

    #[test]
    fn test_dependency_graph_creation() {
        let manifest = create_test_manifest();
        let graph = DependencyGraph::new(&manifest).unwrap();

        assert_eq!(graph.extension_names.len(), 3);
        assert!(graph.dependencies.contains_key("ext-a"));
        assert!(graph.dependencies.contains_key("ext-b"));
        assert!(graph.dependencies.contains_key("ext-c"));
    }

    #[test]
    fn test_resolve_loading_order() {
        let manifest = create_test_manifest();
        let graph = DependencyGraph::new(&manifest).unwrap();

        let order = graph.resolve_loading_order().unwrap();

        // ext-a should be loaded first, then ext-b, then ext-c
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], "ext-a");
        assert_eq!(order[1], "ext-b");
        assert_eq!(order[2], "ext-c");
    }

    #[test]
    fn test_cycle_detection() {
        let manifest = create_cyclic_manifest();
        let graph = DependencyGraph::new(&manifest).unwrap();

        assert!(graph.has_cycles());
        assert!(!graph.get_cycles().is_empty());

        // Loading order should fail due to cycles
        let result = graph.resolve_loading_order();
        assert!(result.is_err());

        if let Err(DependencyError::CircularDependency(_)) = result {
            // Expected error
        } else {
            panic!("Expected CircularDependency error");
        }
    }

    #[test]
    fn test_validate_dependencies() {
        let manifest = create_test_manifest();
        let graph = DependencyGraph::new(&manifest).unwrap();

        let result = graph.validate_dependencies(&manifest);
        assert!(result.is_ok());

        // Create a manifest with a missing dependency
        let mut bad_manifest = manifest.clone();
        let mut ext_d =
            Extension::new("ext-d".to_string(), "ext-d.wasm".into(), "ext-d.bin".into());
        ext_d.add_imported_interface(ImportedInterface::new(
            "missing/lib".to_string(),
            "unknown".to_string(),
            vec!["func".to_string()],
        ));
        bad_manifest.xs.push(ext_d);

        let graph = DependencyGraph::new(&bad_manifest).unwrap();
        let result = graph.validate_dependencies(&bad_manifest);
        assert!(result.is_err());

        if let Err(DependencyError::MissingInterface { .. }) = result {
            // Expected error
        } else {
            panic!("Expected MissingInterface error");
        }
    }

    #[test]
    fn test_text_representation() {
        let manifest = create_test_manifest();
        let graph = DependencyGraph::new(&manifest).unwrap();

        let text = graph.format_text_representation();
        assert!(!text.is_empty());
        assert!(text.contains("Extension: ext-a"));
        assert!(text.contains("Extension: ext-b"));
        assert!(text.contains("Extension: ext-c"));
    }
}
