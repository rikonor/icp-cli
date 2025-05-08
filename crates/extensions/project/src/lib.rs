use std::{thread::LocalKey, thread_local};

use clap::Command;
use once_cell::sync::OnceCell;
use ops::list::ListError;
use serde::Deserialize;

#[allow(warnings)]
mod bindings;
use bindings::{
    exports::icp::{
        cli::{cli, init},
        project::{self, lib::CanisterInfo},
    },
    icp::{
        build::canister_build::build_canister,
        cli::{filesystem, misc::print},
    },
};

mod ops;
use ops::{
    build::{Build, Builder},
    create::{Create, Creator},
    list::{List, Lister},
};

mod spec;
use spec::CommandSpec;

pub type LocalRef<T> = &'static LocalKey<OnceCell<T>>;

struct Component;

thread_local! {
    static CREATOR: OnceCell<Box<dyn Create>> = OnceCell::with_value({
        let v = Creator::new(
            Box::new(filesystem::read_file),
        );

        Box::new(v)
    });
}

thread_local! {
    static LISTER: OnceCell<Box<dyn List>> = OnceCell::with_value({
        let v = Lister::new(
            Box::new(filesystem::read_file),
        );

        Box::new(v)
    });
}

thread_local! {
    static BUILDER: OnceCell<Box<dyn Build>> = OnceCell::with_value({
        let v = Builder::new(
            &LISTER,
            Box::new(build_canister),
        );

        Box::new(v)
    });
}

#[derive(Deserialize, Debug)]
struct ProjectManifest {
    workspace: Workspace,
}

#[derive(Deserialize, Debug)]
struct Workspace {
    members: Vec<String>,
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
    "name": "project",
    "help": "Internet Computer Project CLI",
    "args": [],
    "subcommands": [
        {
            "name": "create",
            "args": [
                { "name": "name", "required": true }
            ]
        },
        {
            "name": "build",
            "help": "Build canisters in the project",
            "args": []
        },
        {
            "name": "list-canisters",
            "help": "List canisters defined in the project",
            "args": []
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
        let cspec: CommandSpec =
            serde_json::from_str(CLI_SPEC).expect("invalid command-line interface spec");

        let cmd: Command = cspec.into();
        let ms = cmd.get_matches_from(args);

        match ms.subcommand() {
            Some(("create", _m)) => {
                match CREATOR.with(|v| v.get().expect("creator not initialized").create()) {
                    // Success
                    Ok(_) => 0,

                    // Failure
                    Err(_) => 1,
                }
            }

            Some(("build", _m)) => {
                match BUILDER.with(|v| v.get().expect("builder not initialized").build()) {
                    // Success
                    Ok(_) => 0,

                    // Failure
                    Err(err) => {
                        print(&err.to_string());
                        err.into()
                    }
                }
            }

            Some(("list-canisters", _m)) => {
                match LISTER.with(|v| v.get().expect("lister not initialized").list()) {
                    // Success
                    Ok(cs) => {
                        print("Found canisters:");
                        for canister in cs {
                            print(&format!("  - {:?}", canister));
                        }

                        0
                    }

                    // Failure
                    Err(err) => {
                        // Print the specific error message
                        print(&err.to_string());

                        // Return the appropriate exit code based on the error type
                        match err {
                            ListError::NoCanistersFound => 3,
                            ListError::ManifestProcessing(_) => 2,
                            ListError::Unexpected(_) => 1,
                        }
                    }
                }
            }

            _ => {
                print("Unknown command or arguments.");

                1 // Failure
            }
        }
    }
}

impl project::lib::Guest for Component {
    fn list_canisters() -> Result<Vec<CanisterInfo>, String> {
        Ok(LISTER.with(|v| v.get().expect("lister not initialized").list())?)
    }
}

bindings::export!(Component with_types_in bindings);
