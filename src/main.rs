use std::{
    env::args_os,
    ffi::OsString,
    fs::read,
    io::ErrorKind,
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Context, Error};
use clap::{value_parser, Arg, Command};
use dashmap::DashMap;
use my_namespace::my_package::host::{self, Host};
use serde_json::from_slice;

use wasmtime::{
    component::{bindgen, Component, Linker},
    Config, Engine, Store,
};

mod extension;
use extension::Manifest;

mod spec;
use spec::CommandSpec;

bindgen!({
    path: "wit",
    world: "extension",
    async: true,
});

const MANIFEST_PATH_DEFAULT: &str = "~/.smt/manifest.json";

const ARG_MANIFEST_SHORT: char = 'm';
const ARG_MANIFEST_LONG: &str = "manifest";

struct State;

impl Host for State {
    async fn print(&mut self, s: String) {
        println!("{s}");
    }

    async fn time(&mut self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("failed to get current time")
            .as_millis() as u64
    }

    async fn rand(&mut self) -> u8 {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // WASM Configuration
    let mut cfg = Config::new();
    let cfg = cfg.async_support(true);

    // Engine
    let ngn = Engine::new(cfg)?;

    // Linker
    let mut lnk = Linker::new(&ngn);

    // Link host imports
    host::add_to_linker(
        &mut lnk,                  // linker
        |state: &mut State| state, // get
    )?;

    // Store
    let mut store = Store::new(
        &ngn,  // engine
        State, // data
    );

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

    // Components (initialize)
    let cmpnts: DashMap<String, Component> =
        m.xs.iter()
            .cloned()
            .map(|x| {
                let c = Component::from_file(
                    &ngn,    // engine
                    &x.path, // path
                )?;

                Ok::<_, Error>((x.name, c))
            })
            .collect::<Result<_, _>>()?;

    // Components (instantiate)
    let insts: DashMap<String, Extension> = DashMap::new();

    for p in &cmpnts {
        let (name, cmpnt) = p.pair();

        let inst = Extension::instantiate_async(
            &mut store, // store
            cmpnt,      // component
            &lnk,       // linker
        )
        .await?;

        insts.insert(
            name.to_owned(), // key
            inst,            // value
        );
    }

    // Extensions (hydrate)
    let mut c = c;

    for p in &insts {
        let (_, inst) = p.pair();

        // Call spec for CommandSpec
        let cspec = inst
            .my_namespace_my_package_cli()
            .call_spec(&mut store)
            .await
            .context("failed to retrieve spec")?;

        c = c.subcommand({
            let cspec: CommandSpec = serde_json::from_str(&cspec)?;
            cspec
        });
    }

    // Subcommand
    let ms = c.get_matches();

    if let Some((cmd, _ms)) = ms.subcommand() {
        println!("{cmd:?}");
    }

    Ok(())
}
