use clap::Command;

#[allow(warnings)]
mod bindings;

// imports
use bindings::local::host::misc::{print, time};
use bindings::local::multiply::lib::multiply;

// exports
use bindings::exports::local::extension::cli;
use bindings::exports::local::power::lib;

mod spec;
use spec::CommandSpec;

struct Component;

const CLI_SPEC: &str = r#"{
    "name": "power",
    "help": "Take a power (a^b)",
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

        let _out = power(a, b);

        0
    }
}

impl lib::Guest for Component {
    fn power(a: u32, b: u32) -> u32 {
        power(a, b)
    }
}

fn power(a: u32, b: u32) -> u32 {
    let mut out = 1;
    for _ in 0..b {
        out = multiply(out, a)
    }

    print(&format!("[{}][power] {a}^{b} = {out}", time()));
    out
}

bindings::export!(Component with_types_in bindings);
