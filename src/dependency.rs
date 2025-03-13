use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::Context;

use crate::manifest::{Extension, Manifest};

/// Errors that can occur during dependency resolution
#[derive(Debug, thiserror::Error)]
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
#[derive(Debug)]
pub struct DependencyGraph {
    /// Map from extension name to the names of extensions it depends on
    dependencies: HashMap<String, Vec<String>>,

    /// Map from extension name to the names of extensions that depend on it
    dependents: HashMap<String, Vec<String>>,

    /// Map from interface name to the extension that exports it
    interface_providers: HashMap<String, String>,

    /// Map from extension name to the interfaces it exports
    exports: HashMap<String, Vec<String>>,

    /// Map from extension name to the interfaces it imports
    imports: HashMap<String, Vec<String>>,

    /// Map from interface name to the functions it provides
    interface_functions: HashMap<String, HashSet<String>>,

    /// All extension names in the graph
    extension_names: Vec<String>,

    /// Detected cycles in the dependency graph
    cycles: Vec<Vec<String>>,
}

impl DependencyGraph {
    /// Creates a new dependency graph from a manifest
    pub fn new(m: &Manifest) -> Result<Self, DependencyError> {
        let mut g = Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
            interface_providers: HashMap::new(),
            exports: HashMap::new(),
            imports: HashMap::new(),
            interface_functions: HashMap::new(),
            extension_names: Vec::new(),
            cycles: Vec::new(),
        };

        g.build(m)?;
        g.detect_cycles();

        Ok(g)
    }
}

