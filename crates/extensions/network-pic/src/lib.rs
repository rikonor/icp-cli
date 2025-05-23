use clap::Command;

#[allow(warnings)]
mod bindings;

use bindings::{
    exports::icp::cli::{cli, init},
    icp::cli::{command::execute, misc::print},
};

mod spec;
use spec::CommandSpec;

struct Component;

const CLI_SPEC: &str = r#"{
    "name": "network-pic",
    "help": "Pocket IC network extension",
    "args": [],
    "subcommands": [
        {
            "name": "start",
            "args": [
                { "name": "ttl", "required": true }
            ]
        }
    ]
}"#;

impl init::Guest for Component {
    fn init() -> Result<(), String> {
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
        let m = c.get_matches_from(args);

        match m.subcommand() {
            // start
            Some(("start", m)) => {
                let ttl = m.try_get_one::<String>("ttl").unwrap().unwrap();
                print(&format!("Starting pocket-ic (ttl: {ttl})"));
                let out = execute("pocket-ic", &["--ttl".to_string(), ttl.to_owned()]);
                print(&format!("{out:?}"));
            }

            _ => {}
        }

        0
    }
}

bindings::export!(Component with_types_in bindings);
