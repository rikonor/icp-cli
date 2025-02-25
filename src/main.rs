use std::{
    env::args_os,
    ffi::OsString,
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Error};
use clap::{value_parser, Arg, Command};
use dashmap::DashMap;
use manifest::{Load as _, LoadError, Manifest, ManifestHandle};
use my_namespace::my_package::host::{self, Host};

use wasmtime::{
    component::{bindgen, Component, Linker},
    Config, Engine, Store,
};

mod extension;
use extension::{AddExtension, ExtensionAdder, ExtensionRemover, RemoveExtension};

mod spec;
use spec::CommandSpec;

mod manifest;

bindgen!({
    path: "wit",
    world: "extension",
    async: true,
});

const MANIFEST_PATH_DEFAULT: &str = "~/.smt/manifest.json";
const ARG_MANIFEST_SHORT: char = 'm';
const ARG_MANIFEST_LONG: &str = "manifest";

const EXTENSIONS_DIR_DEFAULT: &str = "~/.smt/extensions";
const ARG_EXTENSIONS_LONG: &str = "extensions-dir";

const PRECOMPILES_DIR_DEFAULT: &str = "~/.smt/precompiles";
const ARG_PRECOMPILES_LONG: &str = "precompiles-dir";

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

    // Manifest (handle)
    let mh = ManifestHandle(mpath.to_owned());

    // Setup
    let c = c
        .disable_help_subcommand(true)
        .disable_version_flag(true)
        .arg_required_else_help(true);

    // Arg (extensions-dir)
    let c = c.arg(
        Arg::new("extensions-dir")
            .long(ARG_EXTENSIONS_LONG)
            .default_value(EXTENSIONS_DIR_DEFAULT)
            .value_parser(value_parser!(PathBuf)),
    );

    // Arg (precompiles-dir)
    let c = c.arg(
        Arg::new("precompiles-dir")
            .long(ARG_PRECOMPILES_LONG)
            .default_value(PRECOMPILES_DIR_DEFAULT)
            .value_parser(value_parser!(PathBuf)),
    );

    // Extension
    let c = c.subcommand(
        Command::new("extension")
            .subcommand(
                Command::new("add")
                    .arg(Arg::new("name").long("name"))
                    .arg(Arg::new("uri")),
            )
            .subcommand(Command::new("rm").arg(Arg::new("name").required(true))),
    );

    // Manifest (load)
    let m = mh.load().await.or_else(|err| match err {
        // TODO(or.ricon): Prompt the user to create the manifest if it doesn't exist
        LoadError::NotFound(_) => Ok(Manifest::default()),

        //
        _ => Err(err),
    })?;

    // Components (initialize)
    let cmpnts: DashMap<String, Component> =
        m.xs.iter()
            .cloned()
            .map(|x| {
                let c = Component::from_file(
                    &ngn,    // engine
                    &x.wasm, // path
                )?;

                Ok((x.name, c))
            })
            .collect::<Result<_, Error>>()?;

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

    let _extdir = ms
        .get_one::<PathBuf>("extensions-dir")
        .context("missing extensions directory")?;

    let _predir = ms
        .get_one::<PathBuf>("precompiles-dir")
        .context("missing precompiles directory")?;

    // Extension (Adder)
    let add = ExtensionAdder::new(
        ngn.clone(), // engine
        mh.clone(),  // mh
    );

    // Extension (Remover)
    let rm = ExtensionRemover::new(mh);

    match ms.subcommand() {
        Some(("extension", ms)) => {
            match ms.subcommand() {
                Some(("add", ms)) => {
                    add.add(
                        ms.try_get_one::<String>("name")?.expect("missing name"), // name
                        ms.try_get_one::<String>("uri")?.expect("missing uri"),   // uri
                    )
                    .await
                    .context("failed to add extension")?;
                }

                Some(("rm", ms)) => {
                    rm.remove(
                        ms.try_get_one::<String>("name")?.expect("missing name"), // name
                    )
                    .await
                    .context("failed to remove extension")?;
                }

                _ => unreachable!(),
            }
        }

        Some((cmd, _ms)) => {
            // call extension's `run` export with `_ms`
            print!("{cmd}");
        }

        _ => println!("none"),
    }

    Ok(())
}
