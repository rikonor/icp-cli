use clap::Command;
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::thread_local;

#[allow(warnings)]
mod bindings;
mod ops;
mod spec;

use bindings::{
    exports::icp::project::lib::CanisterInfo, icp::cli::filesystem, icp::cli::misc::print,
};

use ops::list::{List, ListError, Lister};
use spec::CommandSpec;

struct Component;

thread_local! {
    static LISTER: OnceCell<Lister> = OnceCell::with_value(Lister::new(
        Box::new(filesystem::read_file), // read_file
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
                    Err(ListError::ManifestProcessing(_)) => return 1,
                    Err(ListError::Unexpected(_)) => return 2,
                };

                // print results

                1
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
        LISTER.with(|v| Ok(v.get().expect("lister not initialized").list()?))
    }
}

impl From<ListError> for String {
    fn from(e: ListError) -> Self {
        format!("List error: {:?}", e)
    }
}

bindings::export!(Component with_types_in bindings);
