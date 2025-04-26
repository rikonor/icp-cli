use clap::Command;
use glob::glob; // Added for glob pattern matching
use serde::Deserialize; // Added for TOML deserialization
use std::{fs, path::Path}; // Added fs for std::fs::read_to_string

#[allow(warnings)]
mod bindings;

use bindings::{
    icp::build::lib as build_lib,  // Imported build library interface
    icp::cli::filesystem,          // Imported custom filesystem interface (write-only for now)
    icp::cli::misc::{print, time}, // Imported misc utilities
};

mod spec;
use spec::CommandSpec;

struct Component;

// Define structs for deserializing icp.toml
#[derive(Deserialize, Debug)]
struct ProjectManifest {
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    members: Vec<String>,
}

const CLI_SPEC: &str = r#"{
    "name": "project",
    "help": "Internet Computer Project CLI",
    "args": [],
    "subcommands": [
        {
            "name": "create",
            "args": [
                { "name": "name", "required": true }
            ]
        },
        {
            "name": "build",
            "help": "Build canisters in the project",
            "args": []
        }
    ]
}"#;

// Creates a project directory and a basic icp.toml file.
// Returns 0 on success, 1 on failure.
fn create(name: &str) -> u8 {
    print(&format!("[{}] Creating project '{}'...", time(), name));

    // Create the main project directory
    match filesystem::create_directory(name) {
        Ok(_) => {
            print(&format!("Created directory: {}", name));
        }
        Err(e) => {
            print(&format!("Error creating directory '{}': {}", name, e));
            return 1; // Indicate failure
        }
    }

    // Create a placeholder icp.toml file inside the new directory
    let icp_toml_path = format!("{}/icp.toml", name);
    let icp_toml_content = b"[workspace]\nmembers = []\n"; // Basic empty workspace
    match filesystem::write_file(&icp_toml_path, icp_toml_content) {
        Ok(_) => {
            print(&format!("Created file: {}", icp_toml_path));
        }
        Err(e) => {
            print(&format!("Error creating file '{}': {}", icp_toml_path, e));
            // Optional: Consider attempting to clean up the created directory here
            return 1; // Indicate failure
        }
    }

    print(&format!("Project '{}' created successfully.", name));
    0 // Indicate success
}

impl bindings::exports::icp::cli::cli::Guest for Component {
    fn spec() -> String {
        CLI_SPEC.to_string()
    }

    fn run(args: Vec<String>) -> u8 {
        // Parse the CLI spec
        let cspec: CommandSpec =
            serde_json::from_str(CLI_SPEC).expect("invalid command-line interface");

        // Convert the spec into a clap Command
        let c: Command = cspec.into();

        // Parse the command-line arguments
        let m = c.get_matches_from(args);

        match m.subcommand() {
            // create
            Some(("create", m)) => {
                let name = m.try_get_one::<String>("name").unwrap().unwrap();

                // Call the create function and return its status code
                return create(name.as_str());
            }

            // build
            Some(("build", _m)) => {
                print(&format!("[{}] Starting project build...", time()));
                let mut overall_success = true;

                // 1. Read icp.toml using std::fs temporarily
                // TODO: Replace with host-mediated filesystem::read_file if added later
                let manifest_content_str = match fs::read_to_string("icp.toml") {
                    Ok(s) => s,
                    Err(e) => {
                        print(&format!("Error reading icp.toml using std::fs: {}", e));
                        return 1;
                    }
                };

                // 2. Parse icp.toml
                let manifest: ProjectManifest = match toml::from_str(&manifest_content_str) {
                    Ok(m) => m,
                    Err(e) => {
                        print(&format!("Error parsing icp.toml: {}", e));
                        return 1;
                    }
                };

                // 3. Iterate through members and build
                if manifest.workspace.members.is_empty() {
                    print("No members found in icp.toml workspace. Nothing to build.");
                } else {
                    print(&format!("Found members: {:?}", manifest.workspace.members));
                }

                for member_pattern in manifest.workspace.members {
                    print(&format!("Processing pattern: {}", member_pattern));
                    match glob(&member_pattern) {
                        Ok(paths) => {
                            let mut found_match = false;
                            for entry in paths {
                                match entry {
                                    Ok(path) => {
                                        found_match = true;
                                        if path.is_dir() {
                                            let path_str = path.to_string_lossy().to_string();
                                            print(&format!(
                                                "Attempting to build canister at: {}",
                                                path_str
                                            ));
                                            // 4. Call build extension
                                            match build_lib::build_canister(&path_str) {
                                                Ok(_) => {
                                                    print(&format!(
                                                        "Successfully called build for: {}",
                                                        path_str
                                                    ));
                                                }
                                                Err(e) => {
                                                    print(&format!(
                                                        "Build call failed for '{}': {}",
                                                        path_str, e
                                                    ));
                                                    overall_success = false; // Mark failure
                                                }
                                            }
                                        } else {
                                            print(&format!(
                                                "Skipping non-directory path from glob: {}",
                                                path.display()
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        print(&format!(
                                            "Error processing path for pattern '{}': {}",
                                            member_pattern, e
                                        ));
                                        overall_success = false; // Mark failure
                                    }
                                }
                            }
                            if !found_match {
                                print(&format!(
                                    "Warning: Glob pattern '{}' did not match any paths.",
                                    member_pattern
                                ));
                            }
                        }
                        Err(e) => {
                            print(&format!(
                                "Error processing glob pattern '{}': {}",
                                member_pattern, e
                            ));
                            overall_success = false; // Mark failure
                        }
                    }
                }

                if overall_success {
                    print("Project build process completed.");
                    return 0;
                } else {
                    print("Project build process completed with errors.");
                    return 1;
                }
            }

            // Handle unknown subcommands or no subcommand
            _ => {
                // TODO: Print usage information from clap?
                print("Unknown command or missing subcommand.");
                return 1; // Indicate failure
            }
        }
        // Note: The end of the function is now technically reachable if a subcommand
        // doesn't explicitly return, though current logic ensures they do.
        // Adding a default return just in case.
        // 0
    }
}

bindings::export!(Component with_types_in bindings);
