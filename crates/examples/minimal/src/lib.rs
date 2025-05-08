// Allow warnings for unused imports, etc., as bindings can be extensive
#[allow(warnings)]
mod bindings;

// Import the host function we declared in WIT
use bindings::icp::cli::misc::print;

// Import the Guest traits for the interfaces we are exporting
use bindings::exports::icp::cli::{cli, init};
use bindings::exports::icp::minimal::lib;

// Define a struct to implement the guest traits
struct Component;

const MINIMAL_CLI_SPEC: &str = r#"{
    "name": "minimal",
    "help": "A minimal extension example.",
    "args": [],
    "subcommands": []
}"#;

// Implement initialization functionality for the extension
impl init::Guest for Component {
    fn init() -> Result<(), String> {
        Ok(())
    }
}

// Implement the standard CLI extension interface
impl cli::Guest for Component {
    // Return the JSON spec string
    fn spec() -> String {
        MINIMAL_CLI_SPEC.to_string()
    }

    // Implement the run logic. Since there are no args, just print and succeed.
    fn run(_args: Vec<String>) -> u8 {
        // Use the imported print function from the host
        print("[minimal extension] run called, doing nothing.");
        // Return success code
        0
    }
}

// Implement the custom minimal library interface
impl lib::Guest for Component {
    fn greet(name: String) -> String {
        print("[minimal extension] ping called.");
        format!("Hello, {name}")
    }
}

// Export the component implementation using the bindings macro
bindings::export!(Component with_types_in bindings);
