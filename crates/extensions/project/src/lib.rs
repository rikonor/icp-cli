use clap::Command;

#[allow(warnings)]
mod bindings;

use bindings::{
    exports::icp::project::lib::Guest,
    icp::cli::misc::{print, time},
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

fn create(name: &str) -> u32 {
    print(&format!("[{}][create] creating {name}", time()));
    0
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

                create(name.as_str());
            }

            _ => {}
        }

        0
    }
}

bindings::export!(Component with_types_in bindings);
