use std::{
    env::args_os,
    ffi::OsString,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Error};
use clap::{value_parser, Arg, ArgAction, Command};
use wasmtime::{
    component::{bindgen, Linker},
    Config, Engine, Store,
};

// Re-exports from our own crates
use dfx_core::interface::{DetectIfaces, IfaceDetector};

// WIT Bindings
use local::host::misc::{self, Host};

bindgen!({
    path: "wit",
    world: "extension",
    async: true,
});

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
    // Create an engine with component support
    let mut cfg = Config::new();
    cfg.wasm_component_model(true).async_support(true);
    let engine = Engine::new(&cfg)?;

    println!("Engine initialized with component model support");
    Ok(())
}
