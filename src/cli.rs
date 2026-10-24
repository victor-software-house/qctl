//! The command-line grammar: one struct of arguments per verb. A flag's doc
//! comment is its `--help` text.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use ctl_core::{ColorLong, EnvPrefix, FormatLong};

use crate::schema::Kind;

/// The whole command line. Every long option also reads `QCTL_<LONG>`, such
/// as `QCTL_FILE` or `QCTL_FORMAT`; the command line wins over the variable.
#[derive(Debug, Parser)]
#[command(
    version,
    about = "Control in-repo YAML work queues",
    arg_required_else_help = true,
    env_prefix = "QCTL"
)]
pub struct Cli {
    /// The verb to run.
    #[command(subcommand)]
    pub command: Command,

    /// `--format`, shared by every verb.
    #[command(flatten)]
    pub format: FormatLong,

    /// `--color` and `--no-color`, shared by every verb.
    #[command(flatten)]
    pub color: ColorLong,
}

/// The verbs.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Write a tasks.yaml for a prefix. Does not copy a schema file.
    Init(InitArgs),
    /// Print the active task and the priority-ordered queue.
    Status(LedgerArgs),
    /// Validate schema, graph, git trailers, and agreement with `openspec`
    /// changes.
    Check(CheckArgs),
    /// Add a task to the queue, or to the horizon.
    Add(Box<AddArgs>),
    /// Make one unblocked queued task active (moves it to queue head).
    Start(IdArgs),
    /// Move a queued task to the archive.
    Archive(ArchiveArgs),
    /// Demote a queued task onto the horizon.
    Park(ParkArgs),
    /// Move a horizon row onto the queue.
    Promote(PromoteArgs),
    /// Rewrite fields or queue position of an existing row.
    Edit(Box<EditArgs>),
    /// Print one queued, archived, or horizon task.
    Show(IdArgs),
    /// Rewrite a ledger into the style it declares. Upgrades schema 3 and 4 to
    /// 5.
    Fmt(FmtArgs),
    /// Archive queued ids that commit-body trailers closed.
    CloseFromGit(CloseFromGitArgs),
    /// Archive queued rows whose `openspec` change was archived, and repoint
    /// plans.
    CloseFromOpenspec(LedgerArgs),
    /// Install git hooks for this ledger.
    Hook(HookArgs),
    /// Write schema/tasks.schema.json from the ledger types.
    Schema(SchemaArgs),
    /// Print the installed-version operator contract.
    Instructions,
}

/// Arguments of `qctl fmt`.
#[derive(Args, Debug)]
pub struct FmtArgs {
    /// The ledger to format.
    #[command(flatten)]
    pub ledger: LedgerArgs,

    /// Say what is not in the declared style and exit non-zero, writing
    /// nothing.
    #[arg(short = 'c', long)]
    pub check: bool,
}

/// Arguments of `qctl schema`.
#[derive(Args, Debug)]
pub struct SchemaArgs {
    /// Where to write it. Defaults to schema/tasks.schema.json.
    #[arg(short = 'o', long, value_hint = clap::ValueHint::FilePath)]
    pub out: Option<PathBuf>,
}

/// Arguments of `qctl check`.
#[derive(Args, Clone, Debug)]
pub struct CheckArgs {
    /// The ledger to check.
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Skip the git trailer scan. Needed for a scratch ledger that is not in
    /// the current repository.
    #[arg(short = 'g', long)]
    pub no_git: bool,
}

/// Arguments of `qctl close-from-git`.
#[derive(Args, Clone, Debug)]
pub struct CloseFromGitArgs {
    /// The ledger to archive closed ids in.
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Read pre-push stdin (`<local-ref> <local-sha> <remote-ref>
    /// <remote-sha>`).
    #[arg(short = 'u', long)]
    pub pre_push: bool,
    /// `git log` revision range, such as `main..HEAD`. Default: the whole
    /// history, same as `check`.
    #[arg(short = 'r', long, conflicts_with = "pre_push")]
    pub range: Option<String>,
}

/// Arguments of `qctl hook`.
#[derive(Args, Debug)]
pub struct HookArgs {
    /// The hook verb to run.
    #[command(subcommand)]
    pub command: HookCommand,
}

/// The `qctl hook` verbs.
#[derive(Debug, Subcommand)]
pub enum HookCommand {
    /// Write `.git/hooks/pre-push` so a push archives closed ids.
    Install(HookInstallArgs),
}

/// Arguments of `qctl hook install`.
#[derive(Args, Debug)]
pub struct HookInstallArgs {
    /// The ledger the hook archives closed ids in.
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Overwrite an existing git pre-push. Rejected when Lefthook is present
    /// and does not already run close-from-git.
    #[arg(short = 'F', long)]
    pub force: bool,
}

/// Which ledger file a verb reads.
#[derive(Args, Clone, Debug)]
pub struct LedgerArgs {
    /// Ledger path. Defaults to `tasks.yaml`.
    #[arg(short = 'f', long, value_hint = clap::ValueHint::FilePath)]
    pub file: Option<PathBuf>,
}

