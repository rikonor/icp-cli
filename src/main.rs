use std::{
    env::args_os,
    ffi::OsString,
    path::PathBuf,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Error};
use clap::{value_parser, Arg, ArgAction, Command};
use dashmap::DashMap;
use manifest::{Load as _, LoadError, Manifest, ManifestHandle, Store as _};
use once_cell::sync::Lazy;
use wasmtime::{
    component::{bindgen, Component, Linker},
    Config, Engine, Store,
};

mod extension;
use extension::{
    AddExtension, ExtensionAdder, ExtensionLister, ExtensionRemover, ListExtensions,
    RemoveExtension,
};

mod spec;
use spec::CommandSpec;

mod manifest;

// WIT Bindings
use local::host::misc::{self, Host};

bindgen!({
    path: "wit",
    world: "extension",
    async: true,
});

const SERVICE_NAME: &str = "dfx-2";

static DEFAULT_PATH_MANIFEST: Lazy<PathBuf> = Lazy::new(|| {
    dirs::home_dir()
        .expect("no home dir found")
        .join(format!(".{SERVICE_NAME}/manifest.json"))
});

const ARG_SHORT_MANIFEST: char = 'm';
const ARG_LONG_MANIFEST: &str = "manifest";

static DEFAULT_DIR_EXTENSIONS: Lazy<PathBuf> = Lazy::new(|| {
    dirs::cache_dir()
        .expect("no cache dir found")
        .join(format!("{SERVICE_NAME}/extensions-dir"))
});

const ARG_LONG_EXTENSIONS: &str = "extensions-dir";

static DEFAULT_DIR_PRECOMPILES: Lazy<PathBuf> = Lazy::new(|| {
    dirs::cache_dir()
        .expect("no cache dir found")
        .join(format!("{SERVICE_NAME}/precompiles-dir"))
});

