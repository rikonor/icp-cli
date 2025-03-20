use bindings::exports::local::add::lib;
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
    "name": "add",
    "help": "Add numbers",
    "args": [{ "name": "a" }, { "name": "b" }],
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
        let m = c.get_matches_from(args);

        let a: u32 = m
            .try_get_one::<String>("a")
            .unwrap()
            .unwrap()
            .parse()
            .unwrap();

        let b: u32 = m
            .try_get_one::<String>("b")
            .unwrap()
            .unwrap()
            .parse()
            .unwrap();

        let _out = add(a, b);

        0
    }
}

impl lib::Guest for Component {
    fn add(a: u32, b: u32) -> u32 {
        add(a, b)
    }
}

fn add(a: u32, b: u32) -> u32 {
    let out = a + b;
    print(&format!("[{}][add] {a} + {b} = {out}", time()));
    out
}

bindings::export!(Component with_types_in bindings);