/// Arguments of `qctl init`.
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Stable id prefix, such as QCTL, VER, or FRK.
    #[arg(short = 'p', long)]
    pub prefix: String,
    /// Where to write the ledger.
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Overwrite an existing ledger.
    #[arg(short = 'F', long)]
    pub force: bool,
}

/// Arguments of `qctl add`.
#[derive(Args, Debug)]
pub struct AddArgs {
    /// The ledger to add to.
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// One-line name of the task.
    #[arg(short = 't', long)]
    pub title: String,
    /// The part of the repository the task touches.
    #[arg(short = 's', long)]
    pub scope: String,
    /// What is true once the task is done.
    #[arg(short = 'o', long)]
    pub outcome: String,
    /// Repeatable end condition that must be demonstrably true to close the
    /// row. Required on the queue; not used on the horizon.
    #[arg(short = 'a', long = "acceptance", required_unless_present = "horizon")]
    pub acceptance: Vec<String>,
    /// The forkctl patch the task edits.
    #[arg(short = 'P', long)]
    pub patch: Option<String>,
    /// Repeatable note item, one line each unless the ledger's
    /// `style.list_items` is `auto`.
    #[arg(short = 'n', long = "note")]
    pub note: Vec<String>,
    /// Repeatable blocker id. Each must sit earlier than the new row.
    #[arg(short = 'b', long = "blocked-by")]
    pub blocked_by: Vec<String>,
    /// A plan file next to the ledger, such as a change proposal.
    #[arg(short = 'l', long)]
    pub plan: Option<String>,
    /// Repeatable URI.
    #[arg(short = 'L', long = "link")]
    pub links: Vec<String>,
    /// Place the new row immediately after this queued id.
    #[arg(short = 'A', long, conflicts_with_all = ["before", "horizon"])]
    pub after: Option<String>,
    /// Place the new row immediately before this queued id.
    #[arg(short = 'B', long, conflicts_with_all = ["after", "horizon"])]
    pub before: Option<String>,
    /// Write a horizon row instead of a queue row.
    #[arg(short = 'H', long, requires_all = ["kind", "open"])]
    pub horizon: bool,
    /// Why it is on the horizon. Required with --horizon.
    #[arg(short = 'k', long, value_parser = parse_kind)]
    pub kind: Option<Kind>,
    /// The missing start condition. Required with --horizon.
    #[arg(short = 'O', long)]
    pub open: Option<String>,
}

fn parse_kind(raw: &str) -> Result<Kind, String> {
    match raw {
        "research" => Ok(Kind::Research),
        "evaluation" => Ok(Kind::Evaluation),
        "deferred" => Ok(Kind::Deferred),
        other => Err(format!(
            "kind must be research, evaluation, or deferred (got {other})"
        )),
    }
}

/// Arguments of a verb that names one row: `start` and `show`.
#[derive(Args, Debug)]
pub struct IdArgs {
    /// The task id, such as QCTL-001.
    pub id: String,
    /// The ledger that holds the row.
    #[command(flatten)]
    pub ledger: LedgerArgs,
}

/// Arguments of `qctl archive`.
#[derive(Args, Debug)]
pub struct ArchiveArgs {
    /// The queued task id.
    pub id: String,
    /// The ledger that holds the row.
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Repeatable evidence line.
    #[arg(short = 'e', long = "evidence", required = true)]
    pub evidence: Vec<String>,
    /// Whether the work was done or dropped.
    #[arg(short = 'd', long, value_enum, default_value = "completed")]
    pub disposition: Disposition,
    /// Why the `openspec` change the row's plan names stays open after the row
    /// is archived. Needed when no other queued or horizon row names that
    /// change.
    #[arg(short = 'K', long, value_name = "REASON")]
    pub keep_change_open: Option<String>,
}

/// Arguments of `qctl park`.
#[derive(Args, Debug)]
pub struct ParkArgs {
    /// The queued task id.
    pub id: String,
    /// The ledger that holds the row.
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Why it goes on the horizon: research, evaluation, or deferred.
    #[arg(short = 'k', long, value_parser = parse_kind)]
    pub kind: Kind,
    /// The missing start condition.
    #[arg(short = 'O', long)]
    pub open: String,
}

/// Arguments of `qctl promote`.
#[derive(Args, Debug)]
pub struct PromoteArgs {
    /// The horizon task id.
    pub id: String,
    /// The ledger that holds the row.
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Repeatable end condition that must be demonstrably true to close the
    /// promoted row.
    #[arg(short = 'a', long = "acceptance", required = true)]
    pub acceptance: Vec<String>,
    /// Repeatable blocker id. Each must already sit on the queue.
    #[arg(short = 'b', long = "blocked-by")]
    pub blocked_by: Vec<String>,
}

