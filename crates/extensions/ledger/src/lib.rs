use clap::Command;

#[allow(warnings)]
mod bindings;

// imports
use bindings::local::host::misc::{print, time};

// exports
use bindings::exports::local::extension::cli;

mod spec;
use spec::CommandSpec;

struct Component;

const CLI_SPEC: &str = r#"{
    "name": "ledger",
    "help": "Ledger extension",
    "args": [],
    "subcommands": []
}"#;

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

        // Print the current time
        print(&format!("[{}] Hello from the ledger extension!", time()));

        0
    }
}

bindings::export!(Component with_types_in bindings);
