//! The `qctl` command: it parses the arguments, runs one verb from the
//! library, and prints its report. ctl-core's `App` owns parsing, help, and
//! output; `qctl instructions` prints the operator contract built into the
//! binary.

use anyhow::Result;
use ctl_core::prelude::{App, ExitCode};
use qctl::cli::{Cli, Command, HookCommand};
use qctl::report::Report;
use qctl::{check, edit, format, hooks, ledger, mutate, schema};

const INSTRUCTIONS: &str = include_str!("instructions.md");

#[cfg(test)]
mod operator_docs;

fn main() -> ExitCode {
    App::<Cli>::new("qctl")
        .mounted_as("q")
        .view(|cli| cli.format.view(cli.color.color()))
        .run(execute)
}

fn execute(cli: Cli) -> Result<Report> {
    match cli.command {
        Command::Init(args) => mutate::init(&args),
        Command::Status(args) => ledger::status(&args),
        Command::Check(args) => check::run(&args),
        Command::Add(args) => mutate::add(&args),
        Command::Start(args) => mutate::start(&args),
        Command::Archive(args) => mutate::archive(&args),
        Command::Park(args) => mutate::park(&args),
        Command::Promote(args) => mutate::promote(&args),
        Command::Edit(args) => edit::run(&args),
        Command::Show(args) => ledger::show(&args),
        Command::Fmt(args) => format::run(&args),
        Command::CloseFromGit(args) => mutate::close_from_git(&args),
        Command::CloseFromOpenspec(args) => mutate::close_from_openspec(&args),
        Command::Hook(args) => match args.command {
            HookCommand::Install(args) => hooks::install(&args),
        },
        Command::Schema(args) => schema::write(&args),
        Command::Instructions => Ok(Report::Instructions {
            markdown: INSTRUCTIONS.to_owned(),
        }),
    }
}
