use clap::Command;

#[allow(warnings)]
mod bindings;

use bindings::{
    exports::icp::project::lib::Guest, // Exported library interface (currently unimplemented)
    icp::cli::filesystem,              // Imported custom filesystem interface
    icp::cli::misc::{print, time},     // Imported misc utilities
};

mod spec;
use spec::CommandSpec;

struct Component;

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
        }
    ]
}"#;

// Creates a project directory and a basic dfx.json file.
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

    // Create a placeholder dfx.json file inside the new directory
    let dfx_path = format!("{}/dfx.json", name);
    let dfx_content = b"{\n  \"canisters\": {}\n}\n"; // Basic empty dfx.json content
    match filesystem::write_file(&dfx_path, dfx_content) {
        Ok(_) => {
            print(&format!("Created file: {}", dfx_path));
        }
        Err(e) => {
            print(&format!("Error creating file '{}': {}", dfx_path, e));
            // Optional: Consider attempting to clean up the created directory here
            return 1; // Indicate failure
        }
    }

    print(&format!("Project '{}' created successfully.", name));
    0 // Indicate success
}

impl Guest for Component {
    fn create() -> u32 {
        unimplemented!()
    }
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

            // Handle unknown subcommands or no subcommand
            _ => {
                // You might want to print usage information here
                print("Unknown command or missing subcommand.");
                return 1; // Indicate failure
            }
        }

        // This part is now unreachable if subcommands always return
        // 0
    }
}

bindings::export!(Component with_types_in bindings);
