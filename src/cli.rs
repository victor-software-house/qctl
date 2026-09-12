use crate::schema::Kind;
use clap::{Args, Parser, Subcommand, ValueEnum};
use ctl_core::{ColorLong, FormatLong};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    about = "Control in-repo YAML work queues",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub format: FormatLong,

    #[command(flatten)]
    pub color: ColorLong,
}

#[derive(Subcommand)]
pub enum Command {
    /// Write a tasks.yaml for a prefix. Does not copy a schema file.
    Init(InitArgs),
    /// Print the active task and the priority-ordered queue.
    Status(LedgerArgs),
    /// Validate schema, graph, and git trailers that closed still-queued ids.
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
    /// Rewrite a ledger into the style it declares. Upgrades schema 3 to 4.
    Fmt(FmtArgs),
    /// Archive queued ids that commit-body trailers closed.
    CloseFromGit(CloseFromGitArgs),
    /// Install git hooks for this ledger.
    Hook(HookArgs),
    /// Write schema/tasks.schema.json from the ledger types.
    Schema(SchemaArgs),
    /// Print the installed-version operator contract.
    Instructions,
}

#[derive(Args)]
pub struct FmtArgs {
    #[command(flatten)]
    pub ledger: LedgerArgs,

    /// Say what is not in the declared style and exit non-zero, writing nothing.
    #[arg(short = 'c', long)]
    pub check: bool,
}

#[derive(Args)]
pub struct SchemaArgs {
    /// Where to write it. Defaults to schema/tasks.schema.json.
    #[arg(short = 'o', long, value_hint = clap::ValueHint::FilePath)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub struct CheckArgs {
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Skip the git trailer scan. Needed for a scratch ledger that is not in
    /// the current repository.
    #[arg(short = 'g', long)]
    pub no_git: bool,
}

#[derive(Args, Clone)]
pub struct CloseFromGitArgs {
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Read pre-push stdin (`<local-ref> <local-sha> <remote-ref> <remote-sha>`).
    #[arg(short = 'u', long)]
    pub pre_push: bool,
    /// `git log` revision range, such as `main..HEAD`. Default: the whole history, same as `check`.
    #[arg(short = 'r', long, conflicts_with = "pre_push")]
    pub range: Option<String>,
}

#[derive(Args)]
pub struct HookArgs {
    #[command(subcommand)]
    pub command: HookCommand,
}

#[derive(Subcommand)]
pub enum HookCommand {
    /// Write `.git/hooks/pre-push` so a push archives closed ids.
    Install(HookInstallArgs),
}

#[derive(Args)]
pub struct HookInstallArgs {
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Overwrite an existing git pre-push. Rejected when Lefthook is present
    /// and does not already run close-from-git.
    #[arg(short = 'F', long)]
    pub force: bool,
}

#[derive(Args, Clone)]
pub struct LedgerArgs {
    /// Ledger path. Defaults to `TASKS_LEDGER`, then `tasks.yaml`.
    #[arg(short = 'f', long, value_hint = clap::ValueHint::FilePath)]
    pub file: Option<PathBuf>,
}

#[derive(Args)]
pub struct InitArgs {
    /// Stable id prefix, such as PST, KAI, or OMX.
    #[arg(short = 'p', long)]
    pub prefix: String,
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Overwrite an existing ledger.
    #[arg(short = 'F', long)]
    pub force: bool,
}

#[derive(Args)]
pub struct AddArgs {
    #[command(flatten)]
    pub ledger: LedgerArgs,
    #[arg(short = 't', long)]
    pub title: String,
    #[arg(short = 's', long)]
    pub scope: String,
    #[arg(short = 'o', long)]
    pub outcome: String,
    /// Repeatable end condition that must be demonstrably true to close the row. Required on the queue; not used on the horizon.
    #[arg(short = 'a', long = "acceptance", required_unless_present = "horizon")]
    pub acceptance: Vec<String>,
    #[arg(short = 'P', long)]
    pub patch: Option<String>,
    /// Repeatable note item. Intentional line and paragraph breaks are preserved.
    #[arg(short = 'n', long = "note")]
    pub note: Vec<String>,
    /// Repeatable blocker id. Each must sit earlier than the new row.
    #[arg(short = 'b', long = "blocked-by")]
    pub blocked_by: Vec<String>,
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

#[derive(Args)]
pub struct IdArgs {
    pub id: String,
    #[command(flatten)]
    pub ledger: LedgerArgs,
}

#[derive(Args)]
pub struct ArchiveArgs {
    pub id: String,
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Repeatable evidence line.
    #[arg(short = 'e', long = "evidence", required = true)]
    pub evidence: Vec<String>,
    #[arg(short = 'd', long, value_enum, default_value = "completed")]
    pub disposition: Disposition,
}

#[derive(Args)]
pub struct ParkArgs {
    pub id: String,
    #[command(flatten)]
    pub ledger: LedgerArgs,
    #[arg(short = 'k', long, value_parser = parse_kind)]
    pub kind: Kind,
    #[arg(short = 'O', long)]
    pub open: String,
}

#[derive(Args)]
pub struct PromoteArgs {
    pub id: String,
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Repeatable end condition that must be demonstrably true to close the promoted row.
    #[arg(short = 'a', long = "acceptance", required = true)]
    pub acceptance: Vec<String>,
    /// Repeatable blocker id. Each must already sit on the queue.
    #[arg(short = 'b', long = "blocked-by")]
    pub blocked_by: Vec<String>,
}

#[derive(Args)]
pub struct EditArgs {
    pub id: String,
    #[command(flatten)]
    pub ledger: LedgerArgs,
    #[arg(short = 't', long)]
    pub title: Option<String>,
    #[arg(short = 's', long)]
    pub scope: Option<String>,
    #[arg(short = 'o', long)]
    pub outcome: Option<String>,
    #[arg(short = 'k', long, value_parser = parse_kind)]
    pub kind: Option<Kind>,
    #[arg(short = 'O', long)]
    pub open: Option<String>,
    #[arg(short = 'P', long)]
    pub patch: Option<String>,
    #[arg(short = 'l', long)]
    pub plan: Option<String>,
    /// Drop `patch` or `plan`.
    #[arg(short = 'U', long, value_enum)]
    pub unset: Vec<UnsetField>,
    /// Repeatable end condition that must be demonstrably true to close the row.
    #[arg(short = 'a', long = "acceptance")]
    pub acceptance: Vec<String>,
    /// Repeatable note item to append. Intentional line and paragraph breaks are preserved.
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

#[derive(Clone)]
pub enum EditPosition {
    Front,
    Back,
    Before(String),
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

#[derive(Clone)]
pub struct RemoveSpec {
    pub field: RemoveField,
    pub selector: String,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum RemoveField {
    Note,
    Acceptance,
    Link,
    BlockedBy,
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

#[derive(Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum UnsetField {
    Patch,
    Plan,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Disposition {
    Completed,
    Dropped,
}

impl Disposition {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Dropped => "dropped",
        }
    }
}
