use std::{
    collections::HashMap,
    env::args_os,
    ffi::OsString,
    fs::{create_dir_all, read},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, bail, Context, Error};
use clap::{value_parser, Arg, ArgAction, Command};
use dashmap::DashMap;
use once_cell::sync::Lazy;

use serde::Deserialize;
use wasmtime::{
    component::{bindgen, Component, Linker, Val as WasmVal},
    Config, Engine, Store as WasmStore,
};

use icp_core::{
    component::{DynamicLinker, FunctionRegistry},
    dependency::DependencyGraph,
    interface::IfaceDetector,
    manifest::{Load, LoadError, Manifest, ManifestHandle, Store as _},
    FunctionRegistryError, Interface,
};
use icp_distribution::Distribution;

mod extension;
use extension::{
    AddExtension, ExtensionAdder, ExtensionLister, ExtensionRemover, ListExtensions,
    RemoveExtension,
};

mod spec;
use spec::CommandSpec;

// Service configuration
const SERVICE_NAME: &str = "icp";
const ARG_SHORT_MANIFEST: char = 'm';
const ARG_LONG_MANIFEST: &str = "manifest";
const ARG_LONG_EXTENSIONS: &str = "extensions-dir";
const ARG_LONG_PRECOMPILES: &str = "precompiles-dir";

// Distribution configuration
static DISTRIBUTION: Lazy<Distribution> = Lazy::new(|| {
    match option_env!("DISTRIBUTION")
        .map(Distribution::try_from)
        .transpose()
    {
        Ok(Some(distribution)) => distribution,
        Err(e) => {
            eprintln!("⚠️ Warning: {}. Falling back to Standard.", e);
            Distribution::Standard
        }
        _ => Distribution::Standard,
    }
});

// Default paths
static DEFAULT_PATH_MANIFEST: Lazy<PathBuf> = Lazy::new(|| match *DISTRIBUTION {
    Distribution::Standard => dirs::home_dir()
        .expect("no home dir found")
        .join(format!(".{SERVICE_NAME}/manifest.json")),
    Distribution::Homebrew => {
        let output = std::process::Command::new("brew")
            .arg("--prefix")
            .output()
            .expect("failed to execute brew --prefix");
        let prefix = String::from_utf8_lossy(&output.stdout).trim().to_string();
        PathBuf::from(prefix).join("var/icp/manifest.json")
    }
    Distribution::NuGet => unimplemented!("nuget paths not yet implemented"),
    Distribution::Apt => PathBuf::from("/var/lib/icp/manifest.json"),
});

static DEFAULT_DIR_EXTENSIONS: Lazy<PathBuf> = Lazy::new(|| match *DISTRIBUTION {
    Distribution::Standard => dirs::cache_dir()
        .expect("no cache dir found")
        .join(format!("{SERVICE_NAME}/extensions-dir")),
    Distribution::Homebrew => DEFAULT_PATH_MANIFEST.parent().unwrap().join("extensions"),
    Distribution::NuGet => unimplemented!("nuget paths not yet implemented"),
    Distribution::Apt => PathBuf::from("/var/lib/icp/extensions"),
});

static DEFAULT_DIR_PRECOMPILES: Lazy<PathBuf> = Lazy::new(|| match *DISTRIBUTION {
    Distribution::Standard => dirs::cache_dir()
        .expect("no cache dir found")
        .join(format!("{SERVICE_NAME}/precompiles-dir")),
    Distribution::Homebrew => DEFAULT_PATH_MANIFEST.parent().unwrap().join("precompiles"),
    Distribution::NuGet => unimplemented!("nuget paths not yet implemented"),
    Distribution::Apt => PathBuf::from("/var/lib/icp/precompiles"),
});

// WIT Bindings
use icp::cli::{filesystem, misc};

bindgen!({
    path: "../../wit/cli",
    world: "extension",
    async: true,
});

struct State;

