// Allow warnings for unused imports, etc., as bindings can be extensive
#[allow(warnings)]
mod bindings;

// Import the host function we declared in WIT
use bindings::local::host::misc::print;

// Import the Guest traits for the interfaces we are exporting
use bindings::exports::local::extension::cli;
use bindings::exports::local::minimal::lib;

// Define a struct to implement the guest traits
struct MinimalComponent;

// Minimal CLI specification: no arguments, no subcommands.
// Represented as a JSON string conforming to the expected schema.
const MINIMAL_CLI_SPEC: &str = r#"{
    "name": "minimal",
    "help": "A minimal extension example.",
    "args": [],
    "subcommands": []
}"#;

// Implement the standard CLI extension interface
impl cli::Guest for MinimalComponent {
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
impl lib::Guest for MinimalComponent {
    // Implement the ping function
    fn ping() -> String {
        print("[minimal extension] ping called.");
        "pong".to_string()
    }
}

// Export the component implementation using the bindings macro
bindings::export!(MinimalComponent with_types_in bindings);
