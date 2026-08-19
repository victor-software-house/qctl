//! Work-queue policy for in-repo `tasks.yaml` files.

#![allow(clippy::missing_errors_doc)]

pub mod check;
pub mod cli;
pub mod document;
pub mod format;
pub mod ledger;
pub mod mutate;
pub mod schema;
pub mod trailers;
