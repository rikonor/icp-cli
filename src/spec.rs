use clap::{Arg, Command};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CommandSpec {
    /// Name of the command
    pub name: String,

    /// Help text for the command
    pub help: Option<String>,

    /// Version of the command
    pub version: Option<String>,

    /// Argument specification
    #[serde(default)]
    pub args: Vec<ArgSpec>,

    /// Subcommands specification
    #[serde(default)]
    pub subcommands: Vec<CommandSpec>,
}

impl From<CommandSpec> for Command {
    fn from(value: CommandSpec) -> Self {
        // Command
        let mut c = Command::new(value.name);

        // Help
        if let Some(help) = value.help {
            c = c.about(help);
        }

        // Version
        if let Some(ver) = value.version {
            c = c.version(ver);
        }

        // Args
        let c = value.args.into_iter().fold(c, |acc, cur| acc.arg(cur));

        // Subcommands
        let c = value
            .subcommands
            .into_iter()
            .fold(c, |acc, cur| acc.subcommand(cur));

        c
    }
}

#[derive(Debug, Deserialize)]
pub struct ArgSpec {
    /// Name of the argument
    pub name: String,

    /// Help text for the argument
    pub help: Option<String>,

    /// Short name for the argument
    pub short: Option<char>,

    /// Long name for the argument
    pub long: Option<String>,
}

impl From<ArgSpec> for Arg {
    fn from(value: ArgSpec) -> Self {
        // Arg
        let mut c = Arg::new(value.name);

        // Help
        if let Some(help) = value.help {
            c = c.help(help);
        }

        // Short
        if let Some(short) = value.short {
            c = c.short(short);
        }

        // Long
        if let Some(long) = value.long {
            c = c.long(long);
        }

        c
    }
}
