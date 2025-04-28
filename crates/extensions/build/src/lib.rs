use clap::Command;

#[allow(warnings)]
mod bindings;

use bindings::{exports::icp::build::lib::Guest, icp::cli::misc::print};

mod spec;
use spec::CommandSpec;

struct Component;

const CLI_SPEC: &str = r#"{
    "name": "build",
    "help": "build stuff",
    "args": [],
    "subcommands": []
}"#;

// Removed old build() function as it's replaced by build_canister

impl Guest for Component {
    // Implement the new function from the WIT interface
    fn build_canister(canister_dir: String) -> Result<(), String> {
        // Mock implementation: Just print the received path
        print(&format!(
            "[build extension] Received build request for canister at: {}",
            canister_dir
        ));

        // TODO: Implement actual build logic here in the future.
        // This might involve:
        // - Reading canister.toml from canister_dir
        // - Determining build steps based on canister type
        // - Executing build commands (e.g., dfx build, cargo build)

        Ok(()) // Simulate success for now
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
        let _m = c.get_matches_from(args);

        // The standalone `icp build` command is less useful now.
        // Building is primarily driven by `icp project build`.
        // Print a message and exit.
        print("Executing standalone `icp build` is not the standard workflow.");
        print("Use `icp project build` to build canisters defined in your project.");

        1 // Return error code
    }
}

bindings::export!(Component with_types_in bindings);
