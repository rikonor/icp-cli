use clap::Command;

#[allow(warnings)]
mod bindings;

use bindings::{
    exports::icp::{
        cli::{cli, init},
        identity,
    },
    icp::cli::misc::print,
};

mod spec;
use spec::CommandSpec;

struct Component;

const CLI_SPEC: &str = r#"{
    "name": "identity",
    "help": "identity stuff",
    "args": [],
    "subcommands": [
        {
            "name": "create",
            "args": [
                { "name": "name", "required": true }
            ]
        },
        {
            "name": "sign",
            "args": [
                { "name": "blob", "required": true }
            ]
        }
    ]
}"#;

fn create(name: &str) -> u32 {
    print(&format!("[identity] creating identity: {name}"));
    0
}

fn sign(blob: &str) -> u32 {
    print(&format!("[identity] signing blob: {blob}"));
    0
}

impl init::Guest for Component {
    fn init() -> Result<(), String> {
        Ok(())
    }
}

impl identity::lib::Guest for Component {
    fn create() -> u32 {
        print("creating identity");
        0
    }

    // fn sign() -> u32 {
    //     unimplemented!()
    // }
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
            // create
            Some(("create", m)) => {
                let name = m.try_get_one::<String>("name").unwrap().unwrap();

                create(&name);
            }

            // sign
            Some(("sign", m)) => {
                let blob = m.try_get_one::<String>("blob").unwrap().unwrap();

                sign(&blob);
            }

            _ => {}
        }

        0
    }
}

bindings::export!(Component with_types_in bindings);
