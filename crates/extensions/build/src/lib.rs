use clap::Command;
use dashmap::DashMap;
use once_cell::sync::{Lazy, OnceCell};
use serde::Deserialize;

#[allow(warnings)]
mod bindings;

use bindings::{
    exports::icp::{
        build::{canister_build, registry},
        cli::{cli, init},
    },
    icp::cli::{filesystem::read_file, misc::print},
};

mod ops;
use ops::build::{Build, Builder};

mod spec;
use spec::CommandSpec;

struct Component;

thread_local! {
    static BUILDER: OnceCell<Box<dyn Build>> = OnceCell::with_value({
        let v = Builder::new(
            Box::new(read_file), //
        );

        Box::new(v)
    });
}

#[derive(Deserialize, Debug)]
struct CanisterProperties {
    name: String,
    #[serde(rename = "type")]
    canister_type: String,
}

#[derive(Deserialize, Debug)]
struct CanisterManifest {
    canister: CanisterProperties,
}

const CLI_SPEC: &str = r#"{
    "name": "build",
    "help": "Canister builder",
    "args": [
        { "name": "dir", "required": true }
    ],
    "subcommands": []
}"#;

static BUILDERS: Lazy<DashMap<String, ()>> = Lazy::new(|| DashMap::new());

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
        let canister_dir = m.try_get_one::<String>("dir").unwrap().unwrap();

        print("available builders are:");
        BUILDERS.iter().for_each(|v| {
            print(&format!("  - {}", v.key()));
        });

        match BUILDER.with(|v| {
            v.get()
                .expect("builder not initialized")
                .build(&canister_dir)
        }) {
            // Ok
            Ok(_) => 0,

            // Failure
            Err(err) => {
                print(&format!("{err}"));
                err.into()
            }
        }
    }
}

impl registry::Guest for Component {
    fn register_provider(canister_type: String) -> Result<(), String> {
        BUILDERS.insert(canister_type, ());

        Ok(())
    }
}

impl canister_build::Guest for Component {
    fn build_canister(canister_dir: String) -> Result<(), String> {
        BUILDER
            .with(|v| {
                v.get()
                    .expect("builder not initialized")
                    .build(&canister_dir)
            })
            .map_err(|err| format!("{err}"))
    }
}

bindings::export!(Component with_types_in bindings);
