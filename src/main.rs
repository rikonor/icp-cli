use std::{env::args_os, ffi::OsString, fs::read, io::ErrorKind, path::PathBuf, str::FromStr};

use anyhow::{anyhow, Context, Error};
use clap::{value_parser, Arg, Command};
use extension::Manifest;
use serde_json::from_slice;

mod extension;
mod spec;

const MANIFEST_PATH_DEFAULT: &str = "~/.smt/manifest.json";

const ARG_MANIFEST_SHORT: char = 'm';
const ARG_MANIFEST_LONG: &str = "manifest";

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Command
    let c = Command::new("smt");

    // Version
    let c = c.version("1.0.0");

    // Arg (manifest)
    let c = c.arg(
        Arg::new("manifest")
            .short(ARG_MANIFEST_SHORT)
            .long(ARG_MANIFEST_LONG)
            .default_value(MANIFEST_PATH_DEFAULT)
            .value_parser(value_parser!(PathBuf)),
    );

    // Load Manifest
    let args = args_os().collect::<Vec<_>>();

    let mpath = args.windows(2).find(|&p| {
        [
            format!("-{ARG_MANIFEST_SHORT}"),
            format!("--{ARG_MANIFEST_LONG}"),
        ]
        .iter()
        .any(|f| *f.as_str() == p[0])
    });

    let mut itr = vec![
        OsString::from_str(c.get_name())?, // bin
    ];

    if let Some(mpath) = mpath {
        itr.append(&mut mpath.to_vec());
    }

    let ms = c.clone().get_matches_from(itr);

    let mpath = ms
        .get_one::<PathBuf>("manifest")
        .context("missing manifest path")?;

    let m = match read(mpath) {
        // Ok
        Ok(bs) => from_slice(&bs).context("failed to parse manifest"),

        // Err
        Err(err) => match err.kind() {
            // NotFound -> default
            // TODO(or): Ask for confirmation from the user first or accept a -y / --yes option
            ErrorKind::NotFound => Ok(Manifest::default()),

            // _
            err => Err(anyhow!("failed to read manifest: {err}")),
        },
    }?;

    // Setup
    let c = c
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg_required_else_help(true);

    // Extension
    let c = c.subcommand(
        Command::new("extension")
            .subcommand(Command::new("add"))
            .subcommand(Command::new("remove"))
            .subcommand(Command::new("invoke")),
    );

    // Extensions
    let c = m.xs.iter().try_fold(c, |c, cur| {
        // c.subcommand(subcmd)

        Ok::<_, Error>(c)
    })?;

    // Subcommand
    let ms = c.get_matches();

    if let Some((cmd, ms)) = ms.subcommand() {
        println!("{cmd:?}");
    }

    Ok(())
}
