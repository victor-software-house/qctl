//! Work-queue policy for in-repo `tasks.yaml` files.

#![allow(
    clippy::missing_errors_doc,
    reason = "errors are anyhow chains shown to the operator, not a typed contract to document per function"
)]

pub mod check;
pub mod cli;
pub mod document;
pub mod edit;
pub mod format;
pub mod hooks;
pub mod ledger;
pub mod mutate;
pub mod presentation;
pub mod report;
pub mod schema;
pub mod trailers;
