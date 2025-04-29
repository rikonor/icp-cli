use clap::Command;
use once_cell::sync::OnceCell;
use ops::build::{Build, Builder};
use ops::create::{Create, Creator};
use serde::Deserialize;
use std::thread::LocalKey;
use std::thread_local;

#[allow(warnings)]
mod bindings;
mod ops;

mod spec;
use spec::CommandSpec;

use bindings::exports::icp::project::lib::CanisterInfo;
use bindings::{icp::build::lib::build_canister, icp::cli::filesystem, icp::cli::misc::print};

use ops::list::{List, Lister};

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
struct CanisterManifest {
    name: String,
    #[serde(rename = "type")]
    canister_type: String,
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

impl bindings::exports::icp::cli::cli::Guest for Component {
    fn spec() -> String {
        CLI_SPEC.to_string()
    }

    fn run(args: Vec<String>) -> u8 {
        let cspec: CommandSpec =
            serde_json::from_str(CLI_SPEC).expect("invalid command-line interface spec");

        let cmd: Command = cspec.into(); // Use impl From<CommandSpec> from spec.rs
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
                    Err(_) => 1,
                }
            }

            Some(("list-canisters", _m)) => {
                let cs = match LISTER.with(|v| v.get().expect("lister not initialized").list()) {
                    Ok(cs) => cs,

                    // Failure
                    Err(err) => {
                        print(&format!("{err:?}"));
                        return err.into();
                    }
                };

                match cs.is_empty() {
                    // empty
                    true => print("No canisters found in the project."),

                    //
                    false => {
                        print("Found canisters:");
                        for canister in cs {
                            print(&format!("  - {:?}", canister));
                        }
                    }
                }

                0 // Success
            }

            _ => {
                print("Unknown command or arguments.");

                1 // Failure
            }
        }
    }
}

impl bindings::exports::icp::project::lib::Guest for Component {
    fn list_canisters() -> Result<Vec<CanisterInfo>, String> {
        Ok(LISTER.with(|v| v.get().expect("lister not initialized").list())?)
    }
}

bindings::export!(Component with_types_in bindings);
