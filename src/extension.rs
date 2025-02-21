use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Extension {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Manifest {
    #[serde(rename = "extensions")]
    pub xs: Vec<Extension>,
}
