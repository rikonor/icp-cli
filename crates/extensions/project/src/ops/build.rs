use crate::{List, ListError, LocalRef, bindings::icp::cli::misc::print};

#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("Failed to list canisters before build: {0}")]
    ListFailed(#[from] ListError),

    #[error("One or more canisters failed to build.")]
    BuildFailed,

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<BuildError> for String {
    fn from(e: BuildError) -> Self {
        match e {
            BuildError::ListFailed(e) => e.to_string(),
            BuildError::BuildFailed => e.to_string(),
            BuildError::Unexpected(err) => {
                format!("An unexpected error occurred: {}", err)
            }
        }
    }
}

impl From<BuildError> for u8 {
    fn from(e: BuildError) -> Self {
        match e {
            BuildError::ListFailed(e) => e.into(),
            BuildError::BuildFailed => 4,
            BuildError::Unexpected(_) => 2,
        }
    }
}

pub trait Build {
    fn build(&self) -> Result<(), BuildError>;
}

pub struct Builder {
    list_canisters: LocalRef<Box<dyn List>>,
    build_canister: Box<dyn Fn(&str) -> Result<String, String>>,
}

impl Builder {
    pub fn new(
        list_canisters: LocalRef<Box<dyn List>>,
        build_canister: Box<dyn Fn(&str) -> Result<String, String>>,
    ) -> Self {
        Builder {
            list_canisters,
            build_canister,
        }
    }
}

impl Build for Builder {
    fn build(&self) -> Result<(), BuildError> {
        print("Listing canisters in project...");

        // List canisters
        let canisters = match self.list_canisters.with(|l| l.get().unwrap().list()) {
            Ok(cs) => cs,
            Err(ListError::NoCanistersFound) => {
                print("No canisters found in the project. Nothing to build.");
                return Ok(());
            }
            Err(e) => {
                return Err(BuildError::ListFailed(e));
            }
        };

        print(&format!("Found {} canisters to build.", canisters.len()));

        let mut build_failed = false;

        // Iterate and build each canister
        for canister in canisters {
            print(&format!(
                "Attempting to build canister '{}' [{}] at path '{}'...",
                canister.name, canister.canister_type, canister.path
            ));

            match (self.build_canister)(&canister.path) {
                Ok(_) => {
                    print(&format!("Successfully built canister '{}'.", canister.name));
                }
                Err(msg) => {
                    print(&format!(
                        "ERROR building canister '{}': {}",
                        canister.name, msg
                    ));
                    build_failed = true;
                }
            }
        }

        // Check overall result
        if build_failed {
            Err(BuildError::BuildFailed)
        } else {
            print("All canisters built successfully.");
            Ok(())
        }
    }
}
