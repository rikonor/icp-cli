use clap::Command;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::thread_local;

#[allow(warnings)]
mod bindings;

use bindings::{
    exports::icp::project::lib::CanisterInfo, icp::cli::filesystem, icp::cli::misc::print,
};

mod ops;
use ops::list::{List, Lister};

mod spec;
use spec::CommandSpec;

struct Component;

thread_local! {
    static LISTER: OnceCell<Lister> = OnceCell::with_value(Lister::new(
        Box::new(filesystem::read_file),
    ));
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
                print("Create command not implemented with new structure yet.");
                1
            }

            Some(("build", _m)) => {
                print("Build command not implemented with new structure yet.");
                1
            }

            Some(("list-canisters", _m)) => {
                let cs = match LISTER.with(|v| v.get().expect("lister not initialized").list()) {
                    Ok(cs) => cs,
                    Err(err) => return err.into(),
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
                1
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
