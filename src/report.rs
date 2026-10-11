use serde::Serialize;

use crate::schema::{ArchivedTask, HorizonTask, Ledger, QueuedTask};

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Report {
    Status {
        path: String,
        ledger: Ledger,
    },
    Check {
        path: String,
        problems: Vec<String>,
    },
    Show {
        path: String,
        task: Task,
    },
    Initialized {
        path: String,
        prefix: String,
    },
    Added {
        path: String,
        id: String,
        destination: Destination,
    },
    Started {
        path: String,
        id: String,
    },
    Archived {
        path: String,
        id: String,
    },
    Parked {
        path: String,
        id: String,
    },
    Edited {
        path: String,
        id: String,
    },
    Promoted {
        path: String,
        id: String,
    },
    ClosedFromGit {
        path: String,
        archived: Vec<String>,
        retry_push: bool,
    },
    Formatted {
        path: String,
        changed: bool,
        check: bool,
        differences: Vec<String>,
    },
    HookInstalled {
        path: String,
        outcome: HookOutcome,
        snippet: Option<String>,
    },
    SchemaWritten {
        path: String,
    },
    Instructions {
        markdown: String,
    },
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Destination {
    Queue,
    Horizon,
}

#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Task {
    Queued { task: QueuedTask },
    Archived { task: ArchivedTask },
    Horizon { task: HorizonTask },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HookOutcome {
    AlreadyConfigured,
    NeedsConfiguration,
    ForceNotApplicable,
    Installed,
}
