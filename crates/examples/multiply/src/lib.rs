use clap::Command;

#[allow(warnings)]
mod bindings;

use bindings::{
    exports::icp::{cli::cli, cli_multiply::lib},
    icp::cli::misc::print,
};

mod spec;
use spec::CommandSpec;

struct Component;

const CLI_SPEC: &str = r#"{
    "name": "multiply",
    "help": "Multiply numbers",
    "args": [{ "name": "a", "required": true }, { "name": "b", "required": true }],
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
            .expect("missing argument 'a'")
            .expect("missing argument 'a'")
            .parse()
            .expect("invalid argument 'a'");

        let b: u32 = m
            .try_get_one::<String>("b")
            .expect("missing argument 'b'")
            .expect("missing argument 'b'")
            .parse()
            .expect("invalid argument 'b'");

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
    print(&format!("[multiply] computing {a} x {b}"));
    let mut out = 0;
    for _ in 0..a {
        out += b;
    }

    print(&format!("[multiply] {a} x {b} = {out}"));
    out
}

bindings::export!(Component with_types_in bindings);
