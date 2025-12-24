use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config{
    pub(crate) diffchecker: Vec<DiffChecker>
}

#[derive(Debug, Deserialize)]
pub struct DiffChecker {
    pub(crate) name: String,
    pub(crate) preprod: String,
    pub(crate) prod: String,
    pub(crate) repo: String,
}