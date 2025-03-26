use clap::Command;

#[allow(warnings)]
mod bindings;

// imports
use bindings::local::host::misc::print;
use bindings::local::multiply::lib::multiply;

// exports
use bindings::exports::local::extension::cli;
use bindings::exports::local::square::lib;

mod spec;
use spec::CommandSpec;

struct Component;

const CLI_SPEC: &str = r#"{
    "name": "square",
    "help": "Take a square (a^2)",
    "args": [{ "name": "a", "required": true }],
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

        // let b: u32 = m
        //     .try_get_one::<String>("b")
        //     .expect("missing argument 'b'")
        //     .expect("missing argument 'b'")
        //     .parse()
        //     .expect("invalid argument 'b'");

        // TMP(orricon): for demo
        let b: u32 = 2;

        let _out = square(a, b);

        0
    }
}

impl lib::Guest for Component {
    fn square(a: u32, b: u32) -> u32 {
        square(a, b)
    }
}

fn square(a: u32, b: u32) -> u32 {
    print(&format!("[square] computing {a}**{b}"));
    let mut out = a;
    for _ in 1..b {
        out = multiply(out, a)
    }

    print(&format!("[square] {a}**{b} = {out}"));
    out
}

bindings::export!(Component with_types_in bindings);
