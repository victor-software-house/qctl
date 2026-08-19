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
    /// Append a new unblocked task and print its id.
    Add(AddArgs),
    /// Make one unblocked queued task active (moves it to queue head).
    Start(IdArgs),
    /// Move a queued task to the archive.
    Archive(ArchiveArgs),
    /// Print one queued, archived, or horizon task.
    Show(IdArgs),
    /// Rewrite a ledger into the style it declares.
    Fmt(FmtArgs),
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
    /// Repeatable acceptance line.
    #[arg(short = 'a', long = "acceptance", required = true)]
    pub acceptance: Vec<String>,
    #[arg(long)]
    pub patch: Option<String>,
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
