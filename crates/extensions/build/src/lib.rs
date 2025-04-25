use clap::Command;

#[allow(warnings)]
mod bindings;

use bindings::{exports::icp::cli_build::lib::Guest, icp::cli::misc::print};

mod spec;
use spec::CommandSpec;

struct Component;

const CLI_SPEC: &str = r#"{
    "name": "build",
    "help": "build stuff",
    "args": [],
    "subcommands": []
}"#;

fn build() -> u32 {
    print(&format!("[build] building"));
    0
}

impl Guest for Component {
    fn build() -> u32 {
        build()
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

        build();

        0
    }
}

bindings::export!(Component with_types_in bindings);
