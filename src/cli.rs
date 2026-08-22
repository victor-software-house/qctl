use crate::schema::Kind;
use clap::{Args, Parser, Subcommand, ValueEnum};
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
    /// Write a horizon row.
    Park(ParkArgs),
    /// Move a horizon row onto the queue.
    Promote(PromoteArgs),
    /// Print one queued, archived, or horizon task.
    Show(IdArgs),
    /// Rewrite a ledger into the style it declares.
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
    #[arg(long)]
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
    /// Skip the git trailer scan.
    #[arg(long)]
    pub no_git: bool,
}

#[derive(Args, Clone)]
pub struct CloseFromGitArgs {
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Read pre-push stdin (`<local-ref> <local-sha> <remote-ref> <remote-sha>`).
    #[arg(long)]
    pub pre_push: bool,
    /// `git log` revision range, such as `main..HEAD`. Default: the whole history, same as `check`.
    #[arg(long, conflicts_with = "pre_push")]
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
    /// Overwrite an existing git pre-push. Does nothing when lefthook.yml exists.
    #[arg(long)]
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
    #[arg(long)]
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
    /// Repeatable acceptance line. Required on the queue; not used on the horizon.
    #[arg(short = 'a', long = "acceptance", required_unless_present = "horizon")]
    pub acceptance: Vec<String>,
    #[arg(long)]
    pub patch: Option<String>,
    #[arg(long)]
    pub notes: Option<String>,
    /// Repeatable blocker id. Each must sit earlier than the new row.
    #[arg(long = "blocked-by")]
    pub blocked_by: Vec<String>,
    #[arg(long)]
    pub plan: Option<String>,
    /// Repeatable URI.
    #[arg(long = "link")]
    pub links: Vec<String>,
    /// Place the new row immediately after this queued id.
    #[arg(long, conflicts_with_all = ["before", "horizon"])]
    pub after: Option<String>,
    /// Place the new row immediately before this queued id.
    #[arg(long, conflicts_with_all = ["after", "horizon"])]
    pub before: Option<String>,
    /// Write a horizon row instead of a queue row.
    #[arg(long, requires_all = ["kind", "open"])]
    pub horizon: bool,
    /// Why it is on the horizon. Required with --horizon.
    #[arg(long, value_parser = parse_kind)]
    pub kind: Option<Kind>,
    /// The missing start condition. Required with --horizon.
    #[arg(long)]
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
    #[arg(long, value_enum, default_value = "completed")]
    pub disposition: Disposition,
}

#[derive(Args)]
pub struct ParkArgs {
    #[command(flatten)]
    pub ledger: LedgerArgs,
    #[arg(short = 't', long)]
    pub title: String,
    #[arg(short = 's', long)]
    pub scope: String,
    #[arg(short = 'o', long)]
    pub outcome: String,
    #[arg(long, value_parser = parse_kind)]
    pub kind: Kind,
    #[arg(long)]
    pub open: String,
    #[arg(long)]
    pub notes: Option<String>,
    #[arg(long)]
    pub plan: Option<String>,
    #[arg(long = "link")]
    pub links: Vec<String>,
    #[arg(long)]
    pub patch: Option<String>,
}

#[derive(Args)]
pub struct PromoteArgs {
    pub id: String,
    #[command(flatten)]
    pub ledger: LedgerArgs,
    /// Repeatable acceptance line the queue row must have.
    #[arg(short = 'a', long = "acceptance", required = true)]
    pub acceptance: Vec<String>,
    /// Repeatable blocker id. Each must already sit on the queue.
    #[arg(long = "blocked-by")]
    pub blocked_by: Vec<String>,
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