impl DependencyGraph {
    /// Builds the dependency graph from a manifest
    fn build(&mut self, m: &Manifest) -> Result<(), DependencyError> {
        for x in &m.xs {
            // Names
            self.extension_names.push(x.name.clone());

            // Dependencies
            self.dependencies.insert(x.name.clone(), Vec::new());

            // Dependents
            self.dependents.insert(x.name.clone(), Vec::new());
        }

        // Track imports
        for x in &m.xs {
            let mut imps = Vec::new();

            for iface in &x.imports {
                imps.push(iface.name.clone());
            }

            self.imports.insert(
                x.name.clone(), // name
                imps,           // imports
            );
        }

        // Track exports
        for x in &m.xs {
            let mut exps = Vec::new();

            for iface in &x.exports {
                exps.push(iface.name.clone());

                self.interface_providers.insert(
                    iface.name.clone(), // interface
                    x.name.clone(),     // extension
                );

                // Track functions provided by this interface
                self.interface_functions.insert(
                    iface.name.clone(),
                    iface.funcs.iter().cloned().collect::<HashSet<String>>(),
                );
            }

            self.exports.insert(
                x.name.clone(), // name
                exps,           // exports
            );
        }

        // Build dependency edges
        for x in &m.xs {
            for imp in &x.imports {
                if let Some(p) = self.interface_providers.get(&imp.name) {
                    // Add dependency edge
                    if let Some(deps) = self.dependencies.get_mut(&x.name) {
                        if !deps.contains(p) {
                            deps.push(p.clone());
                        }
                    }

                    // Add dependent edge
                    if let Some(deps) = self.dependents.get_mut(p) {
                        if !deps.contains(&x.name) {
                            deps.push(x.name.clone());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl DependencyGraph {
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
}

impl DependencyGraph {
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
    pub fn validate_dependencies(&self, m: &Manifest) -> Result<(), DependencyError> {
        for x in &m.xs {
            for imp in &x.imports {
                // Check if the interface is exported by any extension
                match self.interface_providers.get(&imp.name) {
                    Some(p) => {
                        if let Some(fs) = self.interface_functions.get(&imp.name) {
                            for f in &imp.functions {
                                if !fs.contains(f) {
                                    return Err(DependencyError::MissingFunction {
                                        importer: x.name.clone(),
                                        interface: imp.name.clone(),
                                        function: f.clone(),
                                        exporter: p.clone(),
                                    });
                                }
                            }
                        }
                    }

                    None => {
                        return Err(DependencyError::MissingInterface {
                            importer: x.name.clone(),
                            interface: imp.name.clone(),
                        })
                    }
                }
            }
        }

        Ok(())
    }

    /// Validates that a new extension's dependencies are satisfied
    pub fn validate_extension_dependencies(
        &self,
        x: &Extension,
        m: &Manifest,
    ) -> Result<(), DependencyError> {
        for imp in &x.imports {
            // Check if the interface is exported by any extension
            let mut provider_found = false;

            for x in &m.xs {
                for exp in &x.exports {
                    if exp.name == imp.name {
                        provider_found = true;

                        // Check if all required functions are provided
                        for function in &imp.functions {
                            if !exp.funcs.contains(function) {
                                return Err(DependencyError::MissingFunction {
                                    importer: x.name.clone(),
                                    interface: imp.name.clone(),
                                    function: function.clone(),
                                    exporter: x.name.clone(),
                                });
                            }
                        }
                    }
                }
            }

            if !provider_found {
                return Err(DependencyError::MissingInterface {
                    importer: x.name.clone(),
                    interface: imp.name.clone(),
                });
            }
        }

        // Check for potential cycles
        let mut mtmp = m.clone();
        mtmp.xs.push(x.clone());

        let gtmp =
            DependencyGraph::new(&mtmp).context("failed to create temporary dependency graph")?;

        if gtmp.has_cycles() {
            return Err(DependencyError::CircularDependency(gtmp.format_cycles()));
        }

        Ok(())
    }
}

impl DependencyGraph {
    /// Formats a text representation of the dependency graph
    pub fn format_text_representation(&self) -> String {
        let mut result = String::new();

        for ext in &self.extension_names {
            result.push_str(&format!("Extension: {}\n", ext));

            // Exports
            if let Some(exps) = self.exports.get(ext) {
                if exps.is_empty() {
                    result.push_str("├── Exports: none\n");
                } else {
                    result.push_str("├── Exports:\n");
                    for (i, iface) in exps.iter().enumerate() {
                        let prefix = if i == exps.len() - 1 {
                            "    └── "
                        } else {
                            "    ├── "
                        };
                        result.push_str(&format!("{}{}\n", prefix, iface));

                        // Functions
                        if let Some(fs) = self.interface_functions.get(iface) {
                            for (j, f) in fs.iter().collect::<Vec<_>>().iter().enumerate() {
                                let func_prefix = if j == fs.len() - 1 {
                                    "        └── "
                                } else {
                                    "        ├── "
                                };
                                result.push_str(&format!("{}{}\n", func_prefix, f));
                            }
                        }
                    }
                }
            }

            // Imports
            if let Some(imps) = self.imports.get(ext) {
                if imps.is_empty() {
                    result.push_str("└── Imports: none\n");
                } else {
                    result.push_str("└── Imports:\n");
                    for (i, iface) in imps.iter().enumerate() {
                        let prefix = if i == imps.len() - 1 {
                            "    └── "
                        } else {
                            "    ├── "
                        };

                        // Find provider
                        let p = self
                            .interface_providers
                            .get(iface)
                            .map(|p| format!(" (from {})", p))
                            .unwrap_or_else(|| " (provider not found)".to_string());

                        result.push_str(&format!("{}{}{}\n", prefix, iface, p));
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
    use anyhow::Error;

    use super::*;
    use crate::manifest::{ExportedInterface, ImportedInterface};

    fn create_test_manifest() -> Manifest {
        let mut m = Manifest::default();

        // Extension A exports math/lib
        m.xs.push(Extension {
            name: "ext-a".to_string(),
            wasm: "ext-a.wasm".into(),
            pre: "ext-a.bin".into(),
            imports: Vec::new(),
            exports: vec![ExportedInterface {
                name: "math/lib".to_string(),
                funcs: vec!["add".to_string(), "subtract".to_string()],
            }],
        });

        // Extension B imports math/lib and exports calc/lib
        m.xs.push(Extension {
            name: "ext-b".to_string(),
            wasm: "ext-b.wasm".into(),
            pre: "ext-b.bin".into(),
            imports: vec![ImportedInterface {
                name: "math/lib".to_string(),
                provider: "ext-a".to_string(),
                functions: vec!["add".to_string()],
            }],
            exports: vec![ExportedInterface {
                name: "calc/lib".to_string(),
                funcs: vec!["calculate".to_string()],
            }],
        });

        // Extension C imports calc/lib
        m.xs.push(Extension {
            name: "ext-c".to_string(),
            wasm: "ext-c.wasm".into(),
            pre: "ext-c.bin".into(),
            imports: vec![ImportedInterface {
                name: "calc/lib".to_string(),
                provider: "ext-b".to_string(),
                functions: vec!["calculate".to_string()],
            }],
            exports: Vec::new(),
        });

        m
    }

    fn create_cyclic_manifest() -> Manifest {
        let mut m = Manifest::default();

        // Extension A exports a/lib
        m.xs.push(Extension {
            name: "ext-a".to_string(),
            wasm: "ext-a.wasm".into(),
            pre: "ext-a.bin".into(),
            exports: vec![ExportedInterface {
                name: "a/lib".to_string(),
                funcs: vec!["func_a".to_string()],
            }],
            imports: vec![ImportedInterface {
                name: "c/lib".to_string(),
                provider: "ext-c".to_string(),
                functions: vec!["func_c".to_string()],
            }],
        });

        // Extension B imports a/lib and exports b/lib
        m.xs.push(Extension {
            name: "ext-b".to_string(),
            wasm: "ext-b.wasm".into(),
            pre: "ext-b.bin".into(),
            imports: vec![ImportedInterface {
                name: "a/lib".to_string(),
                provider: "ext-a".to_string(),
                functions: vec!["func_a".to_string()],
            }],
            exports: vec![ExportedInterface {
                name: "b/lib".to_string(),
                funcs: vec!["func_b".to_string()],
            }],
        });

        // Extension C imports b/lib and exports c/lib
        m.xs.push(Extension {
            name: "ext-c".to_string(),
            wasm: "ext-c.wasm".into(),
            pre: "ext-c.bin".into(),
            imports: vec![ImportedInterface {
                name: "b/lib".to_string(),
                provider: "ext-b".to_string(),
                functions: vec!["func_b".to_string()],
            }],
            exports: vec![ExportedInterface {
                name: "c/lib".to_string(),
                funcs: vec!["func_c".to_string()],
            }],
        });

        m
    }

    #[test]
    fn test_dependency_graph_creation() -> Result<(), Error> {
        let g = DependencyGraph::new(&create_test_manifest())?;

        // Test extension names
        assert_eq!(g.extension_names.len(), 3);
        assert!(g.extension_names.contains(&"ext-a".to_string()));
        assert!(g.extension_names.contains(&"ext-b".to_string()));
        assert!(g.extension_names.contains(&"ext-c".to_string()));

        // Test dependencies
        assert_eq!(g.dependencies.get("ext-a").unwrap().len(), 0);
        assert_eq!(g.dependencies.get("ext-b").unwrap().len(), 1);
        assert!(g
            .dependencies
            .get("ext-b")
            .unwrap()
            .contains(&"ext-a".to_string()));
        assert_eq!(g.dependencies.get("ext-c").unwrap().len(), 1);
        assert!(g
            .dependencies
            .get("ext-c")
            .unwrap()
            .contains(&"ext-b".to_string()));

        // Test interface providers
        assert_eq!(g.interface_providers.get("math/lib").unwrap(), "ext-a");
        assert_eq!(g.interface_providers.get("calc/lib").unwrap(), "ext-b");

        Ok(())
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
        let mut ext_d = Extension {
            name: "ext-d".to_string(),
            wasm: "ext-d.wasm".into(),
            pre: "ext-d.bin".into(),
            imports: vec![ImportedInterface {
                name: "missing/lib".to_string(),
                provider: "unknown".to_string(),
                functions: vec!["func".to_string()],
            }],
            exports: Vec::new(),
        };
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
