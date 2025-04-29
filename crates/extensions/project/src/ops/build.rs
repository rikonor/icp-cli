use crate::{List, LocalRef};

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<BuildError> for String {
    fn from(e: BuildError) -> Self {
        match e {
            BuildError::Unexpected(err) => {
                format!("An unexpected error occurred: {}", err)
            }
        }
    }
}

impl From<BuildError> for u8 {
    fn from(e: BuildError) -> Self {
        match e {
            BuildError::Unexpected(_) => 2,
        }
    }
}

pub trait Build {
    fn build(&self) -> Result<(), BuildError>;
}

pub struct Builder {
    _list_canisters: LocalRef<Box<dyn List>>,
    _build_canister: Box<dyn Fn(&str) -> Result<(), String>>,
}

impl Builder {
    pub fn new(
        list_canisters: LocalRef<Box<dyn List>>,
        build_canister: Box<dyn Fn(&str) -> Result<(), String>>,
    ) -> Self {
        Builder {
            _list_canisters: list_canisters,
            _build_canister: build_canister,
        }
    }
}

impl Build for Builder {
    fn build(&self) -> Result<(), BuildError> {
        unimplemented!()
    }
}

/*
use bindings::icp::build::lib::build_canister;

let cs = match LISTER.with(|v| v.get().expect("lister not initialized").list()) {
    Ok(cs) => cs,
    Err(err) => return err.into(),
};

if cs.is_empty() {
    print("No canisters found in the project.");
    return 0;
}

// TODO: concurrency?
for c in cs {
    match build_canister(&c.path) {
        Ok(_) => todo!(),
        Err(_) => todo!(),
    }
}
*/
