use std::{error::Error, fmt};

const INGEST_BATCH_SIZE: usize = 100_000;

#[derive(Debug)]
struct DBError {
    details: String,
}
impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for DBError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl DBError {
    pub fn new(details: &str) -> Self {
        Self {
            details: details.to_string(),
        }
    }
}

pub mod client;

pub use client::*;
mod crew;
mod episodes;
pub mod ingest;
pub mod movie;
mod names;
mod principals;
pub mod titles;
