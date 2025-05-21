use clap::Command;

#[allow(warnings)]
mod bindings;

use bindings::{
    exports::icp::{
        build_mo::canister_build,
        cli::{cli, init},
    },
    icp::{build::registry::register_provider, cli::misc::print},
};

mod spec;
use spec::CommandSpec;

struct Component;

const CLI_SPEC: &str = r#"{
    "name": "build-mo",
    "help": "Builder for Motoko canisters",
    "args": [],
    "subcommands": []
}"#;

impl init::Guest for Component {
    fn init() -> Result<(), String> {
        register_provider(
            "motoko",                      // canister-type
            "icp:build-mo/canister-build", // interface-name
            "build-canister",              // function-name
        )?;

        Ok(())
    }
}

impl cli::Guest for Component {
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

impl canister_build::Guest for Component {
    fn build_canister(canister_dir: String) -> Result<String, String> {
        print(&format!(
            "[build-mo] Received build request for canister at: {}",
            canister_dir
        ));

        // TODO: Implement actual build logic here in the future.
        // This might involve:
        // - Reading canister.toml from canister_dir
        // - Determining build steps based on canister type
        // - Executing build commands (e.g., dfx build, cargo build)

        Ok("OUTPUT_PATH".into())
    }
}

bindings::export!(Component with_types_in bindings);
