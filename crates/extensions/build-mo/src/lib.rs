use clap::Command;
use once_cell::sync::OnceCell;
use serde::Deserialize;

#[allow(warnings)]
mod bindings;

use bindings::{
    exports::icp::{
        build_mo::canister_build,
        cli::{cli, init},
    },
    icp::{
        build::registry::register_provider,
        cli::{command::execute, filesystem::read_file, misc::print},
    },
};

mod ops;
use ops::build::{Build, Builder};

mod spec;
use spec::CommandSpec;

struct Component;

thread_local! {
    static BUILDER: OnceCell<Box<dyn Build>> = OnceCell::with_value({
        let v = Builder::new(
            Box::new(read_file),
            Box::new(execute),
        );

        Box::new(v)
    });
}

#[derive(Deserialize, Debug)]
struct CanisterProperties {
    #[allow(unused)]
    name: String,

    #[serde(rename = "type")]
    canister_type: String,
}

#[derive(Deserialize, Debug)]
struct CanisterManifest {
    canister: CanisterProperties,
}

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
        let m = c.get_matches_from(args);
        let canister_dir = m.try_get_one::<String>("dir").unwrap().unwrap();

        match BUILDER.with(|v| {
            v.get()
                .expect("builder not initialized")
                .build(&canister_dir)
        }) {
            // Ok
            Ok(output_path) => {
                print(&format!("{output_path}"));
                0
            }

            // Failure
            Err(err) => {
                print(&format!("{err}"));
                err.into()
            }
        }
    }
}

impl canister_build::Guest for Component {
    fn build_canister(canister_dir: String) -> Result<String, String> {
        print(&format!(
            "[build-mo] Received build request for canister at: {}",
            canister_dir
        ));

        BUILDER.with(|v| {
            v.get()
                .expect("builder not initialized")
                .build(&canister_dir)
                .map_err(|err| format!("{err}"))
        })
    }
}

bindings::export!(Component with_types_in bindings);
