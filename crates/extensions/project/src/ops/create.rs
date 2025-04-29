#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl From<CreateError> for String {
    fn from(e: CreateError) -> Self {
        match e {
            CreateError::Unexpected(err) => {
                format!("An unexpected error occurred: {}", err)
            }
        }
    }
}

impl From<CreateError> for u8 {
    fn from(e: CreateError) -> Self {
        match e {
            CreateError::Unexpected(_) => 2,
        }
    }
}

pub trait Create {
    fn create(&self) -> Result<(), CreateError>;
}

pub struct Creator {
    _read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>,
}

impl Creator {
    pub fn new(read_file: Box<dyn Fn(&str) -> Result<Vec<u8>, String>>) -> Self {
        Creator {
            _read_file: read_file,
        }
    }
}

impl Create for Creator {
    fn create(&self) -> Result<(), CreateError> {
        unimplemented!()
    }
}
