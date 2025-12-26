use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct Config{
    pub diffchecker: Vec<DiffChecker>
}

#[derive(Debug, Deserialize)]
pub(crate) struct DiffChecker {
    pub name: String,
    pub preprod: String,
    pub prod: String,
    pub repo: String,
}