const ARG_LONG_PRECOMPILES: &str = "precompiles-dir";

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
    misc::add_to_linker(
        &mut lnk,                  // linker
        |state: &mut State| state, // get
    )?;

    // Store
    let mut store = Store::new(
        &ngn,  // engine
        State, // data
    );

    // Command
    let c = Command::new(SERVICE_NAME);

    // Version
    let c = c.version("1.0.0");

    // Arg (manifest)
    let c = c.arg(
        Arg::new("manifest")
            .short(ARG_SHORT_MANIFEST)
            .long(ARG_LONG_MANIFEST)
            .default_value(DEFAULT_PATH_MANIFEST.as_os_str())
            .value_parser(value_parser!(PathBuf)),
    );

    // Load Manifest
    let args = args_os().collect::<Vec<_>>();

    let mpath = args.windows(2).find(|&p| {
        [
            format!("-{ARG_SHORT_MANIFEST}"),
            format!("--{ARG_LONG_MANIFEST}"),
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
            .long(ARG_LONG_EXTENSIONS)
            .default_value(DEFAULT_DIR_EXTENSIONS.as_os_str())
            .value_parser(value_parser!(PathBuf)),
    );

    // Arg (precompiles-dir)
    let c = c.arg(
        Arg::new("precompiles-dir")
            .long(ARG_LONG_PRECOMPILES)
            .default_value(DEFAULT_DIR_PRECOMPILES.as_os_str())
            .value_parser(value_parser!(PathBuf)),
    );

    // Extension
    let c = c.subcommand(
        Command::new("extension")
            .about("manage extensions")
            .subcommand_required(true)
            .subcommand(Command::new("ls").alias("list"))
            .subcommand(
                Command::new("add")
                    .arg(Arg::new("name").long("name").required(true))
                    .arg(Arg::new("uri").help("Local path or Uri").required(true)),
            )
            .subcommand(
                Command::new("rm")
                    .alias("remove")
                    .arg(Arg::new("keep").short('k').action(ArgAction::SetTrue))
                    .arg(Arg::new("name").required(true)),
            ),
    );

    // Manifest (load)
    let m = mh.load().or_else(|err| match err {
        LoadError::NotFound(_) => {
            let m = Manifest::default();

            // TODO(or.ricon): Prompt the user to create the manifest if it doesn't exist
            mh.store(&m)
                .context("failed to store initial extensions manifest")?;

            Ok(m)
        }

        //
        _ => Err(err),
    })?;

    // Components (initialize)
    let cmpnts: DashMap<String, Component> =
        m.xs.iter()
            .cloned()
            .map(|x| {
                let c = unsafe {
                    Component::deserialize_file(
                        &ngn,   // engine
                        &x.pre, // path
                    )
                }?;

                Ok((x.name, c))
            })
            .collect::<Result<_, Error>>()?;

    // Components (instantiate)
    let insts: DashMap<String, Extension> = DashMap::new();

    for p in &cmpnts {
        let (name, cmpnt) = p.pair();

        // Component (generic)
        let inst = lnk
            .instantiate_async(
                &mut store, // store
                cmpnt,      // component
            )
            .await?;

        // Component (typed)
        let inst = Extension::new(
            &mut store, // store
            &inst,      // instance
        )?;

        insts.insert(
            name.to_owned(), // key
            inst,            // value
        );
    }

    // Extensions (hydrate)
    let mut c = c;

    for p in &insts {
        let (name, inst) = p.pair();

        // Call spec for CommandSpec
        let cspec = inst
            .local_extension_cli()
            .call_spec(&mut store)
            .await
            .context("failed to retrieve spec")?;

        c = c.subcommand({
            let cspec: CommandSpec = serde_json::from_str(&cspec)?;
            let c: Command = cspec.into();

            // Overwrite name
            c.name(name)
        });
    }

    // Subcommand
    let ms = c.get_matches();

    let extdir = ms
        .get_one::<PathBuf>("extensions-dir")
        .context("missing extensions directory")?;

    let predir = ms
        .get_one::<PathBuf>("precompiles-dir")
        .context("missing precompiles directory")?;

    // Extension (Lister)
    let ls = ExtensionLister::new(mh.clone());

    // Extension (Adder)
    let add = ExtensionAdder::new(
        ngn.clone(),    // engine
        mh.clone(),     // mh
        extdir.clone(), // extensions_dir
        predir.clone(), // precompiles_dir
    );

    // Extension (Remover)
    let rm = ExtensionRemover::new(mh);

    match ms.subcommand() {
        Some(("extension", ms)) => {
            match ms.subcommand() {
                Some(("ls", _)) => {
                    let names = ls
                        .list()
                        .await
                        .context("failed to list installed extensions")?;

                    if names.is_empty() {
                        println!("No extensions installed");
                    } else {
                        names.iter().for_each(|name| println!("{name}"));
                    }
                }

                Some(("add", ms)) => {
                    add.add(
                        ms.try_get_one::<String>("name")?.expect("missing name"), // name
                        ms.try_get_one::<String>("uri")?.expect("missing uri"),   // uri
                    )
                    .await
                    .context("failed to add extension")?;

                    println!("Extension added");
                }

                Some(("rm", ms)) => {
                    rm.remove(
                        ms.try_get_one::<String>("name")?.expect("missing name"), // name
                    )
                    .await
                    .context("failed to remove extension")?;

                    println!("Extension removed");
                }

                _ => unreachable!("invalid command"),
            }
        }

        Some((cmd, _)) => {
            // Trim arguments for extension
            let args: Vec<_> = args
                .iter()
                .skip(1)
                .map(|arg| {
                    arg.to_str()
                        .expect("invalid command-line argument")
                        .to_owned()
                })
                .collect();

            // Invoke extension
            match insts.get(cmd) {
                Some(inst) => {
                    let exit_code = inst
                        .local_extension_cli()
                        .call_run(&mut store, &args)
                        .await?;

                    println!("{exit_code}");
                }
                None => unreachable!("invalid extension"),
            }
        }

        _ => unreachable!("invalid command"),
    }

    Ok(())
}