/// Arguments of `qctl edit`.
#[derive(Args, Debug)]
pub struct EditArgs {
    /// The task id, on any list.
    pub id: String,
    /// The ledger that holds the row.
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// New title.
    #[arg(short = 't', long)]
    pub title: Option<String>,
    /// New scope.
    #[arg(short = 's', long)]
    pub scope: Option<String>,
    /// New outcome.
    #[arg(short = 'o', long)]
    pub outcome: Option<String>,
    /// New horizon kind: research, evaluation, or deferred.
    #[arg(short = 'k', long, value_parser = parse_kind)]
    pub kind: Option<Kind>,
    /// New missing start condition of a horizon row.
    #[arg(short = 'O', long)]
    pub open: Option<String>,
    /// New forkctl patch name.
    #[arg(short = 'P', long)]
    pub patch: Option<String>,
    /// New plan file next to the ledger.
    #[arg(short = 'l', long)]
    pub plan: Option<String>,
    /// Drop `patch` or `plan`.
    #[arg(short = 'U', long, value_enum)]
    pub unset: Vec<UnsetField>,
    /// Repeatable end condition that must be demonstrably true to close the
    /// row.
    #[arg(short = 'a', long = "acceptance")]
    pub acceptance: Vec<String>,
    /// Repeatable note item to append, one line each unless the ledger's
    /// `style.list_items` is `auto`.
    #[arg(short = 'n', long = "note")]
    pub note: Vec<String>,
    /// Repeatable URI to append.
    #[arg(short = 'L', long = "link")]
    pub links: Vec<String>,
    /// Repeatable blocker id to append.
    #[arg(short = 'b', long = "blocked-by")]
    pub blocked_by: Vec<String>,
    /// Repeatable evidence line to append.
    #[arg(short = 'e', long = "evidence")]
    pub evidence: Vec<String>,
    /// Remove one list item as `field:index` or `field:exact text`.
    #[arg(short = 'x', long = "remove", value_parser = parse_remove)]
    pub remove: Vec<RemoveSpec>,
    /// Permutation of 1-based note indexes, such as `3,1,2`.
    #[arg(short = 'R', long = "reorder-notes")]
    pub reorder_notes: Option<String>,
    /// Move this queued row to `front`, `back`, `before:ID`, or `after:ID`.
    #[arg(short = 'p', long = "position", value_parser = parse_position)]
    pub position: Option<EditPosition>,
}

/// Where `--position` moves a queued row.
#[derive(Clone, Debug)]
pub enum EditPosition {
    /// The head of the queue.
    Front,
    /// The tail of the queue.
    Back,
    /// Just before this queued id.
    Before(String),
    /// Just after this queued id.
    After(String),
}

fn parse_position(raw: &str) -> Result<EditPosition, String> {
    match raw {
        "front" => Ok(EditPosition::Front),
        "back" => Ok(EditPosition::Back),
        _ => {
            let (relation, id) = raw
                .split_once(':')
                .ok_or_else(|| "position must be front, back, before:ID, or after:ID".to_owned())?;
            if id.is_empty() {
                return Err("position needs an id after ':'".to_owned());
            }
            match relation {
                "before" => Ok(EditPosition::Before(id.to_owned())),
                "after" => Ok(EditPosition::After(id.to_owned())),
                _ => Err("position must be front, back, before:ID, or after:ID".to_owned()),
            }
        }
    }
}

/// One `--remove` value: a list, and the item to drop from it.
#[derive(Clone, Debug)]
pub struct RemoveSpec {
    /// The list.
    pub field: RemoveField,
    /// A 1-based index, or the item's exact text.
    pub selector: String,
}

/// A list `--remove` can drop an item from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RemoveField {
    /// `notes`.
    Note,
    /// `acceptance`.
    Acceptance,
    /// `links`.
    Link,
    /// `blocked_by`.
    BlockedBy,
    /// `evidence`.
    Evidence,
}

fn parse_remove(raw: &str) -> Result<RemoveSpec, String> {
    let (field, selector) = raw
        .split_once(':')
        .ok_or_else(|| "remove must be field:index or field:exact text".to_owned())?;
    if selector.is_empty() {
        return Err("remove needs an index or exact text after ':'".to_owned());
    }
    let field = match field {
        "note" => RemoveField::Note,
        "acceptance" => RemoveField::Acceptance,
        "link" => RemoveField::Link,
        "blocked-by" => RemoveField::BlockedBy,
        "evidence" => RemoveField::Evidence,
        _ => {
            return Err(
                "remove field must be note, acceptance, link, blocked-by, or evidence".to_owned(),
            );
        }
    };
    Ok(RemoveSpec {
        field,
        selector: selector.to_owned(),
    })
}

/// A field `--unset` can drop.
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
pub enum UnsetField {
    /// The forkctl patch name.
    Patch,
    /// The plan file.
    Plan,
}

/// How a row left the queue.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Disposition {
    /// The work is done.
    Completed,
    /// The work will not be done.
    Dropped,
}

impl Disposition {
    /// The value as the ledger writes it.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Dropped => "dropped",
        }
    }
}