impl misc::Host for State {
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

// Implementation for the custom filesystem interface
impl filesystem::Host for State {
    async fn create_directory(&mut self, path: String) -> Result<(), String> {
        // Note: Paths are relative to the workspace root where icp-cli runs.
        // TODO: Add interception/administration logic here if desired.
        match std::fs::create_dir_all(&path) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to create directory '{}': {}", path, e)),
        }
    }

    async fn write_file(&mut self, path: String, contents: Vec<u8>) -> Result<(), String> {
        // Note: Paths are relative to the workspace root where icp-cli runs.
        // TODO: Add interception/administration logic here if desired.
        match std::fs::write(&path, &contents) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to write file '{}': {}", path, e)),
        }
    }

    async fn read_file(&mut self, path: String) -> Result<Vec<u8>, String> {
        // Note: Paths are relative to the workspace root where icp-cli runs.
        // TODO: Add interception/administration logic here if desired.
        match std::fs::read(&path) {
            Ok(contents) => Ok(contents),
            Err(e) => Err(format!("Failed to read file '{}': {}", path, e)),
        }
    }
}

// Directory setup helper
fn _ensure_directories(
    manifest: &Path,
    extensions: &Path,
    precompiles: &Path,
) -> Result<(), Error> {
    if let Some(parent) = manifest.parent() {
        create_dir_all(parent).context("failed to create manifest directory")?;
    }
    create_dir_all(extensions).context("failed to create extensions directory")?;
    create_dir_all(precompiles).context("failed to create precompiles directory")?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
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
                    .arg(Arg::new("uri").help("Local path or Uri").required(true))
                    .arg(
                        Arg::new("checksum")
                            .long("checksum")
                            .value_name("SHA256")
                            .help("Expected SHA256 checksum for verification"),
                    )
                    .arg(
                        Arg::new("force")
                            .long("force")
                            .action(ArgAction::SetTrue)
                            .help("Overwrite existing extension"),
                    ),
            )
            .subcommand(
                Command::new("rm")
                    .alias("remove")
                    .arg(Arg::new("keep").short('k').action(ArgAction::SetTrue))
                    .arg(Arg::new("name").required(true)),
            )
            .subcommand(
                Command::new("deps")
                    .about("Show extension dependencies")
                    .arg(Arg::new("name").help("Extension name").required(false))
                    .arg(
                        Arg::new("validate")
                            .long("validate")
                            .action(ArgAction::SetTrue)
                            .help("Validate dependencies"),
                    ),
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

    // Create dependency graph and resolve loading order
    let dependency_graph = DependencyGraph::new(&m).context("failed to create dependency graph")?;

    // Check for circular dependencies
    if dependency_graph.has_cycles() {
        eprintln!("Warning: Circular dependencies detected in extensions:");
        eprintln!("{}", dependency_graph.format_cycles());
        eprintln!("Some extensions may not function correctly.");
    }

    // Validate dependencies
    if let Err(err) = dependency_graph.validate_dependencies(&m) {
        eprintln!("Warning: Dependency validation failed: {}", err);
        eprintln!("Some extensions may not function correctly.");
    }

    // Resolve loading order
    let loading_order = dependency_graph
        .resolve_loading_order()
        .context("failed to resolve extension loading order")?;

    // WASM Configuration
    let mut cfg = Config::new();
    let cfg = cfg.async_support(true);

    // Engine
    let ngn = Engine::new(cfg)?;

    // Linker
    let mut lnk = Linker::new(&ngn);

    // Components (initialize)
    let cmpnts: DashMap<String, Component> = DashMap::new();

    // Load components in dependency order
    for name in &loading_order {
        if let Some(extension) = m.xs.iter().find(|x| &x.name == name) {
            let pre = read(&extension.pre)?;

            let component = unsafe {
                Component::deserialize(
                    &ngn, // engine
                    &pre, // bytes
                )
            }?;

            cmpnts.insert(name.clone(), component);
        }
    }

    // Create function registry
    let reg = FunctionRegistry::new();
    let reg = Arc::new(Mutex::new(reg));

    // Create dynamic linker
    let mut dynlnk = DynamicLinker::new(Arc::clone(&reg));

    // Collect unique interfaces
    let mut ifaces: HashMap<String, Interface> = HashMap::new();
    for name in &loading_order {
        if let Some(extension) = m.xs.iter().find(|x| &x.name == name) {
            for iface in extension.imports.iter().chain(extension.exports.iter()) {
                ifaces
                    .entry(iface.name.clone())
                    .or_insert_with(|| iface.clone());
            }
        }
    }

    // Link all unique interfaces at once
    dynlnk.link(
        &mut lnk,                       // linker
        ifaces.into_values().collect(), // interfaces
    )?;

    // Host imports
    misc::add_to_linker(
        &mut lnk,                  // linker
        |state: &mut State| state, // get
    )?;

    filesystem::add_to_linker(
        // Link filesystem host functions
        &mut lnk,                  // linker
        |state: &mut State| state, // get
    )?;

    // NOTE: Well this is gonna be annoying - are we going to have to keep this version up to date??
    lnk.instance("icp:cli/component@0.3.3")?.func_new_async(
        "invoke",
        move |mut store, params: &[WasmVal], results: &mut [WasmVal]| {
            Box::new({
                let reg = Arc::clone(&reg);
                async move {
                    const INTERFACE_NAME_IDX: usize = 0;
                    const FUNCTION_NAME_IDX: usize = 1;
                    const NESTED_PARAMS_IDX: usize = 2;

                    // Extract interface name
                    let interface_name = match params.get(INTERFACE_NAME_IDX) {
                        Some(v) => v,
                        None => bail!("missing interface_name parameter for invoke host function"),
                    };

                    let interface_name = match interface_name {
                        WasmVal::String(s) => s,
                        _ => bail!("interface_name has the wrong type: {interface_name:?}"),
                    };

                    // Extract function name
                    let function_name = match params.get(FUNCTION_NAME_IDX) {
                        Some(v) => v,
                        None => bail!("missing function_name parameter for invoke host function"),
                    };

                    let function_name = match function_name {
                        WasmVal::String(s) => s,
                        _ => bail!("function_name has the wrong type: {function_name:?}"),
                    };

                    // Lookup function using function registry
                    let f = match reg.lock().unwrap().lookup(interface_name, function_name) {
                        // Found function
                        Ok(Some(f)) => Ok(f),

                        // Register, but not resolved
                        Ok(None) => Err(format!(
                            "function {}:{} is registered by not yet resolved",
                            interface_name, function_name
                        )),

                        // Not found
                        Err(FunctionRegistryError::NotFound(key)) => Err(format!(
                            "function {}:{} (key: {}) not found in registry",
                            interface_name, function_name, key
                        )),

                        // Other errors
                        Err(err) => Err(format!(
                            "Error during function lookup for {}:{}: {}",
                            interface_name, function_name, err,
                        )),
                    };

                    // Extract nested params
                    let nparams = match params.get(NESTED_PARAMS_IDX) {
                        Some(v) => v,
                        None => bail!("missing nested params for invoke host function"),
                    };

                    // Convert nparams to Vec<u8>
                    let nparams = match nparams {
                        WasmVal::List(vs) => {
                            let mut bytes = Vec::new();
                            for v in vs.iter() {
                                match v {
                                    WasmVal::String(s) => bytes.extend_from_slice(s.as_bytes()),
                                    _ => bail!("nested params has the wrong type: {nparams:?}"),
                                }
                            }
                            bytes
                        }
                        _ => bail!("nested params has the wrong type: {nparams:?}"),
                    };

                    // Deserialize params
                    let nparams: Vec<Val> = match serde_json::from_slice(&nparams) {
                        Ok(params) => params,
                        Err(err) => {
                            results[0] = WasmVal::Result(Err(Some(Box::new(WasmVal::String(
                                err.to_string(),
                            )))));
                            return Ok(());
                        }
                    };

                    let nparams = nparams
                        .into_iter()
                        .map(|v| v.into())
                        .collect::<Vec<WasmVal>>();

                    let f = match f {
                        Ok(f) => f,
                        Err(err) => {
                            results[0] =
                                WasmVal::Result(Err(Some(Box::new(WasmVal::String(err.into())))));
                            return Ok(());
                        }
                    };

                    // Nested results
                    let mut nresults = vec![WasmVal::Bool(false)];

                    // Invoke function
                    f.call_async(
                        &mut store,    // store
                        &nparams,      // params
                        &mut nresults, // results
                    )
                    .await
                    .context(anyhow!("failed to call {interface_name}:{function_name}"))?;

                    f.post_return_async(&mut store)
                        .await
                        .context(anyhow!("post return failed"))?;

                    // Set results from nested results
                    println!("nresults: {nresults:?}");

                    results[0] = WasmVal::Result(Ok(None));

                    Ok(())
                }
            })
        },
    )?;

    // Store
    let mut store = WasmStore::new(
        &ngn,  // engine
        State, // data
    );

    // Components (instantiate)
    let insts: DashMap<String, Extension> = DashMap::new();

    // Instantiate components in dependency order
    for name in &loading_order {
        let cmpnt = cmpnts
            .get(name)
            .ok_or_else(|| anyhow!("missing component"))?;

        // Component (generic)
        let inst = lnk
            .instantiate_async(
                &mut store,    // store
                cmpnt.value(), // component
            )
            .await?;

        // Resolve exports for this extension
        if let Some(x) = m.xs.iter().find(|x| &x.name == name) {
            dynlnk.resolve(
                &mut store, // store
                &x.name,    // extension
                &inst,      // instance
                &x.exports, // exports
            )?;
        }

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

    // Extensions (init)
    for name in &loading_order {
        let inst = insts
            .get(name)
            .context("extension not initialized properly")?;

        inst.icp_cli_init()
            .call_init(&mut store)
            .await?
            .map_err(|err| anyhow!("failed to initialize extension {name}: {err}"))?;
    }

    // Extensions (hydrate)
    let mut c = c;

    for p in &insts {
        let (name, inst) = p.pair();

        // Call spec for CommandSpec
        let cspec = inst
            .icp_cli_cli()
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

    // Create library interface detector
    let detector = Arc::new(IfaceDetector);

    // Extension (Adder)
    let add = ExtensionAdder::new(
        ngn.clone(),    // engine
        mh.clone(),     // mh
        extdir.clone(), // extensions_dir
        predir.clone(), // precompiles_dir
        detector,       // detector
    );

    // Extension (Remover)
    let rm = ExtensionRemover::new(mh);

    match ms.subcommand() {
        Some(("extension", ms)) => match ms.subcommand() {
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
                    ms.get_one::<String>("checksum").map(|s| s.as_str()),     // checksum
                    ms.get_flag("force"),                                     // force
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
        },

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
                    let _exit_code = inst //
                        .icp_cli_cli()
                        .call_run(&mut store, &args)
                        .await?;
                }
                None => unreachable!("invalid extension"),
            }
        }

        _ => unreachable!("invalid command"),
    }

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
pub enum Val {
    // Primitive types
    Bool(bool),

    // Signed integers
    S8(i8),
    S32(i32),
    S64(i64),
    S16(i16),

    // Unsigned integers
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    // Floating point numbers
    Float32(f32),
    Float64(f64),

    // Text
    Char(char),
    String(String),

    // Containers
    Enum(String),
    List(Vec<Val>),
    Option(Option<Box<Val>>),
    Record(Vec<(String, Val)>),
    Result(Result<Option<Box<Val>>, Option<Box<Val>>>),
    Tuple(Vec<Val>),
    Variant(String, Option<Box<Val>>),

    // Other
    Flags(Vec<String>),
    // TODO: Figure out how to represent this
    // Resource(ResourceAny),
}

impl From<Val> for WasmVal {
    fn from(value: Val) -> Self {
        todo!()
    }
}
