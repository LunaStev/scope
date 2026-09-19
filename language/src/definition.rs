use serde::Deserialize;

/// Data-only registration; a classifier never names an executable or library.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub id: String,
    pub name: String,
    pub classifier: String,
    #[serde(default)] pub extensions: Vec<String>,
    #[serde(default)] pub filenames: Vec<String>,
    #[serde(default)] pub interpreters: Vec<String>,
}
