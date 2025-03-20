use clap::Command;

#[allow(warnings)]
mod bindings;

// imports
use bindings::local::add::lib::add;
use bindings::local::host::misc::{print, time};

// exports
use bindings::exports::local::extension::cli;
use bindings::exports::local::multiply::lib;

mod spec;
use spec::CommandSpec;

struct Component;

const CLI_SPEC: &str = r#"{
    "name": "multiply",
    "help": "Multiply numbers",
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

        let _out = multiply(a, b);

        0
    }
}

impl lib::Guest for Component {
    fn multiply(a: u32, b: u32) -> u32 {
        multiply(a, b)
    }
}

fn multiply(a: u32, b: u32) -> u32 {
    let mut out = 0;
    for _ in 0..a {
        out = add(out, b);
    }

    print(&format!("[{}][multiply] {a} * {b} = {out}", time()));
    out
}

bindings::export!(Component with_types_in bindings);
