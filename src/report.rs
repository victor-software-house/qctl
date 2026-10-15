//! What each verb returns. A report is data: `presentation` renders it as
//! pretty text, and `--format json` serializes it as it stands.

use serde::Serialize;

use crate::schema::{ArchivedTask, HorizonTask, Ledger, QueuedTask};

/// The result of one verb, tagged by `type` in JSON.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Report {
    /// `status`: the whole ledger.
    Status {
        /// The ledger file.
        path: String,
        /// Its contents.
        ledger: Ledger,
    },
    /// `check`: every problem found; none means the ledger passes.
    Check {
        /// The ledger file.
        path: String,
        /// One line per problem.
        problems: Vec<String>,
    },
    /// `show`: one row and the list it is on.
    Show {
        /// The ledger file.
        path: String,
        /// The row.
        task: Task,
    },
    /// `init`: a new ledger was written.
    Initialized {
        /// The ledger file.
        path: String,
        /// Its id prefix.
        prefix: String,
    },
    /// `add`: a row was written under a new id.
    Added {
        /// The ledger file.
        path: String,
        /// The new id.
        id: String,
        /// The list it went on.
        destination: Destination,
    },
    /// `start`: a row became active.
    Started {
        /// The ledger file.
        path: String,
        /// The row's id.
        id: String,
    },
    /// `archive`: a row left the queue.
    Archived {
        /// The ledger file.
        path: String,
        /// The row's id.
        id: String,
    },
    /// `park`: a row moved to the horizon.
    Parked {
        /// The ledger file.
        path: String,
        /// The row's id.
        id: String,
    },
    /// `edit`: a row was rewritten.
    Edited {
        /// The ledger file.
        path: String,
        /// The row's id.
        id: String,
    },
    /// `promote`: a horizon row moved to the queue.
    Promoted {
        /// The ledger file.
        path: String,
        /// The row's id.
        id: String,
    },
    /// `close-from-git`: the ids commit trailers closed were archived.
    ClosedFromGit {
        /// The ledger file.
        path: String,
        /// The ids archived, in trailer order.
        archived: Vec<String>,
        /// Whether the pre-push hook must stop the push, so the edited ledger
        /// can be committed first.
        retry_push: bool,
    },
    /// `fmt`: whether the ledger is in its declared style.
    Formatted {
        /// The ledger file.
        path: String,
        /// Whether the file differed from its declared style.
        changed: bool,
        /// Whether this was `--check`, which writes nothing.
        check: bool,
        /// The lines that differ, by number.
        differences: Vec<String>,
    },
    /// `hook install`: what was installed, or what to add by hand.
    HookInstalled {
        /// The hook or Lefthook file.
        path: String,
        /// What happened.
        outcome: HookOutcome,
        /// The Lefthook command to add, when one is needed.
        snippet: Option<String>,
    },
    /// `schema`: the JSON Schema was written.
    SchemaWritten {
        /// The schema file.
        path: String,
    },
    /// `instructions`: the operator contract.
    Instructions {
        /// The contract, as Markdown.
        markdown: String,
    },
}

/// The list `add` wrote a row on.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Destination {
    /// The queue.
    Queue,
    /// The horizon.
    Horizon,
}

/// One row, tagged by `state` in JSON with the list it is on.
#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Task {
    /// A queue row.
    Queued {
        /// The row.
        task: QueuedTask,
    },
    /// An archive row.
    Archived {
        /// The row.
        task: ArchivedTask,
    },
    /// A horizon row.
    Horizon {
        /// The row.
        task: HorizonTask,
    },
}

/// What `hook install` did.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HookOutcome {
    /// Lefthook already runs `close-from-git`.
    AlreadyConfigured,
    /// Lefthook is present; its `pre-push` needs the printed command.
    NeedsConfiguration,
    /// `--force` was given, but it overwrites only a git hook, and this
    /// repository uses Lefthook.
    ForceNotApplicable,
    /// The git `pre-push` hook was written.
    Installed,
}
